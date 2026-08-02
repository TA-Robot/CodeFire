# CFR-168 C14-LIVE-A1 Pre-effect Cleanup Plan/FREEZE

Status: `ready_for_bounded_implementation`

Packet ID: `cfr168-c14-live-a1-plan-freeze-20260802`

Lifecycle position: Freeze, Plan, and risk-based Preflight complete for A1;
Implement remains future work

Execution role: one bounded High implementer; one fresh Ultra exact-candidate
reviewer; manager retains integration and lifecycle authority

Preferred workers: Claude Opus 5 implementation and a distinct fresh Claude
Opus 5 review session; Codex remains the manager/adjudication lane

Typed companion:
[`json/cfr168-c14-live-a1-plan-freeze-20260802.json`](json/cfr168-c14-live-a1-plan-freeze-20260802.json)

Exact product/evidence base:
`333e41082ffc35c73c02a420511ae2f16839c5b8`, tree
`c0ab3a09e4731ca27d43023be72e577487b11600`

Root issue: CFR-168 remains nonterminal. This packet does not change the issue
status, source-accept a future candidate, or authorize A2/B3, integration,
Install, Dogfood/Evaluate, Close, or Retrospect.

## 0. Decision

The manager freezes one implementation package for `C14-LIVE-A1`, the
reachable pre-CodeFire-authoritative-effect cleanup owners registered in
[`cfr-168.md`](../../code-review-issues/cfr-168.md) at the exact base above.
The package is ready for bounded implementation because the earlier source
audit's two authorization-changing questions are decided here:

1. The exact original primary exception object remains outermost. The first
   cleanup fault is its immediate `__cause__`. Any effective cause/context
   chain already owned by the primary remains below that first cleanup fault,
   by exact exception identity. No new global cleanup container, response
   protocol, or persisted diagnostic is introduced.
2. The live direct ancestry and signature owners belong to A1. Only their
   sealed/combined successor orchestration remains outside A1. Product source,
   not older prerequisite wording, decides reachability.

The packet freezes the state machines, counterexamples, allowed paths,
negative space, validation, candidate topology, and reviewer exclusions below.
An implementer may choose local names and small private helpers, but may not
invent a different failure contract or widen the repair.

This packet itself is docs-only. It implements none of the product behavior it
specifies.

## 1. Why A1 is a live production package

At `333e410`, the production upload call graph is:

```text
make_http_handler.do_POST                         codefire:17767
  -> handle_http_upload                          codefire:14720
       -> enforce_remote_commit_signature_policy_accounted
                                                   codefire:14889 / 7787
            -> AccountedRemoteSignaturePolicyAuthority
            -> AccountedCommitSignaturePolicy
            -> _AccountedUploadCommitSource
       -> run_server_verification                  codefire:14899 / 9656
            -> AccountedServerVerification
       -> is_ancestor_from_objects_accounted       codefire:15006 / 12441
            -> _AccountedUploadCommitSource
            -> AccountedHTTPUploadAncestry
```

Therefore these are false descriptions of the exact base and must be corrected
by the future implementation's canonical-doc synchronization:

- treating `AccountedHTTPUploadAncestry` itself as production-unwired;
- treating `AccountedRemoteSignaturePolicyAuthority` itself as
  production-unwired;
- saying `AccountedCommitSignaturePolicy` is not connected to production
  signature enforcement; and
- saying signature and ancestry are wholly absent from the upload production
  path.

The true distinction is narrower:

- the direct wrappers and the three shared owners above are live;
- `AccountedHTTPUploadSealedAncestry`,
  `AccountedHTTPUploadSignatureVerificationComposite`,
  `AccountedHTTPUploadExistingTerminalOwner`, and their sealed/terminal
  orchestration are production-unwired successors.

The exact base tracker already records this corrected ownership at
`cfr-168.md:1088-1098`. The implementation candidate must not rewrite that
OPEN registration or prematurely mark it accepted/closed.

## 2. Immutable authority and evidence

### 2.1 Base blobs

The candidate is judged against these exact base blobs. SHA-256 values are
content digests; Git blob IDs are repository identities.

