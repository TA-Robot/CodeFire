# ADR-021: Python HTTP Mutation-Response Lifecycle Across the Effect Boundary

Status: Accepted

Date: 2026-08-03

Owners: Remote, CLI, and Store Maintainers

Related: CFR-168 (C14-LIVE-A2 / B3), ADR-017, ADR-020

## Context

The Python fallback HTTP Upload route prepares two frozen responses — the exact
success document and the fixed durable-retry `503` carrying
`HTTP_MUTATION_RETRY_PAYLOAD` — before it performs any durable effect. Exactly
one of them must reach the client, and it must survive every cleanup path that
runs afterwards.

The accepted pre-repair implementation handed ownership off in the wrong order
across three nested lifetimes. The prepared pair recorded its selection and then
immediately ran the *rejected* owner's fallible `close()` while the selected
owner had not yet reached the mutation slot. A rejected-cleanup fault therefore
dropped the route into a generic error arm whose generic slot `close()` could
destroy the response the durable operation had already selected. A second
generic close existed on the route-finally path. Separately, the publication
seam placed both socket writes *and* the owner's accounting release inside
`transport_boundary`, so a post-write accounting `OSError` was marked as a
transport failure even though the socket write had already returned — hiding
cleanup debt and inviting a second response document.

Repairing this needs one decision that the code cannot derive on its own: when
is it still provable that no effect has begun? The actual moment a durable byte
becomes visible is not caller-observable, and it must not be guessed from an
exception class, a traceback, a return timing, or a filesystem probe.

## Decision

### The conservative marker `M`

Introduce a single one-way state, `EFFECT_MAY_HAVE_STARTED` (`M`), set exactly:

```text
last complete pre-effect authority/subject fence
-> consume/seal every prepared-response permit
-> set EFFECT_MAY_HAVE_STARTED
-> immediately invoke the first fallible effect
```

Only one local enum assignment may sit between the seal and the first fallible
effect. No callback, allocation proportional to input, cleanup, filesystem call,
deadline poll, encoding, or authority reread may intervene.

`M` is deliberately conservative and establishes only two propositions:

- **before `M`** the route proves that no authoritative effect began;
- **at or after `M`** the route can no longer prove that no effect began.

`M` is *not* evidence that an effect happened, that a publication was confirmed,
that a client received bytes, or that a transaction committed. Neither `M` nor
either wire state may be cited as proof of a durable byte, peer receipt, TCP
flush, or filesystem visibility.

### Pair state machine

The prepared success/durable pair follows:

```text
OPEN -> SEALED
     -> SELECTING_SUCCESS | SELECTING_DURABLE
     -> SELECTED_SUCCESS  | SELECTED_DURABLE
     -> CLEANUP_PENDING
     -> CLOSED
```

Selection is a **destination-first** handoff and runs nothing fallible.
`select_success` / `select_durable` validate the ledger, reject an opposite
direction permanently, record the immutable direction, clear the selected side,
retain the rejected owner, and return the selected owner. Only then may the
caller run `release_rejected`, which is the one fallible step. After the
handoff the selected owner is unreachable from the pair, so no generic pair
cleanup can destroy an already-selected response. The pair never reconstructs,
re-encodes, clones, or reacquires either response.

### Slot state machine

The mutation slot follows:

```text
EMPTY -> PREPARED -> SEALED -> EFFECT_MAY_HAVE_STARTED
      -> SELECTED -> WIRE_STARTED -> WIRE_COMPLETE
      -> CLEANUP_PENDING -> CLOSED
```

with two explicit failure branches:

```text
PREPARED | SEALED
  -> abort_before_effect -> CLEANUP_PENDING -> CLOSED     (zero wire bytes)

EFFECT_MAY_HAVE_STARTED | SELECTED | WIRE_STARTED | WIRE_COMPLETE
  -> finish_after_effect -> CLEANUP_PENDING -> CLOSED     (never a second document)
```

`CLOSED` is reachable only once the exact debt was actually released; it is
never forged to let a failed request return.

### Two cleanup operations, never one ambiguous `close`

- `abort_before_effect` is legal only before `M`. It disposes both response
  choices, publishes nothing, and preserves the route's original primary
  exception. Once it hits a disposal fault, **no response is published for that
  request at all**, even if the later retry drains the debt, and the connection
  closes. A recovered accounting release is never permission to resume response
  selection.
- `finish_after_effect` is legal at or after `M`. It never destroys the selected
  owner before its one wire attempt, treats cleanup faults as strictly
  secondary, and never calls `send_json` or selects a different response after
  `WIRE_STARTED`.

A generic `close()` survives only as best-effort destructor cleanup. It is not
acceptance evidence for either operation, and neither operation is implemented
in terms of it: the policy is chosen by an asserted state, not inferred from
whichever pointers happened to survive.

### Ownership is never split; the wire borrows

