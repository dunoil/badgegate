// BadgeGate frontend demo logic.
//
// The two agent panels above run on recorded fixtures that mirror
// services/badge-check exactly, including the two live results verified
// on 24 Nov 2026: a real mainnet badge passed with a locally computed
// card hash matching the on chain anchor, and an invented badge was
// rejected with BADGE_NOT_FOUND. The panels stay fixtures on purpose.
// The live badge check panel further down the page (wired at the bottom
// of this file) runs the real gate in the browser through live-gate.js,
// a faithful port of the same service.

const FIXTURES = {
  copycat: {
    allow: false,
    reason: "BADGE_NOT_FOUND",
    text: "REJECTED. Reason: BADGE_NOT_FOUND. No job created, no escrow funded, no money moved.",
  },
  verified: {
    allow: true,
    reason: "OK",
    text: "HIRED. Registry status active, owner matched, card hash matched the on chain anchor. Escrow can now be funded.",
  },
};

const trail = document.getElementById("trail");

document.querySelectorAll("[data-check]").forEach((btn) => {
  btn.addEventListener("click", () => {
    const kind = btn.getAttribute("data-check");
    const f = FIXTURES[kind];
    const el = document.getElementById(`verdict-${kind}`);
    el.textContent = f.text;
    el.classList.toggle("ok", f.allow);
    el.classList.toggle("bad", !f.allow);
    const li = document.createElement("li");
    li.textContent = `GateChecked (${kind}): ${f.allow ? "PASS" : "FAIL"}, reason ${f.reason}, demo fixture.`;
    trail.appendChild(li);
    if (f.allow) {
      for (const step of ["JobCreated: escrow funded in test CCD (demo fixture).", "JobAccepted: provider accepted (demo fixture).", "JobDelivered: deliverable hash anchored (demo fixture).", "JobReleased: escrow paid to the provider (demo fixture)."]) {
        const row = document.createElement("li");
        row.textContent = step;
        trail.appendChild(row);
      }
    }
  });
});

// Live badge check panel. Uses BadgeGateLive from live-gate.js, the
// browser port of services/badge-check. The verdict and the reason code
// come from the live testnet registry at click time.
const liveBtn = document.getElementById("live-run");
if (liveBtn && globalThis.BadgeGateLive) {
  const liveVerdict = document.getElementById("live-verdict");
  const liveDetails = document.getElementById("live-details");
  liveBtn.addEventListener("click", async () => {
    const badgeId = document.getElementById("live-badge").value.trim();
    const claimedOwner = document.getElementById("live-owner").value.trim();
    liveBtn.disabled = true;
    liveVerdict.classList.remove("ok", "bad");
    liveVerdict.textContent = "Checking the live testnet registry...";
    try {
      const { record, verdict } = await globalThis.BadgeGateLive.liveRunGate(badgeId, claimedOwner);
      liveVerdict.textContent = verdict.allow
        ? `HIRED (live). Reason: OK. Registry status Active, owner matched, and the card SHA-256 computed in this browser matches the hash anchored on chain.`
        : `REJECTED (live). Reason: ${verdict.reason}. No job would be created and no money would move.`;
      liveVerdict.classList.toggle("ok", verdict.allow);
      liveVerdict.classList.toggle("bad", !verdict.allow);
      if (record) {
        document.getElementById("live-status").textContent = String(record.status ?? "");
        document.getElementById("live-owner-onchain").textContent = String(record.owner ?? "");
        document.getElementById("live-card-uri").textContent = String(record.agent_uri ?? "");
        document.getElementById("live-local-hash").textContent =
          verdict.receipt?.cardHashHex ?? verdict.localHash ?? "(card not fetched)";
        document.getElementById("live-anchored-hash").textContent = String(record.metadata_hash_hex ?? "");
        liveDetails.hidden = false;
      } else {
        liveDetails.hidden = true;
      }
    } catch (err) {
      liveVerdict.classList.remove("ok", "bad");
      liveVerdict.textContent = `Live check could not reach the registry endpoint: ${err.message}`;
      liveDetails.hidden = true;
    } finally {
      liveBtn.disabled = false;
    }
  });
}
