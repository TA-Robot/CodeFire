# CFR-168 Python HTTP accepted-input composition — manager Plan/FREEZE packet

Status: ready_for_bounded_composition

Packet ID: `cfr168-python-http-accepted-input-composition-20260802`
Role: manager (Plan/FREEZE authority); the executing lanes are one implementer and one reviewer
Provider/model: Claude Opus 5 for both the composition writer and the fresh review
Typed companion: `docs/development/ai/task-packets/json/cfr168-python-http-accepted-input-composition-20260802.json`
Bound preflight: `docs/development/ai/swarm-results/cfr168-python-composition-exact-preflight-claude-20260802/preflight.md` — unchanged, and authoritative for the **step-5 union stage only**; see Correction 2
Original shared-primitives authority, preserved unchanged: `docs/development/ai/swarm-results/cfr168-python-http-shared-primitives-preflight-claude-20260731/preflight.md`

> **Correction 1 — topology.** The first version of this packet was
> published in commit `806f6c8033905cd44e22bf12d42c62dba42c47a8` (tree
> `a4cd138b93b54d8dbcd8c5d87dee3682f79324b3`, parent
> `a972cde890e0d621aaced6aca4b45835871da975`). That version froze all 19 writer
> paths as **one** commit, which is topologically impossible: path 19 is the
> writer's own result report, and a single commit cannot record its own SHA,
> tree or patch digest. `806f6c8` is preserved as immutable history and is
> **not** amended, rebased or reset. The topology correction published in
> `a7e283e5eb5a9f54765c46f10057313a4e858f38` (tree
> `925dc049083c26ad3190c2c14e3c1bebe099f7c2`, parent `806f6c8`) supersedes its
> topology, its execution-base rule, its routed-residual ownership map and the
> stale closure-mode wording, in the three authorized correction paths only. The
> published preflight body is untouched.
>
> Two renames follow from that correction, recorded here so nothing looks lost:
> the reviewer report commit is now named `R`, and the reversal validation family
> that `806f6c8` called `R1`-`R9` is now `X1`-`X9`. No reversal was dropped.
>
> **Correction 2 — canonical source-doc blob stages.** A manager pre-commit
> audit of `a7e283e` found one hard contract contradiction. As of that audit the
> four canonical source docs passed through **two sequential stages**: the step-5
> three-way union, and the step-7 hand-authored `O1` citation, which by
> construction edits two of those same four files. (Correction 3 below adds a
> third stage; the current model is three stages, not two.) `a7e283e`'s single `I5` check nevertheless required the
> **final** committed files to match all four **pre-citation** union blobs, which
> is unsatisfiable for `docs/development/source-architecture.md` and
> `docs/development/command-source-trace.md` and made `I5` ambiguous at best. This
> successor correction separated those two stages, published both blob sets as
> full 40-character values, and split `I5` into `I5-UNION` and `I5-FINAL`.
>
> Correction 2 changed **nothing else at the time it landed**: the 18 + 1 + 1
> topology, the routed-residual owner map, the `B1`/`B2`/`B3` adjudication, the
> accepted inputs, the product behaviour, the scope, the read/write sets and every
> other invariant and gate were carried through unchanged. That statement is
> historical and describes Correction 2 only — Correction 3 below does change the
> product behaviour, the write scope and the doc-stage count, and supersedes the
> two-stage model with a three-stage one. `806f6c8` and `a7e283e` both remain
> immutable history and neither is amended, rebased or reset. The published
> composition preflight is also unchanged and remains valid **as historical
> intermediate-union evidence** — its four blob prefixes are the step-5 union
> stage, never the bytes committed in `C`. Because no commit can name
> itself, this correction carries no SHA of its own; it simply extends the
> manager-integrated Plan/FREEZE chain, and the launch base `L` is the final
> integrated docs-only `HEAD` that results.
>
> **Correction 3 — the `C18` composition bridge.** A real focused gate run on
> the staged composition, before candidate `C` was committed, exposed a second
> and deeper Plan omission. `python3 -m unittest
> tests.test_python_http_upload_accounted_traversal -v` ran 36 tests and failed
> exactly the two subtests of `test_request_loop_accounts_a_drained_cleanup_fault`
> — `route='ancestry'` and `route='signature-history'` — with
> `len(handler_errors)` `0`, expected `1`. The accepted Upload candidate
> `0f43415` passes that same test.
>
> The cause is a genuine **composition incompatibility**, not a merge defect. The
> error-origin taxonomy exists **only** in `b164046`; the synchronous cleanup
> drain exists **only** in `0f43415`; the base `cb13da0` has neither. Each
> accepted tree is internally correct and neither ever executed the other's
> symbol. The union places them in one dispatcher for the first time, and there
> the standalone cleanup `OSError` returned by
> `_drained_commit_source_cleanup_error` reaches `do_POST`'s `except
> BaseException` arm unmarked, where `classify_route_os_error` converts any
> unmarked `OSError` into a typed filesystem-authority `400` — consuming the
> fault instead of letting it escape after `handle_one_request` accounts the
> drained ledger.
>
> **The first Plan was therefore wrong to call this work an accepted-byte-only
> union.** No accepted source blob contains the resolution, because no accepted
> tree contained both symbols. The shared-primitives preflight had already frozen
> the required resolution — `C18` (per-clause register, around line 206), §5.7
> `ErrorOrigin` (around line 592) and the §9.2 conflict-map error-origin row
> (around line 846) — and the Plan omitted it. This correction adds **exactly one
> preflight-prescribed semantic bridge**, freezes it in full below, and changes no
> other gate: the 18 + 1 + 1 topology, the `B1`/`B2`/`B3` adjudication, the routed
> owner map and the Correction 2 doc-stage model are all preserved. `806f6c8`,
> `a7e283e` and the Correction 2 commit all remain immutable history, and this
> correction carries no SHA of its own.

> **Status vocabulary note.** `docs/development/ai/schemas/task-packet.v1.schema.json`
> constrains the machine `status` field to
> `draft|ready|in_progress|blocked|completed|cancelled` with
> `additionalProperties: false`, so the typed companion carries `"status": "ready"`
> plus the exact token `packet-status:ready_for_bounded_composition` in
> `context.evidence_refs` and invariant `INV-STATUS`. The authoritative packet
> status is **`ready_for_bounded_composition`** and nothing broader. This is a
> schema encoding, not a widening of the authorization.

## Objective

Compose the already-accepted Python HTTP inputs — the MR-list candidate
`b164046` (which subsumes the accepted Doctor/Branch candidate `701469f`) and the
accepted Upload live-route cleanup successor delta `cb13da0..0f43415` — onto the
manager-integrated Plan/FREEZE chain head, **and land exactly one
preflight-prescribed `C18` semantic bridge** that the two accepted inputs
require in order to coexist, producing an exact **three-commit linear chain**:
candidate `C` of exactly 18 paths, writer result `W` of exactly one result path,
and reviewer report `R` of exactly one review path. The aggregate writer write
set is 19 paths, split 18 + 1 across `C` and `W`. It is **not** one commit, and
the writer result is **not** part of the candidate.

`C` alone is the immutable review subject. The product base of `C`'s content
remains `a972cde890e0d621aaced6aca4b45835871da975` at blob level.

This packet authorizes **accepted-input composition plus the single `C18`
error-origin bridge frozen below, and nothing else**. It is *not* a byte-only
union: the two accepted inputs are disjoint on the error-origin surface and
their union is not closed under their own accepted behaviour, so a bounded
semantic bridge is mandatory rather than optional. Every byte outside that
frozen bridge still comes from an accepted source blob. The packet does not
authorize MR-write, integration of any of `C`, `W` or `R`, Install, Dogfood, any
Cargo/Rust work, any other new product behaviour, or any lifecycle exit.

### Exact bases and identities

| Role | Value |
|---|---|
| Immutable product base (blob-level authority) | `a972cde890e0d621aaced6aca4b45835871da975`, tree `76a7342031f560a116916cfe978979674841c75f`, parent `1a9e01d462ed39599a1d8cf0b864264cf0861d18` |
| Launch base `L` | the final manager-integrated Plan/FREEZE chain `HEAD`, named by the manager launch prompt. Not knowable inside this file; see the execution-base rule |
| Canonical branch | `integration/cfr168-p0-canonical` |
| Preflight's prospective base (historical) | `fb891b01f45e4d1963c48f06b827f8335150dace`, tree `2dadc3088c14fb624fb30d8060eba7466297959a` |
| Divergence `fb891b0..a972cde` | 4 commits, 0 merges, 5 docs paths, **0 product/runtime/test/packaging paths** |
| Manager post-integration gate on `a972cde` | `python3 docs/development/tools/check_ai_development.py --timeout 180` passed 7/7 in 53.188 s — the integration lane's measurement, not this packet's and not the writer's |

### Accepted input authorities

| Input | Candidate | Tree | Accepted report | Report SHA-256 |
|---|---|---|---|---|
| Doctor + Branch | `701469f91dd0c4c52c5628fb3e9d9e9d0ab0c714` | `dd63dd560514aef08925cc8b1dbe815bafcceef4` | `docs/development/ai/swarm-results/cfr168-python-http-doctor-branch-composition-rereview-20260730/review.md` (blob `e5096bbb3a4da181b0bee471619de61f11b15635`) | `c7a1665fdf4a6e1104bbab18ec8051c048ace1d25e865952bbbcd35a06eb19c0` |
| MR-list | `b16404676639336f705feffd9af4857d4a1fcc69` | `36185e320338b67e1610d82fcc81cba310c36026` | `docs/development/ai/swarm-results/cfr168-python-http-mr-list-final-publication-closure-rereview-claude-20260731/review.md` (blob `ec7a09b795e0c3df22b6394ae03c465aa90f42c7`) | `5c952a9a5c162bfbb93522bb644fcba44ce32ddf3628501e6f94308899fe760e` |
| Upload live-route cleanup | `0f4341561ce95935600bf1398b8a5fbdb206daba` | `da356c2a041e57eea9192ab917f3a293239c60e7` | report commit `c54e5ce4ab9e8f58f1db88ae59edceae96653058`, path `docs/development/ai/swarm-results/cfr168-python-http-upload-live-route-cleanup-rereview-claude-20260802/review.md` (blob `216ee2680872c43dcd7b657e3a8d4312dbd9b1e9`) | `46e7c3e41ac1b132766657e445e017b392ce6a0b72eb46f909b13755fc8a9b3c` |

`merge-base(701469f, b164046) = 701469f`: the Doctor/Branch input is a strict
ancestor of the MR-list input and is fully subsumed by it. There is nothing
separate to replay for Doctor/Branch.

The Doctor/Branch `ACCEPT` carries an explicit carve-out — it neither accepts nor
rejects the adjacent upload slice from `0703f71` — so no duplicate-omission
argument in this packet may rest on that report. The omission rests on the Upload
acceptance chain and on measured blob identity instead.

### Initial-Upload duplicate

`git show 0703f71` and `git diff 5254e3d..cb13da0` share the stable patch-id
`9889888cedeba7344fb987ff5891cf728d9c7e36`. The **`b164046` copy is kept**,
arriving inside the accepted MR-list tree at `0703f71`. The Upload lineage's own
`5254e3d..cb13da0` limb is **omitted**, and only the successor delta
`cb13da0..0f43415` is applied. On the one path both copies touch,
`tests/test_python_http_upload_accounted_traversal.py` is blob
`141dcbd4ee1f8727fbe146faef481cf746c8c73f` at `cb13da0` **and** at `b164046`, so
the omission is a no-op there and the newest 36-test successor blob is taken
whole.

## Manager adjudication of the shared-primitives `B1`/`B2`/`B3`

The original authority
`docs/development/ai/swarm-results/cfr168-python-http-shared-primitives-preflight-claude-20260731/preflight.md`
(blob `ec2a264142c6625dadd4809d99cf18d3371e9847`, SHA-256
`ca1a1451bfc1b25ed1c54f1b012d4d0e508ef0a96f88401e61284c6d156a4441`, unchanged at
`a972cde`) records verdict **`BLOCK`** with blocking conflicts `B1`, `B2` and
`B3`. That report is preserved byte-for-byte, is not rewritten, and is **not**
reinterpreted as `READY`. Its `BLOCK` remains the standing verdict for the
subject it actually adjudicated: the MR-write repair packet. What follows is a
manager decomposition of that `BLOCK` for a *different, narrower* subject — the
accepted-input composition — and nothing else.

### `B1` — resolved only for the accepted-input composition writer

`B1` failed because the frozen MR-write repair packet's `base_revision`
`fcb2324145962e79d7c6ee173b9cf545733b957d` carries a `codefire` byte-identical to
the rejected candidate and contains none of the fourteen accepted primitive
symbols the repair must reuse, and because choosing or creating a base is
manager-owned composition that the preflight declared a Non-Goal.

**Manager decision.** For the accepted-input composition writer only, `B1` is
resolved by this packet, because the manager now performs exactly the act the
preflight said was manager-owned: it assigns the base and the input graph.

- Immutable product base, at blob level: `a972cde890e0d621aaced6aca4b45835871da975`. The launch base `L` is the manager-integrated Plan/FREEZE chain head, which must carry those same product blobs.
- Accepted input graph: `b164046` (subsuming `701469f`) plus the successor delta `cb13da0..0f43415`, with the initial-Upload duplicate omitted as measured above.
- Nothing must be guessed: every source blob, three-way base, expected pre-bridge target blob and omission is named in this packet and in the bound preflight, and the one place where accepted bytes are provably insufficient — the `C18` error-origin bridge — is frozen in full above from the shared-primitives preflight's own `C18`, §5.7 and §9.2 clauses.

**Explicitly not resolved.** MR-write's rejected/frozen base
`fcb2324145962e79d7c6ee173b9cf545733b957d` remains blocked. `B1` is untouched for
MR-write; no MR-write base has been rebound, and this packet does not rebind one.

### `B2` — released only as the composition resolution action; remains OPEN

`B2` failed because the accepted MR-list tree `b164046` carries, on its two live
`handle_http_upload` routes, the `_AccountedUploadCommitSource` cleanup defect
held OPEN as `HIGH-2`, plus an inverted primary/cleanup error precedence at
`b164046:7047`-`7049` — so accepted product bytes and a frozen rejecting
specification contradicted each other about the same symbol, with no frozen
packet owning the accepted tree's copy.

**Manager decision.** The accepted Upload live-route successor now supplies the
missing behaviour, and the **exact composition of `b164046` plus
`cb13da0..0f43415` owns the product contradiction**. The `ACCEPT` at `c54e5ce`
disposes `HIGH-2` as closed on all three limbs for candidate `0f43415`, and the
bound preflight's inversion census measures the contradiction's disappearance in
the composed `codefire`: `raise cleanup_error from run_error` occurrences go
`cb13da0` 2 -> `b164046` 2 (including the cited `b164046:7049` site) ->
`0f43415` 1 -> composed **1**, with the surviving occurrence in
`run_server_verification` and **zero** in
`enforce_remote_commit_signature_policy_accounted`.

**`B2` remains OPEN.** It is released here **only** as the authorization to
perform the composition, not as a finding that the contradiction is already
discharged in the repository. `B2` closes only when candidate `C` receives a
fresh exact `ACCEPT` in report `R`. Until then:

- `C` is a candidate, not an accepted tree;
- no document may state that `B2` is resolved in the repository, only that the composition is its assigned resolution action;
- the `run_server_verification` inversion is pre-existing on both lineages, is not composition-introduced, and routes to `C14-LIVE-A1`.

### `B3` — remains OPEN, routes to `C14-LIVE-A2`, blocks MR-write

`B3` failed because `_PreparedHTTPMutationResponses` is byte-identical in all five
bound trees while two frozen specifications give directly opposed rules for the
post-first-authoritative-effect window (accepted GET: publish nothing on disposal
failure; MR-write `F5`: return the selected success/durable bytes), and no
accepted or rejecting report states the phase index that reconciles them.

**Manager decision.** `B3` is **not** resolved, not narrowed and not reassigned by
this packet.

- `B3` routes to **`C14-LIVE-A2`** post-effect / response / wire work, to be undertaken *after* composition as its own bounded subject with its own authority.
- `B3` continues to **block MR-write and global closure**. Nothing in this packet releases either.
- `B3` does **not** prevent this composition, for one reason only: it must not change or claim the `B3` surface. `_PreparedHTTPMutationResponses`, the handler `POST` arm and the request-loop `finally` are carried through **byte-identical** to their accepted sources, and the `C18` bridge does not touch them either — it changes `classify_route_os_error`'s verdict for one origin, not the arm that consults it, so `if authority is None: raise` is the accepted code path taken with an accepted-shaped input. Stated honestly: for the standalone cleanup fault the observable outcome does change, from a typed `400` to no response at all — but that **is** the accepted Upload behaviour at `0f43415`, restored, not a new post-effect rule. No phase index is claimed, no post-effect clause is implemented, and `B3` and `C14-LIVE-A2` are untouched. A composition that altered that surface, or that claimed a phase index for it, would be outside this authorization.

### Routed residual ownership map — each locus owned exactly once

`git grep C14-LIVE` over `a972cde` returns nothing: these handles do not yet
exist in-tree. Candidate `C` registers them as **OPEN** in
`docs/development/code-review-issues/cfr-168.md`. **Registration is bookkeeping
only. The composition writer registers these handles OPEN and must not implement,
narrow, repair or close any of them.**

**Every routed locus below has exactly one owner, and no locus appears in two
rows.** In particular, correcting the overlap published in `806f6c8`:
`AccountedServerVerification` and `run_server_verification` are owned by
`C14-LIVE-A1`; the handler `POST` arm and the request-loop `finally` are owned by
`C14-LIVE-A2`; all four are **removed** from `C14-SUCCESSOR-OWNER`.
`validate_sealed_commit` is owned by `C14-LIVE-A1` for its constructor-unwind
cause-link locus and appears in no other row.

| Handle | Scope rule | Loci owned | State |
|---|---|---|---|
| `C14-LIVE-A1` | reachable **pre-effect** loci | the inherited `RetryableCleanup` constructor/traceback cleanup debt — constructor unwinds that do not attach the cleanup error as `__cause__` in `_open` and in `validate_sealed_commit`; the `F6` persistent-teardown non-recovery reproduced at 11,456 retained bytes; `F7` `_open` handle orphaning; every reachable pre-effect cleanup owner on the two commit-source wrappers; and `AccountedServerVerification` / `run_server_verification`, including the one surviving `raise cleanup_error from run_error` there, which is pre-existing on both lineages and not composition-introduced | OPEN |
| `C14-LIVE-A2` | `B3` **post-effect, response and wire** loci | `_PreparedHTTPMutationResponses`' post-effect ownership/lifetime contract (`B3` itself); the handler `POST` arm; the request-loop `finally` and the wire-side publication window; plus promoting `C14`/`B2` from a review artifact to a durable in-tree contract document, and the `shape-pending` state of `RetryableCleanup` | OPEN |
| `C14-SUCCESSOR-OWNER` | **only** production-unwired or deferred global loci **not already owned by `A1` or `A2`** | `AccountedHTTPUploadAncestry` and `AccountedHTTPUploadSealedAncestry` (production-unwired successors); planner / stage / sealed factories; signature and ancestry owners outside the two commit-source wrappers, explicitly excluding `AccountedServerVerification` and `run_server_verification`; `F9` `_control_charge_witness`, deferred pending a recorded manager decision; `O2` live-route oracle insensitivity to `m2`, deferred, oracle strength only; and `O3` `except BaseException as retry_error: detach(retry_error)`, deferred and non-blocking per the Upload `ACCEPT` | OPEN |

Also carried forward and **not** `C14` loci: CFB-138 default-profile upload scale,
MR-write, metrics/conformance, Install, Dogfood and terminal lifecycle. All remain
predecessor-gated per closure-mode items 3-6.

The `O1` evidence-authority observation from the Upload `ACCEPT` is discharged by
citation, not by authoring a new authority: `C` adds a relative citation to the
existing in-tree shared-primitives preflight at the `C14`/`B2` mentions in the
two merged canonical source docs. The `C14-LIVE-A2` contract half stays OPEN.

### MR-write and closure

MR-write remains **blocked** until (a) candidate `C` is accepted by the fresh
exact review in `R` and (b) the later required `C14` resolution lands. `B1` stays
blocked for MR-write's own base; `B3` stays open against MR-write's post-effect
window. No root row and no issue state changes: in
`docs/development/current-cycle-burn-down-plan.md`, root row 6
(`CFR-168 + CFB-136 + CFB-138`) stays `Implement in progress`, and no `Close` is
performed on any root, issue or cycle.

## The `C18` composition bridge — frozen

### Authority

Three frozen clauses of
`docs/development/ai/swarm-results/cfr168-python-http-shared-primitives-preflight-claude-20260731/preflight.md`
prescribe this bridge. They are quoted, not reinterpreted, and that report is
**not** edited:

| Locus | Frozen requirement |
|---|---|
| `C18`, per-clause register (around line 206) | "One typed, exhaustive error-origin taxonomy shared by GET and POST: filesystem, transport, deadline, budget, integrity, **post-commit cleanup** — classified at the boundary that raised it, never by message text." Marker `frozen`. Recorded as behavior-closed inside `b164046` and **absent from every other tree** |
| §5.7 `ErrorOrigin` — **adapt** (around line 592) | *Reuse:* the accepted taxonomy — `os_error_origin`, `classify_route_os_error`, `route_filesystem_boundary`, `mark_transport_origin` — "shared across GET and POST with no second route-local protocol". *Adapt:* "add the post-commit cleanup origin" |
| §9.2 conflict map, error-origin row (around line 846) | Error-origin taxonomy, `b164046` only, touched by MR-list, MR-write and Upload: "reuse the accepted taxonomy; add only the post-commit cleanup origin". Risk if unresolved: "second route-local taxonomy" |

### Measured incompatibility

Independently reproduced on the staged composition, read-only:

| Tree | `classify_route_os_error` / `os_error_origin` | `_drained_commit_source_cleanup_error` |
|---|---|---|
| `cb13da0` (base) | absent | absent |
| `b164046` (ours, MR-list) | **present** | absent |
| `0f43415` (theirs, Upload) | absent | **present** |
| composed `C` | present | present — **first tree where both exist** |

The two accepted inputs are disjoint on this surface, so neither `ACCEPT` ever
exercised the interaction, and the union is **not closed** under the accepted
behaviour of either input. Concretely, at the composed tree:
`_drained_commit_source_cleanup_error` returns the first one-shot `close(2)`
`OSError` **unmarked**; both live call sites raise it standalone as `raise
cleanup_error from None`; `do_POST`'s `except BaseException` arm calls
`classify_route_os_error`, which returns `None` only for a non-`OSError` or a
transport-marked `OSError` and otherwise marks the error filesystem-origin; so
the cleanup fault is published as a typed filesystem `400` and never escapes.
Measured effect: `test_request_loop_accounts_a_drained_cleanup_fault` fails both
subtests with `len(handler_errors)` `0`, expected `1`, while the same test passes
at `0f43415`.

### The frozen resolution

**One** shared error-origin taxonomy, extended by exactly one origin. No second
classifier anywhere.

1. **Three distinct origins on the one shared taxonomy** — `filesystem`, `transport`, and a new **post-commit / commit-source-cleanup** origin. `os_error_origin` becomes exhaustive over them and continues to name the boundary, never the exception class and never the message.
2. **Classification is at the boundary or owner that raised the error.** The commit-source cleanup origin is marked by the owner of that cleanup — the standalone `OSError` that `_drained_commit_source_cleanup_error` returns is marked there, at the point the drain produced it. It is *not* marked by the route, the dispatcher, an errno test, a message test or an exception-subclass test.
3. **`classify_route_os_error` must refuse the cleanup origin**, returning `None` for it exactly as it already does for the transport origin. It therefore reaches the `if authority is None: raise` arm, escapes `do_POST`, and escapes `handle_one_request` **after** the request loop has observed the drained ledger — `current` `{}`, `open_permit_count` `0`, `terminal_reason` `complete`, zero response bytes and zero durable bytes.
4. **Everything else in the accepted taxonomy is preserved unchanged.** A generic **unmarked** route `OSError` — including one raised by a patched `handle_http_upload` — still becomes the accepted typed filesystem `400`; a transport-origin `OSError` still abandons the connection without fabricating a response; `MemoryError` and every other process-fatal condition stay outside the `OSError` arm; and a route-raised `ConnectionResetError` or `TimeoutError` still publishes the typed diagnostic, because the class alone proves nothing.
5. **`C14` precedence is untouched.** When the operation itself failed, the operation decision remains the outer exception and the cleanup error remains its `__cause__`. The cleanup origin is a *classification* of the cleanup error, never a promotion of it. Marking a cleanup error that is only a `__cause__` must not change what `classify_route_os_error` decides about the outer exception.

### Forbidden implementations

Reuse and adapt the accepted taxonomy only. Each of the following is explicitly
out of bounds, and the third and fourth are exactly what the preflight's
"second route-local taxonomy" risk names:

- any second, route-local or Upload-local error classifier;
- message-text matching, errno allowlists or denylists, or exception-subclass heuristics as the origin decision;
- disabling, bypassing or route-scoping `classify_route_os_error` for the upload routes;
- broadly removing or weakening filesystem classification so that unmarked `OSError`s stop publishing the typed `400`;
- marking the cleanup origin anywhere other than the cleanup owner, or inferring it from the exception chain, the traceback or the call stack.

### Bridge write scope — 5 of the existing 18 candidate paths

The bridge stays **inside** the existing 18-path candidate set. No path is added
to `C`, and the topology stays 18 + 1 + 1. Exactly these five paths carry
bridge bytes; the other thirteen remain byte-identical to their accepted sources
or to their Correction 2 stage-two values.

| # | Path | Bridge change | Why it is the minimum |
|---:|---|---|---|
| 1 | `codefire` | the taxonomy extension itself: the new cleanup-origin marker and predicate, `os_error_origin` made exhaustive over three origins, the mark applied at the `_drained_commit_source_cleanup_error` owner, and the `classify_route_os_error` refusal | the bridge is a product-behaviour change and has nowhere else to live |
| 6 | `tests/test_python_http_upload_accounted_traversal.py` | one new direct cross-input oracle, plus no change to the existing 36 methods | see the oracle decision below |
| 7 | `docs/development/source-architecture.md` | state the third origin where the doc today says the dispatchers publish the typed family "for any filesystem-origin `OSError`" and that origin is marked at transport or filesystem boundaries only | the doc currently states a live truth that the bridge makes incomplete |
| 8 | `docs/development/command-source-trace.md` | state the third origin where the doc today describes `classify_route_os_error` as "the one shared decision both dispatch arms" over transport and filesystem | same |
| 9 | `docs/development/change-playbook.md` | correct the bullet that today asserts "only client transport failures still abandon it", and the origin-classification bullet that enumerates transport and filesystem only | the bridge makes that sentence **false**: a commit-source cleanup `OSError` now also escapes |

Explicitly **not** edited by the bridge, and therefore still frozen at their
existing values: `docs/development/source-data-model-catalog.md`, which carries
no error-origin content, and `tests/test_python_http_mr_list_decision_work.py`,
which stays **byte-identical to its accepted source** so that
`test_dispatcher_publishes_every_route_filesystem_failure` remains an
unmodified control rather than an oracle the writer touched.

### The new cross-input oracle — required, and why

The two existing tests are each single-sided. The Upload test proves the cleanup
fault escapes; the MR-list test proves a generic unmarked `OSError` is typed. **A
bridge that simply route-scoped the classifier for uploads would pass both**, and
neither test asserts the taxonomy contract itself. One new direct oracle is
therefore required. It is added **inside already-listed path 6**, not as a new
path, so the 18-path candidate set is unchanged. Path 6 is the right host: it
already owns the cleanup-origin producer and the real-socket one-shot `close(2)`
harness, and putting it there leaves the MR-list module untouched as a control.

The oracle must prove, directly:

1. `os_error_origin` returns the cleanup origin for the standalone error the production drain returns, and the taxonomy is exhaustive over the three origins;
2. `classify_route_os_error` returns `None` for a cleanup-origin `OSError` and a non-`None` typed authority for an otherwise identical unmarked one;
3. **identity, not message or type**: two `OSError`s constructed with the *same* errno, message and filename classify differently when and only when one carries the production cleanup mark — so no message, errno or subclass heuristic can satisfy the oracle;
4. `C14` precedence is unaffected — a cleanup error carried as `__cause__` of a failed operation decision does not change the outer exception's classification.

Test-count effect: the Upload module goes **36 → 37** methods. `V2` becomes
`37/37`, `V6` becomes `539/539` in 34 modules, and `V8` becomes `767/767` in 35
modules. `V3` `49/49`, `V4` `32/32`, `V5` `59/59` and `V7` `228/228` are
unchanged. All six baselines were counted on the staged composition.

### Scope of what the bridge closes

On a fresh exact `ACCEPT` of `C` in `R`, this bridge closes **only** the `C18`
composition incompatibility between the two accepted inputs. It does **not**
close `B2` — which closes only on that same `ACCEPT`, and not before review — and
it does not close `B3`, `C14-LIVE-A2`, the constructor-unwind or global `C14`
debt, `C14-LIVE-A1`, `C14-SUCCESSOR-OWNER`, MR-write, any root issue, the cycle,
Install or Dogfood. The bridge adds the post-commit cleanup **origin** to the
taxonomy; it does not implement the §5.7 post-effect publication clause, which
remains `B3`/`C14-LIVE-A2` work.

## Scope

### Execution base and launch rule

The **immutable product base is `a972cde890e0d621aaced6aca4b45835871da975`**, and
that is what the typed companion's `base_revision` records. It is a blob-level
and schema-level authority, not the execution base: a commit cannot name itself,
so no Plan/FREEZE file can carry the SHA of the chain that publishes it.

1. **Integration comes first.** The complete Plan/FREEZE chain — `806f6c8`, the `a7e283e` topology correction, and this successor blob-stage correction commit — is integrated onto `integration/cfr168-p0-canonical` by the manager before any writer starts. Each correction extends the chain; none replaces an earlier commit.
2. **The launch base `L` is that final manager-integrated Plan/FREEZE chain `HEAD`**, and the manager names its exact SHA in the launch prompt. The writer must not infer it and must not substitute anything else.
3. `L` must satisfy, and the writer must prove before writing: `L` **linearly descends from `a972cde`** (`git merge-base --is-ancestor a972cde L`, zero merges in `a972cde..L`); `git diff --name-only a972cde..L` contains **docs-only** paths and **zero** product, runtime, test or packaging paths; and every product and composition-target blob at `L` is identical to its `a972cde` value: `codefire` `61214a21fd866efc0d9a8888dfbfc8e707039d48` at mode `100755`; `tests/test_codefire_cli.py` `10317e13deb531d04b9df44e756d7b8659b31c82`; `docs/development/source-architecture.md` `0fe3dc8470ee2a477afae7fb874fe645071908e2`; `docs/development/command-source-trace.md` `5feeaff7784cea83ad5db9205ddc648fe89703f6`; `docs/development/change-playbook.md` `12d7fc6ab41177454a3f8c2b3cb0ee156055f83f`; `docs/development/source-data-model-catalog.md` `77623025f184a89a87cbcfb6447fb082edc77733`; `docs/development/code-review-issues/cfr-168.md` `3d9a4df9c19eff20a0d38bef4ff2c6a75a919bb8`; and the four new test modules plus the seven imported evidence paths still absent.
4. **The writer is never launched on `a972cde` once the Plan/FREEZE chain is integrated.** Launching on `a972cde` after integration would drop this packet and its correction from the writer's own tree.
5. **Generic `run_ai_role` must not be used to launch this lane after integration.** `run_ai_role.preflight_task_execution` compares the packet's `base_revision` against `HEAD` and errors with `task base_revision ... differs from HEAD ...` whenever they differ. Because `base_revision` intentionally stays pinned at the immutable product base `a972cde` while `L` is a later docs-only head, that rejection is **correct behaviour and must not be worked around** — not with a base_revision edit, not with a flag. Launch is **manual, via second-agent**, with the exact integrated head bound by the manager in the launch prompt.
6. Any mismatch in step 3 is a hard abort with no edit, and returns the base question to the manager.

### Exact three-commit topology

Three commits, strictly linear, in this order. No merge commit anywhere.

| Commit | Parent | Write set | Role |
|---|---|---|---|
| `C` — candidate | `L`, the final manager-integrated Plan/FREEZE chain head | **exactly the first 18 aggregate paths**: product, tests, the four canonical source docs, the imported acceptance evidence, and the CFR-168 registration | the **immutable review subject**. One commit, zero merges, clean tree |
| `W` — writer result | `C` | **exactly one `ADD`**: `docs/development/ai/swarm-results/cfr168-python-http-accepted-input-composition-20260802/result.md` | writer evidence. One commit, zero merges, **no candidate byte changed** |
| `R` — reviewer report | `W` | **exactly one `ADD`**: `docs/development/ai/swarm-results/cfr168-python-composition-exact-review-claude-20260802/review.md` | report-only. One commit, zero merges, **no candidate or result byte changed** |

Rules that follow, and that the older one-commit framing violated:

- **`C` must never contain or claim its own SHA, tree or patch digest.** Nothing inside the 18 paths may reference `C`'s identity. Self-reference is what made the one-commit topology impossible.
- **`W` may record `C` exactly** — SHA, tree, `git diff --binary L..C | sha256sum`, blob ids, mode, test counts and every gate result. That is `W`'s purpose.
- **The reviewer reviews `C`.** `R` binds `W` only as *writer evidence*, never as part of the review subject. A finding against `W`'s prose is a finding against writer evidence, not against the candidate bytes.
- **`R` records both `C` and `W`**, and is not part of `C`.
- The writer must not create `R`; the reviewer must not create `C` or `W` and must not write any of the 18 candidate paths.
- **The manager may integrate the accepted linear chain `C..R` only after `R` is `ACCEPT`.** This packet does not authorize that integration; it remains a separate manager decision.

### Aggregate writer write set — 19 ordered paths, 18 in `C` plus 1 in `W`

| # | Path | Commit | Operation |
|---:|---|---|---|
| 1 | `codefire` | `C` | three-way semantic union, mode preserved `100755`, **plus the `C18` bridge** |
| 2 | `tests/test_codefire_cli.py` | `C` | clean apply from `b164046` (base-equal path) |
| 3 | `tests/test_python_http_doctor_decision_work.py` | `C` | `ADD` from `b164046` |
| 4 | `tests/test_python_http_branch_decision_work.py` | `C` | `ADD` from `b164046` |
| 5 | `tests/test_python_http_mr_list_decision_work.py` | `C` | `ADD` from `b164046`, **unmodified** — the `C18` control oracle |
| 6 | `tests/test_python_http_upload_accounted_traversal.py` | `C` | `ADD` from `0f43415`, duplicate omitted, newest wins, **plus the one new `C18` cross-input oracle** (36 -> 37 methods) |
| 7 | `docs/development/source-architecture.md` | `C` | three-way union + `O1` citation + **`C18` bridge truth** |
| 8 | `docs/development/command-source-trace.md` | `C` | three-way union + `O1` citation + **`C18` bridge truth** |
| 9 | `docs/development/change-playbook.md` | `C` | three-way union + **`C18` bridge truth** |
| 10 | `docs/development/source-data-model-catalog.md` | `C` | three-way union only — no citation, no bridge |
| 11 | `docs/development/ai/swarm-results/cfr168-python-http-upload-live-route-cleanup-rereview-claude-20260802/review.md` | `C` | `ADD` from `c54e5ce` |
| 12 | `docs/development/ai/swarm-batches/cfr168-python-http-upload-live-route-cleanup-repair-20260802.md` | `C` | `ADD` from `0f43415` |
| 13 | `docs/development/ai/swarm-batches/cfr168-python-http-upload-zero-residual-repair-20260730.md` | `C` | `ADD` from `0f43415` |
| 14 | `docs/development/ai/swarm-batches/cfr168-python-http-upload-zero-residual-repair-20260730.json` | `C` | `ADD` from `0f43415` |
| 15 | `docs/development/ai/swarm-results/cfr168-python-http-upload-high2-third-repair-20260731/result.md` | `C` | `ADD` from `0f43415` |
| 16 | `docs/development/ai/swarm-results/cfr168-python-http-upload-zero-residual-review-closure-repair-20260731/result.md` | `C` | `ADD` from `0f43415` |
| 17 | `docs/development/ai/swarm-results/cfr168-python-http-upload-zero-residual-review-closure-repair-20260731/result-2.md` | `C` | `ADD` from `0f43415` |
| 18 | `docs/development/code-review-issues/cfr-168.md` | `C` | hand edit — register `C14-LIVE-A1`, `C14-LIVE-A2`, `C14-SUCCESSOR-OWNER` as OPEN, without implementing any of them |
| 19 | `docs/development/ai/swarm-results/cfr168-python-http-accepted-input-composition-20260802/result.md` | `W` | `NEW` — writer result, a separate commit whose parent is `C` |

Paths 1-18 are the candidate. Path 19 is **not** part of the candidate and must
not appear in `git diff --name-status L..C`. The single review path is in `R` and
is in neither `C` nor `W`.

`docs/development/current-cycle-burn-down-plan.md` is **not** in any writer or
reviewer write set. The bound preflight offered it as an optional writer path and
recommended deferring it to the manager; the manager accepts that recommendation
and owns every burn-down edit.

### Read set

The bound preflight; this packet and its typed companion; the three accepted
reports; the original shared-primitives preflight; `AGENTS.md`;
`docs/development/current-cycle-burn-down-plan.md`;
`docs/development/code-review-issues/cfr-168.md`; the four canonical source docs;
the original shared-primitives preflight's `C18` register row, §5.7 `ErrorOrigin`
and §9.2 conflict-map error-origin row, which are the `C18` bridge authority;
and read-only `git show` / `git diff` over `701469f`, `b164046`, `cb13da0`,
`0f43415`, `c54e5ce`, `5254e3d`, `16d0129`, `5790463`, `0703f71`, `fb891b0`,
`a972cde` and `L`.

### Disallowed paths

Everything outside the 19 aggregate paths for the writer, and everything outside
the single review path for the reviewer. Named explicitly because they are the
live hazards:

- the 5 canonical-newer management docs — `docs/development/README.md`, `docs/development/current-cycle-burn-down-plan.md`, `docs/development/history.md`, `docs/development/codefire-improvement-cycle.md`, `docs/development/ai/swarm-batches/cfr168-recovery-execution-ledger-20260730.md` — must never be checked out from any candidate;
- the AIA-046 Cargo/O1 governance surface — `docs/development/ai/source-keyed-cargo-gates.md`, `docs/development/tools/run_source_keyed_cargo_gate.py`, `docs/development/ai/task-packets/cfr168-source-keyed-cargo-canonical-port-20260731.md`, the three `aia-source-keyed-cargo-*` review reports, and closure-mode item 1;
- `crates/`, `Cargo.toml`, `Cargo.lock`, `install.sh`, `pyproject.toml`, `setup.py`, `demo.sh`, `schemas/`, `adr/`, top-level `README.md`;
- the 18 already-identical candidate docs, the superseded `…upload-third-repair-rereview-claude-20260731/review.md` `REQUEST_CHANGES` report, and the three stale open-audit artifacts — leave all untouched, delete none;
- the original shared-primitives preflight and the published composition preflight — read-only, never edited;
- this packet and its typed companion — read-only to the writer and the reviewer;
- any `.claude-second-agent`, `.claude-worktrees`, `.codex-second-agent`, `.fugu-*`, `.dev-swarm` path, and any agent state, log or worktree debris. The five canonical-newer management docs and agent state must not be imported under any circumstance.

### Disallowed actions

No `git cherry-pick` of any candidate range; no `git merge` of any candidate; no
merge commit anywhere in `C`, `W` or `R`; no amend, rebase or reset of `C` once
frozen; no squashing `C`, `W` and `R` into fewer commits; no self-SHA inside `C`;
no launching on `a972cde` after the Plan/FREEZE chain is integrated; no generic
`run_ai_role` launch after integration, and no editing `base_revision` to force
one; no Cargo/Rust command; no `install.sh`; no dogfood; no network; no Docker; no
`git reset --hard` and no `git checkout --` over user changes; no second
route-local error classifier; no message-text, errno-list or exception-subclass
origin heuristic; no route-scoping or disabling of `classify_route_os_error`; no
broad removal of filesystem classification; no product change beyond the frozen
`C18` bridge; no integration of
`C`, `W` or `R`; no `Close` on any root, issue or cycle; no edit to the original
shared-primitives preflight or the published composition preflight; no self-review
by the writer; no implementation of `C14-LIVE-A1`, `C14-LIVE-A2` or
`C14-SUCCESSOR-OWNER`.

## Exact semantic procedure

Semantic apply throughout, then exactly one frozen semantic bridge. Steps 1-8
compose accepted bytes; step 8 lands the `C18` bridge frozen above and is the
only step that introduces behaviour absent from every accepted source blob. Both candidate lineages carry management docs that
would clobber the canonical governance and Cargo lineage, so no range may be
cherry-picked.

1. **Verify start state, abort on any mismatch.** `HEAD` equals the launch base `L` named by the manager; `git rev-parse HEAD^{tree}`; `git rev-parse HEAD:codefire` = `61214a21fd866efc0d9a8888dfbfc8e707039d48`; `git status --porcelain` empty; every step-3 condition of the execution-base rule proven, including linear docs-only descent from `a972cde`; all `ADD` paths absent.
2. **Product fast-forwards from `b164046`.** `b164046` supplies `codefire`, `tests/test_codefire_cli.py`, and the Doctor, Branch and MR-list decision-work tests. `701469f` is subsumed and is not replayed. The base blob is identical on `tests/test_codefire_cli.py`, so only ours changed it and the apply is clean.
3. **Merge the Upload successor onto `codefire`.** Three-way `git merge-file -p --diff3` with ours `b164046:codefire` `f30575a697c1b525653c13738987ccdb5c0151ea`, base `cb13da0:codefire` `03966af23dd860c5235832320a8f1977ecd6495a`, theirs `0f43415:codefire` `b984c1e7e393af658a618c96270d33d39bf8eeaa`. Not `git apply` against the working tree: the working `codefire` is now ours and the delta's context is the base's. Expect exit 0, **zero** conflict markers, exactly additive line arithmetic, mode preserved `100755`, and **pre-bridge** target blob **`f5e0b6db7c15aeadd3025ae4f038e301a7634caf`**. Recompute with `git hash-object codefire` — note the bound preflight's `W2` pseudocode contains the typo `codefier`; use `codefire`. This is the union value **before** the step-8 `C18` bridge; it is a required intermediate checkpoint and is *not* the blob committed in `C`.
4. **Upload test module: newest wins, duplicate omitted.** Take `0f43415`'s 36-test module, **pre-bridge** blob `829836bc0411bf82030286e6f1b0e3d58e574e14`, **not** `b164046`'s 10-test duplicate `141dcbd4ee1f8727fbe146faef481cf746c8c73f`. Ours equals base on this path, so this is a clean fast-forward, not a merge. Step 8 then adds one oracle to this module, so `829836bc…` is a required intermediate checkpoint and is *not* the blob committed in `C`.
5. **Stage one — three-way union the four canonical source docs**, with `cb13da0` as base, ours `b164046`, theirs `0f43415`. This is an **intermediate** stage: paths 7 and 8 are edited again in step 7 and paths 7, 8 and 9 again in step 8, so for three of these four files these blobs are *not* the bytes committed in `C`. Only `docs/development/source-data-model-catalog.md` keeps its stage-one value all the way into `C`. Reconstruct them, in a scratch location or in the working tree before any hand edit, with `git merge-file -p --diff3 <ours> <base> <theirs>`, expecting exit 0 and zero conflict markers on each. The four **pre-citation union blobs**, in full:

   | Path | Pre-citation union blob (stage one) |
   |---|---|
   | `docs/development/source-architecture.md` | `d8d7aa9881b5f96bcfa13b78168403be9ae62512` |
   | `docs/development/command-source-trace.md` | `0a8d2a886c883a051fe8c0e74bf550da3821b977` |
   | `docs/development/change-playbook.md` | `9a3afd7dd932bd3db7f33909e3cda40fb867abfc` |
   | `docs/development/source-data-model-catalog.md` | `2b68347eaa357f187f794c2a76992401743929eb` |

   These are the values the bound preflight abbreviates as `d8d7aa9881b5…`, `0a8d2a886c88…`, `9a3afd7dd932…` and `2b68347eaa35…`; the preflight is correct and unchanged, and it describes **this stage only**. Expect zero conflicts, zero heading collisions, zero duplicate headings. Preserve the Upload-side scoping sentence that bounds the synchronous drain guarantee to the two commit-source routes and to nothing else.
6. **Add only the exact listed evidence files** — aggregate paths 11 to 17. Do not import the five canonical-newer management docs, any agent state, or any other candidate doc.
7. **Stage two — hand-authored edits inside `C`.** Add the `O1` **relative** citation to the existing in-tree shared-primitives preflight at the `C14`/`B2` mentions in paths 7 and 8 — a repo-relative link to `ai/swarm-results/cfr168-python-http-shared-primitives-preflight-claude-20260731/preflight.md`, resolving from `docs/development/`. This citation, and nothing else, is what moves paths 7 and 8 off their stage-one union blobs. Paths 9 and 10 carry no citation and are therefore **byte-identical to their stage-one union blobs at this stage-two checkpoint**. That is a statement about the checkpoint only: path 9 is edited again by the step-8 `C18` bridge, so **only path 10 is still at its stage-one value in the committed candidate**. Register `C14-LIVE-A1`, `C14-LIVE-A2` and `C14-SUCCESSOR-OWNER` as OPEN in path 18, using the ownership map above verbatim in effect and implementing none of them.

   The four **stage-two** blobs, after the citation and before the step-8 bridge:

   | Path | Stage-two blob | Relation to stage one |
   |---|---|---|
   | `docs/development/source-architecture.md` | `5a1755ebd7e0eff8f55f92ae940557badb3ca567` | union `d8d7aa98…` + `O1` citation only |
   | `docs/development/command-source-trace.md` | `72953356fca0982596bbb81a939fd4e2f2584a75` | union `0a8d2a88…` + `O1` citation only |
   | `docs/development/change-playbook.md` | `9a3afd7dd932bd3db7f33909e3cda40fb867abfc` | unchanged from stage one |
   | `docs/development/source-data-model-catalog.md` | `2b68347eaa357f187f794c2a76992401743929eb` | unchanged from stage one |

   The stage-one → stage-two delta on paths 7 and 8 must contain **only** the `O1`
   citation and the minimal reflow it forces. No other sentence, claim, heading or
   ordering may change, and in particular the `C14`/`B2` statements keep their
   existing meaning: the citation names where the authority lives, it does not
   restate, narrow or resolve it. Stage two is **final only for
   `source-data-model-catalog.md`**; paths 7, 8 and 9 receive the step-8 bridge
   edit and reach their committed values at stage three.
8. **Land the frozen `C18` bridge — stage three.** Implement exactly the resolution frozen in "The `C18` composition bridge", touching only aggregate paths 1, 6, 7, 8 and 9, and honouring every forbidden-implementation rule. Mark the standalone cleanup `OSError` at its owner in `_drained_commit_source_cleanup_error`; make `os_error_origin` exhaustive over filesystem, transport and the post-commit / commit-source-cleanup origin; make `classify_route_os_error` refuse the cleanup origin exactly as it refuses the transport origin; add the one new cross-input oracle to path 6; and state the third origin in paths 7, 8 and 9. Then recompute and publish the **stage-three final blobs** for `codefire` and for paths 6, 7, 8 and 9 — none of these is knowable in advance, so this packet freezes their *pre-bridge* inputs and the *delta contract*, not their final hashes. `docs/development/source-data-model-catalog.md` must still hash `2b68347eaa357f187f794c2a76992401743929eb`, and `tests/test_python_http_mr_list_decision_work.py` must still equal its accepted-source blob. Prove the bridge delta on `codefire` is confined to the named taxonomy symbols and their two call sites, with no other unit changed.
9. **Freeze `C`.** One clean additive commit containing exactly aggregate paths 1-18 and nothing else: `git rev-list --count L..C` = 1; `git rev-list --merges L..C | wc -l` = 0; `git diff --name-status L..C` exactly the 18 paths; `git status --porcelain` empty. `C` contains no reference to its own identity.
10. **Commit `W` on top of `C`.** Exactly one `ADD` at aggregate path 19, recording `C`'s SHA, tree, `git diff --binary L..C | sha256sum`, the six product blob ids and `codefire` mode, **all three** canonical source-doc blob stages in full — the four stage-one pre-citation union blobs, the four stage-two post-citation blobs, and the four stage-three committed blobs, with the deltas between each pair identified — the **pre-bridge and final** identities of `codefire` and of aggregate path 6, the omitted duplicate patch-id, the test counts, every gate and reversal result, and every item not run marked not run. `git diff --name-status C..W` is exactly that one path; `git diff C..W -- <the 18 candidate paths>` is empty. `W` must state truthfully that `C` is composed-not-accepted, that it closes neither CFR-168, CFB-136, CFB-138, global `C14`, `B3`, MR-write, integration, Install nor Dogfood, and that `C14-LIVE-A1`, `C14-LIVE-A2` and `C14-SUCCESSOR-OWNER` are registered OPEN and unimplemented.
11. **Then, and only then, the fresh report-only review starts** and produces `R` on top of `W`.

### Anti-false-closure rule

No composition text may apply `Close`, `Closed`, `complete`, `zero-residual` or
`ACCEPT` to CFR-168, CFB-136, CFB-138, global `C14`, MR-write or the cycle. Only
two narrow claims are licensed by evidence: `HIGH-2` is closed on all three limbs
**for candidate `0f43415`** per report SHA-256 `46e7c3e4…`; and the `B2` product
contradiction between `b164046` and the shared-primitives preflight is
**measured as eliminated on the composed tree**, which is a candidate-scoped
measurement and not a repository-level resolution of `B2`.

The `C18` bridge adds a third licensed claim and no more: the `C18` **composition
incompatibility** between `b164046`'s taxonomy and `0f43415`'s cleanup drain is
resolved in candidate `C`. That claim is candidate-scoped and becomes durable only
on a fresh exact `ACCEPT` in `R`. No artifact may extend it to `B2` before review,
to `B3`, `C14-LIVE-A1`, `C14-LIVE-A2`, `C14-SUCCESSOR-OWNER`, the constructor-unwind
or global `C14` debt, MR-write, any root issue, the cycle, Install or Dogfood, and
no artifact may describe `C18` itself as closed repository-wide.

## Non-Goals

- Composing all 19 aggregate paths into one commit, or treating the writer result or the review report as part of candidate `C`.
- Embedding `C`'s own SHA, tree or patch digest anywhere inside `C`'s 18 paths.
- Launching the writer on `a972cde` after the Plan/FREEZE chain is integrated, or launching through generic `run_ai_role` after integration, or editing `base_revision` to make such a launch pass.
- Resolving `B3`, or stating any phase index for the post-first-effect `PreparedResponse` window.
- Implementing, narrowing, repairing or closing `C14-LIVE-A1`, `C14-LIVE-A2` or `C14-SUCCESSOR-OWNER`; the writer only registers them OPEN.
- Releasing MR-write, or rebinding MR-write's rejected/frozen base — `B1` stays blocked there.
- Declaring the original shared-primitives `BLOCK` to be `READY`, or editing that report.
- Closing `B2` in the repository; `B2` closes only on a fresh exact `ACCEPT` of candidate `C`.
- Integrating `C`, `W` or `R`, or fast-forwarding canonical.
- Any Cargo/Rust work, `install.sh`, packaging, dogfood, Docker or network activity.
- Changing root row 6 off `Implement in progress`, or any issue state or `Close`.
- Fixing the pre-existing `check_ai_development.py` default-timeout behaviour, the `run_server_verification` inversion, `O3`'s `BaseException` handling, `F6`/`F7`/`F9`, or any other routed residual.
- Introducing any new product behaviour, refactor, rename or optimisation beyond the single frozen `C18` bridge; every other byte must come from an accepted source blob.
- Treating the composition as a byte-only union, or requiring `codefire`, aggregate path 6, or aggregate paths 7-9 to equal their pre-bridge or accepted-source blobs in the committed candidate.
- Building a second route-local error classifier, or deciding origin by message text, errno list or exception subclass.
- Route-scoping or disabling `classify_route_os_error`, or broadly removing filesystem classification, to make the Upload oracle pass.
- Implementing the §5.7 post-effect publication clause; the bridge adds the post-commit cleanup **origin** only, and the post-effect window stays `B3`/`C14-LIVE-A2`.
- Closing `C18` repository-wide, or closing `B2` on the strength of the bridge before the fresh exact `ACCEPT` in `R`.
- Authoring a new `C14`/`B2` authority document; the `O1` remediation is a citation only.
- Treating the bound preflight's four abbreviated doc blobs as the bytes committed in `C`, or requiring the committed architecture, trace or playbook files to match their pre-citation union blobs; those are the stage-one intermediate values, the `O1` citation necessarily moves architecture and trace off them, and the `C18` bridge necessarily moves playbook off its one as well.
- Dropping, deferring or relocating the `O1` citation in order to make the committed blobs equal the stage-one union blobs.
- Amending, rebasing or reversing `806f6c8`; it stays immutable history.

## Invariants

| ID | Statement | Falsification |
|---|---|---|
| `INV-LAUNCH` | The launch base `L` is the manager-named, manager-integrated Plan/FREEZE chain `HEAD`; `L` linearly descends from `a972cde` with docs-only differences, zero merges and every product and composition-target blob identical to its `a972cde` value. | `git merge-base --is-ancestor a972cde L` fails, `a972cde..L` contains a merge or any product/runtime/test/packaging path, or any target blob differs. |
| `INV-TOPOLOGY` | The lane is exactly three linear commits `L -> C -> W -> R`: `git rev-list --count L..C` = 1, `C..W` = 1, `W..R` = 1, `L..R` = 3, and `git rev-list --merges L..R \| wc -l` = 0. | Any other count, any merge, or any squash of `C`, `W` and `R`. |
| `INV-CANDIDATE-PATHS` | `git diff --name-status L..C` is exactly the 18 candidate paths — aggregate paths 1-18 — with no extras and nothing missing. | A 19th path in `C`, the result or review path inside `C`, or any missing candidate path. |
| `INV-RESULT-PATH` | `git diff --name-status C..W` is exactly one `ADD` of the aggregate path 19 result report, and `git diff C..W` over the 18 candidate paths is empty. | Any second path in `W`, or any candidate byte changed by `W`. |
| `INV-REVIEW-PATH` | `git diff --name-status W..R` is exactly one `ADD` of the single review path, and `git diff W..R` over the 18 candidate paths and the result path is empty. | Any second path in `R`, or any candidate or result byte changed by `R`. |
| `INV-NO-SELF-REF` | No byte inside `C`'s 18 paths contains or claims `C`'s own SHA, tree or patch digest. `W` may record `C` exactly; `R` may record `C` and `W`. | Any self-identity reference found inside the candidate paths. |
| `INV-BLOBS` | **Pre-bridge**, `codefire` is `f5e0b6db7c15aeadd3025ae4f038e301a7634caf` at mode `100755` and the upload traversal test is `829836bc0411bf82030286e6f1b0e3d58e574e14`. **In `C`**, both differ from those values by the frozen `C18` bridge and nothing else, mode still `100755`, and the writer publishes the final hashes it measured. `tests/test_python_http_mr_list_decision_work.py` and `docs/development/source-data-model-catalog.md` are byte-identical to their accepted-source and stage-two values respectively. | A pre-bridge checkpoint that misses its value; a mode change; a committed `codefire` or path-6 blob whose delta from the checkpoint is not exactly the frozen bridge; any change to the MR-list module or the catalog; or any artifact asserting the committed blobs equal the pre-bridge values. |
| `INV-C18-BRIDGE` | `C` contains exactly one semantic bridge: one shared taxonomy with three origins — filesystem, transport, post-commit/commit-source-cleanup; the cleanup origin marked at the `_drained_commit_source_cleanup_error` owner; `classify_route_os_error` refusing that origin as it refuses transport; and nothing else changed. Origin is decided by marker/boundary identity, never by message, errno or exception class. Generic unmarked route `OSError` still yields the typed filesystem `400`, transport-origin still abandons, `MemoryError` stays outside the `OSError` arm, and `C14` precedence over a cleanup cause is unchanged. | A second or route-local classifier; a message, errno or subclass heuristic; `classify_route_os_error` disabled, bypassed or route-scoped; filesystem classification broadly removed; the cleanup mark applied anywhere but the cleanup owner or inferred from the exception chain; any `codefire` unit changed outside the named taxonomy symbols and their two call sites; or any product change beyond this bridge. |
| `INV-C18-SCOPE` | The bridge closes only the `C18` composition incompatibility, and only on a fresh exact `ACCEPT` of `C` in `R`. | Any artifact using it to close `B2` before review, or to close `B3`, `C14-LIVE-A1`, `C14-LIVE-A2`, `C14-SUCCESSOR-OWNER`, constructor-unwind or global `C14`, MR-write, a root issue, the cycle, Install or Dogfood, or calling `C18` closed repository-wide. |
| `INV-DOC-STAGES` | The four canonical source docs have exactly three stages. Stage one, the pre-citation three-way union reconstructed from `cb13da0`/`b164046`/`0f43415`, is `d8d7aa9881b5f96bcfa13b78168403be9ae62512`, `0a8d2a886c883a051fe8c0e74bf550da3821b977`, `9a3afd7dd932bd3db7f33909e3cda40fb867abfc`, `2b68347eaa357f187f794c2a76992401743929eb`. Stage two, after the `O1` citation, is `5a1755ebd7e0eff8f55f92ae940557badb3ca567`, `72953356fca0982596bbb81a939fd4e2f2584a75`, `9a3afd7dd932bd3db7f33909e3cda40fb867abfc`, `2b68347eaa357f187f794c2a76992401743929eb`, in the path order architecture / trace / playbook / catalog; the first two differ from stage one **only** by the required relative `O1` citation. Stage three, the blobs committed in `C`, differs from stage two on architecture, trace and playbook **only** by the frozen `C18` bridge truth, and the writer publishes the three hashes it measured; the catalog is unchanged across all three stages and is still `2b68347eaa357f187f794c2a76992401743929eb` in `C`. | Any stage-one reconstruction that misses its union blob; a stage-one → stage-two delta on paths 7 or 8 beyond the `O1` citation; any stage-two change to paths 9 or 10; a stage-two → stage-three delta beyond the `C18` bridge truth; any change to path 10 at any stage; or any artifact requiring the committed files to equal the stage-one or stage-two values on paths 7, 8 or 9. |
| `INV-DUP` | The initial-Upload patch appears exactly once in `codefire`, and `tests/test_python_http_upload_accounted_traversal.py` has **37** test methods in `C` — the 36 accepted methods, unchanged, plus exactly one new `C18` oracle; not 10, not 36 and not 46. | A doubled hunk; a method count other than 37 in `C` or other than 36 before the bridge; or any edit to one of the 36 accepted methods. |
| `INV-B3` | The `B3` surface is byte-identical to its accepted source: `_PreparedHTTPMutationResponses`, the handler `POST` arm and the request-loop `finally` are unchanged and only read by tests. | Any diff on that surface, or any text claiming a post-effect phase index. |
| `INV-INVERSION` | `grep -n 'raise cleanup_error from run_error' codefire` returns exactly one occurrence, inside `run_server_verification`, and zero inside `enforce_remote_commit_signature_policy_accounted`. | Two occurrences means the Upload delta was lost; zero means a routed residual was silently repaired. |
| `INV-OWNERS` | Every routed locus has exactly one owner: reachable pre-effect loci including `AccountedServerVerification`, `run_server_verification` and `validate_sealed_commit`'s constructor unwind belong to `C14-LIVE-A1`; `B3` post-effect, response and wire loci including the handler `POST` arm and the request-loop `finally` belong to `C14-LIVE-A2`; `C14-SUCCESSOR-OWNER` holds only production-unwired or deferred global loci owned by neither. | Any locus appearing under two handles, or any of those four loci listed under `C14-SUCCESSOR-OWNER`. |
| `INV-REGISTER-ONLY` | `C` registers the three handles OPEN and implements none of them. | Any behavioural change on a routed residual, or any text claiming one is narrowed, repaired or closed. |
| `INV-GOVERNANCE` | The AIA-046 Cargo/O1 governance surface and the five canonical-newer management docs are byte-identical to `L`. | Any diff on those paths across `L..R`. |
| `INV-NOCLOSE` | No writer or reviewer artifact states `Close`, `Closed`, `complete`, `zero-residual` or `ACCEPT` about CFR-168, CFB-136, CFB-138, global `C14`, MR-write or the cycle; root row 6 stays `Implement in progress`. | Any such phrasing, or a changed root row. |
| `INV-AUTHORITY` | The original shared-primitives preflight and the published composition preflight are unmodified, and the shared-primitives verdict is still described as `BLOCK`. | A diff on either path, or any text calling that verdict `READY`. |
| `INV-STATUS` | This packet's authoritative status is `ready_for_bounded_composition`; the typed companion's `status: "ready"` and its `base_revision` of `a972cde` are schema encodings of exactly that and of the immutable product base, and authorize nothing broader. | Any artifact treating the packet as authorizing MR-write, integration or closure, or treating `base_revision` as the execution base. |

## Validation

Nothing may be reported as `PASS` unless it was executed; anything not run is
recorded as **not run**, never as `PASS`. **Do not run any Cargo/Rust command.**
`L` is the manager-named launch base; `C`, `W` and `R` are the three commits.

### Topology and write-set exactness

| ID | Command | Expected |
|---|---|---|
| `T1` | `git rev-list --count L..C` and `git rev-list --merges L..C \| wc -l` | `1` and `0` |
| `T2` | `git diff --name-status L..C` | exactly the 18 candidate paths, aggregate 1-18 |
| `T3` | `git rev-list --count C..W`, `git rev-list --merges C..W \| wc -l`, `git diff --name-status C..W` | `1`, `0`, and exactly one `ADD` of the aggregate path 19 result report |
| `T4` | `git diff C..W -- <the 18 candidate paths>` | empty — `W` changes no candidate byte |
| `T5` | `git rev-list --count W..R`, `git diff --name-status W..R` | `1` and exactly one `ADD` of the single review path |
| `T6` | `git rev-list --count L..R` and `git rev-list --merges L..R \| wc -l` | `3` and `0` |
| `T7` | `git grep -F <C's SHA> C -- <the 18 candidate paths>` and the same for `C`'s tree and patch digest | no match — `C` never claims its own identity |

