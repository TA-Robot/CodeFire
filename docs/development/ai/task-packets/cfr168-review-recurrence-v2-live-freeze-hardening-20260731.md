# CFR-168 Review Recurrence v2 — Live-Freeze Hardening

Status: Frozen for independent preflight; implementation is not authorized
until the manager records the preflight disposition.

Owner: CFR-168 review-governance lane.

Base revision: `00ad88562068a6101a7530b84d5f52b0cff559f0`.

## Why this repair exists

The first Cargo/KGI live-manifest extraction audit found twelve defects before a
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
5. The current-record entry point rejects escaped lone surrogates before JSON
   Schema traversal, but manifest and retained-history loaders do not. A
   manifest surrogate inside a `uniqueItems` array reaches
   `canonical_json_bytes(...).encode("utf-8")` and raises
   `UnicodeEncodeError` instead of returning a structured `BLOCK`.
6. Timed v2 evidence carries both detailed Git probe fields and summary
   booleans, but the gate trusts the booleans without re-deriving them. A
   committed result can say `clean_before: true` beside a non-empty dirty
   status, claim clean index flags beside flagged entries, or claim unchanged
   HEAD beside different after identities and a probe error, and still reach
   `ALLOW`. The result-output path is not retained, so its claimed outside-tree
   placement cannot be recomputed either.
7. Every v2 raw JSON reader uses ordinary `json.loads`, so duplicate object
   member names are silently last-write-wins before schema, digest-field, or
   cross-field checks see them. A raw record, manifest, timed result, retained
   result, or anchor can therefore carry two conflicting `generation`,
   `sha256`, `kind`, or summary fields while the gate validates only the last.
8. Python's default JSON decoder also admits non-standard `NaN`, `Infinity`,
   and `-Infinity`. The compact schema evaluator treats them as numbers and
   comparisons such as `minimum` do not reject `NaN`, allowing non-finite timed
   durations or thresholds into committed evidence.
9. `live-arm-anchor.v1.json` has a documented shape, but no registered JSON
   Schema. The loader checks the top-level schema string and selected pin
   fields only. It does not reject duplicate arm rows or enforce the documented
   generation pin. Committing live manager authority on that surface would make
   malformed or ambiguous anchors admissible.
10. Retained-result replay compares a hand-picked subset of the committed
    result and silently ignores authority-bearing fields including
    `git_verified`, `input_schema`, `timed_validations`, and
    `authority_chain`. Mutating any of those fields in a retained generation
    currently leaves a successor at `ALLOW`. Unknown result fields are also
    ignored, and no registered closed result JSON Schema exists, so the replay
    contract can drift without a deliberate schema or compatibility decision.
11. Derivation `next_action` is required and parsed but never evaluated or
    emitted. A record can pair `derivation_complete` with `PRODUCT_REPAIR`,
    `REPAIR_CLAIM`, or a manager action and still reach `ALLOW` with no visible
    repair requirement. The explicit action surface is therefore disconnected
    from the closed decision model.
12. Some committed artifacts are consumed after JSON Schema has already found
    the shape invalid. A timed result with scalar or null `argv`, for example,
    reaches an unchecked iteration and raises `TypeError` instead of returning
    structured `BLOCK`. Schema findings currently do not reliably prevent
    partial typed facts from entering later decision code.

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
- Require claim/derivation repository paths to use one canonical safe spelling
  before either current evaluation or retained replay. Reject absolute paths,
  dot/dot-dot components, empty components, repeated/trailing separators, and
  spellings normalized by `PurePosixPath` such as `a/./b`; do not silently skip
  the Git fact check when a current path is unsafe.
- Use one shared predecessor-path presence probe for current evaluation and
  retained replay. An unsafe path or any Git tree-entry read failure is a
  structural finding; neither branch may `continue` without comparing the
  claimed path fact or recording why it was unavailable.
- Require every record derivation's `path` to equal the matching frozen
  manifest claim's `locus_path`. A mismatched path is structural failure even
  when both paths exist and the evidence is otherwise well formed.
