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

## DES-AUTO-067: Research cycle planning handoff review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest

The review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest builder consumes `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdownVerificationMarkdownArtifacts`.

The builder returns `ResearchCyclePlanningPacketManifest` containing:

- source cycles copied from caller-provided metadata
- `ok` manifest status
- one manifest entry per packaged verification Markdown artifact
- relative artifact path
- UTF-8 byte count
- SHA-256 content digest

The builder reuses the shared manifest entry helper so digest and byte count semantics stay identical to earlier handoff artifact manifests. It does not render Markdown, verify drift, read files, write files, or mutate inputs.

## DES-AUTO-068: Research cycle planning handoff review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest Markdown

The review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest Markdown renderer consumes `ResearchCyclePlanningPacketManifest`.

The rendered document contains:

- title
- source cycles
- manifest status
- artifact count
- table rows for path, SHA-256 digest, and byte count

The renderer delegates to the shared planning packet manifest Markdown renderer so table shape, digest display, and artifact ordering stay consistent across all handoff manifest surfaces. It does not rebuild manifests, verify drift, read files, write files, or mutate inputs.

## DES-AUTO-069: Research cycle planning handoff review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest Markdown verifier

The review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest Markdown verifier consumes `ResearchCyclePlanningPacketManifest` and rendered Markdown.

The verifier returns `ResearchCyclePlanningPacketManifestVerification` findings for:

- missing source cycle line
- missing status line
- missing artifact count line
- missing manifest table header
- missing per-artifact table row

The verifier delegates to the shared final audit index manifest Markdown verifier so finding vocabulary and path attribution stay consistent with earlier handoff manifest surfaces. It does not recompute artifact hashes, read files, write files, or mutate inputs.

## DES-AUTO-070: Research cycle planning handoff review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest Markdown verification Markdown

The review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest Markdown verification Markdown renderer consumes `ResearchCyclePlanningPacketManifestVerification`.

The rendered document contains:

- title
- verification status
- finding count
- path-specific finding list, or `none` when clean

The renderer delegates to the shared planning packet manifest verification Markdown renderer so status vocabulary and finding list shape stay consistent across all handoff manifest verification surfaces. It does not rerun verification, inspect manifests, read files, write files, or mutate inputs.

## DES-AUTO-071: Research cycle planning handoff review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest Markdown verification Markdown gate

The review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest Markdown verification Markdown gate consumes `ResearchCyclePlanningPacketManifestVerification` and rendered verification Markdown.

The gate returns a deterministic dictionary containing:

- ready state
- gate status
- verification status
- finding count
- checked finding count
- blocker messages

The gate delegates to the shared manifest Markdown verification Markdown gate so status, count, finding row, and clean `none` checks stay consistent across all handoff verification audit surfaces. It does not rerun verification, inspect manifests, parse files, write files, or mutate inputs.

## DES-AUTO-072: Research cycle planning handoff review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest Markdown verification Markdown gate Markdown

The review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest Markdown verification Markdown gate Markdown renderer consumes the dictionary emitted by `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdownVerificationMarkdownArtifactManifestMarkdownVerificationMarkdownGate`.

The rendered document contains:

- title
- ready state
- gate status
- verification status
- finding count
- checked finding count
- blocker messages, or `none` when clean

The renderer delegates to the shared manifest Markdown verification Markdown gate Markdown renderer so audit document shape stays consistent across all handoff verification readiness surfaces. It does not rerun the gate, inspect manifests, parse files, write files, or mutate inputs.

## DES-AUTO-073: Research cycle planning handoff review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest Markdown verification Markdown artifacts

The review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest Markdown verification Markdown artifact builder consumes `ResearchCyclePlanningPacketManifestVerification`.

The builder returns `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdownVerificationMarkdownArtifactManifestMarkdownVerificationMarkdownArtifacts` containing:

- the machine-readable verification result
- verification Markdown
- verification Markdown gate result
- verification Markdown gate audit document
- two packaged artifacts at caller-provided relative POSIX paths

The builder composes the verification Markdown renderer, verification Markdown gate, and gate Markdown renderer. It validates output paths with the shared manifest path validator. It does not rebuild manifests, rerun manifest Markdown verification, read files, write files, or mutate inputs.

## DES-AUTO-074: Research cycle planning handoff review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest Markdown verification Markdown artifact manifest

The review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest Markdown verification Markdown artifact manifest builder consumes `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdownVerificationMarkdownArtifactManifestMarkdownVerificationMarkdownArtifacts`.

The builder returns `ResearchCyclePlanningPacketManifest` containing:

- source cycles copied from caller-provided metadata
- `ok` manifest status
- one manifest entry per packaged verification audit artifact
- relative artifact path
- UTF-8 byte count
- SHA-256 content digest

The builder reuses the shared manifest entry helper so digest and byte count semantics stay identical to earlier handoff artifact manifests. It does not render Markdown, verify drift, read files, write files, or mutate inputs.

## DES-AUTO-075: Research cycle planning handoff review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest Markdown verification Markdown artifact manifest Markdown

The review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest Markdown verification Markdown artifact manifest Markdown renderer consumes `ResearchCyclePlanningPacketManifest`.

The rendered document contains:

- title
- source cycles
- manifest status
- artifact count
- table rows for path, SHA-256 digest, and byte count

The renderer delegates to the shared planning packet manifest Markdown renderer so table shape, digest display, and artifact ordering stay consistent across all handoff manifest surfaces. It does not rebuild manifests, verify drift, read files, write files, or mutate inputs.

