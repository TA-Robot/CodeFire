# CFR-168 Rust File Ancestry Claude Candidate Independent Review (Claude fallback)

Review date: 2026-07-30

Reviewer: Claude Opus 5 (`claude-opus-5`), fallback for the interrupted Fugu 1.1 Ultra review session

Review worktree base: `9f1fe186df3002198a184fdced4691123037d71f`

Review contract: `docs/development/ai/swarm-batches/cfr168-rust-file-ancestry-claude-independent-review-20260730.md`

Fallback ticket: `docs/development/ai/swarm-batches/cfr168-rust-file-ancestry-review-claude-fallback-20260730.md`

Implementation contract: `docs/development/ai/swarm-batches/cfr168-rust-file-remote-ancestry-20260729.md`

**Verdict: `REQUEST_CHANGES`.**

Both provisional blockers identified before the Fugu session ended are **reproduced**, not rejected.
An unaccounted predecessor full traversal remains on all three live routes, and a pre-decision
object effect remains on the live upload publish route. Review contract item 6 makes either of
these `REQUEST_CHANGES` on its own, "even if the new nucleus itself is correct" — and the new
nucleus *is* correct. The accounted decision itself is well-built, surgical, and green; the blockers
are contract-scope and evidence-sufficiency defects, not defects in the decision.

This verdict was reached from the exact target's source and from independently executed commands.
It does not rest on the 778/778 figure and does not rest on the interrupted Fugu log.

## Exact subject

| Role | Revision | Disposition |
|---|---|---|
| Baseline / task revision | `fe54e60d22dfd081bf271593ba794e4f567ed915` | comparison base |
| Completed Claude candidate (exact target) | `e93ebc2394c8bd0398cfc23e8bf2d23d52e253b9` | reviewed |
| Contract snapshot named by the implementation ticket | `e2a218e015953f082245f36dbc6aee906b837e5a` | referenced only |

Target branch:
`claude-agent/cfr168-rust-file-remote-ancestry-claude-hedge-20260730-file-remote-ancestry-hedge`

Target commit subject: `feat(remote): account file-remote fast-forward ancestry decisions`

The candidate is a single commit directly on the baseline:

```text
fe54e60d22dfd081bf271593ba794e4f567ed915 -> e93ebc2394c8bd0398cfc23e8bf2d23d52e253b9
```

Diff shape, verified: exactly one file, `crates/codefire-cli/src/remote.rs`, `+1751 / -3`. The three
removed lines are precisely the three `is_ancestor_in_objects` production call sites. No test file,
management ledger, issue record, or other crate is touched, so no public limit could have been
raised — `DecisionWorkProfile::local_v1` in `crates/codefire-core/src/work_budget.rs` is byte-identical
to the baseline.

The accepted predecessor nucleus is byte-identical: baseline `remote.rs:3729-4101` equals target
`remote.rs:4436-4808` (`diff` reports no difference), covering
`is_ancestor_in_provisional_subject_with_poll` and everything after it in the baseline file. All new
code is inserted between the legacy helper and that nucleus (`remote.rs:3736-4435`), plus an appended
test module (`remote.rs:4811-5849`).

The original Fugu candidate for the same contract remains separate and is **not** judged here.

## Findings, severity ordered

### `MUST-1` — a pre-decision object effect remains on the live upload publish route, and the no-effect evidence compares against an already-mutated reference

`copy_object_graph` writes the local object closure into the remote object store at
`crates/codefire-cli/src/remote.rs:346`. The `UploadPublish` ancestry decision does not run until
`remote.rs:377`. The ordering is unchanged from the baseline, but the *set of failures reachable at
that point is not*: the baseline could only fail there with the accepted non-fast-forward
`CliError::Usage` or a genuine store error, whereas the accounted decision adds three new typed
rejection classes — `CliError::ResourceBudget`, `CliError::RequestDeadlineExceeded`, and
`StoreError::SealedValidationSubjectChanged`.

Every one of those new rejections therefore lands *after* the rejected head's objects have been
written into the remote object store, which directly contradicts the implementation contract clause
"Limit/deadline/final-fence failure must be typed, release only this operation's ownership, and
leave branch, generation, merge-request, **object**, idempotency, and publication state unchanged."

