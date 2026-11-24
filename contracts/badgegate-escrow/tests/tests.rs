//! Integration tests for the BadgeGate escrow contract, run with the
//! concordium-smart-contract-testing library against the compiled Wasm
//! module (module.wasm.v1 in this package root, built with
//! `cargo concordium build -e -o module.wasm.v1`).
//!
//! Scenarios ported from TEST-CHECKLIST.md (29 items). The interface review
//! item (no withdrawal or sweep entrypoint) is not a runtime test; it is
//! recorded in the checklist itself.

use badgegate_escrow::{CreateJobParams, DeliverParams, GateReceipt, InitParams, Job, RateParams};
use concordium_smart_contract_testing::*;

const MODULE_PATH: &str = "module.wasm.v1";
const CONTRACT_NAME: &str = "badgegate_escrow";

const OWNER: AccountAddress = AccountAddress([1u8; 32]);
const CHECKER: AccountAddress = AccountAddress([2u8; 32]);
const CLIENT: AccountAddress = AccountAddress([3u8; 32]);
const PROVIDER: AccountAddress = AccountAddress([4u8; 32]);
const PROVIDER_B: AccountAddress = AccountAddress([5u8; 32]);
const STRANGER: AccountAddress = AccountAddress([6u8; 32]);

const START_BALANCE: u64 = 10_000; // CCD per account
const ESCROW: u64 = 100; // CCD per job
const DAY_MS: u64 = 24 * 60 * 60 * 1000;
const DEADLINE_MS: u64 = 10 * DAY_MS;

fn card_hash() -> [u8; 32] {
    [7u8; 32]
}

fn receipt_for(owner: AccountAddress) -> GateReceipt {
    GateReceipt {
        badge_id: "TESTBADGE123".to_string(),
        agent_owner: owner,
        card_hash: card_hash(),
        checked_at_ms: 1_000,
    }
}

struct Setup {
    chain: Chain,
    address: ContractAddress,
}

fn setup() -> Setup {
    let mut chain = Chain::new();
    for acc in [OWNER, CHECKER, CLIENT, PROVIDER, PROVIDER_B, STRANGER] {
        chain.create_account(Account::new(acc, Amount::from_ccd(START_BALANCE)));
    }
    let module = module_load_v1(MODULE_PATH).expect("module loads");
    let deploy = chain
        .module_deploy_v1(Signer::with_one_key(), OWNER, module)
        .expect("module deploys");
    let init = chain
        .contract_init(
            Signer::with_one_key(),
            OWNER,
            Energy::from(10_000),
            InitContractPayload {
                amount: Amount::zero(),
                mod_ref: deploy.module_reference,
                init_name: OwnedContractName::new_unchecked(format!("init_{CONTRACT_NAME}")),
                param: OwnedParameter::from_serial(&InitParams { checkers: vec![CHECKER] })
                    .expect("init params serialise"),
            },
        )
        .expect("contract initialises");
    Setup { chain, address: init.contract_address }
}

fn update(
    s: &mut Setup,
    invoker: AccountAddress,
    amount: Amount,
    entrypoint: &str,
    param: &impl concordium_contracts_common::Serial,
) -> Result<ContractInvokeSuccess, ContractInvokeError> {
    s.chain.contract_update(
        Signer::with_one_key(),
        invoker,
        Address::Account(invoker),
        Energy::from(100_000),
        UpdateContractPayload {
            amount,
            address: s.address,
            receive_name: OwnedReceiveName::new_unchecked(format!("{CONTRACT_NAME}.{entrypoint}")),
            message: OwnedParameter::from_serial(param).expect("param serialises"),
        },
    )
}

fn invoke_view<T: concordium_contracts_common::Deserial>(
    s: &Setup,
    entrypoint: &str,
    param: &impl concordium_contracts_common::Serial,
) -> T {
    let res = s
        .chain
        .contract_invoke(
            CLIENT,
            Address::Account(CLIENT),
            Energy::from(100_000),
            UpdateContractPayload {
                amount: Amount::zero(),
                address: s.address,
                receive_name: OwnedReceiveName::new_unchecked(format!(
                    "{CONTRACT_NAME}.{entrypoint}"
                )),
                message: OwnedParameter::from_serial(param).expect("param serialises"),
            },
        )
        .expect("view call succeeds");
    res.parse_return_value().expect("return value parses")
}