### Identity and exactness of candidate `C`

| ID | Command | Expected |
|---|---|---|
| `I1` | `git rev-parse C C^{tree} C^` | recorded; `C^` equals the manager-named `L` |
| `I2` | `git merge-base --is-ancestor a972cde L`, `git rev-list --merges a972cde..L \| wc -l`, `git diff --name-only a972cde..L` | linear descent, `0` merges, docs-only paths and zero product/runtime/test/packaging paths |
| `I3` | `git hash-object codefire` at the pre-bridge checkpoint and again in `C` | pre-bridge exactly `f5e0b6db7c15aeadd3025ae4f038e301a7634caf`; in `C` a different, writer-published value whose delta from it is exactly the frozen `C18` bridge |
| `I4` | `git ls-tree C -- codefire` | mode `100755` preserved through the three-way merge |
| `I5-UNION` | Reconstruct stage one independently — for each of the four canonical source docs, `git merge-file -p --diff3 <b164046 copy> <cb13da0 copy> <0f43415 copy>` into a scratch file, then `git hash-object` it | exit 0 and zero conflict markers on each, and exactly `d8d7aa9881b5f96bcfa13b78168403be9ae62512` (architecture), `0a8d2a886c883a051fe8c0e74bf550da3821b977` (trace), `9a3afd7dd932bd3db7f33909e3cda40fb867abfc` (playbook), `2b68347eaa357f187f794c2a76992401743929eb` (catalog). These are **intermediate** values and are *not* expected to match the committed files for architecture and trace |
| `I5-CITED` | `git hash-object` on the four canonical source docs after the `O1` citation and before the step-8 bridge, plus `diff` against each `I5-UNION` scratch file | exactly `5a1755ebd7e0eff8f55f92ae940557badb3ca567` (architecture), `72953356fca0982596bbb81a939fd4e2f2584a75` (trace), `9a3afd7dd932bd3db7f33909e3cda40fb867abfc` (playbook), `2b68347eaa357f187f794c2a76992401743929eb` (catalog). The playbook and catalog diffs are **empty**; the architecture and trace diffs contain **only** the required relative `O1` citation and the reflow it forces |
| `I5-FINAL` | `git hash-object` on the four canonical source docs as committed in `C`, plus `diff` against each `I5-CITED` value | catalog still exactly `2b68347eaa357f187f794c2a76992401743929eb` with an **empty** diff; architecture, trace and playbook at writer-published values whose diffs contain **only** the frozen `C18` bridge truth. No stage-one or stage-two value may be asserted as the committed value for those three paths |
| `I6` | `git hash-object tests/test_python_http_upload_accounted_traversal.py` at the pre-bridge checkpoint and again in `C`; `git hash-object tests/test_python_http_mr_list_decision_work.py` in `C` | pre-bridge exactly `829836bc0411bf82030286e6f1b0e3d58e574e14`, not the 10-test duplicate `141dcbd4ee1f…`; in `C` a writer-published value differing only by the one new `C18` oracle; the MR-list module byte-identical to its accepted-source blob |
| `I7` | `git show 0703f71 \| git patch-id --stable` and `git diff 5254e3d cb13da0 \| git patch-id --stable` | both `9889888cedeba7344fb987ff5891cf728d9c7e36` |
| `I8` | `git diff --binary L..C \| sha256sum` | recorded in `W`; the bound report digests `46e7c3e4…` and `756bf200…` reconfirmed |
| `I9` | AST proof on the **pre-bridge** merged `codefire`: units changed by ours vs theirs, overlap, drift, qualified-name collisions, module-level union and evaluation order | 0 overlap, 0 non-identical accepted units, 0 drift, 0 collisions, merged equals the union |
| `I9-BRIDGE` | AST diff of `C`'s `codefire` against the pre-bridge checkpoint | the changed-unit set is exactly the frozen `C18` bridge: the cleanup-origin marker and predicate, `os_error_origin`, `classify_route_os_error`, and `_drained_commit_source_cleanup_error`. Zero other units changed, zero renames, zero qualified-name collisions, and no unit deleted |
| `I10` | `grep -c '<<<<<<<\|>>>>>>>\|=======' codefire` and the four canonical source docs, at all three stages: the stage-one union reconstruction, the stage-two `I5-CITED` checkpoint, and the stage-three files committed in `C` | `0` everywhere, at every one of the three stages |
| `I11` | `git diff --check` and `git status --porcelain` at each of `C`, `W` and `R` | clean and empty at all three |

