# UX, Education, Adoption, and Economics

Status: exploration

Last reviewed: 2026-07-11

## Decision Questions

1. Can a qualified user reach a meaningful sealed commit before the conceptual cost overwhelms the value?
2. How can strict consistency be introduced progressively without creating a weaker state that looks fully assured?
3. How does an existing team trial CodeFire when Git coexistence is a product non-goal?
4. What education is required for developers, reviewers, release owners, automation authors, and remote operators?
5. Does the total cost of a trusted change decrease enough to justify workflow, compute, storage, and support costs?

## Observed Baseline

- observed: the README Quick Start focuses on building, installing, and completion, not a complete first-seal outcome.
- observed: the CLI exposes more than thirty top-level or grouped capabilities.
- documented: Git concepts including checkout, staging, stash, WIP commit, partial commit, pull, fetch, rebase, and force upload are intentionally absent.
- documented: open directories, sealed heads, Fires, resolution basis, Evidence, and certificates introduce a new mental model.
- observed: docs/13_risks_and_non_goals.md already identifies Fire fatigue, heavy commits, OS-copy pseudo-checkpoints, and Git-cultural friction.
- observed: public English and internal Japanese prose are mixed, and no canonical localization policy is declared.
- observed: current success metrics focus on internal consistency, automation contract, and diagnostics rather than activation, retention, task success, or return on investment.
- inferred: the strict model is valuable only when users understand which risk it removes and can complete common work without ritualized evidence.

## User Roles and Required Outcomes

| Role | First useful outcome | Later competence |
|---|---|---|
| Developer | make one small change and reach a seal | split work into consistency-sized changes; resolve Fires with meaningful basis |
| Reviewer | see why a sealed change was accepted | challenge evidence, policy, stale basis, and scope |
| Release Owner | accept only a valid sealed head | control remote policy, signatures, support, and rollback |
| Automation Author | consume JSON without text scraping | use capability discovery, dry-run, idempotency, and schema negotiation |
| Repository Maintainer | diagnose layout/object health | migrate, restore, manage storage, and recover incidents |
| Framework Author | define artifacts and Trace Links | maintain extractors, policy profiles, schemas, and compatibility |
| Executive/Buyer | understand avoided risk and cost | assess rollout, support, compliance, and unit economics |

## First-Use Journey

The target journey should be outcome-oriented:

1. Identify a supported project profile.
2. Install an authenticated CodeFire release.
3. Initialize a disposable or net-new repository.
4. Understand repository, branch, open directory, and sealed commit in one diagram.
5. Apply a minimal artifact/Trace policy.
6. Make one intentionally linked change.
7. Run scan and inspect one Fire.
8. Resolve it with decision-specific rationale or evidence.
9. Run verify and understand every zero/blocking count.
10. Create and inspect a seal.
11. Understand what is not backed up or recoverable.
12. Know where to ask for help and report a security problem.

## Onboarding Options

### Option A: Full Strict Model Immediately

Benefits:

- no ambiguity about assurance;
- every tutorial reflects the final workflow;
- no profile migration later.

Costs:

- high Time to First Seal;
- early users may fail on policy configuration before seeing value;
- difficult to isolate whether rejection is conceptual or implementation friction.

### Option B: Progressive Policy Profiles

Candidate profiles:

- starter: small supported artifact set, explicit warnings about coverage, same sealed-history invariant;
- team: required links, external verification, shared evidence and review;
- high-assurance: strict required links, signatures, remote policy, retention, and reviewed operational profile.

Benefits:

- reduces initial configuration burden;
- lets teams add assurance after observing value;
- supports education by capability layer.

Risks:

- starter can be mistaken for complete assurance;
- profile upgrades need migration and comparison;
- configuration combinatorics can increase support cost.

### Option C: Guided Generator Without Named Assurance Profiles

Ask questions and generate explicit config, while reporting exactly which guarantees are enabled and absent.

Benefits:

- avoids marketing labels that imply certification;
- creates a transparent starting configuration.

Risks:

- interactive setup can be hard to automate;
- generated config still needs education and maintenance.

## Recommended Onboarding Decision Candidate

decision-candidate: Combine a small starter profile with an explicit assurance report. Preserve branch/seal invariants, but reduce the initial artifact and policy surface. Every status/verify result should say what was checked, what was not configured, and which profile/schema produced the result.

The starter profile must never imply:

- all repository semantics were modeled;
- all requirements have tests;
- policy consistency proves correctness or security;
- untracked artifacts are safe to ignore;
- local open work is backed up by sealed history.

## Git and Migration Education

The product should explain differences rather than invent false equivalents.

