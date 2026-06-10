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

## DES-AUTO-020: Research cycle retrospective

The research cycle retrospective consumes compact per-cycle signals:

- cycle ID
- completed run count
- improved candidate count
- regressed candidate count
- failed run count
- blocked item count
- mean run cost
- remaining budget
- high-severity frontier drift count
- evidence-ready claim count
- optional notes

It produces a deterministic report with:

- aggregate completed run count
- improvement, failure, and regression rates
- average cost
- budget pressure
- risk pressure
- ordered policy adjustment recommendations
- priority: `high`, `medium`, or `low`
- rationale strings suitable for the next planning document

Recommendation rules are conservative:

- high blocker or high drift pressure recommends risk mitigation
- high cost with low remaining budget recommends reducing experiment cost
- strong improvements with evidence-ready claims recommends consolidation
- low improvement and low failure signal recommends increased exploration
- repeated regressions or failures recommends archiving stale paths
- if no clear pressure exists, the current policy is kept

The retrospective does not mutate frontier, queue, or campaign state. It is a read-only synthesis step used to seed the next docs-first planning cycle.

## DES-AUTO-021: Retrospective planning summary

The retrospective planning summary renderer consumes a `ResearchCycleRetrospectiveReport` and produces Markdown.

The summary includes:

- title
- cycle ID list
- priority
- completed run count
- improvement, regression, and failure rates
- average cost
- budget pressure
- risk pressure
- ordered recommendation list
- ordered rationale list

Formatting is deterministic and intentionally compact. It does not include raw run logs, raw artifacts, or per-experiment debug output. This keeps the planning document focused on policy adjustment rather than rerunning the analysis step.

## DES-AUTO-022: Research cycle plan synthesis

The cycle plan synthesizer consumes a `ResearchCycleRetrospectiveReport` and converts recommendation-level guidance into lane-level next-cycle work.

The output `ResearchCyclePlan` preserves:

- source cycle IDs
- retrospective priority
- active experiment capacity
- remaining budget
- deterministic plan items

Each plan item includes:

- lane: `mitigation`, `active`, `review`, `archive`, or `deferred`
- source recommendation
- title
- concrete next action
- rationale
- budget hint

Lane assignment is deterministic:

- `mitigate_risk` becomes mitigation work
- `reduce_cost` and `increase_exploration` become active experiment planning work
- `consolidate` becomes review work
- `archive_stale` becomes archive work
- `keep_policy` becomes deferred monitoring work

The synthesizer validates positive active capacity and non-negative remaining budget. Active work is bounded by capacity, and budget hints are capped by the remaining budget. Overflow active work is converted to deferred items instead of being dropped, so the next cycle keeps visibility into lower-priority work without overcommitting execution.

## DES-AUTO-023: Research cycle plan Markdown rendering

The research cycle plan renderer consumes `ResearchCyclePlan` and emits deterministic Markdown.

The document includes:

- title
- source cycle IDs
- retrospective priority
- active capacity
- remaining budget
- lane sections in enum order
- one subsection per plan item
- recommendation, action, budget hint, and rationale fields

The renderer does not inspect raw retrospective inputs, run logs, or evidence artifacts. It only formats the bounded plan object produced by `ResearchCyclePlanSynthesizer`. This keeps the planning artifact stable, reviewable, and cheap to regenerate after policy tuning.

## DES-AUTO-024: Research cycle plan linting

The research cycle plan linter consumes `ResearchCyclePlan` and returns `ResearchCyclePlanLintReport`.

The report contains deterministic `ResearchCyclePlanLintFinding` entries with:

- severity: `blocker` or `warning`
- field path
- message

Blockers are used for execution safety:

- missing source cycle IDs
- non-positive active capacity
- negative remaining budget
- active lane item count above active capacity
- active budget hints above remaining budget
- missing item title or action
- negative item budget hint

Warnings are used for reviewer attention without blocking plan storage:

- no lane items
- empty item rationale
- duplicated recommendation

