# Experiment Automation Requirements

## REQ-AUTO-001: Experiment plan schema

An experiment plan shall include hypothesis, benchmark, baseline, metric, expected outcome, budget, runner command, artifacts to collect, and analysis criteria.

## REQ-AUTO-002: Runner interface

The system shall expose a runner interface that can execute local commands first and later support containers, job queues, and remote accelerators.

## REQ-AUTO-003: Run identity

Each experiment run shall receive a stable run ID that is used across logs, metrics, artifacts, reports, and evidence links.

## REQ-AUTO-004: Configuration capture

The runner shall capture the exact experiment configuration, including hyperparameters, seed, dataset version, benchmark split, and code reference.

## REQ-AUTO-005: Artifact collection

The runner shall collect declared artifacts such as metrics JSON, stdout, stderr, model summaries, plots, failure traces, and environment metadata.

## REQ-AUTO-006: Metric normalization

The system shall normalize metric direction and scale so that different benchmarks can be compared by a common scoring layer without losing raw values.

## REQ-AUTO-007: Baseline registry

The system shall maintain a registry of baselines for each benchmark, including source, command, expected metric, and confidence level.

## REQ-AUTO-008: Ablation protocol

The system shall support ablation experiments that isolate which part of a candidate caused improvement or regression.

## REQ-AUTO-009: Replication protocol

The system shall require repeated runs before promoting noisy improvements above configurable thresholds.

## REQ-AUTO-010: Failure classification

The analyst shall classify failures into categories such as implementation error, invalid hypothesis, insufficient budget, benchmark mismatch, instability, or inconclusive result.

## REQ-AUTO-011: Experiment queue

The system shall maintain a queue of proposed experiments with priority, expected information gain, estimated cost, and dependency constraints.

## REQ-AUTO-012: Stop criteria

The runner shall support stop criteria based on timeout, maximum cost, metric plateau, invalid output, or safety policy violation.

## REQ-AUTO-013: Result ingestion

The system shall ingest completed run artifacts into structured `ExperimentResult` records.

## REQ-AUTO-014: Analysis note generation

The analyst shall produce a concise analysis note for each run, including observed result, comparison against baseline, confidence, caveats, and recommended next action.

## REQ-AUTO-015: Candidate mutation

The planner shall generate follow-up candidates by mutating promising hypotheses and by simplifying or ablation-testing uncertain hypotheses.

## REQ-AUTO-016: Portfolio balance

The scheduler shall balance exploitative follow-ups on strong candidates with exploratory trials of novel candidates.

## REQ-AUTO-017: Regression detection

The system shall detect when a new candidate regresses against previously accepted candidates on shared benchmarks.

## REQ-AUTO-018: Experiment dependency graph

The system shall model dependencies between runs, including prerequisites, repeated trials, ablations, and benchmark unlocks.

## REQ-AUTO-019: Research decision records

Major research decisions shall be recorded with evidence, rejected alternatives, and expected follow-up work.

## REQ-AUTO-020: Continuous benchmark refresh

The benchmark registry shall be refreshable as datasets, baselines, or evaluation protocols change.

## REQ-AUTO-021: Campaign execution state

The system shall track research campaign milestones, completion criteria, blocked milestones, and next milestone recommendations from runs, evidence, and results.

## REQ-AUTO-022: Budgeted experiment batch planning

The scheduler shall assemble ready experiments into executable batches that respect total cost, maximum batch size, and benchmark diversity constraints.

## REQ-AUTO-023: Retry and escalation planning

The analyst shall convert failed, unstable, or inconclusive runs into retry, mutate, archive, or human-review recommendations with explicit rationale.

## REQ-AUTO-024: Evidence freshness audit

The system shall detect when evidence references an older benchmark, dataset, or protocol version and require refresh before strong claims.

## REQ-AUTO-025: Learning curve diagnosis

The analyst shall summarize metric trajectories across repeated attempts and detect plateau, regression, or unstable progress.

## REQ-AUTO-026: Candidate promotion policy

The system shall decide whether a candidate should be promoted, replicated, mutated, rejected, or held based on improvement size, confidence, replication count, and open blockers.