## DES-AUTO-076: Research cycle planning handoff review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest Markdown verification Markdown artifact manifest Markdown verifier

The review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest Markdown verification Markdown artifact manifest Markdown verifier consumes `ResearchCyclePlanningPacketManifest` and rendered Markdown.

The verifier returns `ResearchCyclePlanningPacketManifestVerification` findings for:

- missing source cycle line
- missing status line
- missing artifact count line
- missing manifest table header
- missing per-artifact table row

The verifier delegates to the shared final audit index manifest Markdown verifier so finding vocabulary and path attribution stay consistent with earlier handoff manifest surfaces. It does not recompute artifact hashes, read files, write files, or mutate inputs.

## DES-AUTO-077: Research cycle planning handoff review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest Markdown verification Markdown artifact manifest Markdown verification Markdown

The final verification artifact manifest verification Markdown renderer consumes `ResearchCyclePlanningPacketManifestVerification`.

The rendered document contains:

- title
- verification status
- finding count
- path-specific finding list, or `none` when clean

The renderer delegates to the shared planning packet manifest verification Markdown renderer so status vocabulary and finding list shape stay consistent across all handoff manifest verification surfaces. It does not rerun verification, inspect manifests, read files, write files, or mutate inputs.

## DES-AUTO-078: Research cycle planning handoff review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest Markdown verification Markdown artifact manifest Markdown verification Markdown gate

The final verification artifact manifest verification Markdown gate consumes `ResearchCyclePlanningPacketManifestVerification` and rendered Markdown.

The gate returns a deterministic readiness dictionary with:

- ready flag
- gate status
- verification status
- finding count
- checked finding count
- blocker messages

The gate delegates to the shared manifest verification Markdown gate so readiness vocabulary, blocker messages, and clean-state handling stay consistent with earlier verification audit surfaces. It does not rerun verification, recompute manifests, inspect artifacts, read files, write files, or mutate inputs.

## DES-AUTO-079: Identifier length audit

`NameLenAudit` parses Python source with the standard `ast` module and returns deterministic findings for over-limit class, function, argument, and assigned local names.

Each finding records:

- path
- line
- kind
- name
- length

The scanner is read-only and accepts either raw text, one file, or an explicit tuple of files. It does not rename code; it provides a complete list for the next controlled rename pass.

## DES-AUTO-080: Identifier rename planning

`NamePlan` consumes `NameHit` findings and returns deterministic `NameFix` suggestions.

The planner:

- derives short names from snake_case or CamelCase parts
- caps every suggestion at the configured limit
- adds numeric suffixes when multiple findings would produce the same suggestion
- records a reason in `original_length>limit` form

The planner remains read-only. Rename application is left to a later reviewed pass so tests, imports, and CodeFire Atom IDs can be updated deliberately.

## DES-AUTO-081: Identifier audit report

`NameReport` renders `NameFix` suggestions as a Markdown table with path, line, kind, original name, suggestion, and reason columns.

The report is deterministic, read-only, and emits an explicit clean row when there are no fixes. It is intended for review artifacts and manual rename planning before any source rewrite is attempted.

## DES-AUTO-082: Identifier audit run report

`NameAuditRun` composes `NameLenAudit`, `NamePlan`, and `NameReport` for an explicit tuple of Python files.

The runner:

- scans each requested file
- builds deterministic short-name suggestions
- renders the complete Markdown report

The runner remains read-only and does not discover files recursively, rewrite source, or mutate CodeFire Atom IDs. Callers control the file set so review passes can decide whether to audit only touched files or a whole project snapshot.

## DES-AUTO-083: Identifier audit file discovery

`NameFiles` converts explicit roots into a deterministic tuple of Python files for identifier audit runs.

The finder:

- accepts individual `.py` files
- recursively includes `.py` files under directory roots
- skips hidden, cache, and virtual-environment directories
- sorts paths by POSIX display form

The finder does not read file contents, parse Python, or apply rename suggestions. It only defines the reproducible file set that `NameAuditRun` can consume.

## DES-AUTO-084: Identifier project audit report

`NameProjRun` composes `NameFiles` and `NameAuditRun` for project-root naming reviews.

The project runner:

- discovers auditable Python files from explicit roots
- scans and plans identifier rename suggestions with the configured limit
- renders the complete Markdown report

The runner remains read-only. It does not rewrite source, infer repository roots, or mutate trace metadata; callers decide which roots represent the review scope.

## DES-AUTO-085: Identifier audit summary

`NameStats` consumes `NameHit` findings and returns a `NameStat` value.

The summary records:

- total finding count
- affected file count
- maximum identifier length
- longest identifier name

The summarizer is read-only and does not sort, rewrite, or plan rename suggestions. Empty findings produce a zero summary with an empty longest-name field.

## DES-AUTO-086: Identifier audit summary Markdown

`NameStatMd` renders a `NameStat` value as a deterministic Markdown table.

The renderer includes rows for:

- total finding count
- affected file count
- maximum identifier length
- longest identifier name

The renderer is read-only and does not recompute findings. Empty longest-name values render as `none` so clean summaries remain explicit.

## DES-AUTO-087: Identifier audit kind summary

`NameKinds` consumes `NameHit` findings and returns deterministic `(kind, count)` rows.

The grouper:

- counts findings for each Python identifier kind
- sorts rows by descending count
- uses kind name as a stable tie-breaker

