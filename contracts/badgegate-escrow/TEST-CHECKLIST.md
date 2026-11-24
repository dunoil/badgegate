# Escrow Test Checklist

Ported from the AgentSeal Foundry test scenarios (AgentSeal.t.sol, 507 lines, 28 tests passed on BNB testnet rehearsal) into the Rust and concordium test framework.

Implementation (24 Nov 2026, phase 2): the scenarios are implemented as integration tests in tests/tests.rs using the concordium-smart-contract-testing library against the compiled Wasm module. Mapping notes:
- "createJob with a receipt attested by a non checker account fails" is tested at the recordGate step itself (record_gate_by_non_checker_fails), which is the only way a receipt can come to exist, so it is the stronger form of the same check.
- Two scenarios were added beyond the original list: a receipt cannot be used twice (receipt_cannot_be_used_twice), and the checker allowlist view (checker_view_reports_allowlist).
- The pause scenario is fully covered as of the phase 3 hardening pass (24 Nov 2026): three tests were added, paused_deliver_still_works, paused_claim_still_works, and paused_cancel_still_works, alongside the existing paused_blocks_new_jobs_but_not_existing_job_flows (create blocked, release works). Pause is only read in createJob in the contract code (single check in lib.rs), which matches the test results.
- The final item is an interface review, not a runtime test: the compiled module was inspected on 24 Nov 2026 (entrypoint names extracted from module.wasm.v1). The module exposes exactly init plus recordGate, createJob, acceptJob, deliverJob, releaseJob, claimAfterWindow, cancelJob, rateJob, setJobCreationPaused, getJob, jobCount, getProviderStats, and isChecker. No withdrawal, sweep, rescue, or drain entrypoint exists. Interface review PASSED.

Results (24 Nov 2026, phase 2): 25 integration tests run with cargo test against module.wasm.v1, 25 passed, 0 failed (log in test-run2.log at the project root). 26 of 27 checklist items are verified. The one open item is partial: release while paused is verified, while claim, cancel, and deliver while paused share the same code path but are not yet separately exercised; they are scheduled in the hardening pass.

Results (24 Nov 2026, phase 3, hardening): 28 integration tests run with cargo test against module.wasm.v1, 28 passed, 0 failed (log in test-run-phase3.log at the project root). All 27 of 27 checklist items are now verified, including the completed pause item below. Badge check service tests re run the same day: 6 of 6 pass (log in badge-test-phase3.log at the project root).

Results (24 Nov 2026, phase 4, contract read through): three small pre deploy fixes were made (see the phase 4 section in STATUS.md): the misleading error name DeadlineInFuture became DeadlineInPast (same enum position, same serialised ordinal), the JobCreated event now also carries deadline_ms and description_uri so the audit trail row is fully sourced from the event, and recordGate now rejects an empty badge id and a zero card hash (two new error variants appended at the end of the enum, ordinals of all existing variants unchanged, plus two new tests: record_gate_with_empty_badge_fails, record_gate_with_zero_card_hash_fails). The module was rebuilt (module.wasm.v1, 56,787 bytes) and the full suite re run: 30 of 30 integration tests pass, 0 failed (log in test-run-phase4.log at the project root). Badge check service tests re run: 6 of 6 pass (log in badge-test-phase4.log). The interface review was repeated against the rebuilt module: entrypoints are exactly init plus recordGate, createJob, acceptJob, deliverJob, releaseJob, claimAfterWindow, cancelJob, rateJob, setJobCreationPaused, getJob, jobCount, getProviderStats, and isChecker. No withdrawal, sweep, rescue, or drain entrypoint or string exists in the module. The 27 checklist items above are unchanged and remain verified; the two new tests are hardening beyond the checklist, so the suite stands at 30 tests against 27 checklist items.

## Gate

- [x] createJob without a recorded gate receipt fails
- [x] createJob with a receipt attested by a non checker account fails
- [x] createJob with a valid checker attested receipt succeeds and stores badge id, owner, and card hash on the job
- [x] A receipt for agent A cannot be used to hire agent B (owner mismatch fails)

## Lifecycle

- [x] createJob escrows the exact CCD amount and starts in state Open
- [x] acceptJob by anyone other than the named provider fails
- [x] acceptJob after the deadline fails
- [x] deliverJob before acceptance fails
- [x] deliverJob with an empty deliverable hash fails
- [x] deliverJob anchors the hash and moves the job to Delivered
- [x] releaseJob by anyone other than the client fails
- [x] releaseJob pays the provider the full escrow and moves stats (jobs settled, volume)
- [x] releaseJob twice fails the second time
- [x] claimAfterWindow before delivery fails
- [x] claimAfterWindow before 3 days after delivery fails
- [x] claimAfterWindow after 3 days pays the provider and settles stats
- [x] cancelJob before acceptance refunds the client in full
- [x] cancelJob after the deadline with no delivery refunds the client in full
- [x] cancelJob after delivery fails
- [x] rateJob before settlement fails
- [x] rateJob by anyone other than the paying client fails
- [x] rateJob with 0 or 6 fails
- [x] rateJob twice fails the second time
- [x] rateJob after settlement updates rating sum and count exactly once

## Owner powers

- [x] setJobCreationPaused by a non owner fails
- [x] While paused, createJob fails but release, claim, cancel, and deliver on existing jobs still work (COMPLETED 24 Nov 2026, phase 3: release verified in phase 2; deliver, claim, and cancel while paused verified by the three added tests paused_deliver_still_works, paused_claim_still_works, paused_cancel_still_works, all passing in the 28 test run)
- [x] The contract exposes no withdrawal or sweep entrypoint (interface review, recorded in the PR notes)
