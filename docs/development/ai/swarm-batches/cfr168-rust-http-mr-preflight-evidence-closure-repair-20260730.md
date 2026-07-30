# CFR-168 Rust HTTP MR Preflight Evidence-Closure Repair

Status: ready for exact original-implementer continuation

Provider/model: Claude Opus 5 (`claude-opus-5`)

Role: implementation repair

## Exact inputs

| Item | Exact value |
|---|---|
| Reviewed product candidate | `73bc5327ff6d750ce95f63333055ee39b693cb8c` |
| Candidate tree | `430f2e2ccf9dca485575e7f9db0b639390d7e761` |
| Candidate parent | `7e4e685efbfc453a6d2209f37c99edb1ab46243d` |
| Candidate patch SHA-256 | `216c12f8d9e0c6939bf6889856a999e8f45b56237eca9ff63cdef34105ef268d` |
| Exact review commit | `28eb5a94e2f452740df7e8c5b64a8a8f3b9646b0` |
| Exact review tree | `41369845a91a005b50c6d499b1c757c5990f1559` |
| Review report SHA-256 | `8bb1a418ce6db1aa932eca2d19990212a58a5f44ad42eb3ccd439e68b4f1417d` |
| Review verdict | `REQUEST_CHANGES` |
| Original implementer session | `8d2b5b6e-54d7-4151-84bb-0820ea5d1efb` |

Read the immutable report without importing its report commit into the
product lineage:

```bash
git show 28eb5a94e2f452740df7e8c5b64a8a8f3b9646b0:docs/development/ai/swarm-results/cfr168-rust-http-mr-preflight-zero-residual-rereview-claude-20260730/review.md
```

The reviewed candidate's F1--F5 and R1--R4 product obligations are closed.
Do not redesign or rewrite them. Append repair commits only; do not amend,
rebase, squash, reset, or rewrite `73bc532`.

## Objective

Close the review's new evidence/claim findings F-1 through F-4 and locally
close residuals R-1 through R-5 without changing established wire behavior,
authority/effect ordering, error precedence, or typed public phase strings.
Route R-6 and R-7 to their already frozen cross-runtime corpus and file-
authority successors with exact evidence rather than broadening this focused
repair into another multi-surface rewrite.

## Required repairs

### F-1: non-vacuous scaling axis

Repair
`merge_request_response_work_is_flat_while_object_work_scales` so its 1x/3x/
10x input changes the reachable source closure, not unrelated unreachable
objects.

The test must independently show:

- the reachable/escrowed record or object count changes at 1x, 3x, and 10x;
- the intended object-work counters grow according to exact expected formulas;
- response body bytes, header bytes, response encodings, and response-owner
  accounting remain flat;
- retained/temporary peaks have explicit expected behavior;
- the 10x case reaches the intended production owner and not an earlier
  unrelated limit; and
- a deliberate regression that restores unreachable additions or couples
  response accounting to the object count fails for the intended assertion.

Correct the test comment and invert/remove the assertion that currently proves
the object axis is constant.

### F-2: exact durable byte claim

The standard review success body is exactly 60 bytes, not 57. Pin 60 beside
the existing 126-byte create and 149-byte durable-retry expectations, and add
one durable repair-result document that recomputes all three bytes from the
production fixture. Do not rewrite the old commit message.

### F-3: phase identity claim

Do not change stable public phase strings merely to make source sites unique.
Correct the changed comment to state the actual contract: the table assigns
one typed identity per phase name and rejects duplicate table entries, while
co-named call sites remain intentionally indistinguishable unless a separate
future per-site contract is accepted. Add a regression that prevents the
comment/table oracle from claiming call-site uniqueness.

### F-4: suppression scope

Remove the `#[allow(dead_code)]` from the whole
`PreparedHttpMutationTransaction` impl and place the narrowest necessary
suppression only on the three named attach methods, or delete the suppression
if strict Clippy proves none is needed. Add a focused source/compile guard
that fails if an impl-wide suppression returns.

### R-1: retained owner attribution