### Focused Python

| ID | Command | Expected |
|---|---|---|
| `V1` | `python3 -m py_compile codefire` and the five test modules | `PASS` |
| `V2` | `python3 -m unittest tests.test_python_http_upload_accounted_traversal -v` | **37/37** — the 36 accepted methods plus the new `C18` oracle. Both subtests of `test_request_loop_accounts_a_drained_cleanup_fault` must pass; on the pre-bridge tree they fail `0 != 1`, which is the measured defect this bridge exists to fix |
| `V3` | `python3 -m unittest tests.test_python_http_mr_list_decision_work -v` | **49/49**, including `test_dispatcher_publishes_every_route_filesystem_failure` — measured passing at `49/49` on the pre-bridge composed tree, so any regression here is bridge-introduced |
| `V4` | `python3 -m unittest tests.test_python_http_doctor_decision_work tests.test_python_http_branch_decision_work -v` | **32/32** (19 + 13) |
| `V5` | `python3 -m unittest tests.test_python_ancestry_owner tests.test_python_signature_owner tests.test_python_signature_terminal_freeze tests.test_python_sealed_production tests.test_python_external_production -v` | **59/59** adjacent, unchanged by composition |

### Broad Python

| ID | Command | Expected |
|---|---|---|
| `V6` | `python3 -m unittest discover -s tests -p 'test_python_*.py'` | **539/539** in 34 modules — 538 pre-bridge plus the one new oracle |
| `V7` | `python3 -m unittest tests.test_codefire_cli -v` | **228/228** — mandatory composition-only gate; `b164046`'s copy has never run against the merged `codefire`. A `-k upload` subset is a pre-check, never a substitute |
| `V8` | `python3 -m unittest discover -s tests -v` | **767/767** in 35 modules — 766 pre-bridge plus the one new oracle |
| `V9` | `./demo.sh` | run if applicable per `AGENTS.md`; record the exact outcome, and record honestly and explicitly if it is not applicable or not run |