The handler owns one slot from route entry through the request-loop terminal
observation. The Upload path performs no `take()`: publication **borrows** the
slot-owned selected response and writes its exact frozen header and body. The
wire layer never takes, clones, re-encodes, or replaces that owner.

### Transport boundary covers socket writes only

`transport_boundary` wraps exactly the two socket writes. `WIRE_STARTED` is
recorded *before* the first write is attempted, because a header write may
already have put a prefix on the wire before raising. Selected-owner accounting
release, rejected-owner cleanup, slot-state release, diagnostic recording, and
terminal observation all run outside that boundary, so a cleanup `OSError` is
never marked transport merely because it followed a write. A socket
header/body/flush failure remains transport-origin and closes the connection.

Once `WIRE_STARTED` is reached, any subsequent failure — header, body, flush,
cleanup, observer, or route-finally — must abandon or finish that one framing
attempt. It must not invoke `send_json`, `prepare_json`, or another publication
sequence. An apparently empty capture buffer is not authority to send a
fallback document.

### Success/durable precedence after `M`

- reducer returned and its result matched the frozen result: publication is
  confirmed and the selected response is the already-prepared success;
- `M` reached but publication unconfirmed: the selected response is the
  already-prepared fixed durable `503`;
- an exception after a reducer wrapper committed internally but before the
  caller observed its matching return is still unconfirmed to this caller and
  therefore selects the durable `503`;
- before `M`, the primary route error retains its existing typed status, and no
  mutation response publishes while disposal remains uncertain.

Selection is immutable. A retry may retry cleanup for the same selection; it may
never call the opposite selector or select a freshly encoded owner.

### Bounded non-consuming cleanup retry

Ownership handles are non-consuming until their exact release succeeds. The
initial cleanup attempt occurs at the owning transition; on failure the request
loop performs exactly **one** synchronous retry on the identical owner, for two
total attempts **at that failed release boundary**. A boundary whose release
already succeeded never spends another boundary's retry. No replacement owner,
reconstructed scalar, aggregate release, unbounded loop, background task, or
later request is used as the retry mechanism.

If the retry succeeds, one bounded recovered-cleanup diagnostic is consumed
exactly once through the handler's error log. A clean request emits none. If the
retry fails persistently: the connection closes; the reachable owner, exact
remaining debt, permits, selection, wire state, attempts, and bounded faults are
reported truthfully; nothing is cleared to make the snapshot look clean; no
second response is attempted; and `__del__` may make a later best-effort attempt
but is never cited as the retry, cleanup, or zero-debt guarantee.

### Terminal vocabulary is unchanged

Decision-work terminal reason describes decision-work accounting, not socket
delivery. This decision adds **no** new advertised terminal variant — no
`publication_failed`, `wire_failed`, or `response_abandoned`.

- zero current debt and zero open permits after clean or recovered cleanup use
  `complete`, including a successful `200`, a fixed durable `503`, or a
  transport failure after the decision work itself was fully released;
- residual current work or an open permit uses the existing
  `cleanup_incomplete` marker and closes the connection;
- an earlier valid terminal (`budget_exceeded`, `subject_changed`,
  `deadline_exceeded`, …) is never overwritten;
- response selection and wire progress are reported separately as slot and
  observer state, never encoded by inventing a decision-work terminal.

## Consequences

- One selected, pre-encoded response survives every post-effect cleanup path,
  and a rejected-owner fault can no longer cost the client its answer.
- Exactly one HTTP document is framed per request under every failure ordering.
- Accounting faults are no longer misreported as socket failures, so residual
  ownership stays visible instead of hiding behind a transport mark.
- A persistent cleanup failure is truthfully terminal (`cleanup_incomplete`)
  with a reachable owner, rather than a forged clean close.
- No A2 selection, wire state, cleanup fault, owner, debt, or permit can leak
  into a later request on a reused connection.
- The bound is local to the Upload mutation-response slot. It does **not** close
  the global `RetryableCleanup` shape-pending work, the successor composition,
  publication-proof adoption, or wiring the lifecycle into GC, merge-request, or
  other currently slotless POST routes.

## Alternatives Considered

- **Infer the effect boundary from the failure.** Rejected: the first durable
  byte is not caller-observable, and classifying by exception class, errno,
  message, traceback, or timing is exactly the inference this contract forbids.
- **Keep one generic `close()` and add defensive checks.** Rejected: a single
  entry point must then choose the pre-effect and post-effect policies from an
  already-damaged set of pointers, which is the defect being repaired.
- **Let the publisher take the prepared owner.** Rejected: `take()` splits
  selection, wire, cleanup, and observation among uncoordinated owners, which is
  how the selected payload became destroyable in the first place.
- **Add a publication/wire terminal reason.** Rejected: it would contradict
  ADR-020's vocabulary and conflate delivery with decision-work accounting. Wire
  progress is separate observer state.
- **Retry cleanup until it succeeds, or defer it to `__del__`.** Rejected:
  unbounded retry is not bounded work, and destructor timing plus swallowed
  `BaseException` would hide reachable debt.
