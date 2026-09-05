# CodeFire Milestone Board

Status: live execution board

Owner: Orchestrator (Codex)

Last audited: 2026-09-05

Update trigger: exit evidence, blocker admission, user priority, or subscription capacity changes.

The user accepted the [recovery proposal](exploration/recovery-2026-09-05.md) on 2026-09-05:
Grok is the primary worker; Fable exhausts quickly and is reserved for important decisions.
Metered billing, purchased credits, and automatic top-ups are forbidden.
This board replaces the ecosystem sequence recorded at Git `83b4522`; historical `narrow`/`adopt`
results are preserved, not relabelled as human adoption or full stage completion.

## Active slots

| Slot | ID | Outcome | Exit | In-scope now |
|---|---|---|---|---|
| M1 | MS-recovery-baseline | A source-validated, subscription-only development baseline | Rust source gates and CLI smoke; coherent runtime routing; billing guard rejects unverified external execution; evidence identifies remaining limitations | Yes |
| M2 | MS-first-seal-usable | One useful request-to-seal-and-recovery path | Isolated real-task replay and one measured top-friction fix, independently reviewed | No |
| M3 | MS-fast-safe-change | Faster changes without integrity regressions | Focused invariant counterexamples, measured performance, and bounded module changes on the selected path | No |
| M4 | MS-repeat-use | Repeat use on another real change | Small human pilot with repeat use, intervention and friction recorded; no market-validation overclaim | No |
| M5 | MS-limited-distribution | A bounded installable product | Explicit support/rights/version, install/update/rollback rehearsal; publication remains a separate final action | No |

## M1 card

- Preserve the original main worktree and its pre-existing cherry-pick sequencer.
- Work in `/workspace/project/.codex-worktrees/recovery-20260905`, branch `recovery/subscription-first-20260905`.
- Rebuild current product source with a named Rust toolchain. Do not reuse historical binary results as current-source evidence.
- Repair development runtime validation to distinguish native Codex roles from external Claude/Grok profiles.
- Admit external workers only after subscription authentication and no extra-usage billing are verified. A blocked external lane must not trigger paid fallback.
- Use current-session Codex subagents for REC-02/REC-03 bootstrap while external billing remains unverified. This is explicit user-authorized recovery work, not an external API route.
- Do not consume Fable for routine setup or periodic status checking.

## Execution graph

Critical path: REC-02 baseline -> reviewed REC-07 bootstrap fix -> REC-08 Store replacement protection -> independent review and source gate -> M1 exit; REC-03 runtime repair and REC-04 billing guard run alongside.
Parallel now: REC-08 Store implementation and REC-01/05 documentation/evidence audit. Completed toolchain/runtime/billing changes are checkpointed; final source integration waits for Store repair.
External Grok: reverified via read-only ACP at 2026-09-05 11:28 UTC: weekly use 23%, purchased credits/PAYG cap/PAYG used all zero, auto-topup disabled. Built-in route has no API credential/config overrides. Recheck before and after each bounded job.

| Node | State | Owner | Write set / packet | Depends on |
|---|---|---|---|---|
| REC-01 | running | Codex orchestrator | board, AGENTS, governance/provider prose and provider matrix, history | — |
| REC-02 | complete | current-session Codex bootstrap worker | `rust-toolchain.toml`, `docs/development/tools/rust_env.sh`, `docs/development/ai/task-packets/recovery-rust-baseline.md`, ignored local toolchain/logs | — |
| REC-03 | complete | current-session Codex bootstrap worker | `.codex/`, role-routing matrix, external role profiles, runtime validator/launcher/audit prompt, focused tests; `ai/task-packets/recovery-runtime.md` | — |
| REC-04 | complete | native billing investigator | `grok_subscription_preflight.py`, dedicated tests, billing capability evidence; no model call | — |
| REC-05 | complete | fresh Grok/native independent reviewer sessions | bounded review of REC-02/03 diff, no implementation | REC-02, REC-03 |
| REC-07 | complete | Grok 4.6 | CFB-138: remote/project.rs and focused tests; packet recovery-bootstrap-budget.md | reproduced REC-02 failure |
| REC-08 | running | Grok 4.6 | CFB-139: Store recovery candidate code/tests; recovery-stage-ownership.md | Store failure reproduced |
| REC-06 | blocked | orchestrator | integration tests, checkpoint and board shift | REC-01..05, REC-07, REC-08 |

Packet completion records scoped implementation/review, not M1 exit. REC-02 identified the baseline failures; REC-03/04/05 passed independent reviews and the seven-group AI gate; REC-07 passed independent review and all CLI tests. REC-06 owns the final integrated source gate and remains blocked by REC-08.

## Validation

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --locked -- -D warnings
RUST_TEST_THREADS=4 cargo test --workspace --locked
cargo build --workspace --locked
./target/debug/codefire-rs --help
./target/debug/codefire-rs --version
python3 docs/development/tools/check_ai_development.py --timeout 60
git diff --check
```

Record actual environment and commands if Cargo requires an explicit local environment helper.
Source gate failure is a blocker, not a parked test. Setup-only changes do not authorize unrelated product fixes.
If a source failure needs a product change, admit its exact reproduction and write set before dispatching it.

## Parked work

- The five ecosystem milestones from `83b4522` are unscheduled until repeat use produces a real interoperability need.
- Existing CFB-133/136 and CFR-168..173 remain investigating; partial implementations must be checked before new work is admitted.
- Historical product-value `narrow`, undocumented JSON fields, and configured-glob limitations are inputs to M2.
- No broad rewrite, issue-portfolio burn-down, standardization, telemetry service, or agent scheduler inside the product.
- Human recruitment and external publication require their own explicit action authorization.

## Shift rule

Close only with evidence bound to the integrated source. Shift M2–M5 forward after the exit, choosing the next
bounded increment from observed user friction. A terminal experiment outcome does not automatically meet a strategic stage gate.

CFB-138 is explicitly admitted as the smallest blocker of the Rust source gate. Six deterministic bootstrap cleanup failures must be repaired before exit. The separate import full-suite lock timeout is being assessed under bounded test concurrency; single-test success alone does not close it.

REC-06 readmitted the remaining unverified external Codex `run_ai_role.py --execute` path: disable real execution before CLI/capability calls until subscription billing can be verified; preserve native-role dry-run. This does not stop the current host session.

The integrated test harness uses RUST_TEST_THREADS=4 to bound concurrent cases; test-internal threads/barriers are unchanged. The initial 14-way default run and 5s import timeout remain recorded, not relabelled as fixed.

REC-08/CFB-139 explicitly admitted: full gate now reaches Store and reproduces deletion of a replacement candidate. CFB-138 has reviewed code and all CLI/Core gates passed, but M1 remains open pending Store repair. Checkpoints: toolchain0e9c43a, workflow6226488, bootstrap3ca04d7.
