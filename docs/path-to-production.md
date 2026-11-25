# BadgeGate: Path to Production (Final)

Final version, 24 Nov 2026, for the Concordium Trusted Agents Hackathon submission. Live values below are from the deployed testnet demo.

BadgeGate on testnet is the full product shape with test money. Going live is a sequence of swaps, not a redesign: the gate rules, the escrow state machine, and the registry data model do not change.

## Where the demo stands today

- Escrow contract: Rust, concordium-std 11, built and fully tested, deployed on Concordium testnet on 24 Nov 2026. Module reference c32fd94ae6bd16b88e6af12dec13b96ba25a48eab329a5767c9ff0765fa03caf, contract index 13153, subindex 0, explorer record on CCDScan testnet: https://testnet.ccdscan.io/?dcount=1&dentity=contract&dcontractAddressIndex=13153&dcontractAddressSubIndex=0 (CCDScan drawer link format verified 25 Nov 2026; transactions are linked by hash in the project description).
- Demo agent: registered in the testnet CIS-8004 Agent Registry (index 12802, subindex 0), token id 1479, badge JyP9XeEGKMrWwjo74GdGK, Agent Card at https://raw.githubusercontent.com/dunoil/badgegate/main/agents/badgegate-demo-research-agent/agent-card.json.
- Gate: badge check service verified live against the real testnet registry, including three real third party testnet agents passing the full gate and sampled invalid agents rejected with correct reason codes.
- Tests: 30 of 30 contract integration tests and 6 of 6 badge check unit tests pass. The compiled module exposes no withdrawal, sweep, or rescue entrypoint.

## Step 1: Trustless badge reads

Today the badge check service reads the registry through the official indexer backed endpoint and the hosted MCP service, and re computes the card hash locally, which removes the largest trust hole (a served card is never trusted, and neither is a served match flag). Production reads the CIS-8004 registry contract directly through the Concordium Web SDK (mainnet index 10082, subindex 0), so no indexer is trusted for status, owner, or the anchored hash. The five step verification procedure stays identical; only the transport changes. Effort: one service rewrite of the registry client, no contract changes.

## Step 2: Mainnet escrow and mainnet registration

Deploy the same Rust escrow contract to mainnet: one module deploy and one init with concordium-client, preceded by an independent audit of a deliberately small state machine (14 entrypoints, no owner fund access by construction). Register production provider agents in the mainnet registry, which is already live. Where an agent also operates on Ethereum or Solana, bind its external keys with CIS-8 (mainnet index 10081) so the same badge covers it across chains. Note on identity: a mainnet account is created from a real verified identity through an official identity provider. That step belongs to each agent owner in their own wallet; BadgeGate never performs or intermediates identity verification.

## Step 3: x402 settlement for per call work

Escrow suits hired jobs with a delivery step. Per call agent commerce settles over x402: the agent signs a transfer naming a facilitator as sponsor, the facilitator verifies and co signs, and pays the CCD fee under Protocol 10 sponsored transactions, so the agent settles in the currency it holds and never touches CCD. The payment relay service in this repository is the integration point, with the facilitator hand off marked. Production needs one of two things: an agreement with an operating Concordium x402 facilitator, or a self run sponsor account funded with CCD. Honest status at submission time: no official Concordium hosted public testnet x402 facilitator was found during the build, so this step is a partnership or operations task, not a code task. The escrow release remains the settlement for hired jobs in both the demo and production.

## Step 4: Marketplace and framework embedding

The gate is a small service with one decision function and named reason codes, which makes it embeddable. Production packaging: an npm package wrapping the badge check, an MCP tool wrapper so agent frameworks can call the check natively before dealing (the brief itself suggests agents checking each other over MCP or A2A), and a hosted checker with an SLA for marketplaces that do not want to run their own. The on chain side is already shaped for this: checker accounts are an allowlist set at contract init, so a hosted checker is a configuration, not a fork.

## Step 5: Audit export

The audit trail becomes an exportable, signed record per hire: JSON plus the explorer transaction references for the gate receipt, job creation, delivery, and settlement, shaped for EU AI Act Article 12 record keeping and retained by both the client and the provider. The trail answers, for one hire: which agent acted, which verified human or business owns it, what it was paid, what it delivered (by hash), and how the payment ended. The event set needed for this export already exists in the contract; this step is formatting, signing, and retention policy.

## Step 6: Operations hardening (small, named)

- Gate receipt freshness: receipts currently carry the check timestamp (checked_at_ms) but the contract does not expire them. Production adds a validity window at job creation so a stale pass cannot hire after a badge status change, plus a re check at acceptance for long running jobs.
- Checker key management: the demo checker is one account. Production runs the checker as a hosted service account with rotation, and the allowlist is refreshed by deploying with the rotated set or by an owner managed checker update if the contract is extended.
- Rate limits and card fetch safety: the check service caps card fetch size and time today at the service layer; production adds registry level caching with hash revalidation.

## What production does not need

No new token, no change to the gate rules, no change to the escrow state machine, and no migration of registry data. The registry, the badge, sponsored transactions, and the explorer already run on mainnet today. Winners also remain eligible for the Concordium 75M CCD Grants Program, which is the natural funding route for steps 1 to 5.