fn record_receipt(s: &mut Setup, owner: AccountAddress) -> u64 {
    let id = s.chain.block_time().timestamp_millis(); // placeholder, replaced below
    let _ = id;
    let res = update(s, CHECKER, Amount::zero(), "recordGate", &receipt_for(owner))
        .expect("checker records a receipt");
    res.parse_return_value().expect("receipt id parses")
}

fn create_job(s: &mut Setup, receipt_id: u64, provider: AccountAddress) -> u64 {
    let params = CreateJobParams {
        provider,
        description_uri: "ipfs://job-spec".to_string(),
        deadline_ms: DEADLINE_MS,
        receipt_id,
    };
    let res = update(s, CLIENT, Amount::from_ccd(ESCROW), "createJob", &params)
        .expect("job creation succeeds");
    res.parse_return_value().expect("job id parses")
}

fn job_in_state_delivered(s: &mut Setup) -> u64 {
    let receipt = record_receipt(s, PROVIDER);
    let job = create_job(s, receipt, PROVIDER);
    update(s, PROVIDER, Amount::zero(), "acceptJob", &job).expect("provider accepts");
    let deliver = DeliverParams { job_id: job, deliverable_hash: [9u8; 32] };
    update(s, PROVIDER, Amount::zero(), "deliverJob", &deliver).expect("provider delivers");
    job
}

// ---------------------------------------------------------------------------
// Gate
// ---------------------------------------------------------------------------

#[test]
fn create_job_without_receipt_fails() {
    let mut s = setup();
    let params = CreateJobParams {
        provider: PROVIDER,
        description_uri: "ipfs://job-spec".to_string(),
        deadline_ms: DEADLINE_MS,
        receipt_id: 42,
    };
    let res = update(&mut s, CLIENT, Amount::from_ccd(ESCROW), "createJob", &params);
    assert!(res.is_err(), "job without a gate receipt must fail");
}

#[test]
fn record_gate_by_non_checker_fails() {
    let mut s = setup();
    let res = update(&mut s, STRANGER, Amount::zero(), "recordGate", &receipt_for(PROVIDER));
    assert!(res.is_err(), "a non checker must not record gate receipts");
}

#[test]
fn create_job_with_valid_receipt_stores_gate() {
    let mut s = setup();
    let receipt = record_receipt(&mut s, PROVIDER);
    let job_id = create_job(&mut s, receipt, PROVIDER);
    let job: Job = invoke_view(&s, "getJob", &job_id);
    assert_eq!(job.gate.badge_id, "TESTBADGE123");
    assert_eq!(job.gate.agent_owner, PROVIDER);
    assert_eq!(job.gate.card_hash, card_hash());
    assert_eq!(job.amount, Amount::from_ccd(ESCROW));
    assert_eq!(s.chain.contract_balance(s.address), Some(Amount::from_ccd(ESCROW)));
    let count: u64 = invoke_view(&s, "jobCount", &());
    assert_eq!(count, 1);
}

#[test]
fn receipt_for_agent_a_cannot_hire_agent_b() {
    let mut s = setup();
    let receipt = record_receipt(&mut s, PROVIDER);
    let params = CreateJobParams {
        provider: PROVIDER_B,
        description_uri: "ipfs://job-spec".to_string(),
        deadline_ms: DEADLINE_MS,
        receipt_id: receipt,
    };
    let res = update(&mut s, CLIENT, Amount::from_ccd(ESCROW), "createJob", &params);
    assert!(res.is_err(), "receipt owner mismatch must fail");
}

#[test]
fn record_gate_with_empty_badge_fails() {
    let mut s = setup();
    let bad = GateReceipt {
        badge_id: String::new(),
        agent_owner: PROVIDER,
        card_hash: card_hash(),
        checked_at_ms: 1_000,
    };
    let res = update(&mut s, CHECKER, Amount::zero(), "recordGate", &bad);
    assert!(res.is_err(), "a receipt with an empty badge id must fail");
}

