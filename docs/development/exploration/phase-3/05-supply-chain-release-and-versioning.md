# Supply Chain, Release, and Versioning

Status: exploration

Last reviewed: 2026-07-11

## Decision Questions

1. What is the first externally meaningful CodeFire release identity?
2. Which compatibility surfaces share a version and which must be independent?
3. How can a user authenticate a binary and map it to source, build process, SBOM, and support policy?
4. Which platforms, architectures, installers, registries, and upgrade paths are supported?
5. Who may publish, yank, revoke, or issue an emergency release?
6. What dependency and vulnerability practices are required before release?

## Observed Baseline

- observed: Rust crates declare version 0.1.0.
- observed: Python packaging declares version 0.2.0 and name codefire-mvp.
- observed: product and development documents refer to v0.2, Rust v0.6, v0.7, v0.8, v0.9, and v1.0.
- observed: the release Rust binary reports codefire foundation 1 rather than a SemVer product release and source revision.
- observed: the repository has no Git tags.
- observed: Cargo.lock exists.
- documented: install.sh builds Rust by default or accepts a prebuilt binary, and Python remains a fallback.
- documented: package-index publication, signed release artifact, and OS package are not implemented.
- observed: no release workflow, changelog, checksum, SBOM, provenance, or artifact-verification instructions are tracked.
- inferred: current version labels are development-line names, not a reliable public compatibility contract.

## Version Axes

CodeFire needs to separate at least these axes:

| Axis | What it identifies | Compatibility question |
|---|---|---|
| Product release | CLI and supported product behavior | Can a user upgrade without losing supported behavior? |
| Rust crate/package | registry artifact and Rust API/package dependency | Does Cargo resolve a compatible package? |
| Python fallback | reference/fallback behavior | Which historical repository and incident workflows remain supported? |
| Repository format | .codefire layout and persistent metadata | Can this binary open, write, migrate, and recover the repository? |
| Object schema | canonical object payload and identity | Does reading/writing preserve object IDs and sealed validation? |
| JSON envelope | automation command-result contract | Can wrappers parse success/failure and next actions? |
| Remote protocol | file/HTTP/HTTPS operations and request semantics | Can client and server interoperate safely? |
| Configuration schema | codefire YAML/policy/links syntax | Does existing configuration retain meaning? |
| Evidence/certificate schema | assurance and proof representation | Can reviewers verify old seals and external attestations? |
| Documentation version | docs applicable to a release/channel | Are examples and support claims current? |

These axes may use related numbers, but none should be inferred from another without an explicit compatibility table.

## Public Version Baseline Options

### Option A: Continue at v0.6

Interpret Rust v0.6 as the first public product line and align crates, binary, docs, and tags to 0.6.x.

Benefits:

- matches substantial current documentation;
- preserves the story that Python v0.2 preceded the Rust rewrite.

Risks:

- may imply five prior public Rust releases that never existed;
- package versions currently contradict it;
- v0.7-v1.0 plan names may look like shipped versions.

### Option B: Reset First Public Release to 0.1.0-alpha.1

Treat all current labels as internal milestones and establish the first public contract from evidence.

Benefits:

- honest if no external release was distributed;
- fits SemVer initial-development semantics;
- creates room to stabilize packaging and compatibility.

Risks:

- conflicts with existing v0.6 language;
- may appear to move backward;
- cannot be used if external consumers already depend on higher identities.

### Option C: Choose the Highest Externally Consumed Compatibility Line

Audit actual distributed binaries/repositories and preserve the highest identity users rely on.

Benefits:

- protects real consumers;
- bases versioning on compatibility evidence.

Risks:

- requires external inventory;
- could carry accidental historical claims forward.

## Recommended Version Decision Candidate

decision-candidate: Create a release-baseline audit before choosing a number. If no external distribution or repository dependency exists, use 0.1.0-alpha.1 as the first public release candidate and classify v0.2-v1.0 documents as internal/historical milestone labels. If external consumers exist, preserve the highest relied-upon line and publish an explicit mapping/migration notice.

The binary version output should include, in stable machine-readable and human-readable forms:

- product SemVer;
- build/source revision where available;
- repository-format read/write range;
- object/schema range;
- JSON envelope version;
- release channel;
- dirty/unofficial build marker where appropriate.

Semantic Versioning requires a declared public API and identifies 0.y.z as initial development: https://semver.org/

## Release Channels

### Nightly/Development

- no compatibility or support promise;
- source revision is mandatory;
- must be visibly distinct from a release artifact;
- useful for dogfood and benchmark feedback.

### Alpha

- installer and artifact integrity are real;
- persistent formats may change with documented migration;
- supported deployment is local/test or explicitly stated;
- external pilots are expected.

