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

## DES-AUTO-031: Research cycle planning packet manifest verification Markdown

The manifest verification Markdown renderer consumes `ResearchCyclePlanningPacketManifestVerification` and emits a compact audit document.

The document includes:

- title
- status: `ok` or `blocked`
- finding count
- findings section
- explicit `none` finding line when verification is clean
- path-scoped finding lines when verification detects drift

The renderer does not recompute verification, read artifact contents, or suppress duplicate findings for the same path. It formats the already-computed verification result in deterministic order so the same object can drive automated blocking and human review logs.

## DES-AUTO-032: Research cycle planning handoff bundle

The handoff bundle builder consumes a `ResearchCycleRetrospectiveReport` plus active capacity, remaining budget, artifact paths, and reviewer-facing titles.

The output `ResearchCyclePlanningHandoffBundle` contains:

- `packet`
- artifact contents for planning packet Markdown, plan Markdown, and lint Markdown
- `manifest`
- manifest Markdown
- manifest verification result
- manifest verification Markdown

The builder composes the existing packet, manifest, verification, and Markdown components without adding new planning policy. It validates artifact paths through the manifest builder, verifies the generated artifact contents against the manifest before returning, and leaves all filesystem or storage writes to a future adapter.

## DES-AUTO-033: Research cycle planning handoff bundle Markdown

The handoff bundle Markdown renderer consumes `ResearchCyclePlanningHandoffBundle` and emits a compact index document.

The document includes:

- title
- source cycle IDs
- packet status
- manifest status
- verification status
- artifact count
- artifact table with path and byte size
- audit availability lines for manifest Markdown and verification Markdown

The renderer does not rebuild the packet, rehash artifacts, rerun verification, or inspect external storage. It formats the already-built bundle so a reviewer or future storage adapter can see which handoff artifacts exist and whether the bundle is clean before opening the full packet, manifest, or verification documents.

## DES-AUTO-034: Research cycle planning handoff bundle summary

The handoff bundle summary consumes `ResearchCyclePlanningHandoffBundle` and returns a plain Python dictionary for automation.

The dictionary includes:

- `source_cycles`
- `packet_status`
- `manifest_status`
- `verification_status`
- `artifact_count`
- `artifacts` with path, byte count, and SHA-256 digest
- audit availability booleans for manifest Markdown and verification Markdown
- `finding_count`

The summarizer does not read or write files, rerun retrospective planning, or rerun manifest verification. It only reflects the already-built bundle plus artifact content digests so downstream orchestration can decide whether a handoff is clean without scraping Markdown.

## DES-AUTO-035: Research cycle planning handoff readiness gate

The handoff readiness gate consumes `ResearchCyclePlanningHandoffBundle` and returns a plain Python dictionary with `ready`, `status`, `blockers`, `warnings`, `artifact_count`, and `finding_count`.

The gate blocks when:

- packet lint has blocker findings
- manifest status is not `ok`
- manifest verification has findings
- no handoff artifacts exist
- manifest audit Markdown is missing
- verification audit Markdown is missing

The gate preserves lint warnings as non-blocking warning strings. It does not write files, rerun manifest verification, or mutate the bundle; it only turns the already-built handoff state into a storage/reviewer routing decision.

## DES-AUTO-036: Research cycle planning handoff readiness Markdown

The readiness Markdown renderer consumes the plain dictionary returned by `ResearchCyclePlanningHandoffReadinessGate.evaluate`.

The document includes:

- title
- ready yes/no value
- status
- artifact count
- finding count
- blockers section with `none` when empty
- warnings section with `none` when empty

The renderer does not inspect the bundle or recompute readiness. It only serializes an already-made routing decision so reviewers can audit the same data that automation used.

## DES-AUTO-037: Research cycle planning handoff review packet

The handoff review packet builder consumes an existing `ResearchCyclePlanningHandoffBundle`.

The output `ResearchCyclePlanningHandoffReviewPacket` contains:

- the original bundle
- machine-readable handoff summary
- readiness decision dictionary
- readiness Markdown

The builder composes `ResearchCyclePlanningHandoffBundleSummary`, `ResearchCyclePlanningHandoffReadinessGate`, and `ResearchCyclePlanningHandoffReadinessMarkdown`. It does not rebuild the packet, rerun manifest verification, or write files, keeping one already-reviewed bundle as the source of truth for downstream storage and reviewer workflows.

