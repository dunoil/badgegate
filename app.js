// BadgeGate frontend demo logic.
//
// Demo mode verdicts mirror services/badge-check exactly, including the
// two live results verified on 24 Nov 2026: a real mainnet badge passed
// with a locally computed card hash matching the on chain anchor, and an
// invented badge was rejected with BADGE_NOT_FOUND. Live mode (build
// task, 30 Nov) replaces these fixtures with calls to the badge check
// service. Demo fixtures are labelled demo in the page header.

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
