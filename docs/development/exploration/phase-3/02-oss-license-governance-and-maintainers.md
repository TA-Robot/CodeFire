# OSS License, Governance, and Maintainers

Status: exploration

Last reviewed: 2026-07-11

## Legal Disclaimer

This document is not legal advice. The copyright holder must obtain qualified advice before selecting or changing a license, accepting contributions, asserting trademark rights, dual licensing, or making regulatory representations. The purpose here is to expose decisions and evidence requirements.

## Decision Questions

1. Is CodeFire intended to be open source, source-available, or proprietary?
2. What rights apply consistently to Rust, Python, schemas, fixtures, documentation, and generated artifacts?
3. How are third-party and AI-assisted contributions accepted without creating unclear rights?
4. Who can change product invariants, schemas, release artifacts, or security policy?
5. How does the project survive maintainer absence, credential loss, conflict, or organizational change?

## Observed Baseline

- observed: Cargo.toml declares MIT for workspace packages.
- observed: setup.py declares UNLICENSED for the Python package.
- observed: no LICENSE file is present at repository root.
- observed: no contribution, governance, code-of-conduct, support, security, changelog, or maintainer policy file is present.
- observed: all 266 commits visible through git shortlog are attributed to one author identity.
- observed: no release tag or package-registry ownership record is documented in the repository.
- inferred: a user cannot confidently determine the outbound license from the current tree.
- inferred: public contribution would create avoidable copyright, patent, privacy, and governance ambiguity.

## Outbound License Options

### Option A: MIT

Benefits:

- short and widely recognized;
- low compliance burden for users;
- matches the current Cargo declaration.

Costs and questions:

- patent treatment is less explicit than Apache-2.0;
- the current Python UNLICENSED declaration still must be corrected;
- copyright holder and year must be decided;
- documentation and specification reuse still need an explicit policy.

Primary reference: https://opensource.org/license/mit

### Option B: Apache-2.0

Benefits:

- explicit patent license and termination terms;
- common for infrastructure and standards-oriented projects;
- explicit NOTICE handling model.

Costs and questions:

- longer compliance obligations;
- contributors and copyright holders must be comfortable with patent terms;
- migration from any prior effective license needs legal review.

### Option C: MIT OR Apache-2.0

Benefits:

- downstream users choose either license;
- Cargo supports SPDX OR expressions directly;
- common compatibility posture in the Rust ecosystem.

Costs and questions:

- two license texts and consistent metadata must be maintained;
- all rights holders must authorize both choices;
- documentation and non-code material still need a deliberate treatment.

Cargo license-expression reference: https://doc.rust-lang.org/cargo/reference/manifest.html

### Option D: Source-Available or Proprietary

Benefits:

- may preserve exclusive commercial rights;
- can support controlled distribution or paid licensing.

Costs and questions:

- contradicts a claim that CodeFire is OSS;
- substantially raises adoption and ecosystem friction;
- external standardization and package distribution become harder;
- current Cargo MIT declaration would be inaccurate.

## Recommended License Decision Candidate

decision-candidate: If the copyright holder confirms a permissive-OSS strategy and has the necessary rights, evaluate MIT OR Apache-2.0 as the default candidate, with matching root license files and identical Rust/Python metadata.

This is not an automatic legal conclusion. If business, patent, contributor-rights, or prior-distribution evidence contradicts it, select a different model explicitly and remove every inconsistent declaration.

Minimum license completion set:

- root LICENSE-MIT and LICENSE-APACHE, or one chosen LICENSE;
- SPDX expression in Cargo workspace and every published crate;
- matching pyproject.toml metadata; remove duplicate setup.py authority or make it derive from one source;
- copyright holder statement;
- policy for docs, schemas, examples, generated files, and dogfood material;
- third-party notices and dependency-license inventory;
- package and README badges only after the files are authoritative.

SPDX is an international open standard for software-bill-of-material and licensing information: https://spdx.dev/use/specifications/

## Inbound Contribution Options

### Option A: Inbound Equals Outbound Without Sign-Off

Benefits:

- minimum contributor friction.

Risks:

- weak evidence that the submitter had a right to contribute;
- unclear handling of employer-owned or AI-assisted work;
- harder future relicensing.

### Option B: Developer Certificate of Origin

Benefits:

- contributor certifies the right to submit under the indicated license;
- lightweight Signed-off-by workflow;
- compatible with transparent open-source contribution.

