import { test } from "node:test";
import assert from "node:assert/strict";
import { decideGate, sha256Hex, Reason } from "../src/gate.js";

const card = new TextEncoder().encode(JSON.stringify({ name: "Demo Agent", type: "agent" }));
const goodRecord = {
  status: "Active",
  owner: "3ABCownerAccountAddressExample",
  agent_uri: "https://example.com/agent/card.json",
  metadata_hash_hex: sha256Hex(card),
};

test("verified agent with an untouched card is allowed and gets a receipt", () => {
  const v = decideGate({ record: goodRecord, cardBytes: card, claimedOwner: goodRecord.owner });
  assert.equal(v.allow, true);
  assert.equal(v.reason, Reason.OK);
  assert.equal(v.receipt.cardHashHex, goodRecord.metadata_hash_hex);
});

test("unknown badge is rejected", () => {
  const v = decideGate({ record: null, cardBytes: null });
  assert.equal(v.allow, false);
  assert.equal(v.reason, Reason.BADGE_NOT_FOUND);
});

test("copycat card with one edited endpoint fails the hash check", () => {
  const tampered = new TextEncoder().encode(
    JSON.stringify({ name: "Demo Agent", type: "agent", endpoint: "https://evil.example.com" })
  );
  const v = decideGate({ record: goodRecord, cardBytes: tampered, claimedOwner: goodRecord.owner });
  assert.equal(v.allow, false);
  assert.equal(v.reason, Reason.CARD_HASH_MISMATCH);
});

test("inactive agent is rejected even with a good card", () => {
  const v = decideGate({ record: { ...goodRecord, status: "Revoked" }, cardBytes: card });
  assert.equal(v.allow, false);
  assert.equal(v.reason, Reason.AGENT_NOT_ACTIVE);
});

test("claiming someone else's agent is rejected", () => {
  const v = decideGate({ record: goodRecord, cardBytes: card, claimedOwner: "3XYZsomeoneElse" });
  assert.equal(v.allow, false);
  assert.equal(v.reason, Reason.OWNER_MISMATCH);
});

test("registered agent whose card is offline is rejected, not waved through", () => {
  const v = decideGate({ record: goodRecord, cardBytes: null });
  assert.equal(v.allow, false);
  assert.equal(v.reason, Reason.CARD_UNREACHABLE);
});
