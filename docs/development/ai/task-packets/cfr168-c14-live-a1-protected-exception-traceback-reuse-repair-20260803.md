# CFR-168 C14-LIVE-A1 C4 Protected-Exception Traceback-Reuse Repair Freeze

Status: `ready_for_fresh_independent_preflight`

Packet ID:
`cfr168-c14-live-a1-protected-exception-traceback-reuse-repair-20260803`

Lifecycle position: the manager-reproduced C3 defect is adjudicated and frozen
as F4; fresh independent preflight PF4 is required before bounded
implementation C4

Execution roles: one fresh independent contract-preflight session, then one
bounded High implementer; manager retains candidate, integration, review, and
lifecycle authority

Exact product base: `46ed2a957fcf66821387b3414e075c98e31209be`,
tree `1bedd8c67716b3054321e4715a154f567be8f55a`, parent
`4d24460d8bb1b7c052ec465531abfdea1b4d887b`

Root issue: CFR-168 C14-LIVE-A1 remains nonterminal. This packet freezes one
required correction only. It does not source-accept C3 or a future C4, accept
A1, change canonical status, authorize integration, or close any issue.

## 0. Frozen Decision and Requirement Inventory

The complete authorization-changing inventory is:

1. **Exact defect.** On every later retention call,
   `_retain_first_cleanup_failure(first, candidate)` must clear tracebacks from
   the whole protected effective chain rooted at `first`, even when `candidate`
   is exactly `first` or another protected identity. C3 instead stops before
   clearing any traceback when the candidate identity is protected.
2. **Protected evidence.** The exact `first` object, every protected exception
   identity, and every pre-existing `__cause__` / `__context__` link on those
   objects must remain unchanged. Replacing the first error or severing,
   rebuilding, reordering, or deduplicating its protected chain is forbidden.
3. **Distinct later prefixes.** A non-protected later prefix is still detached
   only until it reaches the protected chain. Each visited prefix object's
   traceback, cause, and context are cleared. The protected join object and all
   protected links remain intact, while every protected traceback is cleared.
4. **Unchanged disjoint behavior.** A disjoint later cleanup chain continues to
   receive C3's full destructive prefix detachment, and the exact first cleanup
   object remains the retained result. The `first is None` behavior is also
   unchanged.
5. **Termination.** Protected and later graphs may be cyclic. Traversal must be
   finite, identity-based, and cycle-safe, with no payload-bearing traceback
   left on any protected object or visited non-protected prefix object.
6. **Five required cases.** Exact-first reuse, protected-tail reuse,
   distinct-prefix join, disjoint later cleanup, and cyclic first/later graphs
   each receive a distinct production test and oracle.
7. **Oracle strength.** Tests use the real helper from `codefire`, exact object
   identity and cause/context-link assertions, traceback traversal, a
   distinctive object owned only by the later raising frame, weak references,
   `gc.collect()`, and explicit mutation/reversal reasoning. Message/type-only
   assertions are not evidence.
8. **Closed implementation scope.** C4 may modify only `codefire` and
   `tests/test_python_http_upload_accounted_traversal.py`. The expected repair
   is a minimal local change inside `_retain_first_cleanup_failure` plus five
   exact tests. If that is insufficient, implementation stops and returns to
   manager Freeze.
9. **Adjudication.** Correctness and resource/memory retention are affected.
   The gap is a must-fix `required_correction`, causally classified as
   `repair_introduced_regression` with an incomplete adversarial oracle. It is
   not accepted implementation variance.
10. **Topology.** The logical sequence is
    `F -> C1 -> C2 -> C3 -> F4 -> PF4 -> C4`. F4 is this packet, PF4 is fresh
    independent preflight, and C4 is the implementation candidate. Any byte
    change after C4 creates another candidate.
11. **Validation.** C4 must pass exact compile, new and inherited focused
    tests, the nine original A1 modules with the exact expected count, static
    docs/source/AI checks, two-path and negative-space proof, diff checks, and
    clean status. Known-failing adjacent and broad gates remain excluded.
12. **Post-C4 work.** C4 is still subject to a separately frozen and
    preflighted C4-bound adjacent-oracle two-literal amendment, result-only
    evidence, and a fresh full exact review. None of those later decisions is
    made here.

## Objective

Produce one minimal C4 candidate that preserves the exact protected
first-cleanup evidence graph while dropping every traceback newly attached by
later reuse of any protected exception object, without changing any behavior
outside the frozen helper contract.

