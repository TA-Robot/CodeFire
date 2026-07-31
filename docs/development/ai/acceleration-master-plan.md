# AI Development Acceleration Master Plan

This plan synthesizes the exploration notes and defines the implementation path for making CodeFire development maximally AI-accelerated while keeping CodeFire product scope clean.

## North Star

CodeFire should be developed with an external AI development operating system:

- context is compressed into packs;
- work is delegated as task packets;
- GPT-5.6 Sol Medium/High/Ultra effort is routed by risk and role;
- parallel lanes are safe by construction;
- reviewers challenge invariants;
- validation evidence is standardized;
- friction becomes new automation or issue work.

## Non-Goals

- Do not add agent orchestration to CodeFire product code.
- Do not couple CodeFire product semantics to a development model. Keep the explicit current model in a versioned runtime policy and project Codex configuration.
- Do not optimize for patch volume over trusted integration.
- Do not start discretionary product development until the organization checkpoint is clean and the frozen Stage 1 correctness batch is closed.

## Decision Rationale

The broad exploration phase has been condensed into these operating decisions:

| Source view | Accepted decision |
|---|---|
| Computation models | Treat task packets as typed work specs, context manifests as inputs, review packets as assertions, and validation tables as traces. |
| Distributed systems | Use lane ownership, manager-owned global files, and serialized integration for shared state. |
| Program analysis | Turn correctness claims into explicit invariants, focused tests, stale-state checks, and JSON contract checks. |
| Information retrieval | Prefer small context packs over large repository reads. |
| Security and reliability | Never trade speed for weaker object identity, sealed commit validation, branch atomicity, remote policy, JSON envelopes, or user-change safety. |
| Economics | Optimize for lowest total cost to trusted integration, not fastest first patch or highest patch volume. |
| Platform boundaries | Keep AI orchestration, model routing, prompt libraries, scheduling, and marketplace behavior outside CodeFire product code. |
| Model routing | Use Medium for deterministic work, High for bounded implementation/synthesis, and Ultra for architecture, adversarial review, security, deletion, and completion audits. |

## Stage 0: Stabilize Current Worktree — Complete 2026-07-12

Deliverables:

- `worktree-cleanup-plan.md` used as source of truth.
- Management docs saved separately.
- Dirty tree classifier packet executed through `docs/development/tools/classify_worktree.py`.
- Current-state consistency confirmed.

Exit criteria:

- Management docs can be reviewed independently.
- Unknown dirty files list is empty or explicitly assigned.
- The mixed tree is saved as purpose-specific management, AI, strategy, cleanup, issue, product, dogfood implementation, and dogfood documentation commits.

## Stage 1: Make AI Packets Operational — Complete 2026-07-12

Deliverables:

- `context-packs.json`.
- `operating-protocol.md`.
- Ready packets in `task-packets/`.
- Role routing in `operating-protocol.md` and `role-routing-matrix.json`.
- GPT-5.6 Sol runtime routing in `model-routing-policy.md` and `.codex/`.

Exit criteria:

- A manager can delegate the branch-list timeout task without writing a new prompt from scratch.
- A reviewer can review the result with a standard checklist.
- The local runtime checker proves repository routing/config consistency and detects a missing GPT-5.6 Sol Ultra catalog.

## Stage 2: Add Checkers and Generators — Complete 2026-07-12

Deliverables:

- context pack validator in `docs/development/tools/check_ai_acceleration_docs.py`;
- initial stale current-state phrase checker in `docs/development/tools/check_ai_acceleration_docs.py`;
- dirty tree classifier in `docs/development/tools/classify_worktree.py`;
- validation availability probe;
- focused test map.

Exit criteria:

- A validation runner can report readiness without knowing project history.
- stale current-state contradictions are detected automatically.

## Stage 3: Parallel Lane Execution — Complete 2026-07-12

Deliverables:

- lane plans for management docs, CodeFire product fixes, algorithm implementation, algorithm docs;
- manager-owned integration protocol;
- reviewer lane packet.

Exit criteria:

- At least two lanes can run without overlapping write sets.
- global state files are manager-owned.
- The organization checkpoint demonstrates separate management, AI, strategy, cleanup, issue, product, dogfood implementation, and dogfood documentation commits.

## Stage 4: Dogfood and Learning Loop — Complete 2026-07-12

Deliverables:

- dogfood smoke runner;
- CFB intake/promotion packet;
- failure cookbook;
- validation report template backed by the typed review schema.

Exit criteria:

- Dogfood friction can be converted into a task packet within minutes.
- negative results are captured instead of lost.

## Stage 5: Trusted Throughput Operating System — Active 2026-07-30

The first live CFR-168 portfolio demonstrated that safe parallel candidate
generation is no longer the limiting factor. The measured bottleneck is exact
review yield, review-derived rework, composition latency, validation reuse,
and restart recovery.

The operating decision and baseline are frozen in
[`swarm-results/cfr168-trusted-throughput-audit-20260730/audit.md`](swarm-results/cfr168-trusted-throughput-audit-20260730/audit.md).
The compatible infrastructure design is frozen in
[`swarm-results/dev-swarm-throughput-design-20260730/design.md`](swarm-results/dev-swarm-throughput-design-20260730/design.md).
Until that foundation implementation is reviewable, implemented/manual/future
capabilities and exact operator commands are frozen in
[`stage5-manual-control-runbook.md`](stage5-manual-control-runbook.md).

The current scheduler does not enforce proposed Stage 5 manifest fields and
must not receive them in a `dev-swarm.v1` manifest. The implementation starts
with a fail-closed `dev-swarm.v2` boundary so an older binary rejects, rather
than silently ignores, resource, progress, lineage, continuation, review,
gate, or shadow-train controls.

Deliverables:

- pre-implementation Fugu `READY/BLOCK` counterexample review;
- WIP limited by independent review capacity and conflict domain;
- exact candidate-ready events with automatic independent-review rollover;
- review-derived continuation of the exact original implementer session;
- manager-owned product-only shadow integration trains;
- content-addressed deterministic focused gates and generated evidence cards;
- machine-wide provider/resource/heavy leases and critical-path scheduling;
- restart-safe invocation state, progress leases, adopt/resume, and terminal
  cache/log lifecycle;
- event-derived review yield, queue, rework, integration, resource, and cost
  KGI/KPI views.
- post-CFR behavior-preserving decomposition of the hot dispatcher/owner/test
  files into actual conflict domains.

Exit criteria:

- first-pass exact acceptance is at least 60% for the next five comparable
  product packets without a quality regression;
- review-ready queue p95 is at most 30 minutes;
- accepted-delta-to-composed-focused-gate p95 is at most two hours;
- no broad gate runs more than once on one stable composition revision;
- provider/host restart loses no source/evidence and at most 45 minutes of
  uncheckpointed work;
- the foundation implementation has usable Git provenance, black-box tests,
  and the existing manual rules can be retired without weakening review or
  manager authority.

## Completed Bootstrap Order

1. Ran the dirty-tree classifier and saved eight purpose-specific packages.
2. Restored the Rust source gate and produced clean-HEAD build provenance.
3. Added issue/count/source/document consistency checks and a capability probe.
4. Added focused validation routing, typed packets/reviews, and live tier-routing evidence.
5. Split the historical burn-down archive from the active plan.
6. Added bounded dogfood smoke, CFB intake, failure handling, and validation-report assets.

The branch-list timeout packet remains a bounded product investigation, not an unmet bootstrap dependency.

## Execution Backlog

Only keep items that remove repeated human/AI overhead, can be checked cheaply, avoid product-surface complexity, and have a clear owner artifact.

