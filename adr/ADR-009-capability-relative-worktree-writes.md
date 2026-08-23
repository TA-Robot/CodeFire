# ADR-009: Capability-relative worktree writes

## Status

Accepted for the Rust CLI.

## Context

`open` materializes commit files below a configured workspace boundary. A
lexical or canonical path check followed by ordinary ambient filesystem calls
does not bind the validation result to the directory later used for writes: an
existing component can be renamed or replaced with a symlink between check and
use.

The invariant is stronger than path normalization: every materialization write
must remain below the directory handle that was reached from the validated
workspace boundary, even if ambient names change concurrently.

The symmetric close/discard invariant is that marker validation authorizes only
the pinned open directory. An ambient target replacement after validation must
not redirect recursive deletion into unrelated content.

The same invariant applies to mutable active state. Validating
`.codefire/active/<open-instance>` and then returning to ambient path-based
reads, writes, or removal would allow an instance rename plus symlink
replacement to redirect later workflow state outside the repository.

## Decision

The Rust CLI uses pinned `cap-std = 4.0.2` and `cap-fs-ext = 4.0.2` dependencies
inside `secure_worktree`. On platforms where `rustix` exposes an atomic
no-replace rename, close/discard also declares the already-transitive
`rustix = 1.1.4` directly with only its `fs` feature.

- The workspace boundary is opened component by component without following a
  symlink in the component being opened.
- A deterministic sibling stage and its bounded, no-follow, no-replace owner
  record bind repository path/identity, branch, target, and sealed base. Only exact matching
  provenance permits abandoned-stage cleanup; foreign/mismatched entries fail
  closed.
- The stage and its child directories are created and opened relative to pinned
  directory handles. The final marker and all materialized content are complete
  before one sibling rename makes the target visible.
- Materialized files use create-new semantics and refuse a symlink at the file
  name.
- A failed materialization removes only its unpublished owned stage and leaves
  the target absent or unchanged, so the same operation can retry. There is no
  remove-then-rename fallback when a platform cannot atomically replace an
  existing empty target.
- Mutable repository metadata is published only after the ambient target still
  resolves to the pinned target identity.
- Resolution and capability-use failures are preserved as the typed
  `OpenTargetResolution` boundary so JSON diagnostics carry a stable reason,
  requested/resolved path, workspace boundary, and safe-target retry action
  instead of collapsing the failure into context-free I/O.
- Close/discard binds the recorded parent/target, reads the marker bounded and
  no-follow, verifies repository/branch/open-instance identity, and either
  deletes the bound marker or moves the target to a sibling quarantine. Full
  deletion begins only after one kernel no-replace rename moved the directory,
  the moved identity matches the pin, and the parent rename is synced. A
  quarantine collision is retried without overwriting the colliding entry; a
  platform without this primitive fails closed. Deletion recursively removes
  entries relative to the pinned handle without following symlinks. A failure
  before deletion may restore the original name with the same no-replace
  primitive. Once deletion may have begun, a failure never restores a damaged
  tree as authoritative: it leaves the sibling quarantine named in the error
  while repository state remains unchanged for explicit inspection/recovery.
- Opening local workflow context binds both the active-state parent and exact
  instance directory. Active files are restricted to one normal relative path
  component and are read no-follow/non-blocking with a byte bound. Atomic writes
  create and sync a new temporary file relative to the pinned instance, rename
  it relative to that same handle, sync the directory, and recheck the ambient
  parent/instance identity at operation boundaries. There is no ambient
  path-only fallback.
- An instance rename followed by an outside symlink at the recorded name fails
  closed as typed `ActiveStateError`; CLI JSON exposes the stable
  `active_state_capability_invalid` diagnostic and does not touch the outside
  target. Active-state tree removal uses a transaction-recorded deterministic
  quarantine, stable directory identity, and the shared kernel no-replace
  primitive. A collision is never overwritten, and a failure after deletion
  begins preserves the named quarantine for recovery.
- Read-only active-state inventory also opens the active root and descendants
  through no-follow directory capabilities, so doctor/storage metadata scans
  do not traverse a substituted outside symlink.

The versions are exact so a filesystem-security boundary is not changed by an
unreviewed semver-compatible upgrade.

This decision makes worktree materialization atomic at the sibling rename. It
does not make the later opened-registry, active-state, and branch-state writes
one transaction; interruption after the rename remains a separate recovery
gap and must not be described as full open-state atomicity.

## Impact

This adds two direct crates and their transitive platform support crates to the
Rust CLI build. It increases dependency review and compilation surface, but it
keeps unsafe OS-specific descriptor code out of CodeFire and gives the path
policy one focused module with adversarial race tests. The Python CLI remains a
reference/fallback implementation and does not gain these dependencies.

The close-only `rustix` declaration adds no new package to the lockfile because
`cap-std` already brings the same exact version on supported targets. It is a
direct dependency solely because `cap-std` has no public atomic no-replace
rename operation; using a check followed by replacing rename would reintroduce
the collision race.

## Alternatives considered

- Lexical rejection plus `canonicalize`: rejected because it leaves a
  check/use rename or symlink race.
- A second ambient path check immediately before every write: rejected because
  a path can change after the check and before the write.
- Reconstructing active-state file paths from a previously validated registry:
  rejected because path equality does not bind later I/O to the validated
  instance directory identity.
- Direct `openat`/`openat2` or equivalent platform APIs: rejected because it
  requires unsafe, OS-specific implementations and a separate Windows design.
- Reserving a quarantine name with `symlink_metadata` before ordinary rename:
  rejected because another actor can create the name between the check and a
  replacing rename.
- Refusing all nested target paths: rejected because it weakens the supported
  workspace layout without eliminating races at the remaining component.

## Removal or replacement

Remove these dependencies only after a replacement provides reviewed,
cross-platform, descriptor-relative directory traversal and no-follow file
creation plus atomic no-replace rename with the same race fixtures. The direct
`rustix` declaration can be removed specifically when `cap-std` or the standard
library exposes that rename contract. A stable standard-library API or a small
internal abstraction may replace the dependencies; the migration must keep the
`secure_worktree` and close quarantine contracts and tests before deleting the
crates from `Cargo.toml` and regenerating `Cargo.lock`.
