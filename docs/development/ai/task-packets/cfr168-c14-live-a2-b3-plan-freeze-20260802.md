# CFR-168 C14-LIVE-A2 / B3 post-effect response lifecycle — manager Plan/FREEZE packet

Status: ready

Packet ID: `cfr168-c14-live-a2-b3-plan-freeze-20260802`

Revision: **2026-08-03 A1-base rebind**. The frozen design is unchanged; only
the base authority is rebound onto the accepted `C14-LIVE-A1` source package.

Role: manager Plan/FREEZE authority; a later, separate lane may implement only
after this packet is integrated and the manager names its exact launch base.

Typed companion:
`docs/development/ai/task-packets/json/cfr168-c14-live-a2-b3-plan-freeze-20260802.json`

Immutable product base `P1`: `1e8dd545f27bd1f22879b067ad0e1b3f4e2ecbc4`
(tree `fa59fe69ba69bd199ef2cf3757679c98668a0812`).

Decision: **AUTHORIZE_BOUNDED_IMPLEMENTATION_AFTER_PACKET_INTEGRATION**.

This commit is documentation-only. It does not authorize this packet-writing
lane to mutate product, tests, source docs, issue state, installation, or
dogfood state. The implementation authorization becomes usable only when a
manager supplies the full launch-base commit `L` satisfying the base rule in
this packet. Until then, implementation is forbidden.

### Superseded base authority — retained, not erased

The historical packet is retained as exact prior authority. It is superseded
only as *base*; its design, matrix, hazards, scope, and non-goals remain in
force.

| Historical identity | Exact value |
|---|---|
| Historical packet commit | `e4d85f2158a3f4e353042ab352b749fa2a69eed3` |
| Historical packet tree | `0230e537abcd410ecd4fe582ad431417e4618570` |
| Historical packet parent (`P0`) | `333e41082ffc35c73c02a420511ae2f16839c5b8` |
| Historical Markdown blob / SHA-256 | `7c1b9640aa1006a62ca653a54cbb3071dd2670ea` / `be8438198330d99e5ec35a72dc03df227a147a18a114846605eed8e9d9712563` |
| Historical JSON blob / SHA-256 | `eb4be319786ecf0a98260667c1c3c12911177b7b` / `1636cd49057def6303cd8c6fb85b2ee6065d2c7aadf977956e61939b3ee0af6c` |
| Superseded product authority `P0` | `333e41082ffc35c73c02a420511ae2f16839c5b8`, tree `c0ab3a09e4731ca27d43023be72e577487b11600` |
| Rebind launch commit | `615da7ae917d2a17d3bc97b00b8b0604092ab6ca`, tree `54ea713028b84cc1b36b795486072b38b35de263`, parent `1e8dd545f27bd1f22879b067ad0e1b3f4e2ecbc4` |

The rebind launch commit cherry-picks the two historical packet paths onto the
accepted A1 line; both blobs are byte-identical to `e4d85f21…`. Every `P0`
line number, blob, and segment hash below is retained as historical evidence
and is **not** a current-source claim.

Why the rebind is required: the historical base rule demanded an execution base
with no product difference from `P0`. After A1 acceptance no such base can also
contain the required A1 product/test changes, so that rule became
unsatisfiable rather than merely stale. Whole-file `codefire` selection and
textual conflict resolution outside this contract remain forbidden.

## Objective

Repair the accepted Python HTTP upload mutation-response lifecycle at the
`C14-LIVE-A2` / `B3` post-effect boundary without widening into the still-open
pre-effect, MR-write, or global-C14 work.

The bounded repair must make one selected, pre-encoded response survive every
post-effect cleanup path; keep the exact selected owner in the mutation slot;
let wire publication borrow rather than take that owner; separate the socket
write boundary from accounting cleanup; give cleanup one synchronous retry on
the same non-consuming owner; preserve exactly one response attempt; and make
the request observer report truthful accounting and wire state.

The implementation must not infer when a durable filesystem byte actually
became visible. It instead uses the conservative one-way marker `M`, named
`EFFECT_MAY_HAVE_STARTED`, at the exact caller-observable boundary frozen below.
Before `M`, an effect is proved absent. At and after `M`, an effect cannot be
ruled out. That distinction determines whether cleanup failure suppresses all
publication or remains secondary to the frozen success/durable response.

This packet assigns the phase index and owner that the historical
shared-primitives preflight left unassigned as `B3`. It does not rewrite that
preflight's historical `BLOCK`, and it does not claim that `B3` is repaired,
reviewed, integrated, installed, dogfooded, or closed.

## Authority and exact base

### Immutable source authority `P1`

`P1` is the manager adjudication commit that accepts the `C14-LIVE-A1` source
package. It replaces `P0` as the single product authority for A2/B3.

| Identity | Exact value at `P1` | Value at superseded `P0` |
|---|---|---|
| Commit | `1e8dd545f27bd1f22879b067ad0e1b3f4e2ecbc4` | `333e41082ffc35c73c02a420511ae2f16839c5b8` |
| Tree | `fa59fe69ba69bd199ef2cf3757679c98668a0812` | `c0ab3a09e4731ca27d43023be72e577487b11600` |
| Parent | `7bd4594f01db6d8298ca242db04b0f71ce34f040` | `808513a65dbfbbffff8123792d59d9eadfd875b4` |
| Subject | `docs(cfr168): accept C14 A1 source package` | `docs(cfr168): synchronize accepted source state` |
| `codefire` blob/mode | `15c7ebf5e2c53216c1aaa1add12070cffc12cbdd`, `100755` | `73b1f368f9c1591be6fd9219b839351878b82b9a`, `100755` |
| `tests/test_codefire_cli.py` blob | `b23c43ced6ccb69c0bb553f9856c1a7b2145e770` (unchanged) | `b23c43ced6ccb69c0bb553f9856c1a7b2145e770` |
| `source-architecture.md` blob | `ea54f24b70e3c2ba36896e620d00c597c6e0a6bb` | `fe2b3b74f2beaf30218333f14e05b6e4e6d4e699` |
| `command-source-trace.md` blob | `0c1f24c25ab981de304591e7ab8dd9aed7c47ecf` | `361a5de8f9f881fbedf35ed5a833baade9529e38` |
| `source-data-model-catalog.md` blob | `d687883f1630508319b29ff98d20b8e164d474bf` | `2b68347eaa357f187f794c2a76992401743929eb` |
| `change-playbook.md` blob | `8f460e1315b5dc51b53325e7d450aaf7845d2005` | `18d882813e02ed8cd795aa1e46d8cac6a09be4ef` |
| `cfr-168.md` blob | `29df10886dab2d5950df907d31238e8ea5ce1a8d` (unchanged) | `29df10886dab2d5950df907d31238e8ea5ce1a8d` |

`tests/test_python_http_post_effect_response.py` and
`adr/ADR-021-python-http-mutation-response-lifecycle.md` are absent at `P0`,
`P1`, and the rebind launch commit; both remain create-paths.

`P0..P1` is 11 commits, zero merges, and `P0` is a strict ancestor of `P1`.

### A1 product/test lineage bound by `P1`

