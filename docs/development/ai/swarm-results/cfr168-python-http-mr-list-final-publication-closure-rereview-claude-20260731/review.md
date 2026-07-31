# CFR-168 Python HTTP MR-List Final-Publication Closure Rereview

Role: independent reviewer (report only). Provider/model: Claude /
`claude-opus-5`. Fresh agent, session, and worktree; no implementer
conversation, WIP, conclusion, or mutable `/tmp` result was inherited.

Reviewer scratch prefix: `/tmp/cfr168rr-claude-20260731/` (never the worker's
paths). Durations are `time.perf_counter()` monotonic, not wall clock.

Review task binding:

| Field | Value |
|---|---|
| Manifest | `docs/development/ai/swarm-batches/cfr168-python-http-mr-list-final-publication-closure-rereview-claude-20260731.json` |
| Manifest SHA-256 | `fb4f0cca761fe24c35146730ca6881e60a674edd786ad9422ac00dbefddb9508` |
| Ticket SHA-256 | `9e0a84f27597c43014bad5a1ef01975c65f8a9babccef3c05cc5b99cc681a2fb` |
| Review base | `9c5c276e39a94c96c797543894a302ec14872692` |
| Declared write set | `docs/development/ai/swarm-results/cfr168-python-http-mr-list-final-publication-closure-rereview-claude-20260731/review.md` |

## Verdict

```
ACCEPT
```

Scoped to immutable candidate `b16404676639336f705feffd9af4857d4a1fcc69` only.
It does not authorize integration, Close, Install, Dogfood, accepted
Upload/MR-write bindings, shared-primitives preflight, Python composition, or
any successor protocol.

---

## 1. Recomputed immutable identity

The review checkout sits **above** the product candidate, so the canonical
helper was run against a **fresh, clean, detached scratch checkout** of the
exact candidate — not the implementer's worktree and not a branch tip:

```
git clone --no-hardlinks --no-local file:///workspace/project \
    /tmp/cfr168rr-claude-20260731/scratch
git -C .../scratch checkout --detach b16404676639336f705feffd9af4857d4a1fcc69
```

Helper: `docs/development/tools/capture_candidate_identity.py`, recomputed
SHA-256 `c4606a4b48af8f844a9b42b32e681cbc028df8b884fe75a66dcb6fafc16557b9`
— **matches** the ticket and card.

| Field | Card value | Reviewer-recomputed | Match |
|---|---|---|---|
| Base / merge base | `06e519e4a434c07324744018af80ecfa8366f809` | same / same | yes |
| Parent | `091dc2a562ffcb94557b974ac1828c80d09e7ccc` | same | yes |
| Head / checkout HEAD | `b16404676639336f705feffd9af4857d4a1fcc69` | same / same | yes |
| Tree | `36185e320338b67e1610d82fcc81cba310c36026` | same | yes |
| Additive commits / merges | `3` / `0` | `3` / `0` | yes |
| Binary full-index patch SHA-256 | `8391b582bd219fc0d7a3385cdb3e139839a442aeac47b80951270213bbe095c2` | same | yes |
| Branch | (detached scratch) | `null` (detached) | n/a |
| Clean | clean | `clean: true`, `porcelain: []` | yes |
| Helper decision | `READY_FOR_REVIEW` | `READY_FOR_REVIEW`, `blocking_reasons: []` | yes |

Ordered commits and trees:

| # | Commit | Tree | Parent |
|---|---|---|---|
| 1 | `e59b7ec85548dda6f6dbaed5d9b22b4bd88a1a60` | `ff0b3ca84df40d7c8a0589c3224881221679e791` | `06e519e4…` |
| 2 | `091dc2a562ffcb94557b974ac1828c80d09e7ccc` | `883fd09ae605278f03ef02b6730e9fbed234361d` | `e59b7ec8…` |
| 3 | `b16404676639336f705feffd9af4857d4a1fcc69` | `36185e320338b67e1610d82fcc81cba310c36026` | `091dc2a5…` |

Linear, additive, no merge, no amend/rebase observed.

Exact changed paths — every status, numstat, base blob, head blob, head byte
count, and head content SHA-256 was recomputed and **all six rows match the
card byte for byte**:

| Status | Path | +/− | Base blob | Head blob | Head bytes | Head SHA-256 |
|---|---|---|---|---|---:|---|
| `M` | `codefire` | 265/44 | `18c4cb85…` | `f30575a6…` | 1437333 | `a7515751bc5d0001aed3d168ab385213400500e19503e7f5f88da4fe6fc2335b` |
| `M` | `docs/development/change-playbook.md` | 27/4 | `8594752c…` | `a14c3964…` | 40458 | `8992b7c5f6be3adc2f43669620bd5e125945eaa9d355a4139ed1af0353656568` |
| `M` | `docs/development/command-source-trace.md` | 20/1 | `d315b5d9…` | `a7bd13ba…` | 43540 | `16036d3567d9e844febfdfe9b8814c3d503e973834f5b9205ecbe75f61f0b2c7` |
| `M` | `docs/development/source-architecture.md` | 27/7 | `88313748…` | `acc6c98c…` | 104185 | `2f59fe48bd0cba71875ffebcb239047a861661216f855aae8e606c45bb28b76b` |
| `M` | `docs/development/source-data-model-catalog.md` | 8/0 | `b3f61677…` | `e416eefe…` | 93157 | `2edd69763aa24c5f5308e047f85f29ff2a69fe4afac4688a053efb4fc4cf265d` |
| `M` | `tests/test_python_http_mr_list_decision_work.py` | 466/66 | `9aaa13ad…` | `0d49841a…` | 201866 | `98938b9bfdb1bc4b9add8150618737ee66c315c5b59f7d97ea8023e9edf83474` |

Provenance binding, independently recomputed:

| Evidence | Recomputed | Result |
|---|---|---|
| Prior report blob at `6ee36460…` | `48cb3cb98e5e1dde4a0b5cecd1bfdfaf1af61a45` | matches |
| Prior report SHA-256 | `dc4f66ba9649e4333980996c6f83d7f28b0f5f4bd165683adac55bf6d9704b2e` | matches; verdict `REQUEST_CHANGES` |
| Continuation packet blob | `fad31608b2c71872d816fd3f2537aa6f5fe032bc` | matches |
| Continuation packet SHA-256 | `fee19a3c98d7269c7a4226ddfd30a71cb798a2a75d3e1a54071b70a7c9c800de` | matches; byte-identical at candidate |
| `tests/test_codefire_cli.py` blob at `4930d251…` / `06e519e4…` / `b1640467…` / `0aa0d3d0…` / `9c5c276…` | `b23c43ced6ccb69c0bb553f9856c1a7b2145e770` at all five | identical |
| `install.sh` blob at the same five revisions | `841e1526cb6bdfb192ec1261cb9f5e6679f6fe4c` at all five | identical |

No identity mismatch. Identity is exact.

---

## 2. Production call graph

```
CodeFireHTTPHandler.handle_one_request
└─ do_GET                                            [codefire:16619]
   ├─ read_slot = _HTTPReadResponseSlot(self)        [state = empty]
   ├─ try:
   │  └─ dispatch_get(read_slot)                     [codefire:16510]
   │     └─ with route_filesystem_boundary():        [pinned CM #1, codefire:1499]
   │        └─ handle_http_request_list(...)         [codefire:13593]
   │           └─ with remote_project_gate(root, shared=True) as gate:
   │              │                                  [pinned CM #2, codefire:2535]
   │              │  project_fd = _open_directory_nofollow(root)
   │              │  locks_fd   = _open_directory_at(project_fd, "locks")
   │              │  with stable_remote_lock(..., directory_fd=locks_fd):
   │              │     lock fd + dup'd owned_directory_fd, flock(LOCK_SH|NB)
   │              └─ collect_http_merge_request_list_payload(...)   [13430]
   │                 ├─ operation.capture_authority()
   │                 ├─ operation.validate_records()
   │                 ├─ payload = operation.project()
   │                 ├─ response_slot.prepare(200, payload)   → ENCODE
   │                 │     handler.prepare_json → prepare_json_owned
   │                 │       reserve(response_body) → freeze body → freeze
   │                 │       header → finish_permit → _PreparedHTTPResponseOwner
   │                 │     [slot state = prepared]
   │                 ├─ operation.finalize_authority()        → FENCE 1
   │                 ├─ operation.finalize_sessions()         → FENCE 2
   │                 ├─ operation.poll(PROJECTION_PHASE)
   │                 ├─ gate.fence()                          → FENCE 3
   │                 │     prove_pinned_project_state("holding")
   │                 │       (last authority read / identity compare)
   │                 └─ _finish_merge_request_list_operation(...)  → CLEANUP
   │                       close_with_retry(operation)   (≤2 attempts)
   │                       operation.cleanup_diagnostic()
   │                       → primary note  |  slot.report_cleanup_recovery
   │              ── stable_remote_lock.__exit__:
   │                    flock(fd, LOCK_UN)      ← TEARDOWN 1 (fallible)
   │                    os.close(fd)            ← TEARDOWN 2 (fallible)
   │                    os.close(owned_dir_fd)  ← TEARDOWN 3 (fallible)
   │              ── remote_project_gate.__exit__ (gate.fenced ⇒ no exit fence):
   │                    os.close(locks_fd)      ← TEARDOWN 4 (fallible)
   │                    os.close(project_fd)    ← TEARDOWN 5 (fallible)
   │        ── route_filesystem_boundary.__exit__  (marks unmarked OSError)
   │  except ResourceBudgetExceeded → displace_prepared_response → 429/typed
   │  except RequestDeadlineExceeded → re-raise to outer arm
   │  except CFError               → displace_prepared_response → 400
   │  except OSError               → classify_route_os_error
   │                                 · transport-marked → re-raise (abandon)
   │                                 · filesystem       → displace → typed 400
   │  else:  if read_slot.is_prepared:
   │            prepared = read_slot.prove_and_take()   → PROOF, then TAKE
   │              require_publishable_state() [state → proved]
   │              take()                      [state → published]
   │            publish_prepared(prepared)    → PUBLISH (first response byte)
   │            return
   ├─ if read_slot.closing: close_connection; return
   ├─ send_json(status, payload)                (non-slot routes / error arms)
   ├─ except RequestDeadlineExceeded: read_slot.close(); terminal deadline arm
   └─ finally: read_slot.close()   (reclaims only an unpublishable owner)
```

Publication point: **`_HTTPReadResponseSlot.prove_and_take()` followed by
`_PreparedHTTPResponseOwner.publish(self.wfile)` inside `transport_boundary()`**
— exactly one per request.

---

## 3. Owner / effect / fence / state timeline

Recorded from the **live production dispatcher over a real socket** by reviewer
probe **P1** (`test_probe_records_the_exact_publication_order`), which observes
`fcntl.flock` and `os.close` and not only named methods:

```
encode
authority fence
session fence
project fence
  fallible os.close ×10          (inside gate.fence()'s own prove: rebound
                                  root/locks fds and the pinned path walk)
cleanup
  fallible flock(8)              LOCK_UN on the shared project-gate lock
  fallible os.close              lock fd
  fallible os.close              owned locks-directory fd
  fallible os.close              gate locks_fd
  fallible os.close              gate project_fd
publishable proof                ← handler-owned, after every teardown
take
publish
```

| Stage | Owners live | Ledger effect | Slot state | Fallible after? |
|---|---|---|---|---|
| project → encode | projection | reserve/charge response, acquire retained | `empty` → `prepared` | yes |
| authority fence | prepared owner + operation owners | none | `prepared` | yes |
| session fence | same | none | `prepared` | yes |
| project fence | same | none | `prepared` | yes — **last authority-dependent** |
| cleanup | prepared owner only after success | releases every operation owner | `prepared` | yes |
| teardown ×5 | prepared owner only | none | `prepared` | yes — 5 fallible ops |
| **publishable proof** | prepared owner only | none | `prepared` → `proved` | **no — last fallible** |
| take | transferred reference | none | `proved` → `published` | no |
| publish | frozen bytes → wire | release on `close()` after the write | `published` | n/a |

Slot state machine, as **production** code (not test evidence):

