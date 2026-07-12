# Community, Standardization, and Documentation

Status: exploration

Last reviewed: 2026-07-11

## Decision Questions

1. What is open for community extension: implementation, protocol, schemas, policy profiles, extractors, or all of them?
2. When is CodeFire semantics stable enough for external compatibility and standardization?
3. Which existing standards should CodeFire emit or consume without losing its own meaning?
4. How are extensions named, reviewed, versioned, and kept interoperable?
5. How can users find product truth while preserving audit history and rich maintainer knowledge?
6. Which language is canonical for public specification and which documents should be deleted, archived, generated, or translated?

## Observed Baseline

- observed: CodeFire has named domain concepts, versioned objects, stable-ID goals, automation envelopes, examples, ADRs, and compatibility fixtures.
- observed: schemas currently contain sample YAML/JSON and layout examples rather than a complete normative schema registry with stable public identifiers.
- observed: no external conformance suite, extension registry, specification governance, or independent implementation is evidenced in the repository.
- observed: no CONTRIBUTING, code of conduct, maintainer policy, public roadmap, or community support policy exists.
- observed: tracked files include 332 Markdown documents; 288 tracked files are under docs/development.
- observed: current-cycle-burn-down-plan.md contains about 2,830 lines and mixes active and historical material.
- observed: a v0.2 integrated design exists in Markdown, PDF, and DOCX beside evolved product documents.
- observed: multiple source maps and mental models overlap and contain stale line-oriented claims.
- documented: docs/development/document-consolidation-plan.md defines target information layers, lifecycle states, consolidation batches, and a deletion gate.
- inferred: the repository has enough semantic material for an open specification experiment, but not enough independent use to justify formal standardization.

## Community Formation Options

### Option A: Maintainer-Only Development Until 1.0

Benefits:

- maximum product-direction speed;
- low moderation and contributor-support cost;
- unstable schemas can change privately.

Costs:

- bus factor remains one;
- no evidence that documentation and contribution process work externally;
- ecosystem trust and independent integrations arrive too late.

### Option B: Open Implementation Community First

Accept code, docs, test, extractor, and integration contributions while the core specification remains maintainer-governed and explicitly unstable.

Benefits:

- validates contribution and review systems early;
- increases language/repository coverage;
- avoids premature standards governance.

Costs:

- contributors may build on changing contracts;
- maintainer review and mentoring load rises.

### Option C: Open Specification Community First

Treat the specification and conformance suite as primary, with implementation as reference.

Benefits:

- vendor-neutral trust and multiple implementations are possible;
- aligns with de-facto-standard ambition.

Costs:

- specification process can freeze immature concepts;
- consensus cost may exceed user value;
- hard to attract spec contributors without deployed users.

### Option D: Foundation or Standards Body

Benefits:

- formal IPR, governance, and neutral ownership;
- procurement and ecosystem legitimacy.

Costs:

- premature overhead before multiple organizations depend on the work;
- can substitute process for product-market proof.

## Recommended Community Decision Candidate

decision-candidate: Start with Option B, publish the implementation contracts needed for integrations, and grow toward Option C only after independent consumers appear.

Community launch prerequisites:

- unambiguous license and inbound terms;
- code of conduct and moderation owner;
- security reporting path;
- contributor setup and narrow first-change path;
- public roadmap boundaries and non-goals;
- review expectations and response-capacity statement;
- labels for experimental versus stable contracts;
- a policy for AI-assisted contribution accountability.

## Community Health Questions

- Can an outsider find a useful issue without reading hundreds of internal issue files?
- Can they run a focused test without the founding maintainer's environment?
- Does review explain rejection against a public standard rather than private taste?
- Are docs, design, testing, security, and community contributions valued alongside code?
- Who moderates abuse and resolves conduct conflicts?
- Can maintainers say no without silently ignoring contributors?
- Are contributor metrics collected ethically and minimally?