The grouper is read-only and does not recompute findings, render Markdown, or mutate rename suggestions. It provides a compact workload breakdown for reviewers who need to decide whether class, function, argument, or local-name cleanup should be prioritized first.

## DES-AUTO-088: Identifier audit kind summary Markdown

`NameKindMd` renders `(kind, count)` rows as a deterministic Markdown table.

The renderer includes:

- a stable `Kind` / `Count` table header
- one row per supplied kind summary row
- an explicit `none` row when there are no findings

The renderer is read-only and does not regroup findings. It formats already-reviewed summary rows for reports and handoff documents.

## DES-AUTO-089: Identifier audit dashboard Markdown

`NameDash` renders a compact identifier audit dashboard from `NameHit` findings.

The dashboard:

- summarizes overall finding totals with `NameStats` and `NameStatMd`
- summarizes construct kinds with `NameKinds` and `NameKindMd`
- joins the two Markdown sections in a deterministic order

The dashboard is read-only. It does not scan files, plan renames, or mutate source; callers provide already-reviewed findings.

## DES-AUTO-090: Identifier audit dashboard run

`NameDashRun` composes `NameLenAudit` and `NameDash` for explicit Python file sets.

The runner:

- scans each supplied file with the configured identifier length limit
- summarizes the resulting findings with the dashboard renderer
- returns deterministic Markdown for review

The runner remains read-only. It does not discover roots, plan renames, or apply edits; callers decide the file scope.

## DES-AUTO-091: Identifier project dashboard report

`NameProjDash` composes `NameFiles` and `NameDashRun` for project-root naming dashboards.

The project dashboard:

- discovers auditable Python files from explicit roots
- scans discovered files with the configured identifier length limit
- renders the compact dashboard Markdown

The runner remains read-only. It does not apply rename suggestions, infer repository roots, or mutate trace metadata.

## DES-AUTO-092: Identifier longest-name ranking

`NameTop` ranks `NameHit` findings by descending identifier length.

The ranker:

- accepts already-scanned findings
- applies a positive row limit
- breaks ties by path, line, and name for deterministic output
- returns the original finding records without mutation

The ranker is read-only. It does not scan files, render Markdown, plan renames, or apply edits.

## DES-AUTO-093: Identifier longest-name Markdown

`NameTopMd` renders ranked `NameHit` findings as a deterministic Markdown table.

The renderer includes:

- path, line, kind, name, and length columns
- one row per ranked finding in caller-provided order
- an explicit empty row when no findings are supplied

The renderer is read-only. It does not rank findings, scan files, plan renames, or mutate source.

## DES-AUTO-094: Identifier longest-name run report

`NameTopRun` composes `NameLenAudit`, `NameTop`, and `NameTopMd` for explicit Python file sets.

The runner:

- scans each supplied file with the configured identifier length limit
- ranks the longest findings with the configured row limit
- renders the ranked findings as Markdown

The runner remains read-only. It does not discover project roots, plan renames, or mutate source.

## DES-AUTO-095: Identifier project longest-name report

`NameProjTop` composes `NameFiles` and `NameTopRun` for project-root longest-name reports.

The project runner:

- discovers auditable Python files from explicit roots
- scans discovered files with the configured identifier length limit
- ranks the longest findings with the configured row limit
- renders the ranked findings as Markdown

The runner remains read-only. It does not apply rename suggestions, infer repository roots, or mutate trace metadata.

## DES-AUTO-096: Identifier full audit Markdown

`NameFull` composes `NameDash`, `NameTop`, and `NameTopMd` for already-scanned identifier findings.

The renderer:

- renders the compact dashboard from all supplied findings
- ranks the longest findings with the configured row limit
- renders the ranked findings as Markdown
- joins the sections in deterministic order

The renderer is read-only. It does not scan files, discover roots, plan renames, or mutate source.

## DES-AUTO-097: Identifier full audit run report

`NameFullRun` composes `NameLenAudit` and `NameFull` for explicit Python file sets.

The runner:

- scans each supplied file with the configured identifier length limit
- renders the compact dashboard from all findings
- renders the longest-name priority table with the configured row limit
- returns deterministic Markdown for review

The runner remains read-only. It does not discover project roots, plan renames, or mutate source.

## DES-AUTO-098: Identifier project full audit report

`NameProjFull` composes `NameFiles` and `NameFullRun` for project-root full identifier audit reports.

The project runner:

- discovers auditable Python files from explicit roots
- scans discovered files with the configured identifier length limit
- renders the compact dashboard from all findings
- renders the longest-name priority table with the configured row limit

The runner remains read-only. It does not apply rename suggestions, infer repository roots, or mutate trace metadata.

## DES-AUTO-099: Identifier audit JSON

`NameJson` renders already-scanned `NameHit` findings as deterministic JSON.

The renderer:

- preserves path, line, kind, name, and length fields
- emits one object per finding in caller-provided order
- sorts object keys for stable output

The renderer is read-only. It does not scan files, rank findings, plan renames, or mutate source.

## DES-AUTO-100: Identifier audit JSON run

`NameJsonRun` composes `NameLenAudit` and `NameJson` for explicit Python file sets.

The runner:

- scans each supplied file with the configured identifier length limit
- renders the resulting findings as deterministic JSON
- preserves path, line, kind, name, and length fields

The runner remains read-only. It does not discover project roots, plan renames, or mutate source.

## DES-AUTO-101: Identifier project JSON audit

