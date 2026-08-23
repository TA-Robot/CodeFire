# Strategy Decision Register

Status: canonical

Owner: Product and Engineering Steward

Update trigger: evidence changes a decision status or the reconsideration trigger fires.

## Status Meaning

| Status | Meaning |
|---|---|
| `accepted` | Governs current development/strategy; product semantics still require their named product destination. |
| `experiment` | Leading hypothesis with a defined falsifier; not a promise or scheduled task by itself. |
| `pending-owner` | Cannot be decided responsibly without an accountable owner or external/legal/operational evidence. |
| `rejected` | Not on the roadmap; reconsider only on the stated trigger. |

## Accepted

| ID | Decision | Source candidates | Implementation / evidence | Review trigger |
|---|---|---|---|---|
| STR-001 | Keep model orchestration, prompt management, and agent scheduling outside the CodeFire product core. | DC-AI-002, P3D-011 | ADR-001 boundary, `../ai/`, `.codex/` | A product primitive cannot be expressed through stable automation/evidence contracts. |
| STR-002 | Use versioned typed JSON for Task, Context, and Review packets; Markdown is a human view. | DC-AI-003/011/012/040/043 | `../ai/schemas/`, contract checker | Typed packets fail EXP-AI-003 on total trusted-integration cost. |
| STR-003 | Route by stable role/risk policy and explicit runtime adapter; current model is GPT-5.6 Sol. | DC-AI-005/050/051/052 | model policy, matrix, `.codex/`, routing corpus | Model/runtime capability changes or live eval falsifies a tier. |
| STR-004 | Treat Ultra as maximum reasoning plus proactive delegation intent, not an effort string alone. | DC-AI-005/050/051 | compound adapters and validator | A future surface publishes a portable semantically equivalent capability. |
| STR-005 | Deterministic gates outrank model review and self-reported completion. | DC-AI-007/013/061 | AI gate, packet validation, engineering principles | A deterministic gate is shown unsound; fix the gate rather than lower authority silently. |
| STR-006 | Parallel writes require typed write sets, one writer, and serialized manager integration. | DC-AI-053/054 | operating protocol/task schema | Isolated transactional workspaces provide equivalent protection. |
| STR-007 | Bind task/context/patch/validation/review subjects by digest as each stage becomes automated. | DC-AI-004/014/041/062 | packet digests implemented; patch/review/certificate chain staged | Digest binding prevents interoperability or cannot identify the subject unambiguously. |
| STR-008 | Measure behavioral/evidence reproducibility and cost to trusted integration, not byte-identical patches or patch volume. | DC-AI-060/065, P3D-016 | eval roadmap/KPI | Metric cannot be collected reliably or predicts no user outcome. |
| STR-009 | Preserve the non-Git-compatible core; any Git bridge is an explicit external adapter with a different claim type. | DC-AI-006/022/023/025/072 | ADR-001, product strategy | External evidence proves a narrow core primitive is required; ADR review mandatory. |
| STR-010 | Unknown coverage stays unknown and never renders as clean or complete. | DC-AI-074, P3D-022 | product claim language; future conformance tests | None; this is a safety rule. |
| STR-011 | Do not store raw chain-of-thought as evidence; store decisions, rationale, commands, results, and digests. | DC-AI-045/064 | review/packet policy | A legal/security owner approves a specific alternative with retention controls. |
| STR-012 | Use document lifecycle states and a deletion manifest; Git history is the forensic archive for superseded tracked prose. | P3D-021 | consolidation plan, project docs checker | Findability or audit evidence worsens in the deletion rehearsal. |
| STR-013 | Keep CLI JSON/schema as the compatibility kernel; integrations are thin adapters. | DC-AI-031/070 | automation envelope and Stage 2 roadmap | Independent consumers cannot preserve required semantics. |
| STR-014 | Separate events, telemetry, diagnostics, and attestation by purpose instead of inventing one omnibus format. | DC-AI-034/035/063 | Stage 3 lossiness experiments | Standards cannot carry the required meaning even with bounded extensions. |
| STR-015 | Schedule work on a five-ahead milestone board. Only next-milestone blockers are active; other issues and review findings are parked. Codex orchestrates; Claude Opus 5 and Grok 4.6 implement and audit; a non-Codex auditor runs every 30 minutes. | stall from review-portfolio burn-down | `../milestone-operating-model.md`, `../milestones.md`, `../ai/orchestration-protocol.md` | Evidence that this control plane itself prevents a required safety gate; then change the board, not silently resume unbounded burn-down. |

## Experiment

