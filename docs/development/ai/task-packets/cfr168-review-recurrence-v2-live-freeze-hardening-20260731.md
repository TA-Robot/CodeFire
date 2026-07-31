# CFR-168 Review Recurrence v2 — Live-Freeze Hardening

Status: Frozen for independent preflight; implementation is not authorized
until the manager records the preflight disposition.

Owner: CFR-168 review-governance lane.

Base revision: `00ad88562068a6101a7530b84d5f52b0cff559f0`.

## Why this repair exists

The first Cargo/KGI live-manifest extraction audit found five defects before a
manager-owned live anchor was written:

1. `derivations[].locus_existed_in_predecessor` describes symbol/locus
   existence, while `recurrence_v2_decision.py` compares it with Git **path**
   existence. A new symbol inside an existing file must honestly record
   `locus_existed_in_predecessor: false`, but the current check requires
   `true` because the file existed. This makes sound Cargo and KGI derivations
   fail and conflates a human semantic assertion with a Git fact.
2. A manifest freezes each claim's `locus_path`, but the loader does not
   require the matching derivation's `path` to equal it. A record author can
   therefore move predecessor derivation to an easier unrelated path while
   keeping the frozen claim ID.
3. Bootstrap manifests do not freeze `origin_generation`. With no prior v2
   record to compare, a bootstrap author may assign any origin below the
   current generation, and the incorrect value is then inherited by every
   successor.
4. JSON Schema `uniqueItems` does not make object key fields unique. Manifest
   obligation, claim, and validation rows with the same ID but different
   content pass schema validation and are silently collapsed by dictionaries.
   Record derivation and validation IDs have the same gap; duplicate timed
   validation rows can overwrite one another by ID.
5. `live-arm-anchor.v1.json` has a documented shape, but no registered JSON
   Schema. The loader checks the top-level schema string and selected pin
   fields only. It does not reject duplicate arm rows or enforce the documented
   generation pin. Committing live manager authority on that surface would make
   malformed or ambiguous anchors admissible.

The same audit exposed an operating ambiguity, which this packet resolves
without weakening evidence: the frozen Cargo/KGI tables include destructive or
expected-failure negative controls but do not provide executable argv for
them. A timed validation is an unmodified-candidate command whose required
outcome is exit zero. Independent mutation/held-out evidence belongs to the
manifest claim set and the report/derivation/observation evidence; it must not
be converted into a passing timed validation by suppressing an expected
failure. Candidate-authored tests may be required timed validations, but they
can never be the sole evidence for an independent negative-control claim.

No live anchor, v2 live record, or v2 retained live result exists at the base,
so the record-schema addition below has no live authority to migrate.

## Objective

Make the first Cargo/KGI live freeze structurally unambiguous: bind path facts
separately from locus judgements, bind derivations and obligation origins to
the frozen manifest, reject conflicting semantic keys, validate live manager
anchors as closed committed data, and state exactly where independent
negative-control evidence belongs.

## Scope

### F1 — separate path fact from locus judgement

- Add required boolean `path_existed_in_predecessor` to each v2 derivation.
- Keep `locus_existed_in_predecessor` as the reviewer's semantic statement
  about the stable symbol/locus. The gate must not claim to infer it from Git.
- Compare Git tree-entry presence only with
  `path_existed_in_predecessor`.
- Require every record derivation's `path` to equal the matching frozen
  manifest claim's `locus_path`. A mismatched path is structural failure even
  when both paths exist and the evidence is otherwise well formed.
- Join each derivation to the frozen claim rather than treating `claim_id` as
  the whole claim. Emit the manifest-owned `kind`, `obligation_id`, and
  `locus_path` in deterministic I1 result coverage so `kind` is not parsed and
  discarded. `kind` remains a manager/reviewer semantic classification; the
  gate must not pretend it can infer parity from prose.
- Continue binding `path`, `locus`, predecessor behaviour, candidate behaviour,
  disposition, action, and exact evidence references.
