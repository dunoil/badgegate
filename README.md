# BadgeGate

No badge, no deal.

BadgeGate is a hire gate for AI agents, built for the Concordium Trusted Agents Hackathon (Encode Club). Before any money moves, BadgeGate checks the agent that wants the job: is it registered in the Concordium Agent Registry, does it carry a Verified by Concordium badge, is its Agent Card the exact card anchored on chain, and is a verified human or business accountable for it? If any check fails, the agent is rejected. If all checks pass, the agent is hired into a CCD escrow on Concordium testnet and paid on delivery, and every step lands in a public audit trail on the testnet explorer.

## The demo in one paragraph

A copycat agent presents a copied Agent Card and no valid badge. BadgeGate checks the registry, the badge does not resolve, verdict: REJECTED, no job and no money. The real agent presents its badge. Registry status is Active, the owner matches, and the SHA-256 hash of its card matches the hash anchored on chain. Verdict: HIRED. A checker records the gate receipt on chain, the client funds the escrow, the agent delivers, the escrow releases, and the whole hire can be replayed from the audit trail.

Demo video (at most 3 minutes): TO_BE_FILLED
Live frontend demo: https://dunoil.github.io/badgegate/

## Live deployment (Concordium testnet)

- Escrow module reference: c32fd94ae6bd16b88e6af12dec13b96ba25a48eab329a5767c9ff0765fa03caf
- Escrow contract index: 13153, subindex 0
- Contract explorer link (CCDScan testnet): https://testnet.ccdscan.io/?dcount=1&dentity=contract&dcontractAddressIndex=13153&dcontractAddressSubIndex=0 (opens the contract record; creator is the badgegate account). Setup transactions: module deploy [43a3116be17714cd6e1c79012f01116720e2e5054f7128b9138243054ecefed6](https://testnet.ccdscan.io/?dcount=1&dentity=transaction&dhash=43a3116be17714cd6e1c79012f01116720e2e5054f7128b9138243054ecefed6), contract init [bf0953cd452600e5c77fa2b4ff644a0d0c2c0c107f6f7fe76df668d45265c0bb](https://testnet.ccdscan.io/?dcount=1&dentity=transaction&dhash=bf0953cd452600e5c77fa2b4ff644a0d0c2c0c107f6f7fe76df668d45265c0bb). Explorer note: CCDScan is the working testnet explorer for these records; CCDExplorer testnet was unreachable from the build environment in every attempt.
- Demo agent token id: 1479
- Demo agent badge: JyP9XeEGKMrWwjo74GdGK
- Agent registration transaction: [0858a10d42bf7ab6572ffa78468ab7adfee2cb6cbd5dc07d7c0f22a2429735f8](https://testnet.ccdscan.io/?dcount=1&dentity=transaction&dhash=0858a10d42bf7ab6572ffa78468ab7adfee2cb6cbd5dc07d7c0f22a2429735f8)
- Demo Agent Card: https://raw.githubusercontent.com/dunoil/badgegate/main/agents/badgegate-demo-research-agent/agent-card.json
- Example transactions (escrow 13153,0; each hash links to its CCDScan testnet record): gate receipt [6169b254d9c6e47fd331cb8e296d675d2e533c3565c34ee49c9ceb815b1cc6fd](https://testnet.ccdscan.io/?dcount=1&dentity=transaction&dhash=6169b254d9c6e47fd331cb8e296d675d2e533c3565c34ee49c9ceb815b1cc6fd) (recordGate, receipt id 1). Completed demo job id 0, 10 CCD, run end to end on testnet: createJob [eca76f10f39ec47f517f684bb8082c9de45cb2ee8fd3a32115dd938ea2a77cf3](https://testnet.ccdscan.io/?dcount=1&dentity=transaction&dhash=eca76f10f39ec47f517f684bb8082c9de45cb2ee8fd3a32115dd938ea2a77cf3) (client account, consumes receipt id 1), acceptJob [7e1c4436b1c8d0927d9780867cae41e14481efc1f53c7fa14ee42b691dd3f055](https://testnet.ccdscan.io/?dcount=1&dentity=transaction&dhash=7e1c4436b1c8d0927d9780867cae41e14481efc1f53c7fa14ee42b691dd3f055), deliverJob [eaa2ebbf13529aa258af13e8fe20a414e9a828798d57325276070c741a18ab4e](https://testnet.ccdscan.io/?dcount=1&dentity=transaction&dhash=eaa2ebbf13529aa258af13e8fe20a414e9a828798d57325276070c741a18ab4e), releaseJob [6537366e63a967173fa70e0bf9b2d637fd45e34696cc625b4ccb1d1a2d0b5d3f](https://testnet.ccdscan.io/?dcount=1&dentity=transaction&dhash=6537366e63a967173fa70e0bf9b2d637fd45e34696cc625b4ccb1d1a2d0b5d3f) (escrow paid to the provider), rateJob [b4d239fdd3116b106e297b6a527c2cd141f660c5c4c61a18265415ae487a06e4](https://testnet.ccdscan.io/?dcount=1&dentity=transaction&dhash=b4d239fdd3116b106e297b6a527c2cd141f660c5c4c61a18265415ae487a06e4) (5 stars). The job reads back in state Released with rating 5. Design note: the first single account attempt, d7293791a7efe943620204a26d27fd6ff818a03be0069791fb7c5b84070878d6, was rejected on chain by the contract with error SelfHire (code -21), because the contract forbids the client and the provider being the same account; the completed run uses a second testnet account as client.

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
node src/cli.js <BADGE_OR_TOKEN_ID> --network testnet
```

Escrow contract tests (Rust with the Concordium toolchain, see the project notes in contracts/badgegate-escrow/Cargo.toml for pinned versions: concordium-std 11, cargo-concordium 5, concordium-client 10):

```bash
cd contracts/badgegate-escrow
cargo concordium build -e -o module.wasm.v1
cargo test
```

Current results: 30 of 30 contract integration tests pass against the compiled module, and 6 of 6 badge check unit tests pass. The frontend live panel port was tested headlessly in Node against the live testnet endpoint on 25 Nov 2026: our badge returned OK with the locally computed card hash matching the anchor, an invented badge returned BADGE_NOT_FOUND, and our badge with a deliberately wrong claimed owner returned OWNER_MISMATCH. Live testnet evidence: three real testnet registry agents (token ids 500, 1000, and 1477) passed the full gate end to end, and sampled invalid agents were rejected with the correct reason codes.

## Safety properties

- The escrow contract has no owner withdrawal, sweep, or rescue function. This was verified by inspecting the compiled module: its entrypoints are exactly init, recordGate, createJob, acceptJob, deliverJob, releaseJob, claimAfterWindow, cancelJob, rateJob, setJobCreationPaused, getJob, jobCount, getProviderStats, and isChecker.
- Escrowed CCD can only go to the job provider on settlement, or back to the job client on a valid cancel or refund.
- The owner can only pause new job creation. Pausing never blocks payouts, refunds, deliveries, or claims on existing jobs.
- One gate receipt hires exactly one job, and only for the agent owner named in the receipt.
- Reputation is derived only from settled, paid jobs. A rating without a settled job is impossible by construction.
- BadgeGate never asks for seed phrases, private keys, or documents. Identity steps happen only inside the official Concordium wallet and identity provider flow, and the testnet identity used here is issued without real document verification.

## Status

Built starting 24 Nov 2026 for the Concordium Trusted Agents Hackathon. Submission deadline 9 Dec 2026, 23:59 UTC. Everything runs on Concordium testnet with test CCD from the faucet. Total spend: zero.

## License

MIT. See LICENSE.