- Join each derivation to the frozen claim rather than treating `claim_id` as
  the whole claim. Emit the manifest-owned `kind`, `obligation_id`, and
  `locus_path` in deterministic I1 result coverage so `kind` is not parsed and
  discarded. `kind` remains a manager/reviewer semantic classification; the
  gate must not pretend it can infer parity from prose.
- Emit exact `metrics.i1.claim_coverage`, sorted ascending by ASCII `claim_id`.
  Each closed row contains, in schema, `claim_id`, manifest-owned `kind`,
  nullable `obligation_id`, manifest-owned `locus_path`, `derivation_path`,
  `locus`, both path/locus predecessor booleans, predecessor and candidate
  behaviour, `evidence_state`, lexicographically sorted `evidence_ref_ids`,
  `disposition`, and `next_action`. Also emit
  `metrics.i1.derivation_next_action_counts` with keys `PRODUCT_REPAIR`,
  `RUN_CLOSURE_EVIDENCE`, `REPAIR_ORACLE`, `REPAIR_CLAIM`,
  `DOCUMENT_NON_REACHABILITY`, `MANAGER_SCOPE_DECISION`,
  `MANAGER_REQUIREMENT_DECISION`, and `NONE` (including zeros), plus integer
  `metrics.i1.blocking_derivations` equal to the number of coverage rows whose
  action is not `NONE`.
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
Equal manifest/derivation spellings `.`, `a/./b`, `a//b`, and `a/` must each
fail identically in the current generation and retained replay rather than
bypassing current tree-entry verification. A forced Git tree-entry probe error
must likewise block both paths with the same finding semantics.

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
  origin 2 and exactly `A1-PREREQ`, `A3-N1`, `A5-SCOPE`, `A5-ECHO`, `A6-N2`,
  `A6-N4`, `A7-C1`, `A7-C2`, `A7-C3`, `A7-C4`, `A8-C5`, `R1C-C-TRUST`,
  `N3-MERGE-SEAL`, and `P1-RENAME-SHALLOW` to origin 3 at review generation 4.
  Encode these exact maps in the manager-fixed bootstrap authority as well as
  the independently reviewed manifests; a manager typo must fail closed.

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

### F1d — keep every v2 JSON boundary structured on unsafe text

- Factor the existing recursive lone-surrogate rejection into one reusable v2
  boundary helper; do not leave current-record and committed-artifact readers
  with divergent behaviour.
- Apply it before JSON Schema traversal to the current record, every retained
  record replay, every current or retained gate-result replay, every repair
  manifest, every timed-result artifact, both current and retained live-anchor
  blobs, and every validator-owned JSON Schema blob consumed as authority.
- Return a deterministic `unsafe-json-surrogate` structural finding with an
  escaped/repr-safe location and message. Never interpolate the raw surrogate
  into terminal output.
- Preserve raw-byte digest and UTF-8 decoding checks. This control is in
  addition to invalid repository-path byte detection; the two boundaries are
  not interchangeable.

Required counterexamples place an escaped lone surrogate separately in a
manifest claim path, required-validation argv, retained record array, timed
result string, retained gate result, current anchor pin path, retained
historical anchor, and validator-owned schema. Each must return structured
`BLOCK` without an uncaught exception or partial result.

### F1e — derive timed-evidence summaries from their detailed fields

- Recompute and require:
  - `clean_before == (status_before == [])`;
  - `clean_after == (status_after == [])`;
  - `index_flags_clean == (index_flagged_entries == [])`;
  - `head_unchanged` iff the after revision/tree equal the captured source
    revision/tree and `head_probe_error` is null.
- Require the runner's HEAD-probe detail shape itself: a null
  `head_probe_error` iff both after revision and tree are non-null; a non-null
  error iff both after identities are null. One surviving hash beside a probe
  error, or one missing hash without an error, is not runner-produced evidence.
