# CFR-168 live-freeze hardening implementation-feasibility preflight

## Verdict

**PASS — implementation feasible; unresolved feasibility items: 0.**

The exact packet now removes every implementer choice raised by the prior
feasibility review. Every F1/F2/F3 requirement has an owner inside the allowed
write set, an acyclic implementation order, and a reusable or explicitly new
counterexample surface. No implementation edit was made by this review.

## Frozen review input

- Target commit: `65beb73dd2632cca69eec44354eadca2d9f53b2f`
- Packet:
  `docs/development/ai/task-packets/cfr168-review-recurrence-v2-live-freeze-hardening-20260731.md`
- Packet SHA-256:
  `05ea8c55355ba8674d18be04c0792de2990cdcd981de35c68790ff2c632af20f`
- Review mode: read-only implementation-feasibility audit in a clean isolated
  worktree; only this durable report is committed.

The packet's independent-preflight criterion was applied literally: this
review asks whether every requirement is implementable without inventing an
authorization-affecting choice, not whether the current implementation already
conforms.

## Closure of the four previously unresolved choices

| Prior unresolved choice | Packet resolution | Feasibility disposition |
| --- | --- | --- |
| Exact deterministic I1 output was unspecified | F1 fixes `metrics.i1.claim_coverage`, its complete closed row, ASCII `claim_id` order, all eight `derivation_next_action_counts` keys including zero values, and the exact definition of `blocking_derivations`. | Closed. Schema, decision, render, replay, and fixture assertions can share one literal shape. |
| The first live KGI/KPI origin map was incomplete | F1b fixes all six generation-2 IDs and all fourteen generation-3 IDs, fixes the live review generation to 4, and requires the manager authority and manifest to agree. | Closed. `LegacyRoot` can carry the complete immutable map and parsers/evaluation can compare it without reviewer inference. |
| Timed timestamp/path rules left platform and parsing choices open | F1e fixes `YYYY-MM-DDTHH:MM:SS.ffffffZ`, Python-valid UTC parsing, lexical POSIX normalization, component-prefix containment, root equality, distinct external log/result paths, and numeric tolerances. | Closed. The implementation needs no filesystem-dependent replay or timestamp-format policy choice. |
| `live_arm_pin` still required an unsupported schema union design | F1g fixes one always-present closed object: unavailable with both nullable members null, or verified with both exact closed objects present. Schema uses the evaluator-supported object-or-null type list; semantic validation fixes the cross-field combinations. | Closed. No `oneOf`, schema-engine extension, or alternate wire shape is needed. |

## Requirement ownership and counterexample surfaces

All paths below are explicitly in the packet's allowed write set.