## Context

### 2.1 Immutable lineage

```text
F  = 0f28f477f61b5f64d3a700fc2ff2db91a6cfaf87
C1 = 85c2997457f81616107dfff8e515574147de9aef
C2 = 4d24460d8bb1b7c052ec465531abfdea1b4d887b
C3 = 46ed2a957fcf66821387b3414e075c98e31209be
C3 tree   = 1bedd8c67716b3054321e4715a154f567be8f55a
C3 parent = 4d24460d8bb1b7c052ec465531abfdea1b4d887b
```

The relevant repair sequence is:

| Node | Subject | Relevant effect |
|---|---|---|
| `F` | original A1 Plan/FREEZE | froze exact primary/cleanup identity, payload release, bounded cleanup, and review requirements |
| `C1` | `fix: preserve live cleanup primary errors` | introduced shared exception-chain helpers and retained the operation/first-cleanup precedence |
| `C2` | `fix: retain source cleanup cause chains` | preserved source cleanup cause tails instead of erasing them |
| `C3` | `fix: protect retained cleanup chains` | stopped later prefix detachment at identities reachable from the retained first cleanup, but failed to clear a freshly attached traceback when the later candidate itself was protected |

The C3 blobs that directly bind this packet are:

| Path | C3 blob |
|---|---|
| `codefire` | `e2db3f3a36a6e9d4af9660ac9b707156f09feff5` |
| `tests/test_python_http_upload_accounted_traversal.py` | `9a5ee670de1cbfd9074602400fab54c0f28d88b4` |
| `tests/test_python_ancestry_owner.py` | `37e292feb0b6a384c0c8ec09daba7633658dc53c` |
| `tests/test_python_signature_authority.py` | `9b8eb4ef70888e403625333e2ab4e6865c32b618` |

### 2.2 C3 helper behavior

At C3:

- `_detach_effective_exception_tracebacks` follows the effective
  `__cause__ or __context__` chain, clears only tracebacks, and terminates by
  exception identity;
- `_link_primary_cleanup_exception` intentionally rebuilds the exact
  primary/cleanup/prior ordering for the separate primary-plus-cleanup case;
  it is outside this repair; and
- `_retain_first_cleanup_failure` returns the first exact cleanup object,
  protects the identities in its effective chain, and destructively detaches a
  distinct later prefix until that prefix reaches a protected identity.

The C3 defect is the loop guard on the last helper. It requires
`id(current) not in protected` before entering the loop. Therefore a later
candidate that is already protected is returned past without even clearing its
new traceback. Python may attach a new traceback each time the same exception
object is raised, so protected identity does not imply traceback safety.

The expected correction is local: collect the protected identities in a
cycle-safe pass that also clears each protected object's current traceback,
then retain C3's bounded destructive pass over only the non-protected later
prefix. This describes the required effect, not permission to refactor either
adjacent helper or any call site.

### 2.3 Exact manager counterexample

The clean-C3 manager reproduction retains one first cleanup whose effective
chain contains a shared tail. A later cleanup raises that same exact tail
object again from `raise_shared`, causing Python to attach a fresh traceback to
the protected object. Calling the real C3 helper with the retained first error
and that exact tail returns the first error but leaves the later traceback and
its local payload reachable.

The exact observed output is frozen:

```text
retained_is_first True
later_is_shared_tail True
tail_traceback_retained True
payload_frames [('<module>', True), ('raise_shared', True)]
```

The production tests must strengthen this reproduction by making the
distinctive payload object live only in the later raising frame. Test locals
may retain only a weak reference after the raise. After the helper call,
strong temporary references and caught exceptions are deleted, `gc.collect()`
is run, and the weak reference must be dead while the exact retained exception
graph remains available for identity/link assertions.

### 2.4 Existing C3 adversarial coverage that remains binding

The implementation must rerun, not replace or weaken, these C3 tests:

- `PythonHTTPUploadAccountedTraversalTests.`
  `test_later_cleanup_detach_never_mutates_first_shared_tail`;
- `PythonAncestryOwnerTests.`
  `test_collection_allocation_failure_preserves_exact_prior_chain`;
- `PythonSignatureAuthorityTests.`
  `test_close_later_fault_cannot_mutate_first_shared_tail`;
- `PythonSignatureAuthorityTests.`
  `test_prepare_policy_preserves_generic_child_cleanup_chain`; and