## REQ-AUTO-027: Experiment triage board

The system shall combine queued experiment value, promotion policy, learning curve diagnosis, open blockers, and remaining budget into a ranked triage board of next research actions.

## REQ-AUTO-028: Experiment iteration plan

The system shall convert triage decisions into a bounded next-iteration plan that separates executable experiments, promotion or rejection review items, and deferred work while respecting remaining budget and active-run capacity.

## REQ-AUTO-029: Experiment risk register

The system shall maintain a risk register for experiment campaigns, prioritize open risks by likelihood, impact, and severity, and surface blocker risks that must be mitigated before promotion or external claims.

## REQ-AUTO-030: Experiment evidence pack

The system shall build a compact evidence pack for candidate promotion or external review, combining improvement signal, confidence, replication count, evidence freshness gaps, open blocker risks, missing artifacts, and mitigation checklist into a deterministic readiness decision.

## REQ-AUTO-031: Claim review queue

The system shall turn evidence packs and reviewer objections into a deterministic review queue that prioritizes blocked or revision-needed claims before approval-ready claims and explains the required next action for each claim.

## REQ-AUTO-032: Claim audit trail

The system shall generate a deterministic audit trail for candidate claims that records evidence readiness, review decisions, external-claim eligibility, and required follow-up so that promotion decisions can be reconstructed without reading raw logs.

## REQ-AUTO-033: Claim release gate

The system shall evaluate whether a candidate claim may be externally released by combining evidence readiness, review decision, audit terminal status, and required release artifacts into an allow, hold, or reject gate decision.

## REQ-AUTO-034: Research frontier map

The system shall synthesize a ranked research frontier map from baseline gaps, negative results, novelty reviews, open risks, budget pressure, and challenge objectives. Each frontier item shall identify the mechanism family to explore, the expected evidence gain, the primary risk, the next experiment action, and the reason it is worth pursuing before lower-ranked alternatives.

## REQ-AUTO-035: Frontier experiment plan drafting

The system shall convert executable research frontier items into bounded experiment plan drafts with a hypothesis, benchmark target, baseline, metric, runner command, artifact contract, and analysis criteria. Deferred, risk-mitigation, or redesign frontier items shall not become executable plans until their blocking reason is resolved.

## REQ-AUTO-036: Frontier feedback integration

The system shall integrate completed run outcomes back into research frontier signals. Feedback shall update baseline gap, expected information gain, novelty pressure, negative-result overlap, blocker risks, and rationale without mutating the original signal. The update shall be deterministic, bounded, and conservative so failed, regressed, over-budget, or blocked runs reduce executable opportunity while replicated improvements increase priority for follow-up frontier planning.

## REQ-AUTO-037: Frontier drift report

The system shall summarize how research frontier priorities drift across long campaigns. A drift report shall compare previous and current ranked frontier items, identify new, removed, rising, falling, and stable frontiers, expose score and action changes, and recommend follow-up actions for high-impact drift without inspecting raw experiment logs.

## REQ-AUTO-038: Research cycle retrospective

The system shall summarize completed research cycles into policy adjustment recommendations. A retrospective shall combine completed run counts, improvements, regressions, failures, blockers, budget pressure, frontier drift, and evidence-ready claims into deterministic recommendations for the next planning cycle. The retrospective shall distinguish whether the next cycle should increase exploration, consolidate promising candidates, reduce cost, mitigate risks, archive stale paths, or keep the current policy.

## REQ-AUTO-039: Retrospective planning summary

The system shall render a research cycle retrospective into a compact planning summary suitable for the next docs-first development cycle. The summary shall include cycle IDs, priority, key rates, recommendations, and rationale in deterministic order so the next planning document can be generated without rereading raw run logs.

## REQ-AUTO-040: Research cycle plan synthesis

The system shall synthesize a next-cycle plan from retrospective recommendations. The plan shall assign deterministic lane items for mitigation, active experiments, review, archive, or deferred work; preserve the source cycle IDs and priority; enforce positive active capacity and non-negative remaining budget; and cap active experiment budget hints by the remaining budget so the next docs-first cycle can begin from a bounded execution plan rather than prose recommendations alone.

