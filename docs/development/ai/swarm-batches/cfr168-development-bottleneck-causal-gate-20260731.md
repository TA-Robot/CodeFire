# CFR-168 Development Bottleneck Causal Gate

Status: active manager decision gate  
Owner: portfolio manager  
Product mutation: forbidden  
Process-policy adoption: forbidden  
Codex worker fallback: forbidden

## Decision Question

Determine which mechanisms materially delay a trusted integrated CodeFire
delta, and which bounded interventions improve that outcome without weakening
correctness, review independence, evidence, or workstation safety.

This gate prevents three unsupported shortcuts:

1. equating repeated `REQUEST_CHANGES` with initial-review omission;
2. equating a long observed wait with the stated blocker being its sole cause;
3. equating more agents, more findings, more templates, or more activity with
   faster trusted integration.

The unit of success is a trusted integrated delta. Candidate count, report
count, finding count, agent count, and committed documentation volume are
diagnostics, not throughput outcomes.

## Current Evidence Boundary

The non-review audit at
`docs/development/ai/swarm-results/cfr168-non-review-throughput-bottleneck-audit-claude-20260731/audit.md`
reconstructed useful queue floors, gate durations, resource incidents, and
candidate generations. It also records that the event stream needed for a
complete queue/service/rework decomposition does not exist and that the
observed cohort produced 22 candidate generations but zero trusted product
integrations.

Consequently:

- generation and rejection counts are known;
- some queue lower bounds and exact gate durations are known;
- total rework seconds, flow efficiency, manager queue time, and integrated
  lead time are unavailable;
- no intervention has an observed effect on trusted integration in that
  cohort; and
- causal ranking must remain provisional.

The audit's Fugu `model-list incompatibility` attribution is superseded. Direct
top-level event-log inspection in
`docs/development/ai/swarm-results/cfr168-fugu-quota-review-interruptions-20260731/incident.md`
shows all four affected lanes terminated on a provider usage limit. Repository
text and agent reasoning had been mistaken for runtime errors. Do not use the
superseded attribution in provider-readiness design or impact estimates.

Post-reboot admission evidence in
`docs/development/ai/swarm-results/cfr168-post-reboot-candidate-revalidation-20260731/evidence.md`
also separates provider backoff from workstation exhaustion: every measured
resource/conflict check passed and provider backoff alone blocked both provider
lanes at that instant.

## Evidence Levels

Every bottleneck claim and intervention decision must carry one level:

| Level | Meaning | Permitted claim |
|---|---|---|
| `OBSERVED` | Exact event, object, monotonic gate result, incident, or durable artifact. | What happened to the named subject. |
| `ATTRIBUTED` | The proposed cause is distinguished from plausible alternatives by source/object evidence or a controlled comparison. | Why the named subject behaved that way. |
| `REPLICATED` | The attribution survives a later eligible subject without retuning. | The mechanism recurs in the declared cohort. |
| `INTERVENTION_EFFECT` | A preregistered comparison shows the change improved its target while guardrails held. | The intervention is a candidate for adoption. |
| `UNAVAILABLE` | Evidence cannot decide the value. | No replacement estimate or narrative. |

A long queue is `OBSERVED`, not automatically `ATTRIBUTED`. A tool that would
have prevented one incident is not an `INTERVENTION_EFFECT` until exercised on
an eligible later subject. A correctness guard can still be mandatory for
safety; in that case record the safety basis separately and do not advertise
unmeasured speedup.

## Stage Model

For each package, retain these states independently:

```text
scope frozen
  -> implementation active
  -> candidate frozen
  -> validation active
  -> review ready
  -> review active
  -> verdict settled
  -> report committed
  -> repair or composition ready
  -> integrated
  -> terminal evaluation
```

Every wait must name one immediate reason and one release condition. If two
reasons overlap, record both intervals instead of choosing a narrative
"primary" reason:

```text
provider_backoff
provider_window
resource_admission
conflict_domain
dependency
review_permit
build_control
manager
credential_or_preflight
unknown
```

No duration spanning a reboot or clock discontinuity is computed from wall
time. Monotonic intervals cannot be joined across boot epochs; such values are
`UNAVAILABLE`.

## Hypothesis Register

### Review and repair mechanisms

The dedicated reliability gate
`cfr168-review-root-cause-reliability-gate-20260731.md` owns classification
and intervention choice for:

- defects pre-existing inside an earlier review's declared scope;
- defects introduced by a repair;
- exact obligations weakened, restated, or left open;
- closure oracles that pass deliberate falsification;
- evidence-only, injection-only, policy-promoted, contradictory, or routable
  findings; and
- shared invariant families recurring across surfaces.

None is currently allowed to collapse into the label "review omission". The
existing 51-row audit is an observed-successor-finding census, not an absolute
defect denominator.

### Non-review mechanisms

