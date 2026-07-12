# Phase 3 Decision Candidates and Experiment Backlog

Status: exploration

Last reviewed: 2026-07-11

## Purpose

This file turns Phase 3 questions into evidence-gathering work. It is not the active product roadmap and must not be used as current-state truth. Items become implementation work only after the named evidence is reviewed and the conclusion is promoted.

## Priority Meaning

- P0: blocks responsible public release or external contribution.
- P1: blocks credible beta adoption in the recommended target segment.
- P2: blocks ecosystem scale or formal standardization but not an initial alpha.
- P3: optional exploration whose value depends on earlier evidence.

## Decision Candidates

| ID | Priority | Decision candidate | Evidence required | Falsification/review trigger | Promotion target |
|---|---:|---|---|---|---|
| P3D-001 | P0 | Resolve outbound license and align all package/file declarations | rights inventory; business intent; professional review as needed | third-party rights or business model contradict chosen license | LICENSE files, Cargo/Python metadata |
| P3D-002 | P0 | Use DCO plus inbound-equals-outbound initially; defer CLA | independent contribution dry run; privacy review | material qualified-contributor loss or dual-licensing requirement | CONTRIBUTING.md, DCO enforcement |
| P3D-003 | P0 | Adopt founder-led open governance with explicit backup roles | role/credential inventory; governance tabletop | community scale or conflict requires council earlier | GOVERNANCE.md, MAINTAINERS.md |
| P3D-004 | P0 | Require private security intake and supported-version statement | responder staffing; disclosure tabletop | no backup responder means no external response SLA | SECURITY.md, hosting configuration |
| P3D-005 | P0 | Audit release identity before selecting first public SemVer | tags, binaries, package registries, known external consumers | evidence of external dependency prevents reset | release policy, compatibility table |
| P3D-006 | P0 | Support only local and trusted single-tenant remote initially | remote threat-boundary review; operator test | qualified adoption requires hosted service | strategy, runbook, product limitations |
| P3D-007 | P0 | No outbound telemetry by default | data-flow inventory | a separately approved opt-in metric need emerges | privacy policy/product design |
| P3D-008 | P0 | Block sensitive command/artifact evidence by default | red-team corpus and false-positive study | safe blocking is unusable; narrow capture instead | product security design, CLI spec |
| P3D-009 | P0 | Distribute only artifacts tied to source by checksum, SBOM, signature, and provenance | non-public release rehearsal | verification cannot be reproduced independently | release pipeline and policy |
| P3D-010 | P0 | Do not call a release beta until two people can publish and handle security reports | succession drill | backup process fails | release/governance gates |
| P3D-011 | P1 | Position core as high-assurance consistency-first VCS and market wedge as AI-change trust/evidence control plane | problem interviews and claim test | users still evaluate only Git parity | product strategy, README |
| P3D-012 | P1 | Focus initial ICP on AI-heavy spec-driven/high-assurance teams | at least twelve interviews across candidate ICPs | no recurring costly problem or pilot demand | product strategy |
| P3D-013 | P1 | Test an external bridge without adding Git semantics to core | four-week bridge pilot | bridge adds confusion without adoption value | Phase 2 integration plan or rejection |
| P3D-014 | P1 | Preserve seal invariant while offering a small starter policy profile plus assurance coverage report | first-seal usability and claim comprehension | starter creates false assurance | product requirements/ADR if needed |
| P3D-015 | P1 | English canonical for public spec/docs if global de-facto goal remains | target-market decision; translation capacity | bounded Japan-first strategy is selected | documentation governance |
| P3D-016 | P1 | Measure total cost to trusted integration as economic north star | baseline and pilot time/cost data | measurement burden exceeds decision value | strategy scorecard |
| P3D-017 | P1 | Do not choose permanent monetization before pilot unit economics | willingness-to-pay and support-load study | funding or legal structure requires earlier choice | sustainability strategy |
| P3D-018 | P1 | Publish explicit release channels and independent version axes | release audit and compatibility fixtures | users cannot understand multiple axes | release policy and automation output |
| P3D-019 | P1 | Expand release gates with recovery, privacy, license, artifact, and compatibility evidence | rehearsal results | gate cost is disproportionate; retain P0 controls and narrow matrix | product release acceptance |
| P3D-020 | P1 | Retire Python from user-facing fallback after port/remove criteria are met | usage, incident, migration, command parity evidence | real incident/migration dependency remains | compatibility strategy and known limitations |
| P3D-021 | P1 | Use documentation lifecycle states and deletion manifest | full inventory and one deletion rehearsal | unique evidence is lost or findability worsens | consolidation plan and checkers |
| P3D-022 | P1 | Report policy-consistent rather than semantically correct in all claims | comprehension and claims red-team | terminology still misunderstood | README, CLI UX, product definition |
| P3D-023 | P2 | Open implementation contribution before formal spec governance | contribution dry runs and maintainer capacity | review load harms security/release work | community strategy |
| P3D-024 | P2 | Publish layered specs with maturity per layer | clean-room consumer | hidden implementation dependency remains | product specs and ADRs |
| P3D-025 | P2 | Adopt URI/organization extension namespaces and critical-extension rules | prototype and compatibility review | extension semantics remain unstable | schema/extension specification |
| P3D-026 | P2 | Export optional SARIF/in-toto/SPDX adapters, not canonicalize around them | adapter lossiness study | decision-critical data cannot be represented | Phase 2 integration plan or rejection |
| P3D-027 | P2 | Pursue de-facto open specification before standards body | independent consumers and organizations | no external use after two stable cycles | ecosystem strategy |
| P3D-028 | P2 | Consider maintainer council after threshold of active independent maintainers/organizations | contribution history and governance load | founder bottleneck appears sooner | GOVERNANCE.md revision |
| P3D-029 | P2 | Treat hosted/multi-tenant service as a separate product boundary | funded owner, threat model, legal/ops review | no sustainable service need or capacity | ADR and separate service strategy |
| P3D-030 | P3 | Evaluate foundation/consortium stewardship | multi-organization dependency and funding | administrative cost exceeds neutral-governance value | governance proposal |

