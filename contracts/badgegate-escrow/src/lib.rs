#![no_std]

//! BadgeGate escrow contract (Concordium testnet, Rust, concordium-std).
//!
//! Hire gate escrow for AI agents. A job can only be created against a gate
//! receipt that a checker account recorded on chain first (the two step
//! attestation design, see RESEARCH.md section 6). The checker runs the off
//! chain verification (Agent Registry status, Verified by Concordium badge,
//! Agent Card SHA-256 hash against the on chain anchor) because a contract
//! cannot fetch a card or call the registry indexer. The contract stores
//! the receipt, binds it to the agent owner account, holds the client's CCD
//! in escrow, and settles only to the provider, or refunds only to the
//! client.
//!
//! OWNER POWERS, FULL DISCLOSURE:
//! The owner can ONLY pause and unpause NEW job creation. The owner cannot
//! withdraw, move, redirect, or otherwise touch escrowed funds. This
//! contract deliberately contains no withdrawal, sweep, or rescue function.
//! Escrowed CCD can only ever leave to (a) the job's provider, on release
//! or after the claim window, or (b) the job's client, on a legitimate
//! cancellation or refund. Pausing never blocks payouts, refunds,
//! deliveries, or claims on existing jobs. There is no fee, and no return
//! of any kind is promised to anyone.
//!
//! STATUS (24 Nov 2026): full implementation of the state machine below, ported to concordium-std 11 (concrete Host, Context, and Logger types, no generics).
//! Not deployed. Deployment waits for a testnet account (see the project
//! STATUS.md, Butuh Duan list).

use concordium_std::*;

pub const CLAIM_WINDOW_SECONDS: u64 = 3 * 24 * 60 * 60;
const CLAIM_WINDOW_MS: u64 = CLAIM_WINDOW_SECONDS * 1000;

/// A gate receipt stays fresh for 7 days after the check ran (AUDIT.md
/// finding F3, fixed 25 Nov 2026). createJob rejects an older receipt, so
/// a receipt recorded before an agent was revoked or re anchored cannot be
/// consumed forever.
const RECEIPT_MAX_AGE_MS: u64 = 7 * 24 * 60 * 60 * 1000;
/// A receipt may be dated slightly ahead of the block slot time, to allow
/// for checker clock skew. Anything further ahead is rejected.
const RECEIPT_FUTURE_TOLERANCE_MS: u64 = 5 * 60 * 1000;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serial, Deserial, SchemaType)]
pub enum JobState {
    Open,
    Accepted,
    Delivered,
    Released,
    Refunded,
    ClaimedAfterWindow,
}

impl JobState {
    /// A job is settled when its escrow has been paid out to the provider.
    pub fn is_settled(&self) -> bool {
        matches!(self, JobState::Released | JobState::ClaimedAfterWindow)
    }
}

/// The off chain badge verification result, recorded by a checker account.
#[derive(Debug, Clone, Serial, Deserial, SchemaType)]
pub struct GateReceipt {
    /// The Verified by Concordium badge identifier: the agent CIS-8004 token
    /// address in short Base58Check form or canonical CAIP-19 form (the token
    /// id inside it is a TokenIdU64). See RESEARCH.md section 11, item 2.
    pub badge_id: String,
    /// The Concordium account that owns the registered agent. A job created
    /// against this receipt must name this account as its provider.
    pub agent_owner: AccountAddress,
    /// SHA-256 of the Agent Card bytes, as anchored in the CIS-8004 registry.
    pub card_hash: [u8; 32],
    /// Unix time in milliseconds when the check ran.
    pub checked_at_ms: u64,
}

/// A recorded receipt plus its consumption flag. A receipt hires one job.
#[derive(Debug, Clone, Serial, Deserial, SchemaType)]
pub struct StoredReceipt {
    pub receipt: GateReceipt,
    pub used: bool,
}