### Checkers

| ID | Command | Expected |
|---|---|---|
| `D1` | `python3 docs/development/tools/check_source_docs.py` | `PASS` |
| `D2` | `python3 docs/development/tools/check_project_docs.py` | `PASS`; the Markdown inventory grows from `L` by exactly the Markdown paths added in the commit under test |
| `D3` | `python3 docs/development/tools/check_ai_contracts.py` | `PASS` |
| `D4` | `python3 docs/development/tools/check_ai_acceleration_docs.py` | `PASS` |
| `D5` | `python3 docs/development/tools/check_ai_development.py --timeout 180` | `PASS` 7/7. A larger timeout is permitted **only** with a recorded reason; the default `--timeout 60` is known to fail pre-existing on `tool_tests` timing and is not the writer's to fix |
| `D6` | `git diff L..C -- docs/development/ai/source-keyed-cargo-gates.md docs/development/tools/run_source_keyed_cargo_gate.py` | empty |
| `D7` | `grep -n 'shared-primitives\|C14\|B2'` on aggregate paths 7 and 8 | every mention resolves to the in-tree authority path |
| `D8` | `grep -n` for the origin taxonomy on aggregate paths 7, 8 and 9 | each of the three docs names all three origins — filesystem, transport, post-commit/commit-source cleanup — and none still asserts that only client transport failures abandon the connection, or that origin is decided by exception class |