## Experiment Backlog

### P0 Release and Rights Experiments

#### P3E-001: First-Party Rights and License Inventory

Owner role: Product Steward plus qualified legal reviewer where needed.

Inputs:

- source, docs, schemas, fixtures, diagrams, generated files, dogfood boundary, dependencies;
- Cargo and Python metadata;
- contribution and authorship history.

Method:

1. Record origin, rights holder, current declaration, intended declaration, and unresolved issue for every class.
2. Identify copied/adapted material and generated binaries.
3. Produce allowed/blocked publication list.

Exit evidence:

- no unknown first-party rights in the public release set;
- one approved outbound license decision;
- matching package metadata plan.

#### P3E-002: Release Identity Audit

Owner role: Release Manager.

Method:

- inspect repository tags and history;
- inventory distributed binaries and their --version output;
- check intended package registries and known users;
- record repository/object formats in use;
- decide whether a SemVer reset is compatible with external reality.

Exit evidence:

- version-axis table;
- first public version proposal;
- migration/compatibility obligations.

#### P3E-003: Non-Public Signed Release Rehearsal

Owner role: Release Manager plus independent verifier.

Method:

- build from clean source in CI;
- create source archive, binaries, checksums, SPDX SBOM, signature, and SLSA-compatible provenance;
- verify and install from a clean environment;
- perform rollback.

Exit evidence:

- two-person reproducible procedure;
- no undocumented local state;
- public verification instructions draft.

#### P3E-004: Security Disclosure and Credential Tabletop

Owner role: Security Responder and backup.

Scenarios:

- confidential vulnerability;
- leaked registry credential;
- compromised artifact;
- lost signing identity;
- malicious contributor report.

Exit evidence:

- private intake, triage, embargo, release, advisory, credit, rotation, and recovery path;
- named backup operator.

#### P3E-005: Sensitive Evidence Red Team

Owner role: Security/Privacy owner.

Corpus:

- API tokens, passwords, private keys, signed URLs, usernames, personal identifiers, secret argv, binary data, and benign lookalikes.

Surfaces:

- command, argv, artifact, URI, batch, truncation, JSON, audit, remote, backup, clone.

Exit evidence:

- false-negative/false-positive results by class;
- default block/redact/reference proposal;
- incident and erasure limitations.

#### P3E-006: Backup, Restore, and Unsealed-Work Drill

Owner role: Operator plus independent restorer.

Method:

- create sealed history, open work, remote state, external evidence, keys, and audit records;
- back up according to draft instructions;
- destroy and restore test environment;
- validate with doctor and sealed-graph checks;
- record unrecoverable unsealed state honestly.

Exit evidence:

- tested RPO/RTO;
- backup scope and credential boundary;
- runbook gaps.

### P1 Product and Adoption Experiments

#### P3E-010: ICP Problem Interviews

Owner role: Product Steward.

Sample:

- at least four participants from each of AI-heavy spec-driven, high-assurance/internal-platform, and research/experiment profiles.

Evidence:

- recent real incident;
- current process and cost;
- buyer/operator/user;
- workflow constraints;
- pilot commitment.

Exit threshold:

- at least half identify a recurring costly problem and qualified pilot path.

#### P3E-011: Claims Comprehension Test

Owner role: Product/UX.

Method:

- show public product page only;
- ask what a certificate proves and does not prove;
- present a policy-consistent but wrong example.

Exit threshold:

- at least 80 percent correctly distinguish policy consistency from correctness/security/compliance.

#### P3E-012: First-Seal Usability Test

Owner role: UX/Docs.

Method:

- clean environment;
- public docs only;
- observe install through first meaningful seal;
- record intervention, error, help lookup, and conceptual confusion.

Exit threshold:

- median at most 15 minutes, p90 at most 30 minutes, at least 80 percent completion without private coaching.

#### P3E-013: External Bridge Pilot

Owner role: Phase 2 integration owner plus Product Steward.

Method:

- three existing repositories for four weeks;
- explicit Git/CodeFire authority labels;
- read-only assessment or one-way import;
- measure duplicate work, value, false confidence, and native-migration interest.

Exit threshold:

- at least two teams request deeper native workflow, or evidence supports verifier-first pivot.

#### P3E-014: Fire Actionability and Evidence Audit

Owner role: Product/Reviewer.

Method:

- blind-label at least 200 Fires across repositories and artifact types;
- classify actionable, redundant, false-positive, or unverifiable;
- sample relevance, freshness, and boilerplate of resolutions/evidence.

Exit threshold:

- actionability at least 70 percent and boilerplate at most 15 percent in each supported profile.

#### P3E-015: Four-Week Team Adoption Pilot

Owner role: Adoption lead.

Method:

- baseline review lead time, escaped drift, CI, human time, and support;
- designate champion and skeptic;
- weekly Fire/evidence review;
- predeclare expand, modify, narrow, and stop thresholds.

Exit evidence:

- retention intent;
- workflow tax;
- avoided issues;
- support load;
- role-specific objections.

#### P3E-016: Trusted Change Cost Study

Owner role: Product/Economics.

Method:

- compare at least 20 similar changes where possible;
- include human, AI, CI, storage, rework, incident, onboarding, and support costs;
- record observed benefits separately from speculative risk reduction.

Exit threshold:

- at least one target segment has positive measured or defensible risk-adjusted value.

#### P3E-017: Sustainability and Willingness-to-Pay Interviews

Owner role: Product/Business.

Options tested:

- support, training, enterprise integration, assurance profile, separate managed service, sponsorship.

Evidence:

- budget owner, procurement requirement, expected SLA, alternative spend, paid/sponsored pilot.

Exit rule:

- do not treat verbal enthusiasm as a selected business model.

### P1 Documentation and Governance Experiments

#### P3E-020: Contribution Dry Run

Owner role: Maintainer/Community.

Method:

- independent contributor uses draft license, DCO, contributor guide, tests, and review process;
- measure contributor and maintainer time;
- review AI-assistance disclosure if applicable.

Exit evidence:

- no private setup knowledge required;
- rights/accountability understood;
- review standard predictable.

#### P3E-021: Release Succession Drill

Owner role: Backup Release Manager.

Method:

- release rehearsal without primary maintainer credentials or oral instruction;
- recover access through documented process;
- verify artifact and rollback.

Exit threshold:

- successful independent execution before beta.

#### P3E-022: Full Document Inventory

Owner role: Documentation owner.

Required fields:

- path, title, lines, audience, lifecycle, owner, update trigger, inbound links, replacement, unique evidence, language, generated source.

Exit evidence:

- every document classified;
- deletion/archive batches ordered by risk;
- no candidate deletion without destination or explicit no-unique-content finding.

#### P3E-023: Persona Findability Test

Owner role: Documentation/UX.

Personas:

- evaluator, developer, reviewer, automation author, operator, contributor.

Method:

- five common questions per persona;
- ten-minute time box;
- record wrong sources and contradictory answers.

Exit threshold:

- at least 80 percent correct answers and no current-state contradiction.

