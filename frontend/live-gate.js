// BadgeGate live gate: a faithful browser port of services/badge-check.
//
// badgeCheck.js fetches the registry record from the Concordium Agent
// Registry badge check endpoint (the same data the hosted Agent Registry
// MCP server exposes) and fetches the Agent Card bytes. gate.js then
// decides: the card hash is always computed locally and compared to the
// SHA-256 anchor on chain; a served match flag is never trusted.
//
// The only deliberate differences from the Node version:
//   - SHA-256 uses Web Crypto, so decideGate is async. The decision
//     order and the reason codes are identical to gate.js.
//   - The endpoint returns the owner as owner_account inside on_chain;
//     the fetch layer normalizes it to owner, the field gate.js reads.
//   - The network is fixed to testnet: this page is the testnet demo.

const BADGEGATE_LIVE_BASE = "https://agent-registry-mcp.concordium.com";

async function liveFetchBadgeRecord(badgeId, { base = BADGEGATE_LIVE_BASE, fetchImpl = fetch } = {}) {
  const url = `${base}/v1/badge-check/${encodeURIComponent(badgeId)}?network=testnet`;
  const res = await fetchImpl(url);
  // Same as the Node client: the endpoint answers 400, not 404, for an
  // unknown or malformed badge id. Both mean "no such registration".
  if (res.status === 404 || res.status === 400) return null;
  if (!res.ok) throw new Error(`badge check endpoint returned ${res.status}`);
  const body = await res.json();
  const raw = body.on_chain ?? body;
  return { ...raw, owner: raw.owner ?? raw.owner_account, tokenId: body.token_id };
}

async function liveFetchCardBytes(agentUri, { fetchImpl = fetch } = {}) {
  const res = await fetchImpl(agentUri);
  if (!res.ok) throw new Error(`agent card fetch returned ${res.status}`);
  return new Uint8Array(await res.arrayBuffer());
}

async function liveSha256Hex(bytes) {
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  return [...new Uint8Array(digest)].map((b) => b.toString(16).padStart(2, "0")).join("");
}

// Same decision order as gate.js decideGate:
// not found, not active, owner mismatch, card unreachable, hash mismatch.
async function liveDecideGate({ record, cardBytes, claimedOwner }) {
  if (!record) {
    return { allow: false, reason: "BADGE_NOT_FOUND" };
  }
  if (String(record.status).toLowerCase() !== "active") {
    return { allow: false, reason: "AGENT_NOT_ACTIVE", status: record.status };
  }
  if (claimedOwner && record.owner && claimedOwner !== record.owner) {
    return { allow: false, reason: "OWNER_MISMATCH", owner: record.owner };
  }
  if (!cardBytes) {
    return { allow: false, reason: "CARD_UNREACHABLE", agentUri: record.agent_uri };
  }
  const localHash = await liveSha256Hex(cardBytes);
  const anchored = String(record.metadata_hash_hex || "").toLowerCase();
  if (!anchored || localHash !== anchored) {
    return { allow: false, reason: "CARD_HASH_MISMATCH", localHash, anchoredHash: anchored };
  }
  return {
    allow: true,
    reason: "OK",
    receipt: {
      badgeOwner: record.owner,
      agentUri: record.agent_uri,
      cardHashHex: localHash,
      checkedAt: new Date().toISOString(),
    },
  };
}

// Full run, mirroring services/badge-check/src/cli.js.
async function liveRunGate(badgeId, claimedOwner) {
  const record = await liveFetchBadgeRecord(badgeId);
  let cardBytes = null;
  if (record?.agent_uri) {
    try {
      cardBytes = await liveFetchCardBytes(record.agent_uri);
    } catch {
      cardBytes = null;
    }
  }
  const verdict = await liveDecideGate({ record, cardBytes, claimedOwner: claimedOwner || undefined });
  return { record, verdict };
}

globalThis.BadgeGateLive = { liveFetchBadgeRecord, liveFetchCardBytes, liveSha256Hex, liveDecideGate, liveRunGate };