| From | Transition | To | Guard |
|---|---|---|---|
| `empty` | `prepare` with an owner | `prepared` | `prepare_count == 0` |
| `empty` | `prepare` returning `None` | `closing` | encoder declined |
| `prepared` | `require_publishable_state` | `proved` | headers + prepared owner only, no other dimension, no open permit |
| `prepared` | `dispose` | `disposed` | `prepared.close()` must succeed first |
| `prepared` | `take` | — | **refused**: `CFError(PREPARED_RESPONSE_UNPROVED_STATE_DIAGNOSTIC)`, owner retained |
| `proved` | `take` | `published` | only transition into publication |
| `published` | `take` / `prove_and_take` / `dispose` | `published` | returns `None`, no second publication |
| `disposed` | `take` / `prove_and_take` | `disposed` | returns `None` |
| `closing` | `prove_and_take` | `closing` | returns `None`; `do_GET` closes the connection |

---

## 4. Required independent falsification — findings by item

### F1 — Final authority fence, teardown, and publication proof — **CLOSED**

* The three fences are the last **authority-dependent** boundaries: after
  `gate.fence()` no authority path is read and no pinned identity is compared
  (`remote_project_gate` skips its exit fence when `gate.fenced`,
  `codefire:2628-2632`).
* Five fallible teardown operations follow the fence — one real
  `fcntl.flock(fd, LOCK_UN)` and four real `os.close` calls
  (`codefire:1034-1039`, `codefire:2633-2636`). Reviewer probe **P1** observed
  exactly those five between `cleanup` and the proof, on the live route.
* **Zero** fallible operations after the proof. Probe P1 recorded
  `order[proof+1:] == ["take", "publish"]`. Source read confirms: `take()` is
  three attribute operations, `publish_prepared` enters `transport_boundary()`
  and writes; nothing authority-dependent, callback-capable, or
  input-proportional intervenes.
* Only `prove_and_take` reaches `take`: `grep` over the whole product shows
  `_HTTPReadResponseSlot.take` has exactly **one** production caller
  (`codefire:15522`, inside `prove_and_take`). `mutation_slot.take()` at
  `codefire:16857` is the separate POST class.
* **Scratch mutation, moving the proof back inside the pinned context**: the
  candidate's own regression wraps
  `collect_http_merge_request_list_payload` and calls
  `require_publishable_state()` before the gate unwinds; the production
  ordering oracle then observes `POST_FENCE_TEARDOWN_OPERATIONS == 5` fallible
  operations **after** the proof and fails. Independently reproduced on the
  rejected predecessor `4930d251…`, where the real order is
  `… project fence … cleanup, publishable proof, fallible flock(8),
  fallible os.close ×4, take, publish` — the original five-call counterexample,
  verbatim (`logs/reversal-rejected.log`).

### F2 — Post-route filesystem failure and prepared-response displacement — **CLOSED**

Driven through the **live GET dispatcher** over a real socket:

| Injection (reviewer probe) | Boundary | Result |
|---|---|---|
| `OSError(EIO)` after the real `flock(LOCK_UN)` (candidate regression) | unlock | typed `400 filesystem authority is unusable: …/project-gate.lock: Input/output error`; no `200`, no `"merge_requests"`; `terminal_reason=complete`; zero debt |
| `OSError(EIO)` after post-cleanup `os.close` #1 (lock fd) — **P2** | descriptor close | typed `400`, no `200`, no projection, zero debt |
| … #2 (owned locks-directory fd) — **P2** | descriptor close | same |
| … #3 (gate `locks_fd`) — **P2** | descriptor close | same |
| … #4 (gate `project_fd`) — **P2** | descriptor close | same |

* The older success is disposed **before** another response is selected:
  `displace_prepared_response` is called at the head of every `do_GET` arm and
  returns `False` (publishing nothing) if disposal fails
  (`codefire:16594-16617`, `16627/16633/16649`).
* Origin is exact: `os_error_origin(error) == "filesystem"` for each injected
  fault; `route_filesystem_boundary` marks it, `classify_route_os_error`
  selects the arm.
* **Transport origin still abandons**: probe **P3** raised
  `ConnectionResetError` from the real write inside `prepared.publish` —
  wire `b""`, no typed 400, connection abandoned.
* **`MemoryError`**: probe **P11** raised it after encoding — wire `b""`,
  connection abandoned; not absorbed by the OSError arm.
* **Request deadline**: `except RequestDeadlineExceeded: raise` precedes the
  other arms and the outer terminal arm disposes the slot before deciding
  (`codefire:16671-16706`); the shipped
  `test_deadline_expires_deterministically_at_every_boundary` remains green.
* **Budget and `CFError` arms falsified independently** (probes **P4**, **P5**),
  not merely as a consequence of the unlock case: budget → no `200`,
  `terminal_reason=budget_exceeded`, zero debt; `CFError` → `400` carrying the
  injected message, no projection, zero debt.
* GET/POST origin taxonomy is shared (`classify_route_os_error`,
  `mark_transport_origin` / `mark_filesystem_origin`); no second route-local
  ownership protocol was introduced — the read slot and the pre-existing
  mutation slot are the same two classes as at the base. See §7 note 3 for the
  pre-existing (unchanged) POST asymmetry.

### F3 — Publishability proof and zero debt — **CLOSED**

Three separate production-reachable scratch probes, each injected at a live
route call site (`finalize_sessions`) and driven over a real socket:

| Residual left at the proof point | Wire | Terminal reason | Residual | Permits | Typed diagnostic |
|---|---|---|---|---|---|
| retained 4096 (**probe P-wire-1**) | `b""` — **nothing at all** | `cleanup_incomplete` | `retained_bytes: 4096`, `temporary_bytes: 0` | 0 | `prepared read response cannot publish with unaccounted current work` in the handler error log |
| temporary 2048 (**probe P7**) | `b""` | `cleanup_incomplete` | `temporary_bytes: 2048`, `retained_bytes: 0` | 0 | same |
| one open permit (**probe P6**) | typed `400 Bad Request` — **no success byte** | `None` (see below) | all dimensions `0` | 1 (the injected one) | same |