| Node | Commit | Tree | Exact scope |
|---|---|---|---|
| C4 (A1 product candidate) | `19010f742d358c5c03e19d091c4249420f9e53b9` | `a0ec4ee256f63c7fe7de438eb5c978c918b3d22f` | two paths: `codefire`, `tests/test_python_http_upload_accounted_traversal.py` |
| O1 (test-only correction) | `46d98466c702f111b58a3c1ea080518f75a59580` | `626b28d1989ef21bbc23f94288845be5b0082c90` | one path, two expected literals in `tests/test_python_existing_terminal_owner.py`; zero product bytes |
| R5 (exact review, ACCEPT) | `7bd4594f01db6d8298ca242db04b0f71ce34f040` | `d3bd3a2c60fab7e34e25726305733bf91a2fe134` | adds only `review.md`; raw SHA-256 `2facd0caef19f993f7e33e5be07a6f4e80ee4b7855756f6d1b49d584565d1a08` |
| Manager decision (`P1`) | `1e8dd545f27bd1f22879b067ad0e1b3f4e2ecbc4` | `fa59fe69ba69bd199ef2cf3757679c98668a0812` | adds only `manager-adjudication.md`; `MANAGER_ADJUDICATION: ACCEPT_A1_SOURCE_PACKAGE` |

Changed `P0..P1` paths: `codefire`; nine `tests/test_python_*.py` files; four
canonical source docs; `docs/development/known-limitations.md`; and seven
added A1 packet/result documents. No other path differs.

#### A2-owned surfaces are byte-identical and AST-identical at `P1`

Recomputed in this rebind, not inherited from R5. Every historical `P0` text
hash below reproduces exactly, and every `P1` value equals it. Segments use the
`ast.get_source_segment` convention, so a nested method's span begins at its
`def` keyword.

| Surface | `P0` lines | `P1` lines | Bytes | Text SHA-256 (`P0` = `P1`) | AST SHA-256 (`P0` = `P1`) |
|---|---|---|---:|---|---|
| `_PreparedHTTPResponseOwner` | 16037-16089 | 16512-16564 | 1748 | `82d4ca7bb9df993a4e825c89acf12e973c46921925bcacdb1673d14efae63c04` | `12e71966c4abc915e53eeb42c9c5fdf64c011a385b26f59931b42c6bab16fce3` |
| `_PreparedHTTPMutationResponses` | 16092-16243 | 16567-16718 | 5769 | `16a4501d35455d482e1b1e6bd4b16bd661b0c18cdb4afb86c7afcef5353cf25c` | `2aa732efbcc94152af65cfe4e2bc4ae83e7d098328c55cd17cae3b75302ea6ca` |
| `_HTTPMutationResponseSlot` | 16256-16325 | 16731-16800 | 2635 | `3c3fae78db5e3a61c6a9c33ea03dacab5111ba094284eda3a47bd0c1ea43d9b9` | `0435a770de4e1cb3164cd579172a57b457cdc9901e49f2b0b823ba13e49368a7` |
| `handle_http_upload` | 14720-15226 | 15195-15701 | 21581 | `b9d032b67ba475fbe2baacb9927fb442614b66b366cc443ce24366d255073732` | `7647655fa70f2be8b933f2e4a1df113cea01aead53bd511ee64eec19e17dd42c` |
| `publish_prepared` | 17221-17245 | 17696-17720 | 1168 | `99af445f34cce9ca80b5aafe8988d63efec9c8118979f09a5cb793887e2d1940` | `1ddc11226e388aee5f4e164683f8313c89484f9f19090ec749c27564937e178c` |
| `handle_one_request` | 17264-17419 | 17739-17894 | 7834 | `af8a6a832d3845a236e1f983107dc72b5dec886c8babc90e026b351593eaf007` | `8c6ede01c4b5f6c417ff970d4039372111866cc910b68b32b4c4f779965dd60b` |
| `do_POST` | 17635-17867 | 18110-18342 | 11265 | `ff31cb1ffc9888836553de36cab1b340b43f04f7038b49b1d581e85bae301e1a` | `3668ac022cda54ba1fc418a11f319f94c1c59326ac48ee3c1767c7555edcdbed` |

Every A2-owned surface moves by exactly `+475` lines and changes no byte. No
`P0..P1` `codefire` diff hunk intersects any A2-owned `P0` span; the changed
old-side range is 5003-32673 with a gap across 14720-17867.

#### Classification of every changed `P0..P1` product unit

`codefire` holds 1331 AST units at `P0` and 1337 at `P1`: 6 added, 0 removed,
43 AST-changed, 0 text-only-changed. All 49 are A1-owned pre-effect
cleanup/authority units; none is A2-owned.

| A1 owner group | Units | A2 ownership |
|---|---:|---|
| New exception helpers `_detach_effective_exception_tracebacks`, `_link_primary_cleanup_exception` (+ `capture`), `_retain_first_cleanup_failure` | 4 added | not A2-owned |
| `AccountedServerVerification.close.attempt`, `SealedCommitValidationSession.close.attempt` | 2 added | not A2-owned |
| `AccountedRemoteSignaturePolicyAuthority` and 9 methods | 10 changed | not A2-owned |
| `AccountedCommitSignaturePolicy` and 6 methods | 7 changed | not A2-owned |
| `AccountedServerVerification` and 10 methods, `run_server_verification` | 12 changed | not A2-owned |
| `AccountedHTTPUploadAncestry` and 4 methods, `is_ancestor_from_objects_accounted`, `_detach_ancestry_exception_chain` | 7 changed | not A2-owned |
| `_AccountedUploadCommitSource` and 2 methods, `_drained_commit_source_cleanup_error`, `enforce_remote_commit_signature_policy_accounted` | 5 changed | not A2-owned |
| `SealedCommitValidationSession.close`, `SealedCommitValidationSession`, `validate_sealed_commit` | 3 changed | not A2-owned |

Test units changed by A1 span nine `tests/test_python_*.py` files (94 added and
28 AST-changed units). Zero added or removed test lines across all nine files
reference `_PreparedHTTPResponseOwner`, `_PreparedHTTPMutationResponses`,
`_HTTPMutationResponseSlot`, `handle_http_upload`, `publish_prepared`,
`handle_one_request`, `do_POST`, `select_success`, or `select_durable`.
`tests/test_codefire_cli.py`, which carries the two bound A2 focused oracles,
is byte-identical at `P1`.

#### Changed callees reachable from `handle_http_upload` — recompute obligation

`handle_http_upload` is byte-identical but is **not** behaviorally frozen,
because four functions it calls changed under A1. At `P1`:

| Changed callee | Call line at `P1` | Position relative to the `fence_first_effect` calls at 15401 / 15455 / 15620 |
|---|---:|---|
| `validate_sealed_commit` | 15309 | before every fence — strictly pre-effect |
| `enforce_remote_commit_signature_policy_accounted` | 15364 | before every fence — strictly pre-effect |
| `run_server_verification` | 15374 | before every fence — strictly pre-effect |
| `is_ancestor_from_objects_accounted` | 15481 | after the layout-bootstrap fence, before the publication-effect fence and before `select_success`/`select_durable` at 15657-15668 |

A1's new helpers preserve the exact primary exception object, its type, and its
`str()`; they set `__traceback__` to `None` and rewrite
`__cause__`/`__context__`/`__suppress_context__` on the retained chain. That is
consistent with `D9`'s pre-`M` row and with `T01`'s "primary identity
preserved" oracle, so no frozen A2 decision is contradicted. It does change the
*observable chain and traceback shape* in the pre-`M` arms `T01` and `T13`
exercise, and `is_ancestor_from_objects_accounted` sits in the region where
`D1`/`H01`/`H02` require the implementer to fix `M` against live source.
Baselines must therefore be recomputed on `L` under the rule below; they must
not be inherited from `P0`.