| Path | Git blob | SHA-256 |
|---|---|---|
| `codefire` | `73b1f368f9c1591be6fd9219b839351878b82b9a` | `459d01a13faad9d978557ff8bc8709c7a14e955f70216ce5abafeb58071ac230` |
| `tests/test_python_http_upload_accounted_traversal.py` | `aba7ac9fb4f73bfc1257da00d2a4ddbcbf6f55a4` | `8db617cc9977afe6478753d01faf8f515c4a8b0c6349ee31d7d112a6cfa33ee1` |
| `tests/test_python_ancestry_owner.py` | `5416ec8cc5a7a458a4d7134f9d13ce692dec86c9` | `36aaa30c7f1a8ad71bd8f2b5dff7e097bc9429c7d8a39d424d770d4456fe973d` |
| `tests/test_python_ancestry_borrow_handoff.py` | `2aa3079f0a5e136b636ceaae3cd007d977ebe968` | `74e1521d03722b703a38ebac1c184737044d83bbf07277ca81a78239e12d4339` |
| `tests/test_python_signature_authority.py` | `8d6c4f7e518f98f72d7f2b8ba8f8af4bc673c145` | `6d8b4b01e2fb7419a725dd13f9b36e30170b75cfceaaf511625e15e946c87053` |
| `tests/test_python_signature_owner.py` | `3899efe384394aacafb433e5de79e71bad0993f4` | `2fea18706f981e923ac92438b1a2aee16e66e484ecbafbfae9eb6a5c4195d7c9` |
| `tests/test_python_external_verification.py` | `d1075e7d8b8d6c7c411ed97c1f5ff360b2051cf6` | `220a8075501db49a8c1436363c45a6feb42a046a82f5170f8897be35ec0300ef` |
| `tests/test_python_external_production.py` | `576e97a322f1680accb5666a0fe750c7c71e860b` | `2d7f388c6e9e6955713b862fc7061d9b65877c39327fcec8e7ec87cc3acf316e` |
| `tests/test_python_signature_verification_handoff.py` | `b7f068364187728bc24fbabd10589cc9e60e0374` | `11b1e94ef745f49a60221ab10181ce3186557b653d034541d022e2b2062dc6e7` |
| `tests/test_python_sealed_production.py` | `3097cb3613428ea989ba5c07c63ce003a4d2e4de` | `0d5896bab2a83b4528344e6d6bdab13f6db231678480d5dadd77f75283590b9d` |
| `docs/development/source-architecture.md` | `fe2b3b74f2beaf30218333f14e05b6e4e6d4e699` | `dac295515ccff70b351a2b95fa1cc3aa708444061a1d568d5d64d9b79dfb3ec5` |
| `docs/development/command-source-trace.md` | `361a5de8f9f881fbedf35ed5a833baade9529e38` | `10a1aba9f5ea1075ca043d6d17367cc432cb78fd78cfccc935a31897be9fd5af` |
| `docs/development/source-data-model-catalog.md` | `2b68347eaa357f187f794c2a76992401743929eb` | `86e6dd1f30a312b4296e554b2504ef224cbd8946e04ecb279428d4d9c4d24dc0` |
| `docs/development/change-playbook.md` | `18d882813e02ed8cd795aa1e46d8cac6a09be4ef` | `4d81b4678dd2e0c92c55b882b99b3bc8acef800aefa262ec5e0637cdb66871f6` |
| `docs/development/known-limitations.md` | `b9753961e24737d6e30306269b7111572170557d` | `37a7abcb94825539f194ec266b07cb426db9b2294e1958fbe6b63bc470d95505` |

### 2.2 Frozen evidence

| Evidence | Commit / blob | SHA-256 | Use in this packet |
|---|---|---|---|
| Shared-primitives adversarial preflight | commit `6792f86125c9659bdc8195f3b3c98e1b060ecf1c`, blob `ec2a264142c6625dadd4809d99cf18d3371e9847` | `ca1a1451bfc1b25ed1c54f1b012d4d0e508ef0a96f88401e61284c6d156a4441` | frozen C12-C14 ownership and failure-precedence family; its historical broader `BLOCK` is not rewritten |
| Upload third-repair rereview | commit `c62ab9776060902634e6d36785f58a8eb36e4488`, blob `91bd5a92ea9f2ec8a9ccb1760851274724bb66b5` | `f131284ea7d441781790481f1d356ca73a763d1ddb41ae878a6f7179c88ec649` | production counterexamples, F6 persistence and F7 22,912 orphaning |
| Accepted live-route cleanup rereview | commit `5d3f35108eb08e134a2a0b050848f43343060dc9`, blob `216ee2680872c43dcd7b657e3a8d4312dbd9b1e9` | `46e7c3e41ac1b132766657e445e017b392ce6a0b72eb46f909b13755fc8a9b3c` | proves the bounded one-shot wrapper repair accepted; does not close F6/F7/global C14 |
| Accepted-input composition writer result | commit `1d5915646fcac12b44bb91466b22d53dcf7403d1`, blob `cd1168cdee61223a08495029adf619c7eba99329` | `02a759c40d11c9b79d52872104b53da79c9de069629c8b63aa9bb610adcce3e3` | registers A1/A2/successor residual routing and the one surviving server inversion |
| Exact composition review | commit `b4a3ef35b17c330ba18057a3ff9222f3632096c8`, blob `c404b0393f0adc7eb27e92de9bddf179f37b2d75` | `a3835f3720983cee72a3368fcc149158a44c6bb3ea1d3c89cbca8432503fadd5` | source-accepts composition only and routes A1 as an out-of-scope successor residual |

Historical finding labels are retained. This packet is a later manager
admission of F6/F7 and the complete A1 owner set; it does not rewrite the older
review's then-valid `nonblocking_improvement` disposition.

## Objective

Implement one local, bounded C14 pre-effect cleanup contract across the live
Python fallback upload owners so that:

1. the exact operation/construction/validation exception remains the outer
   exception;
2. the first cleanup fault is its immediate cause without erasing the primary's
   prior effective exception chain;
3. every retryable scalar, permit, process, child, and memory owner remains
   reachable until its own cleanup commits;
4. a persistent close fault across every sealed-session descriptor is handled
   by one bounded logical sweep without raw-FD retry and without leaving the
   fixed `11,456`-byte session-control reservation stranded;