#[test]
fn record_gate_with_zero_card_hash_fails() {
    let mut s = setup();
    let bad = GateReceipt {
        badge_id: "TESTBADGE123".to_string(),
        agent_owner: PROVIDER,
        card_hash: [0u8; 32],
        checked_at_ms: 1_000,
    };
    let res = update(&mut s, CHECKER, Amount::zero(), "recordGate", &bad);
    assert!(res.is_err(), "a receipt with a zero card hash must fail");
}

#[test]
fn receipt_cannot_be_used_twice() {
    let mut s = setup();
    let receipt = record_receipt(&mut s, PROVIDER);
    let _ = create_job(&mut s, receipt, PROVIDER);
    let params = CreateJobParams {
        provider: PROVIDER,
        description_uri: "ipfs://job-spec-2".to_string(),
        deadline_ms: DEADLINE_MS,
        receipt_id: receipt,
    };
    let res = update(&mut s, CLIENT, Amount::from_ccd(ESCROW), "createJob", &params);
    assert!(res.is_err(), "a receipt hires exactly one job");
}

// ---------------------------------------------------------------------------
// Lifecycle
// ---------------------------------------------------------------------------

#[test]
fn accept_by_non_provider_fails() {
    let mut s = setup();
    let receipt = record_receipt(&mut s, PROVIDER);
    let job = create_job(&mut s, receipt, PROVIDER);
    let res = update(&mut s, STRANGER, Amount::zero(), "acceptJob", &job);
    assert!(res.is_err());
}

#[test]
fn accept_after_deadline_fails() {
    let mut s = setup();
    let receipt = record_receipt(&mut s, PROVIDER);
    let job = create_job(&mut s, receipt, PROVIDER);
    s.chain.tick_block_time(Duration::from_millis(DEADLINE_MS + 1)).expect("tick");
    let res = update(&mut s, PROVIDER, Amount::zero(), "acceptJob", &job);
    assert!(res.is_err(), "acceptance after the deadline must fail");
}

#[test]
fn deliver_before_acceptance_fails() {
    let mut s = setup();
    let receipt = record_receipt(&mut s, PROVIDER);
    let job = create_job(&mut s, receipt, PROVIDER);
    let deliver = DeliverParams { job_id: job, deliverable_hash: [9u8; 32] };
    let res = update(&mut s, PROVIDER, Amount::zero(), "deliverJob", &deliver);
    assert!(res.is_err());
}

#[test]
fn deliver_with_empty_hash_fails() {
    let mut s = setup();
    let receipt = record_receipt(&mut s, PROVIDER);
    let job = create_job(&mut s, receipt, PROVIDER);
    update(&mut s, PROVIDER, Amount::zero(), "acceptJob", &job).expect("accepts");
    let deliver = DeliverParams { job_id: job, deliverable_hash: [0u8; 32] };
    let res = update(&mut s, PROVIDER, Amount::zero(), "deliverJob", &deliver);
    assert!(res.is_err(), "empty deliverable hash must fail");
}

#[test]
fn release_by_non_client_fails() {
    let mut s = setup();
    let job = job_in_state_delivered(&mut s);
    let res = update(&mut s, STRANGER, Amount::zero(), "releaseJob", &job);
    assert!(res.is_err());
}

#[test]
fn release_pays_provider_and_settles_stats() {
    let mut s = setup();
    let job = job_in_state_delivered(&mut s);
    let before = s.chain.account_balance_available(PROVIDER).expect("provider exists");
    update(&mut s, CLIENT, Amount::zero(), "releaseJob", &job).expect("client releases");
    let after = s.chain.account_balance_available(PROVIDER).expect("provider exists");
    assert_eq!(after - before, Amount::from_ccd(ESCROW));
    assert_eq!(s.chain.contract_balance(s.address), Some(Amount::zero()));
    let stats: badgegate_escrow::ProviderStats = invoke_view(&s, "getProviderStats", &PROVIDER);
    assert_eq!(stats.jobs_settled, 1);
    assert_eq!(stats.volume_settled, Amount::from_ccd(ESCROW));
}

#[test]
fn release_twice_fails() {
    let mut s = setup();
    let job = job_in_state_delivered(&mut s);
    update(&mut s, CLIENT, Amount::zero(), "releaseJob", &job).expect("first release");
    let res = update(&mut s, CLIENT, Amount::zero(), "releaseJob", &job);
    assert!(res.is_err(), "second release must fail");
}

