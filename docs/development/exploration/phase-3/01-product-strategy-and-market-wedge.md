# Product Strategy and Market Wedge

Status: exploration

Last reviewed: 2026-07-11

## Decision Question

What narrow, valuable problem should CodeFire solve first so that teams can prove the value of sealed, policy-consistent history without being asked to replace their entire development culture before they trust the product?

## Observed Constraints

- observed: CodeFire's durable differentiation is consistent-only sealed history backed by Trace Links, Fires, Evidence, verification, and certificates.
- documented: CodeFire is not Git-compatible, is not a Git layer, and does not provide checkout, staging, stash, WIP commits, partial commits, or force upload.
- documented: CodeFire does not decide whether a specification or implementation is true; it records whether configured consistency obligations were resolved.
- documented: external AI and CI systems are first-class consumers through JSON, dry-run, exit codes, next actions, and sealed objects.
- observed: current validation is deep but concentrated in one algorithm-research dogfood project.
- inferred: asking a team to abandon Git before demonstrating value creates a larger adoption barrier than adding a CI or review tool.
- inferred: describing CodeFire only as a VCS invites comparison on Git feature parity, while its strongest benefit is assurance over cross-artifact consistency.

## Jobs to Be Done

Candidate primary jobs:

1. When AI changes code quickly, show which requirements, designs, tests, APIs, schemas, ADRs, and runbooks require review.
2. Before a change becomes an accepted release state, prove that every configured blocking obligation was resolved with fresh evidence.
3. Let reviewers understand why a change was accepted without reconstructing intent from a file diff and chat history.
4. Let release owners accept only sealed states that satisfy a known policy and verification contract.
5. Let automation consume failures and next actions without scraping human text.

Candidate non-jobs:

- general-purpose source hosting;
- arbitrary Git workflow emulation;
- autonomous coding-agent orchestration;
- proof that software is correct or safe in an absolute sense;
- complete semantic merge or automatic approval;
- public multi-tenant developer platform in the current implementation stage.

## Ideal Initial Customer Profiles

### ICP-A: AI-heavy specification-driven team

Characteristics:

- AI produces substantial code and documentation changes;
- requirements, designs, tests, API/schema, or operations docs must move together;
- review bandwidth, not typing speed, is the bottleneck;
- the team accepts explicit policy and evidence in exchange for lower review uncertainty.

### ICP-B: High-assurance or regulated internal tool team

Characteristics:

- release evidence, traceability, or audit reconstruction is expensive;
- a missed cross-artifact update has material operational or compliance cost;
- local-first or self-hosted operation is valued;
- procurement will require license, SBOM, support, vulnerability, and lifecycle answers.

### ICP-C: Research and experiment platform

Characteristics:

- specifications, experiment records, code, metrics, and conclusions must stay linked;
- reproducibility and negative evidence matter;
- long-running AI-assisted work produces documentation drift;
- CodeFire's current dogfood provides relevant but not independent evidence.

### Anti-Personas for the First Release

- individual developers who primarily want a faster Git command surface;
- teams that depend on rebase, partial commits, and informal WIP history as core coordination tools;
- organizations requiring a turnkey public SaaS or enterprise identity integration immediately;
- repositories with no stable artifacts or policy relationships to trace;
- users expecting CodeFire to certify semantic truth automatically.

## Category Options

### Option A: Full Git Replacement

Claim: CodeFire is the next version-control system for all AI development.

Benefits:

- clearest commitment to the product philosophy;
- all workflow invariants remain under CodeFire control;
- no ambiguity over which history is authoritative.

Costs:

- highest migration and education cost;
- competes on decades of Git ecosystem features;
- users must accept missing WIP and collaboration habits before observing value;
- package, hosting, IDE, review, and CI ecosystem gaps become immediate blockers.

### Option B: Net-New High-Assurance VCS

Claim: CodeFire is the primary VCS for new AI-heavy projects where traceability is designed from the start.

Benefits:

- preserves the complete consistency model;
- avoids legacy-history conversion initially;
- selects teams with a strong reason to change workflow.

Costs:

- small initial market;
- success depends on new-project opportunities;
- still requires a credible review, backup, remote, and distribution story.

### Option C: External Adoption Bridge

Claim: CodeFire core remains a sealed-history VCS, while a separate adapter lets an existing Git/CI workflow evaluate CodeFire scans, review packs, or certificates before migration.

Benefits:

- value can be tested before full replacement;
- core does not gain Git semantics;
- creates an evidence path for migration decisions;
- external integrations can evolve without changing object invariants.

Costs:

- dual-state confusion and duplicated workflow are possible;
- the bridge may become the product users prefer;
- authoritative-history boundaries must be explicit;
- adapter correctness and support become additional work.

### Option D: Open Specification and Certificate Layer

Claim: CodeFire's main asset is a vendor-neutral consistency/evidence specification, with the current CLI as one reference implementation.

Benefits:

- maximizes ecosystem interoperability;
- lowers fear of implementation lock-in;
- aligns with eventual standardization.

Costs:

- premature specification stability can freeze weak semantics;
- difficult to monetize or support before adoption;
- independent implementation is expensive and currently unproven.

## Recommended Decision Candidate