#[derive(Debug, Clone, Serial, Deserial, SchemaType)]
pub struct Job {
    pub client: AccountAddress,
    pub provider: AccountAddress,
    pub amount: Amount,
    pub description_uri: String,
    pub deadline_ms: u64,
    pub delivered_at_ms: u64,
    pub deliverable_hash: [u8; 32],
    pub state: JobState,
    pub rating: u8,
    pub gate: GateReceipt,
}

#[derive(Debug, Default, Clone, Serial, Deserial, SchemaType)]
pub struct ProviderStats {
    pub jobs_settled: u64,
    pub volume_settled: Amount,
    pub rating_sum: u64,
    pub rating_count: u64,
}

#[derive(Serial, DeserialWithState)]
#[concordium(state_parameter = "S")]
pub struct State<S: HasStateApi = StateApi> {
    pub owner: AccountAddress,
    /// Accounts allowed to record gate receipts (the BadgeGate checkers).
    pub checkers: StateSet<AccountAddress, S>,
    pub job_creation_paused: bool,
    pub jobs: StateMap<u64, Job, S>,
    pub next_job_id: u64,
    pub receipts: StateMap<u64, StoredReceipt, S>,
    pub next_receipt_id: u64,
    pub stats: StateMap<AccountAddress, ProviderStats, S>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serial, Deserial, SchemaType)]
pub enum Event {
    GateRecorded { receipt_id: u64, badge_id: String, agent_owner: AccountAddress },
    GateAttested { job_id: u64, badge_id: String, agent_owner: AccountAddress },
    JobCreated {
        job_id: u64,
        client: AccountAddress,
        provider: AccountAddress,
        amount: Amount,
        deadline_ms: u64,
        description_uri: String,
    },
    JobAccepted { job_id: u64 },
    JobDelivered { job_id: u64, deliverable_hash: [u8; 32] },
    JobReleased { job_id: u64, amount: Amount },
    JobClaimedAfterWindow { job_id: u64, amount: Amount },
    JobRefunded { job_id: u64, amount: Amount },
    JobRated { job_id: u64, rating: u8 },
    JobCreationPauseSet { paused: bool },
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serial, Deserial, SchemaType, Reject)]
pub enum Error {
    /// Conversion for parameter parsing failures.
    #[from(ParseError)]
    Parse,
    /// Conversion for event logging failures.
    #[from(LogError)]
    Log,
    NotOwner,
    NotChecker,
    NotClient,
    NotProvider,
    Paused,
    ZeroAmount,
    JobNotFound,
    WrongState,
    DeadlinePassed,
    /// Raised in createJob when the requested deadline is not in the future.
    /// (Renamed from the misleading DeadlineInFuture in the phase 4 read
    /// through, 24 Nov 2026. The variant keeps its original position in the
    /// enum, so its serialised ordinal is unchanged.)
    DeadlineInPast,
    WindowNotElapsed,
    AlreadyRated,
    RatingOutOfRange,
    EmptyDeliverableHash,
    TransferFailed,
    ReceiptNotFound,
    ReceiptAlreadyUsed,
    ProviderMismatch,
    SelfHire,
    /// Appended at the end in the phase 4 read through (24 Nov 2026) so the
    /// serialised ordinals of every earlier variant are unchanged.
    EmptyBadgeId,
    EmptyCardHash,
    /// Appended at the end in the audit fix pass (25 Nov 2026), again so
    /// the serialised ordinals of every earlier variant are unchanged.
    /// deliverJob was called after the job deadline (AUDIT.md F2).
    DeliveryLate,
    /// The gate receipt is older than the freshness window (AUDIT.md F3).
    ReceiptStale,
    /// The gate receipt is dated ahead of slot time beyond the tolerance
    /// (AUDIT.md F3).
    ReceiptFutureDated,
}

type ContractResult<A> = Result<A, Error>;