## DES-AUTO-038: Research cycle planning handoff review packet Markdown

The review packet Markdown renderer consumes a `ResearchCyclePlanningHandoffReviewPacket`.

The rendered document contains:

- source cycles
- ready yes/no value and readiness status
- packet, manifest, and verification statuses
- artifact count and verification finding count
- artifact table with path, byte count, and SHA-256 digest
- blockers and warnings with `none` when empty
- audit document availability for readiness, manifest, and verification Markdown

The renderer only serializes data already present in the packet. It does not rebuild the handoff bundle, recompute readiness, rerun manifest verification, or read artifact files from disk.

## DES-AUTO-039: Research cycle planning handoff review packet artifacts

The review packet artifact builder consumes a `ResearchCyclePlanningHandoffReviewPacket`.

The builder emits `ResearchCyclePlanningHandoffArtifact` entries for:

- review packet Markdown
- readiness Markdown
- manifest Markdown
- verification Markdown

All output paths are caller-controlled relative POSIX paths and are validated with the same manifest path rule used by planning packet artifacts. The builder renders only the review packet Markdown from the supplied packet and copies the already-computed readiness, manifest, and verification Markdown. It does not rebuild the handoff bundle, recompute readiness, rerun manifest verification, or write files.

## DES-AUTO-040: Research cycle planning handoff review packet artifact manifest

The review packet artifact manifest builder consumes a `ResearchCyclePlanningHandoffReviewPacket` and the packaged `ResearchCyclePlanningHandoffArtifact` entries.

The builder emits a `ResearchCyclePlanningPacketManifest` with:

- source cycles copied from `packet.summary["source_cycles"]`
- status `ok`
- one entry per packaged artifact
- each entry containing the artifact path, SHA-256 digest, and byte count

The builder reuses the shared manifest entry path validation and hashing helper. It does not render Markdown, rebuild the review packet, read files, or write files.

## DES-AUTO-041: Research cycle planning handoff review packet artifact manifest Markdown

The review packet artifact manifest Markdown renderer consumes a `ResearchCyclePlanningPacketManifest` produced for review packet artifacts.

The rendered document contains:

- source cycles
- manifest status
- artifact count
- artifact table with path, SHA-256 digest, and byte count

The renderer delegates table semantics to the existing manifest Markdown shape while providing a review-packet-specific title and trace atom. It does not inspect artifact contents, recompute hashes, or write files.

## DES-AUTO-042: Research cycle planning handoff review packet artifact manifest verification

The review packet artifact manifest verifier consumes a `ResearchCyclePlanningPacketManifest` and a mapping of artifact path to content.

The verifier returns `ResearchCyclePlanningPacketManifestVerification` with:

- missing artifact findings
- SHA-256 digest mismatch findings
- byte count mismatch findings
- `ok` true only when no findings exist

The verifier delegates checksum and byte-count semantics to the existing planning packet manifest verifier. It does not read or write files and keeps review packet storage validation aligned with the rest of the planning handoff audit pipeline.

## DES-AUTO-043: Research cycle planning handoff review packet artifact manifest verification Markdown

The review packet artifact manifest verification Markdown renderer consumes `ResearchCyclePlanningPacketManifestVerification` and returns a deterministic Markdown audit document.

The document includes:

- a review-packet-specific title
- `ok` or `blocked` status
- finding count
- one bullet per path-scoped finding, or `none` when clean

The renderer delegates formatting semantics to the existing planning packet manifest verification Markdown renderer while providing a dedicated trace atom and default title for final review artifact audits. It does not recompute verification, inspect artifacts, or write files.

## DES-AUTO-044: Research cycle planning handoff review packet artifact archive

The review packet artifact archive builder consumes a `ResearchCyclePlanningHandoffReviewPacket` and deterministic output paths.

The builder returns a single archive object with:

- packaged review packet artifacts
- artifact manifest
- artifact manifest Markdown
- artifact manifest verification result
- artifact manifest verification Markdown

The builder composes the existing artifact builder, manifest builder, manifest Markdown renderer, manifest verifier, and verification Markdown renderer. It builds verification from the in-memory artifact contents it just packaged, so clean archives have `verification.ok == true` unless the constituent builders disagree. It does not write files, read storage, or mutate the review packet.

## DES-AUTO-045: Research cycle planning handoff review packet artifact archive summary

