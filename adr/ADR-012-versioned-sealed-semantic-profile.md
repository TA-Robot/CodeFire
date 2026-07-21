# ADR-012: Version sealed-snapshot semantic validation

## Status

Accepted for the Rust sealed-commit boundary.

## Context

Content-addressed object records prove bytes and references, but the semantic predicates applied
across those records can evolve. CFB-123 added a closed-world Atom-index/Trace-Graph endpoint
predicate. Applying that new predicate retroactively to every historical commit stranded both
Algorithm dogfood repositories: an earlier Python-derived Atom to Rust explicit-marker migration
had retained a valid `CODE:<path>::class:<symbol>` link after the corresponding derived Atom
disappeared from the Rust index. The affected histories remained hash-valid and had been accepted by
the writer contract of their time, but `status`, `branch list`, `doctor`, and `migrate` could no
longer traverse them.

`root_schema_version` cannot identify this boundary. One affected history uses legacy root schema 1
and the canonical in-repository history already uses root schema 2. Reinterpreting root schema 2 as
if it had always promised every future cross-root invariant is therefore unsafe and incompatible.

## Decision

Current Rust commit payloads carry:

```json
"semantic_validation_profile": "sealed-snapshot-v1"
```

The profile is a separate version axis from commit object version, root schema version, Atom
extractor/hash schema, product version, and signature policy.

- `sealed-snapshot-v1` requires the Atom-index/Trace-Graph endpoint relation to be closed and
  unambiguous, in addition to all existing structural, reference, root-schema, certificate, and
  schema-2 semantic checks.
- A missing profile means `legacy-unversioned`. It preserves only the predicates that historical
  writers actually recorded. The store still checks object identity/hash, commit/root versions,
  required roots and shapes, reachable references, parent traversal, schema-2 domain/certificate
  agreement, and every other pre-existing invariant; only the later endpoint-closure predicate is
  not claimed for that commit.
- Unknown or non-string profile values fail closed. A current Rust writer never omits the field.
- Each parent is interpreted under its own profile. A strict current commit may retain a
  legacy-unversioned parent without rewriting authenticated history; the current commit's own roots
  must pass the strict profile.
- Missing profile is compatibility metadata, not evidence of age, authenticity, or current semantic
  correctness. Commit/request authentication remains a separate boundary.

The Rust Python extractor restores the documented derived class/function/method IDs needed by the
historical graph and advances `AtomIndex.hash_schema_version` to 4. Schema 4 retains the restored
derived IDs while masking string/comment content before declaration and marker recognition. On unchanged source, scan treats
that extractor delta as a bounded tool migration rather than ordinary change Fires. A normal
scan/verify/commit produces a strict-profile bridge commit and retains the legacy parent chain.

Doctor and migration reports expose a validated profile-absent head as non-blocking legacy
compatibility debt. They must not label it as current-semantic proof. New current-profile dangling
graphs remain blocking at scan, candidate publication, branch consumption, and remote boundaries.

## Impact

Existing object bytes and IDs are not rewritten. Repositories can recover by creating one normal
strict bridge commit after current worktree validation. The compatibility read path accepts a wider
historical semantic set than the current writer, so callers must distinguish legacy and current
claims rather than treating successful structural traversal as proof of endpoint closure.

Every new cross-root invariant must decide whether it is compatible with `sealed-snapshot-v1` or
requires a new profile. Adding a predicate outside profile dispatch and applying it to old commits is
not allowed. Removing a predicate from an existing profile is also not allowed.

The Python fallback remains a legacy compatibility writer until its full root/extractor/conformance
contract is upgraded; its profile-absent output cannot be represented as current-profile proof.

## Alternatives considered

- Gate endpoint validation on root schema 2: rejected because the canonical stranded history already
  uses root schema 2.
- Rewrite old Atom/Trace/commit objects and branch heads: rejected because it destroys
  content-addressed history and signature/provenance identity.
- Permanently allow `CODE:<path>` endpoints without Atom membership: rejected because required-link,
  Fire, Resolution, and certificate semantics require endpoints to resolve to actual indexed Atoms.
- Drop or retarget the Algorithm link manually: rejected as the product fix because recursive parent
  validation would still fail and the documented derived symbol still exists.
- Disable the CFB-123 predicate globally: rejected because it would allow new misleading sealed
  graphs and erase the Stage 1 correctness invariant.

## Removal or replacement

This profile may be replaced only by a version registry that keeps the same per-commit dispatch,
unknown-version failure, legacy claim distinction, strict-current writer rule, and mixed-profile
parent fixtures. Removing legacy compatibility requires an explicit support-window decision and a
verified export/migration path; it cannot be inferred from repository age.