// ---------------------------------------------------------------------------
// Parameters
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serial, Deserial, SchemaType)]
pub struct InitParams {
    pub checkers: Vec<AccountAddress>,
}

#[derive(Debug, Clone, Serial, Deserial, SchemaType)]
pub struct CreateJobParams {
    pub provider: AccountAddress,
    pub description_uri: String,
    pub deadline_ms: u64,
    /// Id of a gate receipt previously recorded by a checker (recordGate).
    pub receipt_id: u64,
}

#[derive(Debug, Clone, Serial, Deserial, SchemaType)]
pub struct DeliverParams {
    pub job_id: u64,
    pub deliverable_hash: [u8; 32],
}

#[derive(Debug, Clone, Serial, Deserial, SchemaType)]
pub struct RateParams {
    pub job_id: u64,
    pub rating: u8,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn invoker_account(ctx: &ReceiveContext) -> ContractResult<AccountAddress> {
    // In concordium-std 11 the receive invoker of a top level transaction is
    // always an account address.
    Ok(ctx.invoker())
}

fn now_ms(ctx: &ReceiveContext) -> u64 {
    ctx.metadata().slot_time().timestamp_millis()
}

/// Receipt freshness (AUDIT.md F3): a receipt must have been checked at
/// most RECEIPT_MAX_AGE_MS ago, and must not be dated ahead of slot time
/// by more than the clock skew tolerance. Enforced in both recordGate and
/// createJob, so a stale or future dated receipt can neither be recorded
/// nor consumed.
fn ensure_receipt_fresh(checked_at_ms: u64, now: u64) -> ContractResult<()> {
    ensure!(
        checked_at_ms <= now + RECEIPT_FUTURE_TOLERANCE_MS,
        Error::ReceiptFutureDated
    );
    ensure!(
        now.saturating_sub(checked_at_ms) <= RECEIPT_MAX_AGE_MS,
        Error::ReceiptStale
    );
    Ok(())
}

fn settle_stats<S: HasStateApi>(
    state: &mut State<S>,
    provider: AccountAddress,
    amount: Amount,
) {
    let mut stats = state.stats.entry(provider).or_insert_with(ProviderStats::default);
    stats.jobs_settled += 1;
    stats.volume_settled += amount;
}

// ---------------------------------------------------------------------------
// Init
// ---------------------------------------------------------------------------

#[init(
    contract = "badgegate_escrow",
    parameter = "InitParams",
    error = "Error",
    enable_logger
)]
fn contract_init(
    ctx: &InitContext,
    state_builder: &mut StateBuilder,
    _logger: &mut Logger,
) -> InitResult<State> {
    let params: InitParams = ctx.parameter_cursor().get()?;
    let mut checkers = state_builder.new_set();
    for checker in params.checkers {
        checkers.insert(checker);
    }
    Ok(State {
        owner: ctx.init_origin(),
        checkers,
        job_creation_paused: false,
        jobs: state_builder.new_map(),
        next_job_id: 0,
        receipts: state_builder.new_map(),
        next_receipt_id: 0,
        stats: state_builder.new_map(),
    })
}

// ---------------------------------------------------------------------------
// Gate attestation, step 1 of 2: a checker records a receipt on chain.
// ---------------------------------------------------------------------------

