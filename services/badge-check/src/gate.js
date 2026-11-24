// BadgeGate gate decision.
//
// Pure function. Given the registry record for a badge, the bytes of the
// Agent Card as served today, and the owner the claimant says it has,
// decide whether the agent may be hired. Every denial carries a reason
// code so the frontend and the audit trail can say exactly what failed.
//
// The card hash is always computed locally. We never trust a match flag
// returned by any endpoint: the whole point of the on chain SHA-256 anchor
// in CIS-8004 is that anyone can recompute it.

import { createHash } from "node:crypto";

export function sha256Hex(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

export const Reason = {
  OK: "OK",
  BADGE_NOT_FOUND: "BADGE_NOT_FOUND",
  AGENT_NOT_ACTIVE: "AGENT_NOT_ACTIVE",
  OWNER_MISMATCH: "OWNER_MISMATCH",
  CARD_UNREACHABLE: "CARD_UNREACHABLE",
  CARD_HASH_MISMATCH: "CARD_HASH_MISMATCH",
};

export function decideGate({ record, cardBytes, claimedOwner }) {
  if (!record) {
    return { allow: false, reason: Reason.BADGE_NOT_FOUND };
  }
  if (String(record.status).toLowerCase() !== "active") {
    return { allow: false, reason: Reason.AGENT_NOT_ACTIVE, status: record.status };
  }
  if (claimedOwner && record.owner && claimedOwner !== record.owner) {
    return { allow: false, reason: Reason.OWNER_MISMATCH, owner: record.owner };
  }
  if (!cardBytes) {
    return { allow: false, reason: Reason.CARD_UNREACHABLE, agentUri: record.agent_uri };
  }
  const localHash = sha256Hex(cardBytes);
  const anchored = String(record.metadata_hash_hex || "").toLowerCase();
  if (!anchored || localHash !== anchored) {
    return { allow: false, reason: Reason.CARD_HASH_MISMATCH, localHash, anchoredHash: anchored };
  }
  return {
    allow: true,
    reason: Reason.OK,
    receipt: {
      badgeOwner: record.owner,
      agentUri: record.agent_uri,
      cardHashHex: localHash,
      checkedAt: new Date().toISOString(),
    },
  };
}