## REQ-AUTO-041: Research cycle plan Markdown rendering

The system shall render a synthesized research cycle plan into deterministic Markdown for docs-first planning. The rendered plan shall include source cycles, priority, active capacity, remaining budget, lane sections, item titles, source recommendations, concrete actions, budget hints, and rationales so future planning can be reviewed without inspecting Python objects or raw retrospective signals.

## REQ-AUTO-042: Research cycle plan linting

The system shall lint a synthesized or hand-authored research cycle plan before it is used as the next execution plan. The lint report shall detect missing source cycle IDs, invalid capacity or budget values, empty lane items, active-lane overcommitment, budget overcommitment, missing item titles or actions, empty rationales, negative budget hints, and duplicated recommendations. Findings shall be deterministic, severity-labeled, and machine-readable so docs-first planning can block unsafe execution while preserving non-blocking warnings for reviewer attention.

## REQ-AUTO-043: Research cycle plan lint Markdown rendering

The system shall render a research cycle plan lint report into deterministic Markdown for reviewer-facing planning logs. The rendered report shall include overall status, blocker count, warning count, severity sections, field paths, and finding messages so plan safety decisions can be reviewed without inspecting Python objects or raw lint tuples.

## REQ-AUTO-044: Research cycle planning packet

The system shall build a complete next-cycle planning packet from a retrospective report. The packet shall contain the synthesized plan, lint report, plan Markdown, and lint Markdown using the same deterministic rules as the individual plan synthesis, lint, and renderer components. This packet shall give docs-first automation one stable object to persist, review, and hand off without recomputing each planning artifact separately.

## REQ-AUTO-045: Research cycle planning packet Markdown

The system shall render a complete research cycle planning packet into one deterministic Markdown review document. The document shall include packet-level source cycles, priority, lint status, active item count, remaining budget, blocker count, warning count, the demoted plan Markdown, and the demoted lint Markdown so docs-first cycle handoff can persist one readable artifact without losing the underlying plan or safety decision.

## REQ-AUTO-046: Research cycle planning packet manifest

The system shall build a deterministic manifest for planning packet handoff artifacts. The manifest shall record source cycles, lint status, relative artifact paths, content SHA-256 digests, and byte counts for the combined packet Markdown, plan Markdown, and lint Markdown so downstream automation can verify that persisted handoff files still match the reviewed planning packet.

## REQ-AUTO-047: Research cycle planning packet manifest Markdown

The system shall render a planning packet manifest into deterministic Markdown for audit logs. The rendered document shall include source cycles, status, artifact count, and a stable table of artifact path, SHA-256 digest, and byte count so human reviewers can inspect the same handoff integrity metadata that automation validates.

## REQ-AUTO-048: Research cycle planning packet manifest verification

The system shall verify persisted planning packet artifact contents against a planning packet manifest. The verification shall report missing artifacts, SHA-256 digest mismatches, and byte count mismatches in deterministic manifest order so docs-first cycle handoff can detect drift between reviewed planning artifacts and stored artifacts before the next execution cycle begins.

## REQ-AUTO-049: Research cycle planning packet manifest verification Markdown

The system shall render planning packet manifest verification results into deterministic Markdown for audit logs. The rendered document shall include verification status, finding count, and path-scoped finding messages so human reviewers can inspect artifact handoff drift without reading raw verification objects.

## REQ-AUTO-050: Research cycle planning handoff bundle

The system shall build a complete planning handoff bundle from a retrospective report. The bundle shall include the planning packet, artifact contents for the packet, plan, and lint Markdown, the manifest, manifest Markdown, manifest verification result, and verification Markdown so docs-first automation can pass one deterministic handoff object to future storage or review steps.

## REQ-AUTO-051: Research cycle planning handoff bundle Markdown

The system shall render a planning handoff bundle into deterministic Markdown index form. The rendered index shall include source cycles, packet status, manifest status, verification status, artifact count, artifact paths with byte sizes, and audit document availability so reviewers and storage adapters can inspect the handoff surface without opening every artifact.