### Non-vacuity and reversals

Named `X1`-`X13` so the family cannot be confused with the reviewer report commit
`R`. `X1`-`X9` is the family `806f6c8` published as `R1`-`R9`; none was dropped.
`X10`-`X13` are the `C18` bridge reversals, and every one of them must be
performed and reverted — a bridge that no reversal can break is not evidence.

| ID | Check | Expected |
|---|---|---|
| `X1` | The nine single-edit Upload reversals `r1`-`r6b` and `m2` named in the Upload `ACCEPT` | each fails at the boundary it names |
| `X2` | Upload production accounting observer probe — `make_http_handler` into `handle_one_request` `finally`, read through `_decision_work_observer`, real socket `POST` with one real `close(2)` fault | `current = {}`, `open_permit_count = 0`, `terminal_reason = 'complete'`, 0 response bytes, 0 durable bytes; the pre-repair triple was 11,456 / 0 / `None` |
| `X3` | `close(2)` call-count bound with 8 faults armed on both live routes | exactly 2 `close` calls on the faulting source, exactly 1 on every other |
| `X4` | `C14` precedence by exception object identity, not type or message | primary decision is the outer exception; the first cleanup error attaches as `__cause__` |
| `X5` | MR-list reversals from the `5c952a9a…` `ACCEPT` | each fails at its named boundary |
| `X6` | Branch deadline reversal — revert the corrected `test_codefire_cli.py` expectation to the pre-fix `200 OK` / zero-poll form | fails, proving the corrected expectation is load-bearing |
| `X7` | Composition-specific inversion census: `grep -n 'raise cleanup_error from run_error' codefire` | exactly 1, inside `run_server_verification` only; 0 in `enforce_remote_commit_signature_policy_accounted` |
| `X8` | `grep -c '_drained_commit_source_cleanup_error' codefire` | `5`; both live call sites carry the drain helper intact |
| `X9` | Three-run stability of the three new Upload live-route tests | 3 x 3/3, no port, PID or timing flake |
| `X10` | **Bridge non-vacuity — cleanup marking.** Remove the cleanup-origin mark from `_drained_commit_source_cleanup_error`, changing nothing else | `test_request_loop_accounts_a_drained_cleanup_fault` fails on **both** routes with `len(handler_errors)` `0 != 1` — the exact measured pre-bridge defect |
| `X11` | **Bridge non-vacuity — classifier refusal.** Restore `classify_route_os_error` to its accepted form so it absorbs the cleanup origin, leaving the mark in place | the same Upload production oracle fails on both routes; proves the refusal, not merely the mark, is load-bearing |
| `X12` | **Anti-overreach.** Make `classify_route_os_error` treat a generic **unmarked** `OSError` as cleanup origin | `test_dispatcher_publishes_every_route_filesystem_failure` fails in `tests.test_python_http_mr_list_decision_work`, including its patched `handle_http_upload` subtest; proves the bridge did not weaken filesystem classification |
| `X13` | **Marker, not message or type.** Replace the cleanup-origin marker check with an errno test, a message-substring test or an exception-subclass test | the new `C18` cross-input oracle fails, because its two `OSError`s share errno, message and filename and differ only by the production mark |

