# ADR-011: Capability-bound sealed object validation

## Status

Accepted for the Rust object store.

## Context

Sealed-commit validation recursively reads a commit, its parents, every required
root, and reachable evidence/blob references. A cache may reuse that result
within one command. Binding the cache only to the canonical object-root path and
checking the root directory identity before and after traversal is insufficient:
an object root or one of its object subdirectories can be replaced during the
reads and restored before the final check. That swap-back ABA can validate a
different filesystem subject and then cache the result for the original one.

The required invariant is that every record in one sealed traversal is read from
the root and object-subdirectory subjects that were bound before traversal. A
later call may reuse the result only if those same subjects are still present.

## Decision

`codefire-store` uses exact `cap-std = 4.0.2` and
`cap-fs-ext = 4.0.2` dependencies for sealed validation.

The same component-wise no-follow opening policy is used by ordinary object
write/read and full-store scan entry points. A symlink at the absolute object
root or an intermediate object subdirectory is therefore rejected rather than
canonicalized into a different store.

- Validation opens and retains a capability handle for the canonical objects
  root.
- At bind time it records every known object subdirectory, every additional
  directory already present under the root, and the absence of missing known
  subdirectories. Existing directories are retained as capability handles.
  The twelve built-in subdirectories are fixed; at most 128 additional
  extension subdirectories may be bound. Exceeding that limit is an explicit
  `InvalidSealedCommit` failure before any handles enter the cache.
- Commit, parent, root, and reachable-reference records are opened relative to
  those pinned subdirectory handles. The final file component is opened with
  no-follow semantics.
- The validation-cache subject contains root identity plus the ordered
  subdirectory name/identity-or-missing set. Root or child replacement between
  calls therefore creates a different subject instead of hitting an old cache
  entry.
- Before and after traversal, the ambient root path and every bound child path
  are compared with the pinned identities. Staged cache entries are published
  only after the final comparison.
- Callers that must consume a validated commit or publish mutable authority can
  request a `ValidatedSealedCommit` token. The token owns clones of the exact
  root and subdirectory handles used by validation; commit, root, and reachable
  object reads stay descriptor-relative and recheck the complete ambient
  subject before and after each consumption. Its explicit
  `ensure_subject_identity` check is used immediately before and after a mutable
  visibility point, closing the validation-to-use path-reopen window. A token is
  not issued as a path-only proof and stable identity unavailable platforms
  cannot use its publication recheck.
- Full-store scans use the same bound subject and final identity comparison.
  Object publication separately validates and syncs a create-new temporary
  record, publishes by no-replace hard link, and rechecks root/subdirectory
  identity after either publication or idempotent reuse. Serialized object
  writes and reads share a 64 MiB cap; reads are final-component no-follow,
  non-blocking, and regular-file-only.
- Platforms without a stable directory identity may use pinned handles for one
  traversal, but results are not reused across calls.
- A cache retains only its current object-store subject. Binding a different
  root, root identity, or subdirectory identity set evicts the earlier subject's
  validation entries and handles. Returning to it recomputes validation rather
  than growing retained descriptors with every repository visited.

Object files remain content-addressed and immutable by the cooperative store
contract. This decision binds directory subjects and read handles; it does not
turn the cache into a detector for an in-place rewrite of an already validated
object file.

Filesystem binding does not replace semantic validation. Current root schema 2
records are decoded and passed to the authoritative `codefire-core` sealed
snapshot predicate before the store accepts certificate agreement. Legacy root
schema 1 remains an explicit read path; unknown root schemas and future
commit/root object versions fail closed. ADR-012 separately versions evolving
cross-root predicates: current `sealed-snapshot-v1` commits require endpoint
closure, while profile-absent historical commits retain their weaker legacy
claim without weakening structural or capability-bound validation.

The regression matrix includes a root swap to a complete alternate store and
back, a commits-subdirectory swap and back, replacement of a commits
subdirectory between cache calls, and post-validation replacement of a root or
root-object subdirectory by a newly created directory containing identical
records. The token counterexamples prove that equal content at the same ambient
path is not the validated filesystem subject.

The dependency versions are exact so this filesystem-security boundary cannot
change through an unreviewed semver-compatible upgrade. They support the
workspace MSRV of Rust 1.75; the store test and Clippy gates run on that
toolchain.

## Impact

One validation-cache instance retains one root handle, the twelve known
subdirectory states, and at most 128 extension-subdirectory handles for its
current store subject. Switching subjects evicts the earlier pins and cached
graph results. Descriptor use is therefore bounded while shared-ancestor reuse
within the current store remains available.

One live `ValidatedSealedCommit` independently retains one clone of that root
handle and the same bounded subdirectory handle set until the caller completes
consumption/publication. This intentional lifetime prevents a cache eviction or
ambient path replacement from severing the proof between validation and use.

The two capability crates and their platform support crates become direct
dependencies of `codefire-store` as well as the Rust CLI. The implementation
stays safe Rust and shares the already reviewed versions used by ADR-009, but it
increases dependency-review and compilation surface. Windows uses stable volume
and file identities; Unix uses device and inode identities.

## Alternatives considered

- Root identity checks only at traversal start and end: rejected because a
  swap-back ABA is invisible and subdirectory identity is absent from the cache
  key.
- Disable the validation cache: rejected as a complete fix because uncached
  ambient reads can still mix filesystem subjects during one traversal, and it
  discards shared-DAG reuse without closing that race.
- Pin only the object root and open subdirectories for each read: rejected
  because a child directory can be swapped and restored independently of the
  root.
- Compare mtimes or directory listings: rejected because they are weaker than
  descriptor identity and create platform-specific timestamp assumptions.
- Implement direct `openat`/`openat2` and Windows handle traversal: rejected for
  now because it requires unsafe, platform-specific code and duplicates the
  reviewed capability abstraction.

## Removal or replacement

Remove these direct store dependencies only after a replacement provides
reviewed, cross-platform descriptor-relative traversal, final-component
no-follow reads, stable root/subdirectory cache subjects, explicit behavior for
missing subdirectories, and the same swap-back and between-call race fixtures.
Any replacement must preserve the Rust 1.75 gate or deliberately update the
workspace MSRV in the same decision.