| ID | Hypothesis | Required attribution evidence | Principal alternative |
|---|---|---|---|
| `H-NR-01` | false-green or unavailable source-bound Rust validation is holding the Rust path | candidate source key, executable/toolchain identity, nonzero exact test selection, complete log, and a foreign-artifact reversal | review, provider, or resource wait |
| `H-NR-02` | dependency fan-in unnecessarily blocks independent work | exact dependency graph plus proof that the blocked output does not consume the gating decision | the dependency is semantically required |
| `H-NR-03` | verdict-to-report terminalization consumes material critical-path time | first self-emitted verdict event, immutable report commit, and no unresolved analysis signal between them | reviewer was still analyzing |
| `H-NR-04` | coordinator serialization delays ready rollovers or integrations | ready/settled artifact exists while `manager` queue is explicitly open, followed by manager action | work was still provider-active or dependency-blocked |
| `H-NR-05` | duplicate broad gates consume avoidable time | identical cache key over source, argv, environment, toolchain, fixtures, and deterministic class | runs differ in hidden inputs or are non-cacheable |
| `H-NR-06` | provider launch seams waste work beyond unavoidable quota/backoff | typed preflight/credential/transport failure before useful provider work, followed by successful exact-session retry | provider quota or task failure |
| `H-NR-07` | retained worktrees/targets cause restart risk and recompilation | per-lane retained bytes, active/reference state, pressure sample, cleanup result, and later rebuild bytes | host-only disk pressure or unrelated process |
| `H-NR-08` | excessive WIP dilutes critical-path completion | conflict/resource/provider admissible work is opened while an older critical-path artifact could have advanced, with later measurable completion delay | independent WIP used otherwise idle capacity |
| `H-NR-09` | repeated surface-local work should be replaced by a shared contract | independently adjudicated same-invariant recurrence plus a contract usable without binding rejected product behavior | findings only look similar at a narrative level |

The register is not a ranking. A hypothesis becomes actionable only at the
evidence level required by the proposed action.

## Intervention Cards

Each proposed process or tool change requires one card with:

1. exact hypothesis ID;
2. eligible subjects selected without seeing the outcome;
3. intervention and unchanged comparison;
4. causal estimand;
5. required events and unavailable handling;
6. target metric;
7. correctness, review, resource, and cost guardrails;
8. reversal or negative control;
9. stop condition; and
10. rollback.

Do not select a fixed sample count merely because it is easy to state. Prefer a
complete preregistered cohort, such as all eligible transitions from one
canonical freeze through its terminal composition. Report every subject and
every exclusion. Where the cohort is too small for a stable estimate, preserve
raw outcomes and return `INSUFFICIENT_EVIDENCE`.

### Card A — Source-keyed Cargo gate

- Hypothesis: `H-NR-01`.
- Safety basis: foreign-worktree binaries and zero-selected success are
  already reproduced; fail-closed source binding is a correctness
  requirement independent of throughput.
- Comparison: historical foreign-artifact/zero-selection counterexample plus
  later exact source-keyed runs.
- Target: held Rust candidates obtain decision-grade validation without a
  foreign artifact.
- Throughput measures: build queue, rebuilt bytes, gate duration, and
  concurrency for distinct source keys.
- Guardrails: no direct or indirect unkeyed Cargo execution; no false reclaimed
  byte claim; no global serialization presented as success.
- Adoption boundary: candidate `4e5c0e7` still needs exact independent
  acceptance, canonical semantic port, exact port review, and integration.

### Card B — Cargo/KGI dependency split

- Hypothesis: `H-NR-02`.
- Intervention: integrate an accepted Cargo canonical port before the
  independent KGI/KPI semantic composition.
- Comparison: the superseded joint composition required both decisions before
  either port.
- Target: Rust validation release no longer depends on KGI/KPI acceptance.
- Guardrails: KGI/KPI changes still receive their own exact composition and
  review; no candidate is accepted by manager-green gates.
- Current state: dependency structure is frozen in
  `cfr168-source-keyed-cargo-canonical-port-20260731.md` and
  `cfr168-control-plane-composition-20260731.md`; effect remains unmeasured
  until the accepted port releases an actual Rust transition.

### Card C — Review terminalization

- Hypothesis: `H-NR-03`.
- Intervention: observe first self-emitted settled-verdict signal, then one
  bounded request to commit the exact report; never nudge before that signal.
- Comparison: eligible long reviews without a post-verdict nudge.
- Target: verdict-to-report monotonic duration.
- Guardrails: unchanged finding count/severity after the first settled signal,
  no competing reviewer, exact-session continuation, no manager-written
  verdict.
- Stop: any evidence that the signal was provisional suspends the intervention.

### Card D — Coordinator queue

- Hypothesis: `H-NR-04`.
- Intervention: typed `manager` queue entry/exit around a ready artifact, not a
  generic heartbeat quota.
- Comparison: all other queue reasons and provider-active periods.
- Target: ready-to-rollover and accepted-report-to-integration duration.
- Guardrails: no meaningless activity event, no integration before exact
  acceptance, no expansion of manager mutation authority.
- Decision: do not call historical no-commit gaps "idle"; the needed event was
  absent.

### Card E — Broad-gate reuse

