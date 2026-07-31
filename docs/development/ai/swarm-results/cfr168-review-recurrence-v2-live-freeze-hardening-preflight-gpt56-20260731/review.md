# CFR-168 Review Recurrence v2 Live-Freeze Hardening — Independent Preflight

Reviewer: GPT-5.6 (Codex), independent report-only preflight lane
`cfr168-review-recurrence-v2-live-freeze-hardening-preflight-gpt56-20260731`.
Date: 2026-07-31.

This review evaluates the frozen repair packet, not an implementation. The lane
did not modify schemas, tools, tests, live manifests, anchors, records, product
code, or prior review evidence.

## Exact review identity

| Item | Exact value |
|---|---|
| Target commit | `fabb77ae44ee95411b5505a90366d031f4b2a6cd` |
| Target tree | `c4791a0782e3d44b9b004bb9abe5f0e98d911075` |
| Target parent | `88d0d965c1ca64827afa5199a2c2f26d2d849c29` |
| Target subject | `docs(cfr168): classify deciding gate local paths` |
| Packet | `docs/development/ai/task-packets/cfr168-review-recurrence-v2-live-freeze-hardening-20260731.md` |
| Packet SHA-256 | `d8f69cebec9458948ed8fde0748980137991cfd1942db48cf62a5a28f4dae227` |
| Packet-declared implementation base | `00ad88562068a6101a7530b84d5f52b0cff559f0` |
| Accepted v2 contract read for consistency | `docs/development/ai/task-packets/cfr168-review-recurrence-gate-v2-repair-20260731.md` |
| Accepted v2 contract SHA-256 | `e08e736cbdb7f48fc131ed2b542db951fbff32ebe1b29d399bfe83172bdea6f6` |

The target worktree was clean before this report was added. I read the frozen
packet and accepted v2 contract in full, then traced the affected current
schemas, model/parser, authority walk, decision derivation, evidence binding,
history replay, CLI result emission, timed runner, schema registry, fixtures,
and focused tests.

## Findings

None. I found no unresolved production, bypass, contradiction, or
implementability defect in the frozen packet.

The packet's motivating defects are present in the current implementation. In
particular, current decision code compares predecessor Git path presence to the
semantic locus boolean; manifest/record semantic IDs can be collapsed by
dictionaries; committed JSON readers use permissive `json.loads`; timed facts
trust summary booleans; the live-anchor loader selects the first matching arm
without a registered closed schema; retained-result replay compares only a
subset of authority-bearing output and has no registered closed result schema;
derivation `next_action` is parsed but not routed or emitted; and schema-invalid
timed data can still be consumed far enough to raise or construct partial
facts. The packet addresses those concrete defects rather than introducing
unrelated policy.

## Adversarial coverage assessment