#[test]
fn claim_before_delivery_fails() {
    let mut s = setup();
    let receipt = record_receipt(&mut s, PROVIDER);
    let job = create_job(&mut s, receipt, PROVIDER);
    s.chain.tick_block_time(Duration::from_millis(4 * DAY_MS)).expect("tick");
    let res = update(&mut s, PROVIDER, Amount::zero(), "claimAfterWindow", &job);
    assert!(res.is_err());
}

#[test]
fn claim_before_window_fails() {
    let mut s = setup();
    let job = job_in_state_delivered(&mut s);
    s.chain.tick_block_time(Duration::from_millis(2 * DAY_MS)).expect("tick");
    let res = update(&mut s, PROVIDER, Amount::zero(), "claimAfterWindow", &job);
    assert!(res.is_err(), "claim inside the 3 day window must fail");
}

#[test]
fn claim_after_window_pays_provider() {
    let mut s = setup();
    let job = job_in_state_delivered(&mut s);
    s.chain.tick_block_time(Duration::from_millis(3 * DAY_MS + 1)).expect("tick");
    update(&mut s, PROVIDER, Amount::zero(), "claimAfterWindow", &job).expect("claim succeeds");
    assert_eq!(s.chain.contract_balance(s.address), Some(Amount::zero()));
    let stats: badgegate_escrow::ProviderStats = invoke_view(&s, "getProviderStats", &PROVIDER);
    assert_eq!(stats.jobs_settled, 1);
}

#[test]
fn cancel_before_acceptance_refunds_client() {
    let mut s = setup();
    let receipt = record_receipt(&mut s, PROVIDER);
    let job = create_job(&mut s, receipt, PROVIDER);
    update(&mut s, CLIENT, Amount::zero(), "cancelJob", &job).expect("cancel succeeds");
    assert_eq!(s.chain.contract_balance(s.address), Some(Amount::zero()));
}

#[test]
fn cancel_after_deadline_without_delivery_refunds() {
    let mut s = setup();
    let receipt = record_receipt(&mut s, PROVIDER);
    let job = create_job(&mut s, receipt, PROVIDER);
    update(&mut s, PROVIDER, Amount::zero(), "acceptJob", &job).expect("accepts");
    s.chain.tick_block_time(Duration::from_millis(DEADLINE_MS + 1)).expect("tick");
    update(&mut s, CLIENT, Amount::zero(), "cancelJob", &job).expect("cancel succeeds");
    assert_eq!(s.chain.contract_balance(s.address), Some(Amount::zero()));
}

#[test]
fn cancel_after_delivery_fails() {
    let mut s = setup();
    let job = job_in_state_delivered(&mut s);
    let res = update(&mut s, CLIENT, Amount::zero(), "cancelJob", &job);
    assert!(res.is_err(), "a delivered job cannot be cancelled");
}

#[test]
fn rate_before_settlement_fails() {
    let mut s = setup();
    let receipt = record_receipt(&mut s, PROVIDER);
    let job = create_job(&mut s, receipt, PROVIDER);
    let rate = RateParams { job_id: job, rating: 5 };
    let res = update(&mut s, CLIENT, Amount::zero(), "rateJob", &rate);
    assert!(res.is_err());
}

#[test]
fn rate_by_non_client_fails() {
    let mut s = setup();
    let job = job_in_state_delivered(&mut s);
    update(&mut s, CLIENT, Amount::zero(), "releaseJob", &job).expect("release");
    let rate = RateParams { job_id: job, rating: 5 };
    let res = update(&mut s, STRANGER, Amount::zero(), "rateJob", &rate);
    assert!(res.is_err());
}

#[test]
fn rate_out_of_range_fails() {
    let mut s = setup();
    let job = job_in_state_delivered(&mut s);
    update(&mut s, CLIENT, Amount::zero(), "releaseJob", &job).expect("release");
    for bad in [0u8, 6u8] {
        let rate = RateParams { job_id: job, rating: bad };
        let res = update(&mut s, CLIENT, Amount::zero(), "rateJob", &rate);
        assert!(res.is_err(), "rating {bad} must fail");
    }
}

