# ADR-014: Make unresolved merge state an authoritative sealing blocker

- Status: Accepted
- Date: 2026-07-12
- Decision owners: CodeFire core / local merge workflow
- Related: ADR-013, CFR-162, RVC-039

## Context

Local merge writes conflict markers or binary sidecars and records text, binary, and semantic conflict
metadata in active `state.json`. The Rust verification path did not consume that metadata. A non-Atom
text conflict therefore produced a passed decision and an unblocked two-parent commit plan even though
the product and architecture specifications call merge conflicts blocking.

Treating marker text alone as authority is insufficient: binary conflicts do not use markers, marker-like
text can be legitimate content, and semantic candidates may have no file-level marker. Treating every
historical conflict array as permanently blocking is safe but leaves no supported resolution transition.

## Decision

1. Active merge metadata is the authority for whether a merge requires resolution. Verification projects
   every recorded text, binary, and semantic item into typed `UnresolvedMergeConflict` facts.
2. `CommittableDecision` owns an exhaustive `UnresolvedMergeConflicts` reason. It is a non-relaxable
   blocker and has primary priority over derivative Fires. Its CLI exit is the reserved merge-conflict
   exit code 15.
3. `codefire merge --continue` is the only supported transition that clears unresolved metadata. It:
   - requires a pending merge;
   - verifies exact generated text markers are absent from every text-conflict path;
   - requires every recorded binary sidecar to have been removed after the user chooses/reconstructs the
     target result;
   - requires `--accept-semantic-conflicts` when semantic candidates exist;
   - performs no writes in dry-run or blocked cases;
   - applies under the repository mutation lock through a pinned, no-follow worktree subject, invalidates
     stale verification, records the resolution acknowledgement, and refreshes scan state.
4. The merge parent/base remain pending after continue. A later normal commit seals a two-parent commit;
   successful commit cleanup removes the active merge state.
5. `Verification.unresolved_merge_conflicts` is an optional-empty compatibility extension. Missing means
   empty for historical records. A non-empty current verification is failed by the core decision and can
   never validate as a sealed commit. The certificate schema does not add an always-zero count.

## Consequences

- verify, commit, automation JSON, human output, explain, and Store validation cannot disagree about the
  merge blocker.
- Resolving content and acknowledging semantic overlap become separate, explicit user actions.
- Conflict resolution remains manual; this ADR does not introduce semantic auto-merge or AI approval.
- Python remains the reference/fallback line. Its existing text-conflict blocker is retained, but the new
  Rust resolution writer is not backported implicitly.
- Concurrent non-CodeFire edits during a local mutation remain outside repository-lock serialization;
  continue rechecks the pinned worktree identity and commit still recomputes verification before publish.

