# Experiment Platform Design

## DES-AUTO-001: Experiment plan record

`ExperimentPlan` evolves from the current dataclass into a durable record with:

- plan ID
- hypothesis ID
- benchmark ID
- baseline ID
- metric definition
- command or runner reference
- resource budget
- artifact contract
- analysis checklist

## DES-AUTO-002: Runner abstraction

The runner interface has three phases:

1. prepare workspace
2. execute command
3. collect artifacts

The first runner is local process execution. Later runners can target containers, GPU machines, or queue systems without changing planner logic.

## DES-AUTO-003: Artifact contract

Each experiment declares expected artifacts before execution.

Required first artifacts:

- `metrics.json`
- `run.log`
- `config.json`
- `environment.json`
- `analysis.md`

Missing artifacts classify the run as incomplete rather than failed science.

## DES-AUTO-004: Result ingestion pipeline

Result ingestion parses artifacts into structured records, validates schema, normalizes metrics, and writes analysis notes.

Raw artifacts remain immutable. Derived summaries can be regenerated.

## DES-AUTO-005: Benchmark registry

Benchmarks are described by:

- task
- dataset
- split
- metric
- direction
- baseline references
- known pitfalls
- validation protocol

The registry is part of research memory, not hard-coded runner logic.

## DES-AUTO-006: Baseline registry

Baselines are versioned and linked to evidence.

Baseline records distinguish:

- imported literature number
- reproduced local number
- internal previous best
- sanity-check baseline

## DES-AUTO-007: Queue scheduler

The scheduler orders experiments by priority score, dependency readiness, expected information gain, and budget.

It can defer expensive experiments until cheaper probes support them.

## DES-AUTO-008: Analysis workflow

The analyst produces a structured analysis:

- summary
- metric comparison
- confidence
- failure classification
- suspected mechanism
- next action
- claim readiness update

This makes results useful for future planning rather than merely storing metrics.

## DES-AUTO-009: Experiment triage board

The triage board consumes compact decision signals rather than raw logs:

- plan ID
- priority
- expected information gain
- estimated cost
- promotion decision
- learning trend
- open blocker count
- rationale

It produces ranked actions:

- `run` for ready high-value experiments within budget
- `replicate` for promising but under-confirmed or unstable candidates
- `mutate` for plateaued or insufficient-improvement candidates
- `promote` for candidates that meet promotion gates
- `reject` for confident regressions
- `hold` for blocked or over-budget work

The ranking score is deterministic and cost-aware so repeated planning with the same state returns the same action order.

## DES-AUTO-010: Experiment iteration planner

The iteration planner consumes the same compact signals as the triage board and produces a plan with three lanes:

- `active` for experiments that should run, replicate, or mutate in the next iteration
- `review` for promotion or rejection decisions that need claim, evidence, or human review before changing campaign state
- `deferred` for blocked, over-budget, or lower-ranked work that should remain visible but not consume the next iteration

The planner enforces:

- non-negative remaining budget
- positive active capacity
- deterministic ordering from the triage score
- budget accounting based on estimated experiment cost
- review capacity so promotion/rejection work cannot crowd out execution planning

## DES-AUTO-011: Experiment risk register

The risk register stores compact risk records:

- risk ID
- related plan ID
- category
- severity
- probability
- impact
- status
- mitigation

The register exposes:

- a deterministic priority order for open risks
- a blocker view for critical or high-risk open items
- a mitigation checklist for the next research iteration

Risk priority is computed without hidden mutable state so repeated planning produces the same ordering for the same risk set.

## DES-AUTO-012: Experiment evidence pack

The evidence pack builder consumes compact promotion evidence rather than raw logs:

- plan ID
- candidate ID
- improvement over baseline
- confidence
- replication count
- stale evidence count
- blocker risk count
- missing artifact count
- mitigation notes

It produces:

- a readiness decision: `ready`, `review`, or `blocked`
- a deterministic completeness score
- blocking reasons that must be resolved before promotion
- review notes for non-blocking weaknesses
- a mitigation checklist ordered by input order

The builder is intentionally conservative. Blocker risks or missing required artifacts produce `blocked`; stale evidence or low replication produce `review`; only candidates with sufficient confidence, replication, fresh evidence, and no blockers become `ready`.

## DES-AUTO-013: Claim review queue

The claim review queue consumes compact evidence packs and reviewer objection counts rather than raw experiment logs.

Each review request includes:

- claim ID
- candidate ID
- evidence pack readiness
- open reviewer objection count
- blocking reviewer objection count
- assigned reviewer count

The queue produces:

- a decision: `approve`, `revise`, or `block`
- a deterministic priority score
- action reasons ordered from most severe to least severe
- a checklist combining evidence-pack mitigations with reviewer follow-up

Decision rules are intentionally conservative:

- evidence-pack `blocked` or any blocking objection produces `block`
- evidence-pack `review`, open non-blocking objections, or no assigned reviewer produces `revise`
- only readiness `ready`, no open objections, and at least one reviewer produces `approve`

Blocked items rank before revision items, and revision items rank before approval-ready items. Ties are broken by descending priority score and then stable claim ID.