* The prepared owner is disposed **exactly once** in every case: every
  releasable dimension returns to zero apart from the deliberately injected
  residual.
* The proof is the **only** transition into `proved`, and `take()` refuses a
  slot that is not `proved` — so a silenced proof cannot publish. The
  candidate's non-vacuity mutation (silencing `require_publishable_state`
  *and* forcing `state = PROVED`) restores the rejected outcome: `200 OK`
  with live debt. This is not a test-only flag: the refusal lives in
  `take()` on the production path and is exercised directly by reviewer probe
  **P12**, which shows the refused `take` leaves the owner in the slot.
* Open-permit arm, stated honestly: `DecisionWorkLedger.terminate` raises while
  any permit is open (`codefire:896-898`), so `cleanup_incomplete` is
  structurally unreachable in that state and `reject_unpublishable_response`
  correctly guards on `has_open_permits()`. The binding safety properties still
  hold — no success byte, exact disposal, every releasable owner zeroed, typed
  diagnostic logged, connection closed. I could not construct that state
  without direct injection: the operation holds no permit at the proof point on
  any reachable path (a failed session close becomes `cleanup_error` and never
  reaches the proof).

### 4. Production response-slot state machine — **CLOSED**

Reviewer probe **P12** exercises every transition directly against the
production class:

* `empty` → `take()` yields `None`; nothing publishes.
* `prepared` → `take()` raises
  `CFError("prepared read response was not proved publishable")` and **does
  not** steal the owner (`slot.is_prepared` still `True`).
* `prepared` → `dispose()` → `disposed`; `take()` and `prove_and_take()`
  afterwards both return `None` — no publication after disposal.
* `prepared` → `prove_and_take()` → `published`; a second `take()` /
  `prove_and_take()` returns `None`, and a following `dispose()` leaves the
  state `published` and releases nothing twice — no double publication.
* A second `prepare` on one request raises `CFError("read response is already
  prepared")`.
* A closing encoder (`prepare_json` → `None`) yields state `closing`,
  `prove_and_take()` returns `None`, and `do_GET` closes the connection with no
  response.
* Proof failure cannot be caught by an outer arm that revives an old `200`:
  `prove_and_take` runs inside the `else:` clause under its own dedicated
  `except CFError` → `reject_unpublishable_response`, which disposes first and
  returns; the enclosing `except CFError`/`except OSError` arms are attached to
  the `try` around `dispatch_get` and cannot see it.
* **Destructors are not required for correctness**: reviewer probe **P13**
  neutered `_HTTPReadResponseSlot.__del__` and
  `_PreparedHTTPResponseOwner.__del__` and drove both a clean request (full
  8-record projection, zero debt) and a displaced request (typed `400`, no
  projection, zero debt).

### 5. Disposal failure precedence — **CLOSED**

Reviewer probe **P8** fails `_PreparedHTTPResponseOwner.close` permanently
while each primary replaces the response:

| Primary being selected | Wire | Log | Ledger |
|---|---|---|---|
| `CFError` | `b""` | `prepared response disposal failed while replacing it: RuntimeError(…)` | residual visible, `terminal_reason=cleanup_incomplete` |
| filesystem `OSError(EIO)` | `b""` | same | same |
| `ResourceBudgetExceeded` | `b""` | same | same |

* The primary diagnostic stays primary — the disposal fault is attached with
  `record_secondary_failure` (`add_note` + `codefire_cleanup_failures`), never
  substituted.
* Ownership is not zeroed before release succeeds:
  `_PreparedHTTPResponseOwner.close` keeps `header_retained`/`body_retained`
  when the ledger release fails, and `dispose()` clears `self.prepared` only
  after `prepared.close()` returns (`codefire:15541-15551`).
* The connection closes and **neither** the old success **nor** a competing
  replacement publishes.
* **First-attempt recovery, exercised separately** (probe **P9**): the route's
  own `response_slot.dispose()` fails once, the fault is recorded on the
  primary, `do_GET`'s `displace_prepared_response` retries successfully, and
  the request publishes the primary `400` with `terminal_reason=complete`,
  `{retained_bytes: 0, temporary_bytes: 0}`, zero permits. Exactly one
  response, no swallowed release.

### 6. Recovered cleanup diagnostics — **CLOSED**

* `cleanup_diagnostic()` is consumed on the production path — as a
  `record_secondary_failure` note on an already-selected primary, otherwise via
  `slot.report_cleanup_recovery` → `handler.log_error("recovered cleanup fault:
  %s", …)` (`codefire:13572-13580`, `15473-15484`).
* Verified with a real injected release fault: `attempts == 2`,
  `recovered == True`, `fault_count == 1`, the fault text carries the injected
  message and owner, and the handler log line is present.
* Bounded: `CLEANUP_FAULT_REPORT_LIMIT = 4` truncates the `faults` list while
  `fault_count` stays exact.
* **Silence for a clean request**: a following clean exchange produced
  `recorded == []` and `logs == []`.
* The two dispositions the prior reviewer left open are closed by this:
  `cleanup_recoveries` is now read (`cleanup_diagnostic()["recovered"]`), and
  the write-only `cleanup_faults` property is **removed** — `grep` over the
  whole repository shows no remaining reader of the removed name.

### 7. Exact over-release and latent double release — **CLOSED**

* `_release_exact` now fails closed: an amount larger than the owner returns
  `CFError("decision-work owner over-release: <attr> owns <n>, release
  requested <m>")`, appends it to `_cleanup_faults`, and changes **nothing** —
  verified: owner stays `100`, ledger `retained_bytes` stays `100`, and the
  exact `100` retry then succeeds to zero.
* **Latent double release, independently recreated** (reviewer probe **P10**):
  two live session keys (40 and 60 bytes) plus 7 bytes of aggregate ownership
  with no index entry; the 40-byte per-session release is failed once.
  * Candidate: `close()` returns the fault, key `/a/objects` keeps exactly
    `40`, key `/b/objects` is removed, `_sessions_retained == 40`, ledger
    `retained_bytes == 40`; the retry reaches `0/0` with an empty index.
    Every live session key and the residual aggregate are released **exactly
    once**.
  * Rejected `4930d251…`: the same probe observes `_sessions_retained == 0`
    after the *failed* per-session release — the aggregate
    `attempt("retained_bytes", "_sessions_retained")` released ownership the
    per-key release still held, so a retry would have double-released. This is
    the exact latent defect the typed over-release exposed.
