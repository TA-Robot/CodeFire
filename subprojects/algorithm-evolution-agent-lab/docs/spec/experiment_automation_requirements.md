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

## REQ-AUTO-074: Research cycle planning handoff review packet artifact archive summary artifact archive summary Markdown

The system shall render final archive summary artifact archive summaries into deterministic Markdown. The rendered document shall include source cycles, parent archive status, summary artifact archive status, manifest status, verification status, artifact count, finding count, audit availability, and an artifact path, byte count, and SHA-256 digest table so reviewers can inspect persisted archive summary audit indexes without reading machine-only summary payloads.

## REQ-AUTO-075: Research cycle planning handoff review packet artifact archive summary artifact archive summary Markdown gate

The system shall evaluate final archive summary artifact archive summary Markdown for reviewer handoff readiness. The gate shall compare the supplied summary payload and Markdown text for required status lines, source cycle coverage, parent archive status, artifact count consistency, finding count consistency, audit availability lines, and one artifact table row per summary artifact so storage adapters can block incomplete or stale persisted archive summary audit indexes before reviewer handoff.

## REQ-AUTO-076: Research cycle planning handoff review packet artifact archive summary artifact archive summary Markdown gate Markdown

The system shall render final archive summary artifact archive summary Markdown gate results into deterministic Markdown. The rendered document shall include ready state, status, artifact count, checked artifact count, finding count, and blocker messages so reviewers can audit why a persisted archive summary audit index was accepted or blocked without reading machine-only gate payloads.

## REQ-AUTO-077: Research cycle planning handoff review packet artifact archive summary artifact archive summary artifacts

The system shall package final archive summary artifact archive summary review surfaces into deterministic handoff artifacts. The artifact set shall include archive summary artifact archive summary Markdown and its Markdown gate audit at caller-controlled relative POSIX paths, plus the machine-readable summary and gate result, so storage adapters can persist the final audit index surface without recomputing summaries or revalidating Markdown later.

## REQ-AUTO-078: Research cycle planning handoff review packet artifact archive summary artifact archive summary artifact manifest

The system shall generate a deterministic manifest for final archive summary artifact archive summary artifact sets. The manifest shall include source cycles, an ok status, and one path, byte count, and SHA-256 digest entry per packaged summary artifact so storage adapters and reviewers can verify persisted final audit index artifacts without reading their full Markdown bodies.

## REQ-AUTO-079: Research cycle planning handoff review packet artifact archive summary artifact archive summary artifact manifest Markdown

The system shall render final archive summary artifact archive summary artifact manifests into deterministic Markdown. The rendered document shall include source cycles, status, artifact count, and a path, SHA-256 digest, and byte count table so reviewers can inspect persisted final audit index artifact manifests without reading machine-only manifest payloads.

## REQ-AUTO-080: Research cycle planning handoff review packet artifact archive summary artifact archive summary artifact manifest Markdown verifier

The system shall verify final archive summary artifact archive summary artifact manifest Markdown against its machine-readable manifest. The verifier shall detect missing source cycle, status, artifact count, table header, and artifact row lines, and shall report findings with paths so storage adapters can block stale or incomplete reviewer-facing manifest documents before final handoff.

## REQ-AUTO-081: Research cycle planning handoff review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown

The system shall render final archive summary artifact archive summary artifact manifest Markdown verification results into deterministic Markdown. The rendered document shall include status, finding count, and path-specific findings or `none` when clean so reviewers can audit manifest Markdown readiness without reading machine-only verification payloads.

## REQ-AUTO-082: Research cycle planning handoff review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown gate

The system shall evaluate final archive summary artifact archive summary artifact manifest Markdown verification Markdown for reviewer handoff readiness. The gate shall compare the supplied verification result and rendered Markdown for status, finding count, and each path-specific finding line, or the clean `none` line when no findings exist, so storage adapters can block stale or incomplete verification audit documents before final handoff.

## REQ-AUTO-083: Research cycle planning handoff review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown gate Markdown

The system shall render final archive summary artifact archive summary artifact manifest Markdown verification Markdown gate results into deterministic Markdown. The rendered document shall include ready state, gate status, verification status, finding count, checked finding count, and blocker messages or `none` when clean so reviewers can audit verification audit document readiness without reading machine-only gate payloads.