- Hypothesis: `H-NR-05`.
- Intervention: reuse only deterministic, opt-in, success-only evidence under
  an exact source/environment/toolchain/fixture key.
- Negative controls: source byte, selected test, toolchain digest, fixture, or
  environment changes must miss; failed, flaky, install, dogfood, and final
  release gates are never reused.
- Target: duplicate deterministic gate seconds avoided.
- Guardrails: zero false hit, complete logs retained, independent reviewer
  falsification remains non-cacheable when required.

### Card F — Provider readiness

- Hypothesis: `H-NR-06`.
- Intervention: secret-preserving preflight and typed failure classification,
  followed by exact-session retry only for a clean zero-artifact preflight
  failure.
- Target: avoidable pre-dispatch failure count and time.
- Guardrails: quota/backoff is not auto-retried, secrets are never logged,
  provider failure is not task failure, no automatic Codex fallback.
- Correction: the four recent Fugu lanes are quota cases and are ineligible as
  evidence that a model-list fix would help.

### Card G — Resource retention

- Hypothesis: `H-NR-07`.
- Intervention: reclaim only manager-created reproducible build artifacts
  after process death and durable evidence binding.
- Target: retained bytes, rebuild bytes, and restart incidents.
- Guardrails: no live target, worktree, Git object, state, packet, report, or
  unbound log deletion; container-only data must not claim to observe Windows
  C-drive free space.

### Card H — Shared primitive preflight

- Hypothesis: `H-NR-09`.
- Admission: the already prepared report-only preflight may run as evidence
  collection before the blind calibration because it mutates no product
  byte and binds no rejected implementation as accepted behavior.
- Intervention: enumerate candidate shared contracts from exact immutable
  findings; no extraction or product implementation.
- Target: later same-invariant findings captured before surface repair and
  generations avoided.
- Burden/noise: distinct contracts, duplicated obligations, unavailable
  production entry points, implementer/reviewer artifacts.
- Guardrails: a rejecting report may describe a required invariant but cannot
  confer acceptance on its product implementation.
- Interpretation boundary: neither `READY` nor a later MR-write repair proves
  `H-NR-09` or a speedup. General process adoption and recurrence claims wait
  for the blind calibration, fresh-session comparison, adjudication, and a
  later eligible subject.

## Common Outcome Measures

Every live intervention reports raw subjects before aggregates:

| Measure | Required form |
|---|---|
| trusted lead time | monotonic, boot-local stage intervals; otherwise `UNAVAILABLE` |
| queue time | seconds by immediate reason and blocking artifact |
| service time | provider/gate-active monotonic seconds by stage |
| rework | later-generation service and gate seconds, not generation count alone |
| first-pass decision | exact independent verdict per surface generation |
| terminal integration | exact accepted delta and post-integration gate |
| resource effect | retained/rebuilt bytes, peak admission snapshot, restart count |
| review effect | later-finding capture, noise, duplication, false positives, author and reviewer burden |
| provider effect | typed avoidable failures separate from quota/backoff |
| unavailable | count and reason, retained in every denominator |

No single composite "developer productivity" score is permitted. It would hide
a faster but less trustworthy path.

## Decision Labels

For every intervention return exactly one:

- `ADOPT_FOR_SAFETY` — independently required for correctness; throughput
  effect remains separately measured;
- `PROMISING_FOR_LIVE_EXPERIMENT`;
- `ADOPT_AFTER_REPLICATED_EFFECT`;
- `REVISE_AND_REPLAY`;
- `NO_EVIDENCE_TO_ADOPT`;
- `INSUFFICIENT_EVIDENCE`.

An intervention may have both `ADOPT_FOR_SAFETY` and an unmeasured throughput
effect. Never convert that into a speed claim.

## Immediate Manager Rules

Until this gate has decision-grade evidence:

1. do not require 100 findings, any other fixed finding count, or a fixed
   review-template depth;
2. do not relax exact review/integration authority to improve yield;
3. do not launch extra WIP when provider, resource, conflict, or dependency
   evidence shows it cannot advance;
4. while another lane is legitimately active, advance independent
   report-only, packet, evidence, or composition-preflight work when it has a
   named downstream consumer;
5. never substitute activity for a durable artifact;
6. preserve `UNAVAILABLE` rather than estimate around clock, provider-cost, or
   host-disk gaps; and
7. record corrections to earlier causal attributions without rewriting the
   immutable audit.

## Release Conditions

No permanent development-process change is justified by this gate until:

1. the review reliability blind calibration, fresh-session row/axis
   comparison, temporal check, and disagreement adjudication are complete for
   review/repair hypotheses;
2. the intervention has a complete preregistered eligible cohort or explicitly
   returns `INSUFFICIENT_EVIDENCE`;
3. every target and guardrail is reported with raw numerator, denominator, and
   unavailable count;
4. at least one negative control or reversal demonstrates that the measurement
   can fail;
5. a later eligible subject replicates the attributed mechanism when the claim
   is intended to generalize; and
6. rollback remains possible without rewriting evidence.

The valid conclusion may be that the current data do not support a process
change.
