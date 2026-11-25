# BadgeGate

No badge, no deal.

BadgeGate is a hire gate for AI agents, built for the Concordium Trusted Agents Hackathon (Encode Club). Before any money moves, BadgeGate checks the agent that wants the job: is it registered in the Concordium Agent Registry, does it carry a Verified by Concordium badge, is its Agent Card the exact card anchored on chain, and is a verified human or business accountable for it? If any check fails, the agent is rejected. If all checks pass, the agent is hired into a CCD escrow on Concordium testnet and paid on delivery, and every step lands in a public audit trail on the testnet explorer.

## The demo in one paragraph

A copycat agent presents a copied Agent Card and no valid badge. BadgeGate checks the registry, the badge does not resolve, verdict: REJECTED, no job and no money. The real agent presents its badge. Registry status is Active, the owner matches, and the SHA-256 hash of its card matches the hash anchored on chain. Verdict: HIRED. A checker records the gate receipt on chain, the client funds the escrow, the agent delivers, the escrow releases, and the whole hire can be replayed from the audit trail.

Demo video (at most 3 minutes): https://youtu.be/plbtTMJIMrs
Live frontend demo: https://dunoil.github.io/badgegate/

## Live deployment (Concordium testnet)

- Escrow module reference: f73828b9ec0f9774cbbc9a9c453fcd5b4c2e8de016e3cb6b345fe0a82e1d557a
- Escrow contract index: 13158, subindex 0
- Contract explorer link (CCDScan testnet): https://testnet.ccdscan.io/?dcount=1&dentity=contract&dcontractAddressIndex=13158&dcontractAddressSubIndex=0 (opens the contract record; creator is the badgegate account). Setup transactions: module deploy [81bf4b379a405b431ffc97a7da317a055c09ae48c0f8a46192ffc73fa5137cc2](https://testnet.ccdscan.io/?dcount=1&dentity=transaction&dhash=81bf4b379a405b431ffc97a7da317a055c09ae48c0f8a46192ffc73fa5137cc2), contract init [f268552c5b48ab9bd8a27bb1d1f5e4615fe00773bee7418d1fe6440db3a5ee5f](https://testnet.ccdscan.io/?dcount=1&dentity=transaction&dhash=f268552c5b48ab9bd8a27bb1d1f5e4615fe00773bee7418d1fe6440db3a5ee5f). Explorer note: CCDScan is the working testnet explorer for these records; CCDExplorer testnet was unreachable from the build environment in every attempt.
- Demo agent token id: 1479
- Demo agent badge: JyP9XeEGKMrWwjo74GdGK
- Agent registration transaction: [0858a10d42bf7ab6572ffa78468ab7adfee2cb6cbd5dc07d7c0f22a2429735f8](https://testnet.ccdscan.io/?dcount=1&dentity=transaction&dhash=0858a10d42bf7ab6572ffa78468ab7adfee2cb6cbd5dc07d7c0f22a2429735f8)
- Demo Agent Card: https://raw.githubusercontent.com/dunoil/badgegate/main/agents/badgegate-demo-research-agent/agent-card.json
- Example transactions (escrow 13158,0; each hash links to its CCDScan testnet record): gate receipt [e585e8e01f43e3f56d029c17c50b8ce89cb043842e72e663b9900997c3f1b9b0](https://testnet.ccdscan.io/?dcount=1&dentity=transaction&dhash=e585e8e01f43e3f56d029c17c50b8ce89cb043842e72e663b9900997c3f1b9b0) (recordGate, receipt id 0). Completed demo job id 0, 10 CCD, run end to end on testnet: createJob [9195cfe1dfbfb6c114f4b3f0e47d522b27eeab3d616eb54529de14c0be06cd4c](https://testnet.ccdscan.io/?dcount=1&dentity=transaction&dhash=9195cfe1dfbfb6c114f4b3f0e47d522b27eeab3d616eb54529de14c0be06cd4c) (client account, consumes receipt id 0), acceptJob [8e480b2113b07bdd3ee0e01829aff6f20c1f7a694f29578e38da235e96077133](https://testnet.ccdscan.io/?dcount=1&dentity=transaction&dhash=8e480b2113b07bdd3ee0e01829aff6f20c1f7a694f29578e38da235e96077133), deliverJob [b83e4bfa62cfcdedee8093698d051b398d70b405fed33376da8996520d3547d2](https://testnet.ccdscan.io/?dcount=1&dentity=transaction&dhash=b83e4bfa62cfcdedee8093698d051b398d70b405fed33376da8996520d3547d2), releaseJob [8f9adbc40c86fdc9c05b885ee73605ff0115cd225959d5a2790e2afc5d64c50b](https://testnet.ccdscan.io/?dcount=1&dentity=transaction&dhash=8f9adbc40c86fdc9c05b885ee73605ff0115cd225959d5a2790e2afc5d64c50b) (escrow paid to the provider), rateJob [7378a26fc5f09876cc997b5f24f71461fdc6ba97b9fd5bf69911d15f57ee4baf](https://testnet.ccdscan.io/?dcount=1&dentity=transaction&dhash=7378a26fc5f09876cc997b5f24f71461fdc6ba97b9fd5bf69911d15f57ee4baf) (5 stars). The job reads back in state Released with rating 5. Design note: the first single account attempt on the first deployment, d7293791a7efe943620204a26d27fd6ff818a03be0069791fb7c5b84070878d6, was rejected on chain by the contract with error SelfHire (code -21), because the contract forbids the client and the provider being the same account; the completed run uses a second testnet account as client.
- Audit fix proofs (25 Nov 2026, on the current contract 13158,0): a gate receipt dated 8 days in the past was rejected on chain with the new ReceiptStale error (code -25), transaction [b77594fc0572c84ced34614711a5069038880d7ef8fc6b305dd8450a2d66e6d0](https://testnet.ccdscan.io/?dcount=1&dentity=transaction&dhash=b77594fc0572c84ced34614711a5069038880d7ef8fc6b305dd8450a2d66e6d0); a receipt dated a day in the future was rejected with ReceiptFutureDated (code -26), transaction [3a0b2017a00c7b0806b67d4940a0b7158dc8e4cbf5c690086ef58ff6699c8e24](https://testnet.ccdscan.io/?dcount=1&dentity=transaction&dhash=3a0b2017a00c7b0806b67d4940a0b7158dc8e4cbf5c690086ef58ff6699c8e24); and a delivery attempted after a 150 second job deadline was rejected with the new DeliveryLate error (code -24), transaction [fb0dd85d989fd16e74e3ef207d515060470a380f9c92a6ee954a0da417c502f4](https://testnet.ccdscan.io/?dcount=1&dentity=transaction&dhash=fb0dd85d989fd16e74e3ef207d515060470a380f9c92a6ee954a0da417c502f4), after which the client cancelled that job and recovered its 1 CCD in full (job state Refunded).

## What it uses from Concordium

- Agent Registry and CIS-8004: the provider agent is minted as a CIS-2 token on testnet (registry index 12802, subindex 0), with its Agent Card pointer and card hash anchored on chain.
- Verified by Concordium badge: the portable trust marker BadgeGate checks before every hire, following the official five step verification procedure. The card hash is always computed locally by the check service, never trusted from a served flag.
- Agent Registry MCP server: the hosted official service is the check transport, and the same check is shaped for agent to agent use before two agents deal with each other.
- A native escrow contract on Concordium testnet, written in Rust with concordium-std 11: two step gate attestation (a checker records a gate receipt, job creation consumes exactly one receipt and the provider must equal the receipt agent owner), payable job creation, delivery with an anchored deliverable hash, release, claim after a 3 day review window, cancel and refund, and ratings that are only possible after a settled payment.
- x402 and sponsored transactions: the production settlement rail, documented in services/payment-relay. The hackathon MVP settles through the escrow release on testnet, which is a real on chain payment. No official Concordium hosted public testnet x402 facilitator was found, and this repository says so plainly instead of faking one.

## Repository layout

- contracts/badgegate-escrow: the Rust escrow contract, its integration tests, and its test checklist.
- services/badge-check: the gate. Badge verification, local card hash verification, and the hire decision with reason codes. Includes unit tests and a CLI.
- services/payment-relay: the x402 settlement hand off, an explicit stub with integration points marked.
- frontend: a zero build static demo. Two agent panels run on labelled fixtures; a live badge check panel runs the real gate in the visitor's browser against the testnet registry (a faithful port of services/badge-check in frontend/live-gate.js); and the audit trail of the real demo hire links every step to its CCDScan testnet record.
- docs: the audit trail design and the path to production.
- examples: the demo provider Agent Card in ERC-8004 registration file format.
- scripts: the testnet client wrapper used for deployment (reads proxy settings from the environment at run time, stores no credentials).

## Running the checks locally

Badge check service (Node 20 or later, no dependencies):

```bash
cd services/badge-check
npm test
```

Live badge check against the testnet registry:

```bash
node src/cli.js <BADGE_ID> testnet <OWNER_ACCOUNT>
```

The owner account argument is required: a hire verdict must always verify the agent owner against the registry record.

Escrow contract tests (Rust with the Concordium toolchain, see the project notes in contracts/badgegate-escrow/Cargo.toml for pinned versions: concordium-std 11, cargo-concordium 5, concordium-client 10):

```bash
cd contracts/badgegate-escrow
cargo concordium build -e -o module.wasm.v1
cargo test
```

Current results: 36 of 36 contract integration tests pass against the compiled module, and 10 of 10 badge check unit tests pass. A security audit on 25 Nov 2026 found one High and two Medium issues, and all three were fixed before this deployment: the badge service owner check now genuinely enforces against the real registry endpoint shape (it previously read a field the endpoint never sends), deliverJob rejects delivery after the job deadline, and gate receipts expire 7 days after the check. The live rejection proofs are linked in the deployment section above. The frontend live panel port was tested headlessly in Node against the live testnet endpoint on 25 Nov 2026: our badge returned OK with the locally computed card hash matching the anchor, an invented badge returned BADGE_NOT_FOUND, and our badge with a deliberately wrong claimed owner returned OWNER_MISMATCH. Live testnet evidence: three real testnet registry agents (token ids 500, 1000, and 1477) passed the full gate end to end, and sampled invalid agents were rejected with the correct reason codes.

## Safety properties

- The escrow contract has no owner withdrawal, sweep, or rescue function. This was verified by inspecting the compiled module: its entrypoints are exactly init, recordGate, createJob, acceptJob, deliverJob, releaseJob, claimAfterWindow, cancelJob, rateJob, setJobCreationPaused, getJob, jobCount, getProviderStats, and isChecker.
- Escrowed CCD can only go to the job provider on settlement, or back to the job client on a valid cancel or refund.
- The owner can only pause new job creation. Pausing never blocks payouts, refunds, deliveries, or claims on existing jobs.
- One gate receipt hires exactly one job, and only for the agent owner named in the receipt.
- Gate receipts expire: a receipt older than 7 days, or dated ahead of the block time beyond a 5 minute tolerance, is rejected at both recordGate and createJob.
- Delivery must land by the job deadline. A late delivery is rejected, so it cannot close the client's refund path or unlock the claim window.
- Reputation is derived only from settled, paid jobs. A rating without a settled job is impossible by construction.
- BadgeGate never asks for seed phrases, private keys, or documents. Identity steps happen only inside the official Concordium wallet and identity provider flow, and the testnet identity used here is issued without real document verification.

## Status

Built starting 24 Nov 2026 for the Concordium Trusted Agents Hackathon. Submission deadline 9 Dec 2026, 23:59 UTC. Everything runs on Concordium testnet with test CCD from the faucet. Total spend: zero.

## License

MIT. See LICENSE.
