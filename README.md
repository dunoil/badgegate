# BadgeGate

No badge, no deal.

BadgeGate is a hire gate for AI agents, built for the Concordium Trusted Agents Hackathon (Encode Club). Before any money moves, BadgeGate checks the agent that wants the job: is it registered in the Concordium Agent Registry, does it carry a Verified by Concordium badge, is its Agent Card the exact card anchored on chain, and is a verified human or business accountable for it? If any check fails, the agent is rejected. If all checks pass, the agent is hired into a CCD escrow on Concordium testnet and paid on delivery, and every step lands in a public audit trail on CCDExplorer.

## The demo in one paragraph

A copycat agent presents a copied Agent Card and no valid badge. BadgeGate checks the registry, the badge does not resolve, verdict: REJECTED, no job and no money. The real agent presents its badge. Registry status is active, the owner matches, and the SHA-256 hash of its card matches the hash anchored on chain. Verdict: HIRED. The client funds the escrow, the agent delivers, the escrow releases, and the whole hire can be replayed from the audit trail.

## What it uses from Concordium

- Agent Registry and CIS-8004: the provider agent is minted as a CIS-2 token, with its Agent Card pointer and card hash anchored on chain.
- Verified by Concordium badge: the portable trust marker BadgeGate checks before every hire, through the registry badge check endpoint and the hosted Agent Registry MCP server.
- A native escrow contract on Concordium testnet, written in Rust with concordium-std: payable job creation, delivery with an anchored deliverable hash, release, claim after a 3 day review window, cancel and refund, and ratings that are only possible after a settled payment.
- x402 and sponsored transactions: the production settlement rail, documented in services/payment-relay. The hackathon MVP settles through the escrow release on testnet, which is a real on chain payment.

## Repository layout

- contracts/badgegate-escrow: the Rust escrow contract and its test checklist.
- services/badge-check: the gate. Badge verification, local card hash verification, and the hire decision with reason codes. Includes unit tests.
- services/payment-relay: the x402 settlement hand off, an explicit stub with integration points marked.
- frontend: a zero build static demo (two agent panels, job board, audit trail view).
- docs: the audit trail design and the path to production.
- examples: the demo provider Agent Card in ERC-8004 registration file format.

## Status

Implemented and tested, 24 Nov 2026. Deployment to Concordium testnet is pending a testnet wallet step and will be recorded here with its contract index and explorer links once it is real. Nothing in this repository claims a deployment that has not happened.

- Escrow contract: full Rust implementation (concordium-std 11), two step gate attestation included. Test suite: 30 of 30 integration tests pass against the compiled module, and the module was inspected to confirm it exposes no withdrawal, sweep, or rescue entrypoint.
- Badge check service: 6 of 6 unit tests pass, and live mode is verified against the real testnet Agent Registry (CIS-8004 index 12802, subindex 0): three real testnet agents (token ids 500, 1000, and 1477) passed the full gate end to end, and sampled invalid agents were rejected with the correct reason codes (CARD_UNREACHABLE, CARD_HASH_MISMATCH, BADGE_NOT_FOUND). A real mainnet badge also passed the full gate.
- Frontend demo: static demo with the three step story and clearly labelled demo fixtures; live mode wires in after our own agent is registered.

## Safety properties

- The escrow contract has no owner withdrawal function. Escrowed CCD can only go to the job's provider on settlement, or back to the job's client on a valid cancel or refund.
- The owner can only pause new job creation. Pausing never blocks payouts, refunds, deliveries, or claims on existing jobs.
- Reputation is derived only from settled, paid jobs. A rating without a settled job is impossible by construction.
- BadgeGate never asks for seed phrases, private keys, or documents. Identity verification happens only inside the official Concordium wallet and identity provider flow.