Risks:

- sign-off records include personal information and are intentionally durable;
- does not transfer copyright;
- enforcement and bot checks add process.

Primary source: https://developercertificate.org/

### Option C: Contributor License Agreement

Benefits:

- can grant broader rights and support relicensing or dual licensing;
- explicit organization contribution handling.

Risks:

- higher legal and onboarding friction;
- can reduce contributor trust;
- requires administration and privacy handling.

## Recommended Inbound Decision Candidate

decision-candidate: Use inbound-equals-outbound plus DCO for the initial permissive-OSS phase; defer a CLA unless a documented dual-licensing or foundation requirement appears.

Contribution policy should require:

- confirmation of the right to submit;
- Signed-off-by according to DCO 1.1;
- identification of material copied or adapted from elsewhere;
- disclosure of significant AI assistance when provenance or licensing may be relevant;
- human accountability for generated code, tests, docs, and claims;
- no secrets, personal datasets, or confidential employer material;
- agreement that public contribution metadata is retained and redistributed;
- required tests, docs, and evidence under the same quality bar as maintainer work.

## AI-Assisted Contribution Rights

Phase 2 owns technical AI provenance. Phase 3 must decide the contribution policy.

Questions:

- Which model/provider terms applied to generated material?
- Did prompts contain third-party confidential data?
- Can the contributor make the DCO certification?
- Is generated output substantially copied from an identifiable source?
- Who reviews security, correctness, and attribution risk?

decision-candidate: Do not ban AI-assisted work categorically. Require the human submitter to remain the accountable author, make the same rights certification, disclose material assistance in the change record, and provide tests/evidence. Reject work whose provenance or submitter rights cannot be reasonably established.

## Trademark and Naming

Questions requiring professional or registry-specific review:

- Is CodeFire available and defensible in target jurisdictions and software categories?
- Are codefire, codefire-cli, and related package names available in intended registries?
- Who owns the word mark, logo, domains, and social accounts?
- May forks use the name and logo?
- How are compatibility claims such as CodeFire-compatible governed?

decision-candidate: Perform trademark and package-name clearance before public launch; reserve names without claiming legal ownership; publish a trademark-use policy only after rights are understood.

## Governance Model Options

### Option A: Undocumented Founder Control

Fast but opaque. Appropriate only for private experimentation. It does not scale to external trust or succession.

### Option B: Founder-Led Open Governance

The founder retains final product direction while decision classes, review requirements, roles, appeals, and promotion paths are public.

Benefits:

- fast enough for an early project;
- clear accountability;
- contributors can predict how decisions are made.

Costs:

- founder remains a concentration risk;
- conflict resolution still depends on one person unless a backup exists.

### Option C: Maintainer Council

Shared authority with voting or lazy consensus.

Benefits:

- higher continuity and legitimacy;
- distributes review and release responsibility.

Costs:

- premature process if there are no independent maintainers;
- slower decisions and possible deadlock.

### Option D: Foundation Stewardship

Neutral legal and governance home.

Benefits:

- stronger vendor-neutral standardization story;
- formal trademark, finance, and succession mechanisms.

Costs:

- high administrative overhead;
- not justified before community and funding exist.

## Recommended Governance Decision Candidate

decision-candidate: Adopt founder-led open governance now, with an explicit trigger to form a maintainer council after at least three active independent maintainers or two organizations contribute substantially for two consecutive release cycles.

Required roles, even if one person temporarily holds several:

| Role | Accountable for | Must have backup before |
|---|---|---|
| Product Steward | vision, non-goals, product-boundary decisions | beta |
| Spec and Schema Owner | object identity, format, JSON, protocol compatibility | first public schema promise |
| Release Manager | version, tag, artifacts, provenance, registry publication | first public artifact |
| Security Responder | private reports, triage, advisory, patch coordination | public release |
| Maintainer | review, merge, contributor mentoring | beta |
| Community Moderator | code of conduct and moderation | public contribution launch |
| Infrastructure Owner | CI, domains, registries, backups, credentials | first automated release |

## Decision Classes

