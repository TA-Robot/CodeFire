# Current Cycle Burn-Down Plan

この文書は、直近のcodefire改善サイクルで発見したissue batchを、次のissue探索へ進む前に全部修正するための計画である。

## Policy

今後のcodefire改善サイクルは、次の順序を固定する。

1. Discovery: algorithm dogfoodとCodeFire source reviewでissue batchを作る。
2. Freeze: batch範囲を凍結し、修正対象issue IDを確定する。
3. Burn-down: batch内issueを全部fixedまたは明示wontfixにする。
4. Install: 修正版CodeFireをinstallし、実コマンドとして確認する。
5. Dogfood confirmation: batch内issueの再現コマンドが通ることをalgorithm projectで確認する。
6. Close audit: backlog、detail issue、history、version planが一致していることを確認する。
7. Next discovery: batchが0件になってから次のissue出しへ進む。

Burn-down中に別の問題を見つけた場合の扱い:

- batch内issueの修正を妨げるblockerは、同じbatchへ追加し、そのbatchで消す。
- blockerでない周辺改善は、CFB/CFR採番せず `next discovery candidate` として短くメモする。
- 次の正式issue batchは、現在batchが全部閉じた後に作る。

この運用により、「issueを出す -> 全部消す -> issueを出す」を崩さない。

## Current Open State

Current known open issue count, audited 2026-07-12 after the organization checkpoint:

- CFB/CFR product/review issues: 7 open.
- CFB detail ledger: 127 total; 120 `fixed`, 7 `open` (`CFB-120`..`CFB-126`).
- CFR detail ledger: 150 `fixed`, 10 `fixed in v0.7`.
- Algorithm CodeFire open fires after the latest dogfood seal: 0.
- The organization checkpoint is complete; active work is restricted to this frozen correctness batch.
- Historical cycle sections were moved to `archive/completed-cycle-ledger.md`. Do not infer current issue counts from historical headings or remaining-count notes.
- The management hub `docs/development/README.md` is the current-state entry point; update it and append `history.md` when this section changes.

## Active Organization-Audit Batch

Batch ID: `BATCH-2026-07-stage1-invariants`

Frozen scope:

- `CFB-120`: scan sealed-base validation;
- `CFB-121`: pre-publication sealed-commit validation;
- `CFB-122`: resolved path/symlink containment;
- `CFB-123`: Trace Graph endpoint integrity.
- `CFB-124`: live scan/verify repository-state locking;
- `CFB-125`: short-ID collision full-digest comparison.
- `CFB-126`: import idempotency request binding.

This batch was discovered while correcting canonical source documents. It is the first Stage 1 correctness batch after the management/strategy/AI checkpoint. Do not add discretionary feature work or claim release readiness while it is open.

## Active Batch Execution Gate

| Order | Issue | Required outcome |
|---:|---|---|
| 1 | CFB-120 | `scan` rejects an unsealed or structurally invalid base commit before deriving active state. |
| 2 | CFB-121 | `commit` validates the complete candidate commit graph before publishing the branch head. |
| 3 | CFB-122 | every user-controlled path is contained after canonicalization and symlink resolution. |
| 4 | CFB-123 | Trace Graph endpoints resolve to existing Atoms or return an explicit blocking diagnostic. |
| 5 | CFB-124 | live `scan` and `verify` mutations reload state after acquiring the repository lock. |
| 6 | CFB-125 | abbreviated object lookup compares full digests and reports collisions deterministically. |
| 7 | CFB-126 | import idempotency binds the key to the complete normalized request. |

The seven issues are one correctness batch. Implement by shared invariant/root cause where possible, but close each detail record only after focused regression evidence exists.

## Required Validation

Before closing the batch:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_source_docs.py
```

Then install or select the validated Rust binary, rerun the affected commands against the dogfood project, and synchronize `bug-backlog.md`, the seven detail records, `issue-root-cause-map.json`, this file, and `history.md`.

## Historical Evidence

Completed batches, the full historical issue inventory, root-fix groups, and dogfood Cycles 4–124 are preserved in `docs/development/archive/completed-cycle-ledger.md`. Later dogfood history remains append-only in `docs/development/history.md` and the dogfood project's own history.

Do not reintroduce completed cycle narratives into this active plan.