## REQ-AUTO-084: Research cycle planning handoff review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown artifacts

The system shall package final archive summary artifact archive summary artifact manifest Markdown verification review surfaces into deterministic handoff artifacts. The artifact set shall include verification Markdown and verification Markdown gate Markdown at caller-controlled relative POSIX paths, plus the machine-readable verification result and gate result, so storage adapters can persist final verification audit readiness without recomputing verification results or revalidating Markdown later.

## REQ-AUTO-085: Research cycle planning handoff review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest

The system shall generate a deterministic manifest for final archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact sets. The manifest shall include a stable status and one path, byte count, and SHA-256 digest entry per packaged verification audit artifact so storage adapters and reviewers can verify persisted verification Markdown and gate audit documents without reading their full Markdown bodies.

## REQ-AUTO-086: Research cycle planning handoff review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest Markdown

The system shall render final archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifests into deterministic Markdown. The rendered document shall include source cycles, manifest status, artifact count, and one path, SHA-256 digest, and byte count row per packaged verification audit artifact so reviewers can inspect persisted verification audit artifact integrity metadata without reading machine-only manifest payloads.

## REQ-AUTO-087: Research cycle planning handoff review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest Markdown verifier

The system shall verify final archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest Markdown against its machine-readable manifest. The verifier shall detect missing source cycle, status, artifact count, table header, and artifact row lines, and shall report findings with paths so storage adapters can block stale or incomplete verification audit artifact manifest documents before final reviewer handoff.

## REQ-AUTO-088: Research cycle planning handoff review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest Markdown verification Markdown

The system shall render final archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest Markdown verification results into deterministic Markdown. The rendered document shall include verification status, finding count, and path-scoped finding messages or `none` when clean so reviewers can audit verification audit artifact manifest Markdown readiness without reading machine-only verification payloads.

## REQ-AUTO-089: Research cycle planning handoff review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest Markdown verification Markdown gate

The system shall evaluate final archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest Markdown verification Markdown for reviewer handoff readiness. The gate shall compare the supplied verification result and rendered Markdown for status, finding count, and each path-specific finding line, or the clean `none` line when no findings exist, so storage adapters can block stale or incomplete verification audit artifact manifest Markdown verification audit documents before final handoff.

## REQ-AUTO-090: Research cycle planning handoff review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest Markdown verification Markdown gate Markdown

The system shall render final archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest Markdown verification Markdown gate results into deterministic Markdown. The rendered document shall include ready state, gate status, verification status, finding count, checked finding count, and blocker messages or `none` when clean so reviewers can audit verification audit artifact manifest Markdown verification audit readiness without reading machine-only gate payloads.

## REQ-AUTO-091: Research cycle planning handoff review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest Markdown verification Markdown artifacts

The system shall package final archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest Markdown verification Markdown review surfaces into deterministic handoff artifacts. The artifact set shall include verification Markdown and verification Markdown gate Markdown at caller-controlled relative POSIX paths, plus the machine-readable verification result and gate result, so storage adapters can persist final verification audit artifact manifest Markdown verification readiness without recomputing verification results or revalidating Markdown later.

## REQ-AUTO-092: Research cycle planning handoff review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest Markdown verification Markdown artifact manifest

The system shall generate a deterministic manifest for final archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest Markdown verification Markdown artifact sets. The manifest shall include source cycles, an ok status, and one path, byte count, and SHA-256 digest entry per packaged verification audit artifact so storage adapters and reviewers can verify persisted final verification audit artifact manifest Markdown verification documents without reading their full Markdown bodies.

## REQ-AUTO-093: Research cycle planning handoff review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest Markdown verification Markdown artifact manifest Markdown

The system shall render final archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest Markdown verification Markdown artifact manifests into deterministic Markdown. The rendered document shall include source cycles, manifest status, artifact count, and one path, SHA-256 digest, and byte count row per packaged verification audit artifact so reviewers can inspect persisted final verification audit artifact manifest Markdown verification artifact integrity metadata without reading machine-only manifest payloads.

## REQ-AUTO-094: Research cycle planning handoff review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest Markdown verification Markdown artifact manifest Markdown verifier