- `PythonSignatureAuthorityTests.`
  `test_decoder_payload_is_replaced_but_cleanup_tail_survives`.

These establish the inherited no-mutation and exact-chain boundary. They did
not exercise reuse of an already-protected identity with a newly attached
traceback, so they are necessary but not sufficient evidence for C4.

## Scope

### 3.1 F4 packet write set

This Freeze author changes exactly:

```text
docs/development/ai/task-packets/cfr168-c14-live-a1-protected-exception-traceback-reuse-repair-20260803.md
```

F4 is docs-only and runs no product test.

### 3.2 C4 implementation write set

The implementation successor may modify exactly:

```text
codefire
tests/test_python_http_upload_accounted_traversal.py
```

Required product change:

- modify only the body of `_retain_first_cleanup_failure`;
- preserve its signature, return convention, and every call site;
- clear `__traceback__` on every protected effective-chain object on every
  later retention call, including exact-first and protected-tail reuse;
- retain C3's non-protected-prefix detachment boundary and cycle guards; and
- leave `_detach_effective_exception_tracebacks` and
  `_link_primary_cleanup_exception` byte-for-byte unchanged.

Required test change:

- add `import weakref` if needed;
- add exactly the five test methods frozen below to
  `PythonHTTPUploadAccountedTraversalTests`; and
- do not edit, rename, delete, skip, or weaken any existing test method.

### 3.3 Five exact behavior cases

| ID | Required test method | Graph and raise | Required observation |
|---|---|---|---|
| `TR1` | `test_reused_first_cleanup_object_drops_later_traceback_payload` | a mixed cause/context protected chain exists and later cleanup raises the exact `first` object again | return is `first`; every protected identity/link is exact; every protected traceback is `None`; later-frame-only payload is collectible |
| `TR2` | `test_reused_protected_cleanup_tail_drops_later_traceback_payload` | later cleanup raises an exact protected context-tail object again, with no cause overriding that effective context link | return is `first`; candidate is the exact protected tail; first-to-tail links are unchanged; all protected tracebacks are `None`; payload is collectible |
| `TR3` | `test_distinct_later_prefix_stops_at_protected_tail` | a distinct raised prefix reaches an exact protected tail through its effective chain | prefix traceback/cause/context are cleared only through the non-protected prefix; the protected tail and all protected links remain exact; all protected tracebacks are `None`; prefix and protected-frame payloads are collectible |
| `TR4` | `test_disjoint_later_cleanup_detach_is_unchanged` | a distinct later effective chain never reaches a protected identity | C3's disjoint-prefix behavior is unchanged: return `first` and destructively detach the entire later chain; the required new protected pass also clears protected tracebacks without changing protected links; every payload is collectible |
| `TR5` | `test_cyclic_reused_cleanup_graphs_drop_later_tracebacks` | protected and later graphs contain identity cycles, including a later path that reuses or joins a protected object | call terminates; return and protected identities/links are exact; no protected or visited-prefix traceback retains the payload; weak reference dies after collection |

Immediately after all later raises and immediately before the helper call, each
test snapshots the protected objects. For each protected identity it records
the exact object, `__cause__`, `__context__`, and `__suppress_context__`; every
value must be identity-equal after the helper returns. Before calling the
helper, traceback traversal must prove the intended named raising frame exists
and contains the distinctive sentinel by identity. After the call, traversal
assertions inspect every protected and visited-prefix traceback and frame
local, not only `exception.__traceback__ is None` at one root.

The reuse tests must make candidate-only clearing impossible to pass. Across
`TR1` and `TR2`, and again in `TR5`, at least one protected object other than
the candidate is deliberately re-raised through its own later frame before the
retention call. `TR3` pre-arms its protected join object and then raises the
distinct prefix from another frame. `TR4` similarly pre-arms one protected
object before raising the disjoint chain. Each armed frame owns a distinct
weak-referenceable sentinel and all strong test references are deleted before
collection. Therefore every later call must clear the whole protected chain,
not merely `candidate`, while still applying the case-specific prefix rule.

The five names and five added method count are frozen. C3 has 179 test methods
across the nine original A1 focused modules; C4 must have exactly 184. A
different count, a renamed old test, or a combined subtest that reduces the
method count is outside this packet.

### 3.4 Required reading for PF4 and C4

PF4 and the implementer read completely before deciding or mutating:

- `AGENTS.md`;
- `docs/development/ai/operating-protocol.md` task-packet, preflight, exact
  review, impact/actionability, causal-class, and repair/rereview rules;
- `docs/development/project-management.md` lifecycle and source-acceptance
  rules;
- `docs/development/ai/README.md` current governance map and static gates;
- `docs/development/ai/validation-report-template.md` evidence shape;
- `docs/development/ai/task-packets/cfr168-c14-live-a1-plan-freeze-20260802.md`;
- the exact `F..C1`, `C1..C2`, and `C2..C3` source, test, and canonical-doc
  diffs;
- C3 definitions and every call site of
  `_detach_effective_exception_tracebacks`,
  `_link_primary_cleanup_exception`, and
  `_retain_first_cleanup_failure`; and
- the complete C3 shared-tail tests in
  `tests/test_python_http_upload_accounted_traversal.py`,
  `tests/test_python_ancestry_owner.py`, and
  `tests/test_python_signature_authority.py`.

No older report, branch name, or stopped lane supersedes the exact committed
F4 packet or current lifecycle/review/adjudication policy.

## 4. Impact and Adjudication

| Axis | Frozen disposition | Reason |
|---|---|---|
| Correctness | affected | the original A1 invariant requires retained cleanup evidence without payload-bearing tracebacks |
| Resource/memory retention | affected | a later raising frame and its distinctive local remain reachable through the retained first-cleanup chain |
| Public API | unaffected | helper is private; no signature, exception type/order, CLI, or caller contract change is authorized |
| Persisted data | unaffected | no stored object, branch, receipt, nonce, schema, or migration changes |
| Authority/security | unaffected | no signature, subject, capability, validation, or mutation authority changes |
| Wire/response format | unaffected | no status, payload, envelope, JSON, HTTP, or stderr change |
| Product semantics outside C14 cleanup evidence | unaffected | only traceback release on already-retained cleanup evidence changes |

Must-fix decision: **yes**. The original A1 traceback/payload-release invariant
is violated, exact-candidate evidence is incomplete, and C3 introduced or
retained the reachable gap.

Actionability: `required_correction`, bounded to the two implementation paths
and one private helper above.

Causal class: `repair_introduced_regression` with an incomplete adversarial
oracle. The protected-chain repair added the early protected-identity stop,
while the C3 tests proved link preservation but not fresh traceback attachment
on reused protected objects. This is not `accepted_implementation_variance`, a
style preference, or permission for a generic exception-framework rewrite.

The stopped C3-bound adjacent-oracle R2 lane is non-authoritative. Its current
branch begins at C3 and no result from it may adjudicate F4 or C4. It may be
restarted only after a clean C4 exists and fresh C4-bound evidence is frozen.

## Non-Goals

C4 does not authorize:

- changing `_detach_effective_exception_tracebacks`,
  `_link_primary_cleanup_exception`, or any helper call site;
- replacing the first cleanup object, wrapping it, cloning exceptions,
  rebuilding its chain, changing precedence, or attaching later cleanup ahead
  of the first;
- changing any owner state machine, close order, retry ownership, descriptor,
  permit, process, scalar, ledger, accounting, finalizer, or exception-order
  behavior;
- changing any public surface, CLI behavior, exception type/message contract,
  HTTP response, JSON/wire format, authority decision, persisted data, source
  architecture semantics, or canonical documentation;
- editing any adjacent-oracle literal or the stopped R2 lane;
- weakening an assertion, deleting/renaming an existing test, adding a skip,
  using a message/type-only oracle, or replacing the real helper with a test
  replica;
- changing Rust, Cargo, crates, installer/packaging, A2, B3, result reports,
  review reports, issue state, canonical status, or lifecycle state; or
- treating an implementation difficulty as permission to widen scope. If the
  local helper repair is insufficient, return to manager Freeze.

## Invariants

### 6.1 Protected identity and link invariants

Let `P` be the finite identity set reached from `first` by repeatedly choosing
`__cause__ or __context__` and stopping on `None` or a repeated identity.

For a non-`None` `first`, after every call:

1. the returned object is `first` by exact identity;
2. every identity in `P` is still the same object;
3. for every object in `P`, both its `__cause__` and `__context__` references
   are exactly the pre-call references, including non-effective context fields;
4. `__suppress_context__` on every protected object is unchanged;
5. every object in `P` has `__traceback__ is None`; and
6. no later-frame payload is reachable from any traceback in `P`.