### P2 Ecosystem Experiments

#### P3E-030: Clean-Room Contract Consumer

Owner role: Spec Owner.

Method:

- provide candidate layered specification and fixtures but no internal source maps;
- ask an independent engineer to implement a reader/validator;
- capture every ambiguity and hidden dependency.

Exit threshold:

- conformance passes without private clarification before stable label.

#### P3E-031: Adapter Lossiness Study

Owner role: Ecosystem/Phase 2 integration owner.

Mappings:

- Fires/diagnostics to SARIF;
- Evidence/Certificate to in-toto;
- release components/licenses to SPDX.

Exit evidence:

- field-by-field exact, extended, lossy, and impossible mapping;
- decision whether adapter is supported, experimental, or rejected.

#### P3E-032: Extension Namespace Prototype

Owner role: Spec Owner.

Method:

- define two independent vendor extensions, one non-critical and one blocking/identity-affecting;
- test collision, preservation, rejection, capability, and migration behavior.

Exit evidence:

- criticality and ownership rules are implementable and understandable.

#### P3E-033: Standardization Readiness Review

Owner role: Product Steward plus independent organizations.

Required evidence:

- three organizations using stable contracts;
- two independent integrations or one independent implementation;
- two compatible release cycles;
- public IPR, trademark, governance, conformance, and change policy;
- funded maintainer capacity.

Outcome:

- pursue formal venue, continue de-facto spec, or defer.

## Proposed Execution Order

### Wave 0: Public-Release Blockers

1. P3E-001 rights/license inventory.
2. P3E-002 release identity audit.
3. P3E-004 security/credential tabletop.
4. P3E-005 sensitive evidence red team.
5. P3E-006 backup/restore drill.
6. P3E-003 non-public release rehearsal.

### Wave 1: Product and Adoption Proof

1. P3E-010 problem interviews.
2. P3E-011 claims comprehension.
3. P3E-012 first-seal usability.
4. P3E-014 Fire/evidence audit.
5. P3E-013 bridge pilot.
6. P3E-015 team pilot.
7. P3E-016 trusted-change economics.

### Wave 2: Community and Documentation

1. P3E-022 document inventory.
2. P3E-023 findability.
3. P3E-020 contribution dry run.
4. P3E-021 succession drill.

### Wave 3: Ecosystem

1. P3E-030 clean-room consumer.
2. P3E-031 adapter lossiness.
3. P3E-032 extension prototype.
4. P3E-033 standardization readiness.

## Experiment Record Template

~~~text
Experiment ID:
Owner:
Decision candidate:
Question:
Hypothesis:
Population/fixture:
Method:
Data collected:
Privacy/security constraints:
Success threshold:
Failure threshold:
Expiry date:
Result:
Confounders:
Decision:
Promotion/rejection destination:
Follow-up:
~~~

## Portfolio Governance

- No experiment should start without an owner, decision, threshold, and expiry.
- A failed experiment is evidence; do not rewrite thresholds after results.
- User research must minimize and protect personal/company data.
- Security/legal blockers outrank adoption and feature experiments.
- Metrics from the single dogfood repository must be labeled separately from independent pilots.
- Do not promote numeric thresholds to public SLA or release gate until measurement is repeatable and owned.
- Every promoted decision removes or updates its exploration backlog item; this file must not become a second active roadmap.

## External Primary Sources Used by Experiments

- Cargo publishing and permanence: https://doc.rust-lang.org/cargo/reference/publishing.html
- SemVer: https://semver.org/
- SPDX: https://spdx.dev/use/specifications/
- DCO: https://developercertificate.org/
- NIST SSDF: https://csrc.nist.gov/pubs/sp/800/218/final
- SLSA provenance: https://slsa.dev/spec/v1.2/provenance
- Sigstore blob signing: https://docs.sigstore.dev/cosign/signing/signing_with_blobs/
- GDPR official text: https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32016R0679
- CRA official text: https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32024R2847
- JSON Schema: https://json-schema.org/specification
- SARIF: https://www.oasis-open.org/standard/sarif-v2-1-0/
- in-toto: https://in-toto.io/docs/specs/

## Promotion Target

Accepted decisions must be moved to the specific strategy, ADR, policy, runbook, release, product, or governance destination named in the table. Rejected decisions remain in exploration with the rejection reason and reconsideration trigger. Experiment execution state belongs in the canonical active roadmap only after the manager explicitly schedules it.