The system shall verify final archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest Markdown verification Markdown artifact manifest Markdown against its machine-readable manifest. The verifier shall detect missing source cycle, status, artifact count, table header, and artifact row lines, and shall report findings with paths so storage adapters can block stale or incomplete final verification audit artifact manifest Markdown verification artifact manifest documents before final handoff.

## REQ-AUTO-095: Research cycle planning handoff review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest Markdown verification Markdown artifact manifest Markdown verification Markdown

The system shall render final archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest Markdown verification Markdown artifact manifest Markdown verification results into deterministic Markdown. The rendered document shall include verification status, finding count, and path-scoped finding messages or `none` when clean so reviewers can audit final verification audit artifact manifest readiness without reading machine-only verification payloads.

## REQ-AUTO-096: Research cycle planning handoff review packet artifact archive summary artifact archive summary artifact manifest Markdown verification Markdown artifact manifest Markdown verification Markdown artifact manifest Markdown verification Markdown gate

The system shall evaluate final verification artifact manifest verification Markdown for reviewer handoff readiness. The gate shall compare the supplied verification result and rendered Markdown for verification status, finding count, and each path-scoped finding line, or the clean `none` line when no findings exist, so storage adapters can block stale or incomplete final verification audit artifact manifest readiness documents before final handoff.

## REQ-AUTO-097: Identifier length audit

The system shall scan Python source for class, function, argument, and assigned local names that exceed a configurable maximum identifier length. The default limit shall be 25 characters so review cycles can expose long names while still allowing readable domain terms.

## REQ-AUTO-098: Identifier rename planning

The system shall convert identifier length audit findings into deterministic short-name suggestions. Each suggestion shall stay within the configured maximum length and preserve a reason that records the original length violation.

## REQ-AUTO-099: Identifier audit report

The system shall render identifier rename suggestions into a reviewer-readable table. The report shall include path, line, kind, original name, suggested name, and reason so rename work can be reviewed before code is changed.

## REQ-AUTO-100: Identifier audit run report

The system shall scan an explicit set of Python files, build deterministic rename suggestions for every over-limit identifier, and render the complete result as a reviewer-readable report. The run report shall reuse the configured maximum identifier length so review cycles can list all current naming problems without manually wiring scanner, planner, and renderer calls.

## REQ-AUTO-101: Identifier audit file discovery

The system shall discover Python source files under explicit roots for identifier audit runs. Discovery shall include `.py` files, skip hidden/cache/virtual-environment directories, accept individual Python files, and return a deterministic path order so whole-project naming reviews can be reproduced.

## REQ-AUTO-102: Identifier project audit report

The system shall render an identifier audit report from explicit project roots. The report workflow shall discover auditable Python files, scan every discovered file for over-limit identifiers, build deterministic rename suggestions, and render the complete reviewer-readable table so whole-project naming reviews can be run from root paths.

## REQ-AUTO-103: Identifier audit summary

The system shall summarize identifier audit findings with total finding count, affected file count, maximum identifier length, and the longest identifier name. The summary shall be deterministic so review cycles can quickly understand the rename workload before reading the full table.

## REQ-AUTO-104: Identifier audit summary Markdown

The system shall render identifier audit summary values into deterministic Markdown. The rendered summary shall include total findings, affected file count, maximum identifier length, and longest identifier name so reviewers can inspect naming workload at a glance.

## REQ-AUTO-105: Identifier audit kind summary

The system shall group identifier audit findings by finding kind. The grouped summary shall include each kind and its finding count, sorted by descending count and then kind name, so reviewers can see which Python construct categories create the naming workload.

## REQ-AUTO-106: Identifier audit kind summary Markdown

The system shall render identifier audit kind summary rows into deterministic Markdown. The rendered table shall include kind and count columns, and shall emit an explicit empty row when there are no findings, so reviewers can inspect construct-category naming workload without reading machine-only tuples.

## REQ-AUTO-107: Identifier audit dashboard Markdown

The system shall render a compact identifier audit dashboard from audit findings. The dashboard shall include the overall summary Markdown and the kind summary Markdown in a deterministic order so reviewers can inspect naming workload totals and construct-category distribution from one report section.

## REQ-AUTO-108: Identifier audit dashboard run