* The new second loop is genuinely reachable, not dead code:
  `session_for` acquires and indexes the entry bytes **before** constructing
  `SealedCommitValidationSession`, so a construction fault leaves an index
  entry with no session (`codefire:12580-12596`).

### 8. Preserve all inherited closures — **PRESERVED**

No test was removed: the focused module goes 44 → 49 named tests; the five new
names are additions and
`test_the_terminal_fence_immediately_precedes_the_first_response_byte` was
strengthened in place (the old body asserted only the named-method landmark
order; the new one additionally observes `flock`/`os.close`, pins
`POST_FENCE_TEARDOWN_OPERATIONS`, and carries the in-test regression).

| Prior closure | Re-falsified by | State |
|---|---|---|
| 1. parent-directory authority vs leaf absence | `test_unavailable_parent_authority_is_never_leaf_absence`, `test_absent_authority_fence_reproves_its_parent_identity_and_state` | green ×3 |
| 2. filesystem vs transport origin across GET/POST | `test_error_origin_is_classified_at_the_boundary_that_raised_it`, `test_dispatcher_publishes_every_route_filesystem_failure`, `test_every_shared_read_and_post_route_fails_closed_on_a_symlink`; reviewer P2/P3/P11 | green |
| 3. retryable cleanup without local ownership erasure | `test_cleanup_retries_every_owner_release_without_losing_ownership`, `test_primary_diagnostic_survives_a_cleanup_failure`, `test_unrecoverable_cleanup_never_publishes_hidden_current_work`; reviewer P10 | green |
| 4. terminal mode/link/size/identity/content metadata | `test_terminal_metadata_is_reproved_through_the_read_and_digest` | green |
| 5. authority mutation around encoding/fences/publication | `test_reachable_record_mutation_fails_closed_at_the_final_fence`, `test_authority_fence_rejects_same_metadata_replacement`, `test_authority_fence_rejects_an_appearing_absent_authority`, `test_pinned_mr_and_branch_authority_mutation_has_no_effect`, plus the fence test's late-mutation arm (mutation at the proof point cannot reach the frozen projection) | green |
| 6. route-wide descriptor protocol and scoped byte oracle | `test_descriptor_read_protocol_is_exact_and_route_wide` (5 reads, named owners), `test_stable_project_id_reads_the_descriptor_once_from_charged_bytes` | green |
| 7. per-poll bounded-work necessity and 1/8/24/80/200 curves | `test_every_production_poll_site_is_individually_required`, `test_removing_a_production_poll_breaks_its_phase_cancellation`, `test_poll_density_is_affine_and_free_of_per_request_escrow_polls`, `test_memoized_escrow_work_is_constant_in_merge_request_count` (`SHARED_HEAD_CARDINALITIES = (1, 8, 24, 80, 200)`), `test_removing_the_escrow_memo_restores_the_linear_slope` | green |

Also preserved and green: multi-fault filename precedence
(`test_multi_fault_error_precedence_matches_the_pre_fix_route` +
`test_eager_capture_faults_break_multi_fault_precedence`), leaf-symlink typed
responses, operation-local escrow memoization, one sealed session
(`test_shared_dag_validates_once_per_object_root_session`), exact handler
ledger/deadline (`test_route_threads_the_exact_request_ledger_and_deadline`,
`test_deadline_expires_deterministically_at_every_boundary`),
reserve-before-allocation (`test_exact_minus_one_and_plus_one_limits_bound_every_stage`
plus the affected CLI subset), status/byte parity
(`test_open_and_stale_status_parity`,
`test_cold_and_reused_route_publish_identical_bytes_and_work`), route isolation
(`test_neighbouring_read_routes_keep_their_own_phases`,
`test_successive_requests_on_one_server_stay_isolated`,
`test_eight_request_sequence_on_one_server_stays_isolated`,
`test_http_1_0_never_serves_a_pipelined_second_request`), affine counters,
stable-link compatibility
(`test_stable_pre_existing_links_stay_compatible_authority`), and the
cardinality / transport-harness flake repairs
(`test_response_phase_flag_is_exact_and_reset_for_every_request`).

**Six-test reversal on the exact rejected candidate `4930d251…`** — 6/6
non-green (5 failures + 1 missing-symbol error), each for its named reason:

| Test | Outcome on `4930d251…` | Reason |
|---|---|---|
| `…terminal_fence_immediately_precedes_the_first_response_byte` | FAIL | `0 != 5` teardown ops in the cleanup→proof window; the recorded order shows `publishable proof` followed by `flock(8)` and four `os.close` — the original F1 counterexample |
| `…post_teardown_filesystem_failure_replaces_the_prepared_success` | FAIL | `b'HTTP/1.0 200 OK'` published with the full projection despite the injected unlock `EIO` — the original F2 |
| `…unpublishable_prepared_response_never_reaches_the_wire` | FAIL | `200 OK` published with live debt — the original F3 |
| `…prepared_disposal_failure_publishes_nothing` | FAIL | a `400 Bad Request` published after a failed disposal instead of `b""` |
| `…owner_over_release_fails_closed` | FAIL | `_release_exact` returned `None` (silent) instead of a typed `CFError` |
| `…recovered_cleanup_faults_reach_a_production_diagnostic` | ERROR | `AttributeError: type object '_HTTPReadResponseSlot' has no attribute 'report_cleanup_recovery'` |

**Same six on the legacy `38da8988…`** — 6/6 non-green, honestly split into
**2 failures and 4 absent-symbol errors** (`_HTTPReadResponseSlot`,
`os_error_origin`, `_release_exact`, `report_cleanup_recovery` are absent
there). A missing symbol is an *error*, not proof of a defect; the two genuine
failures (`…unpublishable_prepared_response…`,
`…prepared_disposal_failure…`) fail there through the legacy no-slot path for
the same class of defect. Neither the legacy pass nor a missing symbol is taken
as evidence that the successor is correct — that comes from the candidate-side
probes above.