### Execution-base rule `L`

This file cannot contain the commit that contains itself. The manager launch
prompt must therefore name one full 40-character execution base `L`. `L` must:

1. contain this rebound Markdown packet and its typed companion byte-for-byte;
2. contain the accepted independent rebind preflight report for this packet;
3. descend linearly from `P1` with no merge commit;
4. differ from `P1` only by the rebound Plan/FREEZE packet paths, that
   preflight report, and explicitly named management-only result or
   adjudication records — each named in the launch prompt by exact path;
5. contain no product, test, source-doc, ADR, schema, packaging, or
   issue-detail difference from `P1`; and
6. be clean before the implementation worktree is created.

The implementer must stop if any predicate fails. Editing the typed
`base_revision` to make a generic launcher accept a different base is not an
alternative. `P1` is immutable product authority; `L` is the manager-named
packet-containing execution base. `P0` satisfies no predicate above and is
never a valid `L`.

### Bound evidence

Every digest below was recomputed at `P1` and is unchanged from `P0`; all five
bound evidence blobs are byte-identical across the A1 line.

| Evidence | SHA-256 at `P0` and `P1` | Use |
|---|---|---|
| `docs/development/ai/swarm-results/cfr168-python-http-shared-primitives-preflight-claude-20260731/preflight.md` | `ca1a1451bfc1b25ed1c54f1b012d4d0e508ef0a96f88401e61284c6d156a4441` | Historical `B3`, `C11`-`C14`, and `RetryableCleanup` evidence. Its `BLOCK` remains historical fact. |
| `docs/development/ai/swarm-results/cfr168-python-composition-exact-review-claude-20260802/review.md` | `a3835f3720983cee72a3368fcc149158a44c6bb3ea1d3c89cbca8432503fadd5` | Exact ACCEPT of the composed Python input and proof that the B3 surfaces were intentionally unchanged. |
| `docs/development/ai/swarm-results/cfr168-python-http-accepted-input-composition-20260802/result.md` | `02a759c40d11c9b79d52872104b53da79c9de069629c8b63aa9bb610adcce3e3` | Routes `B3` and durable-contract work to `C14-LIVE-A2`. |
| `docs/development/ai/swarm-batches/cfr168-rust-storage-review-closure-repair-manager-freeze-20260802.md` | `913b0a0b29dc620f60f6cc31013f5a9b59a064e680d64919521eb0edb3462348` | Later manager correction: do not invent a seventh publication terminal; accounting-complete remains `complete`. |
| `adr/ADR-020-decision-equivalent-work-budget.md` | `4566de104181bc94f7d7fd41ae1b59b8c93b267665898f2629a3ff6c58ce5ab9` | Decision-work terminal vocabulary and mutation/deadline boundary. |

The later manager correction supersedes only the historical preflight's literal
`C11` demand that every failed publication/cleanup terminal be “never
`complete`”. It does not erase the preflight, weaken the one-response rule, or
remove the requirement to expose residual ownership.

## Reproduced root cause

Every `codefire:NNNNN` coordinate in this section is a **`P0` line number**,
retained as historical evidence. The named AST units are byte-identical at
`P1`, but each sits `+475` lines lower. The implementer must resolve every
locus by symbol on `L` and must never treat a `P0` line number as a
current-source claim.

The exact accepted base is not merely missing a defensive check. Ownership is
handed off in the wrong order across three nested lifetimes.

1. `_PreparedHTTPMutationResponses._select` records `selection` at
   `codefire:16143`.
2. It then calls the rejected owner's fallible `close()` at `codefire:16144`.
3. The selected owner has not yet reached `_HTTPMutationResponseSlot.selected`;
   the pair still points at both owners.
4. If rejected cleanup fails, `do_POST` enters its generic exception arm and
   calls `mutation_slot.close()` at `codefire:17812-17817`.
5. `_HTTPMutationResponseSlot.close` calls the pair's generic `close` at
   `codefire:16310-16319`, which can destroy the response already selected by
   the durable operation.
6. A second generic slot close exists in the route-finally path at
   `codefire:17849-17857`.

The live Upload route happens to retry `select_success` once at
`codefire:15184-15192`; that masks only a one-shot success-selection cleanup
fault. `select_durable` is called once at `codefire:15193`, persistent faults
still lose ownership, and a later route-finally fault can still destroy the
selected payload.

The wire boundary has a separate defect. `publish_prepared` places both socket
writes and `_PreparedHTTPResponseOwner.publish`'s accounting cleanup inside
`transport_boundary` at `codefire:17227-17240`. A post-write accounting
`OSError` can therefore be marked as transport even though the socket write
already returned. That classification obscures cleanup debt and tempts a
second-response path.

Exact accepted AST source segments: see the `P0`/`P1` segment table under
*A2-owned surfaces are byte-identical and AST-identical at `P1`*. Text and AST
hashes are the current authority at `P1`; only the line spans moved.

An implementer may change these segments only to realize this packet. Segment
movement from formatting is not itself a defect, but every semantic delta must
map to a frozen transition, oracle, or documentation obligation below. Before
editing, the implementer must re-derive each segment on `L` and confirm the
seven text/AST hashes above still hold; a mismatch is a base violation, not a
licence to proceed.

## Frozen design

### D1 — exact phase marker `M`

`M` is the one-way state `EFFECT_MAY_HAVE_STARTED`.

It is set exactly:

```text
last complete pre-effect authority/subject fence
-> consume/seal every prepared-response permit
-> set EFFECT_MAY_HAVE_STARTED
-> immediately invoke the first fallible effect
```

For the live Upload route, the first fallible effect is the first call that may
write durable object/publication state after `fence_first_effect`. Setting one
local scalar or enum state is permitted between the response seal and that
call; no callback, allocation proportional to input, cleanup, filesystem call,
deadline poll, encoding, or authority reread may intervene.

`M` is deliberately conservative. It is not evidence that an effect happened,
that a publication was confirmed, that a client received bytes, or that a
transaction committed. It establishes only these two propositions:

- before `M`, the route proves that no authoritative effect began;
- at or after `M`, the route can no longer prove that no effect began.

The actual first durable byte is not caller-observable and must not be guessed
from exception class, traceback, return timing, or filesystem inspection.

### D2 — pair state machine

The success/durable pair has exactly these logical states:

```text
OPEN
  -> SEALED
  -> SELECTING_SUCCESS | SELECTING_DURABLE
  -> SELECTED_SUCCESS  | SELECTED_DURABLE
  -> CLEANUP_PENDING
  -> CLOSED
```

