# AI Task Packet

Status: completed

Packet ID: MS-remote-happy-path-reporting-fix

Schema version: human-v1

Risk score: 7

Reasoning tier: high

## Objective

Fix only the two machine-readable reporting defects reproduced by remote-run attempt 2:

1. a completed file-backed `request-apply` idempotency replay must return a lightweight plan with
   `replayed: true` and must not claim that it would apply the mutation again;
2. `request-list` must preserve a persisted terminal MR status such as `applied` instead of replacing
   it with `stale` solely because the successful apply advanced the target head.

No remote authority, transaction, receipt, graph, or locking behavior may be redesigned. CFR-168 and
CFB-136 remain parked.

## Scope

The two reproduced file-backed JSON projections, their focused Rust regression coverage, the directly
affected public/source documentation, and this packet only. Preserve the existing publication and
idempotency authorities; this is a reporting correction, not a remote transaction redesign.

## Fixed evidence

- integration base: `564fd5e0f1dbd3ab3df7bd5d201d5aa702d18b59`;
- run evidence commit: `9329fe62002f3c9792239c3ade9fb8cd4363709c` (integrated as `564fd5e`);
- evidence root: `/tmp/m1-remote-happy-path-a2.zvrbzn`;
- MR: `MR-e191c85315e9`, persisted `status=applied`, `applied_head=CF-COMMIT-126555fdb84822a0c09c9e6a`;
- replay receipt and domain authorities were not rewritten, but the JSON response omitted `replayed`
  and reused the live five-operation plan;
- `docs/04_cli_spec.md` already requires file-backed replay metadata `replayed: true` and says remote
  request replay returns the saved result before status/stale/approved checks.

## Exact write set

- `crates/codefire-cli/src/remote.rs`;
- `crates/codefire-cli/src/remote/plans.rs` and
  `crates/codefire-cli/src/remote/publication.rs`, only if required to distinguish a completed-receipt
  replay from pending/quarantine recovery without overloading the publication result;
- focused Rust tests under `crates/codefire-cli/src/tests.rs` or one responsibility-named file below
  `crates/codefire-cli/src/tests/` if separation is clearer;
- only the directly relevant paragraphs/rows in the canonical source docs:
  `docs/development/source-architecture.md`, `command-source-trace.md`,
  `source-data-model-catalog.md`, and `change-playbook.md`;
- only directly relevant public contract text in `docs/04_cli_spec.md` and
  `docs/automation_interface.md`;
- this packet.

If a different product file or schema is genuinely required, stop and report the exact reason before
editing it. Do not broaden the write set silently.

## Required behavior

- Preserve the existing completed receipt as the source of replay identity and result.
- Do not equate every `replay_json_publication` `Some(...)` result with a pure replay: that path also
  finishes quarantine or a pending transaction and may publish authorities, including without an
  idempotency key. Carry or derive an explicit completed-receipt fact before reporting replay.
- On the completed-receipt replay branch only, return a plan that is observably a replay: `replayed: true`,
  `would_apply: false`, and no live mutation operations. Do not regenerate or rewrite the receipt.
- Keep the first live apply plan unchanged.
- In `request-list`, preserve the current target-head sealed validation, but perform the `stale`
  assignment only when the persisted status is not `applied` and the target precondition has drifted.
  `rejected` remains re-reviewable and therefore still receives actionable stale projection.
- Preserve validation of current target sealed heads and malformed MR records.
- Add regression coverage proving the live apply, idempotent replay metadata, unchanged persisted
  authorities, and terminal `request-list` status in one bounded file-backed flow.
- Add a separate regression proving a drifted non-terminal MR still lists as `stale`; a change that
  deletes stale detection must fail. Keep the existing upload replay-plan equality test unchanged.

## Validation

Run the smallest focused Rust tests that cover both regressions, then:

```bash
cargo fmt --check
cargo clippy -p codefire-cli --all-targets -- -D warnings
python3 docs/development/tools/check_source_docs.py
python3 docs/development/tools/check_project_docs.py
git diff --check
```