`NameProjJson` composes `NameFiles` and `NameJsonRun` for project-root JSON audit reports.

The project runner:

- discovers auditable Python files from explicit roots
- scans discovered files with the configured identifier length limit
- renders the resulting findings as deterministic JSON
- preserves path, line, kind, name, and length fields

The runner remains read-only. It does not apply rename suggestions, infer repository roots, or mutate trace metadata.

## DES-AUTO-102: Identifier audit summary JSON

`NameStatJson` renders a `NameStat` summary as deterministic JSON.

The renderer includes:

- total finding count
- affected file count
- maximum identifier length
- longest identifier name

The renderer is read-only. It does not scan files, group findings, render Markdown, or mutate source.

## DES-AUTO-103: Identifier dashboard JSON

`NameDashJson` renders identifier audit findings as a deterministic dashboard JSON object.

The renderer includes:

- summary values from all supplied findings
- finding-kind count rows from all supplied findings
- stable object keys for automation consumers

The renderer is read-only. It does not scan files, plan renames, render Markdown, or mutate source.

## DES-AUTO-104: Identifier dashboard JSON run

`NameDashJsonRun` composes `NameLenAudit` and `NameDashJson` for explicit Python file sets.

The runner:

- scans each supplied file with the configured identifier length limit
- renders summary values and finding-kind counts as deterministic JSON
- preserves stable object keys for automation consumers

The runner remains read-only. It does not discover project roots, apply rename suggestions, render Markdown, or mutate source.

## DES-AUTO-105: Identifier project dashboard JSON

`NamePjDashJson` composes `NameFiles` and `NameDashJsonRun` for project-root dashboard JSON reports.

The project runner:

- discovers auditable Python files from explicit roots
- scans discovered files with the configured identifier length limit
- renders summary values and finding-kind counts as deterministic JSON
- preserves stable object keys for automation consumers

The runner remains read-only. It does not apply rename suggestions, infer repository roots, render Markdown, or mutate source.

## DES-AUTO-106: Identifier longest-name JSON

`NameTopJson` composes `NameTop` and `NameJson` for ranked identifier cleanup priorities.

The renderer:

- ranks supplied findings by longest identifier first
- applies the configured row limit
- renders the ranked subset as deterministic JSON
- preserves path, line, kind, name, and length fields

The renderer is read-only. It does not scan files, discover project roots, plan renames, render Markdown, or mutate source.

## DES-AUTO-107: Identifier longest-name JSON run

`NameTopJsonRun` composes `NameLenAudit` and `NameTopJson` for explicit Python file sets.

The runner:

- scans each supplied file with the configured identifier length limit
- ranks the resulting findings by longest identifier first
- applies the configured row limit
- renders the ranked subset as deterministic JSON

The runner remains read-only. It does not discover project roots, apply rename suggestions, render Markdown, or mutate source.

## DES-AUTO-108: Identifier project longest-name JSON

`NameProjTopJson` composes `NameFiles` and `NameTopJsonRun` for project-root longest-name JSON reports.

The project runner:

- discovers auditable Python files from explicit roots
- scans discovered files with the configured identifier length limit
- ranks the resulting findings by longest identifier first
- applies the configured row limit
- renders the ranked subset as deterministic JSON

The runner remains read-only. It does not apply rename suggestions, infer repository roots, render Markdown, or mutate source.

## DES-AUTO-109: Identifier rename suggestion JSON

`NameFixJson` renders already-planned identifier rename suggestions as deterministic JSON.

The renderer includes:

- finding path, line, kind, and original name
- suggested replacement name
- planning reason
- stable object keys for automation consumers

The renderer is read-only. It does not scan files, generate rename plans, rank findings, render Markdown, or mutate source.

## DES-AUTO-110: Identifier rename suggestion JSON run

`NameFixJsonRun` composes `NameLenAudit`, `NamePlan`, and `NameFixJson` for explicit Python file sets.

The runner:

- scans each supplied file with the configured identifier length limit
- plans deterministic rename suggestions for over-limit identifiers
- renders the suggestions as deterministic JSON
- preserves stable object keys for automation consumers

The runner remains read-only. It does not discover project roots, apply rename suggestions, rank findings, render Markdown, or mutate source.

## DES-AUTO-111: Identifier project rename suggestion JSON

`NameProjFixJson` composes `NameFiles` and `NameFixJsonRun` for project-root rename suggestion JSON reports.

The project runner:

- discovers auditable Python files from explicit roots
- scans discovered files with the configured identifier length limit
- plans deterministic rename suggestions for over-limit identifiers
- renders the suggestions as deterministic JSON
- preserves stable object keys for automation consumers

The runner remains read-only. It does not apply rename suggestions, infer repository roots, rank findings, render Markdown, or mutate source.

## DES-AUTO-112: Identifier audit gate

`NameGate` evaluates already-scanned identifier findings as a pass/fail gate.

The gate:

- summarizes the supplied findings with `NameStats`
- returns `ok=true` only when the finding count is zero
- preserves the summary values for failure explanation

The gate is read-only. It does not scan files, discover project roots, render reports, apply rename suggestions, or mutate source.

## DES-AUTO-113: Identifier audit gate JSON

`NameGateJson` renders `NameGateRes` as deterministic JSON.

The renderer includes:

- the boolean gate result
- total finding count
- affected file count
- maximum identifier length
- longest identifier name