The candidate's own evidence concedes this rather than refuting it. In
`live_publish_failures_leave_authority_state_and_publication_untouched`:

- `remote.rs:5785` takes the reference snapshot with `include_objects = false`;
- `remote.rs:5787-5789` runs a non-fast-forward upload whose in-source comment states it "copies the
  source object graph and then refuses to publish";
- `remote.rs:5791` captures `objects_after_rejection` from that already-mutated state;
- `remote.rs:5815-5819` asserts each injected failure leaves "exactly the object state the accepted
  rejection leaves", i.e. it compares mutated against mutated.

Review contract item 4 asks specifically whether that comparison is sufficient evidence. It is not.
It proves the new failures are no worse than the pre-existing rejection; it cannot prove the
contract clause, because the clause is about object state, and object state has already changed.
The `authority_snapshot(project_root, include_objects)` flag exists only to let this assertion pass.

Two aggravating details:

1. The exhausted-budget case at `remote.rs:5823-5830` asserts nothing at all about object state — it
   checks only `authority_snapshot(&project_root, false)`. So even the weaker mutated-vs-mutated
   comparison is absent for `ResourceBudget` on the publish route.
2. `FileRemoteAncestryStage::PreEffect` is documented at `remote.rs:3796` as "After the final
   fence and before the caller's upload/apply effect". On the publish route that name is inaccurate
   in both directions: the object effect already happened at `remote.rs:346`, and after the fence
   returns the caller still performs a generation read and `execute_json_publication`.

**Failure scenario.** `codefire upload` of a diverged branch against a remote whose ancestry closure
exceeds the `GraphNodes` budget: the command writes the full source object graph into the remote
object store, then returns a `ResourceBudget` error whose contract promises object state is
unchanged. The operator sees a typed "nothing happened" rejection while unreferenced foreign objects
have been permanently injected into the remote store, recoverable only through GC.

Required resolution: fence the object effect behind the decision (move `copy_object_graph` after the
`UploadPublish` decision, or stage it so it is discardable), and replace the mutated-vs-mutated
assertion with a genuine pre-command object snapshot.

### `MUST-2` — the ledger is decision-scoped, not command-scoped; duplicate unaccounted full traversals remain on every live route

The implementation contract requires "one command-scoped `DecisionWorkLedger` before the selected
file-remote decision work" and forbids "second full traversal". The candidate creates the ledger
*inside* `is_file_remote_fast_forward` (`remote.rs:3939-3960`), so it is scoped to the
decision, and the label `FileRemoteAncestryCommand::label()` names a command without the ledger ever
spanning one. Everything the command does around the decision stays outside any ledger:

| Live route | Unaccounted work still on the route | Decision site |
|---|---|---|
| upload dry-run | `validate_sealed_commit(&local_objects, &head)` at `remote.rs:178`; `collect_object_records(&local_objects, &head)` at `remote.rs:259` | `remote.rs:311` |
| upload publish | the same two, plus `copy_object_graph` at `remote.rs:346` | `remote.rs:377` |
| merge-request apply | `validate_sealed_commit(source_objects, &source_head)` at `remote.rs:2382`; `validate_sealed_commit(&target_objects, &target_head)` at `remote.rs:2383` | `remote.rs:2389` |

Review contract item 3 asks whether the residual duplicate `codefire_store::validate_sealed_commit`
calls violate the frozen one-decision-ledger contract or require composition with another accepted
predecessor. The answer is **both**, and the duplication is exact, not approximate:

- `validate_sealed_commit` performs a complete iterative walk of the commit's whole ancestor closure
  (`crates/codefire-store/src/lib.rs:9203-9262`, driving `validate_sealed_commit_inner` over an
  explicit `SealedCommitValidationFrame` stack).
- The accounted decision then validates *the same commit in the same object root a second time*, via
  `session.validate_commit_with_cancellation(&subject.commit_id, ...)` at `remote.rs:4039-4040`.