- Document the pre-live schema break explicitly. Do not silently reinterpret
  old JSON bytes.

Required counterexample: a predecessor contains `product.txt`, the candidate
adds a new stable symbol inside it, and the derivation records
`path_existed_in_predecessor: true` plus
`locus_existed_in_predecessor: false`. That row must not produce a structural
finding. Flipping the path boolean must produce
`predecessor-path-existence-contradicted`. A byte-identical pre-existing locus
must still require an I1 derivation. Repointing the derivation to an unrelated
existing path must fail with a dedicated manifest/derivation path mismatch.

### F1b — bind bootstrap obligation origins

- Add `origin_generation` to every manifest prior-obligation row.
- Require the record row's origin to equal the manifest value in every
  generation.
- For a prior-v2 authority, also require it to equal the predecessor record's
  exported origin. For a legacy bootstrap, the independently preflighted
  manifest is the machine authority.
- Preserve `origin_generation < generation`.
- Extend the manager-fixed legacy-root authority with the exact first live
  review generation (`cargo: 3`, `kgi_kpi: 4`) and reject a bootstrap record or
  manifest at another generation.
- The first live Cargo manifest must bind `R1` to origin 1 and `R2`–`R5` to
  origin 2 at review generation 3. The first live KGI/KPI manifest must bind
  `A1-SUBJECT`, `A2-DENOM`, `A3-D1`, `A3-D5`, `A4-STAGE`, and `A8-C6` to
  origin 2 and the other fourteen frozen rows to origin 3 at review generation
  4. Encode these exact maps in the manager-fixed bootstrap authority as well
  as the independently reviewed manifests; a manager typo must fail closed.

Required counterexamples: bootstrap record origin differs from its manifest;
successor manifest origin differs from its predecessor record; a consistent
rooted chain retains the original origin.

### F1c — enforce semantic-key uniqueness before dictionary conversion

- Reject duplicate `obligation_id`, `claim_id`, and `validation_id` keys in a
  manifest even when the complete JSON objects differ.
- Reject duplicate record derivation `claim_id` and validation `id` keys before
  any dictionary or timed-fact conversion. Preserve the existing duplicate
  checks for artifact refs, prior/new row IDs, path reviews, and surface IDs.
- Reject duplicate anchor `arm` keys under F2.
- Emit deterministic dedicated structural findings that name the duplicated
  keys. Do not rely only on JSON Schema `uniqueItems` for object arrays.

Required counterexamples: two closure mappings for one obligation with
different validation sets; two claims sharing an ID but pointing to different
paths; two required validations sharing an ID but different argv; two record
validation rows sharing an ID where only the later row passes; and two record
derivations sharing an ID with different dispositions. None may be hidden by
last-write-wins behaviour.

### F2 — make live anchor authority schema-exact

- Add and register
  `docs/development/ai/schemas/review-recurrence-live-arm-anchor.v1.schema.json`.
- Add its path to the deciding-gate source identity. A live decision must bind
  the committed schema bytes used to validate the anchor.
- The schema is `additionalProperties: false` at every object level and
  requires exactly:
  - top-level `schema` and non-empty `arms`;
  - per arm: `arm`, `generation`, `predecessor_authority_commit`,
    `repair_packet`, and `repair_manifest`;
  - per artifact pin: `commit`, safe repository-relative `path`, `sha256`, and
    mode `100644`.
- Admit only `cargo` and `kgi_kpi`, at most one row per arm. Reject duplicate
  arm identifiers even when the complete rows differ and JSON Schema
  `uniqueItems` would not catch them.
- Require the selected row's `generation` to equal the current record's
  generation. Apply equivalent schema, duplicate, generation, and identity
  checks when replaying a retained generation against its historical deciding
  gate/anchor pin.
- Preserve the separate-clean-validator requirement and exact packet,
  manifest, authority, commit, mode, path, and digest comparisons.
- An absent anchor remains deliberately fail-closed.