- Extend timed-gate v2 output with exact required sibling object
  `"result": {"path": "<absolute path>"}` next to `log`; that object has
  `additionalProperties: false`. Require cwd, Git root, `log.path`, and
  `result.path` to be absolute normalized paths; cwd must be within the
  declared Git root, while log and result must be distinct and outside it.
  Recompute both inside/outside summary booleans from those paths rather than
  trusting them. This is an explicit pre-live timed-v2 shape break and must be
  documented with the record/manifest shape breaks.
- Freeze timestamps to the runner's exact UTC representation
  `YYYY-MM-DDTHH:MM:SS.ffffffZ`: four-digit year, six fractional digits, literal
  `Z`, and a value accepted by Python's UTC `datetime` parser (so invalid dates
  and leap-second `:60` are rejected). Do not accept alternate offsets,
  fractional precision, or timezone-free strings in committed evidence.
- Use filesystem-independent lexical POSIX path checks during evidence replay.
  An accepted path is `/` or begins `/` and has no empty, `.`, `..`, backslash,
  control, surrogate, repeated-separator, or trailing-separator component;
  `PurePosixPath(value).as_posix()` must equal the original spelling.
  Containment is component-prefix containment, not string prefix and not a
  fresh filesystem `resolve()`. `cwd` may equal or descend from Git root;
  log/result must not equal or descend from it and must differ from each other.
  The runner resolves its live paths before emitting this lexical form.
- Validate the runner-owned timing model rather than accepting internally
  contradictory diagnostics: timeout must be finite and positive; monotonic
  duration and drift threshold must be finite and nonnegative; the runner must
  reject invalid timeout/threshold CLI values before starting a subprocess;
  start/end timestamps must be parseable; their signed difference must agree
  with signed wall elapsed within their represented precision with absolute
  tolerance `0.000002` seconds;
  `wall_minus_monotonic_seconds` must equal wall elapsed minus monotonic
  duration with absolute tolerance `1e-12` seconds; and
  `wall_clock_discontinuity` must equal whether the absolute delta exceeds the
  threshold. Do not require wall timestamps to be ordered: a genuine backward
  wall-clock jump is admissible when these signed relations are consistent and
  the discontinuity flag is true.
- Require terminal fields to describe one outcome:
  `status == "passed"` iff `exit_code == 0` and `timed_out == false`, and a
  timed-out result must use the runner's exit code `124`. Preserve the closed
  status enum for every other failure outcome.
- Preserve exact candidate revision/tree, argv, committed log bytes/digest,
  terminal status, timeout, exit-code, and unchanged-post-command checks.
- Apply the same consistency checks under current-gate semantic replay of
  retained generations. A historical self-contradiction is a structural
  blocker, not compatibility-authorized provenance.
- State the remaining trust boundary honestly: internal consistency and
  committed identity do not prove that an author actually ran the command or
  that mutate-and-restore did not occur during it.

Required counterexamples independently falsify each boolean/detail pair,
provide a non-null probe error with `head_unchanged: true`, pair an error with
surviving after hashes, omit only one after hash without an error, move cwd
outside the declared root, put log/result paths inside the root while setting
the booleans false, make log and result the same path, use negative
timeout/duration/drift values, falsify the wall-minus/discontinuity relations,
detach the timestamps, contradict status/exit/timed-out fields, and replay an
inconsistent retained timed result. Detached/relation-inconsistent timestamps
must block; alternate timestamp forms and lexically noncanonical or
component-prefix-escaping paths must block. Consistent forward and backward
wall-clock jumps must both pass their timing relation checks. Add boundary tests
at each frozen tolerance, immediately below it, and immediately above it. Every
real runner-produced external-path result must remain passing. Named non-finite
and exponent-overflow timeout/threshold CLI values must fail before command
execution; equivalent committed-result mutations must return a structured
`BLOCK`.

### F1f — use a strict deterministic JSON authority boundary

- Use one reusable strict v2 JSON decoder that rejects a repeated member name
  before normal object construction. Ordinary `json.loads` last-write-wins
  behaviour is not an admissible authority boundary.
