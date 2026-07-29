# CodeFire Modernization Roadmap

Status: canonical active strategic roadmap

Owner: Engineering Manager

Update trigger: a stage exit gate passes, a P0 risk is discovered, or evidence changes stage order.

This roadmap replaces version-number task plans as the active ordering mechanism. It uses evidence gates rather than dates because the current repository has no coherent public release identity or reproducible Rust validation environment.

## Stage 0 — Repository and Development Control

Objective: make the project safe to change and cheap to understand.

Deliverables:

- management and strategy hubs with one current-state owner;
- three labeled exploration phases and a promotion register;
- document inventory, lifecycle, deletion manifest, and pruned superseded plans;
- dirty-tree packages separated by product, management, issue metadata, dogfood code, and dogfood docs;
- project-scoped GPT-5.6 Sol Medium/High/Ultra routing;
- typed task/context/review contracts, risk evals, context resolver, focused-test map, capability probe, and one local AI gate;
- Rust validation limitation recorded without treating a stale binary as source-test evidence.

Exit gate:

- active docs have no competing current-state/version owner;
- deletion candidates have destinations and inbound links are clean;
- AI development gate passes;
- every dirty file is assigned to a save package;
- Rust source validation is either rerun in a cargo-capable environment or remains an explicit blocker.

Current Stage 0 evidence: an isolated Rust 1.97.0 toolchain passed `cargo fmt --check`, `cargo test --workspace --locked` (231 tests), and `cargo clippy --workspace --all-targets --locked -- -D warnings` on 2026-07-11. This closes the current-worktree source-gate gap, but not durable toolchain pinning, clean-HEAD build provenance, or release reproducibility.

## Stage 1 — Correctness and Recovery Foundation

Objective: turn CodeFire's strongest claims into executable invariants.

Priority experiments:

1. complete committable-predicate decision table across core, CLI, certificate, JSON, and docs;
2. failpoint matrix for commit/publish/recovery and idempotent retry;
3. cross-runtime canonical JSON/full-digest/public-ID corpus plus artificial short-ID collision;
4. extractor semantic-version and stale-resolution matrix;
5. branch-list shared-history work counters and command-scoped validation memoization;
6. root/path/symlink and server-verification trust-boundary red team;
7. stale lock, pending transaction, corrupt object, and user-work recovery table.

The CFB-120..132 correctness batch and the CFR-161..167 safety/contract roots completed source gates, isolated Install, Algorithm Dogfood, and Retrospect by 2026-07-19; same-root CFB-134/135 and CFR-174/175 are also closed. CFR-163 gives local authority one verified snapshot, CFR-164 binds trust subjects, CFR-165 gives remote publication one durable protocol, CFR-166 establishes support/version, public base-schema, conformance, extension, pagination, and remote-Doctor contracts, and CFR-167 binds effective-policy provenance and typed ChangeCoverage without overstating span precision. The 60-review terminal audit remains frozen as `BATCH-2026-07-review-portfolio-completion`: CFB-133, admitted CFR-168 blocker CFB-136, and CFR-168..173 remain active, with CFR-168/CFB-136 in progress. Bounded-work roots preempt architecture/contract consumers; read-only adoption and product falsification experiments run last and must record an adopt/narrow/stop decision. No later-stage prototype may bypass these frozen safety gates.

Exit gate:

- every P0 invariant has a deterministic fixture and a failing seeded counterexample;
- no unexplained crash state or silent identity alias remains in supported paths;
- branch-list diagnostics are equivalent before/after the performance fix and scale within an accepted work budget;
- trusted versus isolated verification deployment is explicit.

## Stage 2 — Public Machine Contracts and Evals

Objective: make CodeFire independently consumable and AI-development quality measurable.

Deliverables:

- versioned public JSON Schema family and golden vectors;
- capability negotiation and explicit unknown/unsupported behavior;
- historical CFB/CFR task/review eval corpus with hidden outcomes;
- digest chain across task, context, patch, validation, review, and certificate subjects;
- deterministic conformance runner and independent minimal consumer;
- routing benchmark comparing Medium/High/Ultra and escalation, not just configuration validity.

Exit gate:

- an independent consumer passes the selected read-only profile without source-code access or private clarification;
- critical seeded defects are not escaped by routed development relative to the accepted Ultra baseline;
- schema additions and unsupported capabilities never turn DENY/UNKNOWN into ALLOW/CLEAN.

## Stage 3 — Adoption Wedge

Objective: prove standalone value in existing repositories without changing core authority.

Sequence:

1. compare read-only MCP and SARIF/CI advisory prototypes;
2. select at most one first wedge;
3. run snapshot/base-head Git assessment with explicit claim type;
4. add provenance/telemetry exports only where the lossiness study supports them;
5. consider LSP after the diagnostic model is stable.

Exit gate:

- no adapter can mutate outside its declared profile;
- at least two independent clients/repositories pass conformance;
- advisory/native evidence cannot be confused;
- pilot value exceeds setup and maintenance cost.

## Stage 4 — Release and Operational Trust

Objective: become responsibly distributable rather than merely buildable.

Deliverables:

- rights/license decision and aligned Rust/Python metadata;
- first public SemVer and independent version-axis policy;
- SECURITY, CONTRIBUTING, GOVERNANCE, MAINTAINERS, support, and release policies;
- sensitive-evidence red team and privacy/data-flow inventory;
- backup/restore and credential-compromise drills;
- clean signed release rehearsal with checksums, SBOM, provenance, independent verification, and rollback;
- two-person release and security-response capability.

Exit gate:

- every public artifact has known rights and source identity;
- a backup operator can publish, verify, roll back, and respond using documentation only;
- no public beta label while cargo/source gates or release identity remain unresolved.

## Stage 5 — Product and Economics Proof

Objective: validate the problem, claim, workflow, and sustainable value.

Deliverables:

- problem interviews across the candidate ICPs;
- claims-comprehension and first-seal tests;
- Fire/evidence actionability audit;
- three-repository bridge/advisory pilot and four-week team pilot;
- trusted-integration cost study and support-load evidence.

Exit gate:

- independent users understand what CodeFire proves;
- at least one segment has repeatable value and an adoption owner;
- the next product scope follows measured friction, not internal feature enthusiasm.

## Stage 6 — Ecosystem and De-facto Standardization

Objective: earn interoperability before pursuing formal standardization.

Deliverables:

- layered specs with maturity labels;
- reference adapters and conformance profiles;
- extension namespace/critical-extension rules;
- multiple organizations and independent implementations;
- public change, IPR, trademark, governance, and compatibility processes.

Exit gate:

- three organizations use stable contracts;
- two independent integrations or one independent implementation pass conformance;
- two compatible release cycles complete;
- funded maintainer capacity exists.

Until then, keep a de-facto open specification and do not enter a standards body for signaling alone.

## Ordering Rules

- A later-stage prototype may run read-only to gather evidence, but it cannot bypass an earlier safety/release gate.
- P0 security, identity, recovery, and user-work risks preempt adoption work.
- Do not run overlapping product write lanes; the organization checkpoint is clean, but product mutation remains serialized by lane ownership and integration order.
- A failed experiment can narrow or stop a stage; it is not a reason to lower the threshold after seeing results.