Required counterexamples include: missing generation, wrong generation,
duplicate same-arm rows with different pins, unknown arm, extra properties at
each level, wrong mode, wrong digest, on-disk/committed mismatch, and a retained
result whose selected arm row differs from the historical anchor blob.

### F3 — freeze the negative-control/timed-validation boundary

Update the v2 operating documentation so live manifest preflight applies this
set rule:

- `parity_closure_claims` is set-equal to the independently reviewed
  predecessor-parity and held-out/negative-control claims. The review report
  and derivation/observation references carry their exact before, after, and
  mutation evidence.
- `required_validations` contains only exact argv that run on the clean,
  unmodified candidate and are required to terminate with exit zero. Broad and
  focused candidate tests may appear here.
- Every prior obligation has at least one semantically relevant passing timed
  validation in a bootstrap manifest, but that result alone is never closure.
- An expected-rejection CLI invocation, a throwaway-tree mutation, or a manual
  held-out probe is not rewritten as a passing command merely to satisfy timed
  evidence. If a manager later provides a content-bound harness whose own exit
  zero means all expected outcomes were observed, that harness may be frozen as
  a validation; no such harness is asserted by this repair.
- Independent manifest preflight must reject an obligation whose claim set or
  passing validation mapping is semantically incomplete. The machine still
  proves set equality, not semantic completeness.

This adjudication supersedes no immutable historical packet or report. The
old Cargo/KGI recurrence packets remain semantic extraction sources and need
new v2 review-control packets after their canonical port candidates exist.

## Allowed write set

- `docs/development/ai/schemas/review-recurrence-control.v2.schema.json`
- `docs/development/ai/schemas/review-recurrence-repair-manifest.v1.schema.json`
- `docs/development/ai/schemas/review-recurrence-live-arm-anchor.v1.schema.json`
- `docs/development/ai/schemas/README.md`
- `docs/development/tools/recurrence_v2_model.py`
- `docs/development/tools/recurrence_v2_authority.py`
- `docs/development/tools/recurrence_v2_decision.py`
- `docs/development/tools/recurrence_v2_evidence.py`
- `docs/development/tools/recurrence_v2_fixture.py`
- `docs/development/tools/recurrence_v2_history.py` only if retained-anchor
  schema/generation plumbing requires it
- `docs/development/tools/check_review_recurrence.py` only if canonical schema
  loading/plumbing requires it
- `docs/development/tools/test_ai_tools.py`
- `docs/development/ai/review-recurrence/README.md`
- one implementation result under
  `docs/development/ai/swarm-results/cfr168-review-recurrence-v2-live-freeze-hardening-*/`

Any additional path requires a manager scope decision before editing.

## Non-Goals

- Do not author Cargo/KGI manifests, canonical ports, anchors, live records, or
  results in this repair.
- Do not edit accepted packets, immutable reviews, Cargo/KGI candidates, or
  product code.
- Do not weaken the clean-validator, canonical-port, two-generation,
  full-history replay, artifact-binding, or timed-evidence rules.
- Do not add a finding quota or claim that schema validation proves semantic
  completeness, honest execution, or reviewer independence.
- Do not invent a negative-control harness or artificial second-generation
  repair byte.

## Validation

From the repository root:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  docs.development.tools.test_ai_tools.ReviewRecurrenceTests
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  docs.development.tools.test_ai_tools
PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile docs/development/tools/*.py
python3 docs/development/tools/check_ai_contracts.py
python3 docs/development/tools/check_ai_acceleration_docs.py
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_ai_development.py
git diff --check
```

The implementer must report the measured test count and complete outcomes,
not copy a historical number.

## Handoff and acceptance

Implementation is one committed candidate based on the manager-accepted
packet revision. The result must enumerate each changed path, explain schema
compatibility, and report all validations exactly. A fresh independent
report-only reviewer must review the exact candidate and specifically attempt
the F1/F2 counterexamples and F3 boundary reversal. Integration requires zero
unresolved production or bypass findings; test-only gaps must be repaired and
rereviewed rather than waived.