| Class | Examples | Proposed approval |
|---|---|---|
| Product invariant | consistent-only history, no hidden checkpoint, object identity | proposal, ADR, Product Steward plus independent reviewer |
| Public compatibility | repository/object/schema/protocol/CLI break | compatibility analysis, migration plan, Spec Owner plus reviewer |
| Security-sensitive | auth, signatures, remote mutation, release pipeline | threat review and two-person approval |
| Additive implementation | internal refactor, compatible feature | maintainer review and tests |
| Documentation correction | non-normative typo, stale link | one maintainer and docs check |
| Emergency security | privately reported exploitable issue | Security Responder may embargo; retrospective review required |
| Governance/license | license, maintainer rights, code of conduct | public proposal and rights-holder/professional review as applicable |

## Maintainer Lifecycle

The governance policy should define:

- nomination criteria and evidence of sustained contribution;
- permissions granted by role;
- inactivity and emeritus status;
- removal for security, conduct, or persistent non-performance reasons;
- conflict-of-interest disclosure;
- appeal and tie-breaking;
- succession when the Product Steward is unavailable;
- transfer and recovery of domains, registries, signing identities, and CI secrets;
- prohibition on shared personal credentials;
- periodic access review.

## Security Disclosure and Support

Before public release:

- create SECURITY.md with supported versions and a private contact path;
- enable a private vulnerability reporting mechanism where hosting supports it;
- define acknowledgment, triage, coordinated disclosure, CVE/advisory, and credit policy;
- distinguish best-effort community support from paid or contractual support;
- publish no response SLA until staffing can meet it.

GitHub's official documentation distinguishes SECURITY.md from private vulnerability reporting and recommends a clear private channel: https://docs.github.com/en/code-security/how-tos/report-and-fix-vulnerabilities/report-privately

## Trade-Offs

- DCO is lighter than a CLA but limits future relicensing flexibility.
- Founder-led governance preserves speed but does not by itself fix bus factor.
- Two-person review raises latency but protects high-impact decisions.
- Dual licensing increases metadata work but may reduce downstream friction.
- Public decision records increase trust but can expose security details if embargo rules are absent.

## Falsification and Review Conditions

- If no independent contributor is active after two public release cycles, simplify community process but retain license and security clarity.
- If DCO causes material qualified-contributor abandonment, measure why before replacing it.
- If dual licensing creates recurring downstream confusion, choose one outbound license with rights-holder approval.
- If one person still controls all release and recovery paths at beta, do not claim mature community governance.
- If the project requires proprietary dual licensing, revisit CLA and outbound license decisions before accepting contributions under incompatible assumptions.

## KPIs

| KPI | Candidate target |
|---|---|
| Package/license metadata agreement | 100 percent |
| Files or dependencies with unknown license status | 0 at release |
| DCO sign-off on external contributions | 100 percent if adopted |
| Critical changes with two-person review | 100 percent after backup reviewer exists |
| People capable of performing a release | at least 2 before beta |
| People capable of receiving a security report | at least 2 before stable |
| Median external review latency | under 3 business days during active development |
| Contributor concentration | no single person above 80 percent of merged changes after community stage |
| Quarterly access review completion | 100 percent |

## Experiments

### P3-GOV-001: License Inventory and Rights Audit

- Inventory every source, script, fixture, document, diagram, generated binary, and dependency class.
- Record origin, author/rightsholder, current declaration, intended declaration, and unresolved question.
- Run package metadata and dependency-license tooling when Rust is available.
- Exit: no unknown first-party rights before a license decision is promoted.

### P3-GOV-002: Contribution Dry Run

- Ask an independent contributor to follow a draft CONTRIBUTING plus DCO process for a small docs or test change.
- Measure completion time, confusion, privacy concern, and reviewer work.
- Exit: contributor can complete without private coaching and understands rights/accountability.

### P3-GOV-003: Release Succession Drill

- Give a backup maintainer documented access but no private oral instructions.
- Have them build, verify, sign a non-public rehearsal artifact, and explain rollback.
- Exit: rehearsal succeeds without the primary maintainer's credentials.

### P3-GOV-004: Governance Tabletop

- Simulate a product-boundary dispute, a maintainer conduct issue, and an embargoed vulnerability.
- Identify missing authority and communication rules.
- Exit: every scenario has a named decision owner and appeal path.

## Promotion Targets

- Accepted license: root license files and package metadata, after rights/legal review.
- Accepted contribution terms: CONTRIBUTING.md and DCO enforcement.
- Accepted governance: GOVERNANCE.md and MAINTAINERS.md.
- Accepted conduct rules: CODE_OF_CONDUCT.md.
- Accepted security process: SECURITY.md and private reporting configuration.
- Product-boundary changes: ADR and docs/01_product_definition.md.
