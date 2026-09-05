# CodeFire Modernization Roadmap

Status: canonical active strategic roadmap

Owner: Engineering Manager

Update trigger: a stage exit gate passes, a P0 risk is discovered, or evidence changes stage order.

This roadmap replaces version-number task plans as the active ordering mechanism. It uses evidence gates rather than dates because the current repository has no coherent public release identity or reproducible Rust validation environment.

## Recovery ordering adopted 2026-09-05

The user accepted the [recovery assessment](../exploration/recovery-2026-09-05.md). The execution board now
orders: source/runtime baseline -> usable first-seal/recovery path -> fast safe change -> repeat use -> bounded
distribution. The stages below remain capability gates, not a competing task queue.

The previous n=1 AI-assisted study ended `narrow`; it did not meet Stage 5. Ecosystem work is parked until
repeat use yields a real interoperability need. Past experiment outcomes remain immutable evidence; they do
not prohibit a newly scoped improvement or justify automatically advancing to the next strategic stage.
Grok is primary, Fable scarce, and metered billing forbidden. The development control plane must produce
observable product improvements rather than merely closing review/document packets.

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

The CFB-120..132 correctness batch and the CFR-161..167 safety/contract roots completed source gates, isolated Install, Algorithm Dogfood, and Retrospect by 2026-07-19; same-root CFB-134/135 and CFR-174/175 are also closed. Live execution moved on 2026-08-21 to the five-ahead board `../milestones.md` (STR-015). Control restore, the local sealed loop, the agent-consumable local contract, the remote happy path, the adoption wedge, the bounded internal release rehearsal, and the bounded first-seal product-value study have exited. That historical board advanced to Stage 6; the 2026-09-05 recovery decision now parks that sequence. Open issue status stays in the ledgers; the board alone controls which slice may run. This roadmap remains a stage/gate document, not a second task board.

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

The completed `MS-release-operational-trust` milestone exercised only a bounded internal-only rights decision and one independently verified release-and-rollback path. Missing outbound rights, SBOM/signing, external publication, and the remaining deliverables above stay here as Stage 4 work.

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

The completed `MS-product-value-proof` milestone drew a **bounded** slice of this stage: one candidate segment, one claims-comprehension/first-seal study, measured setup and friction, and an ordered adopt / narrow / stop decision. Its terminal decision was literal `narrow` — one clean-room AI-assisted maintainer/operator scored 7/7 with task value 3/3 and sealed the handed task, but the handed public docs pack did not enumerate every JSON identifier the envelopes return, so the admitted consequence was to narrow the public claim rather than retry or fix the product. That study is n=1 and machine-timed; it is not human adoption and not market validation. Problem interviews across the candidate ICPs, the Fire/evidence actionability audit, a three-repository bridge/advisory pilot, a four-week team pilot, the trusted-integration cost study, and support-load evidence all remain Stage 5 work, so this stage's exit gate is not met.

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

Board slot M1 `MS-ecosystem-conformance-seed` draws a bounded slice of this stage: one maturity-labelled public profile, its extension rules, and one independently authored adapter/client passing a bounded conformance corpus. No ecosystem work has run yet. Board slot M2 `MS-ecosystem-multi-implementation-proof` is the next bounded slice: a second integration from a distinct organization consumes one compatibility-preserving profile update and passes the corpus. Board slot M3 `MS-ecosystem-third-organization-proof` is the next bounded slice: a third distinct organization consumes the stable profile without a compatibility exception and passes the corpus. Board slot M4 `MS-ecosystem-compatible-release-cycles` is the next bounded slice: two successive compatibility-preserving profile releases pass the corpus without an emergency exception. Board slot M5 `MS-ecosystem-funded-maintainer-capacity` is the next bounded slice, derived from this stage's remaining funded-maintainer capacity gate: a named funded maintainer rotation independently handles one compatibility review and one bounded conformance release rehearsal. Formal governance/IPR/trademark process work and standards-body activity remain roadmap work.

## Ordering Rules

- A later-stage prototype may run read-only to gather evidence, but it cannot bypass an earlier safety/release gate.
- P0 security, identity, recovery, and user-work risks preempt adoption work.
- Do not run overlapping product write lanes; the organization checkpoint is clean, but product mutation remains serialized by lane ownership and integration order.
- A failed experiment can narrow or stop a stage; it is not a reason to lower the threshold after seeing results.