| State | Pair ownership | Allowed transition | Failure rule |
|---|---|---|---|
| `OPEN` | success owner, durable owner, response permit, header permit | seal or pre-effect abort | Abort retains every failed owner for retry; no response may publish. |
| `SEALED` | success owner and durable owner; no open response permits | choose one direction exactly once, or pre-effect abort | Opposite selection is forever invalid after choice begins. |
| `SELECTING_SUCCESS` / `SELECTING_DURABLE` | both pointers are still locally known while the chosen direction is validated | perform the non-fallible destination-first handoff | No rejected cleanup runs before handoff. |
| `SELECTED_SUCCESS` / `SELECTED_DURABLE` | the slot owns the selected owner; the pair owns only the rejected owner | enter rejected-owner cleanup | Generic pair cleanup must not see or close the selected owner. |
| `CLEANUP_PENDING` | the pair retains the exact rejected owner until release succeeds | initial close, then at most one synchronous retry on the same owner | A fault is secondary after `M`; it never revokes the slot's selection. |
| `CLOSED` | no owner and no permit | none | Re-entry is idempotent only for cleanup; selection cannot be repeated or changed. |

The selected-owner handoff is destination-first and precedes the first fallible
rejected-owner cleanup. In concrete terms, the exact owner reference reaches
`_HTTPMutationResponseSlot.selected`, the pair's selected-side reference is
cleared, and the immutable selection direction is recorded before
`rejected.close()` can run. Those local reference assignments are one logical
handoff and must not invoke user code.

If rejected cleanup then fails, the slot still owns the selected payload and
also retains the pair containing only the rejected cleanup owner. The selected
payload remains eligible for the frozen post-`M` wire attempt. The pair cannot
reconstruct, re-encode, clone, or reacquire either response.

### D3 — mutation-slot state machine

The mutation slot has exactly these logical states:

```text
EMPTY
  -> PREPARED
  -> SEALED
  -> EFFECT_MAY_HAVE_STARTED
  -> SELECTED
  -> WIRE_STARTED
  -> WIRE_COMPLETE
  -> CLEANUP_PENDING
  -> CLOSED
```

The failure branches are explicit:

```text
PREPARED | SEALED
  -> abort_before_effect
  -> CLEANUP_PENDING
  -> CLOSED                         (zero wire bytes)

EFFECT_MAY_HAVE_STARTED | SELECTED | WIRE_STARTED | WIRE_COMPLETE
  -> finish_after_effect
  -> CLEANUP_PENDING
  -> CLOSED                         (never a second document)
```

| State | Slot ownership and meaning |
|---|---|
| `EMPTY` | No pair and no selected owner. |
| `PREPARED` | Owns the unsealed success/durable pair. |
| `SEALED` | Owns the sealed pair; all response work needed after `M` was reserved and consumed. |
| `EFFECT_MAY_HAVE_STARTED` | `M` is set. An effect cannot be ruled out, but publication is not yet confirmed. |
| `SELECTED` | Owns exactly one immutable selected owner and any rejected-owner cleanup remainder. |
| `WIRE_STARTED` | The first header write has been attempted. A second HTTP document is permanently forbidden. |
| `WIRE_COMPLETE` | Header and body write calls returned. This is not an assertion that the peer received or flushed them. |
| `CLEANUP_PENDING` | Owns every exact accounting remainder and bounded secondary diagnostic until cleanup succeeds or the request is terminally observed. |
| `CLOSED` | No open permit, no current debt, no response payload, and no request-to-request slot state. |

The implementation may encode these states with an enum or equivalent private
state whose transitions are asserted. A loose set of independent booleans is
acceptable only if tests prove the same transition graph and impossible-state
rejection. There is no transition from `WIRE_STARTED` back to a selectable or
send-json state. If both cleanup attempts fail, the request is terminally
observed while the slot remains truthfully in `CLEANUP_PENDING`; `CLOSED` is
reachable only after the exact debt is actually released and is never forged as
a prerequisite for returning from the failed request.

### D4 — two cleanup APIs, never one ambiguous `close`

The slot must expose conceptually separate operations:

- `abort_before_effect`: legal only before `M`; disposes both response choices,
  preserves the route's original primary exception, publishes nothing, and
  closes the connection if disposal cannot be made exact;
- `finish_after_effect`: legal at/after `M`; never destroys the selected owner
  before its one wire attempt, treats cleanup faults as secondary, and never
  calls `send_json` or selects a different response after `WIRE_STARTED`.

Names may differ, but one generic `close()` must not decide both policies from
an already-damaged collection of pointers. Destructor cleanup is best effort
only and is not acceptance evidence for either operation.

Once `abort_before_effect` encounters a cleanup fault, no response is published
for that request even if the one synchronous retry later drains the debt. The
original primary remains observable through its existing exception/diagnostic
channel plus bounded cleanup context, and the connection closes. This preserves
the accepted pre-effect silence rule without confusing a recovered accounting
release with permission to resume response selection.

The handler owns the same slot through the request-loop terminal observer. A
local `prepared = mutation_slot.take()` transfer is forbidden for the repaired
Upload path because it splits selection, wire, cleanup, and observation among
uncoordinated owners. The wire layer borrows the slot-owned selected response;
it does not take, clone, re-encode, or replace it.

### D5 — success/durable precedence after `M`

The route records publication confirmation separately from `M`:

- if the reducer returned and its result matched the frozen result, publication
  is confirmed and the selected response is the already-prepared success;
- if `M` was reached but publication is not confirmed, the selected response is
  the already-prepared fixed durable `503` with
  `HTTP_MUTATION_RETRY_PAYLOAD`;
- an exception after a reducer wrapper committed internally but before the
  caller observed its matching return is still “unconfirmed” to this caller and
  therefore selects the durable `503`;
- before `M`, the primary route error retains its existing typed status and no
  success/durable mutation response may publish if cleanup remains uncertain.

Selection is immutable. A retry may retry cleanup for the same selection, but
may not call the opposite selector or select a freshly encoded owner.

### D6 — wire publication borrows; accounting cleanup is outside transport

The transport-origin boundary covers only the actual socket-facing operations:

1. transition to `WIRE_STARTED` before the first `wfile.write` attempt;
2. write the exact frozen header;
3. write the exact frozen body;
4. transition to `WIRE_COMPLETE` only if both calls return.

Selected-owner accounting release, rejected-owner cleanup, response-slot state
release, diagnostic recording, and terminal observation run outside
`transport_boundary`. A cleanup `OSError` is therefore never marked transport
merely because it followed a write. A socket header/body/flush failure remains
transport-origin and closes the connection.

Once `WIRE_STARTED` is reached, any failure—header, body, flush, cleanup,
observer, or route-finally—must abandon or finish that one framing attempt. It
must not invoke `send_json`, `prepare_json`, or another
`publish_prepared`/write sequence. A header write may have partially written
before raising, so even an apparently empty captured buffer is not authority to
send a fallback document.

### D7 — bounded `RetryableCleanup` choice for this slice

The historical shared-primitives report called the global mechanism
`shape-pending` and named three possible shapes. This packet chooses exactly one
shape only for the A2 response-slot lifecycle:

- ownership handles are non-consuming until their exact release succeeds;
- the handler retains the same slot and exact owner identities;
- the initial cleanup attempt occurs at the owning transition;
- on failure, the request loop performs exactly one synchronous retry on the
  same owner, for two total attempts at that failed release boundary;
- no replacement owner, reconstructed scalar, aggregate release, unbounded
  loop, or second request is used as the retry mechanism.

If that retry succeeds, the bounded recovered-cleanup diagnostic is consumed
exactly once: attach it as secondary information to an existing primary, or log
`recovered cleanup fault: <bounded json>` through the handler when the selected
response is the primary outcome. A clean request emits no cleanup diagnostic.