The review packet artifact archive summary consumes `ResearchCyclePlanningHandoffReviewPacketArtifactArchive` and returns a JSON-serializable dictionary.

The summary contains:

- source cycles copied from the archive manifest
- archive status derived from manifest and verification status
- readiness status copied from the review packet readiness gate
- artifact count and artifact path, byte count, and SHA-256 digest records
- manifest status, verification status, and finding count
- audit document availability for manifest Markdown and verification Markdown

The summarizer recomputes only per-artifact digest metadata from in-memory artifact contents. It does not rebuild the archive, rerun verification, read files, or mutate the packet.

## DES-AUTO-046: Research cycle planning handoff review packet artifact archive summary Markdown

The review packet artifact archive summary Markdown renderer consumes the dictionary emitted by `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummary`.

The rendered document includes:

- source cycles
- archive, readiness, manifest, and verification status
- artifact count and finding count
- audit availability for manifest and verification Markdown
- artifact table with path, byte count, and SHA-256 digest

The renderer treats the summary as the single source of truth. It does not rebuild archives, recompute hashes, rerun verification, or read files.

## DES-AUTO-047: Research cycle planning handoff review packet artifact archive summary Markdown gate

The review packet artifact archive summary Markdown gate consumes a summary dictionary and the already-rendered summary Markdown.

The gate returns a deterministic dictionary with:

- `ready` and `status`
- blocker messages for missing scalar lines or table rows
- artifact count and finding count copied from the summary
- checked artifact row count derived from the summary artifact list

The gate verifies that the Markdown contains the source cycle line, archive/readiness/manifest/verification status lines, ready state, artifact count, finding count, audit availability lines, and one exact artifact table row for each artifact record. It does not rebuild the summary, render Markdown, recompute hashes, inspect archives, read files, or write files.

## DES-AUTO-048: Research cycle planning handoff review packet artifact archive summary Markdown gate Markdown

The review packet artifact archive summary Markdown gate Markdown renderer consumes the dictionary emitted by `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryMarkdownGate`.

The rendered document contains:

- ready state and status
- artifact count, checked artifact count, and finding count
- blocker messages, or `none` when clean

The renderer treats the gate result as the single source of truth. It does not reevaluate the summary Markdown, rebuild the archive, recompute hashes, inspect artifacts, read files, or write files.

## DES-AUTO-049: Research cycle planning handoff review packet artifact archive summary artifacts

The review packet artifact archive summary artifact builder consumes `ResearchCyclePlanningHandoffReviewPacketArtifactArchive` and caller-controlled relative POSIX output paths.

The builder returns one deterministic handoff object with:

- the source archive
- machine-readable archive summary
- archive summary Markdown
- archive summary Markdown gate result
- archive summary Markdown gate Markdown
- two packaged artifacts: summary Markdown and gate Markdown

The builder composes the existing archive summary, summary Markdown renderer, summary Markdown gate, and gate Markdown renderer. It validates output paths with the shared manifest path rule. It does not rebuild the archive, rerun archive manifest verification, read files, write files, or mutate the archive.

## DES-AUTO-050: Research cycle planning handoff review packet artifact archive summary artifact manifest

The review packet artifact archive summary artifact manifest builder consumes `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifacts`.

The builder emits `ResearchCyclePlanningPacketManifest` with:

- source cycles copied from the summary payload
- status `ok`
- one entry per packaged summary artifact
- each entry containing artifact path, SHA-256 digest, and byte count

The builder reuses the shared manifest entry path validation and hashing helper. It does not rebuild summary artifacts, render Markdown, inspect the archive, read files, or write files.

## DES-AUTO-051: Research cycle planning handoff review packet artifact archive summary artifact manifest Markdown

The review packet artifact archive summary artifact manifest Markdown renderer consumes a `ResearchCyclePlanningPacketManifest` produced for archive summary artifacts.

The rendered document contains:

- source cycles
- manifest status
- artifact count
- artifact table with path, SHA-256 digest, and byte count

The renderer delegates table semantics to the existing manifest Markdown shape while providing an archive-summary-specific title and trace atom. It does not inspect artifact contents, recompute hashes, or write files.

## DES-AUTO-052: Research cycle planning handoff review packet artifact archive summary artifact manifest verification

The review packet artifact archive summary artifact manifest verifier consumes the manifest produced for archive summary artifacts and a path-to-content map for the persisted summary artifacts.