| ID | Item | Status / output |
|---|---|---|
| AIA-001 | Dirty tree package splitter | Seeded as `../tools/classify_worktree.py`. |
| AIA-002 | Current-state and issue-count consistency checker | Implemented across `../tools/check_ai_acceleration_docs.py` and `../tools/check_project_docs.py`, including active-state phrases and CFB/CFR detail-ledger reconciliation. |
| AIA-003 | Validation availability probe | Implemented in `../tools/probe_validation_capabilities.py`; presence, revalidated clean-HEAD provenance, and runnable source gates are separate. |
| AIA-004 | Branch-list timeout investigation | Packet exists at `task-packets/branch-list-timeout.md`. |
| AIA-020 | Focused test map | Implemented in `focused-test-map.v1.json` and `../tools/suggest_validation.py`. |
| AIA-021 | Rust validation/provenance wrapper | Source-bound release build wrapper implemented in `../tools/build_source_bound_rust.py`; it requires tracked and untracked Cargo/crates/`.cargo`/toolchain inputs to match HEAD, rejects changes across the build, and emits source/toolchain/build-log/binary digests. A higher-level fmt/focused/clippy/workspace orchestrator remains optional after repeated need. |
| AIA-022 | Broader docs consistency runner | Implemented for local links, CFB/CFR ledgers, removed-plan references, strategy metadata, inventory, and source-symbol freshness in `../tools/check_project_docs.py` and `../tools/check_source_docs.py`. |
| AIA-023 | Dogfood smoke runner | Implemented in `../tools/run_dogfood_smoke.py`; default mode runs only read-only `status`, while `scan`/`verify` require explicit `--allow-mutation`. JSON schema, command identity, exit-code agreement, containment, timeout, and fail-fast behavior are checked. |
| AIA-024 | GPT-5.6 Sol role routing | Implemented in `.codex/`, `model-routing-policy.md`, `role-routing-matrix.json`, and `validate_ai_runtime.py`. The runner selects a registered major/minor CLI adapter and currently validates local 0.146.0 strict-config/catalog capability; a dated 0.144.1 account-level Medium/High/Ultra smoke remains recorded in `evals/live-routing-smoke-2026-07-11.v1.json`. Scored quality comparison remains AIA-026. |
| AIA-025 | Typed task/context/review schemas | Implemented in `schemas/`, with schema/semantic checks, recomputed context digests, task-bound review validation, dirty/base execution preflight, typed branch-list packet, and digest-aware runner. Markdown generation/round-trip lint remains. |
| AIA-026 | AI development eval corpus | Routing cases cover all tiers and mandatory-Ultra reasons. Eight historical CFB/CFR defect/completion cases, withheld evaluator oracles, deterministic input preparation, and corpus checks are implemented; scored live model cohorts remain Stage 2. |
| AIA-027 | Integration/conformance wedge | Pending; evaluate read-only MCP, SARIF, and advisory CI before any core product-boundary change. |
| AIA-028 | Atomic second-agent invocation state and adopt/resume | Design ready in `swarm-results/dev-swarm-throughput-design-20260730/design.md`; blocked on restoration of the foundation repository Git identity. |
| AIA-029 | Global execution profiles and adaptive admission | Design ready: shared low-memory Rust profile, per-candidate reservations, host/cgroup pressure floors, adaptive provider windows, active/private target cleanup rules, and critical-path queue semantics are specified. Fixed total/heavy counts are explicitly excluded. |
| AIA-030 | Exact review rollover and repair continuation | Design ready; candidate event, reviewer independence, immutable subject, exact-session continuation, and fail-closed identity tests are specified. |
| AIA-031 | Content-addressed validation and generated evidence cards | Design ready; cache identity, non-cacheable gate classes, typed output, and worker-claim override prevention are specified. |
| AIA-032 | Manager shadow integration trains | Design ready; product-only delta admission, per-delta focused gates, stable-head broad gate, and conflict rereview rules are specified. |
| AIA-033 | Portfolio event/KGI telemetry | Active manual baseline in `swarm-results/cfr168-trusted-throughput-audit-20260730/audit.md`; automatic provider/gate/review/manager/resource event capture remains blocked with the foundation implementation. |
| AIA-034 | Counterexample-bound compact context | Failure signatures are reusable in `failure-cookbook.md`; automatic preflight resolution and minimal context generation remain pending. |
| AIA-035 | Hot-file conflict-domain decomposition | After accepted CFR-168 composition, extract stable request/session/authority/response/cleanup/publication primitives and owner-layered tests from the 33k-line Python reference, Rust `http.rs`, `main.rs`, and `remote.rs`; preserve exact behavior and measure write-set overlap, conflict time, context bytes, and focused-suite size. |
| AIA-036 | Manual Stage 5 capability/control board | Implemented in `stage5-manual-control-runbook.md`: current versus manual versus future capability matrix, adaptive portfolio admission, candidate card, prepared review rollover, exact continuation, gate index, shadow train, event/KPI record, and resource floors. |
| AIA-037 | Fail-closed Stage 5 manifest boundary | Design corrected to use `dev-swarm.v2`. The project-side AI checker now rejects known Stage 5 fields anywhere in a `dev-swarm.v1` batch manifest, with negative tests. Full recursive unknown-field rejection in both versions and old-binary v2 rejection remain foundation work blocked on usable Git provenance. |
| AIA-038 | Manager-recomputed candidate and report identity | Implemented in `../tools/capture_candidate_identity.py`: one read-only command binds exact base/merge-base/parents/head/tree, additive and merge commit lists, canonical binary patch digest, name-status/numstat/blob OID/byte/SHA-256 identities, checkout and clean state, manifest/ticket digests, role, and write-set containment/completeness. Unsafe or moving subjects emit `BLOCK` with exit 2. Validation/session evidence and verdict routing remain explicit manager work rather than worker claims. A review task commit must carry the manager-vetted helper blob and digest when the candidate baseline predates it. |
| AIA-039 | Compact exact-review handoff | Implemented in `validation-report-template.md`: immutable subject, required dispositions, owner/effect/fence rows, mutation matrix, content-addressed validation, integration overlap, unavailable evidence, and one machine-stable overall verdict line. It reduces repeated prose without weakening production reachability or evidence requirements. |
| AIA-040 | Monotonic gate evidence | Implemented in `../tools/run_timed_gate.py`: shell-free argv execution, process-group timeout, monotonic duration, complete log bytes/SHA-256, atomic JSON result, and explicit wall-clock drift. This prevents host clock jumps from corrupting queue/gate/KPI decisions or forcing unnecessary reruns. |
| AIA-041 | Count-free swarm admission | Implemented in `../tools/check_swarm_admission.py`: each candidate is evaluated from a class reservation, projected effective memory, swap/disk/PSI/PID pressure, adaptive provider window/backoff, and explicit conflict ownership. It has no total-session input; bootstrap reservations are replaced by measured comparable-lane p95 times 1.5. |
| AIA-042 | Provider-ready retry and adoption | Required foundation repair: inherit provider readiness without logging secrets, distinguish typed preflight failure from worker failure, retry a zero-commit clean launch, and adopt an exact manually resumed session. Storage exposed the gap when `dev-swarm.v1` lost `SAKANA_API_KEY`, terminally failed in five seconds, and could not own the successful exact-session restart. Blocked on usable foundation Git provenance. |
| AIA-043 | Verdict-to-rollover latency | Active measurement/control: capture decision-settled to immutable-report-commit separately from review execution, require the compact exact-review form and one terminal verdict, and bind a prepared repair continuation in the same manager interval. This targets the current period where a decisive rejection exists but downstream repair cannot safely start. |