If that retry fails persistently:

1. the connection closes;
2. the decision-work terminal is `cleanup_incomplete` when current debt or an
   open permit remains;
3. the observer reports the reachable slot/owner identity, exact remaining
   debt/permit count, selection, wire state, two attempts, and bounded faults;
4. no pointer or scalar is cleared to make the snapshot look clean;
5. no second response is attempted; and
6. `__del__` may make a later best-effort attempt but cannot be cited as the
   retry, cleanup, or zero-debt guarantee.

This local choice does not globally close `RetryableCleanup` shape-pending work
in `C14-LIVE-A1` or `C14-SUCCESSOR-OWNER`.

### D8 — terminal vocabulary and observer truth

Decision-work terminal reason describes decision-work accounting, not socket
delivery. This packet adds no `publication_failed`, `wire_failed`,
`response_abandoned`, or other seventh advertised terminal variant.

- zero current debt and zero open permits after clean or recovered cleanup use
  `complete`, including a successful `200`, fixed durable `503`, or transport
  failure after the decision work itself was fully released;
- an earlier valid terminal such as `budget_exceeded`, `subject_changed`, or
  `deadline_exceeded` is never overwritten;
- residual current work or an open permit uses the existing Python handler
  marker `cleanup_incomplete` and closes the connection;
- response selection and wire progress are reported separately as slot/observer
  state and never encoded by inventing a decision-work terminal.

The historical preflight's synthesized statement that every post-effect
cleanup/publication fault is “never `complete`” is superseded for this assigned
slice by ADR-020 plus the later manager correction. Its other requirements—one
terminal observation, secondary-fault precedence, exact owner retention, and no
second document—remain binding.

### D9 — primary and secondary precedence

| Phase | Primary outcome | Secondary cleanup behavior | Wire rule |
|---|---|---|---|
| Before `M` | Existing route/business/budget/deadline/filesystem error | Preserve and annotate/log cleanup; persistent uncertainty closes connection | No mutation response and no fallback if disposal is incomplete |
| At/after `M`, publication unconfirmed | Fixed durable `503` | Never substitutes; one same-owner retry | Attempt the exact durable response once |
| At/after `M`, publication confirmed | Exact prepared success | Never substitutes; one same-owner retry | Attempt the exact success response once |
| After `WIRE_STARTED` | Transport outcome of the existing framing attempt | Recorded outside transport; persistent debt becomes `cleanup_incomplete` | No second document under any condition |

`MemoryError` and process-fatal `BaseException` behavior remains outside the
filesystem diagnostic arm. A pre-effect filesystem `OSError` remains the typed
`400` authority family. The packet does not convert cleanup errors into
filesystem, transport, business, or budget errors.

## Execution routing

The Ultra architecture decision is completed by this manager packet. Long
execution must follow the repository's Low-Codex provider policy:

| Lane | Provider/model | Runaway limit | Authority |
|---|---|---:|---|
| Implementation | Claude `claude-opus-5` | 6h | Exact nine-path candidate only |
| Deterministic validation/reversal assistance | Fugu `fugu-ultra-v1.1` | 2h per bounded lane | Report-only or disposable reversal work; never candidate integration |
| Independent exact review | fresh Claude `claude-opus-5` session | 4h | Report-only exact `L..C` review |
| Adjudication/integration | manager Codex `gpt-5.6-sol`, explicit authorization only | 3h | Control plane only |

Implementation and review cannot use the same Claude session. Provider outage,
quota, timeout, or authentication failure preserves the worktree, session, log,
and checkpoint commits; it never silently falls back to Codex. A timeout is a
runaway bound, not an expected task duration or permission to discard work.

## Scope

### Exact nine-path candidate write set

Candidate `C` may change exactly these nine paths, with no optional tenth path:

1. `codefire`
2. `tests/test_codefire_cli.py`
3. `tests/test_python_http_post_effect_response.py` *(create)*
4. `adr/ADR-021-python-http-mutation-response-lifecycle.md` *(create)*
5. `docs/development/source-architecture.md`
6. `docs/development/command-source-trace.md`
7. `docs/development/source-data-model-catalog.md`
8. `docs/development/change-playbook.md`
9. `docs/development/code-review-issues/cfr-168.md`

Candidate `C` is one clean non-merge commit whose parent is `L`. A result file,
review report, flow record, history entry, current-state update, packet edit, or
generated artifact is not part of `C`. Candidate evidence is returned in the
handoff and may be persisted later by the manager in a separate, explicitly
scoped commit without changing `C`.

### Product surface allowed inside `codefire`

Only the following concerns may change:

- `_PreparedHTTPResponseOwner` wire/cleanup separation needed for borrowed
  publication;
- `_PreparedHTTPMutationResponses` transactional destination-first selection
  and rejected-owner retry retention;
- `_HTTPMutationResponseSlot` state, ownership, pre-effect abort,
  post-effect finish, wire state, bounded cleanup diagnostic, and observer
  projection;
- `handle_http_upload` at the response-seal, `M`, publication-confirmation,
  selection, and cleanup handoff seams;
- `make_http_handler.CodeFireHTTPHandler.publish_prepared`, `do_POST`, and
  `handle_one_request` only where required to keep one slot through wire,
  cleanup, and terminal observation.

No other route or primitive may be behaviorally changed merely because it
shares a helper.

### Documentation obligations

The new ADR must durably define `M`, both state machines, destination-first
ownership, borrowed wire publication, two cleanup operations, the bounded
non-consuming retry, persistent failure, terminal vocabulary, and
one-document precedence. It must state that the actual first durable byte is
not observable and that `M` is conservative.

The four canonical source docs must describe the implementation actually
landed, without claiming Install, Dogfood, MR-write, global C14, or all-route
closure. `cfr-168.md` must record candidate/source evidence and the exact A2
scope while keeping `Status: investigating` and every unrelated residual open.

### Read set

The implementer must read this packet and typed companion, `AGENTS.md`,
`docs/development/project-management.md`,
`docs/development/ai/operating-protocol.md`, ADR-020, the four bound evidence
documents above, the nine candidate paths, and the existing focused mutation
response and real-socket tests. Reading adjacent helpers needed to understand a
call is permitted; writing them is not.

The rebind adds these mandatory reads, because the A1 base changes the
pre-effect exception behavior the A2 route observes:

- `docs/development/ai/swarm-results/cfr168-c14-live-a1-c4-bound-adjacent-oracle-amendment-20260803/review.md`
  and `manager-adjudication.md`;
- `docs/development/ai/task-packets/cfr168-c14-live-a1-plan-freeze-20260802.md`;
- the A1 exception helpers `_detach_effective_exception_tracebacks`,
  `_link_primary_cleanup_exception`, and `_retain_first_cleanup_failure`, plus
  the four changed callees reachable from `handle_http_upload`; and
- `tests/test_python_http_upload_accounted_traversal.py` at `L`, which A1
  changed and which the typed companion already lists.

## Counterexample, oracle, and reversal matrix

Every row is required. A test is non-vacuous only when its named reversal fails
for the intended behavioral reason; syntax errors, missing symbols, import
failures, or a different earlier assertion do not count. Reversal patches are
temporary evidence, must be restored before candidate freeze, and must never be
committed.