The system shall scan an explicit set of Python files and render the compact identifier audit dashboard for the resulting findings. The run shall reuse the configured maximum identifier length and shall preserve deterministic dashboard output so reviewers can inspect touched-file naming workload without manually wiring scanner and dashboard calls.

## REQ-AUTO-109: Identifier project dashboard report

The system shall render an identifier audit dashboard from explicit project roots. The workflow shall discover auditable Python files, scan every discovered file for over-limit identifiers, and render deterministic dashboard Markdown so whole-project naming reviews can inspect totals and construct-kind distribution from root paths.

## REQ-AUTO-110: Identifier longest-name ranking

The system shall rank identifier audit findings by longest identifier first. The ranking shall be deterministic, configurable by maximum row count, and preserve original finding metadata so reviewers can prioritize the most severe rename work before reading every finding.

## REQ-AUTO-111: Identifier longest-name Markdown

The system shall render ranked identifier audit findings into deterministic Markdown. The table shall include path, line, kind, name, and length columns, and shall emit an explicit empty row when no ranked findings are present so reviewers can paste longest-name priorities into cleanup notes.

## REQ-AUTO-112: Identifier longest-name run report

The system shall scan an explicit set of Python files, rank the longest identifier findings with a configurable row limit, and render the ranked findings as deterministic Markdown. The run report shall reuse the configured maximum identifier length so reviewers can generate cleanup priority notes without manually wiring scanner, ranker, and renderer calls.

## REQ-AUTO-113: Identifier project longest-name report

The system shall render a longest-name identifier audit report from explicit project roots. The workflow shall discover auditable Python files, scan every discovered file with the configured identifier length limit, rank the longest findings with a configurable row limit, and render deterministic Markdown so whole-project naming cleanup can start from root paths.

## REQ-AUTO-114: Identifier full audit Markdown

The system shall render a full identifier audit Markdown report from already-scanned findings. The report shall include the compact dashboard summary and a longest-name priority table in deterministic order so reviewers can inspect both workload size and first cleanup targets in one report.

## REQ-AUTO-115: Identifier full audit run report

The system shall scan an explicit set of Python files and render the full identifier audit Markdown report for the resulting findings. The run report shall reuse the configured maximum identifier length and longest-name row limit so reviewers can inspect workload size, construct-kind distribution, and first cleanup targets from a file list.

## REQ-AUTO-116: Identifier project full audit report

The system shall render a full identifier audit Markdown report from explicit project roots. The workflow shall discover auditable Python files, scan every discovered file with the configured identifier length limit, and render the dashboard plus longest-name priority sections so whole-project naming reviews can start from root paths.

## REQ-AUTO-117: Identifier audit JSON

The system shall render identifier audit findings as deterministic JSON. The JSON output shall preserve each finding path, line, kind, name, and length so automation can consume the same audit data without parsing Markdown reports.

## REQ-AUTO-118: Identifier audit JSON run

The system shall scan an explicit set of Python files and render identifier audit findings as deterministic JSON. The run shall reuse the configured maximum identifier length and preserve each finding path, line, kind, name, and length so automation can consume touched-file audit data directly.

## REQ-AUTO-119: Identifier project JSON audit

The system shall render identifier audit findings as deterministic JSON from explicit project roots. The workflow shall discover auditable Python files, scan every discovered file with the configured identifier length limit, and preserve each finding path, line, kind, name, and length so automation can consume whole-project audit data directly.

## REQ-AUTO-120: Identifier audit summary JSON

The system shall render identifier audit summary values as deterministic JSON. The output shall preserve total finding count, affected file count, maximum identifier length, and longest identifier name so automation can consume naming workload summaries without parsing Markdown.

## REQ-AUTO-121: Identifier dashboard JSON

The system shall render a compact identifier audit dashboard as deterministic JSON from already-scanned findings. The output shall include the summary values and finding-kind counts so automation can consume naming workload totals and construct-category distribution without parsing Markdown.

## REQ-AUTO-122: Identifier dashboard JSON run

The system shall scan an explicit set of Python files and render a compact identifier audit dashboard as deterministic JSON. The run shall reuse the configured maximum identifier length and include summary values plus finding-kind counts so automation can consume touched-file dashboard totals directly.