**Reviewer-owned probes are non-vacuous too.** Re-running my own 15 probe
methods on `4930d251…` (`logs/probes-reversal-rejected.log`): the four
descriptor-close boundaries, the open-permit and temporary-byte proof probes,
both zero-wire residual probes, all three disposal-precedence arms, the
per-session double-release probe, and the state-machine probe are **non-green**
there. Recorded honestly: `test_probe_records_the_exact_publication_order` (a
recorder, not an oracle), the transport/`MemoryError`/destructor probes
(preserved behaviour, correctly unchanged), and the budget/`CFError`
displacement probes **pass** on `4930d251…` — because those faults are raised
*inside* `collect_…`'s own `try`, which already disposed the slot at that
revision. The prior F2 was about faults raised **outside** that `try`, and
those probes do fail there.

### 9. Same-root residual and composition audit — **NO NEW RESIDUAL**

| Residual class searched | Result |
|---|---|
| Proof bypass / second owner-transfer path | None. `take()` has exactly one production caller (`prove_and_take`); no code outside the class touches `slot.prepared`. |
| Swallowed post-route error | None on the read path. `collect_…` records the primary and re-raises; disposal faults are logged **and** attached to the primary; the `finally` swallow only converts a residual-owner reclaim failure into `close_connection`, reachable only when nothing was published. |
| Incomplete disposal on an outer exception arm | None. Every `do_GET` arm — budget, `CFError`, `OSError`, deadline, `finally` — disposes before selecting, and refuses to select when disposal fails. |
| Diagnostic written but never consumed | The recovered-cleanup diagnostic **is** consumed (log line / primary note). `slot.cleanup_diagnostics` and the `state_proved` compatibility property are redundant retention, not unread evidence — see note 1. |
| Ownership erased before a successful release | None. `_release`, `_release_exact`, `dispose`, and `_PreparedHTTPResponseOwner.close` all keep exact counts until the release succeeds. |
| Unbounded diagnostic work | Bounded by `CLEANUP_FAULT_REPORT_LIMIT = 4`; see note 2 for the one residual sharp edge. |
| Response allocation before reservation | Unchanged: `prepare_json_owned` reserves `response_body` before `freeze_response_body`; the affected CLI subset re-proves it. |
| Route depending on destructor cleanup | None — probe **P13** with both `__del__` methods neutered. |
| Divergence Upload / MR-write would inherit | The GET protocol is strictly stronger than the unchanged POST one; see note 3. `do_POST` is **byte-identical** at base and candidate. |

---

## 5. Mutation and reversal evidence

| Mutation | Where applied | Observed |
|---|---|---|
| Restore the proof inside the pinned gate (candidate regression) | wrapper around `collect_http_merge_request_list_payload` | oracle observes 5 fallible teardown ops after the proof — fails for the original reason |
| Real `flock(LOCK_UN)` → `EIO` (candidate regression) | production unlock site | typed `400`, prepared `200` disposed, zero debt |
| Real `os.close` → `EIO` at each of the four post-cleanup descriptor boundaries (**reviewer P2**) | production teardown | typed `400` ×4, no `200`, zero debt |
| Silence the proof only, force `state = PROVED` (candidate regression) | `require_publishable_state` + `take` | rejected `200`-with-debt returns — proof is the cause |
| Refused-`take` and disposed/published re-transfer (**reviewer P12**) | production class | typed refusal, owner retained, no second publication |
| `ConnectionResetError` at the real response write (**reviewer P3**) | `prepared.publish` destination | wire `b""`, connection abandoned |
| `MemoryError` after encoding (**reviewer P11**) | live route | wire `b""`, connection abandoned |
| Permanent `_PreparedHTTPResponseOwner.close` failure under 3 primaries (**reviewer P8**) | live route | wire `b""` ×3, secondary recorded, `cleanup_incomplete` |
| Single-shot disposal failure then recovery (**reviewer P9**) | live route | primary `400`, zero debt, `terminal_reason=complete` |
| Injected retained 4096 / temporary 2048 / open permit at the proof point (**reviewer P6/P7/P-wire**) | `finalize_sessions` on the live route | `b""` / `b""` / typed `400` — never a success byte |
| Per-session release failure with a live aggregate (**reviewer P10**) | `DecisionWorkLedger.release` | candidate keeps exactly `40`; `4930d251…` drops to `0` — latent double release reproduced |
| Both `__del__` methods neutered (**reviewer P13**) | production classes | clean and displaced requests both correct |
| Silent `_release_exact` restored (candidate regression) | patched method | the over-release is masked again |
| Rejected-candidate and legacy reversal | `4930d251…`, `38da8988…` | 5F+1E and 2F+4E respectively |

---

## 6. Validation

Toolchain: `python3` 3.10.12 (main, Mar 3 2026, GCC 11.4.0), `git` 2.34.1,
Linux 6.6.87.2-microsoft-standard-WSL2 x86_64. `cargo` **absent**, `rustc`
**absent** (`which` exit 1).

Process deviation, recorded as required: `docs/development/tools/run_timed_gate.py`
is absent from the candidate. No repository file was added or changed to
compensate. Every gate below was wrapped by a reviewer-local auditable monotonic
wrapper at `/tmp/cfr168rr-claude-20260731/timed.py`, SHA-256
`22189902e6d6a721aa7079ef26f496ff45fd93b15240c2e2380dd153686b55b5`, which
records the exact argv, `time.perf_counter()` duration, exit status, log path,
and log SHA-256.

