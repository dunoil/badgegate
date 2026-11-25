import { test } from "node:test";
import assert from "node:assert/strict";
import { decideGate, sha256Hex, Reason } from "../src/gate.js";
import { fetchBadgeRecord } from "../src/badgeCheck.js";

const card = new TextEncoder().encode(JSON.stringify({ name: "Demo Agent", type: "agent" }));
const OWNER = "3ABCownerAccountAddressExample";

// The REAL endpoint response shape, verified live on 24 and 25 Nov 2026:
// the owner arrives as on_chain.owner_account, never as owner. The old
// fixture invented an owner field on the record the gate reads, which is
// exactly why the suite passed while the production owner check was dead
// code (AUDIT.md F1). Every fixture below therefore starts from the raw
// endpoint body and goes through fetchBadgeRecord, like production does.
const rawGoodBody = {
  on_chain: {
    status: "Active",
    owner_account: OWNER,
    agent_uri: "https://example.com/agent/card.json",
    metadata_hash_hex: sha256Hex(card),
  },
};

function stubFetch(body, status = 200) {
  return async () => ({
    status,
    ok: status >= 200 && status < 300,
    json: async () => body,
  });
}

async function recordFromEndpoint(body = rawGoodBody) {
  return fetchBadgeRecord("TESTBADGE", { network: "testnet", fetchImpl: stubFetch(body) });
}

test("verified agent with an untouched card is allowed and gets a receipt", async () => {
  const record = await recordFromEndpoint();
  const v = decideGate({ record, cardBytes: card, claimedOwner: OWNER });
  assert.equal(v.allow, true);
  assert.equal(v.reason, Reason.OK);
  assert.equal(v.receipt.cardHashHex, rawGoodBody.on_chain.metadata_hash_hex);
});

test("unknown badge is rejected", async () => {
  const record = await fetchBadgeRecord("NOSUCHBADGE", {
    network: "testnet",
    fetchImpl: stubFetch({ error: "not found" }, 400),
  });
  assert.equal(record, null);
  const v = decideGate({ record, cardBytes: null, claimedOwner: OWNER });
  assert.equal(v.allow, false);
  assert.equal(v.reason, Reason.BADGE_NOT_FOUND);
});

test("copycat card with one edited endpoint fails the hash check", async () => {
  const record = await recordFromEndpoint();
  const tampered = new TextEncoder().encode(
    JSON.stringify({ name: "Demo Agent", type: "agent", endpoint: "https://evil.example.com" })
  );
  const v = decideGate({ record, cardBytes: tampered, claimedOwner: OWNER });
  assert.equal(v.allow, false);
  assert.equal(v.reason, Reason.CARD_HASH_MISMATCH);
});

test("inactive agent is rejected even with a good card", async () => {
  const record = await recordFromEndpoint({
    on_chain: { ...rawGoodBody.on_chain, status: "Revoked" },
  });
  const v = decideGate({ record, cardBytes: card, claimedOwner: OWNER });
  assert.equal(v.allow, false);
  assert.equal(v.reason, Reason.AGENT_NOT_ACTIVE);
});

test("claiming someone else's agent is rejected", async () => {
  const record = await recordFromEndpoint();
  const v = decideGate({ record, cardBytes: card, claimedOwner: "3XYZsomeoneElse" });
  assert.equal(v.allow, false);
  assert.equal(v.reason, Reason.OWNER_MISMATCH);
});

test("registered agent whose card is offline is rejected, not waved through", async () => {
  const record = await recordFromEndpoint();
  const v = decideGate({ record, cardBytes: null, claimedOwner: OWNER });
  assert.equal(v.allow, false);
  assert.equal(v.reason, Reason.CARD_UNREACHABLE);
});

// F1 regression tests. Before the fix, fetchBadgeRecord returned the raw
// on_chain object, gate.js read record.owner (always undefined), and both
// of these scenarios returned allow OK.

test("F1 regression: fetchBadgeRecord normalizes owner_account into the owner the gate reads", async () => {
  const record = await recordFromEndpoint();
  assert.equal(record.owner, OWNER);
  assert.equal(record.owner_account, OWNER);
});

test("F1 regression: a wrong claimed owner against the real endpoint shape yields OWNER_MISMATCH", async () => {
  const record = await recordFromEndpoint();
  const v = decideGate({ record, cardBytes: card, claimedOwner: "3ATTACKERwrongOwnerAccount" });
  assert.equal(v.allow, false);
  assert.equal(v.reason, Reason.OWNER_MISMATCH);
  assert.equal(v.receipt, undefined);
});

test("F1 regression: the receipt carries the verified owner from the record, not the claim", async () => {
  const record = await recordFromEndpoint();
  const v = decideGate({ record, cardBytes: card, claimedOwner: OWNER });
  assert.equal(v.allow, true);
  assert.equal(v.receipt.badgeOwner, OWNER);
});

test("F1 regression: a record with no owner at all fails closed when an owner is claimed", async () => {
  const record = await recordFromEndpoint({
    on_chain: { ...rawGoodBody.on_chain, owner_account: undefined },
  });
  assert.equal(record.owner, undefined);
  const v = decideGate({ record, cardBytes: card, claimedOwner: OWNER });
  assert.equal(v.allow, false);
  assert.equal(v.reason, Reason.OWNER_MISMATCH);
});
