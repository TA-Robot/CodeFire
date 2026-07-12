# CodeFire Product Strategy

Status: canonical strategy with explicitly labeled market hypotheses

Owner: Product Steward

Update trigger: independent interview/pilot evidence, a product-boundary ADR, or a falsified positioning claim.

## Durable Product Core

CodeFire is a consistency-first, version-control-style system for specification-driven change. Its durable differentiators are:

- explicit Atoms and Trace Links;
- Fires and evidence-backed resolutions;
- verification under a declared policy;
- content-addressed objects and sealed commits;
- machine-readable diagnostics, plans, and next actions.

The product can prove that a change satisfied a declared CodeFire policy and that the recorded evidence/objects bind to a sealed subject. It does not prove semantic correctness, security, regulatory compliance, or business fitness by itself.

## Leading Category Hypothesis

The external wedge to test is:

> AI-change consistency and evidence protocol/control plane for teams where rapid machine-produced changes create traceability, review, and completion-trust failures.

This phrasing is a hypothesis, not a validated market claim. The core can remain a high-assurance native repository while external adoption begins with evidence/diagnostic adapters.

Falsify or narrow the hypothesis if problem interviews and pilots show that buyers evaluate CodeFire only on Git parity, cannot understand the assurance boundary, or do not experience costly trace/evidence drift.

## Initial Customer Hypothesis

Test first with AI-heavy teams that are already specification-driven or high-assurance:

- internal platform or regulated-domain engineering teams;
- teams using AI agents for multi-file changes where requirements, tests, and review evidence drift;
- research/experiment teams that need reproducible claims and negative-result history.

Do not claim a broad developer ICP from the single dogfood repository. Require independent interviews and at least three external pilot repositories before expanding the category claim.

## Product Boundary

| In the core | Thin external adapter | Out of scope unless a new ADR changes it |
|---|---|---|
| Atoms, Trace Links, Fires, policy verification, evidence references, sealed objects/commits, current versioned Rust JSON envelope, capability discovery; public payload schemas/conformance after Stage 2 | read-only MCP, SARIF diagnostics, CI advisory, LSP views, Git snapshot/PR assessment, CDEvents/OTel/in-toto/SLSA/SPDX export | agent scheduler, prompt/model marketplace, autonomous code generation platform, transparent Git compatibility, hosted multi-tenant control plane |

Adapters must preserve claim type. A snapshot or PR assessment is not a native sealed CodeFire certificate. Unknown or uncovered material stays explicit.

## Adoption Ladder

1. **Inspect** — read-only local assessment and context; no repository mutation.
2. **Advisory** — SARIF/CI or MCP diagnostics; no blocking claim.
3. **Critical trace** — selected requirements/paths with declared coverage.
4. **Selected blocking** — policy gates only where evidence and false-positive rates justify them.
5. **Native seal** — full CodeFire repository semantics and sealed history.

Each step must produce standalone value. Do not force a native migration to test whether the problem exists.

## First Integration Experiments

Compare two thin wedges before selecting one:

- read-only MCP over the current CLI envelope and the Stage 2 public schema/conformance kernel;
- SARIF/CI advisory for existing repositories.

The Git bridge begins as read-only snapshot/base-head assessment. Stop if it requires widespread Git-specific semantics in core, creates ambiguous subject identity, or causes users to confuse advisory output with native consistency evidence.

## Metrics

North Star: total cost to trusted integration for a meaningful change.

Guardrails:

- zero unauthorized mutation or outside-root access;
- zero false completion caused by missing/forged/stale evidence;
- zero user-work loss;
- zero unredacted secret disclosure in supported evidence flows;
- no critical invariant escape relative to the established baseline.

Adoption evidence:

- time to first meaningful trusted signal;
- first-seal completion and comprehension;
- Fire actionability and false-positive rate;
- retained pilot use and request for deeper integration;
- setup/maintenance time per trusted repository;
- independent integrations passing conformance.

## Claim Language

Prefer:

- “policy-consistent under profile X”;
- “evidence bound to subject digest Y”;
- “coverage includes A and excludes B”;
- “verification command passed in environment Z.”

Avoid:

- “the change is correct”;
- “secure/compliant by CodeFire”;
- “all changes covered” when coverage is unknown;
- “Git-compatible” or “drop-in replacement.”

## Pivot and Stop Rules

- If advisory pilots produce useful findings but native workflow demand is absent, pivot toward an independent verifier/evidence protocol rather than forcing a replacement VCS.
- If Fire actionability stays below the predeclared pilot threshold after one focused policy iteration, narrow supported profiles and propagation.
- If assurance claims remain misunderstood, narrow claims before adding features.
- If adapter maintenance dominates value, retain the validated CLI/schema kernel available at that stage and drop the adapter.
- Hosted multi-tenant operation requires a separate owner, threat model, legal/operations review, and ADR; it is not an incremental default.