- Reject non-standard non-finite numeric constants (`NaN`, `Infinity`, and
  `-Infinity`) and every otherwise-standard numeric token that decodes to a
  non-finite value (including exponent overflow such as `1e999` and `-1e999`)
  with a structured `non-finite-json-number` finding/error. A strict
  `parse_constant` hook alone is insufficient; recursively reject every
  decoded non-finite float. Schema number checks are not a substitute because
  comparisons against `NaN` and infinities do not provide the required
  deterministic boundary.
- Limit object/array container nesting to 64 levels: a root object/array is
  depth 1 and each directly nested object/array increments depth by one;
  scalars do not increment it. Depth 64 is accepted and depth 65 returns
  structured `json-nesting-too-deep`; catch an earlier decoder
  `RecursionError` as the same finding rather than a traceback.
- Convert decoder numeric `ValueError` (including Python's oversized-integer
  digit guard) into deterministic `json-number-out-of-range`. A standards-valid
  but impractically large integer token must never escape the shared boundary
  as an exception.
- Apply it to the current record and every committed current/historical record,
  result, manifest, timed result, and live anchor consumed by authority walk or
  semantic replay. Apply the same decoder to every current or historical
  record, manifest, gate-result, timed-result, and live-anchor JSON Schema blob
  loaded as deciding-gate authority; content-addressing a schema does not make
  ambiguous JSON member semantics safe.
- Return a deterministic structured `duplicate-json-object-key` finding (or a
  CLI input error before a record model exists) naming the duplicated member
  safely. Do not continue with either value and do not emit a partial retained
  authority chain.
- Keep this distinct from F1c: F1c rejects repeated semantic IDs across array
  objects; F1f rejects repeated member names inside one raw JSON object.

Required counterexamples duplicate a top-level generation and separately a
nested artifact digest, manifest claim kind, timed cleanliness boolean,
retained-result candidate field, and anchor pin digest. Each raw byte sequence
must fail deterministically before the conflicting value can influence a
decision. Separate current and retained timed-result controls inject all three
named non-finite constants and both exponent-overflow forms into
duration/threshold fields and must also fail before schema/decision use.
Duplicate members and non-finite constraints inside each validator-owned JSON
Schema must likewise fail during schema registration or gate loading rather
than being parsed last-write-wins. Add exact depth-64/depth-65,
decoder-recursion, and oversized-integer controls for current and retained
boundaries.

### F1g — make retained-result replay complete and closed

- Add and register
  `docs/development/ai/schemas/review-recurrence-gate-result.v2.schema.json`,
  and add its exact path/digest to deciding-gate source identity. It validates
  retention-authoritative parsed v2 results, is closed at every object level,
  models all stable and explicitly gate-relative fields named below, and may
  admit only the known optional CLI-added machine-local `record` path.
- Replace the result's string/object `live_arm_pin` union with one closed object
  that the compact schema evaluator can validate without `oneOf`:
  `{"state":"unavailable","anchor":null,"arm":null}` when no verified pin
  exists, or `{"state":"verified","anchor":{...},"arm":{...}}` when it
  does. `anchor` and `arm` each use the supported object-or-null type; semantic
  validation enforces unavailable iff both are null and verified iff both
  closed objects are present. The closed anchor object has exactly
  `validator_commit`, `path`, `mode`, and `sha256`; the closed arm object has
  exactly `arm`, `generation`, `predecessor_authority_commit`, `repair_packet`,
  and `repair_manifest`, using F2's exact artifact-pin shape. No other
  state/combination is admissible.
- Validate a fully assembled current parsed-v2 result before it can authorize
  `ALLOW`, and validate every retained result before semantic replay. A result
  from an unparseable input may still be emitted as diagnostic `BLOCK`, but it
  cannot become predecessor authority.
- Define one explicit retained-result replay contract. Do not maintain a
  drifting allowlist of a few convenient fields while ignoring the rest of the
  committed decision result.
