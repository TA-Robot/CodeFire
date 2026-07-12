# Operations, Privacy, and Compliance

Status: exploration

Last reviewed: 2026-07-11

## Legal Disclaimer

This document is not legal, privacy, compliance, or security certification advice. Applicability depends on where and how CodeFire is developed, distributed, operated, monetized, and used. Qualified professionals must review obligations for the relevant organization and jurisdictions.

## Decision Questions

1. Which CodeFire deployment profiles are officially supported?
2. Is the current remote a development tool, trusted single-tenant service, or production multi-tenant service?
3. What data can enter sealed objects, active state, logs, audit records, and release infrastructure?
4. What can be deleted, redacted, encrypted, revoked, or recovered after replication?
5. What backup, restore, incident, vulnerability, support, and end-of-life promises can maintainers actually meet?
6. What regulatory role might the project or a commercial distributor occupy?

## Observed Operational Boundary

- observed: core workflows are local-first and can operate without a remote.
- observed: file-backed, HTTP, and HTTPS remotes exist with permissions, branch protection, token checks, HMAC commit/request signatures, nonce replay checks, retention, and audit records.
- documented: server-side verification executes configured commands with bounded cwd, environment, and timeout.
- documented: server-side verification does not have OS-level sandbox or container isolation.
- observed: identity and permissions are file-backed rather than designed as a multi-tenant organization/identity plane.
- observed: local and remote object storage, GC, doctor, migration, and storage reports exist.
- inferred: the safe current deployment boundary is a trusted administrative domain, not an untrusted public build service.

## Supported Deployment Options

### Option A: Local CLI Only

Support promise:

- one user or trusted team controls filesystem and verification commands;
- no network service is production-supported;
- remotes are experimental or development-only.

Benefits:

- smallest threat and support surface;
- strongest local privacy posture;
- fastest route to a credible initial release.

Costs:

- weak collaboration and distribution story;
- harder organization-wide adoption;
- no central release-owner enforcement.

### Option B: Local Plus Trusted Single-Tenant Remote

Support promise:

- one organization controls server policy, projects, verification commands, keys, storage, and operators;
- untrusted tenants do not share a process or security boundary;
- operator follows a documented backup, upgrade, and hardening profile.

Benefits:

- matches the current implementation more closely;
- enables team collaboration and remote policy;
- keeps identity and sandbox requirements bounded.

Costs:

- requires operational documentation and incident ownership;
- users may still mistake it for a public multi-tenant service;
- self-hosting creates support burden.

### Option C: Multi-Tenant or Public Hosted Service

Additional requirements include:

- strong tenant and process isolation;
- sandboxed verification execution;
- organization identity, RBAC, audit, quota, rate limit, and abuse prevention;
- managed secret and key storage;
- network perimeter, patching, monitoring, availability, backup, and disaster recovery;
- privacy notices, processor/controller responsibilities, contracts, and support staffing;
- billing, deletion, export, and account lifecycle.

Benefits:

- lowest user setup cost;
- easier ecosystem integration and operational metrics;
- possible recurring revenue.

Costs:

- a different operational and regulatory product class;
- significantly larger security blast radius;
- conflicts with the current no-hosted-service product boundary unless separated.

## Recommended Deployment Decision Candidate

decision-candidate: Support local CLI plus a trusted single-tenant remote profile. Explicitly mark public or multi-tenant hosting unsupported until a separate threat model, sandbox, identity plane, operator organization, and product-boundary decision exist.

The remote documentation should state:

- who may define verification commands;
- that verification commands execute with the service's OS privileges unless external isolation is provided;
- which ports and TLS modes are appropriate;
- how keys/tokens are provisioned and rotated;
- what is logged and retained;
- that insecure TLS mode is local/test only;
- that one service process is not a tenant-isolation boundary;
- how to back up and restore before upgrades.

## Operational Readiness Model

### Tier 0: Developer Preview

- no availability promise;
- source build permitted;
- manual recovery may be required;
- only test data recommended.

### Tier 1: Supported Local

- signed release artifact or reproducible source package;
- supported OS/architecture list;
- installer, upgrade, downgrade, and uninstall procedure;
- local backup and restore drill;
- SECURITY.md and supported-version policy.

### Tier 2: Supported Trusted Remote

- reference deployment and hardening guide;
- backup/restore, capacity, log, retention, key rotation, and incident runbooks;
- compatibility-tested upgrade and rollback;
- internal SLO measurement;
- trusted single-tenant limitation.

### Tier 3: Hosted/Multi-Tenant

- separate approval; not an automatic continuation of Tier 2;
- service-level security/privacy architecture and staffed operations required.

## SRE Questions and Candidate Internal SLOs

External SLOs must not be published until repeated measurements and staffing show they are sustainable. Initial internal experiment targets:

| Area | Measurement | Initial target |
|---|---|---|
| Durability | accepted sealed head references missing/corrupt object after normal operation | 0 |
| Restore | quarterly restore drill from documented backup | 100 percent success |
| RPO | maximum sealed remote history lost in tested failure | 0 accepted heads; explicit treatment of unsealed work |
| RTO | trusted remote recovery in rehearsal | under 4 hours for initial small deployment |
| Upgrade | supported-version fixture upgrade | 100 percent in release matrix |
| Rollback | rollback without object/schema corruption | 100 percent where declared supported |
| Security triage | time to classify a credible private report | internal target under 2 business days |
| Capacity | storage/scan/verify p95 within documented scale envelope | Phase 1 budgets |

## Backup and Recovery Boundaries

CodeFire separates formal sealed history from unsealed filesystem work. The backup policy must therefore distinguish:

- repository .codefire objects and branch heads;
- remote project state and audit/retention metadata;
- open directories containing unsealed user work;
- active metadata that intentionally cannot reconstruct working files;
- signing and request keys, which should not be copied into ordinary repository backups;
- release and registry credentials;
- external evidence artifacts referenced by hash or URI.

decision-candidate:

- document a consistent snapshot procedure for repository and remote metadata;
- require restore validation through doctor plus sealed-graph validation;
- treat open-directory backup as user/organization responsibility and explain that it is not formal CodeFire history;
- never imply that CodeFire can restore unsealed working files when the product invariant says it cannot;
- test credential recovery separately from data restore.

## Incident Taxonomy

| Severity | Example | Required response concept |
|---|---|---|
| SEV-0 | release signing or registry compromise; malicious artifact distributed | halt distribution, revoke/rotate, publish verified recovery path |
| SEV-1 | sealed-head corruption, remote authorization bypass, arbitrary untrusted command execution | contain, preserve evidence, patch, assess affected versions |
| SEV-2 | evidence leaks secret/PII, data loss limited to one project, serious availability failure | notify owner, remove reachability where possible, rotate secrets, document limits |
| SEV-3 | recoverable command defect or degraded performance | normal issue and release process |

The security policy must define private intake, embargo, reporter communication, advisory/CVE decision, release coordination, and post-incident learning. It must not promise response times that one maintainer cannot sustain.

## Data Inventory

Candidate data classes requiring explicit classification:

| Data | Possible sensitivity | Current/likely location | Required decision |
|---|---|---|---|
| source and documentation content | proprietary, personal, regulated | content manifests and blobs | access, encryption, retention, export |
| actor/signer names | personal or organizational identity | commits, audit, signatures | purpose, minimization, correction |
| command stdout/stderr | secrets, personal data, internal paths | evidence objects | preflight, redaction, retention |
| argv and command text | secrets in flags, confidential tooling | evidence/verification | prohibit secret args, redact patterns |
| artifact paths and URIs | usernames, infrastructure, signed URLs | artifact refs | default redaction and URI policy |
| audit and request metadata | actor, IP/network, operation history | server audit | retention, access, incident use |
| DCO sign-off | name/email and durable public statement | Git commits | privacy notice and purpose |
| tokens and signing keys | credentials | policy/env/secret store | never seal; storage and rotation |
| external evidence | may outlive source retention | outside object store | ownership, availability, deletion |
| optional usage telemetry | behavior and repository metadata | not currently defined | no collection by default |

## Privacy Options

### Option A: Responsibility Disclaimer Only

Tell users not to store secrets and make them fully responsible.

Benefit: minimal implementation.

Risk: inconsistent with a product whose value is safe, evidence-backed automation; users will capture sensitive output accidentally.

### Option B: Privacy by Default

- no outbound telemetry by default;
- bounded capture plus sensitive-pattern preflight;
- explicit confirmation for risky evidence;
- external-reference mode for sensitive or large evidence;
- data inventory and retention controls;
- export, reachability removal, key rotation, and incident playbooks;
- documentation of irreversibility after replication.

Benefit: aligns with local-first trust and reduces user burden.

Cost: false positives, implementation complexity, and no perfect secret detector.

### Option C: Compliance Product Profiles

Provide predefined retention, audit, identity, encryption, and attestation profiles for regulated use.

Benefit: stronger high-assurance market value.

Risk: creates costly legal/assurance claims and should follow, not precede, a privacy-safe baseline.

## Recommended Privacy Decision Candidate

decision-candidate: Adopt Option B as baseline. Do not claim compliance profiles until independently assessed.

Specific candidate rules:

1. No network telemetry unless a user explicitly enables a versioned, documented event schema.
2. Never read or transmit source content for telemetry.
3. Reject obvious private keys, credential files, tokens in command arguments, and known secret formats during evidence capture by default.
4. Allow an explicit override only with a persistent warning and policy control.
5. Prefer artifact hash plus external URI over inline sensitive evidence.
6. Record exactly what was truncated or redacted without storing the removed value.
7. Treat sealed/replicated data as potentially non-erasable; document reachability deletion versus copy revocation honestly.
8. Design an emergency secret-compromise workflow around immediate credential rotation, not the assumption that object deletion retracts exposure.