The AIA-021 provenance is intentionally local evidence. It does not claim a trusted/reproducible builder or dependency supply-chain attestation.

Deferred ideas stay out of separate backlog files until repeated friction proves they are worth implementing: issue-to-packet generation, review-packet generation, handoff normalization, module-boundary diff checks, JSON envelope contract checks, next-action path checks, invariant checklists, source-map freshness checks, command ownership generators, error taxonomy, and failure cookbook.

Current recommended next items:

1. Restore the actual `/workspace` foundation Git metadata or bind its remote
   and intended base; do not initialize an unrelated replacement history.
2. Use AIA-036 for the active portfolio; keep unsupported future fields out
   of every v1 manifest and treat scheduler pass as process evidence only.
3. Implement AIA-037, AIA-028, and AIA-029 first so every later scheduler
   change is version-safe, restart-safe, and resource-safe.
4. Implement AIA-030 through AIA-032 as independently reviewable black-box
   partitions.
5. Run the next five comparable CFR-168 packets under the Stage 5 WIP,
   preflight, review, and shadow-composition controls.
6. Use AIA-038 for every completed repair so identity collection and
   candidate-card rendering leave the candidate-to-review critical path.
7. Start AIA-035 only after the accepted Python/Rust composition trees are
   frozen so move-only work does not invalidate active exact reviews.
8. Promote AIA-033 KGI/KPI changes only from complete event evidence; keep
   unavailable queue/cost fields explicit.

## Success Metrics

- Time from issue to task packet under 10 minutes.
- Time from patch to review packet under 5 minutes.
- No stale current-state contradictions in active docs.
- No product/dogfood mixed commits.
- Every AI implementation handoff includes validation evidence.
- Reviewers can identify invariants without rereading entire repo.
- Every active batch records KGI/KPI and incident evidence under `flow-metrics/`; throughput is
  evaluated at trusted integration rather than first patch or agent count.

## Governance

This plan is governed by:

- `docs/development/project-management.md`
- `docs/development/ai/README.md`
- `docs/development/worktree-cleanup-plan.md`

Update `history.md` whenever a stage completes or the plan changes materially.
