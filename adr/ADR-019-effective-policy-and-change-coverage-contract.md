# ADR-019: Effective Policy Provenance and Change Coverage Contract

Status: Implemented

Date: 2026-07-19

Owners: Core, CLI, Store, and Verification Maintainers

Related: CFR-167, RVC-008, RVC-016, ADR-013, ADR-018

## Context

ADR-013 made the committable outcome authoritative, but its inputs still lost important meaning.
`VerificationPolicy` retained only final values, so a built-in default and an explicit repository
setting were indistinguishable. `ScanResult.non_atom_changed_files` was the manifest difference after
removing every path that contained a changed Atom. That made a changed Atom hide unrelated bytes in
the same file, and it could not identify a deleted Atom or a change outside an unchanged Atom.

The current Atom selector model does not persist byte or line spans. Claiming complete span coverage
would therefore be false. Conversely, treating all non-Atom bytes as blockers would change the
product's safety policy without an adopted review/evidence workflow.

## Decision

### Effective policy

Core owns `effective-verification-policy-v1`. Every effective rule has a stable ID, source
(`built_in_default`, `repository_config`, or `cli_override`), effective JSON value, applicability,
and safety class. Rules are sorted by ID before hashing; required-link input order does not affect
the contract or digest. Duplicate verification command IDs are rejected because they cannot provide
a stable rule identity.

Repository parsing records provenance before final values are projected into
`VerificationPolicy`. The provenance model defines CLI precedence even though no new general-purpose
policy override flags are introduced by this ADR. A rejected safety-floor relaxation reports its
stable rule ID, repository source, and rejected adoption. Existing safety-floor behavior is not
relaxed.

The effective-policy digest is a decision-subject digest, distinct from the historical semantic
`policy_hash` in Resolution bases. An effective value change continues to stale a Resolution through
the semantic policy hash. A source-only rewrite changes new verification/seal identity and its
explanation, but does not falsely stale a Resolution whose semantic predicate is unchanged.

### Change coverage

Core owns `file-relation-v1` `ChangeCoverage`. Every manifest-changed path is deterministically one
of:

- `non_atom_file`: neither base nor current index has an Atom on the path;
- `deleted_atom`: a deleted path had one or more base Atoms;
- `out_of_atom_change`: the file changed while its indexed Atom hashes did not;
- `atom_change_with_unclassified_spans`: at least one current Atom changed, while bytes outside its
  selector cannot yet be proven covered.

The contract contains normalized base/current/changed Atom IDs, blob identities, a summary, an
optional typed tool-migration fact, and a digest. `span_precision=file_only_no_span_claim` is
mandatory. A source-unchanged hash-schema migration has zero changed-file entries and one typed tool
migration; it is never presented as a source edit.

The current coverage disposition is the built-in effective rule
`coverage.unclassified_changes=allow_with_disclosure`. This ADR adds disclosure and binding, not a
new blocker. Path/class-specific require-review or deny rules require a later policy ADR and a
workflow that can satisfy them.

### Projection and seal binding

Scan JSON/text, verification JSON/text, diagnostics, explain, commit plans, typed
`CommittableDecision`, policy roots, verification roots, and commit certificates project the same
contracts or their bound digests. New seals store the complete effective policy in both the policy
and verification roots, store coverage in verification, and bind both digests in the certificate.

Sealed validation checks contract profile, ordering, unique IDs/paths, values, classifications,
summary, digest, policy semantics, root-to-root equality, and certificate-to-verification equality.
Changing a rule, coverage fact, result projection, or any bound digest while rebuilding object IDs
still fails cross-root semantic validation.

These fields are additive claims inside existing version-1 roots under root schema 2. This is safe
for the current writer because coverage is disclosure-only and effective policy duplicates the
already enforced predicate. Historical records that omit both contracts remain readable and make no
provenance or coverage claim. A one-sided policy/verification claim or one-sided certificate digest
is invalid. A future blocking coverage policy must use an explicit schema/profile evolution rather
than being smuggled into this additive contract.

## Consequences

- Default and explicit repository policy are distinguishable without changing the authoritative
  committable predicate.
- Pure non-Atom, out-of-Atom, mixed/unproven-span, deleted-Atom, and tool-migration cases have stable
  machine-readable identities.
- The model explicitly admits current span uncertainty instead of calling an entire changed file
  Atom-covered.
- Coverage construction is linear in changed files plus indexed Atoms and uses ordered maps/sets for
  deterministic output.
- Exact byte/span classification remains unavailable until selectors persist owned spans.

## Rejected Alternatives

- Treat a path with any changed Atom as fully covered: makes an unsupported whole-file claim.
- Block every non-Atom or unknown span immediately: provides no adopted evidence/review escape path
  and would make normal generated/config changes unusable.
- Store only counts: cannot reconstruct path/Atom relation or reject classification drift.
- Put final values only in the certificate: loses rule source and prevents policy-root comparison.
- Bump root schema solely for disclosure-only additive fields: needlessly rejects historical/current
  readers; a schema bump is reserved for a newly required or blocking semantic claim.

## Verification

- golden effective-policy digest and required-link order independence;
- default/repository/CLI precedence and rejected relaxation provenance;
- pure non-Atom, out-of-Atom, mixed/unclassified-span, deleted-Atom, and tool-migration fixtures;
- scan, verify, diagnostic, explain, commit-plan, policy-root, verification-root, and certificate
  projection tests;
- root/verification/certificate digest drift and semantic rule drift rejection;
- full Rust/Python compatibility gates, fresh install, installed reversal probes, and CodeFire
  dogfooding before CFR-167 is closed.
