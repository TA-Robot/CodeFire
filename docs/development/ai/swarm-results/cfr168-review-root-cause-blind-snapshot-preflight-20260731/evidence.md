# CFR-168 Blind Root-Cause Snapshot Preflight

Status: manager preflight passed; no provider launched
Date: 2026-07-31
Product mutation: none

## Purpose

Verify that the prepared Fugu blind-calibration lane can start from a tree
that does not contain the provisional causal labels or temporal reports,
without telling the worker the provisional successor-effect row count.

This is launch-boundary evidence only. It does not classify a finding, prove
review omission, or authorize a process change.

## Exact subject

| Item | Value |
|---|---|
| Calibration commit | `c258cbfc061b14905554c9d319b350d2da7f771b` |
| Calibration tree | `75135efad20c5908884231b152cde72d0c9be965` |
| Temporary worktree | `/tmp/cfr168-blind-snapshot-preflight-20260731` |
| Worktree mode | detached |
| Final worktree status | clean |
| Temporary worktree after preflight | removed; `git worktree prune` completed |

The manager created the detached worktree with:

```text
git worktree add --detach \
  /tmp/cfr168-blind-snapshot-preflight-20260731 \
  c258cbfc061b14905554c9d319b350d2da7f771b
```

## Report-universe result

The exact snapshot command:

```text
git ls-tree -r --name-only HEAD docs/development/ai/swarm-results
```

found exactly 21 paths ending in `/review.md`. The SHA-256 of the
lexicographically sorted `content_sha256  path` manifest for those 21 files
was:

```text
53f364d58adc3b7aae9d24af55b220cb2279bdeb81c797451ff3d50117d4afaf
```

This binds the report universe, not an expected effect-episode or observation
row count.

## Exclusion proof

`git cat-file -e HEAD:<path>` returned nonzero for all four paths:

```text
docs/development/ai/swarm-results/cfr168-review-recursion-root-cause-audit-claude-20260731/audit.md
docs/development/ai/swarm-results/cfr168-python-http-upload-second-review-closure-rereview-claude-fallback-20260731/review.md
docs/development/ai/swarm-results/development-flow-kgi-kpi-r2-rereview-claude-fallback-20260731/review.md
docs/development/ai/swarm-results/aia-source-keyed-cargo-gate-closure-rereview-claude-20260731/review.md
```

Therefore the worktree's `HEAD` tree contains neither the provisional audit
nor the three temporal reports.

## Historical object availability

The 22 candidate-generation abbreviations already reconstructed by the
manager all resolved as `commit` objects from the detached worktree:

```text
e2a218e dece688 76baff8 467b0c2 37f7427 38da898 4930d25 b164046
21fe751 6dd4527 701469f 52a025e 1a04490 cd130e5 71854e2 73bc532
92fc810 b3bc32a cdcb9bd e93ebc2 02b0d63 0ae583e
```

This verifies object availability only. The worker must derive an object's
use from snapshot-visible reports, packets, or results; this list and its
count are not included in the worker execution ticket.

## Residual isolation limitation

The detached worktree shares the repository's Git object database. It removes
newer files from the checked-out tree but is not an OS-level or object-store
sandbox. A worker could still contaminate calibration by deliberately reading
a newer ref, commit, reflog entry, or another worktree.

`dev-swarm` reads the current task ticket before launch and injects its text
into the worker prompt. That exact envelope/ticket is the sole intentional
post-snapshot control input; it carries the rubric and prohibitions but no
provisional labels or expected effect-episode/observation-row count. It does
not authorize tool access to its current canonical path or to any other newer
artifact.

Accordingly, acceptance additionally requires:

1. the exact `c258cbf…` launch base;
2. no newer-ref/worktree access in the complete Fugu event log;
3. event-log byte count and SHA-256 preservation;
4. report-only output scope and clean worktree; and
5. rejection of the calibration if any forbidden access appears, regardless
   of the report conclusion.

No Fugu, Claude, or Codex session was started by this preflight.