### Not run, not to be claimed

No Cargo/Rust (`cargo fmt`, `cargo clippy`, `cargo test`), no `install.sh`, no
dogfood, no Docker, no network. `check_swarm_admission.py`,
`check_review_terminalization.py` and `check_review_recurrence.py` require
mandatory arguments and gate lane admission rather than this composition's
content; do not run them blind.

## Handoff

### Candidate `C` — required outputs

1. One clean additive commit whose parent is the manager-named launch base `L`, containing exactly the 18 candidate paths, zero merges, clean tree, `codefire` mode `100755` preserved.
2. No self-identity reference anywhere inside those 18 paths.
3. `C14-LIVE-A1`, `C14-LIVE-A2` and `C14-SUCCESSOR-OWNER` registered OPEN in aggregate path 18 under the ownership map above, implemented nowhere.
4. The `O1` relative citation present in aggregate paths 7 and 8, with the stage-two checkpoint hashes architecture `5a1755ebd7e0eff8f55f92ae940557badb3ca567`, trace `72953356fca0982596bbb81a939fd4e2f2584a75`, playbook `9a3afd7dd932bd3db7f33909e3cda40fb867abfc`, catalog `2b68347eaa357f187f794c2a76992401743929eb` recorded before the bridge.
5. The frozen `C18` bridge present in aggregate paths 1, 6, 7, 8 and 9 and nowhere else, with `docs/development/source-data-model-catalog.md` still `2b68347eaa357f187f794c2a76992401743929eb` and `tests/test_python_http_mr_list_decision_work.py` still byte-identical to its accepted source.