5. `_AccountedUploadCommitSource._open` settles an already-published memory
   owner before any new acquisition and can never grow the same source from
   `11,456` to `22,912` by overwriting the first owner;
6. live direct ancestry, signature, and server-verification semantics remain
   otherwise compatible; and
7. source documentation distinguishes those live direct owners from the
   still-unwired sealed/composite successor design.

The successful end state is a clean, exact, independently reviewed A1
candidate. It is not A2/B3 implementation, canonical integration, or CFR-168
closure.

## Scope

The future implementer may edit `codefire` only at the following owners and at
small private helpers used exclusively to express this contract.

| Owner/locus at base | Existing failure shape | Frozen repair obligation |
|---|---|---|
| `_AccountedUploadCommitSource._open` (`7646`) | constructor primary survives, but cleanup is detached and omitted as cause; a retained `_memory` can be overwritten on reentry | settle any existing `_memory` before poll/acquire; preserve exact primary/cause chain; publish no replacement owner while settlement fails |
| `_AccountedUploadCommitSource.close` and `_drained_commit_source_cleanup_error` (`7689`, `7743`) | exactly two wrapper calls handle one-shot faults, but cannot by themselves drain an arbitrary descriptor count | keep two-call wrapper bound; make the session's single close call sweep all independent descriptors; first fault remains visible |
| `SealedCommitValidationSession.close` and `_SealedValidationFileDescriptor.close` (`25110`, `23935`) | session stops at the first descriptor fault; each later call advances at most one descriptor | detach each numeric FD before exactly one close; continue the bounded descriptor sweep; release logical control; never retry a consumed numeric |
| `validate_sealed_commit` (`32598`) | constructor cleanup is not attached; body primary discards cleanup error | apply exact-primary/first-cleanup cause ordering in constructor and body cleanup, preserving caller-owned ledger semantics |
| `AccountedHTTPUploadAncestry` (`11902`) | constructor and `contains` raise cleanup outer from primary (`11985`, `12373`) | exact construction/decision error outer; first cleanup immediate cause; same owner remains reachable for retry; traversal result/order unchanged |
| `AccountedRemoteSignaturePolicyAuthority` (`5308`) | constructor inversion (`5405`) and cleanup in `_open_subject`, `_open_absolute_directory`, `_revalidate_project`, `_revalidate_policy` can replace an in-flight primary | preserve operation primary and its chain; preserve capability ownership/order; consumed FDs are never retried |
| `AccountedCommitSignaturePolicy` (`6543`) | constructor overwrites primary at `6684`; HMAC cleanup wins at `7250` | exact policy/HMAC error outer; first cleanup cause; transition/permit retry ownership unchanged |
| `AccountedServerVerification` (`8086`) | constructor raises cleanup outer (`8221`); `_run_plan` cleanup and selector close can replace run failure; sequential close stops at first independent fault | exact constructor/run error outer; bounded cleanup of independent owners; unreaped process remains the same retry owner; no external-effect rollback claim |
| `run_server_verification` (`9656`) | the only remaining literal `raise cleanup_error from run_error` at `9692` | exact run error outer, cleanup error immediate cause; cleanup-only behavior unchanged |

### 4.1 Shared-helper boundary

A private helper is permitted only if all of these remain true:

- it is non-persisted and has no HTTP response, wire, ledger-terminal, logging,
  or public API role;
- it operates only on exception objects and/or local A1 cleanup results;
- it creates no `RetryableCleanup`, `ExceptionGroup`, global registry, sidecar,
  or new cleanup result container shared with A2;
- every call site is inside the A1 owner table above; and
- deleting or bypassing it fails an identity-and-chain behavioral oracle, not a
  symbol-count oracle.

Generic changes to `_StablePermitTransition`, `DecisionWorkLedger`, request
accounting, the error-origin taxonomy, and response owners are forbidden unless
a newly reproduced A1 failure makes them unavoidable. That discovery returns
to manager Freeze; it is not an implementer choice.

## 5. Frozen exception-chain contract

### 5.1 Primary plus cleanup

For an operation/construction/validation exception `P` and the first cleanup
fault `C`:

```text
outermost: P (same object identity)
immediate explicit cause: C (same object identity)
below C: C's pre-existing effective chain, then P's pre-existing effective chain
```

“Effective chain” means `__cause__` when present, otherwise `__context__`.
Before relinking, the implementation must capture the chain by identity,
bounded by the finite reachable exception graph, detect cycles, and include
each exception object at most once. Relinking uses explicit `__cause__` edges
and suppresses incidental context so rendering is deterministic.

This order preserves the cleanup fault's own explanation before appending the
primary's prior explanation. If `C` has no prior chain, the first node below it
is the primary's former effective cause. The implementation must not replace
`P`, wrap it in a new exception, stringify it, or make the old primary cause the
outer cleanup exception.

Tracebacks and payload-bearing locals must still be detached before a release
observer can see them. Preserving exception identity and the bounded cause
chain does not authorize retaining raw policy bytes, parsed policy trees,
commit payloads, process output, paths beyond existing diagnostics, or owner
frames.

### 5.2 Cleanup-only and later cleanup faults

- With no primary, the first cleanup fault is the exact outer exception and no
  artificial primary is created.
- Later cleanup faults never replace `P` or `C` and are not attached through a
  new global collection. They are detached after their independent cleanup
  action has been accounted for.