### Beta

- public CLI/JSON/persistence compatibility policy exists;
- supported local and trusted-remote profiles are tested;
- backup maintainer and security responder exist;
- upgrade/rollback and recovery evidence exists;
- no known release-blocking security or license ambiguity.

### Stable 1.0

- public API and compatibility promises are precise;
- production support window and vulnerability policy are sustainable;
- multiple independent repositories and at least one independent integration pass conformance;
- migration from supported historical formats is tested;
- release provenance, SBOM, signature, recovery, docs, and support are routine rather than one-off.

## Distribution Options

### Option A: Source Build Only

Benefits:

- low artifact matrix cost;
- users build in their environment.

Costs:

- requires Rust toolchain and dependency access;
- weak onboarding;
- source authenticity and dependency reproducibility still need a solution.

### Option B: Signed Release Binaries

Benefits:

- fast installation;
- controlled tested artifact matrix;
- provenance and checksums can bind binary to source.

Costs:

- CI, signing, platform, libc, architecture, and support complexity;
- compromised release workflow has high impact.

### Option C: Package Registries and OS Packages

Benefits:

- familiar updates and discovery;
- integrates with ecosystem tooling.

Costs:

- registry account security and permanent release history;
- dependency between package cadence and support promises;
- multiple packaging systems multiply maintenance.

## Recommended Distribution Decision Candidate

decision-candidate: Start with signed release binaries plus a source archive, then add registries only after package ownership, metadata, names, and trusted publication are rehearsed. Do not make OS packages an initial stable-release blocker unless a target ICP requires them.

Initial artifact matrix should be selected from pilot evidence, not aspiration. At minimum record:

- OS, architecture, libc/runtime assumptions;
- checksum and file size;
- source revision and release tag;
- compiler/toolchain and build workflow identity;
- SBOM and license notice;
- signature/provenance bundle;
- install, verify, upgrade, rollback, and uninstall commands.

Cargo notes that published crate versions are permanent and recommends cargo publish --dry-run plus complete metadata: https://doc.rust-lang.org/cargo/reference/publishing.html

## Release Integrity

### Candidate Release Flow

1. Freeze a release commit with a clean tree.
2. Verify license, issue, docs, schema, version, and support declarations.
3. Run required source and compatibility gates in CI.
4. Build on an approved isolated build service from the release commit.
5. Produce checksums, SBOM, source archive, binaries, and provenance.
6. Sign artifacts using workload identity or protected release identity.
7. Independently verify signatures, provenance claims, checksums, binary version, and smoke behavior.
8. Require second-person approval for publication.
9. Publish tag, artifacts, curated changelog, known limitations, and upgrade/rollback instructions.
10. Verify public downloads from a clean environment.
11. Monitor reports and execute the rollback/advisory path if needed.

### Provenance and Signing

- decision-candidate: target SLSA-compatible build provenance without claiming a level until every requirement is audited.
- decision-candidate: use Sigstore/Cosign blob signing for binary/source artifacts or an equivalently reviewed mechanism.
- decision-candidate: publish an SPDX SBOM and machine-readable license notices.
- the product's HMAC commit/request signatures do not substitute for release-artifact signatures.

Primary sources:

- SLSA 1.2 provenance: https://slsa.dev/spec/v1.2/provenance
- Sigstore blob signing: https://docs.sigstore.dev/cosign/signing/signing_with_blobs/
- Sigstore verification: https://docs.sigstore.dev/cosign/verifying/verify/
- SPDX current specification: https://spdx.dev/use/specifications/

## Dependency Governance

Required decisions:

- allowed, reviewed, restricted, and prohibited licenses;
- direct-dependency admission criteria;
- dependency owner and update cadence;
- vulnerability source and severity policy;
- unmaintained/yanked dependency response;
- MSRV and compiler support;
- reproducible/offline build expectations;
- vendoring policy;
- cryptographic dependency review;
- transitive dependency and feature minimization;
- exception approval and expiry.

decision-candidate:

- keep Cargo.lock for the CLI/application build;
- add automated license/advisory checks and a human-readable policy;
- pin a supported Rust toolchain separately from MSRV;
- review security-critical dependencies and dangerous feature flags explicitly;
- block releases on unresolved known high/critical vulnerabilities unless a documented, expiring, reviewed non-applicability decision exists;
- use OpenSSF Scorecard and NIST SSDF as gap-assessment inputs, not score-chasing certifications.

Primary sources:

- NIST SSDF 1.1: https://csrc.nist.gov/pubs/sp/800/218/final
- OpenSSF Scorecard: https://scorecard.dev/
- OpenSSF Best Practices Badge: https://openssf.org/projects/best-practices-badge/