The repair may mutate no other protected exception field.

### 6.2 Later-prefix invariants

Let `L` be the sequence reached from `candidate` along the same effective-link
rule until it reaches `None`, a repeated later identity, or an identity in
`P`.

- For each non-protected object visited before a protected join, clear
  traceback, cause, and context exactly as C3 does.
- Do not follow destructive detachment into `P`.
- A disjoint or cyclic non-protected later graph receives the same finite
  C3 detachment for every visited object.
- If `candidate` itself belongs to `P`, the later prefix is empty but the
  protected traceback-clearing pass still runs.
- If `first is None`, return the exact candidate after the unchanged
  `_detach_effective_exception_tracebacks(candidate)` behavior.

Runtime complexity remains linear in the number of distinct protected and
visited later identities, with linear identity-set storage. No recursion,
unbounded retry, frame copy, side registry, or global state is allowed.

### 6.3 Repository negative space

- `codefire` remains mode `100755`.
- The only changed top-level production unit is
  `_retain_first_cleanup_failure`.
- No class or public/private sibling function changes.
- The test delta is only one import and the five new methods.
- All five canonical source documents remain byte-for-byte C3.
- All paths outside the two-path C4 write set remain byte-for-byte F4.

## 7. Required Oracle and Mutation/Reversal Matrix

The five production tests must make these single reversals load-bearing. Each
mutation is applied only in a throwaway export, never in the candidate
worktree, and must fail the named evidence boundary:

| Mutation | Expected failing evidence |
|---|---|
| restore C3's protected-candidate early return without clearing protected tracebacks | `TR1` and `TR2` retain the later-frame payload and expose a protected traceback |
| clear or rebuild protected cause/context links while clearing tracebacks | `TR1`, `TR2`, `TR3`, or `TR5` fails exact link/identity snapshots |
| continue destructive prefix detachment through the protected join | `TR3` fails because the protected tail link is severed |
| stop detaching a disjoint later chain or preserve its later links | `TR4` fails the unchanged-C3 oracle |
| remove either identity-cycle guard or use equality instead of identity | `TR5` fails or exceeds the bounded mutation-run timeout |
| clear only the candidate traceback instead of the whole protected chain | `TR1`, `TR2`, or `TR5` finds another protected traceback/payload |
| use only exception type/message assertions or keep a strong test reference to the payload | oracle audit fails even if the test process reports green |

The implementation handoff records, for every mutation, the throwaway source
digest, exact focused command, exit status or timeout, and the assertion that
failed. Mutation runs are supporting evidence; the unmodified exact C4 must be
rerun afterward and remain clean.

## 8. Preflight and Linear Candidate Topology

The logical topology is fixed:

```text
F -> C1 -> C2 -> C3 -> F4 -> PF4 -> C4
```

- `F`, `C1`, `C2`, and `C3` are the immutable identities in section 2.1.
- `F4` is one docs-only commit adding exactly this packet on top of C3.
- `PF4` is a fresh independent contract-preflight decision bound to exact C3
  and exact F4. It re-derives the five cases, identity/link boundary, test
  strength, closed scope, validation, and forbidden gates. It may not mutate
  product or packet bytes and must return to manager Freeze on any unresolved
  authorization-changing choice.
- `C4` is one clean implementation commit linearly descending from F4, created
  only after PF4 passes, changing exactly the two implementation paths.

PF4 is a lifecycle/evidence node, not a source-changing commit in the C4
candidate ancestry. If its report is committed, it stays on a manager-owned
report-only branch and is not cherry-picked into `F4..C4`; otherwise the
required exact two-path diff would be false.

Any byte change after C4, including formatting, test edits, evidence inserted
into product files, an amend, rebase, conflict resolution, or regenerated
literal, creates a new candidate and invalidates the old C4-bound evidence.

C4 remains subject to all of the following future steps:

1. separately freeze and independently preflight the C4-bound adjacent-oracle
   two-literal amendment;
2. run the newly authorized adjacent gate only after that amendment;
3. record result-only evidence without modifying C4; and
4. obtain a fresh full exact review of the final frozen subject and evidence.

This packet does not accept C3, C4, A1, CFR-168, or any later amendment.

## Validation

Set `PYTHONDONTWRITEBYTECODE=1`. A timeout, killed process, missing unittest
summary, or command run against another tree is `unavailable`, never PASS.
Preserve exact commands, durations, exit statuses, selected/passed counts, and
log SHA-256 values.