## REQ-AUTO-123: Identifier project dashboard JSON

The system shall render a compact identifier audit dashboard as deterministic JSON from explicit project roots. The workflow shall discover auditable Python files, scan every discovered file with the configured identifier length limit, and include summary values plus finding-kind counts so automation can consume whole-project dashboard totals directly.

## REQ-AUTO-124: Identifier longest-name JSON

The system shall rank identifier audit findings by longest identifier first and render the ranked subset as deterministic JSON. The JSON output shall preserve path, line, kind, name, and length fields so automation can consume cleanup priorities without parsing Markdown.

## REQ-AUTO-125: Identifier longest-name JSON run

The system shall scan an explicit set of Python files, rank the longest identifier findings with a configurable row limit, and render the ranked subset as deterministic JSON. The run shall reuse the configured maximum identifier length so automation can consume touched-file cleanup priorities without parsing Markdown.

## REQ-AUTO-126: Identifier project longest-name JSON

The system shall render longest-name identifier audit priorities as deterministic JSON from explicit project roots. The workflow shall discover auditable Python files, scan every discovered file with the configured identifier length limit, rank findings with a configurable row limit, and preserve finding metadata so automation can consume whole-project cleanup priorities without parsing Markdown.

## REQ-AUTO-127: Identifier rename suggestion JSON

The system shall render identifier rename suggestions as deterministic JSON from already-planned fixes. The JSON output shall preserve path, line, kind, original name, suggestion, and reason fields so automation can consume rename plans without parsing Markdown reports.

## REQ-AUTO-128: Identifier rename suggestion JSON run

The system shall scan an explicit set of Python files, plan over-limit identifier rename suggestions, and render the suggestions as deterministic JSON. The run shall reuse the configured maximum identifier length and preserve path, line, kind, original name, suggestion, and reason fields so automation can consume touched-file rename plans directly.

## REQ-AUTO-129: Identifier project rename suggestion JSON

The system shall render identifier rename suggestions as deterministic JSON from explicit project roots. The workflow shall discover auditable Python files, scan every discovered file with the configured identifier length limit, plan over-limit rename suggestions, and preserve path, line, kind, original name, suggestion, and reason fields so automation can consume whole-project rename plans directly.

## REQ-AUTO-130: Identifier audit gate

The system shall evaluate identifier audit findings as a deterministic pass/fail gate. The gate shall pass only when no over-limit identifiers are present, and it shall preserve the finding summary so CI or review automation can explain why the naming check failed.

## REQ-AUTO-131: Identifier audit gate JSON

The system shall render identifier audit gate results as deterministic JSON. The output shall preserve the pass/fail result plus the summary values so CI or review automation can consume gate status without parsing Markdown or Python objects.

## REQ-AUTO-132: Identifier audit gate Markdown

The system shall render identifier audit gate results as deterministic Markdown. The output shall preserve the pass/fail status plus the summary values so CI or review automation can publish a human-readable naming gate result without parsing JSON or Python objects.

## REQ-AUTO-133: Identifier audit gate run

The system shall scan an explicit set of Python files and render the identifier audit gate result as deterministic Markdown. The run shall reuse the configured maximum identifier length so CI or review automation can evaluate touched-file naming status without manually wiring scanner, gate, and renderer calls.

## REQ-AUTO-134: Identifier audit gate JSON run

The system shall scan an explicit set of Python files and render the identifier audit gate result as deterministic JSON. The run shall reuse the configured maximum identifier length so CI or review automation can consume touched-file naming gate status without manually wiring scanner, gate, and JSON renderer calls.

## REQ-AUTO-135: Identifier project gate

The system shall render an identifier audit gate result from explicit project roots. The workflow shall discover auditable Python files, scan every discovered file with the configured identifier length limit, evaluate the pass/fail naming gate, and render deterministic Markdown so whole-project naming status can be reviewed from root paths.

## REQ-AUTO-136: Identifier project gate JSON

The system shall render an identifier audit gate result as deterministic JSON from explicit project roots. The workflow shall discover auditable Python files, scan every discovered file with the configured identifier length limit, evaluate the pass/fail naming gate, and preserve summary values so whole-project naming status can be consumed by automation.