Strengthen `review_retained_limit_minus_exact_plus_one_is_effect_safe` so it
proves the rejection occurred at the review response-preflight retained owner,
not an earlier retained acquisition, while preserving exact limit/observed,
typed dimension, unchanged authority, and zero residual ownership.

### R-2: suppression re-addition gate

Add a narrow, maintainable gate for this changed owner/transaction region that
rejects a new broad `allow`/`expect` suppression or a redundant suppression.
Do not turn the whole file's historical suppressions into this task.

### R-3: release-mode Drop evidence

Exercise route unwind after prepared responses are owned and before normal
selection/cleanup. Prove response permits and request-ledger ownership
terminalize in debug and release behavior. If the remaining `debug_assert`
can hide a release-mode leak/invalid terminal state, use the existing
fail-closed drop mechanism without introducing a double panic during unwind.

### R-4: pre-effect accounting-release fault

Add an injection at the create precommit release seam. Prove:

- no first authoritative effect occurred;
- original authority and escrow remain byte-identical;
- the primary typed outcome survives a redundant/secondary release failure;
- the exact owner remains retryable until released; and
- all request and response owners end at zero.

### R-5: postcommit terminal observation

`record_http_post_commit_cleanup_failure` must not degrade to an unobservable
release-mode no-op if request finish rejects. Preserve the committed response
and primary outcome, but leave exactly one durable/typed terminal observation
or fail closed through the established safe mechanism. Add a deliberate
regression for an open-permit/finish-rejection path.

## Routed residuals

- R-6, the Rust/Python 149-byte durable cross-runtime wire oracle, is owned by
  `cfr168-final-complexity-conformance-freeze-20260730`. Record the exact Rust
  149-byte fixture and required Python counterpart in the repair result; do
  not edit active Python product lineages here.
- R-7, snapshot mode/ownership metadata, is owned by the file-authority and
  final selected-surface successor after accepted file-ancestry/storage
  composition. Record exact current snapshot fields and the missing metadata
  transition in the repair result; do not expand this MR response repair into
  a file-transport redesign.

Routing is not closure. Both residuals remain open in the manager ledger until
their exact successor reports accept them.

## Scope

Allowed product/test paths:

- `crates/codefire-cli/src/http.rs`;
- `crates/codefire-core/src/work_budget.rs` only if a local fail-closed owner
  primitive cannot be closed in `http.rs`;
- direct Rust HTTP/core tests already colocated in those files; and
- one new durable result at
  `docs/development/ai/swarm-results/cfr168-rust-http-mr-preflight-evidence-closure-repair-20260730/result.md`.

Canonical source documents may be changed only if the repaired behavior or
counter formula changes their exact statement. Do not edit Python product or
tests, storage, file ancestry, File Doctor, issue state, lifecycle state,
capability claims, devcontainer, packaging, or management history.

## Validation

Use the shared low-memory environment:

```bash
export RUSTUP_HOME=/home/devuser/.cache/codefire-rust-1.97/rustup
export CARGO_HOME=/home/devuser/.cache/codefire-rust-1.97/cargo
export PATH="$CARGO_HOME/bin:$PATH"
export CARGO_TARGET_DIR=/workspace/.cache/cargo-target-shared
export CARGO_BUILD_JOBS=2
export RUST_TEST_THREADS=2
export CARGO_INCREMENTAL=0
```

Run focused tests for every repaired finding/residual, the exact merge-request
HTTP test group, strict Clippy for `codefire-cli`, format, affected core tests,
source-doc checks if source docs change, and deliberate regressions. Reuse
unchanged broad evidence only as historical context; the fresh exact rereview
will determine the final gate set.

Before handoff:

- worktree is clean;
- every change is committed additively above this task commit;
- exact parent/head/tree/binary patch digest and changed paths are recorded;
- the result document records commands, counts, durations, unavailable
  evidence, routed R-6/R-7, and no product overclaim; and
- no worker claim is treated as independent acceptance.

## Non-Goals

- Do not redesign the already closed F1--F5/R1--R4 implementation.
- Do not rewrite history or the incorrect old commit message.
- Do not implement R-6 Python parity or R-7 file metadata in this lane.
- Do not integrate, accept, close issues, or advertise capability.