The verifier emits `ResearchCyclePlanningPacketManifestVerification` with one finding per detected integrity issue:

- missing artifact content
- SHA-256 digest mismatch
- byte count mismatch

The verifier delegates integrity semantics to the shared manifest verifier while providing an archive-summary-specific trace atom. It does not rebuild summary artifacts, rerender Markdown, read files, write files, or mutate the archive.

## DES-AUTO-053: Research cycle planning handoff review packet artifact archive summary artifact manifest verification Markdown

The review packet artifact archive summary artifact manifest verification Markdown renderer consumes `ResearchCyclePlanningPacketManifestVerification`.

The rendered document contains:

- verification status
- finding count
- path-scoped finding messages, or `none` when clean

The renderer delegates finding layout to the shared manifest verification Markdown shape while providing an archive-summary-specific title and trace atom. It does not rerun verification, inspect artifacts, recompute hashes, read files, or write files.

## DES-AUTO-054: Research cycle planning handoff review packet artifact archive summary artifact archive

The review packet artifact archive summary artifact archive builder consumes `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifacts`.

The builder returns one deterministic in-memory archive with:

- the source summary artifact handoff
- the packaged summary artifacts
- the summary artifact manifest
- manifest Markdown
- manifest verification result
- manifest verification Markdown

The builder composes the existing summary artifact manifest builder, manifest Markdown renderer, manifest verifier, and verification Markdown renderer. It does not rebuild the final review archive, rerender summary or gate Markdown, read files, write files, or mutate inputs.

## DES-AUTO-055: Research cycle planning handoff review packet artifact archive summary artifact archive summary

The review packet artifact archive summary artifact archive summary consumes `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchive`.

The summary dictionary contains:

- source cycles copied from the summary artifact manifest
- parent archive status copied from the source archive summary payload
- summary artifact archive status derived from manifest status and verification result
- manifest and verification status
- artifact count and per-artifact path, byte count, and SHA-256 digest
- audit availability for manifest Markdown and verification Markdown
- verification finding count

The summarizer treats the in-memory archive as the single source of truth. It does not rerun verification, recompute the manifest, read files, write files, or mutate inputs.

## DES-AUTO-056: Research cycle planning handoff review packet artifact archive summary artifact archive summary Markdown

The review packet artifact archive summary artifact archive summary Markdown renderer consumes the dictionary emitted by `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummary`.

The rendered document contains:

- source cycles
- parent archive status
- summary artifact archive status
- manifest and verification status
- artifact count and finding count
- audit availability for manifest Markdown and verification Markdown
- artifact table with path, byte count, and SHA-256 digest

The renderer is pure and deterministic. It does not inspect the archive object, rerun verification, recompute hashes, read files, write files, or mutate the summary payload.

## DES-AUTO-057: Research cycle planning handoff review packet artifact archive summary artifact archive summary Markdown gate

The review packet artifact archive summary artifact archive summary Markdown gate consumes:

- the machine-readable summary dictionary emitted by `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummary`
- the Markdown emitted by `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryMarkdown`

The gate returns a deterministic dictionary containing ready status, blocker messages, artifact count, checked artifact count, and finding count.

The gate blocks when:

- required source cycle, status, count, or audit lines are missing
- artifact count does not match the number of artifact records
- artifact records are not dictionaries
- any artifact row with path, byte count, and SHA-256 digest is missing

The gate does not inspect archive objects, rerun verification, recompute hashes, parse files, write files, or mutate inputs.

## DES-AUTO-058: Research cycle planning handoff review packet artifact archive summary artifact archive summary Markdown gate Markdown

The review packet artifact archive summary artifact archive summary Markdown gate Markdown renderer consumes the dictionary emitted by `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryMarkdownGate`.

The rendered document contains:

- ready state
- status
- artifact count
- checked artifact count
- finding count
- blocker messages, or `none` when clean

The renderer is pure and deterministic. It does not rerun the gate, inspect summary artifacts, parse files, write files, or mutate inputs.

## DES-AUTO-059: Research cycle planning handoff review packet artifact archive summary artifact archive summary artifacts

The review packet artifact archive summary artifact archive summary artifact builder consumes `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchive`.

The builder returns `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifacts` containing:

- the source summary artifact archive
- the machine-readable summary
- summary Markdown
- Markdown gate result
- Markdown gate audit document
- two packaged artifacts at caller-provided relative POSIX paths

The builder composes the summary renderer, Markdown renderer, Markdown gate, and gate Markdown renderer. It validates output paths with the shared manifest path validator. It does not rebuild manifests, rerun archive verification, read files, write files, or mutate inputs.

## DES-AUTO-060: Research cycle planning handoff review packet artifact archive summary artifact archive summary artifact manifest

The review packet artifact archive summary artifact archive summary artifact manifest builder consumes `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifacts`.

The builder returns `ResearchCyclePlanningPacketManifest` containing:

- source cycles copied from the machine-readable summary
- `ok` manifest status
- one manifest entry per packaged summary artifact
- relative artifact path
- UTF-8 byte count
- SHA-256 content digest

The builder reuses the shared manifest entry helper so digest and byte count semantics stay identical to earlier handoff artifact manifests. It does not render Markdown, verify drift, read files, write files, or mutate inputs.

## DES-AUTO-061: Research cycle planning handoff review packet artifact archive summary artifact archive summary artifact manifest Markdown

The review packet artifact archive summary artifact archive summary artifact manifest Markdown renderer consumes `ResearchCyclePlanningPacketManifest`.

The rendered document contains:

- title
- source cycles
- status
- artifact count
- table rows for path, SHA-256 digest, and byte count

The renderer delegates to the shared planning packet manifest Markdown renderer so table shape and ordering stay consistent across all handoff manifest surfaces. It does not rebuild manifests, verify drift, read files, write files, or mutate inputs.

## DES-AUTO-062: Research cycle planning handoff review packet artifact archive summary artifact archive summary artifact manifest Markdown verifier

The review packet artifact archive summary artifact archive summary artifact manifest Markdown verifier consumes `ResearchCyclePlanningPacketManifest` and rendered Markdown.

The verifier returns `ResearchCyclePlanningPacketManifestVerification` findings for:

- missing source cycle line
- missing status line
- missing artifact count line
- missing manifest table header
- missing per-artifact table row

Document-level findings use path `manifest.md`. Per-artifact findings use the affected artifact path. The verifier is pure and deterministic. It does not recompute artifact hashes, read files, write files, or mutate inputs.

## DES-AUTO-063: Research cycle planning handoff review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown

The review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown renderer consumes `ResearchCyclePlanningPacketManifestVerification`.

The rendered document contains:

- title
- status
- finding count
- path-specific finding list, or `none` when clean

The renderer delegates to the shared planning packet manifest verification Markdown renderer so status vocabulary and finding list shape stay consistent across all handoff manifest verification surfaces. It does not rerun verification, read files, write files, or mutate inputs.

## DES-AUTO-064: Research cycle planning handoff review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown gate

The review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown gate consumes `ResearchCyclePlanningPacketManifestVerification` and rendered verification Markdown.

The gate returns a deterministic dictionary containing:

- ready state
- gate status
- verification status
- finding count
- checked finding count
- blocker messages

The gate verifies that the Markdown contains the expected status line, finding count line, and one path-specific finding row for every verification finding. When the verification is clean, it verifies the clean `none` line instead. The gate is pure and deterministic. It does not rerun verification, read files, write files, parse manifests, recompute hashes, or mutate inputs.

## DES-AUTO-065: Research cycle planning handoff review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown gate Markdown

The review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown gate Markdown renderer consumes the dictionary emitted by `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdownVerificationMarkdownGate`.

The rendered document contains:

- title
- ready state
- gate status
- verification status
- finding count
- checked finding count
- blocker messages, or `none` when clean

The renderer is pure and deterministic. It does not rerun the gate, inspect manifests, parse files, write files, or mutate inputs.

## DES-AUTO-066: Research cycle planning handoff review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown artifacts

The review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact builder consumes `ResearchCyclePlanningPacketManifestVerification`.

The builder returns `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdownVerificationMarkdownArtifacts` containing:

- the machine-readable verification result
- verification Markdown
- verification Markdown gate result
- verification Markdown gate audit document
- two packaged artifacts at caller-provided relative POSIX paths

The builder composes the verification Markdown renderer, verification Markdown gate, and gate Markdown renderer. It validates output paths with the shared manifest path validator. It does not rebuild manifests, rerun manifest Markdown verification, read files, write files, or mutate inputs.
