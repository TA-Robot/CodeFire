# Phase 3 Observed Baseline

Status: exploration

Last reviewed: 2026-07-11

## Purpose

This file separates facts visible in the current repository from hypotheses about market fit, governance, operations, or compliance. It should be refreshed before Phase 3 decisions are promoted.

## Product Boundary

- observed: README.md describes CodeFire as a consistency-first, specification-driven, version-control-style CLI and explicitly says it is neither Git-compatible nor a Git layer.
- observed: docs/01_product_definition.md defines the core result as a sealed, policy-consistent system state with Atoms, Trace Links, Fires, Evidence, verification, and certificates.
- documented: Git compatibility, transparent Git repository coexistence, staging, stash, WIP commits, partial commits, GUI, hosted-service behavior, AI-agent orchestration, marketplace behavior, and prompt orchestration are product non-goals.
- documented: external AI, CI, and human systems are expected to use the CLI, JSON envelope, dry-run plans, and sealed history.
- inferred: the core differentiation is strong, but the required behavioral break from Git creates a higher adoption cost than an additive developer tool.
- inferred: marketing language that says consistent without saying policy-consistent can be misunderstood as a proof of semantic truth, which the product explicitly does not claim.

## Implementation and Distribution Identity

- observed: Rust is described as the default implementation, while the Python implementation is a reference and fallback.
- observed: Cargo workspace packages declare version 0.1.0.
- observed: pyproject.toml and setup.py declare Python version 0.2.0 and package name codefire-mvp.
- observed: docs and runbook call the Rust line v0.6, while later plans use v0.7 through v1.0.
- observed: target/release/codefire-rs --version returned codefire foundation 1.
- observed: git tag returned no tags.
- documented: package-index publication, signed release artifacts, and OS packages are not implemented.
- inferred: users currently cannot unambiguously map a binary to a source revision, compatibility promise, repository format, or support period.

## License and Contribution Surface

- observed: Cargo.toml declares MIT.
- observed: setup.py declares UNLICENSED.
- observed: no LICENSE or LICENSE-* file is tracked at repository root.
- observed: no CONTRIBUTING.md, SECURITY.md, CODE_OF_CONDUCT.md, GOVERNANCE.md, SUPPORT.md, CHANGELOG.md, MAINTAINERS, or CODEOWNERS file is tracked.
- observed: no .github directory is present in the current worktree.
- documented: Cargo interprets license fields as SPDX expressions and recommends complete package metadata plus cargo publish --dry-run before publication: https://doc.rust-lang.org/cargo/reference/publishing.html
- inferred: public reuse and contribution rights are ambiguous until the repository license and all package metadata agree.

## Maintainer Concentration

- observed: git shortlog over all refs reports 266 commits from one author identity.
- observed: no formal backup release owner, security responder, or spec/schema owner is recorded.
- inferred: release credentials, security response, product-boundary decisions, and review continuity have a bus factor of one even if additional people collaborate outside Git history.

## Release and Supply-Chain Controls

- observed: Cargo.lock is committed.
- observed: no checked-in release workflow, checksum manifest, SBOM generator, provenance policy, artifact-signing policy, dependency license policy, or vulnerability disclosure process was found.
- observed: the current release acceptance list covers issues, Rust formatting/lint/tests, installer smoke, dogfood commands, and documentation agreement.
- inferred: those gates validate source behavior but do not prove that a downloaded binary was built from the reviewed source or that consumers can authenticate it.
- documented: SLSA 1.2 distinguishes build and source provenance: https://slsa.dev/spec/v1.2/provenance
- documented: Sigstore supports signing and verifying ordinary files and blobs: https://docs.sigstore.dev/cosign/signing/signing_with_blobs/

## Operations and Trust Boundary

- observed: CodeFire is local-first and includes file-backed, HTTP, and HTTPS remote workflows.
- observed: the remote supports authentication policy, HMAC commit/request signatures, nonce replay checks, retention, GC audit output, doctor, migration, and storage reporting.
- documented: server-side verification runs configured commands below the remote project root, with bounded environment and timeout, but without OS-level sandbox or container isolation.
- observed: permission and branch protection are file-backed policy rather than a multi-tenant identity service.
- inferred: the current serve surface is suitable only for a trusted, administratively controlled, single-tenant environment unless a stronger threat model proves otherwise.
- inferred: multi-tenant or public hosted operation would materially expand product scope and operational liability.

