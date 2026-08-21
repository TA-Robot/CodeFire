# ADR-010: Stable kernel-backed lock files

## Status

Accepted for repository mutation locks and the import bootstrap lock.

## Context

Creating a lock pathname with `create_new` and deleting it on guard drop gives
mutual exclusion only while every process exits normally. A process crash
leaves a stale pathname that blocks recovery. Deleting a lock pathname also
permits an unlink/recreate ABA race: waiters can open different inodes and both
believe they own the same logical lock.

Import has an additional bootstrap interval before `.codefire/locks` exists.
Its lock must distinguish a CodeFire retry from a colliding user file and must
retain enough immutable provenance to reconstruct the first durable
transaction record after a crash.

## Decision

The Rust CLI pins `fs2 = 0.4.3` and uses advisory exclusive kernel locks for
the ordinary `RepoLock`, import's repository lock, and the import bootstrap
lock.

- A canonical lock pathname and its inode are persistent and are never unlinked
  as part of normal unlock.
- Repository and import lock paths are opened relative to pinned directory
  capabilities with final-component no-follow semantics and must name regular
  files. A symlink at the lock path is rejected without truncating its target.
- Both ordinary and import-created `.codefire/locks/repo.lock` files must have
  exactly one provable filesystem link. Their file identity and link
  provenance are checked before locking and rechecked through the canonical
  pathname after ownership is acquired. Unsupported platforms fail closed.
- Closing or explicitly unlocking the file releases ownership, including after
  process termination; the file's mere existence is not ownership.
- The bytes of `.codefire/locks/repo.lock` are never truncated or written as
  part of acquisition. Owner diagnostics live in the common sibling sidecar
  `repo.lock.owner.json`, atomically replaced only after the kernel lock is
  held. Contenders read that sidecar with no-follow regular-file semantics and
  a 16 KiB limit; an absent, malformed, symlinked, non-regular, or oversized
  sidecar is ignored rather than followed or trusted. Both the stable lock
  inode and the last sidecar may remain after unlock.
- Import atomically publishes a fully formed, versioned bootstrap-provenance
  file from a private staging name before locking it. The writer first syncs an
  unpublished `.codefire-import.lock.bootstrap-*.part`, hard-links those bytes
  to a fully formed `.codefire-import.lock.bootstrap-*.tmp`, and only then
  links the canonical lock name. Bootstrap/control reads are no-follow,
  non-blocking, regular-file-only, and bounded to 1 MiB with metadata and
  limit-plus-one checks. `.part` is never authority and is preserved unless
  identity proves it is another name for the canonical lock. Its mutable
  recovery phase lives in a separate atomically replaced transaction record so
  replacing state never replaces the locked inode.
- Import creates and locks the persistent repository lock before publishing
  repository metadata. Pre-initialization recovery permits only that exact lock
  layout and proves its pathname still identifies the held file descriptor.
- Import atomic-write temp names use
  `.codefire-import.write-<transaction-id>-*.tmp`. Recovery removes only
  matching remnants; foreign IDs and malformed fully published bootstrap
  `.tmp` provenance are preserved and fail closed.
- Foreign, partial, or request-mismatched bootstrap provenance fails closed.

The existing specialized remote/resource `FileLock` is not silently redefined
by this decision. It must be migrated with its own compatibility and recovery
fixtures before the old create-new representation is removed.

## Impact

This adds one exact direct dependency plus its platform support dependency. It
changes `repo.lock` from an ephemeral ownership/metadata token to a persistent,
content-opaque kernel-lock inode and moves best-effort owner diagnostics to a
separate persistent sidecar. Diagnostics must test the actual advisory lock
rather than treating either path's existence as contention. The mechanism is
cooperative: a process that ignores advisory locks can still write repository
files, so validation and atomic publication remain required.

## Alternatives considered

- Keep create-new locks and delete stale files by PID/time: rejected because PID
  reuse, clock policy, and races cannot prove that the prior owner is gone.
- Unlink a kernel-locked file on drop: rejected because new callers can lock a
  replacement inode while an earlier waiter still references the old inode.
- Directory locks: rejected because they have the same crash/stale-path problem
  without a separate ownership protocol.
- Direct `flock`, `fcntl`, or Windows APIs: rejected because that duplicates
  unsafe platform-specific behavior in CodeFire.
- Standard-library file locking: unavailable at the repository's Rust 1.75
  minimum version.

## Removal or replacement

Replace `fs2` only when the minimum supported Rust standard library provides the
same cross-platform try-lock/unlock behavior, or another reviewed abstraction
passes the crash, stale-file, ABA, wait-timeout, and concurrent-process
fixtures. Migrate every remaining create-new lock user first, then remove the
dependency and regenerate `Cargo.lock`.