## REQ-AUTO-052: Research cycle planning handoff bundle summary

The system shall summarize a planning handoff bundle into a deterministic machine-readable index. The summary shall include source cycles, packet status, manifest status, verification status, artifact count, per-artifact path, byte count, SHA-256 digest, audit document availability, and verification finding count so automation can route clean or blocked handoff bundles without parsing Markdown.

## REQ-AUTO-053: Research cycle planning handoff readiness gate

The system shall evaluate a planning handoff bundle for storage or reviewer handoff readiness. The readiness result shall include a boolean ready flag, status label, blockers, warnings, artifact count, and verification finding count so automation can reject drifted or incomplete handoffs before persisting or forwarding them.

## REQ-AUTO-054: Research cycle planning handoff readiness Markdown

The system shall render planning handoff readiness results into deterministic Markdown. The rendered document shall include ready state, status, artifact count, finding count, blockers, and warnings so reviewers can audit the same routing decision that automation uses before storage or handoff.

## REQ-AUTO-055: Research cycle planning handoff review packet

The system shall assemble a planning handoff review packet from an already-built handoff bundle. The review packet shall include the original bundle, machine-readable bundle summary, readiness decision, and readiness Markdown so storage and reviewer workflows can consume one deterministic object without recomputing planning, verification, or routing state.

## REQ-AUTO-056: Research cycle planning handoff review packet Markdown

The system shall render a planning handoff review packet into deterministic Markdown. The rendered document shall include source cycles, readiness status, packet, manifest, and verification statuses, artifact count, finding count, artifact paths with byte sizes and content digests, blockers, warnings, and audit document availability so reviewers can inspect the final handoff decision from one document.

## REQ-AUTO-057: Research cycle planning handoff review packet artifacts

The system shall package a planning handoff review packet into deterministic handoff artifacts. The artifact set shall include review packet Markdown, readiness Markdown, manifest Markdown, and verification Markdown at caller-controlled relative POSIX paths so storage adapters can persist the final review surface without recomputing planning, routing, or manifest verification.

## REQ-AUTO-058: Research cycle planning handoff review packet artifact manifest

The system shall build a deterministic manifest for planning handoff review packet artifacts. The manifest shall preserve source cycles from the review packet summary and record each packaged artifact path, SHA-256 digest, and byte count so storage adapters can verify the persisted final review surface without reparsing Markdown.

## REQ-AUTO-059: Research cycle planning handoff review packet artifact manifest Markdown

The system shall render planning handoff review packet artifact manifests into deterministic Markdown. The rendered document shall include source cycles, manifest status, artifact count, and a path, SHA-256 digest, and byte count table so reviewers can audit the persisted final review artifact set without opening each artifact.

## REQ-AUTO-060: Research cycle planning handoff review packet artifact manifest verification

The system shall verify planning handoff review packet artifact manifests against artifact contents. The verification shall detect missing artifacts, SHA-256 digest drift, and byte count drift using the same finding shape as planning packet manifest verification so storage adapters can reject corrupted final review surfaces before handoff.

## REQ-AUTO-061: Research cycle planning handoff review packet artifact manifest verification Markdown

The system shall render planning handoff review packet artifact manifest verification results into deterministic Markdown. The rendered document shall include verification status, finding count, and path-scoped finding messages so reviewers can audit final review artifact integrity without reading machine-only verification payloads.

## REQ-AUTO-062: Research cycle planning handoff review packet artifact archive

The system shall build a final planning handoff review packet artifact archive. The archive shall contain the packaged review artifacts, their manifest, manifest Markdown, manifest verification result, and manifest verification Markdown as one deterministic in-memory object so storage adapters can persist a complete final review surface without recomputing or hand-assembling audit documents.

## REQ-AUTO-063: Research cycle planning handoff review packet artifact archive summary

The system shall summarize final planning handoff review packet artifact archives into a deterministic machine-readable index. The summary shall include source cycles, archive status, artifact count, artifact path, byte count, SHA-256 digest, manifest status, verification status, finding count, readiness status, and audit document availability so storage adapters and reviewers can inspect archived review surfaces without opening every artifact.

