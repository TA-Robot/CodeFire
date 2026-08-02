# CFR-168 C14-LIVE-A2 / B3 post-effect response lifecycle — manager Plan/FREEZE packet

Status: ready

Packet ID: `cfr168-c14-live-a2-b3-plan-freeze-20260802`

Role: manager Plan/FREEZE authority; a later, separate lane may implement only
after this packet is integrated and the manager names its exact launch base.

Typed companion:
`docs/development/ai/task-packets/json/cfr168-c14-live-a2-b3-plan-freeze-20260802.json`

Immutable product base: `333e41082ffc35c73c02a420511ae2f16839c5b8`
(tree `c0ab3a09e4731ca27d43023be72e577487b11600`).

Decision: **AUTHORIZE_BOUNDED_IMPLEMENTATION_AFTER_PACKET_INTEGRATION**.

This commit is documentation-only. It does not authorize this packet-writing
lane to mutate product, tests, source docs, issue state, installation, or
dogfood state. The implementation authorization becomes usable only when a
manager supplies the full launch-base commit `L` satisfying the base rule in
this packet. Until then, implementation is forbidden.

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

### Immutable source authority `P`

| Identity | Exact value |
|---|---|
| Commit `P` | `333e41082ffc35c73c02a420511ae2f16839c5b8` |
| Tree | `c0ab3a09e4731ca27d43023be72e577487b11600` |
| Parent | `808513a65dbfbbffff8123792d59d9eadfd875b4` |
| Subject | `docs(cfr168): synchronize accepted source state` |
| `codefire` blob/mode | `73b1f368f9c1591be6fd9219b839351878b82b9a`, `100755` |
| `tests/test_codefire_cli.py` blob | `b23c43ced6ccb69c0bb553f9856c1a7b2145e770` |
| `source-architecture.md` blob | `fe2b3b74f2beaf30218333f14e05b6e4e6d4e699` |
| `command-source-trace.md` blob | `361a5de8f9f881fbedf35ed5a833baade9529e38` |
| `source-data-model-catalog.md` blob | `2b68347eaa357f187f794c2a76992401743929eb` |
| `change-playbook.md` blob | `18d882813e02ed8cd795aa1e46d8cac6a09be4ef` |
| `cfr-168.md` blob | `29df10886dab2d5950df907d31238e8ea5ce1a8d` |

### Execution-base rule `L`

This file cannot contain the commit that contains itself. The manager launch
prompt must therefore name one full 40-character execution base `L`. `L` must:

1. contain this Markdown packet and its typed companion byte-for-byte;
2. descend linearly from `P`;
3. differ from `P` only by the manager-integrated Plan/FREEZE packet paths and
   any explicitly named management-only integration record;
4. contain no product, test, ADR, source-doc, packaging, schema, or issue-detail
   difference from `P`; and
5. be clean before the implementation worktree is created.

The implementer must stop if any predicate fails. Editing the typed
`base_revision` to make a generic launcher accept a different base is not an
alternative. `P` is immutable product authority; `L` is the manager-named
packet-containing execution base.

### Bound evidence

| Evidence | SHA-256 at `P` | Use |
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

Exact accepted AST source segments at `P`:

| Surface | Lines | Bytes | SHA-256 |
|---|---:|---:|---|
| `_PreparedHTTPResponseOwner` | 16037-16089 | 1748 | `82d4ca7bb9df993a4e825c89acf12e973c46921925bcacdb1673d14efae63c04` |
| `_PreparedHTTPMutationResponses` | 16092-16243 | 5769 | `16a4501d35455d482e1b1e6bd4b16bd661b0c18cdb4afb86c7afcef5353cf25c` |
| `_HTTPMutationResponseSlot` | 16256-16325 | 2635 | `3c3fae78db5e3a61c6a9c33ea03dacab5111ba094284eda3a47bd0c1ea43d9b9` |
| `handle_http_upload` | 14720-15226 | 21581 | `b9d032b67ba475fbe2baacb9927fb442614b66b366cc443ce24366d255073732` |
| `publish_prepared` | 17221-17245 | 1168 | `99af445f34cce9ca80b5aafe8988d63efec9c8118979f09a5cb793887e2d1940` |
| `handle_one_request` | 17264-17419 | 7834 | `af8a6a832d3845a236e1f983107dc72b5dec886c8babc90e026b351593eaf007` |
| `do_POST` | 17635-17867 | 11265 | `ff31cb1ffc9888836553de36cab1b340b43f04f7038b49b1d581e85bae301e1a` |

An implementer may change these segments only to realize this packet. Segment
movement from formatting is not itself a defect, but every semantic delta must
map to a frozen transition, oracle, or documentation obligation below.

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

- `L` commit/tree and its proof against `P`;
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

The reviewer receives `P`, `L`, exact `C`, this packet and companion, the four
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

1. manager integrates this packet and names `L`;
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

## Handoff

### Packet-author handoff

The packet-author lane returns only:

- packet commit and tree;
- the two created paths and blob IDs;
- proof that the branch started at exact `P`;
- project/AI docs-check commands and results;
- `git diff --check` and clean committed status; and
- an explicit statement that no implementation path was touched.

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

There is no unresolved requirement-authority choice inside this bounded slice.
If live source at `L` violates the base rule, if any required behavior needs a
tenth candidate path, or if a test exposes a contract that changes authority
beyond A2, the implementer must stop and return that exact blocker to the
manager. It must not improvise a wider repair.