| ID | Counterexample / injection | Required oracle | Required reversal that must fail |
|---|---|---|---|
| `T01` | A pre-`M` cleanup release fails while a typed route error is already primary. | Zero response bytes, no authority/object/nonce/transaction diff, primary identity preserved, connection closed if cleanup stays uncertain. | Move `M` before the last fence or allow the generic error arm to encode a fallback despite incomplete disposal. |
| `T02` | Durable selection's rejected-success owner has a one-shot release fault. | Exact fixed `503` header/body, same selected owner, exactly two cleanup attempts, zero debt/permits, terminal `complete`. | Run rejected cleanup before destination-first handoff. |
| `T03` | Confirmed-success selection's rejected-durable owner fails persistently. | Exact `200` success bytes are attempted once; selected owner is never destroyed; rejected owner remains reachable; connection closes; terminal `cleanup_incomplete`. | Let generic pair/slot close see and destroy the selected owner. |
| `T04` | A fault is injected after selection in route-finally/teardown. | The already-selected success remains primary and is written once; the later fault is secondary. | Permit route-finally to clear the slot or call `prepare_json`/`send_json` again. |
| `T05` | After a cleanup failure, call the opposite selector and then retry cleanup. | Opposite selection is typed-invalid without state change; retry uses the identical selected/rejected owner identities and direction. | Reset selection on failure, reconstruct an owner, or permit opposite selection. |
| `T06` | The first header `write` raises after it may have written a prefix. | `WIRE_STARTED`, connection close, no second status line/document, accounting cleanup outside transport, truthful terminal/debt. | Send a fallback because the capture buffer appears empty or leave `WIRE_STARTED` unset until `write` returns. |
| `T07` | Header succeeds and body `write` fails after a body prefix. | Exactly one status line/header framing attempt; no appended JSON error document; selected cleanup still executes. | Catch body failure and invoke `send_json` or a second prepared owner. |
| `T08` | Both writes return, then selected-owner accounting cleanup raises `OSError`. | Wire state is complete; the error has no transport origin; response bytes are unchanged; same-owner retry and terminal rules apply. | Widen `transport_boundary` back around owner cleanup. |
| `T09` | Initial rejected/selected cleanup fails once and the request-loop retry runs. | The handler retains the exact same slot; total attempts are two; observer is called once after retry; no detached local owner exists. | Use `take()`, a new slot, a reconstructed scalar, or an unbounded retry loop. |
| `T10` | Initial cleanup and its one retry both fail. | Observer exposes selection, wire state, reachable owner, exact current debt/open permits, two attempts and bounded faults; terminal `cleanup_incomplete`; connection closes. | Clear the pointer/scalar, report `complete`, or rely on `__del__` to make the snapshot pass. |
| `T11` | Clean confirmed upload. | Exact success response; zero current; zero permits; one observer; terminal `complete`; no cleanup diagnostic. | Add a publication/wire terminal or leave slot state into the next request. |
| `T12` | A one-shot post-`M` cleanup fault recovers. | Exact selected response; zero current/permits; terminal `complete`; one bounded recovered-cleanup diagnostic consumed by primary note or handler log. | Store the diagnostic unread, emit it twice, replace the response, or classify the cleanup error as transport. |
| `T13` | A pre-`M` filesystem `ETIMEDOUT`/`EIO` occurs on the POST route. | Existing typed filesystem-authority `400`, no mutation effect, and no transport mark; `MemoryError` remains outside this arm. | Broaden transport marking by exception class or route every `OSError` through the post-effect path. |
| `T14` | Exercise slotless POST GC and merge-request create/review/apply success and error paths. | Their baseline status/body, error origin, and decision-work observations are unchanged byte-for-byte or semantically as already specified. | Apply the mutation slot globally, alter generic `send_json`, or require an A2 slot on slotless routes. |
| `T15` | Two requests share one reusable connection/server: first has recovered or persistent A2 cleanup; second is a clean read/POST as applicable. | No selection, wire state, cleanup fault, owner, debt, or permit leaks; persistent failure closes before a second request; recovered failure resets all A2 fields. | Retain handler slot state after observer, reset it before reporting, or allow a persistent-failure connection to process another request. |

`T02`, `T03`, `T06`, `T07`, `T08`, `T09`, `T10`, and `T15` must use the
real handler/socket harness, not only direct class calls. Direct unit tests are
also required for the pair/slot transition graph and foreign-ledger rejection.
`T14` must not manufacture new expected behavior from the candidate; its
baseline is `L`.

### Mandatory baseline recomputation on `L`

Every `T01`-`T15` baseline is a **current-source** obligation on `L`. The
historical matrix rows are behavioral requirements, not recorded observations.

1. Recompute all fifteen baselines against `L`. A historical line number,
   expected byte string, exception repr, traceback shape, or cause-chain
   ordering carried over from `P0` is not evidence and may not substitute for
   an observation made on `L`.
2. Establish A1 exception behavior on `L` before writing oracles for the
   pre-`M` rows. `T01` must bind the exact primary object, type, and `str()`
   that `L` actually raises, and must state separately what
   `__traceback__`/`__cause__`/`__context__`/`__suppress_context__` normalization
   A1 applies. `T13`'s typed filesystem `400` authority must be observed on
   `L`, not assumed from `P0`.
3. Fix `M` against live source per `D1`, `H01`, and `H02`, explicitly
   adjudicating whether `is_ancestor_from_objects_accounted` — whose A1
   behavior changed and which sits between the layout-bootstrap and
   publication-effect fences — falls before or at/after `M`, with the exact
   `L` line and fence evidence for the choice.
4. Re-derive the `T14` slotless and `T15` next-request baselines on `L`; A1
   changed nine test modules and the four callees above, so a `P0` baseline is
   stale by construction.
5. If a recomputed baseline contradicts a frozen `D1`-`D9` decision or a
   `T01`-`T15` oracle, stop and return an exact blocker to the manager. Do not
   edit the design to absorb the difference, and do not weaken an oracle to
   make a recomputed baseline pass.

## Hazards and mandatory mitigations

| Hazard | Why it is dangerous | Required mitigation/evidence |
|---|---|---|
| `H01` Marker too early | A pre-effect fault could be misreported as durable/retry. | Exact ordering oracle around final fence, response seal, `M`, and first effect; `T01`. |
| `H02` Marker too late | An effect may occur while cleanup still follows the silence rule. | No callback/fallible call between `M` and first effect; source assertion plus `T02`/`T03`. |
| `H03` Split ownership | Pair, slot, local `prepared`, and publisher can double-close or orphan one owner. | Destination-first handoff and no Upload `take()`; identity assertions in `T05`/`T09`. |
| `H04` Generic close after `M` | Selected bytes can be destroyed by rejected cleanup. | Separate abort/finish APIs and state guards; `T03`/`T04`. |
| `H05` Transport boundary includes cleanup | Accounting `OSError` is misclassified as socket failure. | Wire-only boundary; `T08`. |
| `H06` Second-document recovery | Partial header/body becomes ambiguous or corrupt framing. | `WIRE_STARTED` before first attempt and permanent no-second-document guard; `T06`/`T07`. |
| `H07` Terminal conflates delivery and work | New terminal vocabulary contradicts ADR-020 and other consumers. | `complete` for zero, `cleanup_incomplete` for residual, separate wire state; `T10`-`T12`. |
| `H08` Destructor treated as guarantee | GC timing and swallowed `BaseException` hide reachable debt. | Exactly one synchronous same-owner retry; persistent observer evidence; `T10`. |
| `H09` Diagnostic becomes new primary | Cleanup text can replace the exact 200/503 or typed pre-effect error. | Secondary channel only, bounded text, one consumption; `T01`/`T12`. |
| `H10` Current one-shot test masks defect | Upload's catch/reselect path passes while ownership remains wrong. | Persistent selection fault and route-finally cases; `T03`/`T04`. |
| `H11` Re-encoding after `M` | Encoding/budget failure can hide a durable effect. | Use only the pre-encoded pair; payload/header identity assertions in `T02`/`T03`. |
| `H12` Broad helper edit regresses GET/slotless POST | Shared `publish_prepared` and dispatcher serve other routes. | Narrow call contract and `T13`/`T14`; full Python gate. |
| `H13` Observer clears evidence too early | Persistent owner appears clean but is unreachable. | Observe before final best-effort destruction and report exact reachability; `T09`/`T10`. |
| `H14` Raw `BaseException` swallowing | Process-fatal or accounting defects disappear. | Preserve existing fatal boundaries; only secondary cleanup is bounded/recorded; focused fault identity tests. |
| `H15` Candidate/result topology drifts | Reviewer no longer sees one immutable subject. | One nine-path candidate commit, external handoff, exact candidate capture, fresh review. |