The renderer is read-only. It does not scan files, discover project roots, evaluate findings, render Markdown, or mutate source.

## DES-AUTO-114: Identifier audit gate Markdown

`NameGateMd` renders `NameGateRes` as deterministic Markdown.

The renderer includes:

- the pass/fail gate status
- total finding count
- affected file count
- maximum identifier length
- longest identifier name

The renderer is read-only. It does not scan files, discover project roots, evaluate findings, render JSON, or mutate source.

## DES-AUTO-115: Identifier audit gate run

`NameGateRun` composes `NameLenAudit`, `NameGate`, and `NameGateMd` for explicit Python file sets.

The runner:

- scans each supplied file with the configured identifier length limit
- evaluates the resulting findings as a pass/fail naming gate
- renders the gate result as deterministic Markdown
- preserves summary values for failure explanation

The runner remains read-only. It does not discover project roots, apply rename suggestions, render JSON, or mutate source.

## DES-AUTO-116: Identifier audit gate JSON run

`NameGateJsonRun` composes `NameLenAudit`, `NameGate`, and `NameGateJson` for explicit Python file sets.

The runner:

- scans each supplied file with the configured identifier length limit
- evaluates the resulting findings as a pass/fail naming gate
- renders the gate result as deterministic JSON
- preserves summary values for automation consumers

The runner remains read-only. It does not discover project roots, apply rename suggestions, render Markdown, or mutate source.

## DES-AUTO-117: Identifier project gate

`NameProjGate` composes `NameFiles` and `NameGateRun` for project-root naming gate reports.

The project runner:

- discovers auditable Python files from explicit roots
- scans discovered files with the configured identifier length limit
- evaluates the resulting findings as a pass/fail naming gate
- renders the gate result as deterministic Markdown

The runner remains read-only. It does not apply rename suggestions, infer repository roots, render JSON, or mutate source.

## DES-AUTO-118: Identifier project gate JSON

`NameProjGateJson` composes `NameFiles` and `NameGateJsonRun` for project-root naming gate JSON reports.

The project runner:

- discovers auditable Python files from explicit roots
- scans discovered files with the configured identifier length limit
- evaluates the resulting findings as a pass/fail naming gate
- renders the gate result as deterministic JSON

The runner remains read-only. It does not apply rename suggestions, infer repository roots, render Markdown, or mutate source.

## DES-AUTO-119: Identifier gate detail JSON

`NameGatePack` renders already-scanned identifier findings as one gate detail JSON report.

The renderer:

- evaluates the supplied findings with `NameGate`
- preserves the pass/fail gate result
- preserves summary values for automation consumers
- includes every finding path, line, kind, name, and length

The renderer is read-only. It does not scan files, discover project roots, plan rename suggestions, render Markdown, or mutate source.

## DES-AUTO-120: Identifier gate detail JSON run

`NameGatePackRun` composes `NameLenAudit` and `NameGatePack` for explicit Python file sets.

The runner:

- scans each supplied file with the configured identifier length limit
- evaluates the resulting findings as a pass/fail naming gate
- renders the gate status and finding details as deterministic JSON
- preserves summary values and every finding path, line, kind, name, and length

The runner remains read-only. It does not discover project roots, plan rename suggestions, render Markdown, or mutate source.

## DES-AUTO-121: Identifier project gate detail JSON

`NameProjGatePack` composes `NameFiles` and `NameGatePackRun` for project-root gate detail JSON reports.

The project runner:

- discovers auditable Python files from explicit roots
- scans discovered files with the configured identifier length limit
- evaluates pass/fail naming gate status
- renders the gate status and finding details as deterministic JSON
- preserves summary values and every finding path, line, kind, name, and length

The runner remains read-only. It does not infer repository roots, plan rename suggestions, render Markdown, or mutate source.

## DES-AUTO-122: Identifier gate exit code

`NameGateExit` converts `NameGateRes` into a process exit code.

The mapper:

- returns `0` for passing gate results
- returns `1` for failing gate results
- avoids reading files or parsing rendered reports

The mapper is read-only. It does not scan files, discover project roots, render reports, or mutate source.

## DES-AUTO-123: Identifier gate exit code run

`NameGateExitRun` composes `NameLenAudit`, `NameGate`, and `NameGateExit` for explicit Python file sets.

The runner:

- scans each supplied file with the configured identifier length limit
- evaluates the resulting findings as a pass/fail naming gate
- returns `0` for clean scans
- returns `1` when any over-limit identifiers are found

The runner remains read-only. It does not discover project roots, render reports, apply rename suggestions, or mutate source.

## DES-AUTO-124: Identifier project gate exit code

`NameProjGateExit` composes `NameFiles` and `NameGateExitRun` for project-root naming gate exit codes.

The project runner:

- discovers auditable Python files from explicit roots
- scans discovered files with the configured identifier length limit
- returns `0` for clean project scans
- returns `1` when any over-limit identifiers are found

The runner remains read-only. It does not infer repository roots, render reports, apply rename suggestions, or mutate source.

## DES-AUTO-125: Identifier gate log line

`NameGateLine` renders `NameGateRes` as one deterministic log line.

The renderer:

- emits pass/fail status
- emits hit and file counts
- emits the maximum identifier length
- emits the maximum identifier name, or `none` for clean results

The renderer is read-only. It does not scan files, discover project roots, render Markdown, render JSON, or mutate source.

## DES-AUTO-126: Identifier gate log line run