decision-candidate: Measure contributor journey and review sustainability, not star count. Avoid gamifying commit volume when CodeFire's own philosophy values trusted integration over patch volume.

## Specification Scope Options

### Monolithic CodeFire Specification

One specification covers repository, objects, CLI, remote, policies, extractors, and evidence.

Risk: every change becomes globally coupled and independent implementation is prohibitively expensive.

### Layered Specifications

Candidate layers:

1. Core identity and canonical object model.
2. Repository and sealed-history format.
3. Trace, Fire, resolution, verification, and certificate semantics.
4. Automation envelope and diagnostic contract.
5. Remote transport/protocol.
6. Configuration and extractor profiles.
7. Interchange adapters and extension registry.

Benefit: implementers can adopt stable lower layers without supporting every CLI feature.

Risk: version negotiation and cross-layer compatibility become explicit work.

## Recommended Specification Decision Candidate

decision-candidate: Use layered specifications with explicit maturity per layer. Stabilize identity/canonicalization and automation contracts before extractor breadth or hosted protocol behavior. Publish examples only as examples; normative requirements must have schemas, prose semantics, and conformance tests.

Each public schema/specification should state:

- stable identifier and version;
- normative versus informative content;
- required and optional fields;
- canonicalization and identity implications;
- unknown-field and extension behavior;
- compatibility and deprecation rules;
- security/privacy considerations;
- conformance fixtures and expected failures;
- owner, change process, and review date.

## Extension Governance

Questions:

- Can third parties define Atom kinds, link types, evidence types, diagnostics, or policy fields?
- How are namespace collisions prevented?
- Must extensions preserve unknown fields?
- Can an extension affect commit eligibility or object identity?
- What capability negotiation is required?

decision-candidate:

- reserve a CodeFire-owned namespace for normative fields;
- use organization/URI-based namespaces for third-party extensions;
- require extensions that affect identity or blocking semantics to declare that fact;
- reject unknown critical extensions and preserve or safely ignore explicitly non-critical ones;
- keep a registry of stable and experimental extensions;
- require conformance examples and migration before promotion.

Technical encoding belongs in Phase 1/2; Phase 3 owns the approval, ownership, IPR, and lifecycle rules.

## Interoperability Candidates

### JSON Schema

Use for public JSON shapes, examples, validation, and stable identifiers where it accurately represents the contract.

Primary source: https://json-schema.org/specification

### SARIF

Experiment with exporting Fires or diagnostics to security/code-analysis ecosystems.

Potential loss:

- Fire resolution basis, trace path, sealed history, policy profile, and certificate semantics may not fit ordinary analysis results.

Primary source: https://www.oasis-open.org/standard/sarif-v2-1-0/

### in-toto Attestation

Experiment with expressing or referencing build/evidence/certificate claims as attestations.

Potential loss:

- CodeFire's ongoing branch/open-state and cross-artifact responsibility model is broader than a supply-chain statement.

Primary source: https://in-toto.io/docs/specs/

### SPDX

Use for release SBOM, licenses, provenance relationships, and third-party notices.

Potential loss:

- SPDX is not a replacement for CodeFire Trace Links or Fire resolution semantics.

Primary source: https://spdx.dev/use/specifications/

## Standardization Timing Options

### Early Formal Standardization

Benefits: visibility, neutral process, early ecosystem signal.

Risks: freezes weak semantics, consumes maintainer capacity, and lacks implementer feedback.

### De-Facto Open Specification First

Publish versioned schemas, prose, conformance tests, and reference implementation; seek formal venue only after adoption.

Benefits: evidence-driven evolution, lower overhead, easier experimentation.

Risks: founding maintainer retains more control; external users may fear lock-in.

## Recommended Standardization Decision Candidate

decision-candidate: Pursue a de-facto open specification first. Consider a neutral foundation or formal standards venue only when all are true:

1. at least three independent organizations use stable CodeFire contracts;
2. at least two independent integrations or one independent implementation pass conformance;
3. the relevant layer has completed two compatible release cycles;
4. IPR, trademark, governance, and change-control policies are public;
5. maintainers can support the process without delaying security and product quality.

## Documentation Audiences

| Audience | Needed truth | Must not be forced to read |
|---|---|---|
| Evaluator | value, non-goals, install, first seal, limitations | internal issue history |
| Developer user | workflow, policy, diagnostics, recovery | source anatomy |
| Reviewer/release owner | certificate, evidence, review, support | implementation work logs |
| Automation author | schemas, capabilities, exit codes, compatibility | human tutorials only |
| Remote operator | deployment, trust boundary, backup, incident, upgrade | product brainstorming |
| Contributor | architecture, issue, tests, review, governance | obsolete version plans |
| Maintainer | source maps, decisions, current state, release process | duplicate generated documents |
| Auditor/researcher | historical decisions and evidence | mutable current-state claims |

## Documentation Information Architecture

The existing consolidation plan is the execution source. Phase 3 adds user/community requirements.

Candidate layers:

- Root README: stable value, status, install/first outcome, support boundary, links.
- Public user guide: concepts, tutorials, persona workflows, policy, migration, troubleshooting.
- Public reference: CLI, JSON, schema, format, protocol, compatibility.
- Operator guide: deployment, security, backup, restore, incident, upgrade.
- Contributor guide: setup, architecture, tests, governance, security, release.
- Development control: current project state, active roadmap, issues, handoff.
- Strategy: accepted product, adoption, operations, community, release direction.
- Exploration: alternatives and hypotheses, never current truth.
- Audit/archive: closed plans, historical issues, old design snapshots.

## Lifecycle Metadata

Every surviving development document should inherit or declare:

- lifecycle state: canonical, supporting, exploration, historical, generated, or superseded;
- audience;
- owner;
- update trigger;
- last verified date where claims can stale;
- canonical replacement or promotion target;
- generated source if not hand-authored.

## Consolidation Candidates

### Legacy Product Material

- preserve one text-searchable v0.2 snapshot only if it has unique historical value;
- remove duplicate PDF/DOCX from the active source tree unless generated as release artifacts;
- remove or archive IMPLEMENTATION_TODO.md and completed MVP plans after unique requirements/evidence are mapped;
- reconcile docs/12 and docs/13 with current requirements, or mark them historical rather than leaving v0.2 claims in the product-truth range.

### Version and Cycle Plans

- replace future-tense completed v0.6-v1.0 plans with one canonical version/format roadmap;
- move closed plans to archive;
- reduce current-cycle-burn-down-plan.md to the active bounded batch;
- keep detailed issue evidence in ledgers and Git history, not duplicated active prose.

### Source Documentation

- consolidate overlapping map, blueprint, mental-model, anatomy, atlas, executable-model, and reconstruction documents;
- prefer symbol-checked generated maps over fixed line numbers;
- keep one architecture map, one command trace, one data catalog, and one change playbook as proposed by the consolidation plan.

## Language and Translation

Options:

1. Japanese canonical, English summary.
2. English canonical, Japanese contributor/user translation.
3. Two normative languages.

decision-candidate: If global de-facto adoption remains the goal, use one English canonical public specification and user/reference set, with Japanese translation or internal contributor guidance. Reject two independently normative languages because semantic drift would be difficult to review.

Translation policy should define:

- canonical marker and source link;
- version/last-synced metadata;
- behavior when translation lags;
- non-translated machine identifiers;
- reviewer for high-impact legal/security text.

## Documentation Quality Gates

- Markdown internal and external link check;
- orphan document detection;
- duplicate/current-state claim detection;
- lifecycle metadata validation;
- command/help/completion/reference consistency;
- JSON/YAML example schema validation;
- executable snippet tests where safe;
- source-symbol freshness checks;
- version/support/language consistency;
- generated-output reproducibility;
- deletion manifest and inbound-link check.