- A2 owns any future durable/global secondary-failure protocol. A1 does not
  write notes, request logs, response metadata, or wire diagnostics for later
  cleanup faults.
- A process-fatal `BaseException` observed as the primary follows the same
  bounded cleanup rule. This packet does not promote deferred O3 retry-error
  recording into scope.

### 5.3 Existing wrong oracles that must reverse

The following base tests currently encode cleanup-first behavior and must be
changed as part of A1, not preserved as compatibility:

- `tests/test_python_external_production.py:239-275` expects the release
  `RuntimeError` outer and the verification `CFError` as cause;
- `tests/test_python_signature_verification_handoff.py:487-522` expects the
  constructor release fault outer and the construction `CFError` as cause; and
- `tests/test_python_ancestry_borrow_handoff.py:202-245` expects ancestry
  constructor cleanup outer.

Each replacement oracle must capture the exact primary and cleanup objects and
assert identity, not only type/message.

## 6. F6 bounded persistent-descriptor recovery

### 6.1 Required state machine

The logical owner states are:

```text
OPEN
  -> CLOSING
       -> CLEANUP_PENDING   (retryable logical debt remains)
       -> CLOSED_WITH_FAULT (all logical debt consumed; first fault reported)
       -> CLOSED            (all logical debt consumed; no fault)
```

The implementation need not store those literal strings. Its fields must make
the same states distinguishable and idempotent.

For retryable scalar, permit, process, or child ownership:

```text
OWNED -> release attempt -> RELEASED
                         -> OWNED/CLEANUP_PENDING on a pre-commit failure
```

Clear the field or scalar only after the underlying owner accepted the release.
When an accounting callback commits and then raises, use the established
before/after ledger observation to record the release exactly once while still
propagating the fault.

For an OS descriptor number:

```text
OWNED_NUMBER -> DETACHED_ATTEMPTED -> CLOSED_OR_ABANDONED
```

The descriptor field is set to its unavailable value before the one allowed
close call. An exception consumes numeric ownership. Never place that integer
back into a retry state: POSIX does not prove whether the kernel closed it, and
a retry can close unrelated authority after same-number reuse.

### 6.2 Session sweep

One call to `SealedCommitValidationSession.close` must:

1. drop payload collections in the established order;
2. attempt every currently registered independent descriptor owner exactly
   once, even when the first, second, or every close call raises;
3. retain the exact first close fault and detach later faults;
4. clear descriptor registries/pools only for logically consumed owners;
5. continue releasing independent logical control claims after all descriptor
   numbers are consumed;
6. stop before releasing a parent claim whose dependent retryable child still
   owns logical debt;
7. mark the session logically closed if and only if every retryable logical
   debt is gone, even when a consumed descriptor close fault will be raised;
8. raise the exact first cleanup fault after the bounded sweep; and
9. on the next wrapper close, return idempotently so the source can clear its
   session pointer and release `_AccountedUploadSourceMemory`.

The existing wrapper remains an initial call plus one retry. The repair is not
“retry until success”; it makes the first session call complete all finite
descriptor work, and the second wrapper call completes source/memory handoff.

### 6.3 What F6 does and does not guarantee

The required persistent fault is “every descriptor close attempt reports a
fault.” It is recoverable at the logical-accounting layer because every numeric
FD is consumed once. The candidate must finish with zero logical session debt,
zero open permits, and no `11,456`-byte reservation after the bounded wrapper
retry.

If an underlying retryable ledger release or process reap can never commit,
the same exact owner remains reachable and `CLEANUP_PENDING`; the packet does
not promise success against an eternally non-committing primitive. A consumed
descriptor may remain open in the process until process exit after an
ambiguous close error. That conservative leak is preferable to closing an
unrelated reused FD and must be documented honestly.

## 7. F7 pre-acquisition reconciliation

Before `_AccountedUploadCommitSource._open` polls, constructs a new
`_AccountedUploadSourceMemory`, calls `acquire`, or constructs a session, it
must inspect the published old state.

If `_session is None` and `_memory is not None`:

1. call `close` on that exact memory owner;
2. if close raises, preserve that exact pointer and propagate the fault; do not
   poll, allocate, acquire, or construct anything new in that call;
3. if a post-commit release fault left the memory logically closed, still
   preserve the pointer for the faulting call; a later call may observe an
   idempotent close and clear it;
4. clear `_memory` only after an error-free idempotent close; and
5. only then create/acquire one new memory owner.

The required counterexample begins with a session constructor primary and a
memory cleanup fault, holds the first error/source live, and calls `_open`
again. While cleanup remains unavailable:

- the exact first memory identity remains published;
- constructor/acquire call counts do not increase;
- `ledger.current["retained_bytes"]` remains exactly `11,456`, never
  `22,912`; and
- explicit retry after the fault clears drains the same owner before one new
  session may open.

Deleting the reconciliation or restoring the base overwrite must fail this
oracle behaviorally.

## 8. Per-owner implementation requirements

### 8.1 Ancestry

- Constructor allocation failure plus close failure: exact allocation error
  outer, exact first close fault cause, published ancestry owner retained.
- `contains` traversal/deadline/loader failure plus close failure: exact
  decision error outer, cleanup cause, same owner retryable.
