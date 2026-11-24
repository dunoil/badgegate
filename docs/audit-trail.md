# Audit Trail Design

Every BadgeGate hire produces one audit trail, built only from on chain events and the gate receipt, so it can be replayed without trusting a BadgeGate server.

## Rows in a trail

1. Gate receipt: badge id, registry status, owner, card hash computed locally, card hash anchored on chain, match result, timestamp, checker account. Source: badge check service output, recorded on chain by the checker as a GateRecorded event, then attested onto the job at creation as a GateAttested event. (CONCEPT.md calls this row GateChecked; the contract event names above are the canonical ones, aligned in the phase 4 read through, 24 Nov 2026.)
2. JobCreated: client, provider, escrow amount in CCD, deadline, description URI, transaction link.
3. JobAccepted: provider, transaction link.
4. JobDelivered: deliverable hash, transaction link.
5. Settlement, exactly one of: JobReleased (client released), JobClaimedAfterWindow (provider claimed after 3 days), JobRefunded (client cancelled under the cancel rules).
6. JobRated: rating 1 to 5, only possible after settlement, only by the paying client.

## Links

Each on chain row links to CCDExplorer (testnet) for its transaction and to the escrow contract instance page. The gate row links to the Agent Registry entry for the badge. Testnet URL formats are a 25 Nov verification task in the project research file; the frontend keeps the base URL in one config constant.

## Why this shape

EU AI Act Article 12 asks for record keeping that lets an operator trace what an AI system did. A BadgeGate trail answers, for one hire: which agent acted, which verified human or business owns it, what it was paid, what it delivered (by hash), and how the payment ended. That is the lawful route from an agent action back to an accountable owner that the hackathon brief asks for, and it is the production audit export described in path-to-production.md.