### Writer result `W` — required outputs

1. One clean additive commit whose parent is `C`, containing exactly one `ADD` of `docs/development/ai/swarm-results/cfr168-python-http-accepted-input-composition-20260802/result.md`, zero merges, clean tree, and no candidate byte changed.
2. `W` records `C` exactly: SHA, tree, parent `L` and the chain from `L` back to `a972cde`; the 18-path `L..C` name-status; `git diff --binary L..C | sha256sum`; the six product blob ids and `codefire` mode; **all three canonical source-doc blob stages in full**, stated as three distinct sets — the four stage-one pre-citation union blobs from `I5-UNION`, the four stage-two post-citation blobs from `I5-CITED`, and the four stage-three blobs committed in `C` from `I5-FINAL` — with the delta between each pair identified: stage one to stage two changes architecture and trace by the `O1` citation only, stage two to stage three changes architecture, trace and playbook by the `C18` bridge truth only, and `docs/development/source-data-model-catalog.md` is unchanged across all three; the omitted duplicate patch-id `9889888cedeba7344fb987ff5891cf728d9c7e36`; the **pre-bridge and final** identities of `codefire` and of aggregate path 6, with the exact bridge delta and the `I9-BRIDGE` changed-unit set; test counts `539` and `767` plus every focused count; every `T`, `I`, `V`, `D` and `X` result including `X10`-`X13`; and every item not run marked explicitly not run.
3. Truthful scoping: `C` is composed-not-accepted; it closes nothing; `C` is accepted-input composition **plus one preflight-prescribed `C18` bridge**, not a byte-only union, and `W` must say so plainly and cite `C18`, §5.7 and the §9.2 conflict-map row; the bridge resolves only the `C18` composition incompatibility and only on a fresh exact `ACCEPT`; `B2` remains OPEN pending that `ACCEPT`; `B3` remains OPEN and routed to `C14-LIVE-A2`; MR-write remains blocked; the three handles are registered OPEN and unimplemented; root row 6 unchanged.
4. Evidence digests are required, not optional.