- Live `goal_directed=True` behavior remains goal-directed.
- The successor full-BFS path still walks after a match for legacy late-error
  ordering. A1 must not convert both paths to one traversal mode.
- Seen/frontier/frame accounting, same-ledger checks, deadline phase, node/edge
  counters, and payload drop order remain unchanged except where cleanup state
  requires a local correction.

### 8.2 Signature authority and policy

- Authority construction, absolute-walk cleanup, project revalidation, policy
  revalidation, policy-child construction, history traversal, and HMAC cleanup
  all obey the frozen chain contract.
- Child-before-parent and payload-before-scalar ordering is retained.
- Raw policy and parsed JSON are absent from retained exception frames before
  scalar release observers run.
- Each detached capability number receives at most one close call.
- Permit/transition fields remain published until consume/cancel/release has
  committed; no broad `_StablePermitTransition` rewrite is authorized.
- Signature verification result, key-window behavior, history order, HMAC
  input bytes, deadline phases, and policy subject fences remain unchanged.

### 8.3 Server verification

- Constructor failure plus owner close failure preserves the exact constructor
  error outer and publishes the partial child to its established parent slot.
- `run_server_verification` preserves the exact run error outer when close
  fails; cleanup-only failure remains outer when run succeeded.
- `_run_plan` must not let `_stop_process`, pipe close, selector close, permit
  cancellation, or later owner close replace an in-flight plan/deadline/read
  error.
- Independent plan/capability descriptors are swept boundedly with detach-before-close.
- If a process cannot be reaped, the exact `_process` remains owned for retry
  and the owner is not marked closed.
- Configured verification commands can create filesystem/network/process
  effects before failure. Cleanup kills/reaps owned processes when possible;
  it does not roll back command effects and this candidate must not claim that
  it does.

### 8.4 `validate_sealed_commit`

- Constructor primary plus memory cleanup fault follows the frozen chain.
- Validation/finalize primary plus session and/or memory cleanup faults follows
  the same chain, using the first cleanup fault only.
- A cleanup-only failure remains visible.
- Caller-owned versus local ledger, terminalization, `seen` update, deadline
  phase, and validation order remain unchanged.
- No new public retry API or exception-attached owner handle is added. Existing
  local ownership/finalizer reachability remains the fallback when a
  retryable primitive cannot commit.

## 9. Closed implementation write set

The future product candidate may modify only these paths:

```text
codefire
tests/test_python_http_upload_accounted_traversal.py
tests/test_python_ancestry_owner.py
tests/test_python_ancestry_borrow_handoff.py
tests/test_python_signature_authority.py
tests/test_python_signature_owner.py
tests/test_python_external_verification.py
tests/test_python_external_production.py
tests/test_python_signature_verification_handoff.py
tests/test_python_sealed_production.py
docs/development/source-architecture.md
docs/development/command-source-trace.md
docs/development/source-data-model-catalog.md
docs/development/change-playbook.md
docs/development/known-limitations.md
```

Not every allowed test file must change. Every changed path must be necessary,
and Candidate Freeze records the exact subset. The five documentation paths
must agree with the final source; the four canonical source documents are
required when their owned behavior changes.

The implementation result is not part of the candidate. It is a separate
report-only commit adding exactly:

```text
docs/development/ai/swarm-results/cfr168-c14-live-a1-implementation-20260802/result.md
```

The independent reviewer report is another report-only commit adding exactly:

```text
docs/development/ai/swarm-results/cfr168-c14-live-a1-exact-review-claude-20260802/review.md
```

The implementer must stop and request manager scope adjudication if a required
behavioral test cannot be expressed in the closed write set. It may not quietly
add a neighboring successor test or global document.

## Non-Goals

The following remain byte/AST-identical to base unless a manager creates a new
Freeze amendment:

- `_PreparedHTTPMutationResponses`;
- `_HTTPMutationResponseSlot`;
- handler POST arm, request-loop `finally`, response selection and wire
  publication;
- `record_secondary_failure`, cleanup diagnostics and the C18 error-origin
  taxonomy;
- `AccountedHTTPUploadSealedAncestry`;
- `AccountedHTTPUploadSignatureVerificationComposite`;
- `AccountedHTTPUploadExistingTerminalOwner` and the pair/effect/publication
  successors;
- sealed borrow activation/revocation and terminal coordinator contracts;
- planner/stage/publication/response factories;
- `DecisionWorkLedger`, public profiles and resource limits;
- `_control_charge_witness` F9, O2 and deferred O3;
- Rust, schemas, packaging, Install, devcontainer and dogfood sources; and
- CFR-168 tracker status, current-cycle state, history, roadmap and root-cause
  map.

Shared A1 class behavior may be observed through an unwired composite test.
Updating an existing test's C14 expectation does not authorize changing that
composite's orchestration, token, traversal, cleanup order, or production
wiring.

## 11. Hazards and rejected fixes

