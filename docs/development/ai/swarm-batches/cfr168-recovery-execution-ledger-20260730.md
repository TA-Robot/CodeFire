# CFR-168 / CFB-136 Recovery Execution Ledger

This is the live execution ledger for the frozen first batch after the 2026-07-29 host restart.
Canonical issue status remains `investigating`; this ledger does not authorize Close.

## Authority and recovery boundary

- Immutable audited product snapshot: `e2a218e015953f082245f36dbc6aee906b837e5a`.
- Canonical integration branch: `integration/cfr168-p0-canonical`.
- The host restarted at 2026-07-29 23:31 UTC. All old manager PIDs became stale while their
  worktrees and Git objects remained intact.
- No old task was classified as a product failure merely because its process disappeared.
- Every non-empty interrupted worktree was first committed on its own agent branch. Recovery tasks
  start from those commits and do not mutate the dirty main worktree.
- External implementation models are fixed to Claude Opus 5 (`claude-opus-5`) and Fugu 1.1 Ultra
  (`fugu-ultra-v1.1`). No external Codex implementation lane is active.
- Recovery task timeout is eight hours.

## Active recovery lanes

| Surface | Recovery base | Provider | New batch | State at restart audit | Required finish |
|---|---|---|---|---|---|
| Python HTTP Doctor | `d79be3e` | Claude Opus 5 | `cfr168-python-http-doctor-canonical-recovery-20260729` | canonical WIP; focused 16/16 pass | corrupted-object direct/HTTP regression, full suite, clean commit |
| Python HTTP upload ancestry/signature | `3153d75` | Claude Opus 5 | `cfr168-python-http-upload-recovery-20260729` | canonical WIP; focused 7/10 pass | repair three reproduced failures, full suite, clean commit |
| Python HTTP branch session | `8265ea1` | Claude Opus 5 | `cfr168-python-http-branch-recovery-20260729` | canonical WIP; syntax/diff checks pass, dedicated test absent | production tests, exact boundaries, full suite, clean commit |
| Python HTTP MR create/review/apply | `cab3126` | Claude Opus 5 | `cfr168-python-http-mr-mutation-recovery-20260729` | task packet only | one request ledger/deadline, pre-effect response owner, no-effect tests |
| Rust storage | `40c095d` | Fugu 1.1 Ultra | `cfr168-rust-storage-recovery-20260729` | interrupted owner skeleton; 9 compile errors and 2 warnings reproduced after toolchain recovery | complete or replace skeleton, focused/broad tests, strict Clippy |
| Rust file-remote ancestry | `fe54e60` | Fugu 1.1 Ultra | `cfr168-rust-file-remote-ancestry-recovery-20260729` | task packet only | live file upload/apply adoption, final fence, bounded traversal tests |
| Rust file-remote Doctor | `ea8f2a6` | Fugu 1.1 Ultra | `cfr168-rust-file-doctor-recovery-20260729` | task packet only | unify live file command with accounted Doctor path |
| Rust HTTP MR create/review preflight | `3c23e38` | Fugu 1.1 Ultra | `cfr168-rust-http-mr-preflight-20260729` | task packet only | prepared response before effect and no-effect fault tests |

## Serialized integration order

Product commits are reviewed and integrated, never entire agent worktrees:

1. Python Doctor, then upload, then branch-session changes.
2. Resolve the adjacent `do_GET` hunks while preserving one exact request ledger and deadline.
3. Integrate Python MR mutation after upload response-owner invariants are present.
4. Integrate non-overlapping Rust `remote.rs`, `remote/doctor.rs`, and `http.rs` changes.
5. Port the accepted Rust storage commit from its probe base onto the canonical branch; do not merge
   the rejected Python candidate line or unrelated probe history.
6. Run focused tests after every product commit and broad gates after each runtime group.

## Predecessor-gated residual packets

These are still part of CFR-168 / CFB-136 but deliberately wait for a reusable predecessor or
overlapping write set:

| Residual | Predecessor |
|---|---|
| Python `GET …/merge-requests` request ledger/session | Python branch shared-session result |
| Python `POST …/gc` request ledger/deadline and pre-effect response | Python upload/MR mutation response-owner result |
| Rust local `cf merge` ancestry | Rust storage task releases overlapping `main.rs` write set |
| Rust `cf scan` validation ledger/session | Rust storage task releases overlapping `main.rs`/metrics write set |
| cache `invalidated`/entry-count metrics and command-wide decision-work projection | all selected production surfaces integrated |
| asymptotic/order table and cross-runtime corpus | stable final counters and production call graph |
| capability re-advertisement | every surface/schema/conformance gate passes |

Later roots CFR-169 through CFR-173 and CFB-133 remain predecessor-gated. Their probes are evidence,
not authorization to violate the frozen-batch-to-zero rule.

## Mandatory acceptance sequence

The batch cannot Close until all of the following are bound to the exact integrated revision:

1. focused production-path tests and exact/minus-one/plus-one boundaries;
2. 1x/3x/10x stable counter curves and explicit complexity/order evidence;
3. full Python, Rust workspace, format, strict Clippy, demo, source docs, and project docs;
4. Rust/Python shared conformance corpus and independent High review;
5. clean-HEAD release install with provenance;
6. installed reversal tests and canonical Algorithm Dogfood;
7. issue/evidence/history synchronization and Retrospect;
8. a final ledger audit proving no CFR-168 / CFB-136 residual remains.

## Evidence

- `docs/development/ai/swarm-results/current-state-closure-audit-20260729/01-issue-ledger.md`
- `docs/development/ai/swarm-results/current-state-closure-audit-20260729/02-candidate-reconciliation.md`
- `docs/development/ai/swarm-results/current-state-closure-audit-20260729/03-acceptance-evidence.md`
- `docs/development/ai/swarm-results/current-state-closure-audit-20260729/04-cfr168-review.md`
- `docs/development/ai/swarm-results/claude-cfr168-review-20260729/review.md`