## Compliance Questions

### GDPR and Similar Privacy Regimes

Questions for professional assessment:

- Who is controller or processor for local, self-hosted, and any future hosted use?
- What lawful basis and purpose apply to actor identity, evidence, and audit data?
- What retention is necessary?
- How are access, correction, export, restriction, and erasure requests handled?
- Can immutable objects be designed to avoid personal data rather than rely on deletion?

Primary source: https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32016R0679

### EU Cyber Resilience Act

Questions for professional assessment:

- Is CodeFire supplied in the course of commercial activity?
- Is an organization acting as manufacturer, open-source steward, distributor, or only non-commercial contributor?
- What support period, vulnerability handling, technical documentation, SBOM, incident, and update obligations apply?
- Does a separate hosted remote change the classification?

Primary source: https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32024R2847

### Secure Development Evidence

decision-candidate: Map project practices to NIST SSDF as a gap-assessment vocabulary, not as a certification claim.

Primary source: https://csrc.nist.gov/pubs/sp/800/218/final

## Trade-Offs

- Privacy-safe capture reduces evidence completeness in some debugging cases.
- External evidence references reduce sealed-history size but introduce availability and retention dependencies.
- Single-tenant support limits adoption but avoids unsupported isolation claims.
- Strong backup and incident processes consume maintainer time before visible features.
- Data deletion can conflict with content-addressed integrity and replicated audit history.
- Compliance-oriented documentation can help procurement but creates risk if it overstates assurance.

## Falsification and Scope-Change Conditions

- If qualified pilots require hosted multi-tenancy to participate, do not silently expand serve; create a separate service proposal and threat model.
- If sensitive-capture prevention has unacceptable false positives, narrow default capture rather than disable privacy protections globally.
- If external evidence cannot meet availability requirements, define a supported retention contract or disallow it for blocking proof.
- If full erasure cannot be reconciled with a target data class, prohibit that data class from sealed objects.
- If the project lacks two security responders, avoid public response-time commitments and stable remote claims.

## KPIs

| KPI | Candidate target |
|---|---|
| Sensitive evidence incidents | 0 |
| Secret-pattern regression coverage | 100 percent of agreed corpus |
| Undocumented outbound telemetry events | 0 |
| Data fields in inventory | 100 percent of persisted/public fields |
| Restore drill success | 100 percent per quarter before remote stable |
| Supported upgrade fixture success | 100 percent |
| Critical private report acknowledgment | measured first; target within 2 business days after staffing |
| Remote deployments explicitly classified by profile | 100 percent of official examples |
| Unsupported multi-tenant claims | 0 |
| Compliance statements linked to evidence and review date | 100 percent |

## Experiments

### P3-OPS-001: Backup and Restore Drill

- Create sealed history, open unsealed work, remote state, audit records, and external evidence.
- Back up according to draft instructions, destroy the test environment, and restore with a different operator.
- Validate with doctor, branch/object checks, and application-level commands.
- Record exactly which unsealed state is not recoverable.

### P3-OPS-002: Remote Trust-Boundary Tabletop

- Model malicious project config, malicious verification command, compromised operator, token leak, replay, tenant collision, and insecure TLS misuse.
- Separate mitigated, administratively trusted, and unsupported cases.
- Exit: every official deployment example stays inside the supported boundary.

### P3-PRIV-001: Sensitive Evidence Red Team

- Build a corpus containing API tokens, private keys, passwords in argv, signed URLs, usernames, personal identifiers, binary secrets, and benign lookalikes.
- Exercise command, argv, artifact, URI, batch, truncation, JSON, and remote paths.
- Measure false negative and false positive rates by class.

### P3-PRIV-002: Erasure and Secret-Compromise Drill

- Insert a synthetic secret and synthetic personal identifier into each persistent surface.
- Enumerate local, remote, backup, log, audit, package, and cloned copies.
- Practice rotation, reachability removal, notification, and documentation.
- Exit: no claim says deleted when a copy remains.

### P3-COMP-001: Regulatory Role Workshop

- With qualified counsel or compliance owner, compare non-commercial OSS, commercial OSS, steward, distributor, and hosted-service scenarios.
- Record the facts that change classification and the decisions that trigger re-review.
- Do not promote a compliance claim from this exploration alone.

## Promotion Targets

- Supported deployment profiles: docs/development/strategy/operations-strategy.md and docs/runbook.md.
- Security intake and support versions: SECURITY.md.
- Data inventory and capture policy: product security/privacy design plus relevant CLI specification.
- Product-boundary expansion to hosted or multi-tenant: ADR and product definition.
- External service SLOs: separate service operations policy after evidence.
- Regulatory conclusions: reviewed compliance record outside raw exploration, with owner and review date.