The linter does not mutate the plan, rerun retrospective analysis, or infer hidden state from logs. It only validates the bounded plan object that will be rendered, reviewed, and executed in the next docs-first cycle.

## DES-AUTO-025: Research cycle plan lint Markdown rendering

The lint Markdown renderer consumes `ResearchCyclePlanLintReport` and produces a compact deterministic review document.

The document includes:

- title
- overall status: `ok` or `blocked`
- blocker count
- warning count
- blocker section when blocker findings exist
- warning section when warning findings exist
- field path and message for each finding
- explicit `none` finding line when the report has no findings

Findings are rendered in the already-sorted report order and grouped by severity. The renderer does not rerun linting, inspect the original plan, or hide warnings behind a passing status. This keeps automated planning logs stable and lets reviewers see why a plan is blocked or merely needs attention.

## DES-AUTO-026: Research cycle planning packet

The planning packet builder consumes a `ResearchCycleRetrospectiveReport` plus active capacity and remaining budget, then composes the existing planning components:

- `ResearchCyclePlanSynthesizer`
- `ResearchCyclePlanLint`
- `ResearchCyclePlanMarkdown`
- `ResearchCyclePlanLintMarkdown`

The output `ResearchCyclePlanningPacket` contains:

- `plan`
- `lint_report`
- `plan_markdown`
- `lint_markdown`

The builder does not add new planning policy. It is a deterministic orchestration layer that keeps the plan object, safety decision, and reviewer-facing Markdown synchronized for docs-first cycle handoff.

## DES-AUTO-027: Research cycle planning packet Markdown

The planning packet Markdown renderer consumes `ResearchCyclePlanningPacket` and emits one review document for cycle handoff.

The document includes:

- title
- source cycle IDs
- priority
- lint status
- active item count
- remaining budget
- blocker count
- warning count
- plan section
- lint section

The renderer reuses the packet's existing `plan_markdown` and `lint_markdown` fields rather than rerunning planning or linting. It demotes headings inside those embedded documents by one level so the packet remains a single coherent Markdown artifact with a stable hierarchy.

## DES-AUTO-028: Research cycle planning packet manifest

The planning packet manifest builder consumes `ResearchCyclePlanningPacket` and derives artifact metadata without writing files.

The manifest includes:

- source cycle IDs
- status: `ok` or `blocked`
- combined planning packet Markdown artifact entry
- plan Markdown artifact entry
- lint Markdown artifact entry

Each entry stores a relative POSIX path, `sha256:<hex>` digest, and UTF-8 byte count. Absolute paths, Windows separators, empty segments, current-directory segments, and parent-directory segments are rejected before any manifest entry is returned. This keeps the handoff contract deterministic and safe to pass to a future writer or storage adapter.

## DES-AUTO-029: Research cycle planning packet manifest Markdown

The manifest Markdown renderer consumes `ResearchCyclePlanningPacketManifest` and emits a compact audit document.

The document includes:

- title
- source cycle IDs
- status
- artifact count
- artifact table with path, SHA-256 digest, and byte count

The renderer does not recompute hashes and does not inspect packet contents. It formats the manifest that automation already validated, preserving entry order so reviewer-facing logs and machine handoff metadata stay aligned.

## DES-AUTO-030: Research cycle planning packet manifest verification

The manifest verifier consumes:

- `ResearchCyclePlanningPacketManifest`
- a mapping from relative artifact path to UTF-8 content

The output `ResearchCyclePlanningPacketManifestVerification` contains deterministic findings. A verification is `ok` only when every manifest entry has matching content.

Findings include:

- missing artifact
- SHA-256 digest mismatch
- byte count mismatch

The verifier does not read files, write artifacts, or infer extra required paths outside the manifest. It walks manifest entries in order and can emit multiple findings for the same artifact when both hash and byte count drift. This keeps the integrity check independent from the eventual storage adapter while preserving enough detail for docs-first automation to stop a stale handoff before execution.