`NameGateLineRun` composes `NameLenAudit`, `NameGate`, and `NameGateLine` for explicit Python file sets.

The runner:

- scans each supplied file with the configured identifier length limit
- evaluates the resulting findings as a pass/fail naming gate
- renders a one-line log summary
- preserves status, hit count, file count, maximum identifier length, and maximum identifier name

The runner remains read-only. It does not discover project roots, render Markdown tables, render JSON, apply rename suggestions, or mutate source.

## DES-AUTO-127: Identifier project gate log line

`NameProjGateLine` composes `NameFiles` and `NameGateLineRun` for project-root naming gate log lines.

The project runner:

- discovers auditable Python files from explicit roots
- scans discovered files with the configured identifier length limit
- evaluates the resulting findings as a pass/fail naming gate
- renders a one-line log summary
- preserves status, hit count, file count, maximum identifier length, and maximum identifier name

The runner remains read-only. It does not infer repository roots, render Markdown tables, render JSON, apply rename suggestions, or mutate source.

## DES-AUTO-128: Identifier CI gate run

`NameCiRun` composes `NameLenAudit`, `NameGate`, `NameGateExit`, and `NameGateLine` for explicit Python file sets.

The runner:

- scans each supplied file with the configured identifier length limit
- evaluates the resulting findings once as a pass/fail naming gate
- returns the CI-oriented exit code from the gate result
- returns a one-line log summary from the same gate result
- avoids duplicate scans when CI wrappers need both status and printable output

The runner remains read-only. It does not discover project roots, render Markdown tables, render JSON, apply rename suggestions, or mutate source.

## DES-AUTO-129: Identifier project CI gate run

`NameProjCiRun` composes `NameFiles` and `NameCiRun` for project-root CI naming gate results.

The project runner:

- discovers auditable Python files from explicit roots
- scans discovered files with the configured identifier length limit
- evaluates the resulting findings once as a pass/fail naming gate
- returns the CI-oriented exit code from the gate result
- returns a one-line log summary from the same gate result
- avoids duplicate scans when CI wrappers need both status and printable output

The runner remains read-only. It does not infer repository roots, render Markdown tables, render JSON, apply rename suggestions, or mutate source.

## DES-AUTO-130: Identifier CI gate JSON

`NameCiJson` renders an identifier CI gate exit code and log line as deterministic JSON.

The renderer:

- preserves the numeric exit code
- preserves the printable one-line gate summary
- emits stable key ordering for downstream automation

The renderer is read-only. It does not scan files, discover project roots, evaluate gates, render Markdown tables, apply rename suggestions, or mutate source.

## DES-AUTO-131: Identifier CI gate JSON run

`NameCiJsonRun` composes `NameCiRun` and `NameCiJson` for explicit Python file sets.

The runner:

- scans each supplied file with the configured identifier length limit
- evaluates the resulting findings once as a pass/fail naming gate
- renders the CI-oriented exit code and one-line log summary as deterministic JSON
- avoids duplicate scans when CI wrappers need both machine-readable status and printable output

The runner remains read-only. It does not discover project roots, render Markdown tables, apply rename suggestions, or mutate source.

## DES-AUTO-133: Identifier CI status JSON

`NameCiStatusJson` renders an identifier CI gate exit code, log line, and explicit boolean status as deterministic JSON.

The renderer:

- preserves the numeric exit code
- preserves the printable one-line gate summary
- sets `ok` to true only when the exit code is 0
- emits stable key ordering for downstream automation

The renderer is read-only. It does not scan files, discover project roots, evaluate gates, render Markdown tables, apply rename suggestions, or mutate source.

## DES-AUTO-134: Identifier CI status JSON run

`NameCiStatusRun` composes `NameCiRun` and `NameCiStatusJson` for explicit Python file sets.

The runner:

- scans each supplied file with the configured identifier length limit
- evaluates the resulting findings once as a pass/fail naming gate
- renders the CI-oriented exit code, one-line log summary, and explicit `ok` boolean as deterministic JSON
- avoids duplicate scans when CI wrappers need both machine-readable status and printable output

The runner remains read-only. It does not discover project roots, render Markdown tables, apply rename suggestions, or mutate source.

## DES-AUTO-135: Identifier project CI status JSON run

`NameProjCiStatusRun` composes `NameFiles` and `NameCiStatusRun` for project-root CI naming gate payloads.

The project runner:

- discovers auditable Python files from explicit roots
- scans discovered files with the configured identifier length limit
- evaluates the resulting findings once as a pass/fail naming gate
- renders the CI-oriented exit code, one-line log summary, and explicit `ok` boolean as deterministic JSON
- avoids duplicate scans when CI wrappers need both machine-readable status and printable output

The runner remains read-only. It does not infer repository roots, render Markdown tables, apply rename suggestions, or mutate source.

## DES-AUTO-136: Identifier CI detail JSON run

`NameCiPack` renders an identifier CI gate exit code, one-line summary, explicit boolean status, and finding details as deterministic JSON. `NameCiPackRun` composes `NameLenAudit`, `NameGate`, `NameGateExit`, `NameGateLine`, and `NameCiPack` for explicit Python file sets.

The runner:

- scans each supplied file with the configured identifier length limit
- evaluates the resulting findings once as a pass/fail naming gate
- renders the CI-oriented exit code, one-line log summary, explicit `ok` boolean, and finding detail list as deterministic JSON
- preserves each finding path, line, kind, name, and length for CI annotations
- avoids duplicate scans when CI wrappers need both gate status and finding details