## Trade-Offs

- Open contracts increase adoption trust but reduce freedom to change schemas.
- Extension flexibility encourages ecosystem growth but can fragment semantics.
- Formal standardization provides neutrality but imposes heavy IPR/governance process.
- Rich historical documentation improves auditability but harms findability when mixed with active truth.
- Deleting rendered artifacts reduces clutter but removes convenient offline formats unless release generation exists.
- English canonical docs increase global reach but create translation and maintainer burden.
- Automated docs improve freshness but can hide important design rationale if generation replaces prose indiscriminately.

## Falsification and Stop Conditions

- If external consumers cannot implement a layer from the specification and fixtures, do not label it stable.
- If SARIF/in-toto mappings lose decision-critical meaning, keep them optional exports rather than canonical storage.
- If no independent consumer appears after two stable contract cycles, defer formal standardization.
- If extension proposals repeatedly require core-invariant changes, keep extension points closed until semantics mature.
- If docs consolidation removes unique requirement or decision evidence, restore from Git and fix the deletion inventory.
- If translation cannot be kept current, publish a clearly labeled summary rather than a stale normative copy.
- If community review load prevents security and release work, narrow contribution scope and publish capacity limits.

## KPIs

| KPI | Candidate target |
|---|---|
| Independent schema/protocol consumers | at least 2 before formal standardization discussion |
| Independent implementation or conformance adapter | at least 1 before 1.0 standard claim |
| Conformance suite pass rate for stable layers | 100 percent |
| Breaking change within a declared schema major | 0 |
| External contributors completing first accepted change | measure funnel; at least 3 before beta community claim |
| Median contributor first-response time | under 3 business days during active intake |
| Broken internal links | 0 |
| Conflicting current-state claims | 0 |
| Surviving docs without lifecycle owner/update rule | 0 |
| Public persona tasks answered within a 10-minute findability test | at least 80 percent |
| Stale translation presented as current | 0 |

## Experiments

### P3-COMM-001: Independent Contribution Trial

- Recruit one contributor unfamiliar with internal history.
- Ask them to find, implement, test, document, and submit a bounded change using only public contributor material.
- Measure setup time, review ambiguity, and maintainer effort.

### P3-SPEC-001: Clean-Room Consumer

- Give an engineer the candidate automation/object layer spec and fixtures, not implementation source navigation.
- Ask them to implement a validator or reader.
- Record every ambiguity and hidden implementation dependency.
- Exit: the consumer passes conformance without private clarification.

### P3-STD-001: Adapter Lossiness Study

- Map representative Fires/diagnostics to SARIF and Evidence/Certificate to in-toto.
- Round-trip or compare all decision-critical fields.
- Classify exact, extended, lossy, and impossible mappings.
- Exit: optional adapter scope is honest and tested.

### P3-DOC-001: Document Inventory and Deletion Rehearsal

- Generate title, lines, lifecycle, owner, inbound links, and replacement for every document.
- Select one low-risk superseded group and rehearse deletion on a review branch/worktree.
- Run all link/current-state checks and recover one item from Git.

### P3-DOC-002: Persona Findability Test

- Give evaluator, user, automation author, operator, and contributor five common questions each.
- Measure time, path, wrong-source use, and confidence.
- Use failures to restructure hubs before writing more prose.

## Promotion Targets

- Community governance and contribution process: root governance/contribution files.
- Stable specification layers and extension rules: product specification plus ADRs.
- Interoperability adapters: Phase 2 implementation plans after lossiness evidence.
- Standardization strategy: docs/development/strategy/ecosystem-strategy.md.
- Documentation architecture: docs/development/document-consolidation-plan.md execution and directory hubs.
- Translation policy: public docs governance.
- Historical material: docs/development/archive/ with deletion manifest and Git references.