- `remote.rs:178` validates `head` in `local_objects`; the upload decisions validate
  `commit_id == head` in `local_objects`. Exact duplicate.
- `remote.rs:2382` validates `source_head`; the apply decision validates
  `commit_id == source_head` in the same root. Exact duplicate.
- `remote.rs:2383` (target head) is *not* duplicated — the decision validates only `commit_id`, never
  the `ancestor` argument. Worth recording as an asymmetry, not a defect.

Each live route therefore performs two full closure validations of the identical subject, one of them
outside every ledger, plus (on upload) two more unaccounted full graph passes at `remote.rs:259` and
`remote.rs:346`. Preserving `remote.rs:178` and `remote.rs:2382` for error precedence is a legitimate
constraint — removing them would move corrupt-head diagnostics behind the registry-state and
signature-policy checks — but the correct discharge of that constraint is to *account* them, not to
leave them outside the ledger.

Composition with an accepted predecessor is available and is already the house pattern. The same
module exposes `collect_object_records_with_owned_work_and_poll` (`remote.rs:3065`) and the
`_with_work_and_poll` / `_without_work_and_poll` split (`remote.rs:732`, `remote.rs:753`,
`remote.rs:977`, `remote.rs:1260`). The HTTP transport's live routes already thread one
request-scoped ledger through sealed validation *and* object collection under a shared deadline
(`crates/codefire-cli/src/http.rs:6100-6141` and `http.rs:8571`). The file transport is now the only
live route that validates and collects unaccounted while claiming a command-scoped ledger.

### `SHOULD-1` — the deadline fence cannot fire in a production build, so the four-fence deadline evidence is vacuous with respect to shipped behavior

`DecisionWorkProfile::local_v1()` sets `request_deadline_ms: None`
(`crates/codefire-core/src/work_budget.rs:287`). Consequently, in `is_file_remote_fast_forward`,
`limit_ms` is `0` (`remote.rs:3946`), and `FileRemoteAncestryDeadline::new` under
`#[cfg(not(test))]` hard-codes `expire_at: None` (`remote.rs:3826-3833`). `is_expired()` compares
`None == Some(stage)`, which is always false, so every `deadline.check()` in the decision is
statically unreachable-failing in a shipped binary, and the `RequestDeadlineExceeded { limit_ms: 0 }`
diagnostic can never be emitted.