| Area | Preflight assessment |
|---|---|
| F1 path fact and semantic locus | Acceptable. `path_existed_in_predecessor` is a separate required Git fact, while `locus_existed_in_predecessor` remains explicitly human semantic evidence. Exact manifest-to-derivation path equality, canonical spelling, and one shared fail-closed current/history path probe close both the unrelated-path substitution and silent-probe-skip routes. The new-symbol-in-existing-file positive control is preserved. |
| F1b bootstrap origins | Acceptable. Origin is bound manifest-to-record and, after bootstrap, predecessor-to-successor. The exact Cargo/KGI first-live generations and row maps make bootstrap authority deterministic and independently checkable. |
| F1c semantic uniqueness | Acceptable. Every key currently vulnerable to dictionary overwrite is named, checked before conversion, and paired with a distinct-object duplicate counterexample. This correctly complements rather than duplicates JSON Schema `uniqueItems`. |
| F1d unsafe text boundary | Acceptable. A single recursive boundary covers current and retained records/results, manifests, timed evidence, anchors, and deciding-gate schema blobs before schema traversal. Repr-safe structured findings prevent both uncaught UTF-8 encoding failures and unsafe terminal interpolation. |
| F1e timed evidence | Acceptable. Summary booleans are derived from details; the HEAD-probe nullability shape is closed; the required result field is frozen as sibling object `result.path`; path containment and separation are recomputed; terminal state relations are exact; and retained evidence receives the same checks. The signed timestamp rule correctly permits genuine backward wall-clock movement. The frozen 2-microsecond timestamp tolerance and 1-picosecond arithmetic-relation tolerance are explicit and have at/below/above boundary tests. Non-finite or otherwise invalid runner timing inputs must fail before subprocess launch. |
| F1f strict JSON | Acceptable. Raw duplicate member names, named non-standard constants, standard exponent overflow, oversized integers, decoder recursion, and excessive nesting are rejected before authority use. Depth accounting is unambiguous: container depth 64 passes and 65 blocks. Recursive finiteness checking closes the `parse_constant`-only gap. Registration and gate-loading coverage includes the relevant current and historical authority schemas. |
| F1g retained results | Acceptable. A new registered, fully closed gate-result schema is bound into deciding-gate source identity and validates both authorization-eligible current results and every retained result before replay. The semantic replay is complete for path-independent authority/decision fields. Stable authority fields are exact comparisons; machine-local exclusions are exhaustively named as the three result paths plus each current/link `deciding_gate.source_root`; all neighboring non-path gate identity remains exact. Later-gate diagnostics are shape-checked and recomputed, while live-arm/pin/blob and invalid-path evidence remain subject to their named exact historical checks. Relocation-positive and unknown-neighbor-negative controls prevent this exception becoming an extension point. `limits` is also frozen to the provenance constant, preventing silent trust-boundary drift. |
| F1h derivation routing | Acceptable. The complete/contradicted/unavailable dispositions have one deterministic action each. Non-`NONE` derivation actions block and remain visible in I1 coverage and action counts, closing the currently parsed-but-ignored action surface without pretending to infer prose semantics. |
| F1i invalid typed input | Acceptable. Decode, registered-schema validation, typed parsing, and decision use are explicitly staged; a failing stage cannot supply a typed record, manifest, timed fact, anchor, or retained result. Parsers remain total for wrong scalar/null/nested shapes, current and retained paths share the rule, and the counterexamples require structured `BLOCK` without partial facts or tracebacks. Validator schemas use strict decoding plus the existing registry checks, avoiding an undefined self-schema recursion. |
| F2 live-anchor authority | Acceptable. A registered, source-identity-bound schema is closed at every object level, exact arm values are enumerated, semantic arm uniqueness is checked separately, generation equality is required, and the same identity rules apply to historical replay. The absent-anchor state remains fail-closed. |
| F3 negative controls vs timed validations | Acceptable. The packet explicitly supersedes the older extraction sentence without altering historical bytes. No negative-control claim may disappear: all such rows remain set-equal in `parity_closure_claims` and bind independent report/derivation/observation evidence. Only clean-candidate commands whose required outcome is exit zero enter timed validations. This avoids fabricating a passing command for an expected rejection while retaining independent negative evidence and semantic preflight. |

## Contract and implementation-boundary audit

The additions do not weaken the accepted v2 invariants for direct ancestry,
separate clean validator authority, canonical ports, two-generation recurrence,
full history replay, artifact identity, or timed evidence. F3 is a narrowly
stated precedence decision over one operating extraction rule, and it preserves
the negative-control set rather than waiving it.

The pre-live schema changes retain their identifiers, but the packet states the
reason this is safe: no live v2 anchor, record, or retained result exists at the
declared base. It also requires all record, manifest, and timed-v2 breaks to be
documented and forbids silently reinterpreting old bytes. That is an explicit
pre-live compatibility decision, not an accidental migration gap.

The allowed write set contains every implementation surface needed for the
specified repair: the three existing schemas plus new closed gate-result and
anchor schemas, schema documentation, the v2
model/authority/decision/evidence/history/fixture modules, CLI plumbing, timed
output and input validation, strict schema registration, tests, operating
documentation, and one result report. No live authority data or accepted
historical artifact needs to be rewritten. The required validation commands
cover focused and full tests, compilation, schema/docs checks, and patch
hygiene, and the handoff requires measured outcomes plus a new exact-byte
implementation review.

This report supplies the packet's primary contract-review check: there is no
remaining contradiction, missing authorization-changing cross-field rule,
scope conflict, or material implementer choice on the exact target above. The
packet correctly keeps implementation authorization conditional on the
adversarial-boundary and implementation-feasibility reviewers reaching the
same zero-unresolved disposition on these same bytes. Its three-check stopping
rule is count-free, terminates review once the named evidence is complete, and
classifies any later failure by originating stage instead of starting another
unbounded review round.

## Residual trust boundary

The packet correctly leaves semantic review quality, honest command execution,
and mutate-then-restore behavior outside the machine proof. It does not present
those limits as verified facts. Those are declared provenance limits, not open
findings against this repair contract.

OVERALL_VERDICT: ACCEPT
