# CodeFire Engineering Principles

Status: canonical

Owner: Engineering Steward

Update trigger: an ADR changes a cross-cutting invariant, a failure falsifies a principle, or the implementation boundary changes.

## 1. Consistency Before Throughput

CodeFire may make valid work easier, but it must not make an invalid state look complete. Object identity, sealed-commit validation, branch-head atomicity, path safety, remote mutation policy, and machine-readable error contracts outrank patch volume and nominal speed.

Performance work must preserve the decision and diagnostics of the unoptimized path. Cache and memoization keys include every input that can change validity.

## 2. One Authoritative Predicate per Decision

The same semantic decision must not be independently reimplemented in specification prose, command handlers, certificates, JSON rendering, and tests. Define a domain predicate or typed result, then render it for each surface.

The first product-correctness target is the committable predicate: blocker classes, policy input, reason codes, certificate facts, and command exit behavior must agree over a complete decision table.

## 3. Explicit State Machines and Visibility Points

For mutating workflows, name:

- preconditions;
- validated inputs and expected head;
- plan;
- writes and their ordering;
- the single visibility/linearization point;
- crash states;
- idempotent retry result;
- recovery action.

Immutable object writes are preparation. Publishing a validated branch or request pointer is the candidate visibility point; this remains an experiment-backed product decision until the failpoint matrix confirms it.

## 4. Parse, Validate, Plan, Apply, Render

Keep these responsibilities separable and testable. Rendering never decides validity. Applying never reparses an untrusted input with weaker rules. Dry-run and real execution use the same validated plan wherever practical.

## 5. Identity Uses Canonical Bytes and Full Digests

Public short identifiers are navigation, not collision proof. The target invariant is that storage and comparison retain and verify the full digest. Current `store_object` reuse does not yet compare the existing and newly requested full digests for a same-prefix collision (CFB-125); that case must fail explicitly in every path.

Canonical serialization requires cross-runtime golden vectors. Format, root schema, hash schema, extractor semantics, and product release are separate version axes.

## 6. Complexity Is a Contract

Every repository-wide or graph command declares the entities it traverses and an expected complexity budget. Prefer stable work counters—object reads, parses, hashes, graph visits—to wall time alone.

Shared ancestry is validated once per command context when the validity inputs are identical. Any optimization needs equivalence tests for success and corruption diagnostics.

Before a semantic refactor or performance rewrite, add or identify at least one focused regression that fails for the targeted defect or protects the invariant being moved. Test count alone is not evidence; the test must discriminate the relevant behavior.

## 7. Trust Boundaries Are Product Behavior

Repository roots, symlinks, external commands, remotes, evidence payloads, credentials, and generated artifacts cross trust boundaries. Validate paths at use time, constrain resources, minimize captured data, and state the deployment boundary honestly.

Until an isolation profile is implemented and red-teamed, server-side verification is for trusted local or single-tenant operation only. Evidence capture must not imply that arbitrary command output is safe to retain or distribute.

## 8. Typed Contracts Before Prompt Conventions

Machine contracts use versioned JSON plus schemas. Markdown is a human view. Each automated stage must bind its subject by digest before it can authorize the next stage. Task and context binding exist now; complete task→patch→validation→review→certificate chaining remains a Stage 2 deliverable.

A schema pass proves shape, not truth. Semantic validators, deterministic commands, and independent review remain separate gates.

## 9. Deterministic Evidence Outranks Model Judgment

Authority order:

1. safety and permission boundary;
2. deterministic invariant/schema/test gate;
3. independent evidence-bound review;
4. model grader or heuristic;
5. self-reported completion.

Skipped, blocked, stale-base, or wrong-subject validation is never converted into a pass. Raw chain-of-thought is not audit evidence; retain decisions, commands, outputs, digests, and concise rationale.

## 10. Parallelism Requires Ownership

Read-only investigation can run in parallel. Writes require declared read/write sets, one writer per file or state object, and manager-owned serialized integration for global files. Agents never use parallelism as permission to overwrite user or peer changes.

Review is independent of implementation for invariant, security, deletion, migration, and release-completion work.

## 11. AI Orchestration Is Development Infrastructure

The CodeFire product exposes automation-safe contracts. Model routing, prompt construction, agent scheduling, marketplace behavior, and provider adapters live outside the product core unless a separate ADR demonstrates that a consistency/evidence primitive belongs in the product.

Role semantics are stable; runtime selection is explicit and versioned. Unavailable models or Ultra capabilities stop or use an explicitly recorded non-equivalent fallback—never silent substitution.

## 12. Optimize Cost to Trusted Integration

The unit of progress is not lines, commits, agent count, or first-patch latency. Measure the time and total human/model/CI/rework cost from a bounded task to a reviewed, validated, integration-ready change.

Critical-escape rate, user-work loss, unauthorized mutation, false completion, and secret disclosure are guardrails with a target of zero. Speed improvements that regress a guardrail are rejected.

## 13. Reversible Adoption and Compatibility

Integrations begin read-only, then advisory, then selectively blocking, and only then native or mutating. Unknown coverage is reported as unknown, never clean.

Thin adapters translate stable CodeFire contracts to ecosystem standards. They do not redefine core semantics or create transparent Git compatibility.

## 14. Documentation Has a Lifecycle

Every durable document has a purpose, authority, owner, update trigger, and lifecycle state. Current state has one owner. Git history preserves superseded detail; active documents do not retain unlimited historical task prose.

Canonical-family deletion is Ultra work and requires an inventory, destination/no-unique-content finding, inbound-link migration, and post-deletion checks.