## REQ-AUTO-137: Identifier gate detail JSON

The system shall render identifier audit gate status and finding details as one deterministic JSON object from already-scanned findings. The output shall preserve the pass/fail result, summary values, and every finding path, line, kind, name, and length so CI or review automation can consume both gate status and failure details without joining separate reports.

## REQ-AUTO-138: Identifier gate detail JSON run

The system shall scan an explicit set of Python files and render identifier audit gate status and finding details as one deterministic JSON object. The run shall reuse the configured maximum identifier length and preserve both summary values and each finding path, line, kind, name, and length so CI or review automation can consume touched-file gate details directly.

## REQ-AUTO-139: Identifier project gate detail JSON

The system shall render identifier audit gate status and finding details as one deterministic JSON object from explicit project roots. The workflow shall discover auditable Python files, scan every discovered file with the configured identifier length limit, evaluate pass/fail gate status, and preserve summary values plus each finding path, line, kind, name, and length so automation can consume whole-project gate details directly.

## REQ-AUTO-140: Identifier gate exit code

The system shall convert an identifier audit gate result into a deterministic process exit code. Passing gate results shall map to exit code 0, and failing gate results shall map to exit code 1 so CI wrappers can use gate status without parsing report text.

## REQ-AUTO-141: Identifier gate exit code run

The system shall scan an explicit set of Python files and return a deterministic identifier audit gate exit code. The runner shall reuse the configured maximum identifier length, map clean scans to exit code 0, and map scans with over-limit identifiers to exit code 1 so CI wrappers can evaluate touched-file naming status without parsing report text.

## REQ-AUTO-142: Identifier project gate exit code

The system shall return an identifier audit gate exit code from explicit project roots. The workflow shall discover auditable Python files, scan every discovered file with the configured identifier length limit, map clean projects to exit code 0, and map projects with over-limit identifiers to exit code 1 so CI wrappers can evaluate whole-project naming status from root paths.

## REQ-AUTO-143: Identifier gate log line

The system shall render identifier audit gate results as one deterministic log line. The output shall preserve the pass/fail status, hit count, file count, maximum identifier length, and maximum identifier name so CI logs can show naming gate status without multi-line Markdown or JSON parsing.

## REQ-AUTO-144: Identifier gate log line run

The system shall scan an explicit set of Python files and render the identifier audit gate result as one deterministic log line. The runner shall reuse the configured maximum identifier length and preserve pass/fail status plus summary values so CI logs can show touched-file naming status without multi-line Markdown or JSON parsing.

## REQ-AUTO-145: Identifier project gate log line

The system shall render an identifier audit gate result as one deterministic log line from explicit project roots. The workflow shall discover auditable Python files, scan every discovered file with the configured identifier length limit, evaluate the pass/fail naming gate, and preserve summary values so CI logs can show whole-project naming status without multi-line Markdown or JSON parsing.

## REQ-AUTO-146: Identifier CI gate run

The system shall scan an explicit set of Python files and return both an identifier audit gate exit code and one deterministic log line from the same gate result. The runner shall reuse the configured maximum identifier length, map clean scans to exit code 0, map scans with over-limit identifiers to exit code 1, and preserve the log summary so CI wrappers can evaluate and print touched-file naming status without duplicate scans.

## REQ-AUTO-147: Identifier project CI gate run

The system shall return both an identifier audit gate exit code and one deterministic log line from explicit project roots. The workflow shall discover auditable Python files, scan every discovered file with the configured identifier length limit, evaluate the pass/fail naming gate once, map clean projects to exit code 0, map projects with over-limit identifiers to exit code 1, and preserve the log summary so CI wrappers can evaluate and print whole-project naming status without duplicate scans.

## REQ-AUTO-148: Identifier CI gate JSON

The system shall render an identifier CI gate exit code and log line as deterministic JSON. The output shall preserve the numeric exit code and the printable one-line summary so CI wrappers can pass both machine-readable status and human-readable output to downstream automation without parsing shell text.

## REQ-AUTO-149: Identifier CI gate JSON run