### 9.1 F4 Freeze-author validation

The F4 author may run Git identity and static docs/source/AI checks only. No
product test, Python product compile, broad discovery, demo, Cargo, or Rust
command is authorized in this lane.

```bash
git rev-parse HEAD HEAD^{tree} HEAD^
git diff --check

timeout 900s python3 docs/development/tools/check_source_docs.py
timeout 900s python3 docs/development/tools/check_project_docs.py
timeout 900s python3 docs/development/tools/check_ai_contracts.py
timeout 900s python3 docs/development/tools/check_ai_acceleration_docs.py
timeout 900s python3 docs/development/tools/check_ai_development.py \
  --timeout 600 --skip-cli
```

After the single F4 commit, prove:

```bash
git rev-parse HEAD HEAD^{tree} HEAD^
git merge-base --is-ancestor \
  46ed2a957fcf66821387b3414e075c98e31209be HEAD
git rev-list --count \
  46ed2a957fcf66821387b3414e075c98e31209be..HEAD
git rev-list --merges \
  46ed2a957fcf66821387b3414e075c98e31209be..HEAD
git diff --name-status \
  46ed2a957fcf66821387b3414e075c98e31209be..HEAD
git diff --check \
  46ed2a957fcf66821387b3414e075c98e31209be..HEAD
git status --porcelain
sha256sum \
  docs/development/ai/task-packets/cfr168-c14-live-a1-protected-exception-traceback-reuse-repair-20260803.md
```

Required result: one non-merge commit; one added path, exactly this packet;
clean status.

### 9.2 C4 compile and exact traceback-reuse tests

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile \
  codefire \
  tests/test_python_http_upload_accounted_traversal.py

PYTHONDONTWRITEBYTECODE=1 python3 -m unittest -v \
  tests.test_python_http_upload_accounted_traversal.PythonHTTPUploadAccountedTraversalTests.test_reused_first_cleanup_object_drops_later_traceback_payload \
  tests.test_python_http_upload_accounted_traversal.PythonHTTPUploadAccountedTraversalTests.test_reused_protected_cleanup_tail_drops_later_traceback_payload \
  tests.test_python_http_upload_accounted_traversal.PythonHTTPUploadAccountedTraversalTests.test_distinct_later_prefix_stops_at_protected_tail \
  tests.test_python_http_upload_accounted_traversal.PythonHTTPUploadAccountedTraversalTests.test_disjoint_later_cleanup_detach_is_unchanged \
  tests.test_python_http_upload_accounted_traversal.PythonHTTPUploadAccountedTraversalTests.test_cyclic_reused_cleanup_graphs_drop_later_tracebacks
```

Required result: exactly 5 selected, exactly 5 passed.

### 9.3 Existing C3 shared-tail and chain tests

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest -v \
  tests.test_python_http_upload_accounted_traversal.PythonHTTPUploadAccountedTraversalTests.test_later_cleanup_detach_never_mutates_first_shared_tail \
  tests.test_python_ancestry_owner.PythonAncestryOwnerTests.test_collection_allocation_failure_preserves_exact_prior_chain \
  tests.test_python_signature_authority.PythonSignatureAuthorityTests.test_close_later_fault_cannot_mutate_first_shared_tail \
  tests.test_python_signature_authority.PythonSignatureAuthorityTests.test_prepare_policy_preserves_generic_child_cleanup_chain \
  tests.test_python_signature_authority.PythonSignatureAuthorityTests.test_decoder_payload_is_replaced_but_cleanup_tail_survives
```

Required result: exactly 5 selected, exactly 5 passed.

### 9.4 Nine original A1 focused modules

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest -v \
  tests.test_python_http_upload_accounted_traversal \
  tests.test_python_ancestry_owner \
  tests.test_python_ancestry_borrow_handoff \
  tests.test_python_signature_authority \
  tests.test_python_signature_owner \
  tests.test_python_external_verification \
  tests.test_python_external_production \
  tests.test_python_signature_verification_handoff \
  tests.test_python_sealed_production