| Requirement | Owning implementation paths | Required dependency edge | Reusable or explicitly new test surface | Result |
| --- | --- | --- | --- | --- |
| F1 path fact/locus separation, canonical relative paths, shared tree-entry fact, claim join, exact I1 output | control/result schemas; `recurrence_v2_model.py`; `recurrence_v2_evidence.py`; `recurrence_v2_decision.py`; `recurrence_v2_history.py`; `check_review_recurrence.py` | Schema/model fields before current and retained comparisons; shared probe before both callers; joined rows before result validation/replay | Reuse `FixtureRepo`, `build_bootstrap`, `retain_generation`, `build_second`, and `evaluate_built_v2`; add stable-symbol/path split, unsafe equal spellings, path mismatch, and forced Git-probe-error cases | Owned |
| F1b manifest-owned origin and exact bootstrap generations/maps | control/manifest schemas; `recurrence_v2_model.py`; `recurrence_v2_authority.py`; `recurrence_v2_evidence.py`; `recurrence_v2_history.py`; checker; fixture/tests | Extend `LegacyRoot` and manifest model first; compare manifest/record, then predecessor replay; canonical live map check last | Reuse bootstrap and successor builders; add record/manifest mismatch, successor/predecessor mismatch, wrong live generation, exact Cargo/KGI map, and stable-origin chain cases | Owned |
| F1c uniqueness before dictionary conversion | model parsers; evidence anchor/timed loaders; checker plumbing; fixture/tests | Duplicate collection checks must precede every index/dict/`TimedFacts` construction | Reuse fixture JSON mutation; add distinct-object duplicate IDs for obligations, claims, validations, derivations, and anchor arms | Owned |
| F1d unsafe surrogate handling | reusable boundary helper in `recurrence_v2_model.py`; loaders in authority/evidence/history/checker; schema registry in `check_ai_contracts.py`; fixture/tests | Shared safe decoder/boundary first; replace every current, retained, and validator-schema reader before schema or typed use | Add raw-byte writers/mutators for each named artifact and schema boundary; assert structured `unsafe-json-surrogate` and no traceback/partial chain | Owned |
| F1e timed detail/summary, path, timestamp, numeric, terminal, and replay relations | timed schema; `run_timed_gate.py`; `recurrence_v2_evidence.py`; `recurrence_v2_history.py`; checker/result schema; fixture/tests | Runner input validation and canonical emission first; one semantic binder next; retained replay consumes the same binder; result rendering/schema follows | Reuse `timed_result` and timed-runner tests; add table-driven relation mutations, tolerance boundaries, forward/backward jumps, lexical path escapes, invalid CLI floats, and no-subprocess-launch assertion | Owned |
| F1f strict JSON member/number/depth boundary | reusable strict decoder in `recurrence_v2_model.py`; all allowed authority/evidence/history/checker readers; `check_ai_contracts.py`; fixture/tests | Decoder and structured error type first; schema loading and artifact loading migrate to it; typed parsing remains downstream only | Add raw duplicate-member bytes, all named non-finite tokens, exponent overflow, exact depth 64/65, forced decoder recursion, and oversized integer at current/retained/schema boundaries | Owned |
| F1g closed complete result and replay | new gate-result schema; schema README; model/constants; `recurrence_v2_evidence.py` gate-source identity; checker assembly/validation; authority/history replay; registry; fixture/tests | Define closed result schema and normalized `live_arm_pin`; validate assembled current result before ALLOW; then implement historical closed validation and stable-field comparison with named exclusions | Reuse `retain_generation`, `amend_timed_result`, `mocked_authority_walk`; add render/schema helper and mutations for every stable field, closed-shape unknown member, allowed relocated paths, and recomputed diagnostics | Owned |
| F1h fixed derivation route table and blocking metrics | control/result schemas; model; decision; checker; history; fixture/tests | Route validation before authorization; exact coverage/count rendering before result validation; retained recomputation/comparison after current semantics exist | Reuse byte-identical locus and derivation tests; table-drive all invalid disposition/action pairs plus positive complete/NONE and retained action/count mutations | Owned |
| F1i schema-failure short circuit and total typed parsers | model; authority/evidence/history/checker; registry; fixture/tests | Strict decode, registered schema validation, typed parse, then decision use as four explicit stages; no downstream construction on failure | Table-drive wrong scalar/null nested objects for current and retained timed evidence plus representative record, manifest, anchor, result, and validator-schema cases; assert no `TimedFacts` or partial chain | Owned |
| F2 schema-exact current/retained live anchor | new anchor schema; schema README; evidence gate-source identity/load/verify; authority/history/checker; registry; fixture/tests | Register and bind schema bytes first; schema and duplicate validation before arm lookup; current exact pin before result assembly; historical exact pin during replay | Reuse `live_anchor` and retained-anchor tests; add missing/wrong generation, duplicate arms, unknown arm, every-level extra property, mode/digest/blob mismatch cases | Owned |
| F3 negative-control versus positive timed-validation rule | `docs/development/ai/review-recurrence/README.md` | Documentation lands after executable terminology/shapes are final; no accepted packet bytes are rewritten | Docs consistency check against the packet's exact set rule and explicit supersession paragraph | Owned |

## Feasible implementation order

The following order avoids circular ownership and prevents temporary typed use
of newly invalid shapes:

1. Add the two schemas, update the three existing schemas, register them, and
   extend deciding-gate source identity. Keep the new result schema closed and
   use only compact-evaluator-supported keywords.
2. Add the strict decoder, surrogate/depth/number controls, canonical path and
   UTC/numeric relation helpers, immutable bootstrap generation/origin maps,
   and total model parsing primitives.
3. Migrate current and retained artifact/schema readers to the strict staged
   boundary: decode, schema validate, typed parse, decision use. Add semantic
   uniqueness before any dictionary conversion.