The runner remains read-only. It does not discover project roots, render Markdown tables, apply rename suggestions, or mutate source.

## DES-AUTO-137: Identifier project CI detail JSON run

`NameProjCiPackRun` composes `NameFiles` and `NameCiPackRun` for project-root CI naming gate detail payloads.

The project runner:

- discovers auditable Python files from explicit roots
- scans discovered files with the configured identifier length limit
- evaluates the resulting findings once as a pass/fail naming gate
- renders the CI-oriented exit code, one-line log summary, explicit `ok` boolean, and finding detail list as deterministic JSON
- preserves each finding path, line, kind, name, and length for whole-project CI annotations
- avoids duplicate scans when CI wrappers need both gate status and finding details

The runner remains read-only. It does not infer repository roots, render Markdown tables, apply rename suggestions, or mutate source.

## DES-AUTO-138: Identifier CI note run

`NameCiNote` renders identifier audit findings as deterministic one-line notes. `NameCiNoteRun` composes `NameLenAudit` and `NameCiNote` for explicit Python file sets.

The runner:

- scans each supplied file with the configured identifier length limit
- renders each finding as `path:line: kind name length=N`
- returns a stable `clean` marker when no findings exist
- preserves finding path, line, kind, name, and length without requiring JSON parsing

The runner remains read-only. It does not discover project roots, evaluate exit codes, render Markdown tables, apply rename suggestions, or mutate source.

## DES-AUTO-139: Identifier project CI note run

`NameProjCiNoteRun` composes `NameFiles` and `NameCiNoteRun` for project-root CI naming notes.

The project runner:

- discovers auditable Python files from explicit roots
- scans discovered files with the configured identifier length limit
- renders each finding as `path:line: kind name length=N`
- returns a stable `clean` marker when no findings exist
- preserves finding path, line, kind, name, and length without requiring JSON parsing

The runner remains read-only. It does not infer repository roots, evaluate exit codes, render Markdown tables, apply rename suggestions, or mutate source.

## DES-AUTO-140: Identifier bounded CI notes

`NameCiNoteTop` renders the top identifier audit findings as deterministic one-line CI notes.

The renderer:

- accepts already-scanned identifier findings and a positive note limit
- ranks findings with `NameTop`
- renders each emitted finding as `path:line: kind name length=N`
- appends `omitted=N` when additional findings are hidden
- returns the stable `clean` marker when no findings exist

The renderer remains read-only. It does not scan files, discover project roots, evaluate exit codes, render Markdown tables, apply rename suggestions, or mutate source.

## DES-AUTO-141: Identifier bounded CI note run

`NameCiNoteTopRun` composes `NameLenAudit` and `NameCiNoteTop` for bounded CI naming notes from explicit Python file sets.

The runner:

- scans each supplied file with the configured identifier length limit
- ranks findings with `NameTop` through `NameCiNoteTop`
- renders at most the configured number of findings as `path:line: kind name length=N`
- appends `omitted=N` when additional findings are hidden
- returns the stable `clean` marker when no findings exist

The runner remains read-only. It does not discover project roots, evaluate exit codes, render Markdown tables, apply rename suggestions, or mutate source.

## DES-AUTO-142: Identifier project bounded CI note run

`NameProjCiNoteTopRun` composes `NameFiles` and `NameCiNoteTopRun` for bounded CI naming notes from project roots.

The project runner:

- discovers auditable Python files from explicit roots
- scans discovered files with the configured identifier length limit
- renders at most the configured number of findings as `path:line: kind name length=N`
- appends `omitted=N` when additional findings are hidden
- returns the stable `clean` marker when no findings exist

The runner remains read-only. It does not infer repository roots, evaluate exit codes, render Markdown tables, apply rename suggestions, or mutate source.

## DES-AUTO-143: Identifier CI note Markdown

`NameCiNoteMd` wraps already-rendered identifier CI notes in a deterministic text code fence.

The renderer:

- accepts note text from `NameCiNote`, `NameCiNoteTop`, or their runners
- trims trailing newline noise from the supplied note
- renders the body as a Markdown `text` code fence
- preserves the `clean`, finding, and `omitted=N` lines without parsing them

The renderer remains read-only. It does not scan files, discover project roots, evaluate exit codes, apply rename suggestions, or mutate source.

## DES-AUTO-144: Identifier bounded CI note Markdown run

`NameCiNoteMdRun` composes `NameCiNoteTopRun` and `NameCiNoteMd` for bounded Markdown CI naming notes from explicit Python file sets.

The runner:

- scans each supplied file with the configured identifier length limit
- ranks findings with `NameCiNoteTopRun`
- renders at most the configured number of findings plus any omitted-count marker
- wraps the resulting note text in a stable Markdown `text` code fence

The runner remains read-only. It does not discover project roots, evaluate exit codes, apply rename suggestions, or mutate source.

## DES-AUTO-145: Identifier project bounded CI note Markdown run

`NameProjCiNoteMdRun` composes `NameFiles` and `NameCiNoteMdRun` for bounded Markdown CI naming notes from project roots.

The project runner:

- discovers auditable Python files from explicit roots
- scans discovered files with the configured identifier length limit
- renders bounded note details through `NameCiNoteMdRun`
- returns a Markdown `text` code fence suitable for review comments

The runner remains read-only. It does not infer repository roots, evaluate exit codes, apply rename suggestions, or mutate source.