The system shall scan an explicit set of Python files and render the identifier audit CI gate result as deterministic JSON. The runner shall reuse the configured maximum identifier length, evaluate pass/fail status once, map clean scans to exit code 0, map scans with over-limit identifiers to exit code 1, and preserve the one-line summary so CI wrappers can consume touched-file naming status as one machine-readable payload.

## REQ-AUTO-150: Identifier project CI gate JSON run

The system shall render an identifier audit CI gate result as deterministic JSON from explicit project roots. The workflow shall discover auditable Python files, scan every discovered file with the configured identifier length limit, evaluate pass/fail status once, map clean projects to exit code 0, map projects with over-limit identifiers to exit code 1, and preserve the one-line summary so CI wrappers can consume whole-project naming status as one machine-readable payload.

## REQ-AUTO-151: Identifier CI status JSON

The system shall render an identifier CI gate exit code and log line as deterministic JSON with an explicit boolean status. The output shall preserve the numeric exit code, preserve the printable one-line summary, and expose `ok` as true only when the exit code is 0 so CI wrappers can branch without decoding process conventions.

## REQ-AUTO-152: Identifier CI status JSON run

The system shall scan an explicit set of Python files and render the identifier audit CI gate result as deterministic JSON with an explicit boolean status. The runner shall reuse the configured maximum identifier length, evaluate pass/fail status once, map clean scans to exit code 0, map scans with over-limit identifiers to exit code 1, preserve the one-line summary, and expose `ok` as true only when the exit code is 0.

## REQ-AUTO-153: Identifier project CI status JSON run

The system shall render an identifier audit CI gate result as deterministic JSON with an explicit boolean status from explicit project roots. The workflow shall discover auditable Python files, scan every discovered file with the configured identifier length limit, evaluate pass/fail status once, map clean projects to exit code 0, map projects with over-limit identifiers to exit code 1, preserve the one-line summary, and expose `ok` as true only when the exit code is 0.

## REQ-AUTO-154: Identifier CI detail JSON run

The system shall scan an explicit set of Python files and render the identifier audit CI gate result plus finding details as deterministic JSON. The runner shall reuse the configured maximum identifier length, evaluate pass/fail status once, map clean scans to exit code 0, map scans with over-limit identifiers to exit code 1, preserve the one-line summary, expose `ok` as true only when the exit code is 0, and include every finding path, line, kind, name, and length for CI annotations.

## REQ-AUTO-155: Identifier project CI detail JSON run

The system shall render the identifier audit CI gate result plus finding details as deterministic JSON from explicit project roots. The workflow shall discover auditable Python files, scan every discovered file with the configured identifier length limit, evaluate pass/fail status once, map clean projects to exit code 0, map projects with over-limit identifiers to exit code 1, preserve the one-line summary, expose `ok` as true only when the exit code is 0, and include every finding path, line, kind, name, and length for whole-project CI annotations.

## REQ-AUTO-156: Identifier CI note run

The system shall scan an explicit set of Python files and render over-limit identifier findings as deterministic one-line notes for CI logs or annotation adapters. Each finding note shall preserve path, line, kind, name, and length. Clean scans shall render a stable clean marker so CI wrappers can emit readable output without parsing JSON.

## REQ-AUTO-157: Identifier project CI note run

The system shall render over-limit identifier findings as deterministic one-line notes from explicit project roots. The workflow shall discover auditable Python files, scan every discovered file with the configured identifier length limit, preserve path, line, kind, name, and length in each finding note, and render a stable clean marker when no findings exist so CI wrappers can emit whole-project naming notes without parsing JSON.

## REQ-AUTO-158: Identifier bounded CI notes

The system shall render identifier audit findings as deterministic one-line CI notes with a configurable positive maximum number of findings. The renderer shall preserve path, line, kind, name, and length for each emitted finding, append a stable omitted-count marker when findings are hidden, and render the same clean marker as the unbounded note renderer when no findings exist so CI logs can remain bounded without losing overflow visibility.

## REQ-AUTO-159: Identifier bounded CI note run

The system shall scan an explicit set of Python files and render a bounded top-N set of identifier audit findings as deterministic one-line CI notes. The runner shall reuse the configured maximum identifier length, preserve path, line, kind, name, and length for each emitted finding, append a stable omitted-count marker when findings are hidden, and render the clean marker when no findings exist so touched-file CI logs can stay bounded without losing overflow visibility.