| Familiar Git concept | CodeFire answer |
|---|---|
| branch checkout | branch materializes into an explicit open directory |
| current branch | intentionally absent |
| index/staging | intentionally absent; change unit is a whole consistent state |
| WIP commit/stash | intentionally absent from formal history |
| commit | policy-consistent sealed state with evidence and certificate |
| merge | sealed source integration plus explicit conflict/impact Fires |
| force push/rewrite | intentionally absent |
| CI status | part of verification evidence, not the whole certificate |

decision-candidate: Provide an external, read-only or one-way trial adapter rather than transparent coexistence in core. The adapter must label which system is authoritative, which snapshot was assessed, and whether a real CodeFire seal exists.

## Education Architecture

Recommended learning layers:

1. Five-minute concept page: problem, seal, Fire, evidence, non-goals.
2. Fifteen-minute first-seal tutorial.
3. Persona paths for developer, reviewer, automation, and operator.
4. Error and recovery cookbook based on stable diagnostics.
5. Policy design guide with good and bad trace examples.
6. Migration laboratory using a disposable project.
7. High-assurance deployment and evidence guide.
8. Reference for every CLI/JSON/schema contract.
9. Maintainer and contributor development guide.

Documentation examples should include negative outcomes, not only a successful golden path.

## Accessibility and Internationalization

Questions:

- Is public documentation English-first, Japanese-first, or equally maintained?
- Which CLI text is stable enough to translate?
- Are diagnostics understandable without color, emoji, interactive TTY, or wide tables?
- Can all actions be copied and invoked non-interactively?
- Are code examples and schemas locale-independent?

decision-candidate:

- make English the canonical public product/specification language if global de-facto adoption is the goal;
- retain Japanese contributor guidance or reviewed translation where it materially improves development;
- never maintain two independent normative specifications;
- keep JSON keys, schema identifiers, exit codes, and machine timestamps locale-independent;
- test NO_COLOR-like behavior, non-TTY output, narrow terminals, and screen-reader-friendly text structure.

If the first target market is intentionally Japan-only for a bounded period, Japanese may remain canonical, but the decision and globalization trigger should be explicit.

## Organizational Adoption Model

### Pilot Stage

- one bounded project;
- one champion and one skeptical reviewer;
- baseline incident/review/workflow measurements;
- no claim of organization-wide replacement;
- weekly Fire and evidence-quality review.

### Team Stage

- shared policy owner;
- reviewer and release-owner training;
- supported remote profile if needed;
- backup/restore and security contact;
- documented exceptions and migration path.

### Organization Stage

- policy governance and template ownership;
- compatibility and support windows;
- integration with identity, CI, artifact, and compliance systems through external boundaries;
- centralized metrics that do not leak source or personal data;
- procurement, support, and incident accountability.

decision-candidate: Do not scale from pilot to team based only on tests passing. Require measured value, acceptable workflow tax, meaningful evidence, trained backup operators, and a recovery drill.

## Economic Model

The principal unit should be total cost to a trusted integrated change, not patch generation speed.

Candidate cost equation:

~~~text
Trusted Change Cost
  = developer active minutes
  + reviewer active minutes
  + Fire resolution and evidence minutes
  + AI/model cost
  + CI and verification compute
  + storage and transfer cost
  + failure, rework, and incident allocation
  + onboarding and support allocation
~~~

Candidate benefit equation:

~~~text
Trusted Change Benefit
  = avoided escaped inconsistency
  + reduced review reconstruction
  + reduced incident investigation
  + reusable release/compliance evidence
  + lower automation integration failure
  + lower recovery uncertainty
~~~

## Sustainability Options

### Option A: Volunteer/Donation OSS

Low user friction but vulnerable to maintainer burnout and unfunded security/support obligations.

### Option B: Paid Support and Training

Preserves an OSS core and monetizes adoption knowledge. Revenue scales with human service unless training becomes reusable.

### Option C: Enterprise Integration and Assurance Packs

Organizations pay for deployment profiles, support, migration, compliance evidence, or certified integrations. Risk: product claims and support obligations expand quickly.

### Option D: Separate Managed Service

Recurring revenue and easy adoption, but requires the separate hosted-service boundary described in 03-operations-privacy-and-compliance.md.

### Option E: Foundation or Consortium

Supports neutral specification stewardship after real multi-organization adoption. Premature before contributors and funding exist.

### Option F: Open Core

Can fund development, but deciding which trust/security functions are proprietary can damage adoption and contradict secure-by-default goals.

## Recommended Economic Decision Candidate

decision-candidate: Do not select a permanent monetization model before measuring pilot unit economics. Keep the candidate core/specification permissive OSS, test paid support/training and enterprise integration demand, and treat any managed service as a separate business and threat model.