#[receive(
    contract = "badgegate_escrow",
    name = "recordGate",
    parameter = "GateReceipt",
    error = "Error",
    enable_logger,
    mutable
)]
fn contract_record_gate(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<u64> {
    let receipt: GateReceipt = ctx.parameter_cursor().get()?;
    let invoker = invoker_account(ctx)?;
    ensure!(!receipt.badge_id.is_empty(), Error::EmptyBadgeId);
    ensure!(receipt.card_hash != [0u8; 32], Error::EmptyCardHash);
    ensure_receipt_fresh(receipt.checked_at_ms, now_ms(ctx))?;
    let state = host.state_mut();
    ensure!(state.checkers.contains(&invoker), Error::NotChecker);
    let receipt_id = state.next_receipt_id;
    state.next_receipt_id += 1;
    state.receipts.insert(receipt_id, StoredReceipt { receipt: receipt.clone(), used: false });
    logger.log(&Event::GateRecorded {
        receipt_id,
        badge_id: receipt.badge_id,
        agent_owner: receipt.agent_owner,
    })?;
    Ok(receipt_id)
}

// ---------------------------------------------------------------------------
// Gate attestation, step 2 of 2, and job creation (payable escrow deposit).
// ---------------------------------------------------------------------------

#[receive(
    contract = "badgegate_escrow",
    name = "createJob",
    parameter = "CreateJobParams",
    error = "Error",
    enable_logger,
    mutable,
    payable
)]
fn contract_create_job(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    amount: Amount,
    logger: &mut Logger,
) -> ContractResult<u64> {
    let params: CreateJobParams = ctx.parameter_cursor().get()?;
    let client = invoker_account(ctx)?;
    ensure!(!host.state().job_creation_paused, Error::Paused);
    ensure!(amount > Amount::zero(), Error::ZeroAmount);
    ensure!(params.provider != client, Error::SelfHire);
    ensure!(params.deadline_ms > now_ms(ctx), Error::DeadlineInPast);

    let state = host.state_mut();
    let receipt = {
        let stored =
            state.receipts.get(&params.receipt_id).ok_or(Error::ReceiptNotFound)?;
        ensure!(!stored.used, Error::ReceiptAlreadyUsed);
        ensure!(
            stored.receipt.agent_owner == params.provider,
            Error::ProviderMismatch
        );
        ensure_receipt_fresh(stored.receipt.checked_at_ms, now_ms(ctx))?;
        stored.receipt.clone()
    };
    if let Some(mut stored) = state.receipts.get_mut(&params.receipt_id) {
        stored.used = true;
    }

    let job_id = state.next_job_id;
    state.next_job_id += 1;
    state.jobs.insert(
        job_id,
        Job {
            client,
            provider: params.provider,
            amount,
            description_uri: params.description_uri.clone(),
            deadline_ms: params.deadline_ms,
            delivered_at_ms: 0,
            deliverable_hash: [0u8; 32],
            state: JobState::Open,
            rating: 0,
            gate: receipt.clone(),
        },
    );
    logger.log(&Event::GateAttested {
        job_id,
        badge_id: receipt.badge_id,
        agent_owner: receipt.agent_owner,
    })?;
    logger.log(&Event::JobCreated {
        job_id,
        client,
        provider: params.provider,
        amount,
        deadline_ms: params.deadline_ms,
        description_uri: params.description_uri,
    })?;
    Ok(job_id)
}

// ---------------------------------------------------------------------------
// Lifecycle transitions
// ---------------------------------------------------------------------------

#[receive(
    contract = "badgegate_escrow",
    name = "acceptJob",
    parameter = "u64",
    error = "Error",
    enable_logger,
    mutable
)]
fn contract_accept_job(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    let job_id: u64 = ctx.parameter_cursor().get()?;
    let invoker = invoker_account(ctx)?;
    let state = host.state_mut();
    let mut job = state.jobs.get_mut(&job_id).ok_or(Error::JobNotFound)?;
    ensure!(job.provider == invoker, Error::NotProvider);
    ensure!(job.state == JobState::Open, Error::WrongState);
    ensure!(now_ms(ctx) <= job.deadline_ms, Error::DeadlinePassed);
    job.state = JobState::Accepted;
    logger.log(&Event::JobAccepted { job_id })?;
    Ok(())
}