## Non-Goals

- `C14-LIVE-A1`, including constructor unwind, commit-source wrappers,
  server-verification cleanup, signature owners, or pre-effect ancestry repair.
- Changing `AccountedHTTPUploadAncestry` ownership or the stale historical
  tracker wording about whether it is live; that is an A1 bookkeeping concern.
- `C14-SUCCESSOR-OWNER`, production-unwired sealed ancestry, planner/stage
  factories, `_control_charge_witness`, O2, or O3.
- GET/read-slot behavior, its proof state, or its accepted dispose-before-select
  semantics.
- A wholesale replacement of the shared error-origin classifier or any message,
  errno, or exception-class classifier.
- Wiring the response lifecycle into MR create/review/apply, GC, or other
  currently slotless POST routes.
- MR-write implementation, review, apply, rejected-base rebinding, or clearing
  its separate `B1` blocker.
- Publication reducer, nonce, gate, recovery, garbage collection, ancestry,
  signature, validation, bootstrap, object-copy, or authority semantics.
- Rust, Cargo, crates, schemas, conformance, capability advertisement,
  packaging, install scripts, Docker, network, or dogfood mutation.
- Adding a new advertised decision-work terminal reason or editing ADR-020 to
  rationalize one.
- Unbounded retry, asynchronous background cleanup, a global cleanup registry,
  or cross-request adoption of a failed owner.
- Proving peer receipt, durable byte timing, TCP flush, or exact filesystem
  visibility from `M`, `WIRE_STARTED`, or `WIRE_COMPLETE`.
- Closing CFR-168, CFB-136, CFB-138, global C14, MR-write, the current batch, or
  any Install/Dogfood/Retrospect stage.
- Editing current-cycle, management hub, history, packet, flow-metric, or review
  documents inside candidate `C`.
- Opportunistic refactor, rename, formatting sweep, performance tuning, or
  style-only change not needed by a frozen state transition.

## Validation

Run focused checks first and stop on the first unexplained failure. Record the
full command, exit code, selected/executed test count, duration, and log digest
in the handoff. A skipped command is a gap, not a pass.

### Base rule proof against `P1`

Run before any edit, and stop on the first failure:

```bash
P1=1e8dd545f27bd1f22879b067ad0e1b3f4e2ecbc4
test "$(git rev-parse "$P1^{tree}")" = fa59fe69ba69bd199ef2cf3757679c98668a0812
git merge-base --is-ancestor "$P1" "$L"
test -z "$(git rev-list --merges "$P1".."$L")"
git diff --name-only "$P1".."$L"
test -z "$(git status --porcelain)"
```

The name-only output must contain only the two rebound packet paths, the
accepted rebind preflight report, and the management-only records the launch
prompt names by exact path. Any product, test, source-doc, ADR, schema,
packaging, or issue-detail path in that output is a base violation.

### Candidate identity and scope

```bash
test "$(git rev-parse HEAD^)" = "$L"
test "$(git rev-list --count "$L"..HEAD)" = 1
test -z "$(git rev-list --merges "$L"..HEAD)"
git diff --name-only "$L"..HEAD
git diff --check "$L"..HEAD
python3 docs/development/tools/capture_candidate_identity.py \
  --repo . --base "$L" --head HEAD --format json
```

The name-only output must equal the nine paths above as a set and contain no
tenth path. `capture_candidate_identity.py --manifest` consumes a swarm-batch
manifest, not this typed task packet, so it is intentionally run without that
option. Its `changed_paths` output and the independent name-only comparison
together prove the exact write set; do not mislabel the task packet as a swarm
manifest or edit its role.

### Focused product and socket checks

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile \
  codefire tests/test_codefire_cli.py tests/test_python_http_post_effect_response.py
python3 -m unittest tests.test_python_http_post_effect_response -v
python3 -m unittest \
  tests.test_codefire_cli.CodeFireCliTests.test_python_upload_response_preflight_precedes_effect_and_retries_durable_result \
  tests.test_codefire_cli.CodeFireCliTests.test_python_mutation_response_owner_rejects_foreign_and_retries_cleanup -v