- Reject unknown top-level and authority-bearing nested result fields unless a
  versioned compatibility decision explicitly classifies them as diagnostic.
  List every deliberately excluded machine-local diagnostic field; an omission
  is not an exclusion policy.
- Recompute and compare every path-independent decision or authority field,
  including result/input schema identities, record digest, `git_verified`,
  control/generation, candidate identity, the exact timed-validation facts,
  rows, metrics, successor obligations, record validity, review and integration
  decisions, blockers, findings/count, and the complete rooted authority-chain
  meaning (`kind`, root arm/commit, depth, and links).
- Compare `limits` to the exact v2 provenance-disclosure constant; changing its
  meaning requires a result-schema/compatibility decision rather than silent
  replay drift.
- Fields whose value necessarily changes with the replay location, such as an
  analysed local root path, may be excluded only by name and rationale. The
  machine-local path exclusions are exactly `analysed_git_root`,
  `validator_source_root`, CLI-added `record`, and every top-level or
  authority-link `deciding_gate.source_root`, because they are machine-local
  paths. They remain known closed-shape fields, not arbitrary extension points;
  all non-path deciding-gate state/revision/tree/artifact identity remains
  exact authority.
- `authority_chain.validator_identity_divergences` and
  `authority_chain.revalidated_generations` are current-gate replay
  diagnostics: a later gate necessarily derives different values. Validate
  their closed shapes, deliberately exclude their immutable historical bytes
  from direct equality, recompute them under the current gate, and emit those
  recomputed values in the successor result. They must never supply root or
  integration authority. Stable `kind`, root arm/commit, depth, and link
  identities remain exact comparisons.
- Fields checked by a separate exact historical authority operation—
  `deciding_gate`, `live_arm`/pin/blob, and invalid-path interval evidence—must
  be present in the closed result shape and linked to that operation, not
  silently skipped. A mutation must fail in that named operation.
- Apply the same closed-shape and semantic replay rule at every retained
  generation. A result produced by an older deciding gate remains immutable,
  but current authorization must fail closed if its authority-bearing claim is
  false under the current contract.

Required counterexamples independently mutate retained `git_verified`,
`input_schema`, one timed validation fact, the authority root/arm/link set, and
an otherwise unknown authority-looking member. Each must block a successor.
Machine-local diagnostic paths may differ only where the contract explicitly
permits them: a retained `deciding_gate.source_root` relocation must replay,
while an unknown neighboring member must block. An otherwise unmodified
retained result must still replay cleanly.

### F1h — route derivation dispositions and actions

- Freeze one deterministic derivation route table:
  `derivation_complete -> NONE`,
  `derivation_contradicted -> REPAIR_CLAIM`, and
  `derivation_unavailable -> RUN_CLOSURE_EVIDENCE`.
- Reject every other disposition/action pair. Any derivation action other than
  `NONE` is blocking and must remain visible in deterministic I1 result
  coverage and action counts; it cannot disappear behind the obligation-row
  `product_repair_required` metric.
- Preserve the evidence-state/disposition checks. This route table classifies
  what must happen next; it does not turn reviewer prose into machine proof.

Required counterexamples pair `derivation_complete` with each non-`NONE`
action, pair contradicted/unavailable dispositions with `NONE`, and mutate a
retained result's emitted derivation action/count. Every mismatch must block;
the complete/`NONE` positive fixture must remain passing.

### F1i — stop typed consumption after schema failure

- At every v2 record, manifest, timed-result, live-anchor, and retained-result
  boundary, separate decode, its registered JSON Schema validation, typed
  parse, and decision use. Validator-owned schema blobs instead use strict
  decode followed by the existing registry structural checks; do not invent a
  recursive self-validation step. If either applicable validation stage fails,
  retain the structured findings but do not consume the invalid value as a
  typed model or emit partial facts from it.
- Typed parsers must still be total over their declared input boundary: a wrong
  scalar, null, or malformed nested object returns deterministic findings and
  never an uncaught `TypeError`, `AttributeError`, `KeyError`, encoding error,
  or partial authority chain.