| Hazard or tempting shortcut | Required disposition |
|---|---|
| Retry `close(2)` until success | forbidden: can hang a request and closes a reused numeric FD |
| Retry a raw descriptor integer after a close exception | forbidden same-number ABA authority violation |
| Clear ledger scalar/owner field to make the observer read zero | forbidden false accounting; ownership clears only after commit |
| Catch only `Exception` and skip `BaseException` cleanup | forbidden behavior drift from current owner contract |
| Wrap the exact primary in a new C14 exception | forbidden diagnostic/API drift |
| Put cleanup only in a note and omit immediate `__cause__` | fails the frozen C14 contract |
| Drop the primary's old cause/context chain | fails the manager decision and chain identity oracle |
| Create an `ExceptionGroup`, list/registry, or global `RetryableCleanup` object | forbidden A2 overlap and compatibility expansion |
| Attach later cleanup failures ahead of the first | forbidden precedence drift |
| Release parent storage while a dependent child/permit/process remains owned | forbidden use-after-release/false-zero state |
| Duplicate `AccountedHTTPUploadAncestry` for the live route | forbidden owner divergence; repair the shared class |
| Treat direct live signature/ancestry owners as wholly unwired | documentation and review-scope defect |
| Change traversal order, signature decision, process command, or HTTP response to make tests pass | forbidden semantic widening |
| Claim configured command cleanup rolls back its external effects | forbidden false recovery claim |
| Modify A2/B3 while already editing `codefire` | stop; A1 and A2 implementation are serialized |

## 12. Required behavioral evidence

### 12.1 Focused positive matrix

| ID | Surface | Injection | Required observation |
|---|---|---|---|
| A1-P01 | `_open` constructor unwind | exact constructor primary plus memory close fault | primary identity outer; first cleanup identity immediate cause; same memory owner retained |
| A1-P02 | `_open` prior chain | primary and cleanup each carry distinct cause/context chains including a shared/cyclic node | deterministic cycle-free identity chain: primary, cleanup, cleanup prior chain, primary prior chain; each identity once |
| A1-P03 | F7 reentry | hold P01 error and call `_open` repeatedly while old memory close fails | same owner/call counts; retained exactly 11,456; no constructor/new acquire; later drain then one new open |
| A1-P04 | live ancestry source F6 | every session descriptor close reports a distinct error | finite close count; first error preserved; second wrapper call drains logical current/permits; no raw numeric retry |
| A1-P05 | live signature-history source F6 | same as P04 | same bounded/zero result on the second live wrapper |
| A1-P06 | FD reuse safety | a failed close number is reused by an ambient descriptor before cleanup retry | retry never closes the ambient descriptor |
| A1-P07 | ancestry constructor | seen/frontier construction primary plus close fault | exact construction primary outer; first cleanup cause; published owner retry closes |
| A1-P08 | ancestry contains | loader/deadline/traversal primary plus close fault | exact decision primary outer; cleanup cause; full and goal-directed semantics unchanged |
| A1-P09 | signature authority constructor/final fence | primary plus descriptor/scalar cleanup fault | exact primary/cause identities; capability/transition retry ownership preserved; no raw payload retention |
| A1-P10 | signature policy constructor/HMAC | parse/HMAC primary plus cancel/release fault | exact primary/cause; permits/transitions zero after same-owner retry |
| A1-P11 | server constructor | config/capability primary plus release fault | exact constructor primary outer; partial child published; retry closes it |
| A1-P12 | server run wrapper | verification command `CFError` plus retained release fault | exact run `CFError` outer; release fault cause; owner remains retryable |
| A1-P13 | server process cleanup | run/read/deadline primary plus kill/wait/selector fault | exact primary outer; first cleanup cause; unreaped process retained; later retry reaps |
| A1-P14 | multi-plan capability cleanup | every capability close reports a fault | bounded sweep, first fault, no numeric retry, logical debt zero when only consumed descriptors remained |
| A1-P15 | `validate_sealed_commit` constructor | constructor primary plus memory close fault | exact primary outer; cleanup cause; no caller-ledger double release |
| A1-P16 | `validate_sealed_commit` body | validation primary plus session and memory faults | primary outer; first cleanup cause; later cleanup cannot replace; eventual same-owner/finalizer drain when primitive clears |
| A1-P17 | cleanup-only | each repaired owner succeeds in work then cleanup fails | exact first cleanup fault outer; no artificial cause/wrapper |
| A1-P18 | payload detachment | raw policy/parsed tree/commit/process output is weakly observed at release | payload is not retained by the rebuilt exception chain |

### 12.2 Compatibility and negative-space matrix

- ancestry goal-directed and full-BFS fixtures retain their exact boolean,
  late-error and accounting behavior;
- signature head/history, malformed UTF-8/JSON, HMAC, deadline and subject-swap
  fixtures retain their exact outcomes;
- server verification success, timeout, output cap, no-policy/missing-project,
  cwd capability, process-group cleanup and legacy no-ledger fixtures retain
  their outcomes;
- upload live routes preserve zero durable branch/object/nonce/transaction/
  receipt mutation on pre-effect failures;
- the C18 cleanup-origin classifier remains unchanged;
- A2 response and wire owners are AST-identical to `333e410`; and
- sealed/composite successor class bodies are AST-identical to `333e410`.

## 13. Required mutation/reversal matrix

Each reversal is applied one at a time to a disposable copy of the clean
candidate, must fail for the named behavioral reason, and is fully reverted
before the next gate. Symbol-count or missing-name failures are not evidence.