## Support and Deprecation Policy

Questions:

- How many product release lines receive security fixes?
- How long can old repository/object formats be read?
- When is write support removed?
- How are unsafe releases revoked or yanked?
- How long does the Python fallback remain installable and incident-supported?
- What makes a release end-of-life?

decision-candidate for early releases:

- alpha: best-effort current prerelease only, migration documented;
- beta: current minor plus previous minor for migration/security where capacity permits;
- stable: publish a time-bounded support policy only after staffing and CRA/legal review;
- retain read/migration support longer than write support where object integrity permits;
- never silently change the meaning of an existing schema version;
- publish deprecation in machine-readable capability output and release notes before removal.

## Release Gate Expansion

The existing Rust, installer, dogfood, and docs gates remain necessary. Add:

- license and third-party notice consistency;
- clean version and release-identity check;
- package dry-run and included-file inspection;
- supported platform artifact smoke;
- checksum, SBOM, signature, and provenance verification;
- repository/object/JSON/protocol compatibility fixtures;
- upgrade, rollback, backup, and restore rehearsal;
- security policy and supported-version update;
- known vulnerability and dependency exception review;
- public docs/examples applicable to the release;
- independent clean-environment install;
- no unresolved release-blocking privacy/security issue;
- two-person publication approval.

## Trade-Offs

- More artifacts improve adoption but multiply CI and support cost.
- Strong compatibility slows schema cleanup but protects automation and history.
- Long support windows build trust but require funded maintainers.
- Reproducible builds and high provenance levels improve assurance but consume engineering effort.
- Frequent releases shorten feedback loops but increase user upgrade burden.
- Registry publication improves discovery but is permanent and credential-sensitive.

## Falsification and Stop Conditions

- If external distribution history contradicts a version reset, preserve consumer compatibility.
- If a platform cannot be built, tested, and supported repeatedly, remove it from the official matrix rather than publish a best-effort artifact.
- If a release cannot be independently verified from public material, do not call it stable.
- If only one person can publish or recover credentials, do not advance to beta.
- If migrations cannot preserve object identity and sealed validation, stop the release and resolve the format contract.
- If dependency/license status is unknown, block registry publication.
- If release rehearsal repeatedly requires undocumented local state, treat the pipeline as unready.

## KPIs

| KPI | Candidate target |
|---|---|
| Published artifacts with checksum, SBOM, signature, provenance | 100 percent |
| Public artifact clean-install smoke | 100 percent of supported matrix |
| Release source/tag/version agreement | 100 percent |
| Supported upgrade fixture pass | 100 percent |
| Rollback/restore rehearsal pass | 100 percent where promised |
| Known unresolved high/critical vulnerability at release | 0, absent reviewed expiring exception |
| People able to publish and recover a release | at least 2 before beta |
| Emergency patch lead time | measure in alpha; set promise only after staffing |
| Accidental breaking changes within a schema major | 0 |
| Release steps requiring undocumented manual state | 0 |

## Experiments

### P3-REL-001: Release Identity Audit

- Search tags, distributed binaries, package registries, archived repositories, and known users.
- Record every version string and compatibility dependency.
- Decide whether reset, v0.6 alignment, or consumer-preserving continuation is valid.

### P3-REL-002: Non-Public Release Rehearsal

- Use a clean CI commit to build the initial target matrix.
- Produce checksum, SPDX SBOM, signature, and provenance bundle.
- Have a second operator verify and install without repository-local state.
- Record all undocumented steps and failures.

### P3-REL-003: Upgrade and Rollback Matrix

- Create fixtures for every claimed readable/writable repository and object version.
- Test old-to-new read, write, migration, failed migration recovery, and supported rollback.
- Exit: compatibility table matches observed behavior.

### P3-REL-004: Registry Dry Run

- Verify names, owners, metadata, package file list, size, license, readme, repository, and build.
- Use cargo publish --dry-run or equivalent without uploading.
- Exit: publication is reproducible and no generated/private/large unintended file is included.

### P3-REL-005: Compromised Release Tabletop

- Simulate leaked publishing credential, malicious artifact, lost signing identity, and bad migration.
- Exercise halt, revoke/yank, rotate, advisory, clean rebuild, and user communication.
- Exit: two operators can execute the response from documented procedures.

## Promotion Targets

- Version-axis and compatibility decision: release policy plus ADR where persistent/public meaning changes.
- Supported release matrix and channels: docs/development/strategy/release-strategy.md.
- Executable release flow: CI/release configuration and runbook.
- Support/deprecation promise: SUPPORT.md and SECURITY.md.
- Dependency/license policy: repository policy and automated checks.
- Historical version plans: archive after the canonical release roadmap exists.