## DES-AUTO-146: Identifier CI note Markdown section

`NameCiNoteSec` wraps already-rendered identifier CI notes in a titled Markdown section.

The renderer:

- accepts a caller-provided title and note text
- falls back to a stable heading when the title is blank
- delegates the fenced note body to `NameCiNoteMd`
- preserves the note lines without parsing them

The renderer remains read-only. It does not scan files, discover project roots, evaluate exit codes, apply rename suggestions, or mutate source.

## DES-AUTO-147: Identifier bounded CI note Markdown section run

`NameCiNoteSecRun` composes `NameCiNoteTopRun` and `NameCiNoteSec` for titled bounded Markdown CI naming notes from explicit Python file sets.

The runner:

- scans each supplied file with the configured identifier length limit
- ranks findings with `NameCiNoteTopRun`
- renders at most the configured number of findings plus any omitted-count marker
- wraps the resulting note text in a titled Markdown section

The runner remains read-only. It does not discover project roots, evaluate exit codes, apply rename suggestions, or mutate source.

## DES-AUTO-148: Identifier project bounded CI note Markdown section run

`NameProjCiNoteSecRun` composes `NameFiles` and `NameCiNoteSecRun` for titled bounded Markdown CI naming notes from project roots.

The project runner:

- discovers auditable Python files from explicit roots
- scans discovered files with the configured identifier length limit
- renders bounded note details through `NameCiNoteSecRun`
- returns a titled Markdown section suitable for review comments

The runner remains read-only. It does not infer repository roots, evaluate exit codes, apply rename suggestions, or mutate source.

## DES-AUTO-149: Identifier naming audit operations guide

`NameOpsGuide` renders the identifier naming audit's operator-facing Markdown guide from explicit display settings.

The renderer:

- documents the audit purpose in review-oriented language
- records the maximum identifier length and bounded review note limit that will be used
- states which Python syntax forms are audited
- lists the review workflow that contributors should follow
- names the CI payload renderers for status, annotation, and review-comment use cases

The renderer remains deterministic and read-only. It does not scan files, infer project roots, evaluate exit codes, apply rename suggestions, or mutate source.

## DES-AUTO-150: Identifier project operations document

`NameProjOpsDoc` composes `NameOpsGuide` and `NameProjCiNoteSecRun` for project-root naming audit operations documents.

The project document:

- renders the shared operations guide with the configured identifier length and note limit
- discovers auditable Python files from explicit project roots
- appends a titled Markdown section containing current bounded naming findings
- keeps guidance and live findings in one publishable review artifact

The renderer remains read-only. It does not infer repository roots, evaluate exit codes separately from the naming note runner, apply rename suggestions, or mutate source.

## DES-AUTO-151: Identifier CI aggregate summary JSON

`NameCiSummary` renders identifier audit findings as compact deterministic JSON for CI dashboard and telemetry surfaces.

The renderer:

- summarizes total findings, affected files, maximum identifier length, and maximum identifier name
- emits the longest finding with path, line, kind, name, and length
- groups findings by Python syntax kind using deterministic count/name ordering
- groups findings by file using deterministic count/path ordering
- returns schema-stable empty arrays and `null` longest finding for clean scans

The renderer remains read-only. It does not scan files, discover project roots, evaluate exit codes, render Markdown, apply rename suggestions, or mutate source.

## DES-AUTO-152: Identifier CI aggregate summary run

`NameCiSummaryRun` composes `NameLenAudit` and `NameCiSummary` for aggregate summary JSON from explicit Python file sets.

The runner:

- scans each supplied file with the configured identifier length limit
- passes all findings to the aggregate summary renderer
- preserves deterministic ordering from the scanner and renderer
- returns compact JSON suitable for dashboard ingestion

The runner remains read-only. It does not discover project roots, evaluate exit codes, apply rename suggestions, or mutate source.

## DES-AUTO-153: Identifier project CI aggregate summary run

`NameProjCiSummaryRun` composes `NameFiles` and `NameCiSummaryRun` for project-root aggregate summary JSON.

The project runner:

- discovers auditable Python files from explicit roots
- scans discovered files with the configured identifier length limit
- renders aggregate file/kind/longest summary JSON
- keeps whole-project naming telemetry compact enough for CI dashboards

The runner remains read-only. It does not infer repository roots, evaluate exit codes, apply rename suggestions, or mutate source.

## DES-AUTO-132: Identifier project CI gate JSON run

`NameProjCiJsonRun` composes `NameFiles` and `NameCiJsonRun` for project-root CI naming gate payloads.

The project runner:

- discovers auditable Python files from explicit roots
- scans discovered files with the configured identifier length limit
- evaluates the resulting findings once as a pass/fail naming gate
- renders the CI-oriented exit code and one-line log summary as deterministic JSON
- avoids duplicate scans when CI wrappers need both machine-readable status and printable output

The runner remains read-only. It does not infer repository roots, render Markdown tables, apply rename suggestions, or mutate source.

## DES-AUTO-154: Seed sweep planner

`SeedSweepPlanner` expands a base configuration dict into per-seed run configuration dicts.

The planner:

- validates that the maximum run count is a positive integer
- validates that every seed is an integer (booleans are rejected)
- rejects a duplicate seed with ValueError naming the duplicated seed
- truncates the validated seeds to the maximum run count
- returns a copy of the base configuration plus a `seed` entry for each run

The planner is pure. It does not mutate the base configuration or execute runs.