#[receive(
    contract = "badgegate_escrow",
    name = "deliverJob",
    parameter = "DeliverParams",
    error = "Error",
    enable_logger,
    mutable
)]
fn contract_deliver_job(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    let params: DeliverParams = ctx.parameter_cursor().get()?;
    let invoker = invoker_account(ctx)?;
    ensure!(params.deliverable_hash != [0u8; 32], Error::EmptyDeliverableHash);
    let state = host.state_mut();
    let mut job = state.jobs.get_mut(&params.job_id).ok_or(Error::JobNotFound)?;
    ensure!(job.provider == invoker, Error::NotProvider);
    ensure!(job.state == JobState::Accepted, Error::WrongState);
    // AUDIT.md F2, fixed 25 Nov 2026: delivery after the deadline is
    // rejected, mirroring acceptJob. A late delivery must not close the
    // client's refund path (cancel after the deadline) or unlock the
    // claim window.
    ensure!(now_ms(ctx) <= job.deadline_ms, Error::DeliveryLate);
    job.deliverable_hash = params.deliverable_hash;
    job.delivered_at_ms = now_ms(ctx);
    job.state = JobState::Delivered;
    logger.log(&Event::JobDelivered {
        job_id: params.job_id,
        deliverable_hash: params.deliverable_hash,
    })?;
    Ok(())
}

#[receive(
    contract = "badgegate_escrow",
    name = "releaseJob",
    parameter = "u64",
    error = "Error",
    enable_logger,
    mutable
)]
fn contract_release_job(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    let job_id: u64 = ctx.parameter_cursor().get()?;
    let invoker = invoker_account(ctx)?;
    let (provider, amount) = {
        let state = host.state_mut();
        let mut job = state.jobs.get_mut(&job_id).ok_or(Error::JobNotFound)?;
        ensure!(job.client == invoker, Error::NotClient);
        ensure!(job.state == JobState::Delivered, Error::WrongState);
        job.state = JobState::Released;
        (job.provider, job.amount)
    };
    settle_stats(host.state_mut(), provider, amount);
    host.invoke_transfer(&provider, amount).map_err(|_| Error::TransferFailed)?;
    logger.log(&Event::JobReleased { job_id, amount })?;
    Ok(())
}

#[receive(
    contract = "badgegate_escrow",
    name = "claimAfterWindow",
    parameter = "u64",
    error = "Error",
    enable_logger,
    mutable
)]
fn contract_claim_after_window(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    let job_id: u64 = ctx.parameter_cursor().get()?;
    let invoker = invoker_account(ctx)?;
    let (provider, amount) = {
        let state = host.state_mut();
        let mut job = state.jobs.get_mut(&job_id).ok_or(Error::JobNotFound)?;
        ensure!(job.provider == invoker, Error::NotProvider);
        ensure!(job.state == JobState::Delivered, Error::WrongState);
        ensure!(
            now_ms(ctx) >= job.delivered_at_ms + CLAIM_WINDOW_MS,
            Error::WindowNotElapsed
        );
        job.state = JobState::ClaimedAfterWindow;
        (job.provider, job.amount)
    };
    settle_stats(host.state_mut(), provider, amount);
    host.invoke_transfer(&provider, amount).map_err(|_| Error::TransferFailed)?;
    logger.log(&Event::JobClaimedAfterWindow { job_id, amount })?;
    Ok(())
}

/// Cancel rules mirror AgentSeal: the client may cancel while the job is
/// still Open, or after the deadline has passed with no delivery. A
/// delivered job can never be cancelled, it can only be released or
/// claimed after the window.
#[receive(
    contract = "badgegate_escrow",
    name = "cancelJob",
    parameter = "u64",
    error = "Error",
    enable_logger,
    mutable
)]
fn contract_cancel_job(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    let job_id: u64 = ctx.parameter_cursor().get()?;
    let invoker = invoker_account(ctx)?;
    let (client, amount) = {
        let state = host.state_mut();
        let mut job = state.jobs.get_mut(&job_id).ok_or(Error::JobNotFound)?;
        ensure!(job.client == invoker, Error::NotClient);
        let cancellable = job.state == JobState::Open
            || (job.state == JobState::Accepted && now_ms(ctx) > job.deadline_ms);
        ensure!(cancellable, Error::WrongState);
        job.state = JobState::Refunded;
        (job.client, job.amount)
    };
    host.invoke_transfer(&client, amount).map_err(|_| Error::TransferFailed)?;
    logger.log(&Event::JobRefunded { job_id, amount })?;
    Ok(())
}

