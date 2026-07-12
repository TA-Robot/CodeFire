# Phase 3: Product, Governance, Operations, and Adoption Exploration

Status: exploration

Last reviewed: 2026-07-11

## Purpose

This directory explores the conditions under which CodeFire can become an operationally sustainable and broadly adopted tool, even when its core computer-science model and AI-development integration are technically sound.

These files are not sources of current project state and are not accepted product policy. They deliberately retain alternatives, objections, falsification criteria, and ideas that may later be rejected. Accepted conclusions must be promoted to strategy, ADRs, product documentation, governance files, the runbook, or release policy.

## Scope

Phase 3 owns the non-core-technical questions around:

- product category, target users, adoption wedge, and credible product claims;
- OSS licensing, inbound contribution terms, trademark, governance, and maintainer continuity;
- supported deployment, SRE, incident response, privacy, compliance, and data lifecycle;
- UX, education, migration, organizational adoption, and economics;
- distribution, release integrity, version identity, support periods, and supply-chain governance;
- community formation, interoperability governance, standardization timing, and documentation information architecture;
- premortem, stop conditions, scope reduction, and pivot criteria.

## Boundary With Other Exploration Phases

- Phase 1 owns cryptographic design, formal security properties, transaction semantics, performance models, test theory, and technical supply-chain threat analysis.
- Phase 2 owns AI-native SDLC mechanics, agent/tool contracts, model routing, evals, MCP/CLI/CI interoperability, and technical migration adapters.
- Phase 3 owns who is responsible, who pays, who supports, what is promised, what is legally distributable, how adoption is measured, and when an experiment should be stopped.

Where a topic crosses phases, these documents state the Phase 3 decision question and link the technical implementation question back to the relevant phase instead of duplicating it.

## Files

| File | Question |
|---|---|
| 00-observed-baseline.md | What product, legal, release, community, operational, and documentation facts are directly visible now? |
| 01-product-strategy-and-market-wedge.md | What category and initial adoption wedge can prove CodeFire's unique value? |
| 02-oss-license-governance-and-maintainers.md | Under what rights and decision model can outsiders safely use and contribute to CodeFire? |
| 03-operations-privacy-and-compliance.md | What deployment and data-handling promises can CodeFire responsibly make? |
| 04-ux-education-adoption-and-economics.md | Can a team learn, adopt, and economically justify the workflow? |
| 05-supply-chain-release-and-versioning.md | How does a user identify, verify, install, upgrade, and support a real release? |
| 06-community-standardization-and-documentation.md | How can CodeFire form an ecosystem without standardizing unstable semantics or drowning in documentation? |
| 07-premortem-kill-and-pivot-criteria.md | How is failure detected early, and when should scope be reduced or direction changed? |
| decision-candidates-and-experiment-backlog.md | Which decisions are ready for evidence gathering, in what order, and with what promotion targets? |

## Evidence Method

Important claims use the labels defined by ../README.md:

- observed: directly checked in the current worktree, command output, or repository history;
- documented: stated by a project source of truth or external primary source;
- inferred: a conclusion drawn from observations but not directly validated with users or operators;
- hypothesis: requires an experiment;
- decision-candidate: a possible policy to promote after evidence review;
- rejected: kept with the reason and reconsideration trigger.

Numeric thresholds in this directory are experiment thresholds, not product promises. A metric can only become a release gate or external SLO after its measurement method, owner, sample population, and review cadence are accepted.

## Legal and Regulatory Disclaimer

Nothing in this directory is legal advice. License selection, copyright and patent ownership, trademark clearance, privacy obligations, export questions, and regulatory classification require review by a qualified professional for the relevant jurisdictions and distribution model. External legal sources are included to frame questions, not to claim compliance.

## External Primary Sources

- Cargo publishing: https://doc.rust-lang.org/cargo/reference/publishing.html
- Cargo manifest and SPDX expressions: https://doc.rust-lang.org/cargo/reference/manifest.html
- Semantic Versioning 2.0.0: https://semver.org/
- SPDX specifications: https://spdx.dev/use/specifications/
- Developer Certificate of Origin 1.1: https://developercertificate.org/
- NIST Secure Software Development Framework 1.1: https://csrc.nist.gov/pubs/sp/800/218/final
- SLSA provenance 1.2: https://slsa.dev/spec/v1.2/provenance
- Sigstore blob signing: https://docs.sigstore.dev/cosign/signing/signing_with_blobs/
- OpenSSF Scorecard: https://scorecard.dev/
- GDPR official text: https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32016R0679
- EU Cyber Resilience Act official text: https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32024R2847
- JSON Schema specification: https://json-schema.org/specification
- SARIF 2.1.0 OASIS standard: https://www.oasis-open.org/standard/sarif-v2-1-0/
- in-toto specifications: https://in-toto.io/docs/specs/

## Promotion Rule

Promoting a conclusion from this directory requires:

1. a named owner and accountable reviewer;
2. the observed evidence and experiment result;
3. alternatives and rejected reasons;
4. impact on CodeFire invariants and product non-goals;
5. implementation, migration, support, and rollback implications;
6. a review date or falsification trigger;
7. a destination in strategy, an ADR, governance policy, the runbook, a release contract, or product documentation.