| ID | Deliberate reversal | Test that must fail |
|---|---|---|
| A1-X01 | restore `raise cleanup_error from run_error` | server exact-primary identity |
| A1-X02 | restore cleanup-first ancestry constructor/contains | ancestry exact-primary identity |
| A1-X03 | restore signature authority/policy cleanup-first branch | signature exact-primary identity |
| A1-X04 | omit `from first_cleanup` while retaining primary | immediate cause identity |
| A1-X05 | discard or reorder the primary's prior effective chain | prior-chain identity sequence |
| A1-X06 | stop the session descriptor loop on first fault | persistent multi-descriptor F6 zero-debt oracle |
| A1-X07 | retry a detached FD number | ambient reused-FD survival oracle |
| A1-X08 | clear a scalar/owner before its release commits | same-owner retry and exact-current oracle |
| A1-X09 | restore `_open` memory overwrite/new acquire | F7 11,456-not-22,912 and call-count oracle |
| A1-X10 | make `_drained_commit_source_cleanup_error` unbounded | finite close-count oracle |
| A1-X11 | mark session closed while retryable logical debt remains | cleanup-pending retry oracle |
| A1-X12 | let `_run_plan` selector/process cleanup replace the body error | server body exact-primary oracle |
| A1-X13 | alter goal-directed or full-BFS continuation order | established ancestry compatibility fixtures |
| A1-X14 | retain raw policy/commit/output through the rebuilt chain | weakref/release-observer payload test |
| A1-X15 | modify an A2 or sealed/composite excluded AST body | base-vs-candidate negative-space census |

## Validation

Run from the clean implementation candidate. Set
`PYTHONDONTWRITEBYTECODE=1`; do not create `__pycache__` evidence in the
worktree.

### 14.1 Identity and patch integrity

```bash
git rev-parse HEAD HEAD^{tree} HEAD^
git merge-base --is-ancestor 333e41082ffc35c73c02a420511ae2f16839c5b8 HEAD
git diff --name-status <freeze-commit>..HEAD
git diff --binary <freeze-commit>..HEAD | sha256sum
git diff --check <freeze-commit>..HEAD
git status --short
```

The manager records the exact freeze commit containing this packet. Product
diff comparison uses that commit as `F`; base-source identity comparison uses
`333e410` as `P`. `P..F` must contain only this Markdown packet and typed JSON
companion. The candidate `C` must linearly descend from `F` with zero merges.

### 14.2 Compile and focused owners

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile \
  codefire \
  tests/test_codefire_cli.py \
  tests/test_python_http_upload_accounted_traversal.py \
  tests/test_python_ancestry_owner.py \
  tests/test_python_ancestry_borrow_handoff.py \
  tests/test_python_signature_authority.py \
  tests/test_python_signature_owner.py \
  tests/test_python_external_verification.py \
  tests/test_python_external_production.py \
  tests/test_python_signature_verification_handoff.py \
  tests/test_python_sealed_production.py

PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  tests.test_python_http_upload_accounted_traversal \
  tests.test_python_ancestry_owner \
  tests.test_python_ancestry_borrow_handoff \
  tests.test_python_signature_authority \
  tests.test_python_signature_owner \
  tests.test_python_external_verification \
  tests.test_python_external_production \
  tests.test_python_signature_verification_handoff \
  tests.test_python_sealed_production -v
```

Record selected and passed counts per module. The base has 165 methods across
these nine modules; the candidate count must grow by the exact number of new
tests, not by replacing unrelated coverage.

### 14.3 Adjacent shared-owner regressions

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  tests.test_python_signature_terminal_freeze \
  tests.test_python_sealed_signature_composite \
  tests.test_python_existing_terminal_owner \
  tests.test_python_existing_consumer_terminal_freeze \
  tests.test_python_sealed_owner \
  tests.test_python_sealed_borrow -v

PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  tests.test_codefire_cli -k upload
```

An adjacent failure does not authorize editing that test or successor. Report
the exact failure and return to manager scope adjudication unless it is an
existing A1 expectation in the closed write set.

### 14.4 Broad product and docs gates

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  discover -s tests -p 'test_python_*.py' -v

PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tests -v

./demo.sh