```

The new module must identify all `T01`-`T15` rows in test names, subtests, or a
checked matrix so the result cannot claim 15 while silently omitting one.

### Reversal evidence

Apply each `T01`-`T15` reversal separately to a disposable copy or unstaged
worktree, run its named focused oracle, record the intended behavioral failure,
and restore the exact candidate bytes before the next reversal. At the end:

```bash
git diff --check
git status --short
```

The final candidate worktree must be clean after its commit. No reversal patch
or generated cache may appear in `C`.

### Canonical docs and broad Python gates

```bash
python3 docs/development/tools/check_source_docs.py
python3 docs/development/tools/check_project_docs.py
python3 -m unittest discover -s tests -v
./demo.sh
git diff --check "$L"..HEAD
```

No Cargo command is applicable. Because the candidate changes Python runtime
behavior, docs-only exemption does not apply.

## Candidate Freeze and independent review

### Candidate freeze

After every required validation passes, commit the nine paths once as candidate
`C`. Do not amend, squash, rebase, merge, or add evidence to `C`. The handoff
must bind:

- `L` commit/tree and its proof against `P1`, plus the recomputed
  A2-owned segment table on `L`;
- `C` commit/tree/parent;
- canonical patch SHA-256 for `L..C`;
- exact nine-path name-status and each final blob/mode;
- all validation commands, exit codes, counts, durations, and log digests;
- `T01`-`T15` result and reversal evidence;
- source-segment hashes for every changed B3 surface;
- no-effect authority snapshots for pre-`M` cases;
- exact wire bytes/status-line count for post-`M` cases;
- observer snapshots for clean, recovered, transport, and persistent cleanup;
- a statement of every non-goal and path intentionally untouched; and
- any unavailable evidence or environment limitation.

Any byte change after freeze creates a new candidate with a new identity. It
does not amend `C`.

### Reviewer exclusion and review subject

The reviewer must be a fresh report-only lane and must not be:

- this Plan/FREEZE packet authoring lane;
- the implementation lane or any agent that wrote, repaired, cherry-picked,
  rebased, formatted, or selected candidate bytes;
- the manager who adjudicates observations or integrates the candidate; or
- a lane with an uncommitted shared write set against the candidate.

The reviewer receives `P1`, `L`, exact `C`, this packet and companion, the four
bound evidence documents, the candidate handoff, validation logs, and reversal
evidence. It reviews the immutable `L..C` patch and writes only a manager-named
review report path outside `C`; it does not fix code or tests.

Review must independently cover:

1. exact `M` placement and phase claims;
2. pair and slot state-machine completeness;
3. destination-first handoff and single-owner accounting;
4. pre-effect abort versus post-effect finish;
5. exact-once same-owner retry and persistent reachability;
6. success/durable precedence and response byte identity;
7. `WIRE_STARTED` no-second-document behavior;
8. transport-versus-cleanup error origin;
9. ADR-020 terminal compatibility and observer truth;
10. all 15 counterexamples/reversals and non-vacuity;
11. slotless route and next-request regression surface;
12. exact nine-path scope and canonical documentation truth.

Every observation must include file/line evidence, impact, certainty, must-fix
trigger, and actionability disposition under
`docs/development/ai/operating-protocol.md`. Severity alone does not authorize
repair. Style preference, alternative decomposition, or speculative future
generality is not a required correction without a frozen invariant or
reproduced must-fix trigger.

If review finds a `required_correction`, the manager freezes the smallest
causal repair and rereview delta. No observation may silently widen into A1,
MR-write, Rust, publication reducer, or global C14. A fresh exact acceptance is
required before integration.

## Lifecycle and exit conditions

This packet completes Plan/FREEZE for only `C14-LIVE-A2` / `B3`. The subsequent
order is mandatory:

0. manager integrates this 2026-08-03 A1-base rebind and commissions a fresh
   independent rebind preflight against `P1`;
1. manager accepts that preflight, integrates it, and names `L`;
2. independent implementer creates and validates one nine-path `C`;
3. manager Candidate-Freezes exact `C`;
4. fresh excluded reviewer reports on exact `C`;
5. manager adjudicates every observation;
6. bounded repair/rereview occurs only for required corrections;
7. manager integrates an exact accepted candidate and reruns focused checks;
8. source acceptance is recorded without final issue closure;
9. applicable fresh Install and real-socket Dogfood/Evaluate run later; and
10. only then may the manager consider final Close/Retrospect with all other
    CFR-168 predecessors.

An exact review ACCEPT is not integration. Integration is not Install or
Dogfood. This slice cannot by itself close CFR-168, because A1, successor work,
MR-write, scale/capability work, installed evaluation, and other frozen
predecessors remain outside it.

### What the 2026-08-03 rebind is not

The rebind authoring result and the fresh exact preflight that follows it are
documentation-only steps at stage 0 and 1 above. Neither is, nor implies, any
of the following:

- A2/B3 implementation, or any product, test, ADR, source-doc, or issue-state
  change;
- review acceptance of a candidate, or a Candidate Freeze;
- integration of A2/B3 into the mainline;
- Install, Dogfood/Evaluate, Retrospect, or final Close; or
- reopening, re-adjudicating, or widening the accepted A1 source package.

A2/B3 itself remains open. The A1 manager decision `1e8dd545…` accepted only
the A1 source package and selected it as the A2/B3 base; it accepted nothing
in this packet.

## Handoff

### Packet-author handoff

The packet-author lane returns only:

- packet commit and tree;
- the two created paths and blob IDs;
- proof that the branch started at exact `P0` (historical) or, for the
  2026-08-03 rebind, at exact launch commit
  `615da7ae917d2a17d3bc97b00b8b0604092ab6ca` over `P1`;
- project/AI docs-check commands and results;
- `git diff --check` and clean committed status; and
- an explicit statement that no implementation path was touched.

### Rebind-author handoff

The 2026-08-03 rebind lane additionally returns the recomputed `P0`/`P1`
identity, topology, blob and segment tables; the classification of every
changed `P0..P1` product and test unit; the semantic-delta result for all seven
A2-owned surfaces; and Markdown/JSON consistency evidence. It returns no
implementation artifact.

### Implementer handoff

The later implementer returns the Candidate Freeze evidence enumerated above,
plus this concise state block:

```text
State:
- Product base P:
- Launch base L:
- Candidate C/tree/parent:
- Patch SHA-256:
- Exact nine paths:
- T01-T15 and reversals:
- Focused/broad/docs/demo validation:
- Clean worktree:

Changed:
- Product lifecycle:
- Tests:
- ADR/source docs/issue evidence:

Protected:
- Pre-M no-effect:
- Selected owner and wire bytes:
- Cleanup/terminal/observer:
- Slotless and next-request behavior:

Risks or unavailable evidence:
-

Next:
- Candidate Freeze and fresh excluded review; no integration performed.
```

## Packet self-check

The packet is ready only because every previously unassigned decision has one
answer and one oracle:

| Required decision | Frozen location | Oracle |
|---|---|---|
| Exact conservative `M` | D1 | `T01`-`T03` |
| Pair states | D2 | `T02`, `T03`, `T05` |
| Slot states | D3 | `T04`, `T06`-`T10`, `T15` |
| Selected handoff before rejected cleanup | D2 | `T02`, `T03`, `T05` |
| Slot ownership / wire borrow | D4/D6 | `T06`-`T09` |
| Pre/post cleanup separation | D4 | `T01`, `T03`, `T04` |
| Success versus durable selection | D5 | `T02`-`T04` |
| Wire-only transport boundary | D6 | `T06`-`T08` |
| Shape-pending local choice | D7 | `T09`, `T10`, `T12` |
| Exactly one synchronous retry | D7 | `T02`, `T09`, `T10` |
| Persistent owner reachability | D7 | `T03`, `T10` |
| ADR terminal vocabulary | D8 | `T10`-`T12` |
| One-document precedence | D6/D9 | `T04`, `T06`, `T07` |
| Nine-path bound | Scope | candidate identity gate |
| Fifteen counterexamples/reversals | Matrix | focused/reversal evidence |
| Fresh reviewer exclusion | Candidate Freeze | manager review launch record |
| Single product authority | `P1` = `1e8dd545…` | `P1` identity/topology/blob gate |
| Satisfiable execution base | Execution-base rule `L` | base rule proof against `P1` |
| A2-owned surfaces unchanged by A1 | `P0`/`P1` segment table | recomputed text/AST hash equality |
| Historical coordinates are not current source | Reproduced root cause | symbol resolution on `L` |
| Changed-callee exception behavior | Mandatory baseline recomputation on `L` | `T01`, `T13`, plus the `M` placement adjudication |

There is no unresolved requirement-authority choice inside this bounded slice,
and authority is single-valued: `P1` is the only product base, and the rebound
`L` rule is the only execution-base rule.

If live source at `L` violates the base rule, if any required behavior needs a
tenth candidate path, if a recomputed `T01`-`T15` baseline contradicts a frozen
`D1`-`D9` decision, or if a test exposes a contract that changes authority
beyond A2, the implementer must stop and return that exact blocker to the
manager. It must not improvise a wider repair, resolve a `codefire` conflict by
whole-file selection, or reintroduce `P0` as an authority.