### Reviewer report `R` — report-only, exactly one path

`docs/development/ai/swarm-results/cfr168-python-composition-exact-review-claude-20260802/review.md`

One commit whose parent is `W`, adding exactly that one path and changing no
candidate or result byte. A fresh agent, session and worktree, with no inherited
implementer conversation, WIP or `/tmp` state, and its own scratch prefix. It must
not call the candidate's own test helpers when reproducing oracles, must not write
any of the 18 candidate paths or the result path, and must not create `C` or `W`.

**The review subject is `C`.** `R` binds `W` only as writer evidence, never as
part of the subject, and records both `C` and `W`.

The reviewer must independently recompute, not accept from any handoff: `L`, `C`
and `W` identities and the full parent chain back to `a972cde`; the topology
counts `T1`-`T7`; the six product blobs; **both canonical source-doc blob stages**
— independently re-reconstructing the four stage-one pre-citation union blobs from
`cb13da0`/`b164046`/`0f43415` per `I5-UNION`, then rechecking the four stage-two
blobs per `I5-CITED` and confirming the architecture and trace deltas carry only
the required relative `O1` citation, then rechecking the committed blobs per
`I5-FINAL` and confirming the stage-three deltas on architecture, trace and
playbook carry only the frozen `C18` bridge truth while the catalog is unchanged
across all three stages; **the `C18` bridge itself** — independently reproducing
the pre-bridge failure, re-deriving the `I9-BRIDGE` changed-unit set, and proving
no drift outside it; the
duplicate stable patch-id `9889888cedeba7344fb987ff5891cf728d9c7e36`; the binary
patch SHA-256s; the test counts `539` and `767` and every focused count; the
reversals `X1`-`X13`, performing `X10`-`X13` rather than inferring them; the
inversion census; the AST disjointness; the `B1`/`B2`/`B3`
state as adjudicated here; the single-owner ownership map; and the negative space
— zero product divergence between `16d0129` and `a972cde`, docs-only linear
descent from `a972cde` to `L`, the five canonical-newer management docs
byte-identical, the 18 already-identical docs unchanged, the Cargo/O1 governance
surface untouched, and no agent-state or worktree debris imported. It must confirm
the duplicate is omitted exactly once, must reproduce rather than infer `X2`,
`X3`, `X4` and `V7`, and must render an explicit scope verdict.

Any `ACCEPT` must state that it is scoped to the immutable candidate `C` only,
that the `C18` bridge it accepts resolves the `C18` **composition
incompatibility** and nothing wider, and that it does not close CFR-168,
CFB-136, CFB-138, all of Upload, global `C14`, `C18` repository-wide, `B3`,
MR-write, integration, Install, Dogfood or any predecessor-gated successor, and
must not move root row 6 off `Implement in progress`. Any `Close`, `Closed`,
`complete` or `zero-residual` phrasing applied to the root, the cycle or global
`C14` is grounds for rejection. Two consecutive required-correction rounds on the
same mechanism return that mechanism to Plan/Preflight.

### Dependencies and ordering

1. The manager integrates the complete Plan/FREEZE chain — `806f6c8`, the `a7e283e` topology correction, and this successor blob-stage correction — onto `integration/cfr168-p0-canonical`, and names the resulting `HEAD` as the launch base `L` in the launch prompt. `L` is that final integrated docs-only head, whatever its SHA turns out to be; no Plan/FREEZE file may hard-code it.
2. Launch is manual via second-agent on `L`. Generic `run_ai_role` is not used, because it would correctly reject the packet's intentionally pinned `base_revision`.
3. One composition writer produces `C`, then `W` on top of `C`.
4. One fresh exact review produces `R` on top of `W`.
5. On `ACCEPT` in `R`: `B2` closes for candidate `C`, and the manager may then consider integrating the linear chain `C..R` as a separate decision that this packet does not authorize.
6. `C14-LIVE-A1` and `C14-LIVE-A2` are undertaken as their own bounded subjects afterwards. `B3` and `C14-LIVE-A2` gate MR-write. MR-write stays blocked until the `ACCEPT` of `C` and the later required `C14` resolution both land.