```

Clean C3 contains exactly 179 test methods across these modules. Because C4
adds exactly five and changes none, the required C4 result is exactly 184
selected and exactly 184 passed. Record per-module and total counts.

### 9.5 C4 docs/source/AI static checks

Each process receives at least 600 seconds; use the larger outer bound below.

```bash
timeout 900s python3 docs/development/tools/check_source_docs.py
timeout 900s python3 docs/development/tools/check_project_docs.py
timeout 900s python3 docs/development/tools/check_ai_contracts.py
timeout 900s python3 docs/development/tools/check_ai_acceleration_docs.py
timeout 900s python3 docs/development/tools/check_ai_development.py \
  --timeout 600 --skip-cli
```

### 9.6 Exact F4..C4 identity, two-path diff, and clean state

The manager records exact `F4` and `C4` identities. Required proof:

```bash
git rev-parse "$F4" "$F4^{tree}" "$F4^"
git rev-parse "$C4" "$C4^{tree}" "$C4^"
git merge-base --is-ancestor "$F4" "$C4"
test "$(git rev-list --count "$F4..$C4")" -eq 1
test -z "$(git rev-list --merges "$F4..$C4")"
git diff --name-status "$F4..$C4"
git diff --binary "$F4..$C4" | sha256sum
git diff --check "$F4..$C4"
git ls-tree "$F4" -- codefire \
  tests/test_python_http_upload_accounted_traversal.py
git ls-tree "$C4" -- codefire \
  tests/test_python_http_upload_accounted_traversal.py
git status --porcelain
```

The exact name-status is:

```text
M	codefire
M	tests/test_python_http_upload_accounted_traversal.py
```

Status must be empty. `codefire` mode remains `100755`.

### 9.7 Structural negative-space proof

The result evidence must compare parsed top-level AST units from
`F4:codefire` and `C4:codefire` and show the changed production-unit set is
exactly:

```text
_retain_first_cleanup_failure
```

It must also prove:

- `_detach_effective_exception_tracebacks` and
  `_link_primary_cleanup_exception` are byte-for-byte F4;
- every `_retain_first_cleanup_failure` call site is byte-for-byte F4;
- no other function, class, method, assignment, import, public name, or
  executable mode changed in `codefire`;
- the test AST adds exactly the five frozen methods, with no removed or changed
  existing method; the only non-method test-file delta is `import weakref`;
- no skip decorator, expected-failure marker, broad mock, copied helper, or
  message/type-only substitute was added;
- `git diff --name-only F4..C4` contains exactly the two authorized paths; and
- every forbidden surface in Non-Goals is absent from the patch.

### 9.8 Commands explicitly forbidden in the C4 implementation lane

Do not run:

- the known-failing C3-bound adjacent gate;
- any restarted adjacent-oracle R2 command or two-literal correction;
- `python3 -m unittest discover -s tests -p 'test_python_*.py' -v`;
- `python3 -m unittest discover -s tests -v`;
- any other full-repository or broad Python discovery;
- `./demo.sh`;
- Cargo, Rust, installer, or packaging validation.

Those gates run only after the C4-bound two-literal adjacent-oracle correction
is separately frozen and independently preflighted. A forbidden run does not
expand implementation authority or become acceptance evidence.

## Handoff

### F4 Freeze-author handoff

Report exactly:

```text
Role: bounded Freeze/Plan docs synchronizer
Objective: freeze C14-LIVE-A1 C4 protected-exception traceback-reuse repair
Files changed: exact one packet path
Commit: F4 commit/tree/parent
Packet digest: SHA-256 of the committed packet bytes
Validation: Git identity, one-path proof, diff check, static docs/source/AI checks
Risks: no product tests by design; PF4, C4, amendment, evidence, and review pending
Next: fresh independent PF4 against exact F4 and C3
```

### PF4 handoff

PF4 reports pass or return-to-Freeze, exact F4/C3 identities, every
authorization-changing question considered, unresolved risks, and whether C4
may start. PF4 does not modify product bytes or claim implementation success.

### C4 implementation handoff

```text
Role: bounded High implementer
Objective: C14-LIVE-A1 protected-exception traceback-reuse correction only
Files changed: codefire; tests/test_python_http_upload_accounted_traversal.py
Candidate: C4 commit/tree/parent and F4..C4 patch SHA-256
Validation: exact command/count/duration/exit/log-digest table; mutation table
Risks: unavailable evidence and any unresolved payload/link/cycle concern
Next: freeze C4; separately freeze/preflight the C4-bound adjacent amendment
```

No handoff may claim candidate acceptance, A1 acceptance, integration, issue
closure, or authority to run the excluded adjacent/broad gates.
