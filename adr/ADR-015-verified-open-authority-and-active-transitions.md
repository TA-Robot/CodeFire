# ADR-015: Verify one local-open authority and replay active-state transitions

- Status: Accepted
- Date: 2026-07-12
- Decision owners: Local lifecycle / recovery / doctor
- Related: CFR-163, RVC-012, RVC-024, RVC-027, RVC-028, RVC-037, ADR-009, ADR-010, ADR-011

## Context

A live open workspace is represented by several records with different durability and recovery roles:

- worktree `.codefire-open` identifies repository, branch, original open commit, path, and open instance;
- repository `repo.json` identifies the repository;
- `branches/<branch>.json` publishes the current sealed head and open state;
- `opened/<branch>.json` links the current base, path, instance, active directory, and last-known state;
- `active/<instance>/state.json` plus scan/fire/resolution/verification and transaction records describe working state.

Keeping these records separate preserves recovery evidence, but current consumers validate different subsets. A
valid branch head and a different valid opened base can therefore make status, doctor, and commit dry-run report
success while a live command detects the divergence only after active-state writes. Existing commit recovery also
cannot resume every partial active reset, and manual/batch Fire/Resolution changes publish dependent records one at
a time without a replay authority.

## Decision

1. Introduce one `VerifiedOpenSnapshot` loader/validator for stable command consumption. It binds and checks:
   - canonical repository path plus `repo.json.repository_id` against the worktree marker;
   - marker, opened registry, and branch names;
   - open path, open-instance ID, original-open commit, and exact active-state path;
   - opened current base against the published branch head;
   - branch, registry, and active stable-state agreement;
   - sealed current base through the existing pinned Store validator when the consumer requires content authority.
2. The validator returns typed field-level divergence, not a boolean. Diagnostics carry authority kind, expected and
   actual values, record paths, and whether retry, doctor inspection, or manual preservation is safe. Status,
   scan, verify, commit, batch mutation, and doctor consume the same issue vocabulary.
3. Normal stable state and documented in-flight transaction state are distinct. A valid pending transaction is not
   silently treated as a stable snapshot or generic corruption:
   - read-only/dry-run consumers report typed recovery-pending state without writes or apply guidance;
   - mutating consumers acquire the repository lock, validate the pending authority, run its exact reducer, and then
     load a fresh stable snapshot;
   - malformed, foreign, or divergent evidence is preserved and rejected.
4. Every dependent active-state transition has a durable authority before its first postimage write. The authority
   binds operation/request identity, preimage identities or digests, intended postimage identities or digests,
   staged artifacts where used, and a strict adjacent phase. Recovery accepts only the exact preimage, exact
   postimage, or the documented immediate write-before-phase gap and converges idempotently to one postimage.
5. Commit active reset is one such reducer. Removing scan/fire/resolution/verification state and publishing the clean
   state may resume from every recorded file boundary. A retry never requires a deleted preimage that the transaction
   did not retain or authenticate.
6. Manual and batch Fire/Resolution changes use one shared ledger-transition reducer. Fire status/
   `resolution_uid` and active Resolution records become visible as one recoverable semantic transition even though
   the filesystem has no multi-file atomic rename.
7. All control reads/writes remain capability-relative, no-follow, bounded, and identity-rechecked. The repository
   lock serializes CodeFire writers but is not treated as protection from arbitrary external filesystem edits.

## Consequences

- A command cannot plan from an opened base that differs from the published branch head.
- Doctor and lifecycle commands share corruption/recovery classifications instead of reimplementing authority rules.
- More transaction evidence is retained temporarily, but partial failure becomes diagnosable and replayable.
- Existing stable version-1 marker/branch/opened/active records remain readable. New transaction records are
  versioned extensions; unknown versions fail closed.
- This ADR does not merge all authority into one mutable JSON file, auto-delete divergent user data, or claim atomic
  multi-file visibility without a durable reducer.

The canonical source contracts are maintained in [source architecture](../docs/development/source-architecture.md), [persisted data model catalog](../docs/development/source-data-model-catalog.md), and [command source trace](../docs/development/command-source-trace.md). CFR-163 remains the implementation-cycle owner until its separate Install, Dogfood, and Retrospect gates complete.

## Verification

- Field-by-field marker/repository/branch/opened/active mismatch matrix in both mismatch directions.
- Status/scan/verify/commit/batch/doctor diagnostic equivalence and pre-write authority byte snapshots.
- Every commit-reset and Fire/Resolution publication/failpoint gap, exact replay, foreign collision, malformed
  transaction, symlink/identity replacement, and dry-run/no-write case.
- Full source gate, fresh install, isolated counterexample reversal, real-repository read-only matrix, canonical
  Algorithm dogfood, and Retrospect before CFR-163 closes.
