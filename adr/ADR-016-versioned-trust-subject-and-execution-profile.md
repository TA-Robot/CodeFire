# ADR-016: Bind trust decisions to versioned subjects and execution profiles

- Status: Accepted
- Date: 2026-07-12
- Decision owners: Remote security / Verification / Evidence / Automation
- Related: CFR-164, RVC-005, RVC-018, RVC-038, RVC-041, RVC-043, RVC-046, ADR-013, ADR-015

## Context

CodeFire previously authenticated remote mutation metadata without authenticating the request body. A valid HMAC
could therefore be paired with a different head, object set, lock option, or idempotency key. Request-key validity
windows were not applied, and nonce-cache capacity recovery could forget a live nonce. Separately, verification and
evidence commands recorded results without one stable description of the evaluated subject, effective environment,
input, resource bounds, and execution phase. Context, explain, review packs, verification roots, and commit
certificates could describe related work without sharing one subject identity.

These are one trust-model problem: a result is meaningful only when the authenticated or evaluated subject and the
execution profile are explicit, canonical, and projected across every boundary that consumes the result.

## Decision

1. Remote mutations use `remote_request_signature.version=2` and a version-1
   `codefire_remote_mutation_subject`. The canonical subject binds actor, operation, target, key ID, UTC timestamp,
   nonce, idempotency key, and a SHA-256 digest of the complete canonical payload after removing only
   `request_signature`. The HMAC covers the complete subject metadata, including its independently recomputed
   subject digest. File and HTTP transports and the Rust and Python implementations share one golden fixture.
2. A supplied legacy version-1 request signature is rejected rather than interpreted as body authentication. Request
   keys apply signer/status plus `not_before`/`not_after` to the signed timestamp. Payload, idempotency, subject,
   key-window, and HMAC validation all precede nonce publication and remote mutation.
3. The nonce cache removes only structurally valid TTL-expired entries. The same nonce under one actor/key is rejected
   even if a newly signed payload differs. If all bounded entries are live, a new request fails closed; an existing
   live replay remains remembered and is still rejected. Malformed cache/entry data is preserved and fails closed
   instead of being reinitialized. The local file-backed cache stays protected by its existing lock. This ADR does
   not claim a distributed replay store.
4. Local assessment uses version-1 `codefire_assessment_subject` values. A live-open subject binds repository,
   branch, open instance, base, canonical scan content, and the exact canonical worktree-manifest digest under
   `trust_kind=local_unisolated`; a review pack uses an immutable base/source pair under
   `trust_kind=sealed_validated`. Context, explain, verification, review pack, and commit certificate project the
   applicable subject digest rather than inventing separate identities. The sealed validator enforces the exact
   field/trust/immutability contract for each subject kind and rejects unbound version-1 extensions.
5. Verification commands use `trusted_local_shell_v1`. The effective cwd must canonicalize below the open directory
   before active-state publication. The default environment is a small allowlist plus explicit policy overrides;
   `inherit_env: true` is an explicit weaker mode whose Unicode environment snapshot is still materialized and
   hashed. Because command and explicit environment policy are sealed, secret-like inline metadata is rejected in
   preflight without reflecting its value. Each execution record binds phase, assessment subject, raw-input digest, environment digest, cwd,
   worktree before/after digests, limits, byte counters, truncation, timeout, exit, termination scope, and redaction
   count. Verification refuses to start against a worktree that no longer matches the live subject, and a command
   that changes the assessed worktree is blocking even when it exits zero. Commit repeats the subject/worktree check
   before its transaction, after manifest planning, and immediately before branch publication.
6. Evidence commands use `local_evidence_command_v1`, a minimal environment, canonical input/environment digests,
   streaming bounded stdout/stderr readers, and process-group termination where supported. Secret-like
   command/argv/output values, artifact URI credentials, and local path literals are redacted before object or plan publication. External cwd is stored
   as `<redacted>` and classified `local_only_redacted`; repository-relative capture is `portable_redacted`.
7. Command output is drained while the process runs and only the configured prefix per stream is retained. Timeout
   terminates the process group on Unix and the direct child on other platforms; the recorded termination scope makes
   that platform distinction explicit. CodeFire does not claim OS-level sandboxing.

## Consequences

- Changing any authenticated payload/object field causes a pre-write authentication failure on both transports.
- Request-signature v1 producers must upgrade to v2; unsigned operation compatibility still follows remote policy.
- Nonce-cache saturation can reject legitimate new traffic until TTL expiry, but it cannot create a replay window.
- Verification tools that write generated files inside the assessed worktree must redirect those outputs or the
  result fails as subject-changing. This is an intentional proof boundary.
- Environment values and raw secrets are not serialized, although their canonical digests intentionally bind the
  execution. HMAC and digest contracts do not turn local execution into a sandbox or remote execution into a
  distributed trust service.
- Historical verification and commit records without assessment/execution extensions remain readable. When either a
  new verification or certificate carries a subject, sealed validation requires exact agreement, validates the
  versioned variant fields, and recomputes the subject identity digest. A version-1 live subject includes its
  worktree-manifest digest; sealed-pair subjects do not claim a mutable worktree.

The canonical source contracts are maintained in [source architecture](../docs/development/source-architecture.md),
[persisted data model catalog](../docs/development/source-data-model-catalog.md), and
[command source trace](../docs/development/command-source-trace.md). CFR-164 owns Install, Dogfood, and Retrospect
closure.

## Verification

- Field-by-field body/object/idempotency/lock tamper tables at the central verifier and HTTP pre-write boundary.
- Both request-key validity-window edges, timestamp skew, replay, cache capacity, and live-entry preservation.
- Cross-runtime golden payload/subject/HMAC bytes for Rust and Python.
- Large and infinite stdout/stderr, descendant cleanup, timeout, minimal-environment, secret/path redaction, cwd
  symlink escape, and worktree-mutation counterexamples.
- Context/explain/review/verification/certificate subject equality, stale-worktree rejection after verification,
  manifest planning, and candidate validation, sealed subject tamper rejection, full source gates, fresh install,
  isolated dogfood, and independent High/Ultra review before CFR-164 closes.
