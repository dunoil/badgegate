// BadgeGate badge check client.
//
// Talks to the Concordium Agent Registry badge check endpoint, which is
// backed by the registry indexer and the CIS-8004 contract. The same data
// is exposed to agents through the hosted Agent Registry MCP server at
// https://agent-registry-mcp.concordium.com/mcp, which is how an agent
// checks another agent before dealing with it.
//
// Endpoint shape (verified live against the real endpoint on 24 and
// 25 Nov 2026, see project RESEARCH.md section 9 and AUDIT.md finding F1):
//   GET {base}/v1/badge-check/{badgeId}?network={mainnet|testnet}
//   -> { on_chain: { status, agent_uri, metadata_hash_hex,
//                    owner_account, ... } }
// The owner field is named owner_account, NOT owner. An earlier version of
// this client returned the raw on_chain object and the gate read
// record.owner, which was always undefined, so the owner check silently
// never ran (AUDIT.md F1). fetchBadgeRecord therefore normalizes the
// record here: the returned object always carries the verified owner in
// the `owner` field the gate reads, sourced from owner_account (falling
// back to a plain owner field only if a future endpoint renames it back).

export const DEFAULT_BASE = "https://agent-registry-mcp.concordium.com";

export async function fetchBadgeRecord(badgeId, { network = "mainnet", base = DEFAULT_BASE, fetchImpl = fetch } = {}) {
  const url = `${base}/v1/badge-check/${encodeURIComponent(badgeId)}?network=${encodeURIComponent(network)}`;
  const res = await fetchImpl(url);
  // Verified live on 24 Nov 2026: an unknown or malformed badge id gets a
  // 400 from this endpoint, not a 404. Both mean "no such registration".
  if (res.status === 404 || res.status === 400) return null;
  if (!res.ok) throw new Error(`badge check endpoint returned ${res.status}`);
  const body = await res.json();
  const onChain = body.on_chain ?? body;
  return { ...onChain, owner: onChain.owner_account ?? onChain.owner };
}

export async function fetchCardBytes(agentUri, { fetchImpl = fetch } = {}) {
  const res = await fetchImpl(agentUri);
  if (!res.ok) throw new Error(`agent card fetch returned ${res.status}`);
  return new Uint8Array(await res.arrayBuffer());
}
