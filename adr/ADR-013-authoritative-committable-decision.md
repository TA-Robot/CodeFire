# ADR-013: Make committable a typed authoritative decision

## Status

Accepted for CFR-161 implementation.

## Context

CodeFire records verification facts in `Verification`, but it also stores a string `result` and
lets CLI consumers reinterpret the fact collections. `build_verification` previously applied five
policy booleans when producing `result`; exit codes, diagnostics, text, next actions, state, and
commit then applied partially different rules.

The divergence is executable. With `require_no_required_fires: false` and one open required Fire,
the final Stage 1 installed binary returned process exit 0, `result=passed`, envelope `ok=true`, and
next action `commit`, while the same envelope contained a `blocking=true` diagnostic. Commit dry-run
reported `blocked=false` and planned publication. This violates the consistency predicate in
`docs/05_consistency_model.md` and makes rendering participate in validity.

The persisted policy format already contains the boolean fields, so silently removing them or
changing object shapes would create an unrelated format migration. Historical profile-less commits
also require non-destructive readability.

## Decision

Core owns one typed `CommittableDecision` derived from raw `Verification` facts. It contains:

- `outcome`: passed or failed;
- `blocking_reasons`: ordered, typed reason classes;
- `warning_reasons`: ordered, typed reason classes;
- `primary_reason`: the first blocking reason in the stable priority order.

The stable blocking priority is:

1. open required Fires;
2. policy-required missing Trace links;
3. stale Resolutions;
4. missing or invalid evidence references;
5. duplicate Atom IDs;
6. failed required verification commands.

The following are non-relaxable safety-floor facts: open required Fire, stale Resolution, missing
evidence, duplicate Atom ID, and failed required verification command. Existing boolean fields
`require_no_required_fires`, `require_no_stale_resolutions`, `reject_duplicate_atom_ids`, and
`require_verification_success` remain in the version-1 policy object for wire compatibility, but a
working configuration may not set them to `false`. Programmatic or deserialized false values cannot
downgrade the decision.

Missing required links are the one current policy-controlled reason. With
`require_trace_completeness=false`, the facts remain recorded and become typed warnings; they do not
appear in `blocking_reasons`. A verification command explicitly configured with `allow_failure=true`
does not enter `failed_checks`; once a check is a failed required fact it is not relaxable.

The persisted `Verification.result` and certificate `result` remain compatibility projections of
the typed outcome, so this decision does not itself change the verification, root, or commit object
schema. Strict sealed validation recomputes the same safety-floor decision and rejects a recorded
result that used a false policy field to hide blocker facts. A historical clean snapshot containing
one of those false fields remains readable when its facts satisfy the safety floor; opening it for a
new write requires correcting the working policy. `legacy-unversioned` history retains ADR-012's
weaker read contract.

Every consumer must call or receive this decision. Exhaustive matches map typed reasons to process
exit codes, diagnostic codes/severity, human labels, actions, state, commit gate, and certificate.
No consumer may infer validity directly from a fact collection or silently map an unknown reason to
success.

## Impact

Unsafe working policy relaxation becomes an explicit configuration error before immutable object
planning. Repositories using only `require_trace_completeness=false`, including both Algorithm
dogfood repositories, remain supported.

Raw facts remain observable even when non-blocking, preserving policy explainability. The existing
JSON `result`, fact arrays, exit taxonomy, certificate shape, and object IDs keep their formats; the
behavioral correction is that they can no longer contradict one another.

A new blocker class requires adding a core reason variant. Exhaustive projection matches and the
decision-table tests then force each surface to choose a mapping before compilation and validation
can pass.

## Alternatives considered

- Keep boolean relaxation and only make diagnostics non-blocking: rejected because it would allow
  required Fire, stale Resolution, duplicate identity, or failed required checks to be sealed,
  contradicting the product's committable predicate.
- Reject every policy object containing false, including clean historical seals: rejected because
  the snapshot can satisfy the safety floor and authenticated history must not be reinterpreted more
  strictly than necessary.
- Add more consumer-specific tests without a core type: rejected because independent predicates
  would remain and drift again when a reason is added.
- Add `decision` to persisted verification immediately: rejected because a new object/root schema is
  unnecessary to close the current semantic split; the string result remains a checked projection.

## Removal or replacement

This decision can be replaced only by a versioned domain decision with the same or stronger safety
floor, explicit compatibility handling, stable reason identity, and exhaustive consumer mappings.
Relaxing a safety-floor class requires a new product consistency model and semantic profile; it
cannot be achieved by re-enabling a version-1 boolean.