## REQ-AUTO-064: Research cycle planning handoff review packet artifact archive summary Markdown

The system shall render final planning handoff review packet artifact archive summaries into deterministic Markdown. The rendered document shall include source cycles, archive status, readiness status, manifest status, verification status, finding count, audit availability, and an artifact path, byte count, and SHA-256 digest table so reviewers can inspect the final archive index without reading machine-only summary payloads.

## REQ-AUTO-065: Research cycle planning handoff review packet artifact archive summary Markdown gate

The system shall evaluate final planning handoff review packet artifact archive summary Markdown for reviewer handoff readiness. The gate shall compare the supplied summary payload and Markdown text for required status lines, source cycle coverage, artifact count consistency, finding count consistency, audit availability lines, and one artifact table row per summary artifact so storage adapters can block incomplete or stale archive index documents before reviewer handoff.

## REQ-AUTO-066: Research cycle planning handoff review packet artifact archive summary Markdown gate Markdown

The system shall render final planning handoff review packet artifact archive summary Markdown gate results into deterministic Markdown. The rendered document shall include ready state, status, artifact count, checked artifact count, finding count, and blocker messages so reviewers can audit why an archive summary Markdown document was accepted or blocked without reading machine-only gate payloads.

## REQ-AUTO-067: Research cycle planning handoff review packet artifact archive summary artifacts

The system shall package final planning handoff review packet artifact archive summary review surfaces into deterministic handoff artifacts. The artifact set shall include archive summary Markdown and archive summary Markdown gate Markdown at caller-controlled relative POSIX paths, plus the machine-readable summary and gate result, so storage adapters can persist the final archive index and its handoff readiness audit without recomputing archive summaries or revalidating Markdown later.

## REQ-AUTO-068: Research cycle planning handoff review packet artifact archive summary artifact manifest

The system shall build a deterministic manifest for final planning handoff review packet artifact archive summary artifacts. The manifest shall preserve source cycles from the archive summary payload and record each packaged summary artifact path, SHA-256 digest, and byte count so storage adapters can verify persisted archive summary and gate audit documents without reparsing Markdown.

## REQ-AUTO-069: Research cycle planning handoff review packet artifact archive summary artifact manifest Markdown

The system shall render final planning handoff review packet artifact archive summary artifact manifests into deterministic Markdown. The rendered document shall include source cycles, manifest status, artifact count, and a path, SHA-256 digest, and byte count table so reviewers can audit persisted archive summary and gate audit artifact integrity metadata without opening each artifact.

## REQ-AUTO-070: Research cycle planning handoff review packet artifact archive summary artifact manifest verification

The system shall verify final planning handoff review packet artifact archive summary artifact manifests against artifact contents. The verification shall detect missing artifacts, SHA-256 digest drift, and byte count drift using the shared planning packet manifest verification shape so storage adapters can reject corrupted archive summary and gate audit documents before reviewer handoff.

## REQ-AUTO-071: Research cycle planning handoff review packet artifact archive summary artifact manifest verification Markdown

The system shall render final planning handoff review packet artifact archive summary artifact manifest verification results into deterministic Markdown. The rendered document shall include verification status, finding count, and path-scoped finding messages so reviewers can audit archive summary and gate audit artifact integrity without reading machine-only verification payloads.

## REQ-AUTO-072: Research cycle planning handoff review packet artifact archive summary artifact archive

The system shall build a final archive summary artifact archive. The archive shall contain the packaged archive summary artifacts, their manifest, manifest Markdown, manifest verification result, and manifest verification Markdown as one deterministic in-memory object so storage adapters can persist a complete archive summary audit surface without recomputing or hand-assembling integrity documents.

## REQ-AUTO-073: Research cycle planning handoff review packet artifact archive summary artifact archive summary

The system shall summarize final archive summary artifact archives into a deterministic machine-readable index. The summary shall include source cycles, parent archive status, summary artifact archive status, artifact count, artifact path, byte count, SHA-256 digest, manifest status, verification status, finding count, and audit document availability so storage adapters and reviewers can inspect persisted archive summary audit surfaces without opening every artifact.