Security fixes, safe defaults, artifact verification, and data export should not be artificially withheld to create a paid tier.

## Trade-Offs

- Faster onboarding can reduce assurance coverage if profile boundaries are unclear.
- More tutorials improve success but add stale-documentation risk.
- External adapters reduce migration cost but add dual-state complexity.
- Internationalization expands reach but doubles review work if canonical ownership is unclear.
- Rich evidence improves auditability but raises human, storage, and privacy cost.
- Strict workflows can reduce escaped inconsistency while increasing local cycle time.
- Paid support funds sustainability but can incentivize complexity.

## Falsification and Pivot Conditions

- If median first-seal time stays above 30 minutes after three UX iterations, reduce setup and supported scope.
- If users cannot explain policy-consistent versus correct after onboarding, change claims and UI before adding features.
- If more than 25 percent of sampled resolutions are boilerplate, revise policy precision and training.
- If workflow tax stays above 20 percent after two weeks with no measurable risk reduction, narrow the ICP or pivot toward a verifier.
- If more than half of pilot teams retain only the bridge, evaluate whether the bridge is the real product rather than forcing migration.
- If support exceeds four maintainer hours per active team per month after onboarding, simplify product/profile/support scope.
- If no target buyer expresses willingness to pay for any support/integration model, plan for a bounded volunteer sustainability model rather than implicit commercial obligations.

## KPIs

| KPI | Definition | Initial experiment threshold |
|---|---|---|
| Time to First Seal | successful install to first meaningful seal | median at most 15 min; p90 at most 30 min |
| Tutorial Completion | qualified participants completing without private help | at least 80 percent |
| Concept Accuracy | participants correctly explaining seal, Fire, evidence, and non-guarantees | at least 80 percent |
| Diagnostic Recovery | failures resolved using output/docs without maintainer | at least 75 percent |
| Fire Actionability | meaningful review/update/decision Fires over sample | at least 70 percent |
| Boilerplate Evidence | decision-free resolutions over sample | at most 15 percent |
| Workflow Tax | CodeFire-specific active minutes / trusted change minutes | at most 15 percent after week two |
| Eight-Week Retention | activated qualified teams active in week eight | at least 40 percent |
| Support Load | maintainer hours per active team per month | under 2 after onboarding target |
| Trusted Change Cost | measured total relative to baseline | lower by pilot end or justified by avoided risk |
| Documentation Findability | persona tasks answered from public docs within time box | at least 80 percent |

## Experiments

### P3-UX-001: First-Seal Observation

- Recruit users who match the ICP but have not read internal docs.
- Record every hesitation, error, help lookup, command, and misunderstanding.
- Do not coach unless the participant is completely blocked; mark intervention.
- Repeat after each material tutorial or CLI change.

### P3-UX-002: Claims Comprehension Test

- Show the README/product page without verbal explanation.
- Ask what a CodeFire certificate proves and does not prove.
- Present a policy-consistent but semantically wrong example.
- Exit: target Concept Accuracy threshold is met.

### P3-ADOPT-001: Four-Week Team Pilot

- Baseline review lead time, escaped drift, CI cost, and support burden.
- Run weekly Fire-value and rationale-quality samples.
- End with champion, skeptic, reviewer, and manager interviews.
- Decide expand, modify, narrow, or stop using predeclared criteria.

### P3-ADOPT-002: Shadow-Workflow Detection

- Ask participants how they pause, back up, share, or recover unsealed work.
- Inspect whether Git, OS copies, cloud sync, or IDE history becomes an undeclared authoritative system.
- Exit: the resilience model is understood and does not routinely bypass CodeFire's stated guarantees.

### P3-ECON-001: Trusted Change Cost Study

- Compare at least 20 similar changes with and without CodeFire where possible.
- Include human, AI, CI, storage, rework, and support cost.
- Record prevented issues separately from speculative value.
- Exit: a target segment shows positive measured or risk-adjusted value.

### P3-ECON-002: Sustainability Interview

- Test willingness to pay for support, training, enterprise integration, and a separate managed service.
- Ask for budget owner, procurement barrier, expected SLA, and alternative spend.
- Do not treat stated interest as revenue; seek a paid or formally sponsored pilot.

## Promotion Targets

- Accepted user journeys and personas: public user documentation strategy.
- Accepted profile design: product requirements, CLI specification, and ADR if assurance semantics change.
- Accepted adoption model: docs/development/strategy/adoption-strategy.md.
- Accepted language/accessibility policy: documentation governance and CLI UX standard.
- Accepted economics metrics: strategy scorecard, not product runtime unless privacy-reviewed.
- Any managed service decision: separate product boundary, operations plan, and legal review.