- In particular, timed validation binding must not construct `TimedFacts`,
  argv equality, path summaries, or log claims from a schema-invalid result.
  Current evaluation and retained replay use the same rule.
- Preserve useful independent findings when safe, but do not trade fail-closed
  execution for cascades produced from invalid assumed types.

Required counterexamples replace timed `argv` with a number and null, and
replace each nested `source`, `worktree`, `log`, and `result` object with a
wrong scalar/null in current and retained evidence. Add representative
malformed record, manifest, anchor, retained-result, and validator-schema
shapes. Every case must terminate with structured `BLOCK`, no traceback, and
no `TimedFacts` for the invalid artifact.

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

This F3 adjudication **supersedes the operating extraction rule** in the
accepted v2 contract §“Prior obligations” that said required validation
IDs/argv are derived from both the packet Validation table and its independent
negative-control table. The accepted packet's bytes and every immutable
historical report remain unchanged, but the two collections are no longer
conflated: every expected-failure/mutation row is preserved in the frozen
claim set and must bind exact independent report/derivation/observation
evidence; only positive, clean-candidate, exit-zero argv enter timed required
validations. This is a non-weakening replacement, not permission to omit a
negative-control row. The old Cargo/KGI recurrence packets remain semantic
extraction sources and need new v2 review-control packets after their canonical
port candidates exist.

## Independent-preflight stopping rule

This packet does not use a finding quota. Implementation becomes authorized
only after all three bounded checks target the same exact commit and packet
SHA-256:

1. the primary contract reviewer finds no unresolved contradiction, missing
   cross-field rule, scope conflict, or implementer choice that can change
   authorization;
2. the adversarial reviewer has exercised the existing implementation at each
   claimed boundary and finds no remaining reproducible bypass/crash class that
   the packet leaves unspecified; and
3. the implementation-feasibility reviewer confirms that every requirement has
   an owning allowed path, dependency order, and reusable or explicitly new
   counterexample test surface.

Every raised item must be mapped to a packet clause, explicitly dispositioned
as a truthful non-goal/trust boundary, or kept unresolved. A reviewer merely
running out of findings is not acceptance. Conversely, once all three checks
report zero unresolved items on the same bytes, the manager records the exact
dispositions and starts implementation; another arbitrary review round is not
required. Any later implementation finding is classified as contract omission,
implementation nonconformance, or repair-introduced regression before rework,
so the process can measure which stage caused the recurrence.

## Allowed write set

- `docs/development/ai/schemas/review-recurrence-control.v2.schema.json`
- `docs/development/ai/schemas/review-recurrence-repair-manifest.v1.schema.json`
- `docs/development/ai/schemas/review-recurrence-gate-result.v2.schema.json`
- `docs/development/ai/schemas/review-recurrence-live-arm-anchor.v1.schema.json`
- `docs/development/ai/schemas/timed-gate-result.v2.schema.json`
- `docs/development/ai/schemas/README.md`
- `docs/development/tools/recurrence_v2_model.py`
- `docs/development/tools/recurrence_v2_authority.py`
- `docs/development/tools/recurrence_v2_decision.py`
- `docs/development/tools/recurrence_v2_evidence.py`
- `docs/development/tools/recurrence_v2_fixture.py`
- `docs/development/tools/recurrence_v2_history.py` for retained-anchor and
  complete retained-result semantic replay
- `docs/development/tools/check_review_recurrence.py` for canonical schema
  loading, strict authority decoding, and result/replay plumbing
- `docs/development/tools/run_timed_gate.py` to emit the new bound result path,
  reject invalid finite/positive timing inputs before subprocess launch, and
  preserve all frozen runner timing/path/cleanliness/HEAD relations
- `docs/development/tools/check_ai_contracts.py` to register the new canonical
  gate-result and live-anchor schemas and apply the strict decoder to
  registered authority schema blobs
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
