// Usage: node src/cli.js <badgeId> [network] [claimedOwner]
// Runs the full gate: registry lookup, card fetch, local hash, decision.
import { fetchBadgeRecord, fetchCardBytes } from "./badgeCheck.js";
import { decideGate } from "./gate.js";

const [badgeId, network = "mainnet", claimedOwner] = process.argv.slice(2);
if (!badgeId) {
  console.error("usage: node src/cli.js <badgeId> [network] [claimedOwner]");
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