| # | Gate | Exact command (cwd) | Count | Monotonic s | Exit | Log SHA-256 |
|---|---|---|---:|---:|---:|---|
| 1 | identity / ancestry / tree / patch / paths / blobs / clean | `python3 capture_candidate_identity.py --repo …/scratch --base 06e519e4… --head b1640467… --format json` (scratch) | 6 paths, 3 commits | 0.622 | 0 | `37493356f0ba550a3ead14ab04926afdd8ba1dfdd8d24e3dd6fb1adf9a615b99` |
| 2 | `py_compile` | `python3 -m py_compile codefire tests/test_python_http_mr_list_decision_work.py tests/test_codefire_cli.py` (scratch) | 3 files, `COMPILE_OK` | 0.680 | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` (empty) |
| 3 | focused MR-list | `python3 -m unittest tests.test_python_http_mr_list_decision_work -v` (scratch) | **49/49 OK** | 103.755 (suite 103.607) | 0 | `f72c54eeaae56e82512f3a8566ebd12184ac6eb27328b789dbd8a4ca63601ede` |
| 4 | six new regressions on `4930d251…` | `python3 -m unittest -v <6 exact IDs>` (rejected worktree, candidate test blob `98938b9b…`) | 6: **5 F, 1 E** | 5.235 | 1 | `23d809a8b82d71cd2f78c4bf98f36841ffd05ffb3318f9cc7a436ae9c29cdc2f` |
| 5 | six new regressions on `38da8988…` | same 6 IDs (legacy worktree) | 6: **2 F, 4 E** | 4.800 | 1 | `692fa881b48aa372d7f589b02ff15f5e39c742c870c7db6ccebc29574b82dcdb` |
| 6 | reviewer probes (candidate) | `python3 -m unittest tests.test_reviewer_probes -v` (probe worktree) | **111/111 OK** (49 inherited + 62) | 324.049 | 0 | `bb3fc3a130add2f400f569027ba7406db25c3685fb41f00f5beff44b67ba10e9` |
| 7 | reviewer wire probes (candidate) | `python3 -m unittest -v tests.test_reviewer_probes_wire.ReviewerWireProbes` (probe worktree) | **51/51 OK** | 90.254 | 0 | `9a4afd1112072812d1ccc815ecb586c2ea544e1016586a5c7419c32ee643b9ac` |
| 8 | reviewer probes on `4930d251…` | same two probe modules (rejected worktree) | 113: **22 F, 3 E** | 211.024 | 1 | `be3e16a9009aefa2414a87066a92c084d36d4217ec86ad541e3e086e41802f98` |
| 9 | affected `tests.test_codefire_cli` | `python3 -m unittest -v <7 exact IDs>` (scratch) | **7/7 OK** | 5.779 | 0 | `810a5ac087744e927e40c5ffbbec5b75a032ea35af60f973d2012d25eb959ef2` |
| 10 | whole `tests.test_codefire_cli` (candidate) | `python3 -m unittest tests.test_codefire_cli -v` (scratch) | 228: **226 pass, 2 fail** | 1015.183 (suite 1014.997) | 1 | `3d813cce87ced8debc0bb20e765311c9285d8eafddfe892f694ef41ab87317bd` |
| 11 | two CLI IDs at continuation base `06e519e4…` | `python3 -m unittest -v <2 exact IDs>` (contbase worktree) | 2: **2 fail** | 1.253 | 1 | `428eb291f26eacbead8d397304abd356a9b512f1902c5e455c92c03381c9c945` |
| 12 | two CLI IDs at review base `9c5c276…` | same 2 IDs (review worktree) | 2: **2 fail** | 1.221 | 1 | `6ab1f620c891167ee80fecceebe7089de5001f72f63f9547049245e0a429dd57` |
| 13 | broad Python-pattern suite | `python3 -m unittest discover -s tests -p 'test_python_*.py'` (scratch) | **512/512 OK** | 342.801 (suite 341.910) | 0 | `4de3116a5413a2c78e10dc2c5907b5be4c93e498d374c26dc62d3035c2008efd` |
| 14 | `check_source_docs.py` | `python3 docs/development/tools/check_source_docs.py` (scratch) | 44 modules, 34 commands, 12 object kinds — passed | 0.221 | 0 | `a3679b77701e02f0697f91c6c53a362f7ab9d372ed700d042c1a73ec329cdb03` |
| 15 | `check_project_docs.py` | `python3 docs/development/tools/check_project_docs.py` (scratch) | 546 Markdown files — passed | 0.921 | 0 | `7843883f742c945ede42bdc5966d7c27cf74bc07f02df2e639deb648e5f65f5f` |
| 16 | patch hygiene | `git diff --check 06e519e4… b1640467…` (scratch) | no output | 0.125 | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` (empty) |

Reviewer probe sources (auditable, created outside the review worktree):
`tests/test_reviewer_probes.py` SHA-256
`be5022a8908c30de2ea92453f232a7a2e20d2997caf3698504a1d7e427190d4e`;
`tests/test_reviewer_probes_wire.py` SHA-256
`c49a1015de5d41f62fc00d394bda52eb02190dd3e4c8f2c69d52fd593705dc8e`.

Stability: the 49 focused tests ran green **three times** in this session
(gates 3, 6, 7 — the probe modules re-execute the inherited class). No
intermittent failure was observed.

### Affected legacy CLI scope

I derived the affected set independently by symbol reference rather than
adopting the worker's count of 16. Seven `tests.test_codefire_cli` IDs reference
a changed production surface (`prepare_json`, `publish_prepared`, or a
cleanup-fault name):
`test_python_http_bundle_deadline_interrupts_and_preserves_precedence`,
`test_python_http_bundle_subject_change_terminal_transport`,
`test_python_http_close_lookup_failure_explicitly_closes_owner`,
`test_python_http_freezes_request_aliases_before_ast_release`,
`test_python_http_payload_close_failure_cancels_prepared_owner`,
`test_python_http_prebuilds_one_owned_header_buffer`, and
`test_python_upload_response_preflight_precedes_effect_and_retries_durable_result`.
All 7 pass. The complete 228-test module (gate 10) subsumes any wider set the
worker may have used, so the count difference changes no conclusion.

### Whole-CLI environment discrimination

The whole `tests.test_codefire_cli` module is **not** reported as green.

