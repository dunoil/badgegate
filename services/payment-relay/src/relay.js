// BadgeGate payment relay: x402 settlement hand off.
//
// STATUS: explicit stub. The hackathon MVP settles through the escrow
// contract release on Concordium testnet, which is a real on chain
// payment. This module marks exactly where x402 plugs in for production
// per call settlement, based on the official Concordium description of
// x402 with sponsored transactions (Protocol 10):
//
//   1. The paying agent holds a stablecoin (a Protocol-Level Token) and
//      signs a transfer naming the facilitator as sponsor. The payload
//      fixes recipient and amount, so the signature cannot be reused.
//   2. The facilitator verifies the payload against the scheme rules,
//      co signs, pays the fee (about one euro cent) from its own CCD,
//      and submits. The agent never holds CCD.
//   3. The chain validates both signatures: stablecoin out of the agent
//      account, fee out of the facilitator account.
//
// OPEN QUESTION (project RESEARCH.md section 5): no public testnet x402
// facilitator endpoint for Concordium has been verified yet. When one is
// confirmed, implement settleViaX402 below against it. Do not point this
// at any endpoint that has not been verified from official docs.

export async function settleViaX402({ recipient, amount, asset }) {
  throw new Error(
    "x402 facilitator not integrated yet: no verified public Concordium testnet facilitator. " +
      "MVP settlement is the escrow release. See docs/path-to-production.md step 3."
  );
}

export function describeX402Flow() {
  return {
    rail: "x402 with sponsored transactions (Concordium Protocol 10)",
    steps: ["agent signs stablecoin transfer naming facilitator as sponsor", "facilitator verifies, co signs, pays CCD fee, submits", "chain validates both signatures"],
  };
}
