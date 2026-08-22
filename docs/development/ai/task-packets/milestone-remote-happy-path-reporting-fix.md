# AI Task Packet

Status: ready

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
- On the replay branch only, return a plan that is observably a replay: `replayed: true`,
  `would_apply: false`, and no live mutation operations. Do not regenerate or rewrite the receipt.
- Keep the first live apply plan unchanged.
- In `request-list`, perform stale projection only for a non-terminal MR whose target precondition has
  drifted. Persisted terminal states remain authoritative; at minimum `applied` must remain `applied`.
- Preserve validation of current target sealed heads and malformed MR records.
- Add regression coverage proving the live apply, idempotent replay metadata, unchanged persisted
  authorities, and terminal `request-list` status in one bounded file-backed flow.

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

Not implemented.