decision-candidate: Use Option B as the core product position and Option C as the adoption wedge.

Proposed two-layer statement:

- Product category: consistency-first, specification-driven sealed-history VCS.
- Initial market promise: a trust and evidence control plane for AI-produced changes in high-assurance, specification-driven projects.

The external bridge must live outside CodeFire core and must not silently claim that Git commits are CodeFire-sealed commits. It may import a snapshot, run read-only assessment, produce a review pack, or verify a published certificate. Transparent coexistence or dual-authoritative history requires an ADR because it conflicts with the existing product boundary.

## Product Claim Ladder

Claims should only advance when evidence exists.

| Level | Allowed claim | Required evidence |
|---|---|---|
| 0 | CodeFire records configured cross-artifact obligations | unit/integration tests and schema documentation |
| 1 | CodeFire detects selected classes of drift in supported repositories | labeled dogfood and benchmark corpus results |
| 2 | CodeFire reduces review uncertainty or escaped inconsistency for a target team | controlled pilot baseline and follow-up |
| 3 | CodeFire is production-ready for a documented deployment profile | release, recovery, security, support, and independent pilot evidence |
| 4 | CodeFire is an ecosystem standard | multiple independent consumers/implementations and governed conformance suite |

Never collapse policy-consistent into correct, secure, compliant, or complete without a separately validated assurance claim.

## Trade-Offs

- Narrow ICP improves evidence quality but slows headline adoption.
- External bridging reduces migration risk but can weaken the urgency to adopt CodeFire-native history.
- Strict invariants create differentiated trust but increase workflow tax.
- Local-first operation improves control and privacy but reduces network-effect distribution.
- Open specifications build trust but constrain later schema changes.

## Falsification and Pivot Conditions

The recommended position is weakened if any of the following persists after onboarding improvements:

1. Target teams cannot identify a costly inconsistency problem CodeFire would have prevented.
2. Fewer than three of ten qualified pilots reach a second sealed change.
3. Eight-week retention in the target segment remains below 30 percent.
4. Median CodeFire-specific workflow tax remains above 20 percent after the second week.
5. Fire actionability remains below 60 percent or boilerplate resolutions exceed 25 percent.
6. Escaped cross-artifact inconsistency is not lower than the team's baseline.
7. Most retained users only want the external bridge and reject sealed history.

Possible pivots:

- narrow supported artifacts and industries;
- become a CI consistency/evidence verifier rather than a primary VCS;
- focus on the certificate/specification and reference implementation;
- suspend broad adoption claims and remain an internal high-assurance research tool.

## KPI Definitions

| KPI | Definition | Initial experiment threshold, not a promise |
|---|---|---|
| Time to First Seal | elapsed time from successful install to first non-demo sealed commit | median at most 15 minutes, p90 at most 30 minutes |
| Activation | qualified users producing a seal within 24 hours | at least 60 percent |
| Eight-Week Retention | activated teams producing or verifying a seal in week eight | at least 40 percent |
| Fire Actionability Rate | sampled fires that caused a meaningful review, update, or documented decision | at least 70 percent |
| Evidence Boilerplate Rate | sampled resolutions judged to contain no decision-specific information | at most 15 percent |
| Workflow Tax | CodeFire-specific active minutes divided by total trusted-change minutes | at most 15 percent after onboarding |
| Escape Rate | post-seal cross-artifact inconsistencies per 100 sealed changes | at least 30 percent lower than baseline |
| Review Lead Time | time from review-ready change to accepted sealed state | no worse than baseline; target 20 percent reduction |
| Independent Adoption | repositories operated by teams outside the founding maintainer | three before beta candidate |

## Experiments

### P3-PROD-001: Problem Interviews

- Recruit at least four people from each ICP.
- Ask for the last costly specification/design/code/test drift incident.
- Record current detection method, cost, buyer, operator, and workflow constraints.
- Do not pitch feature names until the incident is understood.
- Success: at least half can name a recurring problem and agree to a repository-based pilot.

### P3-PROD-002: First-Seal Usability Test

- Give a clean machine or container, one small supported project, and public docs only.
- Observe install, init, artifact configuration, scan, extinguish, verify, and commit.
- Success: median completion within the Time to First Seal threshold without maintainer intervention.

### P3-PROD-003: External Bridge Pilot

- Run read-only snapshot assessment or certificate verification beside three existing Git repositories for four weeks.
- Clearly label Git and CodeFire authority.
- Measure value, duplicated work, false confidence, and migration willingness.
- Success: at least two teams request a deeper CodeFire-native pilot.

### P3-PROD-004: Fire Value Audit

- Blind-label at least 200 fires across multiple repositories as actionable, redundant, false-positive, or unverifiable.
- Sample rationales for boilerplate and stale basis.
- Success: actionability and boilerplate thresholds are met by artifact kind, not only in aggregate.

## Promotion Targets

- Accepted category and ICP: docs/development/strategy/product-strategy.md.
- Any product-boundary change: a new ADR and docs/01_product_definition.md.
- Stable public claim language: README.md and user documentation.
- External bridge scope: Phase 2 interoperability exploration followed by a separate integration plan.
- Accepted product KPIs: strategy scorecard with named owner and measurement scripts.