python3 docs/development/tools/check_source_docs.py
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_ai_contracts.py
python3 docs/development/tools/check_ai_acceleration_docs.py
python3 docs/development/tools/check_ai_development.py --timeout 180
git diff --check
git status --short
```

A command that times out, is killed, or has no unittest summary is
`unavailable`, never PASS. Preserve its log and continue only after recording
the candidate identity and resource state. Do not substitute an earlier branch
run for exact-candidate evidence.

### 14.5 Structural and negative-space proof

The result must include an AST/body comparison between
`333e410:codefire` and `C:codefire` showing:

- every changed class/function is in the A1 locus table or is a small private
  helper satisfying the shared-helper boundary;
- `_PreparedHTTPMutationResponses`, `_HTTPMutationResponseSlot`, handler
  response/wire methods, the error-origin taxonomy, and all named
  sealed/composite successors are unchanged;
- the literal `raise cleanup_error from run_error` census is zero in
  `run_server_verification` and remains zero in the two commit-source wrappers;
- no new public class/function, persisted field, HTTP status, response payload,
  resource profile, or serialized format appears; and
- `codefire` remains executable mode `100755`.

## 15. Candidate Freeze, review, and exclusions

### 15.1 Topology

Use a linear, no-merge chain:

```text
P  333e410 exact product/evidence base
|
F  this Plan/FREEZE packet commit; exactly two added docs paths
|
C  clean implementation candidate; only the closed implementation write set
|
W  result-only commit; exactly one added result path
|
R  fresh reviewer report-only commit; exactly one added review path
```

Small implementation checkpoint commits between `F` and `C` are permitted for
multi-hour continuity, but `C` is the only review subject and `F..C` must have
zero merges and no path outside the candidate write set. Any candidate byte
change after freeze creates `C2` and invalidates the old patch/evidence/review.

### 15.2 Candidate Freeze record

`W` must record:

- `P`, `F`, `C`, their full commit/tree/parent identities and zero-merge proof;
- exact `F..C` name-status, Git blob map and binary patch SHA-256;
- base and candidate mode of `codefire`;
- implementer provider/model/session/worktree and start/end clean state;
- exact test counts, durations, exit statuses and log digests;
- every A1-P and A1-X result, including unavailable evidence;
- AST changed-unit and excluded-unit comparison;
- exact ledger currents, permit counts, owner identities, FD close call counts
  and reused-FD observations for F6/F7;
- before/after durable authority hashes on failing upload/server paths;
- canonical-doc truth changes and remaining limitations;
- no A2/B3 implementation claim; and
- remaining risks and any requested successor amendment.

`W` must not modify `C` or claim its own commit identity inside `C`.

### 15.3 Reviewer exclusions and subject

The exact independent reviewer must be a fresh report-only session and
worktree. Excluded from reviewer eligibility:

- the agent/session that authored this Plan/FREEZE packet;
- the manager session that supplied the chain/ownership decisions;
- every implementation or repair session that produced any byte in `F..C`;
- any validation session that modified, amended, rebased, or restored candidate
  bytes;
- the prior Upload third-repair reviewer session;
- the prior live-route cleanup reviewer session; and
- the prior accepted-input composition writer/reviewer sessions.

Provider diversity alone does not establish independence. A new session of the
same approved provider/model is allowed if it receives only the committed
packet/evidence/candidate/result and not an inherited implementation session.

The reviewer examines `C`, uses `W` only as claimed evidence, independently
recomputes identities and reruns or verifies the required matrix, and writes
only `R`. It may not fix code/tests/docs, resolve conflicts, or classify an
observation as accepted variance on the implementer's behalf.

One fresh exact reviewer is sufficient for the frozen contract. A new public,
persisted, authority, response/wire, or state-machine effect returns to
Freeze/Plan and may require a distinct reviewer; it is not hidden inside a
generic additional-review count.

### 15.4 Review questions

The exact review must answer, with evidence:

1. Is the primary/cause/prior-chain identity sequence exact and cycle-safe?
2. Did traceback detachment still release every payload before accounting
   release observers?
3. Does F6 sweep all finite descriptor owners without raw numeric retry,
   unbounded loops, false zero, or parent-before-child release?
4. Does F7 prevent every new allocation while the old memory owner remains?
5. Are retryable process/scalar/permit owners retained until commit?
6. Are live ancestry/signature/server routes all covered, including the three
   old cleanup-first tests?
7. Did goal-directed/full-BFS, signature, server command, deadline and
   accounting semantics remain compatible?
8. Are A2/B3 and sealed/composite successors unchanged in product bytes/AST?
9. Do all five docs describe direct-live versus combined-unwired truth and the
   conservative FD limitation?
10. Did every mutation fail for the intended behavior, and is any missing
    evidence acceptance-critical?

Every observation then follows the operating protocol's impact/actionability
adjudication. Only a manager-frozen `required_correction` authorizes another
repair. A reviewer preference, finding count, style difference, or optional
abstraction does not.

## 16. A1/A2 implementation order

The A1 and A2 Plan/FREEZE documents may be prepared independently on the same
base, but product implementation is serialized because both can touch
`codefire` and canonical source docs.

1. Complete A1 through exact candidate review and manager source-acceptance
   decision.
2. Rebind the A2/B3 packet to the manager-selected A1-integrated product base,
   or create an explicit manager composition packet if A2 work already exists.
3. Recompute A2 response/wire tests against A1's exception behavior.
4. Never merge A1 and A2 by taking one whole `codefire` blob or by resolving a
   textual conflict without a composed contract review.

This packet implements no A2/B3 behavior and does not make A2 a hidden
acceptance prerequisite for the A1 candidate.

## Handoff

The implementation lane returns:

```text
Role: bounded High implementer
Objective: CFR-168 C14-LIVE-A1 only
Files changed: exact subset of the closed write set
Validation: command/count/duration/exit/log digest table
Risks: unavailable evidence and remaining conservative cleanup limits
Next: freeze C, add W, request fresh exact report-only review
```

The successful reviewed handoff still says:

- A1 candidate accepted or changes requested for the exact subject only;
- A2/B3 and successor owners remain OPEN;
- CFR-168 and CFB-138 remain nonterminal;
- canonical integration is manager-owned;
- Install/Dogfood/Evaluate are required later for the eventual runtime batch;
  and
- Close/Retrospect are not authorized.