| ID | Hypothesis | Required evidence | Falsifier / decision destination |
|---|---|---|---|
| STR-X01 | The external category is AI-change consistency and evidence protocol/control plane. | ICP interviews, claim comprehension, pilot demand | Users only value Git parity or do not face costly trace/evidence drift; narrow/pivot in product strategy. |
| STR-X02 | AI-heavy spec-driven/high-assurance teams are the first ICP. | At least twelve cross-profile interviews and qualified pilots | No recurring costly problem or adoption owner. |
| STR-X03 | Read-only MCP or SARIF/CI advisory is the first adoption wedge. | Competing prototypes, conformance, setup/value metrics | Neither creates standalone value; retain CLI/schema kernel only. |
| STR-X04 | Read-only Git snapshot/base-head assessment can create value without core Git semantics. | Three-repository four-week pilot | Identity ambiguity, false assurance, or core coupling dominates value. |
| STR-X05 | Public schemas, conformance vectors, and capability negotiation can form a de-facto standard kernel. | Clean-room consumer and independent integrations | Private implementation knowledge remains necessary. |
| STR-X06 | Command-scoped sealed-graph memoization fixes branch-list scale without semantic drift. | EXP-PERF-001 work counters and diagnostic equivalence | Validity is context-dependent or memory/diagnostic regressions dominate. |
| STR-X07 | One versioned committable predicate can align specs, core, certificate, and JSON. | Complete blocker decision table | Legitimate command-specific semantics require explicit typed variants. |
| STR-X08 | Branch-head publish can be the only visibility point for commit transactions. | Failpoint/crash/recovery matrix | A normal reader can observe an unclassifiable partial state. |
| STR-X09 | Extractor semantics need explicit version identity in resolution validity. | Extractor drift corpus | Existing content/Atom/policy hashes safely detect every semantic drift. |
| STR-X10 | Standards exports reduce integration cost without losing decision-critical semantics. | SARIF/in-toto/SPDX/OTel/CDEvents lossiness study | Most meaning becomes opaque custom data or unused extensions. |

## Pending Owner or External Decision

| ID | Decision needed | Blocker | Required destination |
|---|---|---|---|
| STR-P01 | Outbound license and aligned Cargo/Python declarations | rights/business/legal review; current MIT versus UNLICENSED conflict | LICENSE, package metadata |
| STR-P02 | First public SemVer and release channels | release identity/external-consumer audit | release policy and manifests |
| STR-P03 | DCO/contribution and founder-led backup governance | accountable maintainers and dry run | CONTRIBUTING, GOVERNANCE, MAINTAINERS |
| STR-P04 | Security intake and supported-version policy | primary/backup responder | SECURITY and release policy |
| STR-P05 | Public canonical documentation language | target-market and translation capacity | documentation governance |
| STR-P06 | Telemetry/privacy/evidence retention policy | data-flow and sensitive-evidence review | privacy/security docs and product spec |
| STR-P07 | Python fallback retirement | usage, parity, migration, incident evidence | compatibility strategy |

## Rejected for the Current Roadmap

| ID | Rejected direction | Reason | Reconsideration trigger |
|---|---|---|---|
| STR-R01 | Turn CodeFire core into an agent platform or model marketplace. | Dilutes consistency/evidence authority and couples product semantics to providers. | A new independent product proposal with separate boundary and ownership. |
| STR-R02 | Transparent Git compatibility or drop-in replacement. | Conflicts with current ADR and creates ambiguous authority/identity. | Never silently; only a superseding ADR with compatibility evidence. |
| STR-R03 | Always use Ultra for every task. | Wastes cost/latency and suppresses useful deterministic routing. | Live eval shows routed tiers are inferior on trusted integration. |
| STR-R04 | Let a model grader override failed deterministic gates. | Creates false completion and unauditable authority. | No current trigger; fix an unsound gate explicitly. |
| STR-R05 | Build hosted multi-tenant service as an incremental core feature. | Isolation, legal, operational, and funding boundary is absent. | Separate funded owner, threat model, legal review, and ADR. |
| STR-R06 | Enter formal standardization before independent adoption. | Risks freezing unstable semantics for signaling. | Stage 6 organization/integration/release thresholds pass. |
| STR-R07 | Select permanent monetization before pilot economics. | No measured willingness-to-pay/support load. | Qualified paid/sponsored pilot evidence. |
| STR-R08 | Promote unmeasured latency numbers from old plans to release gates. | No stable fixture, environment, or work-counter baseline supports them. | Reproducible benchmark corpus and accepted distributions. |
| STR-R09 | Preserve the historical eight-crate target as current architecture. | The implemented four-crate boundary superseded the sketch; crate count is not a goal. | A focused ADR demonstrates a new responsibility boundary. |