| Evidence | Reviewer-observed |
|---|---|
| Candidate whole module | 228 tests, 1014.997 s suite / 1015.183 s monotonic, 226 pass, **2 fail**, exit 1 |
| Failure 1 | `tests.test_codefire_cli.CodeFireCliTests.test_completion_command_and_install_completion` — `AssertionError: 2 != 0 : error: cargo is required to build the Rust codefire binary; pass --binary PATH to use a prebuilt binary` |
| Failure 2 | `tests.test_codefire_cli.CodeFireCliTests.test_install_script_installs_codefire` — identical assertion and identical stderr |
| At continuation base `06e519e4…` | the same two IDs fail with the identical message, 2 tests in 0.929 s |
| At review base `9c5c276…` | the same two IDs fail with the identical message, 2 tests in 0.958 s |
| Environment | `cargo` absent, `rustc` absent |
| Immutable blobs | `tests/test_codefire_cli.py` = `b23c43ce…` and `install.sh` = `841e1526…` at `4930d251…`, `06e519e4…`, `b1640467…`, `0aa0d3d0…`, and `9c5c276…` |

Both failures are inherited environment failures caused by the missing Rust
toolchain, reproduce identically at two independent bases, and cannot be
candidate behaviour because both the test file and `install.sh` are
byte-identical across every revision in the chain. **No additional failure, no
changed reason, and no failed affected in-scope test was observed.** I did not
install a toolchain or alter the environment.

---

## 7. Non-blocking observations

These are recorded for the manager and any successor. **None of them is a
condition of this `ACCEPT`**, and none is a new same-root residual.

1. **Redundant retained state.** `_HTTPReadResponseSlot.cleanup_diagnostics`
   is appended to and never read in production, and the `state_proved`
   compatibility property has no production reader (only
   `tests/…:996`). Unlike the residual the prior reviewer flagged, the
   diagnostic *itself* is consumed — via `handler.log_error` and via
   `record_secondary_failure` — so this is redundant retention, not unread
   evidence. A successor could drop both.

2. **Per-fault text length is unbounded.** `CLEANUP_FAULT_REPORT_LIMIT` caps the
   number of faults at four and `fault_count` stays exact, but each entry is
   `f"{type(fault).__name__}: {fault}"` with no length bound. Every producer
   today raises a bounded `CFError`/ledger message, so the log line is bounded
   in practice; a future fault carrying attacker-influenced text would not be.

3. **Pre-existing GET/POST displacement asymmetry (unchanged by this
   candidate).** `do_POST`'s `except BaseException` arm wraps
   `mutation_slot.close()` and `prepared.close()` in
   `try: … except BaseException: pass` (`codefire:16889-16903`) and then
   publishes a replacement response; `do_GET` now refuses to publish anything
   when disposal fails. `do_POST` is byte-identical at base and candidate, so
   this is inherited, not introduced. The candidate's own `change-playbook.md`
   entry ("keep the shape identical across GET and POST so a successor cannot
   inherit only half of it") is the correct forward binding; the POST side is
   the natural next step and is outside this candidate's write set.

4. **Source-contract precision.** `source-architecture.md` and
   `command-source-trace.md` state that a failed proof "terminates the ledger as
   `cleanup_incomplete`" without qualification, while
   `reject_unpublishable_response` correctly skips `terminate` when a permit is
   still open (the ledger forbids terminating with open permits). As shown in
   F3 this state is not reachable without direct injection, so the contracts are
   accurate for every production-reachable case; the sentence is one clause less
   precise than the code.

---

## 8. Unavailable evidence and integration overlaps

* **Unavailable.** Cargo/Rust toolchain (`cargo`, `rustc` absent) — the two
  whole-CLI failures above and any Rust/`Cargo` gate. Docker, Install, Dogfood,
  unrestricted all-language discovery, and Python-composition integration gates
  remain manager-owned and were not run. `docs/development/tools/run_timed_gate.py`
  is absent from the candidate (deviation recorded in §6).
* **Manifest binding.** The product candidate lineage has no scheduler manifest,
  so `capture_candidate_identity.py` was run without `--manifest`; the manifest
  that exists (`…rereview-claude-20260731.json`) describes *this review task* at
  base `9c5c276…`, not the product candidate. All six changed paths are inside
  the continuation packet's declared write set, checked by hand against the
  packet blob `fad31608…`.
* **Overlaps.** The changed shared primitives — `_HTTPReadResponseSlot`,
  `_release_exact`, `close_with_retry`/`close()`, `cleanup_diagnostic`,
  `record_secondary_failure`, `classify_route_os_error` — are also on the
  Upload/MR-write and shared-primitives paths. The read-route changes are
  additive there (`cleanup_faults` was the only removal, and it had no
  readers), and the affected CLI subset plus the 512-test Python suite are
  green, but accepting this candidate does **not** discharge the Upload,
  MR-write, shared-primitives preflight, or Python-composition gates.

---

## 9. Decision

```
ACCEPT
```

Rationale against each `ACCEPT` precondition:

| Precondition | Status |
|---|---|
| Exact identity | recomputed from a fresh clean detached scratch checkout; every field matches, `blocking_reasons: []` |
| Every blocking item closed | F1, F2, F3 all closed with independent production-reachable counterexamples |
| Both recorded residuals closed | write-only cleanup evidence now consumed; silent `_release_exact` now typed fail-closed, which exposed and repaired a real latent double release |
| All seven inherited closures preserved | re-falsified; no test removed, the fence test strengthened in place |
| Non-vacuous production-reachable counterexamples | every probe drives the live `make_http_handler` over a real socket; each has a matching non-green result on `4930d251…` or a stated reason why not |
| One terminal outcome | every probed path publishes at most one response; no double publication and no publication after disposal |
| No success with debt or permits | retained/temporary residual → zero bytes on the wire; open permit → typed `400`, never a `200` |
| In-scope gates independently green | focused 49/49, broad 512/512, affected CLI 7/7, source docs, project docs, `git diff --check` |
| Two inherited CLI failures explained at base | reproduced identically at `06e519e4…` and `9c5c276…` with `cargo` and `rustc` absent and byte-identical test/install blobs |
| No new same-root residual | audited across every caller of the changed primitives and both publication paths; the only asymmetry found is pre-existing and byte-identical at base |

`ACCEPT_WITH_NOTES` is forbidden and was not used; §7 is informational only.

### Required next action

Manager-owned integration decision for `b16404676639336f705feffd9af4857d4a1fcc69`.
This report authorizes nothing beyond that decision. Suggested successor scope,
not a condition of acceptance: bring `do_POST`'s displacement precedence up to
the read route's (§7 note 3) before any Upload or MR-write composition binds to
the new protocol.
