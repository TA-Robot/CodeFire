# Premortem, Kill, and Pivot Criteria

Status: exploration

Last reviewed: 2026-07-11

## Purpose

Assume that two years from now CodeFire is technically sophisticated but not trusted, adopted, maintainable, or economically sustainable. This file lists plausible causes, leading indicators, prevention experiments, and explicit conditions for stopping or narrowing work.

Kill criteria here apply to a claim, market, deployment profile, integration, or roadmap branch. They do not automatically mean deleting the CodeFire research or abandoning every use case.

## Failure 1: Category Confusion

Failure story:

Users hear version control and evaluate CodeFire as an incomplete Git clone. They never reach the consistency/evidence value.

Leading indicators:

- interviews focus on missing rebase, stash, hosting, and pull-request features;
- evaluators cannot explain sealed policy-consistent history after reading the product page;
- first-use abandonment occurs before scan/verify;
- marketing repeatedly says correct rather than policy-consistent.

Prevention:

- two-layer product statement;
- claim-comprehension tests;
- explicit anti-personas and non-goals;
- first-seal demo centered on a real missed cross-artifact change.

Pivot/kill:

- if qualified target users still evaluate only Git parity after three positioning iterations, stop broad VCS-replacement marketing and position as a consistency/evidence verifier for a narrow segment.

## Failure 2: Migration Cliff

Failure story:

Teams agree with the model but cannot justify replacing repositories, review, CI, IDE, backup, and collaboration systems at once.

Leading indicators:

- many interested interviews but few repository pilots;
- pilot requires dual manual entry;
- shadow Git remains the actual source of truth;
- external bridge is the only retained feature.

Prevention:

- external read-only/one-way bridge experiment;
- net-new and bounded project focus;
- explicit authority boundary;
- migration cost model and rollback.

Pivot/kill:

- if fewer than three of ten qualified pilots produce a second native seal, stop assuming immediate VCS replacement; evaluate verifier/specification-first strategy.

## Failure 3: Fire Fatigue

Failure story:

Fires become notification noise. Users bulk-extinguish them and the certificate becomes ritual rather than review.

Leading indicators:

- actionability below 60 percent;
- repeated identical rationale;
- bulk resolution without inspecting targets;
- users weaken policy after each large change;
- review focuses on clearing count, not evaluating basis.

Prevention:

- labeled Fire-quality corpus;
- artifact-kind precision metrics;
- grouping and bounded output;
- change-kind-aware policies;
- rationale sampling and stale-basis tests.

Pivot/kill:

- disable or narrow low-precision propagation/profile rules rather than keeping noisy coverage; withdraw assurance claims for unsupported artifact kinds.

## Failure 4: Evidence Laundering

Failure story:

Users attach any passing command or generic rationale, so evidence looks formal but does not support the resolution.

Leading indicators:

- high evidence count with unchanged review confidence;
- same test output linked to unrelated Fires;
- no-change-required boilerplate above 25 percent;
- reviewers never inspect evidence content or freshness.

Prevention:

- decision-specific evidence guidance;
- evidence-to-Fire relevance sampling;
- freshness and basis checks;
- reviewer challenge workflow;
- distinguish captured data from proof status.

Pivot/kill:

- if evidence relevance cannot be improved, reduce automated assurance claims and require explicit reviewer approval for high-risk seals.

## Failure 5: False Assurance

Failure story:

A valid certificate is interpreted as proof that the system is correct, secure, compliant, or complete.

Leading indicators:

- marketing or docs omit policy coverage;
- post-seal bugs are described as certificate failures even when outside policy;
- users stop independent review/testing;
- procurement treats CodeFire as certification without assessment.

Prevention:

- assurance claim ladder;
- report configured and unconfigured coverage;
- negative examples that are policy-consistent but wrong;
- forbid certification/compliance language without review.

Pivot/kill:

- if users cannot reliably understand limits, rename outputs and narrow claims before stable release.

## Failure 6: Single-Dogfood Overfitting

Failure story:

The product works beautifully for the algorithm lab's Python/Markdown workflow but fails in other languages, organizations, scales, or release cultures.

Leading indicators:

- regressions cluster outside the dogfood artifact mix;
- policy configuration requires maintainer-specific knowledge;
- benchmarks and examples all use one project shape;
- external pilots disable most features.

Prevention:

- representative corpus by language, size, artifact, and organization;
- independent pilots;
- publish a supported scale/artifact envelope;
- avoid universal claims.

Pivot/kill:

- narrow the official profile to proven repository types when cross-domain results remain weak.

## Failure 7: Secret or Personal-Data Immutability Incident

Failure story:

Command output, URI, actor data, or source content containing a secret or personal identifier enters sealed and replicated history; maintainers cannot truthfully erase every copy.

Leading indicators:

- secrets in argv or evidence examples;
- no data inventory or sensitive-pattern tests;
- deletion means only local reachability removal;
- backups and clones are absent from incident playbooks.

Prevention:

- privacy-by-default capture;
- secret-pattern preflight and risky override policy;
- external evidence refs;
- data inventory, retention, erasure, and compromise drills;
- no telemetry by default.

Pivot/kill:

- disable inline command/artifact capture for a data class if safe lifecycle handling cannot be demonstrated.

## Failure 8: Remote Verification Becomes Remote Code Execution

Failure story:

An operator exposes serve to untrusted projects or tenants, and configured verification executes with service privileges.

Leading indicators:

- public deployment examples omit trust assumptions;
- multiple organizations share one service process;
- verification configuration comes from untrusted repository content;
- hosted requests arrive before sandbox/isolation exists.

Prevention:

- trusted single-tenant support profile;
- explicit no-sandbox warning;
- external process/container isolation for experiments;
- separate hosted-service threat model and ADR.

Pivot/kill:

- stop public/multi-tenant remote work until isolation, identity, operations, and staffing are funded and reviewed.

## Failure 9: Supply-Chain Compromise

Failure story:

Users install an artifact that cannot be tied to reviewed source, or a registry/signing credential is compromised.

Leading indicators:

- unsigned binary distribution;
- builds from a dirty workstation;
- no SBOM/provenance/checksum;
- one personal registry owner;
- release cannot be reproduced by a second operator.

Prevention:

- clean CI release, signed artifacts, SLSA-compatible provenance, SPDX SBOM;
- two-person publication and credential recovery;
- compromised-release tabletop.

Pivot/kill:

- halt distribution when artifact authenticity or registry control is uncertain; do not downgrade the issue to a documentation warning.

## Failure 10: Version and Schema Chaos

Failure story:

Crate, Python, binary, docs, repository, object, JSON, and protocol versions diverge; wrappers and histories break unpredictably.

Leading indicators:

- multiple current version claims;
- no tag maps to binary;
- schema meaning changes without a version;
- migration tests cover only current fixtures;
- external wrappers pin source commits rather than releases.

Prevention:

- release-baseline audit;
- independent version axes and compatibility table;
- conformance and migration matrix;
- stable capability output.

Pivot/kill:

- remain alpha and stop registry publication until persistent/public contracts are coherent.

## Failure 11: License and IP Ambiguity

Failure story:

Users or contributors cannot determine their rights; enterprise adoption and package publication stop during legal review.

Leading indicators:

- MIT and UNLICENSED remain together;
- no license text or copyright owner;
- AI-assisted contributions lack accountability;
- dependency license inventory is missing.

Prevention:

- rights audit, selected license, consistent metadata, DCO/CLA decision, dependency policy, trademark review.

Pivot/kill:

- block public release and external contribution acceptance until rights are clarified.

## Failure 12: Maintainer or Credential Loss

Failure story:

One person becomes unavailable, and no one can review security fixes, publish a release, recover domains/registries, or interpret product invariants.

Leading indicators:

- all commits and approvals remain with one person;
- undocumented personal credentials;
- no backup security contact;
- release rehearsal fails without oral help.

Prevention:

- founder-led open governance, backup owners, access review, succession drill, architecture and release playbooks.

Pivot/kill:

- do not declare beta/stable until at least two people can execute release and security response.

## Failure 13: Documentation Becomes the Product

Failure story:

Hundreds of plans, source maps, issue narratives, and explorations consume more maintenance than user value and contradict one another.

Leading indicators:

- active state appears in multiple files;
- source maps use stale line numbers;
- agents read large historical documents for simple work;
- new documents are added instead of promoting/deleting old ones.

Prevention:

- lifecycle inventory, deletion gate, compact hubs, archive, generated symbol maps, findability tests.

Pivot/kill:

- freeze new permanent docs until owner, audience, lifecycle, and replacement are defined; delete/archive superseded material as reviewable packages.

## Failure 14: Python Fallback Never Ends

Failure story:

Rust and Python remain two user-facing products with divergent commands, security fixes, package names, and support promises.

Leading indicators:

- new behavior repeatedly requires dual fixes;
- Python-only maintenance commands block retirement;
- users cannot tell which binary is active;
- packaging/version identity remains split.

Prevention:

- explicit freeze, port/remove criteria, migration fixtures, incident-only support scope, retirement decision date.

Pivot/kill:

- split Python into an archived compatibility tool when its mainline cost exceeds verified incident/migration value.

## Failure 15: Performance or Storage Cliff

Failure story:

The model works on small repositories but scan, verify, evidence, branch listing, or object retention becomes unusable at real scale.

Leading indicators:

- timeouts without progress;
- p95 grows nonlinearly with repository/Atom/object count;
- users disable checks or avoid seals;
- storage/GC cost dominates.

Prevention:

- Phase 1 scale model and benchmark corpus;
- published support envelope;
- progress, cancellation, bounded output, retention economics.

Pivot/kill:

- remove large-repository claims and narrow supported scale until budgets are met.

## Failure 16: Premature Standardization

Failure story:

Maintainers spend time on committees and frozen schemas before any independent implementation or durable user demand exists.

Leading indicators:

- specification discussion exceeds pilot work;
- no external consumer passes conformance;
- frequent breaking changes require political process;
- implementation quality and security work slow down.

Prevention:

- de-facto open spec first;
- threshold of independent organizations and consumers;
- layered maturity labels.

Pivot/kill:

- defer formal venue until three organizations and independent conformance evidence exist.

## Failure 17: Negative Unit Economics

Failure story:

Human review, Fire resolution, AI, CI, storage, and support cost more than the inconsistency risk CodeFire removes.

Leading indicators:

- workflow tax above 20 percent after onboarding;
- support above four hours per active team/month;
- no measured avoided incidents or review savings;
- users praise concept but do not renew pilots or pay.

Prevention:

- trusted-change cost study;
- narrow ICP with expensive consistency failures;
- policy precision and automation;
- explicit support scope.

Pivot/kill:

- narrow to high-value segments, become a verifier/specification, or maintain as bounded research if no segment has positive value.

## Failure 18: Unsafe or Unsustainable Business Expansion

Failure story:

Hosted service, compliance profiles, enterprise features, or support promises are added to satisfy prospects without threat modeling, legal review, or staffing.

Leading indicators:

- roadmap commitments precede funded owners;
- security features become paid rather than default;
- contractual SLA exceeds observed internal SLO;
- CRA/privacy roles are unknown.

Prevention:

- separate product-boundary decision;
- owner/cost/threat/legal analysis;
- paid pilot before permanent scope;
- secure-by-default rule.

Pivot/kill:

- stop the expansion when required staffing, isolation, legal classification, or recovery evidence is absent.

## Failure 19: Community Capture or Toxicity

Failure story:

A small group dominates roadmap or review, contribution becomes hostile, and maintainers cannot resolve conduct or conflicts of interest.

Leading indicators:

- private exceptions to public rules;
- review depends on affiliation rather than evidence;
- unanswered conduct reports;
- one organization controls release and spec votes.

Prevention:

- code of conduct, transparent governance, conflict disclosure, appeal path, contribution attribution review.

Pivot/kill:

- pause community expansion or transfer neutral assets when governance cannot protect participants and project integrity.

## Failure 20: AI Vendor Coupling Becomes Operational Lock-In

Failure story:

Development policy hardcodes one model/runtime so cost, availability, or product changes halt the project.

Leading indicators:

- roles are defined by vendor names rather than risk and verification needs;
- no fallback or periodic eval;
- procurement and privacy terms are unreviewed;
- model cost is absent from trusted-change economics.

Prevention:

- Phase 2 stable role/risk taxonomy plus versioned runtime policy;
- Phase 3 cost, vendor, data, and continuity review;
- benchmark-driven model replacement.

Pivot/kill:

- switch runtime policy or reduce AI-dependent workflow when measured value does not justify cost/risk; do not change CodeFire core semantics to preserve one vendor workflow.

## Portfolio Stop Rules

Before starting a major roadmap branch, require:

- named problem and target user;
- owner and estimated ongoing support cost;
- invariant and product-boundary impact;
- evidence unavailable today;
- smallest reversible experiment;
- success, failure, and expiry date;
- promotion or deletion destination.

Stop or archive an exploration when:

- its falsification condition is met;
- no owner or experiment exists for two review cycles;
- it duplicates a promoted strategy or ADR;
- prerequisite evidence is impossible or uneconomic to gather;
- its expected value is lower than security, recovery, license, or release blockers.

## Premortem Review Cadence

- Before first public release: review Failures 7-14.
- Before beta: review all failures and require mitigations for 1-15.
- Before hosted/multi-tenant work: repeat Failures 7, 8, 9, 17, and 18 as a dedicated threat/business tabletop.
- Before formal standardization: review Failures 1, 6, 10, 12, 13, and 16.
- Quarterly during active pilots: recompute adoption, Fire quality, workflow tax, support, maintainer, and incident signals.

## KPI Dashboard for Early Warning

| Signal | Warning | Stop/pivot review |
|---|---|---|
| Time to First Seal median | above 20 min | above 30 min after three iterations |
| Eight-week qualified retention | below 40 percent | below 30 percent after ten pilots |
| Fire actionability | below 70 percent | below 60 percent in supported profile |
| Boilerplate evidence | above 15 percent | above 25 percent |
| Workflow tax | above 15 percent | above 20 percent after week two |
| Support load | above 2 hours/team/month | above 4 after onboarding |
| Sensitive capture incidents | any | repeated class without effective mitigation |
| Independent release/security operators | fewer than 2 | block beta/stable |
| Stable-schema accidental breaks | any | withdraw stable label until corrected |
| Independent consumers | fewer than 2 | defer formal standardization |
| Conflicting current-state documents | any | freeze promotion until fixed |

All thresholds are hypotheses until their measurement method and target population are accepted.

## Experiments

### P3-PRE-001: Failure Tabletop

- Assign advocate, skeptic, operator, user, security, legal/compliance, and maintainer roles.
- Walk through the top ten failures with current evidence.
- Require a detection source, decision owner, and executable first response for each.

### P3-PRE-002: Kill-Decision Rehearsal

- Select one low-cost proposed feature or documentation branch.
- Predeclare success and stop thresholds.
- Run the experiment and practice archiving/rejecting it if the threshold fails.
- Goal: prove the project can stop work, not only add plans.

### P3-PRE-003: External Red Team of Product Claims

- Ask independent reviewers to identify where CodeFire appears to promise correctness, security, compliance, recovery, or scale beyond evidence.
- Convert each finding into claim correction, evidence task, or rejected claim.

## Promotion Targets

- Accepted early-warning metrics: strategy scorecard.
- Product claim limits: README and product definition.
- Security/operations stop rules: SECURITY.md, runbook, and release policy.
- Roadmap kill criteria: canonical active roadmap.
- Rejected paths: retain here or a dated exploration decision log; do not place them in current-state docs.