`live_decision_reports_a_typed_deadline_at_every_stage_without_mutating` (`remote.rs:5674-5711`)
therefore exercises a `#[cfg(test)]`-only constructor (`remote.rs:3835-3841`), not production
behavior. The candidate documents the reasoning honestly at `remote.rs:3812-3817` ("the file
transport bounds this decision with ledger limits instead of wall-clock time"), and the
implementation contract does demand deadline tests — so this is a design/evidence gap to resolve,
not concealment. It must not be reported to the manager as production deadline coverage.

### `SHOULD-2` — required fixture and dimension coverage is incomplete

Review contract item 2 requires stable exact counters on **chain, diamond, cycle, missing, and
corrupt** fixtures. Grepping the whole new test module (`remote.rs:4813-5849`) for `cycle`,
`missing`, `dangling`, and `corrupt` returns nothing.

- chain: covered exactly (`remote.rs:5173-5227`, lengths 8/24/80; `remote.rs:5229-5273`, length 600).
- diamond: covered exactly (`remote.rs:5275-5338`, `Revisits == 1`, `UniqueNodesDiscovered == 4`).
- cycle: absent. Plausibly unconstructible under content addressing, but the candidate neither
  states nor tests that.
- missing / dangling parent: absent. Presumably rejected at the subject-validation fence before
  traversal, but no test demonstrates the fence, the typed error, or the counters at that point.
- corrupt record: absent as a *traversal* fixture. `final_fence_rejects_a_same_metadata_content_rewrite`
  covers subject *change*, which is a different property.

Separately, item 5's exact/minus-one/plus-one matrix omits two dimensions the implementation contract
names explicitly ("logical depth/frontier"). `precharge_injected_file_remote_ancestry_budget`
refuses maximum-measurement dimensions by design (`remote.rs:4385-4390`), and
`CUMULATIVE_DIMENSIONS` / `CONCURRENT_DIMENSIONS` (`remote.rs:4824-4836`) accordingly exclude
`LogicalDepth` and `MaximumFrontier`. Those dimensions are charged and observable — the 600-depth
test asserts `maximum(LogicalDepth) == 599` and `maximum(MaximumFrontier) == 600` — but their limit
boundaries are never enforced by a test.

### `SHOULD-3` — the "goal-directed, no full materialization" claim holds only at query-phase scope, and the counters proving it never reach a production caller

The candidate's own 600-depth assertions state the position plainly at `remote.rs:5247-5250`:
"Pinning the subject already walks the full 600-commit closure, so it owns both logical maxima", with
`FilesRead == 0` and `RecordReads == 0` in `Phase::Query` (`remote.rs:5266-5273`). So the decision as
a whole still performs a complete closure walk and retains every reachable commit document; only the
boolean query on top of that materialized subject is goal-directed.

This is a real improvement over the baseline — the baseline walked the closure twice, once in
`validate_sealed_commit` and once in `ancestor_distances` (`crates/codefire-cli/src/main.rs:10767-10793`),
whereas the target walks it once and answers from retained documents. But the headline claim asserted
at `remote.rs:5224-5225` — "the decision exits at its goal instead of materializing every reachable
distance", backed by `near == [(8,2,1,2), (24,2,1,2), (80,2,1,2)]` — is `Phase::Query`-scoped. Total
decision work still scales 1x/3x/10x with history size. Retained *bytes* may also now be higher than
the baseline's distance map, since the session retains full documents rather than ids and distances.

Compounding this: the three tests carrying the goal-directed evidence
(`accounted_decision_agrees_with_the_legacy_materializing_helper`,
`traversal_work_is_goal_directed_and_linear_at_1x_3x_and_10x`,
`deep_linear_history_and_shared_dag_traverse_once_without_duplicate_reads`) call
`is_file_remote_fast_forward` directly. They are the exact three tests that still pass when the
production call sites are reverted (see the mutation experiment below), i.e. the counter evidence
item 5 requires to "reach production callers" does not.

### `SHOULD-4` — the effect-bearing routes are the least covered by failure injection

- `final_fence_rejects_a_same_metadata_content_rewrite` (`remote.rs:5714-5771`) runs
  `fixture.upload("descendant", "main", true)` — dry run. The same-metadata tamper is never injected
  on a route that has already produced an effect, which is exactly the route where the final fence
  matters.
- `live_decision_reports_a_typed_deadline_at_every_stage_without_mutating` is likewise dry-run only;
  the publish test injects three of the five stages and one budget dimension.
- `FileRemoteAncestryCommand::ApplyMergeRequest` has **no** failure injection at all. Its two tests
  (`remote.rs:5445-5530`, `remote.rs:5532-5597`) cover success and non-fast-forward rejection only —
  no budget, no deadline, no tamper, despite apply being a publication route.

### `COULD-1` — two ledger-loss paths drop the ledger without a terminal reason and record no observation

`discard_file_remote_ancestry_session` returns `(Err(error), None)` on
`Ok(None) | Err(_)` (`remote.rs:4127-4139`), and the `into_prepublication_validation_proof` failure
path returns `None` when both cancellation attempts fail (`remote.rs:4055-4068`). On those paths
`is_file_remote_fast_forward` returns at `remote.rs:3962-3968` without reaching
`close_file_remote_ancestry_ledger`, so the ledger is dropped un-finished. `DecisionWorkLedger` has
no `Drop` implementation, so this is silent: no terminal reason, no
`record_file_remote_ancestry_observation`, and no possible test assertion. No injection can reach
these paths, so they are untested. The stated invariant "the decision refuses to return unless its
ledger owns nothing" does not hold on them.

### `COULD-2` — residual-ownership error masks the typed outcome error

In `close_file_remote_ancestry_ledger` the terminal reason prefers the outcome error
(`file_remote_ancestry_terminal_reason(outcome.as_ref().err().or(ownership.as_ref().err()))`,
`remote.rs:4201-4204`) but the returned value prefers the ownership error (`ownership?;` before
`outcome`, `remote.rs:4209-4210`). If an internal invariant violation coincides with a typed
limit/deadline/tamper failure, the caller sees an `InvalidRepository` string while the ledger records
the typed reason. Diagnostics-only, and only on an already-broken invariant.

## Verdicts on all six required-review items

| # | Item | Verdict |
|---|---|---|
| 1 | Each of the three decisions owns one ledger/session end to end | **PASS**, with `COULD-1` attached |
| 2 | Owner reservation, iterative determinism, single visit, goal stop, exact counters on five fixture classes | **FAIL** (fixture coverage) |
| 3 | Residual duplicate `validate_sealed_commit` / unaccounted predecessor traversal | **FAIL** — `MUST-2` |
| 4 | Live upload ordering, pre-decision effect, sufficiency of the mutated-reference comparison | **FAIL** — `MUST-1` |
| 5 | Test matrix reaches production callers, non-vacuous, no public limit raised | **PARTIAL FAIL** |
| 6 | Independent validation runs, and the unaccounted-predecessor / pre-decision-effect rule | **FAIL** — rule triggered twice |

**Item 1 — `PASS`.** Each of the three decisions constructs exactly one `DecisionWorkLedger`
(`remote.rs:3947-3952`) labelled by its own `FileRemoteAncestryCommand` (`remote.rs:3771-3777`) and
exactly one `SealedCommitValidationSession` (`remote.rs:4025-4031`), which takes ownership of the
ledger rather than nesting a second one. The ordered stages Entry → SubjectValidation → Traversal →
FinalRevalidation → PreEffect are all reached (`remote.rs:3787-3810`, `remote.rs:3989-3990`,
`remote.rs:4020`, `remote.rs:4071`, `remote.rs:4094`, `remote.rs:4098`); the retained subject scalar
is charged *before* the two `to_string()` allocations it pays for (`remote.rs:3880-3910`);
terminal-reason selection is exhaustive over budget, deadline, subject-change, and corrupt-subject
(`remote.rs:4154-4176`); and `close_file_remote_ancestry_ledger` refuses to return unless
`snapshot().current` is all zero and `open_permit_count() == 0` (`remote.rs:4178-4211`). Tests assert
one observation per command invocation (`remote.rs:5052-5059`) and distinct ledger identities between
the dry-run and publish decisions (`remote.rs:5376-5379`). The qualification is scope, not identity:
"command-scoped" in the implementation contract is not satisfied by a decision-scoped ledger — that
is `MUST-2`, not a defect in item 1's own claim.

**Item 2 — `FAIL`.** Reservation-before-allocation, iterative traversal, and single-visit
determinism all hold: the accepted nucleus is byte-identical, the store's validation walk uses an
explicit frame stack rather than recursion (`crates/codefire-store/src/lib.rs:9528-9540`), and the
600-commit fixture resolves without stack growth. Counters are exact equalities, not
nonzero-liveness assertions — `far == [(8,8,7,8,0), (24,24,23,24,0), (80,80,79,80,0)]` and
`near == [(8,2,1,2), (24,2,1,2), (80,2,1,2)]` (`remote.rs:5219-5225`), diamond `Revisits == 1`
(`remote.rs:5305-5308`). The failure is coverage: cycle, missing/dangling-parent, and
corrupt-record fixtures are entirely absent (`SHOULD-2`), and the goal-directed counters are
query-phase-scoped and helper-level (`SHOULD-3`).

**Item 3 — `FAIL`.** See `MUST-2`. Preserving the earlier full traversal for error precedence does
violate the frozen one-decision-ledger contract as written, *and* it requires composition with the
accepted accounted-validation/collection predecessor already used by the HTTP live routes at
`crates/codefire-cli/src/http.rs:6100-6141`. The candidate did neither: it left three to four
unaccounted full passes per live command outside the ledger while naming the ledger command-scoped.

**Item 4 — `FAIL`.** See `MUST-1`. Yes, an object effect can occur before a budget, deadline, or
tamper rejection — `copy_object_graph` at `remote.rs:346` precedes the `UploadPublish` decision at
`remote.rs:377`. And no, comparing against an already-mutated accepted rejection is not sufficient
evidence: the assertion at `remote.rs:5815-5819` compares mutated object state to mutated object
state, and the `ResourceBudget` case at `remote.rs:5823-5830` omits the object comparison entirely.

**Item 5 — `PARTIAL FAIL`.**

| Required dimension | Result |
|---|---|
| exact / minus-one / plus-one | **PASS** for 6 cumulative + 2 concurrent dimensions, with typed `ResourceBudget`, matching `exhaustion.dimension`, `RESOURCE_BUDGET_EXIT_CODE`, `BudgetExceeded` terminal, owns-nothing, and unchanged state (`remote.rs:5608-5672`). **GAP**: `LogicalDepth` and `MaximumFrontier` boundaries untested (`SHOULD-2`) |
| 1x / 3x / 10x | **PASS** as exact equalities, but `Phase::Query`-scoped and helper-level (`SHOULD-3`) |
| 600-depth | **PASS** (`remote.rs:5229-5273`) |
| same-metadata tamper | **PASS** mechanically — inode, size, and mtime provably preserved (`remote.rs:5745-5747`), typed `SealedValidationSubjectChanged`, `SubjectChanged` terminal — but dry-run only (`SHOULD-4`) |
| deadline | reaches a production caller, but the mechanism is inert in production (`SHOULD-1`) |
| failure cleanup | **PASS** on dry-run and apply; objects excluded on publish (`MUST-1`) |
| compatibility | **FAIL as contracted** — see below |
| no-effect | **PASS** on dry-run and apply; **FAIL** on publish (`MUST-1`) |
| no public limit raised | **PASS** — diff is one file, `work_budget.rs` untouched, and `inject_file_remote_ancestry_budget` only lowers remaining budget (`remote.rs:4310-4320`) |

On compatibility: the implementation contract says "Keep direct helpers compatible for callers
outside this packet." The candidate moved `is_ancestor_in_objects` under `#[cfg(test)]`
(`remote.rs:3746-3747`), so it no longer exists in a shipped binary and cannot be called from any
non-test code. Its own doc comment claims the opposite — "The signature and truth value are preserved
verbatim so a caller outside this packet can still adopt it" (`remote.rs:3742-3745`). No live caller
exists today (crate-wide grep finds references only inside `remote.rs`), so the breakage is latent
rather than active, and the differential-oracle use is genuinely valuable
(`remote.rs:5123-5171`, nine ancestor/head pairs across chain, side branch, merge, unrelated, and
reversed directions). But it is a contract deviation reported as a preserved compatibility surface,
and the manager should see it as such.

**Item 6 — `FAIL`.** All independently run gates are green (table below), and the reviewed nucleus is
sound. The rule fires regardless: an unaccounted predecessor traversal remains on the live route
(`MUST-2`) and a pre-decision effect remains on the live route (`MUST-1`).

## Commands and results

Every command was run against a `git archive` snapshot of exact target
`e93ebc2394c8bd0398cfc23e8bf2d23d52e253b9` at `/tmp/cfr168-ancestry-target`. No product or test file
in any tracked worktree was modified.

```bash
export RUSTUP_HOME=/tmp/codefire-rustup/rustup
export CARGO_HOME=/tmp/codefire-rustup/cargo
export PATH="$CARGO_HOME/bin:$PATH"
```

Toolchain: `cargo 1.97.1 (c980f4866 2026-06-30)`, `rustc 1.97.1 (8bab26f4f 2026-07-14)`.

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | **PASS**, no diff |
| `cargo clippy -p codefire-cli --bin codefire-rs --locked -- -D warnings` | **PASS**, zero warnings, 1m14s |
| `cargo test -p codefire-cli --bin codefire-rs remote --locked` | **118 passed / 0 failed**, 660 filtered, 6.92s |
| `cargo test -p codefire-cli --bin codefire-rs --locked` (clean rerun) | **778 passed / 0 failed**, 50.81s |
| `cargo test -p codefire-cli --bin codefire-rs --locked` (first run, under concurrent build load) | **777 passed / 1 failed** — see note |
| `python3 docs/development/tools/check_source_docs.py` | **PASS** — "44 modules, 34 commands, 12 object kinds" |
| `git diff --check` | **PASS**, clean |

Representative legacy upload/apply coverage inside the focused `remote` run, all `ok`:
`tests::file_remote_upload_clone_show_diff_and_merge_request_flow`,
`tests::remote_mutator_idempotency_replays_same_payload_and_rejects_conflicts`,
`tests::remote_object_graph_copies_resolution_evidence_refs`,
`tests::file_remote_upload_dry_run_rejects_existing_invalid_layout`,
`tests::legacy_remote_identity::file_remote_upload_and_clone_preserve_legacy_short_commit_identity`,
`tests::storage_report_counts_file_remote_project_storage`,
`remote::publication::tests::recovery_preflights_all_staging_and_schema_before_first_write`.

**Note on the one observed failure.** The first full-suite run failed
`http::tests::missing_upload_generic_body_limit_dominates_before_bootstrap_dispatch`
(`crates/codefire-cli/src/http.rs:15002`, `assertion failed: response.starts_with("HTTP/1.1 200 OK")`).
That run was concurrent with a second `cargo` build on the same machine. Rerun in isolation the test
passes and takes 31.91s wall clock; the clean full rerun is 778/778. I am recording it because it is
a real observation and because a 31-second wall-clock-sensitive HTTP test is fragile under load — but
it is in `http.rs`, not on the reviewed surface, and it is **not** part of this verdict.

## Non-vacuity: production-wiring mutation experiment

To decide item 5's "must reach production callers" independently of the candidate's own claims, I
copied the target snapshot, un-gated `is_ancestor_in_objects`, and reverted only the three production
call sites (`remote.rs:311`, `:377`, `:2389`) to the baseline helper, leaving the new decision and
all ten new tests untouched.

`cargo test -p codefire-cli --bin codefire-rs --locked file_remote_ancestry_tests` → **3 passed / 7 failed.**

Failed (i.e. genuinely wired to production callers):

- `live_upload_dry_run_and_publish_agree_on_semantics_and_diagnostic_bytes`
- `live_merge_request_apply_uses_its_own_command_ledger`
- `live_merge_request_apply_rejects_a_non_fast_forward_source`
- `live_decision_enforces_exact_limit_minus_one_and_plus_one_boundaries`
- `live_decision_reports_a_typed_deadline_at_every_stage_without_mutating`
- `final_fence_rejects_a_same_metadata_content_rewrite`
- `live_publish_failures_leave_authority_state_and_publication_untouched`

Passed unchanged (i.e. helper-level, not production-wired):

- `accounted_decision_agrees_with_the_legacy_materializing_helper`
- `traversal_work_is_goal_directed_and_linear_at_1x_3x_and_10x`
- `deep_linear_history_and_shared_dag_traverse_once_without_duplicate_reads`

Conclusion: the seven live tests are non-vacuous against the baseline wiring — the "one file-remote
decision must own exactly one command ledger" assertion collapses to `left: 0, right: 1` the moment
a production call site regresses. This is strong evidence and should be credited. It also confirms
`SHOULD-3`: all three goal-directed counter tests are helper-level, so the specific evidence item 5
demands reach production callers is the evidence that does not.

## Comparison requirements for the original Fugu candidate

The Fugu candidate for `cfr168-rust-file-remote-ancestry-20260729` must not be selected merely for
finishing, and must not be compared on test count. Whichever candidate is integrated, decide these
on the same axes:

1. **Pre-decision effect fence (decisive).** Does the candidate move, stage, or otherwise fence
   `copy_object_graph` (`remote.rs:346`) relative to the `UploadPublish` decision? If it does, it
   clears `MUST-1` and this candidate does not, and that alone should decide the selection.
2. **No-effect evidence shape (decisive).** Does its no-effect assertion snapshot object state from a
   genuine pre-command baseline, or does it also compare against an already-mutated rejection? Look
   specifically for an `include_objects`-style escape hatch.
3. **Ledger scope.** Is the ledger command-scoped — spanning `validate_sealed_commit`,
   `collect_object_records`, the copy, and the decision — or decision-scoped as here? Compare against
   the HTTP precedent at `crates/codefire-cli/src/http.rs:6100-6141`.
4. **Duplicate subject validation.** Does the same commit get validated twice per live route
   (`remote.rs:178` / `:2382` plus the in-session validation at `remote.rs:4039-4040`), and if
   precedence forced the early call, was it accounted or left outside the ledger?
5. **Deadline reality.** Does its deadline fence do anything in a `#[cfg(not(test))]` build given
   `local_v1().request_deadline_ms == None` (`work_budget.rs:287`), or is it test-only scaffolding as
   here?
6. **Fixture classes.** Does it carry cycle, missing/dangling-parent, and corrupt-record counter
   fixtures that this candidate lacks?
7. **Boundary dimensions.** Does it enforce `LogicalDepth` and `MaximumFrontier` limit boundaries,
   which the precharge harness here structurally cannot?
8. **Failure injection reach.** Does injection cover `ApplyMergeRequest` and the live publish route,
   or only the dry-run route?
9. **Compatibility surface.** Does it keep `is_ancestor_in_objects` callable in a shipped binary, or
   also gate it `#[cfg(test)]`?
10. **Nucleus and diff discipline.** Is the accepted nucleus byte-identical and the diff confined to
    `crates/codefire-cli/src/remote.rs` with no profile or limit edits? This candidate sets a high
    bar here: `+1751 / -3`, three surgical call-site edits, nucleus byte-identical, clippy clean.
11. **Run the same gates on the same machine without concurrent builds**, and record the
    `http::tests::missing_upload_generic_body_limit_dominates_before_bootstrap_dispatch` timing
    fragility if it recurs.

If neither candidate fences the pre-decision object effect, that is a shared blocker in the
implementation contract's framing and should be escalated to the manager as a contract question
rather than re-run as another candidate lane. The contract simultaneously demands "object state
unchanged" on typed failure and freezes the existing upload ordering that makes it impossible.

## Residual risk

- **Accepted-if-integrated risk (highest).** If this candidate is integrated as-is, every new typed
  ancestry rejection on `codefire upload` silently injects the rejected head's object closure into
  the remote store while reporting that object state is unchanged. Impact scales with the closure
  size of rejected uploads and is bounded only by GC.
- **False-assurance risk.** Three of the candidate's most quotable claims are narrower than they
  read: the 1x/3x/10x early-exit numbers are `Phase::Query`-only, the deadline coverage is test-build
  only, and the publish-route no-effect proof is mutated-vs-mutated. A manager reading the commit
  message alone would over-credit all three.
- **Unmeasured retained-bytes regression.** The session retains every reachable commit document under
  ledger ownership, where the baseline retained a distances map. Charged and bounded by
  `RetainedBytes` (1 GiB), but no test compares peak retained bytes against the baseline, so a
  memory-profile regression on very large histories would be invisible.
- **Untestable cleanup paths.** `COULD-1`'s two ledger-loss paths cannot be reached by any injection
  and `DecisionWorkLedger` has no `Drop` guard, so a future refactor could widen them without any
  test noticing.
- **Latent compatibility break.** The first non-test caller to adopt `is_ancestor_in_objects` will
  fail to compile, with a doc comment actively asserting that it will not.
- **Unreviewed adjacent surfaces.** This review covers only the three file-remote fast-forward
  decisions. Still outside the packet and unaccounted: `ancestor_distances` and
  `nearest_common_ancestor_from_right` on the local merge path
  (`crates/codefire-cli/src/main.rs:10757-10793`), and the file-remote clone, storage-report, and
  GC routes that walk object graphs without a ledger.
- **Environment fragility.** The 778-test binary target contains at least one wall-clock-sensitive
  HTTP test that fails under CPU contention. Any future gate treating a single 778/778 run as
  dispositive should run it unloaded, exactly as the fallback ticket's warning implies.