## DES-AUTO-014: Claim audit trail

The claim audit trail consumes compact review artifacts:

- evidence pack
- claim review item
- optional external-claim eligibility
- optional reviewer or governance note

It produces ordered audit events with:

- event ID
- claim ID
- candidate ID
- stage
- status
- source references
- follow-up actions

The builder validates that every event has a claim ID, candidate ID, stage, status, and at least one source reference. Events are ordered by deterministic stage rank and then event ID:

1. `evidence`
2. `review`
3. `external_claim`
4. `follow_up`

The audit trail is intentionally compact. It stores decisions and references, not raw metrics or logs. Raw artifacts remain in evidence objects and report bundles.

## DES-AUTO-015: Claim release gate

The release gate consumes:

- claim ID
- candidate ID
- evidence readiness
- review decision
- audit trail terminal status
- required release artifact names
- available artifact names

It produces:

- release decision: `allow`, `hold`, or `reject`
- blocking reasons
- release checklist
- missing artifacts

Gate rules:

- evidence `blocked`, review `block`, or audit terminal status `required` produces `reject`
- evidence `review`, review `revise`, audit terminal status other than `allowed`, or missing artifacts produces `hold`
- only evidence `ready`, review `approve`, audit terminal status `allowed`, and all required artifacts present produces `allow`

The gate is deterministic and does not inspect raw logs. Missing artifacts preserve input order from the required artifact list.

## DES-AUTO-016: Research frontier map

The research frontier map is the entry point for the next exploration cycle. It consumes compact research signals rather than raw logs:

- baseline gap by benchmark and metric
- known negative result avoid conditions
- novelty overlap against known algorithm families
- open campaign risks
- remaining budget
- challenge objective and target metric

It produces ranked frontier items with:

- frontier ID
- mechanism family
- target benchmark or domain
- opportunity score
- expected evidence gain
- primary risk
- next experiment action
- rationale

Scoring is deterministic and conservative. A large baseline gap increases opportunity. Novel mechanism families increase opportunity. Repeated negative-result overlap, blocker risks, and severe budget pressure reduce opportunity. The map does not claim SOTA readiness; it only decides where the agent should spend the next research iteration.

Tie-breaking is stable by frontier ID. This keeps repeated planning runs reproducible and makes CodeFire diffs meaningful.

## DES-AUTO-017: Frontier experiment plan drafting

The frontier experiment planner consumes ranked `ResearchFrontierItem` values and emits bounded `ExperimentPlan` drafts.

For each runnable frontier item it creates:

- a hypothesis title based on the mechanism family
- a hypothesis rationale from the frontier rationale and reasons
- benchmark and baseline references
- the target metric
- an estimated cost capped by remaining budget
- a deterministic runner command
- required artifact paths
- analysis criteria suitable for a first probe

The planner skips frontier items whose action is `defer`, `mitigate_risk`, or `redesign`. This keeps the runner from executing work that the frontier map has already identified as blocked or unsafe. Output is bounded by `max_plans`, and ordering follows frontier ranking order.

## DES-AUTO-018: Frontier feedback integration

The frontier feedback integrator consumes existing `ResearchFrontierSignal` values and compact run feedback records.

Each feedback record contains:

- frontier ID
- outcome: `improved`, `replicated`, `regressed`, `failed`, `blocked`, or `inconclusive`
- metric delta against the tracked baseline
- confidence
- observed cost
- blocker count
- notes

The integrator returns update records rather than mutating signals in place. Each update includes:

- frontier ID
- prior signal
- updated signal
- applied outcome count
- deterministic reasons

Update rules are conservative:

- improved or replicated feedback increases baseline gap and challenge alignment only when confidence is positive
- regressed or failed feedback increases negative-result overlap and reduces expected information gain
- blocked feedback increases blocker risk and requires mitigation before execution
- over-budget feedback reduces expected information gain so cheap probes remain favored
- unknown frontier feedback is reported as an update with no signal rather than silently changing another frontier

All numeric fields are clamped to bounded non-negative ranges. This keeps the next `ResearchFrontierMap` ranking stable and prevents a single noisy result from producing unbounded priority swings.

## DES-AUTO-019: Frontier drift report

The frontier drift reporter consumes two ranked frontier snapshots:

- previous frontier items
- current frontier items after feedback, new evidence, or changed budget

It produces a compact report with:

- per-frontier drift records
- rank delta
- score delta
- action transition
- drift category: `new`, `removed`, `rising`, `falling`, `stable`, or `action_changed`
- severity: `high`, `medium`, or `low`
- deterministic recommendation

The reporter does not inspect raw logs. It compares stable frontier IDs and the already-ranked `ResearchFrontierItem` records. This keeps long-campaign review cheap and reproducible.

Severity rules are conservative:

- new or removed executable frontiers are high severity
- action changes into mitigation, redesign, or defer are high severity
- large rank or score movement is medium or high depending on magnitude
- small rank movement without action change is low severity

Report ordering is deterministic: high severity first, then absolute rank delta, then absolute score delta, then frontier ID. This lets the next planning cycle focus on drift that changes what the agent should do, rather than on cosmetic score noise.