Do not run the whole workspace unless a focused failure proves it necessary.

## Non-Goals

- changing receipt/pending/publication formats or idempotency key derivation;
- changing HTTP behavior beyond any already-shared result projection required by the same typed plan;
- adding a new MR lifecycle or changing apply semantics;
- fixing lock owner sidecar mtimes;
- CFR-168 decision-work, CFB-136 validation caching, scale, HTTP transport, installer, or later slots;
- mutating either preserved M1 evidence root or canonical Algorithm state;
- rerunning the remote scenario before a source-bound runner is built from the accepted fix.

## Handoff

Return the exact source seam, invariant argument, focused tests and counts, full changed-file list,
validation results, commit SHA, and whether a source-bound runner can be built. Commit only this packet
and the admitted write set.

## Result

Status **completed**. No `block-m1`. CFR-168 and CFB-136 remain **parked**. Integration base `564bb808ba23294033a625b07535f1777ab8b69f`.

Exact source seam:

- `crates/codefire-cli/src/remote.rs::apply_merge_request` — on a completed `replay_json_publication` receipt only, replace the live five-operation plan with `request_apply_replay_plan` (`replayed: true`, `would_apply: false`, empty `operations` / `next_actions`). Live apply still uses `request_apply_operation_plan`. Receipt lookup, gate, signature, and `execute_json_publication` are unchanged.
- `crates/codefire-cli/src/remote.rs::list_merge_requests` — still validates the current target sealed head and malformed MR records; `stale` is projected only when the persisted status is non-terminal and `target_head_at_request` drifted. Persisted `applied` and `rejected` remain the list status.

Invariant: reporting only. The completed v2 receipt stays the replay identity and result. Replay does not regenerate or rewrite receipt, branch, generation, or MR authorities. First live apply plan is unchanged. Stale remains a list projection, not a persisted lifecycle state.

Focused tests (`cargo test -p codefire-cli --bin codefire-rs`):

- `tests::request_apply_replay::file_backed_request_apply_replay_and_applied_list_status` — ok
- `tests::request_apply_replay::file_backed_request_list_keeps_stale_projection_for_open_mr` — ok
- `tests::remote_mutator_idempotency_replays_same_payload_and_rejects_conflicts` — ok (existing conflict/replay-head coverage)

Counts: 2/2 new focused tests passed; 1/1 related existing idempotency test passed; `2 passed; 0 failed; 771 filtered out` for the new pair, then `1 passed; 0 failed; 772 filtered out` for the existing test.

Validation:

```text
cargo fmt --check
cargo clippy -p codefire-cli --all-targets -- -D warnings
python3 docs/development/tools/check_source_docs.py   -> 44 modules, 34 commands, 12 object kinds
python3 docs/development/tools/check_project_docs.py  -> 547 Markdown files
git diff --check                                      -> clean
```

Changed files:

- `crates/codefire-cli/src/remote.rs`
- `crates/codefire-cli/src/tests.rs`
- `crates/codefire-cli/src/tests/request_apply_replay.rs`
- `docs/development/source-architecture.md`
- `docs/development/command-source-trace.md`
- `docs/development/source-data-model-catalog.md`
- `docs/development/change-playbook.md`
- `docs/04_cli_spec.md`
- `docs/automation_interface.md`
- `docs/development/ai/task-packets/milestone-remote-happy-path-reporting-fix.md`

HTTP, scale, publication/receipt formats, lock sidecars, CFR-168, and CFB-136 were not touched. Isolated evidence roots and canonical Algorithm state were not mutated.

Active milestone: `MS-remote-happy-path`.
This packet is the admitted reporting fix on the critical path.
Parked instead of done: CFR-168, CFB-136, HTTP apply/list projection, upload/merge/review replay plans, lock-owner mtimes.
Next allowed action: build the source-bound runner from the accepted rust HEAD and hand it to node 4b; do not rerun the remote scenario from this packet.

### Source-bound runner

Recorded after the accepted source commit. See the runner identity block below once the release binary is built from that HEAD.