## REQ-AUTO-160: Identifier project bounded CI note run

The system shall render a bounded top-N set of identifier audit findings as deterministic one-line CI notes from explicit project roots. The workflow shall discover auditable Python files, scan every discovered file with the configured identifier length limit, preserve path, line, kind, name, and length for each emitted finding, append a stable omitted-count marker when findings are hidden, and render the clean marker when no findings exist so whole-project CI logs can stay bounded without losing overflow visibility.

## REQ-AUTO-161: Identifier CI note Markdown

The system shall render already-produced identifier CI notes as deterministic Markdown. The renderer shall wrap the note text in a stable text code fence so review systems can publish naming audit notes without interpreting paths, colons, or omitted-count markers as Markdown syntax.

## REQ-AUTO-162: Identifier bounded CI note Markdown run

The system shall scan an explicit set of Python files, render a bounded top-N set of identifier audit findings as deterministic one-line CI notes, and wrap those notes in deterministic Markdown. The runner shall reuse the configured maximum identifier length and note limit so touched-file CI logs can publish bounded naming findings directly into Markdown-capable review surfaces.

## REQ-AUTO-163: Identifier project bounded CI note Markdown run

The system shall render a bounded top-N set of identifier audit findings as deterministic Markdown from explicit project roots. The workflow shall discover auditable Python files, scan every discovered file with the configured identifier length limit, preserve bounded note details, and wrap the result in a stable text code fence for whole-project review comments.

## REQ-AUTO-164: Identifier CI note Markdown section

The system shall render already-produced identifier CI notes as a deterministic Markdown section. The renderer shall add a stable heading above the fenced note body so review systems can publish multiple naming audit outputs with clear labels.

## REQ-AUTO-165: Identifier bounded CI note Markdown section run

The system shall scan an explicit set of Python files, render a bounded top-N set of identifier audit findings, and wrap the result in a titled Markdown section. The runner shall reuse the configured maximum identifier length, note limit, and caller-provided title so touched-file review comments can group naming findings.

## REQ-AUTO-166: Identifier project bounded CI note Markdown section run

The system shall render a bounded top-N set of identifier audit findings as a titled Markdown section from explicit project roots. The workflow shall discover auditable Python files, scan every discovered file with the configured identifier length limit, preserve bounded note details, and publish the result under a caller-provided heading for whole-project review comments.

## REQ-AUTO-167: Identifier naming audit operations guide

The system shall render a deterministic Markdown operations guide for the identifier naming audit. The guide shall document the audit purpose, default maximum identifier length, bounded review note limit, audited Python syntax, review workflow, and CI payload choices so contributors can use the naming audit without reading implementation code.

## REQ-AUTO-168: Identifier project operations document

The system shall render a deterministic project-level identifier naming audit operations document. The document shall combine the operations guide with the current bounded project naming findings section so reviewers can publish both usage guidance and live findings from the same project-root scan.

## REQ-AUTO-169: Identifier CI aggregate summary JSON

The system shall render identifier audit findings as deterministic aggregate JSON for CI dashboards. The output shall include total finding count, affected file count, maximum identifier length, longest identifier details, finding counts by Python syntax kind, and finding counts by file so CI systems can publish compact naming telemetry without embedding every finding.

## REQ-AUTO-170: Identifier CI aggregate summary run

The system shall scan an explicit set of Python files and render the identifier audit aggregate summary as deterministic JSON. The runner shall reuse the configured maximum identifier length, preserve deterministic file and kind ordering, and return an empty but schema-stable summary when no findings exist.

## REQ-AUTO-171: Identifier project CI aggregate summary run

The system shall render the identifier audit aggregate summary as deterministic JSON from explicit project roots. The workflow shall discover auditable Python files, scan every discovered file with the configured identifier length limit, preserve deterministic file and kind ordering, and return a compact whole-project summary for CI dashboards.

## REQ-AUTO-172: Seed sweep planning

The system shall expand a base experiment configuration into per-seed run configurations. Each run configuration shall be the base configuration plus its seed, duplicate seeds shall be dropped while keeping first-seen order, and the result shall be truncated to a maximum run count. A non-positive maximum or a non-integer seed shall be rejected with ValueError.