#[test]
fn rate_twice_fails_and_stats_update_once() {
    let mut s = setup();
    let job = job_in_state_delivered(&mut s);
    update(&mut s, CLIENT, Amount::zero(), "releaseJob", &job).expect("release");
    let rate = RateParams { job_id: job, rating: 4 };
    update(&mut s, CLIENT, Amount::zero(), "rateJob", &rate).expect("first rating");
    let res = update(&mut s, CLIENT, Amount::zero(), "rateJob", &rate);
    assert!(res.is_err(), "second rating must fail");
    let stats: badgegate_escrow::ProviderStats = invoke_view(&s, "getProviderStats", &PROVIDER);
    assert_eq!(stats.rating_count, 1);
    assert_eq!(stats.rating_sum, 4);
}

// ---------------------------------------------------------------------------
// Owner powers
// ---------------------------------------------------------------------------

#[test]
fn pause_by_non_owner_fails() {
    let mut s = setup();
    let res = update(&mut s, STRANGER, Amount::zero(), "setJobCreationPaused", &true);
    assert!(res.is_err());
}

#[test]
fn paused_blocks_new_jobs_but_not_existing_job_flows() {
    let mut s = setup();
    let job = job_in_state_delivered(&mut s);
    update(&mut s, OWNER, Amount::zero(), "setJobCreationPaused", &true).expect("owner pauses");
    let receipt = record_receipt(&mut s, PROVIDER);
    let params = CreateJobParams {
        provider: PROVIDER,
        description_uri: "ipfs://job-spec".to_string(),
        deadline_ms: DEADLINE_MS,
        receipt_id: receipt,
    };
    let res = update(&mut s, CLIENT, Amount::from_ccd(ESCROW), "createJob", &params);
    assert!(res.is_err(), "creation must fail while paused");
    update(&mut s, CLIENT, Amount::zero(), "releaseJob", &job)
        .expect("release still works while paused");
}

#[test]
fn paused_deliver_still_works() {
    let mut s = setup();
    let receipt = record_receipt(&mut s, PROVIDER);
    let job = create_job(&mut s, receipt, PROVIDER);
    update(&mut s, PROVIDER, Amount::zero(), "acceptJob", &job).expect("provider accepts");
    update(&mut s, OWNER, Amount::zero(), "setJobCreationPaused", &true).expect("owner pauses");
    let deliver = DeliverParams { job_id: job, deliverable_hash: [9u8; 32] };
    update(&mut s, PROVIDER, Amount::zero(), "deliverJob", &deliver)
        .expect("deliver still works while paused");
    let stored: Job = invoke_view(&s, "getJob", &job);
    assert_eq!(stored.deliverable_hash, [9u8; 32]);
}

#[test]
fn paused_claim_still_works() {
    let mut s = setup();
    let job = job_in_state_delivered(&mut s);
    update(&mut s, OWNER, Amount::zero(), "setJobCreationPaused", &true).expect("owner pauses");
    s.chain.tick_block_time(Duration::from_millis(3 * DAY_MS + 1)).expect("tick");
    update(&mut s, PROVIDER, Amount::zero(), "claimAfterWindow", &job)
        .expect("claim still works while paused");
    assert_eq!(s.chain.contract_balance(s.address), Some(Amount::zero()));
    let stats: badgegate_escrow::ProviderStats = invoke_view(&s, "getProviderStats", &PROVIDER);
    assert_eq!(stats.jobs_settled, 1);
}

#[test]
fn paused_cancel_still_works() {
    let mut s = setup();
    let receipt = record_receipt(&mut s, PROVIDER);
    let job = create_job(&mut s, receipt, PROVIDER);
    update(&mut s, OWNER, Amount::zero(), "setJobCreationPaused", &true).expect("owner pauses");
    update(&mut s, CLIENT, Amount::zero(), "cancelJob", &job)
        .expect("cancel still works while paused");
    assert_eq!(s.chain.contract_balance(s.address), Some(Amount::zero()));
}

#[test]
fn checker_view_reports_allowlist() {
    let s = setup();
    let yes: bool = invoke_view(&s, "isChecker", &CHECKER);
    let no: bool = invoke_view(&s, "isChecker", &STRANGER);
    assert!(yes);
    assert!(!no);
}