#[receive(
    contract = "badgegate_escrow",
    name = "rateJob",
    parameter = "RateParams",
    error = "Error",
    enable_logger,
    mutable
)]
fn contract_rate_job(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    let params: RateParams = ctx.parameter_cursor().get()?;
    let invoker = invoker_account(ctx)?;
    ensure!((1..=5).contains(&params.rating), Error::RatingOutOfRange);
    let provider = {
        let state = host.state_mut();
        let mut job = state.jobs.get_mut(&params.job_id).ok_or(Error::JobNotFound)?;
        ensure!(job.client == invoker, Error::NotClient);
        ensure!(job.state.is_settled(), Error::WrongState);
        ensure!(job.rating == 0, Error::AlreadyRated);
        job.rating = params.rating;
        job.provider
    };
    let state = host.state_mut();
    let mut stats = state.stats.entry(provider).or_insert_with(ProviderStats::default);
    stats.rating_sum += u64::from(params.rating);
    stats.rating_count += 1;
    logger.log(&Event::JobRated { job_id: params.job_id, rating: params.rating })?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Owner powers: pause new job creation only. Nothing else, by design.
// ---------------------------------------------------------------------------

#[receive(
    contract = "badgegate_escrow",
    name = "setJobCreationPaused",
    parameter = "bool",
    error = "Error",
    enable_logger,
    mutable
)]
fn contract_set_paused(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    let paused: bool = ctx.parameter_cursor().get()?;
    let invoker = invoker_account(ctx)?;
    let state = host.state_mut();
    ensure!(state.owner == invoker, Error::NotOwner);
    state.job_creation_paused = paused;
    logger.log(&Event::JobCreationPauseSet { paused })?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Views
// ---------------------------------------------------------------------------

#[receive(
    contract = "badgegate_escrow",
    name = "getJob",
    parameter = "u64",
    return_value = "Job",
    error = "Error"
)]
fn contract_get_job(ctx: &ReceiveContext, host: &Host<State>) -> ContractResult<Job> {
    let job_id: u64 = ctx.parameter_cursor().get()?;
    let job = host.state().jobs.get(&job_id).ok_or(Error::JobNotFound)?;
    Ok(job.clone())
}

#[receive(
    contract = "badgegate_escrow",
    name = "getProviderStats",
    parameter = "AccountAddress",
    return_value = "ProviderStats",
    error = "Error"
)]
fn contract_get_stats(
    ctx: &ReceiveContext,
    host: &Host<State>,
) -> ContractResult<ProviderStats> {
    let provider: AccountAddress = ctx.parameter_cursor().get()?;
    match host.state().stats.get(&provider) {
        Some(stats) => Ok(stats.clone()),
        None => Ok(ProviderStats::default()),
    }
}

#[receive(
    contract = "badgegate_escrow",
    name = "jobCount",
    return_value = "u64",
    error = "Error"
)]
fn contract_job_count(_ctx: &ReceiveContext, host: &Host<State>) -> ContractResult<u64> {
    Ok(host.state().next_job_id)
}

#[receive(
    contract = "badgegate_escrow",
    name = "isChecker",
    parameter = "AccountAddress",
    return_value = "bool",
    error = "Error"
)]
fn contract_is_checker(ctx: &ReceiveContext, host: &Host<State>) -> ContractResult<bool> {
    let account: AccountAddress = ctx.parameter_cursor().get()?;
    Ok(host.state().checkers.contains(&account))
}
