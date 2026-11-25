// Usage: node src/cli.js <badgeId> <network> <claimedOwner>
// Runs the full gate: registry lookup, card fetch, local hash, decision.
//
// claimedOwner is REQUIRED (AUDIT.md F1 and F6). A gate verdict that can
// produce a hire receipt must always verify the agent's owner against the
// registry record; making the owner an optional argument is what allowed
// the owner check to be skipped silently. The receipt printed on success
// carries the verified owner from the registry record (badgeOwner), which
// is the value a checker then records on chain as the receipt agent_owner.
import { fetchBadgeRecord, fetchCardBytes } from "./badgeCheck.js";
import { decideGate } from "./gate.js";

const [badgeId, network = "mainnet", claimedOwner] = process.argv.slice(2);
if (!badgeId || !claimedOwner) {
  console.error("usage: node src/cli.js <badgeId> <network> <claimedOwner>");
  console.error("claimedOwner is required: a hire verdict must verify the agent owner.");
  process.exit(2);
}

const record = await fetchBadgeRecord(badgeId, { network });
let cardBytes = null;
if (record?.agent_uri) {
  try {
    cardBytes = await fetchCardBytes(record.agent_uri);
  } catch {
    cardBytes = null;
  }
}
const verdict = decideGate({ record, cardBytes, claimedOwner });
console.log(JSON.stringify({ badgeId, network, verdict }, null, 2));
process.exit(verdict.allow ? 0 : 1);