## Evidence, Privacy, and Retention

- observed: evidence can capture command output, argv commands, artifacts, URIs, and batch records.
- observed: output is bounded and truncation is recorded.
- observed: external absolute artifact paths are redacted by default, while repository-relative paths may be stored.
- documented: secrets should not be stored in object payloads or diagnostics.
- observed: no repository-wide data inventory, sensitive-output detector, deletion workflow, privacy notice, telemetry policy, or encryption-at-rest policy was found.
- inferred: stdout/stderr, actor names, DCO sign-offs, URIs, source paths, and audit logs can contain personal information or secrets.
- inferred: content-addressed sealed history and remote clones make complete erasure difficult; reachability removal alone does not revoke copies already distributed.
- documented: GDPR includes purpose limitation, data minimisation, storage limitation, integrity/confidentiality, accountability, and qualified erasure obligations: https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32016R0679

## Adoption and User Evidence

- observed: this repository contains one named dogfood project, subprojects/algorithm-evolution-agent-lab.
- observed: the product documentation defines developers, reviewers, release owners, automation tools, repository maintainers, and framework authors, but no initial ideal-customer profile or buyer is selected.
- observed: README Quick Start is primarily build, install, and completion setup rather than a complete first sealed commit.
- observed: the command surface contains more than thirty top-level or grouped operations.
- observed: risks already acknowledge Fire fatigue, heavier commits, OS-copy shadow backups, and friction with Git culture.
- inferred: one dogfood project is strong implementation feedback but weak evidence of cross-organization adoption, willingness to change workflow, or economic value.

## Documentation Baseline

- observed: git ls-files reports 555 tracked files, including 332 Markdown files and 288 files under docs/development.
- observed: current-cycle-burn-down-plan.md is approximately 2,830 lines and contains active state plus historical cycles.
- observed: CodeFire_Design_Document_v0.2 exists as Markdown, DOCX, and PDF; root IMPLEMENTATION_TODO.md and multiple MVP/version plans remain beside current Rust documentation.
- observed: the product uses both CodeFire and Code Fire spellings, and public-facing English and internal Japanese prose are mixed without a declared localization policy.
- observed: schemas currently contain examples and sample configuration, not a complete normative schema registry with stable identifiers.
- documented: docs/development/document-consolidation-plan.md already defines lifecycle states and a deletion gate.
- inferred: documentation volume and duplicated temporal claims increase search cost and make obsolete plans appear authoritative.

## Community and Standards Baseline

- observed: CodeFire has a stable JSON envelope goal and versioned internal object shapes.
- observed: no external conformance kit, extension registry, public specification governance process, or independent implementation is evidenced in the repository.
- inferred: CodeFire semantics may map partially to JSON Schema, SARIF, in-toto, and SPDX, but mapping loss has not been measured.
- documented: JSON Schema 2020-12 is the current published specification: https://json-schema.org/specification
- documented: SARIF 2.1.0 is an OASIS standard for static-analysis results: https://www.oasis-open.org/standard/sarif-v2-1-0/
- documented: in-toto publishes stable specification and attestation framework versions: https://in-toto.io/docs/specs/

## Baseline Risks Requiring Immediate Decision Before Public Release

1. License intent and repository/package metadata conflict.
2. Product release identity and version-axis conflict.
3. Security contact and coordinated disclosure path.
4. Authenticatable release artifacts and source mapping.
5. Supported deployment boundary for the remote server.
6. Evidence data classification, sensitive capture prevention, and erasure limitations.
7. Backup release authority and maintainer continuity.
8. A first-user path that proves value before requiring total workflow replacement.

## Refresh Commands

The following read-only commands should be rerun when this baseline is audited:

~~~text
git status --short --branch
git tag --sort=-version:refname
git shortlog -sne --all
git ls-files
git ls-files '*.md'
rg --files -g 'LICENSE*' -g 'SECURITY*' -g 'CONTRIBUTING*' -g 'GOVERNANCE*'
target/release/codefire-rs --version
~~~

## Promotion Target

- Facts about current project state belong in docs/development/README.md after verification.
- Accepted product positioning belongs in docs/development/strategy/ and README.md.
- Accepted release/version decisions belong in a release policy and, where compatibility changes, an ADR.
- Accepted legal, security, and contribution decisions belong in repository-root policy files after professional review where necessary.