4. Implement the shared predecessor path probe and manifest-owned origin,
   claim-path, claim-metadata, and derivation-route binding.
5. Make the timed runner validate CLI numbers before process launch and emit
   canonical paths plus the new `result.path`; make evidence binding recompute
   all detailed relations. Reuse that binding during historical replay.
6. Validate current and retained anchors against their content-bound schema,
   including arm uniqueness, generation, and exact artifact pins.
7. Render the exact I1 metrics and closed `live_arm_pin`; validate the fully
   assembled current result before ALLOW.
8. Implement complete retained-result schema validation, exact stable-field
   replay, named machine-local exclusions, separate historical authority
   operations, and recomputed gate-relative diagnostics.
9. Update F3 operating documentation, complete all current/retained negative
   controls, then run the focused and full contract suites.

Steps 1–3 form the shared boundary foundation. Steps 4–6 can then be developed
in separate small patches, but steps 7–8 must consume their final shapes. This
is a dependency constraint, not an unresolved design choice.

## Existing-code feasibility observations

- The compact schema evaluator already accepts `type` as a string list and
  supports `object` and `null`; the packet's `live_arm_pin` representation is
  therefore expressible without changing `ai_contracts.py` or introducing
  `oneOf`.
- `GATE_SOURCE_PATHS` already centralizes content-bound validator identity, so
  the new gate-result and live-anchor schemas have a direct allowed owner.
- Existing loaders in authority, evidence, and history expose every current
  and retained JSON boundary named by F1d/F1f/F1i. They can call one model-level
  decoder without adding a new module or path.
- Current predecessor tree-entry logic is duplicated/partial, but a shared
  probe can be owned by `recurrence_v2_evidence.py` and imported by current
  evaluation and history. No edit to `recurrence_v2_git.py` or
  `recurrence_v2_topology.py` is necessary.
- `timing_evidence`, `run_gate_v2`, `TimedFacts`, and
  `bind_timed_validations` provide a direct runner-to-replay seam for F1e.
- `FixtureRepo`, bootstrap/successor/retention builders, timed-result mutation,
  live-anchor helpers, and the mocked authority walk cover the repository and
  authority topology. Raw-byte, nesting, subprocess-launch, schema-mutation,
  and result-render helpers are explicitly new test-only surfaces permitted in
  `recurrence_v2_fixture.py` and `test_ai_tools.py`.

## Allowed-path and scope audit

No requirement needs a path outside the allowed write set. In particular:

- no product source, Cargo/KGI canonical-port candidate, accepted historical
  packet, or immutable report needs modification;
- no dependency or general JSON Schema engine extension is required;
- no change to `ai_contracts.py`, `recurrence_v2_git.py`, or
  `recurrence_v2_topology.py` is required;
- the implementation result location allowed by the packet covers the durable
  implementation handoff, while this preflight report is the manager-requested
  review artifact only.

F3's semantic completeness remains independent-review authority rather than a
machine proof, and F1e explicitly retains the run-authenticity and
mutate-and-restore trust boundaries. These are truthful packet non-goals, not
missing implementation owners.

## Validation evidence

At the frozen target, before this report was added:

```text
git rev-parse HEAD
65beb73dd2632cca69eec44354eadca2d9f53b2f

sha256sum docs/development/ai/task-packets/cfr168-review-recurrence-v2-live-freeze-hardening-20260731.md
05ea8c55355ba8674d18be04c0792de2990cdcd981de35c68790ff2c632af20f

PYTHONDONTWRITEBYTECODE=1 python3 -m unittest docs.development.tools.test_ai_tools.ReviewRecurrenceTests
Ran 43 tests in 33.521s
OK
```

The passing baseline is not an implementation-conformance claim; it confirms
that the reusable fixture/test seam is healthy at the frozen target. The new
counterexamples listed above are still implementation work.

## Stopping-rule disposition

This feasibility lane reports zero unresolved contradictions, scope conflicts,
missing owners, dependency gaps, or authorization-affecting implementer
choices. Its portion of independent preflight is complete on the exact target
and packet hash above. Later defects must be classified as contract omission,
implementation nonconformance, or repair-introduced regression as required by
the packet; they are not grounds for an arbitrary new feasibility-review loop.
