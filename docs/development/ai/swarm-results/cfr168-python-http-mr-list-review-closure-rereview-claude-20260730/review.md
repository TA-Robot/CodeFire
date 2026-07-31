# CFR-168 Python HTTP MR-List Review-Closure Rereview

Reviewer: fresh Claude Opus 5 (`claude-opus-5`), independent report-only lane.
New agent, new session, new worktree; no implementer conversation, WIP, or
conclusion was inherited.

Captured: 2026-07-31 (review worktree base `8143f437342cf7bf11322d82f6ca63ab888382e8`).

Mode: report-only. The only write is this file.

## Verdict

`REQUEST_CHANGES`

Identity is exact and all seven prior findings are closed. Every required gate
is independently green except two `install.sh`/Cargo tests that fail identically
at the task base for want of a `cargo` toolchain and are out of this lane's
scope. The candidate is nevertheless returned because required falsification
item 5 could not be proven — it was disproven — and because the new
prepared-response publication path introduces three same-root residuals, one of
which publishes `200 OK` with live ownership debt. Details in
[§5 Reviewer Findings](#5-reviewer-findings).

## 1. Recomputed Identity

Every value below was recomputed in this worktree from the object database, not
read from the manager card or any worker prose. No branch tip was followed.

| Item | Ticket / card value | Reviewer-recomputed | Match |
|---|---|---|---|
| Candidate SHA | `4930d25178be11e064060487f8c254802b09868d` | `4930d25178be11e064060487f8c254802b09868d` | yes |
| Parent | `2045955951d595d05602aa31f3cdae2596f6054a` | `2045955951d595d05602aa31f3cdae2596f6054a` (from `git cat-file -p`) | yes |
| Tree | `17cb396ee30a887be6698b822b60f52ac2adb67c` | `17cb396ee30a887be6698b822b60f52ac2adb67c` | yes |
| Ancestry from task base `7f37394c76e3626f810d0cd133be70feea89b5d2` | ancestor | `git merge-base --is-ancestor` → true | yes |
| Ancestry from rejected predecessor `38da8988ac8bc1fcdfcb82c62e7aa0d28791500c` | ancestor | `git merge-base --is-ancestor` → true | yes |
| Additive commit list | 6 listed SHAs | identical list, identical order, via `git rev-list --reverse 7f37394..4930d25` | yes |
| Additive commit count | `6` | `6` | yes |
| Merge commits | `0` | `git rev-list --merges 7f37394..4930d25` → `0` | yes |
| Changed paths | six declared paths | `codefire`, `docs/development/change-playbook.md`, `docs/development/command-source-trace.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md`, `tests/test_python_http_mr_list_decision_work.py` | yes |
| Numstat | 768/236, 40/1, 24/7, 45/13, 26/10, 1420/20 | identical | yes |
| All 12 base/head blob IDs | card table | all 12 identical | yes |
| Cumulative binary patch SHA-256 | `6a9ca3d1046ba5ec213645c0710f900425191269360a70a3d5c4a50090528543` | `6a9ca3d104…8543` | yes, see note |
| Clean tracked/untracked | empty porcelain | `git status --porcelain=v1 --untracked-files=all` empty at `8143f43` | yes |
| Write-set escape | none | none; all six paths inside the repair lane's declared write set | yes |
| First report SHA-256 | `95617c3cb48320e7256d23ae658f4cbedb18f9c4f8e9a7803a0417972f28042c` | identical | yes |
| Second report SHA-256 | `6840035553871f5251e32d67bb5b2f9d6c6ee061b7ab3106e94de373ec97b20d` | identical | yes |

**Patch-digest note (resolved, not a mismatch).** The card records the digest
without recording its command. `git diff --binary <base> <head> | sha256sum`
yields `27830bdf4414…`, not the declared value. The reviewer enumerated eleven
plausible command forms; exactly one reproduces the declared digest bit for bit:

```
git diff --binary --full-index 7f37394c76e3626f810d0cd133be70feea89b5d2 \
    4930d25178be11e064060487f8c254802b09868d | sha256sum
6a9ca3d1046ba5ec213645c0710f900425191269360a70a3d5c4a50090528543
```

The gap is the `--full-index` flag (abbreviated vs. full blob IDs in the index
lines), not content divergence. Content identity is independently established
by the tree hash and by all twelve blob IDs, which are command-independent.
The generating tool named on the card,
`docs/development/tools/capture_candidate_identity.py`, **does not exist in this
tree** and could not be re-executed; this is recorded under
[Unavailable Evidence](#8-unavailable-evidence). Recommendation for the manager:
record the exact digest command alongside the digest.

No identity mismatch. Identity is not the reason for this verdict.

## 2. Production Call Graph

`GET /v1/projects/{org}/{app}/merge-requests`, as reached live through the real
socket dispatcher. Fence and publication points marked.

```
handle_one_request
  transport_boundary: rfile.readline(65537)                      [marks transport origin]
  transport_boundary: parse_request()
  finish_request_headers -> _request_header_retained
  do_GET
    read_slot = _HTTPReadResponseSlot(handler)
    dispatch_get(read_slot)
      route_filesystem_boundary                                  [marks filesystem origin]
        handle_http_request_list(..., response_slot=read_slot)
          deadline.check("merge request list planning")
          http_project_root / remote_dirs
          deadline.check("merge request list project gate")
          with remote_project_gate(project_root, shared=True) as gate:
            _open_directory_nofollow(project_root)
            _read_regular_bounded_at("remote_project.json")       [descriptor read 1/5]
            _open_directory_at("locks")
            stable_remote_lock("project-gate.lock", shared)
            prove_pinned_project_state("acquiring")               [descriptor read 2/5]
            collect_http_merge_request_list_payload(..., response_slot, gate)
              op.capture_authority()
                accounted_sorted_json_paths(merge_requests)
                read_json_accounted(<each MR record>, capture)
                  _open_authority_parent(parent)                  [parent capability pinned]
                op.stable_project_id -> read_json_accounted(...)  [descriptor read 3/5, accounted]
                op.consulted_branch  -> read_json_accounted(...)
              op.validate_records() -> validate_record / validate_subject
                SealedCommitValidationSession (one per pinned object root)
                  escrow_object_set -> _collect_escrow_object_ids / _digest_escrow_records (memoized)
              payload = op.project()
              response_slot.prepare(200, payload)                 == ENCODE
              op.finalize_authority()                             == FENCE 1 authority [descriptor read 4/5]
              op.finalize_sessions()                              == FENCE 2 sessions
              op.poll(PROJECTION_PHASE)
              gate.fence() -> prove_pinned_project_state          == FENCE 3 project  [descriptor read 5/5]
              _finish_merge_request_list_operation -> close_with_retry(op)  == CLEANUP
              response_slot.require_publishable_state()           == PUBLISHABLE PROOF
          gate.__exit__: gate.fenced -> exit fence SKIPPED
          stable_remote_lock.__exit__: fcntl.flock(LOCK_UN); os.close(fd); os.close(dup dirfd)   <-- POST-FENCE
          remote_project_gate finally: os.close(locks_fd); os.close(project_fd)                  <-- POST-FENCE
    prepared = read_slot.take()
    publish_prepared -> transport_boundary: prepared.publish(wfile)  == FIRST RESPONSE BYTE
```

Descriptor read protocol totals exactly five physical
`remote_project.json` opens, matching both `DESCRIPTOR_READ_PROTOCOL` in the
test module and the "exactly five" claim added to
`docs/development/command-source-trace.md`.

## 3. Owner / Effect / Fence Timeline

| Stage | Owners live | Ledger effect | Fence |
|---|---|---|---|
| header parse | `_request_header_retained` | `request_header_bytes` charged, retained acquired | — |
| capture | per-capture `AUTHORITY_CAPTURE_BYTES`, `_paths_retained`, `_records_retained`, `_branches_retained`, `_project_identities_retained` | `record_reads`, `files_read`, `bytes_hashed` | parent capability pinned per leaf |
| validate | `_sessions_retained`, `_session_entries`, `_escrow_ordered_retained`, `_escrow_memo_retained` | `objects_walked`, `semantic_validations`, memo hits | sealed-session internal |
| project | `_projection_retained` | projection entries | — |
| **encode** | prepared owner takes projection ownership | `response_bytes`, `response_header_bytes` | — |
| **fence 1** | all captures still live | `snapshot_revalidation_records/bytes` | authority: parent id, descriptor id, S_ISREG, mode, nlink, size, digest |
| **fence 2** | sessions still live | session revalidation | sealed object roots |
| **fence 3** | gate fds still live | — | project root, locks dir, descriptor bytes |
| **cleanup** | every operation owner released with retry | releases | `close_with_retry`, 2 attempts |
| **publishable proof** | request header + prepared owner only | — | `require_publishable_state` |
| post-fence teardown | prepared owner only | — | **none — 5 fallible ops, see F1** |
| publish | prepared owner consumed | `response_bytes` published | transport boundary |

## 4. Row-by-Row Prior-Finding Disposition

Findings are the seven raised by the second (Fugu) report at
`6840035553871f5251e32d67bb5b2f9d6c6ee061b7ab3106e94de373ec97b20d`, which
subsume the first (Claude) report at `95617c3c…`. Each disposition below was
re-falsified independently through the live GET/POST dispatcher; none is taken
on the candidate's word.

| Finding | Reviewer disposition | Independent evidence |
|---|---|---|
| **MUST-1** parent-directory failure recorded as absence, silently corrupting status | **Closed** | `read_json_accounted` no longer swallows a parent `OSError`; `_open_authority_parent` pins the parent no-follow and raises the typed family. Reviewer confirmed by candidate test `test_unavailable_parent_authority_is_never_leaf_absence` (7 subtests fail on `38da898`, pass here) and by reading the removed `except OSError: return default, 0` arm. `parent_identity` is stored in the capture and reproved at the fence. |
| **MUST-2** type-only transport classification drops filesystem `TimeoutError` on GET and POST | **Closed** | Origin is now marked at the boundary, not inferred from class. `transport_boundary()` wraps request-line, header, body and response transfers; `route_filesystem_boundary()` wraps GET route execution; `classify_route_os_error` is shared by both dispatch arms. Reviewer probe **P7** drove a real leaf symlink to a typed `400` (`ELOOP`) with zero debt. Candidate test covers filesystem `ETIMEDOUT`/`ECONNRESET`/`EIO` across `handle_http_request_list`, `handle_http_request_merge`, `handle_http_upload`, and four real socket faults; 13 subtests fail on `38da898`. **Caveat:** one post-fence window is still unclassified — see **F2**. |
| **MUST-3** cleanup failures swallowed after ownership locally erased | **Closed** | `_release`/`_release_exact` zero an owner only after the exact ledger release succeeds; sessions are removed from `_sessions` only after `close` and the exact release succeed; `close()` is idempotent and returns the first unreleased fault; `close_with_retry` retries once. Reviewer probe **P5** blocked *every* retained release permanently: the request published **zero bytes**, `terminal_reason=cleanup_incomplete`, residual `20665` — no `200` with debt on that path. 7 subtests fail on `38da898`. **Caveat:** a different route to a `200` with debt exists — see **F3**. |
| **MUST-4** mode/link-count changes after the last metadata check pass the fence | **Closed** | `_same_pinned_metadata` and `_require_pinned_authority_metadata` reprove dev/ino, `S_ISREG`, mode, nlink and size together; `read_json_accounted` adds a terminal `fstat` after the digest, and `revalidate_json_authority_capture` reproves metadata before *and* after the interruptible digest plus the parent identity before the digest comparison. New reasons `parent_changed`, `mode_changed`, `link_count_changed` are wired into the diagnostic family. 14 subtests fail on `38da898` across `before open` / `during read` / `during digest` boundaries. |
| **SHOULD-1** authority can change after all fences but before response preparation/publication | **Closed as stated; overclaimed in its replacement** | The order is inverted correctly: encode → authority fence → session fence → project fence → cleanup → publishable proof → take → publish, asserted exactly by the candidate's ordering oracle and reproduced by reviewer probe **P9**. Reviewer confirmed **no authority read and no identity comparison** occurs after fence 3, so the original defect is genuinely closed. The candidate's *stronger* replacement claim ("no fallible … work") is false — see **F1**. |
| **COULD-1** descriptor "read once" oracle measures only one helper | **Closed** | Both options the ticket allows are delivered: the renamed `test_stable_project_id_reads_the_descriptor_once_from_charged_bytes` scopes its claim to the accounted `stable_project_id` boundary, and the new `test_descriptor_read_protocol_is_exact_and_route_wide` instruments every physical `os.open("remote_project.json")` with a `(reader, owner)` protocol of exactly 5. Both restoration mutations are present and fail the oracle: un-memoising `stable_project_id` raises its count, and bypassing to `remote_project_stable_id` drops it to 0. Fails on `38da898`. |
| **COULD-2** poll mutation proves helper ownership, not per-poll necessity | **Closed** | Sites are resolved from live production frames through a helper-frame filter, so the enumeration cannot be satisfied by swapping a phase helper. Each of ≥18 owned sites is silenced individually with a `_TriggeredClock` armed at that same site, and the test requires `lag > 0` — work admitted that no other poll could cancel. The in-test counter-mutation (silencing `_validation_poll`) is shown to silence more than one site, proving the new oracle is strictly finer than the rejected one. Reviewer probe **P10** independently reproduced the 1/8/24/80/200 curves. |

### Prior residual items

| Residual | Disposition | Evidence |
|---|---|---|
| SHOULD-2 evidence overclaims | **Closed** | Source contracts now state the parent-capability rule, boundary-marked origin, retryable cleanup, terminal metadata protocol, the five-read descriptor protocol, and the prepublication order. `change-playbook.md` additionally records the *undetectable* boundaries as pinned-snapshot outcomes instead of claiming rejections, and records the intermittent-flake root cause. This is honest, non-overclaiming evidence. |
| Multi-fault filename precedence (first report MUST-1) | **Closed and re-falsified** | Deferred per-record outcomes are still raised in filename order; the eager-capture non-vacuity mutation is retained and still diverges. |
| Legacy baseline parity | **Closed** | `test_stable_pre_existing_links_stay_compatible_authority` compares the live route against `legacy_request_list` for the multi-link descriptor and requires byte-identical diagnostics. |

## 5. Reviewer Findings

Three new residuals, all rooted in the same new code: in `do_GET`,
`prepared = read_slot.take()` and the unconditional `publish_prepared(prepared)`
run **after** the `except CFError` / `except OSError` arms, so any error raised
once the slot holds a prepared response — but outside
`collect_http_merge_request_list_payload`'s own `try` — is discarded and the
prepared `200` is published regardless. `do_POST` does **not** have this shape:
its `except BaseException` arm closes `prepared` before re-preparing. This is a
GET/POST composition asymmetry directly relevant to required item 10.

### F1 — Five fallible operations run between the terminal fence and the first response byte (blocks item 5)

Required item 5 says: *"After the final fence, prove no fallible or
authority-dependent work remains before first byte."* The reviewer instrumented
`fcntl.flock` and `os.close` and armed the trace at the publishable proof
(probe **P9**, `/tmp/cfr168rr/probe3.log`):

```
TERMINAL project fence
publishable proof
  post-fence fcntl.flock(LOCK_UN)  [fallible]
  post-fence os.close(fd)          [fallible]
  post-fence os.close(fd)          [fallible]
  post-fence os.close(fd)          [fallible]
  post-fence os.close(fd)          [fallible]
>>> FIRST RESPONSE BYTE (publish)
fallible operations strictly after the fence: 5
```

These are `stable_remote_lock.__exit__` (unlock + two closes) and
`remote_project_gate`'s `finally` (two closes), all reached because
`handle_http_request_list` returns from *inside* `with remote_project_gate(...)`.

The candidate asserts the opposite in three places it added:

* `codefire:13393`, `collect_http_merge_request_list_payload` docstring — "After
  that terminal fence only the handler's `take`/`publish` pair remains, so no
  fallible, authority-dependent, callback-capable, or input-proportional work
  can run before the first response byte."
* `docs/development/source-architecture.md:740` — "After that terminal fence only
  the prepared owner's transfer and its publication remain, so no fallible,
  authority-dependent, callback-capable, or input-proportional work can run
  between the last authority observation and the wire."
* `docs/development/change-playbook.md:351` — "… prove the publishable ledger
  state, and leave nothing fallible, authority-dependent, callback-capable, or
  input-proportional between the fence and the first response byte."

The *authority-dependent* half of the claim is true and verified. The
*fallible* half is false. The candidate's own ordering oracle cannot see this
because it instruments only named methods (`prepare`, `finalize_*`, `close`,
`require_publishable_state`, `take`, `publish`), never the gate/lock teardown
that runs between the last two.

### F2 — A real filesystem-origin `OSError` in that window is silently swallowed (blocks item 2 scope)

Reviewer probe **P2**/**P6** injected a genuine `OSError(EIO)` at the real
production call site `fcntl.flock(fd, LOCK_UN)` in `stable_remote_lock`'s
teardown — an OS-level fault of exactly the class required item 1/2 demands be
falsified, not a logic mutation:

```
[P2] injected        = True
[P2] status line     = b'HTTP/1.0 200 OK'
[P2] has projection  = True
[P2] terminal_reason = complete
[P6] handler logs    = []          <-- no log_error, no log_message
```

The error escapes the route with a filesystem origin, is turned into a typed
`400` by `do_GET`'s `except OSError` arm, and is then **thrown away** by the
unconditional `read_slot.take()` publish. Nothing is published about it, nothing
is logged, `close_connection` is not set, and the ledger reports `complete`.

This contradicts the contract the candidate added to
`docs/development/command-source-trace.md`: "The shared Python
`do_GET`/`do_POST` dispatchers publish that family for any filesystem-origin
`OSError` escaping a route."

Publishing the already-fenced `200` may well be the *desired* outcome here —
the response was legitimately proven — but silently discarding a typed
filesystem fault is not, and the behaviour is neither documented nor tested.

### F3 — `require_publishable_state` detects unaccounted work and the `200` publishes anyway (blocks item 3 / ACCEPT "zero debt")

`_HTTPReadResponseSlot.require_publishable_state` documents: *"A success
response may not publish while any route ownership, temporary byte, or open
permit survives, so a cleanup that could not reach zero cannot hide behind a
published 200."* `source-architecture.md` states it as a guarantee: *"A success
can therefore never publish while route ownership survives."*

Reviewer probe **P1** made a residual owner survive into the proof point, so the
guard fires for exactly its intended reason:

```
[P1] guard raised     = True  CFError: prepared read response cannot publish with unaccounted current work
[P1] status line      = b'HTTP/1.0 200 OK'
[P1] has projection   = True
[P1] current          = {'retained_bytes': 4096, 'temporary_bytes': 0}
[P1] terminal_reason  = cleanup_incomplete
```

The guard's `CFError` is caught by `do_GET`'s `except CFError` arm, which
computes a typed `400`; `read_slot.take()` then returns the prepared `200` and
publishes it, discarding the `400`. A `200 OK` carrying the full projection went
onto the wire with 4096 bytes of live retained ownership. The `cleanup_incomplete`
terminal reason is set in `handle_one_request`'s `finally` — *after* the response
was already on the wire — so it records the debt but cannot prevent it.

The guard is therefore inert: it cannot enforce the guarantee it documents. Its
only observable effect in production is `self.state_proved = True`, a flag
written twice in `codefire` and read **only** by a test (`codefire:15348`,
`codefire:15392`, test line 950) — a helper-only flag, which the ticket
explicitly says does not count as evidence.

**Reachability, stated honestly.** Triggering the guard required injecting a
residual owner; the reviewer could not exhibit a production input that leaks
ownership past a successful `close()`. So F3 is a *dead safety net plus a false
documented guarantee*, not a demonstrated wrong answer on clean inputs. Control
probe **P3** shows the legacy no-slot `/branches` route behaves the same way
(`200` + residual 4096 + `cleanup_incomplete`), so the outcome is not a
regression against the base — but the whole point of the new guard was to stop
it on the slot path, and it does not.

### Minor observations (recorded, not blocking)

* `_HTTPMergeRequestListOperation.cleanup_recoveries` is incremented and never
  read anywhere; the `cleanup_faults` property is never read anywhere. The
  claim "a fault is recorded rather than swallowed" holds only in the weak
  sense of an append to a list nothing consumes. `record_secondary_failure`'s
  `add_note`/`codefire_cleanup_failures` path *is* observable, so the primary
  diagnostic genuinely survives.
* `_release_exact` silently returns `None` when `amount > owned`. Reviewer
  verified no production path reaches that branch (`_session_entries` and
  `AUTHORITY_CAPTURE_BYTES` are both exactly matched by their scalars), so it
  is defensive only, but it would mask a future accounting bug.
* `remaining_response_bytes` replaces `profile.limit("response_bytes")` in the
  `response_body` reserve. The reviewer checked this is a *fix*, not a
  weakening of reserve-before-allocation: the second encoding after a discarded
  first one still reserves before allocating, and reserving the full limit on
  top of an already-charged cumulative would fail outright. `freeze_response_body`
  still bounds the freezer by the full limit, which is the correct upper bound.

## 6. Mutation Evidence

| Mutation | Where applied | Result |
|---|---|---|
| Missing / symlinked / file-replaced / denied parent of `branches` | live fixture, before request | typed `filesystem authority is unusable: <branches>`, `terminal_reason=complete`; stale downgrade never published as open |
| Injected parent open `EIO`, parent metadata `EIO`, parent `ETIMEDOUT` | `_open_directory_nofollow` / metadata | same typed family; injection counter > 0 |
| Parent replaced between capture and fence (absent leaf) | inside `finalize_authority` | `pinned authority record changed: … (parent_changed)` |
| Leaf symlink (reviewer **P7**) | `MR-*.json` → decoy | `400`, `filesystem authority is unusable: …: Too many levels of symbolic links`, zero debt |
| Filesystem `ETIMEDOUT`/`ECONNRESET`/`EIO` from three routes | patched route functions | typed `400` on GET and POST; `os_error_origin` → `filesystem` |
| Real socket write reset / broken pipe / read timeout / read disconnect | `_FailingSocketFile` on real `rfile`/`wfile` | connection abandoned, `wire == b''`, no second response, zero debt, `os_error_origin` → `transport` |
| `MemoryError` from the route | patched route | abandoned, stays outside the filesystem arm |
| Each cleanup release failed once, ordinals 1..N | `DecisionWorkLedger.release` inside `close` | one retry, identical body, zero debt |
| Every retained release failed permanently (reviewer **P5**) | same | zero-byte reply, `cleanup_incomplete`, no `200` |
| Session close failure / prepared-owner close failure | patched `close` | retried, identical body, zero debt |
| mode / hardlink add / hardlink remove / symlink / size / identity / content at `before open`, `during read`, `after EOF`, `during digest`, `after digest`, `before publication` | live fixture | each applicable drift rejected by the terminal metadata/digest fence; undetectable boundaries documented as pinned-snapshot outcomes |
| Authority mutated at the publishable-proof point | inside `require_publishable_state` | frozen projection unchanged — pinned snapshot, as documented |
| Descriptor replaced before the project fence | inside `finalize_sessions` | `400 remote project descriptor changed`, no partial response |
| `stable_project_id` un-memoised / bypassed to legacy helper | patched | descriptor-read oracle fails for the intended reason |
| Each of ≥18 production poll sites silenced individually | frame-resolved | `lag > 0` in every case: no redundant poll |
| `_validation_poll` helper silenced | patched | silences > 1 site, proving the per-site oracle is strictly finer |
| **`fcntl.flock(LOCK_UN)` → `EIO` after the fence (reviewer P2/P6)** | real production call site | **`200` published, error and log both absent** — F2 |
| **Residual owner surviving into the proof point (reviewer P1)** | patched `finalize_sessions` | **guard raises, `200` publishes anyway with 4096 debt** — F3 |

## 7. Validation Results

Toolchain: Python 3.10.12, git 2.34.1, Linux 6.6.87.2-microsoft-standard-WSL2.
All commands run in the review worktree at `8143f43` unless noted. Logs under
`/tmp/cfr168rr/`.

| Gate | Command | Count | Duration | Exit | Log SHA-256 |
|---|---|---:|---:|---:|---|
| Identity/ancestry/tree/patch/paths/clean state | `git cat-file -p`, `git merge-base --is-ancestor`, `git rev-list`, `git diff --binary --full-index … \| sha256sum`, `git rev-parse <rev>:<path>`, `git status --porcelain=v1 -uall` | 16 checks | <1 s | 0 | inline in §1 |
| py_compile | `python3 -m py_compile codefire tests/test_python_http_mr_list_decision_work.py tests/test_codefire_cli.py` | 3 files | 0.805 s | 0 | `f5a70233f0f819a232b6861b0fa88401fd1de652d32b4af9502475e83f1fe4aa` |
| Focused MR-list | `python3 -m unittest tests.test_python_http_mr_list_decision_work -v` | 44 pass | 94.967 s (real 1m35.130s) | 0 | `44df6950146e030d5e5a8f884dc5bab33870a39e433821f17851236003715206` |
| Rejected-candidate reversal | candidate tests vs `38da898` `codefire` in `/tmp/cfr168rr/rev` | **41 failures, 8 errors** | 1m38.729s | 1 | `7e614c2bee64df6dc8ca650c2011e59d570cd230bc1a81ed97ce43d007d176ab` |
| Live-socket adversarial probes | reviewer probes P1–P3 | 4 | — | 0 | `80b004cda35b204f7186f8fa6750e6613a50046af71f432d8ca702d17286afaa` |
| Live-socket adversarial probes | reviewer probes P5–P8 | 4 | — | 0 | `5534e9d9b54e7dccefe57a46804466fecf3c479a12e5dd46ac431976f6bc88a7` |
| Live-socket adversarial probes | reviewer probes P9–P10 | 2 | — | 0 | `7d01d6c3e6c97e9d7426d70050279a3c539a8a4f007e40d3e1dd17b5f24a1850` |
| Broad Python | `python3 -m unittest discover -s tests -p 'test_python_*.py'` | 507 pass | 407.631 s test time | 0 | `ab0769d7483f5ed648aa644a359f059248d809cbf34515c3c4cee01ff4f01ea8` |
| Affected legacy CLI | `python3 -m unittest tests.test_codefire_cli -v` | **226 pass, 2 fail** (228 ran) | 1129.750 s (real 19m23.496s) | 1 | `ac8e14146d75f6e668fd9395128284a7d649ac7049ebd272b065c810fd785b08` |
| Legacy CLI failure discrimination | same two test IDs at task base `7f37394` in `/tmp/cfr168rr/base` | 2 fail | 1.260 s | 1 | `f1ee49de7f56985caa4a6a150a0da53a14f3e67b41b3cd13d997bf9adb560b2a` |
| Source docs | `python3 docs/development/tools/check_source_docs.py` | 44 modules, 34 commands, 12 object kinds | 0.344 s | 0 | `329325e2b93b76f9f92e7898648413077c660a61f357c88e4e33491620319a93` |
| Project docs | `python3 docs/development/tools/check_project_docs.py` | 546 Markdown files | 1.191 s | 0 | `d6ce881062eb2b2e3c1acd7ad79744f010878b717c04af055db4cea3440afb11` |
| Patch hygiene | `git diff --check` | clean | <1 s | 0 | `19eaf43821a7660ec323a87c8457bf74823beb296c39f5e01aa8a683aa50f061` |

The broad-suite wall clock (`real 275m`) reflects the run being descheduled
behind other background work in this lane; `user 6m8s` and the suite's own
`407.631s` are the honest figures. Result is unaffected: 507/507 `OK`.

Project docs reports 546 files against the card's 544; the delta is the two
freeze/rereview documents added by `8143f43` above the candidate, not a
candidate change.

### Exact affected `test_codefire_cli` scope

`tests/test_codefire_cli.py` is byte-identical between base and candidate, so
"affected" means tests that traverse the changed shared primitives
(`read_json_accounted`, `accounted_sorted_json_paths`,
`revalidate_json_authority_capture`, `remote_project_gate`,
`_RemoteProjectGateFence`, `is_transport_os_error`, `transport_boundary`,
`_PreparedHTTPResponseOwner.publish`, `prepare_json_owned`). The reviewer ran
the **whole module** rather than a subset, so the affected set is covered by
construction. The 83 directly affected IDs are every
`test_python_http_*`, `test_python_https_*`, `test_python_remote_*` and
`test_remote_*` test in the module; all 83 pass. Representative IDs that
exercise the exact primitives this candidate changed, all `ok`:

`test_python_http_reserves_response_limit_before_freeze`,
`test_python_http_response_body_is_all_or_typed_budget_error`,
`test_python_http_response_header_is_never_partially_sent`,
`test_python_http_prepare_faults_release_all_response_owners`,
`test_python_http_payload_close_failure_cancels_prepared_owner`,
`test_python_http_owners_live_until_their_release_points`,
`test_python_http_base_exceptions_release_response_owners`,
`test_python_http_base_exceptions_release_post_request_owners`,
`test_python_http_bundle_owner_close_failure_is_retryable`,
`test_python_http_bundle_subject_change_terminal_transport`,
`test_python_http_get_budget_error_keeps_typed_parity`,
`test_python_remote_list_and_doctor_use_one_zero_current_ledger`,
`test_python_remote_doctor_exception_closes_all_ledger_ownership`,
`test_python_remote_lock_uses_persistent_kernel_lock`.

`test_python_http_reserves_response_limit_before_freeze` is the one that would
have caught a weakening of reserve-before-allocation from the
`remaining_response_bytes` change; it passes.

The two failures are `test_completion_command_and_install_completion` and
`test_install_script_installs_codefire`, neither of which is in the affected
set — see below.

### New-regression reversal against `38da898`, per test

Reviewer reproduced the manager card's aggregate exactly (41 failures / 8
errors). Per-test status of the **12 new tests**:

| New test | Status on `38da898` |
|---|---|
| `test_unavailable_parent_authority_is_never_leaf_absence` | FAILS ×7 |
| `test_absent_authority_fence_reproves_its_parent_identity_and_state` | FAILS ×1 |
| `test_error_origin_is_classified_at_the_boundary_that_raised_it` | FAILS/ERRORS ×13 |
| `test_primary_diagnostic_survives_a_cleanup_failure` | FAILS ×1 |
| `test_cleanup_retries_every_owner_release_without_losing_ownership` | FAILS ×7 |
| `test_unrecoverable_cleanup_never_publishes_hidden_current_work` | FAILS ×1 |
| `test_terminal_metadata_is_reproved_through_the_read_and_digest` | FAILS ×14 |
| `test_the_terminal_fence_immediately_precedes_the_first_response_byte` | ERRORS ×1 |
| `test_descriptor_read_protocol_is_exact_and_route_wide` | FAILS ×1 |
| `test_every_production_poll_site_is_individually_required` | **PASSES** |
| `test_stable_pre_existing_links_stay_compatible_authority` | **PASSES** |
| `test_stable_project_id_reads_the_descriptor_once_from_charged_bytes` | **PASSES** |

Required item 9 says every new regression must fail on `38da898`. Three do not.
The reviewer investigated each and finds all three benign and explainable — but
the candidate's evidence never states this, so it is recorded as an
**explained deviation**, not a hidden gap:

1. `test_every_production_poll_site_is_individually_required` — the rejected
   finding was that the *oracle* was too coarse, not that production polls were
   redundant. `38da898`'s polls were already individually necessary, so a finer
   oracle correctly passes on it. Non-vacuity is carried instead by the
   in-test helper mutation, which the reviewer confirmed silences more than one
   site.
2. `test_stable_pre_existing_links_stay_compatible_authority` — the removed
   `if capture.get("nlink") != 1` check applied **only** to the project
   descriptor, and the shared gate's `_read_regular_bounded_at` already rejects
   `st_nlink != 1` for that same file (`codefire:2278`). The removal is
   therefore strictly redundant and observationally inert, which is exactly why
   the test passes on both revisions. The test still pins the contract and
   compares against the legacy baseline.
3. `test_stable_project_id_reads_the_descriptor_once_from_charged_bytes` — a
   **rename** of the pre-existing
   `test_project_descriptor_is_read_once_from_the_bytes_it_charges`, narrowed in
   title to the boundary it actually measures. Not a new regression; the
   route-wide claim moved to `test_descriptor_read_protocol_is_exact_and_route_wide`,
   which does fail on `38da898`.

No test was deleted (33 base → 44 head, one rename, twelve additions).

### Earlier failures — recorded, not erased

Two legacy CLI tests failed. Per required item 9 they are recorded in full and
are **not** erased by any rerun; the reviewer did not rerun them to obtain a
different result.

```
FAIL: test_completion_command_and_install_completion (tests.test_codefire_cli.CodeFireCliTests)
  File "tests/test_codefire_cli.py", line 13523, in test_completion_command_and_install_completion
    self.assertEqual(proc.returncode, 0, proc.stderr)
AssertionError: 2 != 0 : error: cargo is required to build the Rust codefire binary;
                        pass --binary PATH to use a prebuilt binary

FAIL: test_install_script_installs_codefire (tests.test_codefire_cli.CodeFireCliTests)
  File "tests/test_codefire_cli.py", line 13367, in test_install_script_installs_codefire
    self.assertEqual(proc.returncode, 0, proc.stderr)
AssertionError: 2 != 0 : error: cargo is required to build the Rust codefire binary;
                        pass --binary PATH to use a prebuilt binary
```

**Discrimination.** Both are `install.sh`-driven and both reproduce **identically
at the task base `7f37394`**, which precedes every candidate commit
(`/tmp/cfr168rr/base`, log `f1ee49de…`, 1.260 s). `command -v cargo` and
`command -v rustc` both report not installed in this environment. The failures
are therefore environmental toolchain absence, not candidate-caused, and both
land squarely in the Cargo/Install surface this ticket declares manager-owned
and out of scope. No wire bytes, authority snapshot, or phase/poll trace apply:
neither test reaches the HTTP dispatcher.

The manager card's "115/115 pass" for this suite does not match the 228 tests
the module actually contains; the reviewer ran the whole module rather than a
subset. That discrepancy is recorded for the manager, not charged against the
candidate.

No unexplained failure remains. Every other reviewer-run gate was green on first
execution and no rerun was used to convert a failure into a pass. The
candidate's own two intermittent defects were identified, root-caused in
`change-playbook.md`, and fixed in the final additive commit `4930d25`; the
reviewer's 44/44 focused run and 507/507 broad run are consistent with that.

## 8. Unavailable Evidence

* `docs/development/tools/capture_candidate_identity.py`, named on the manager
  card as the identity generator, is absent from this tree. The digest was
  reconstructed by command enumeration instead (§1).
* The card's exact digest command is not recorded; `--full-index` was inferred.
* Manager log, event log and transcript digests
  (`667005fc…`, `da61edc2…`, `edf76b13…`) could not be verified — those artefacts
  are not present in the worktree.
* Cargo, Docker, Install, Dogfood, unrestricted all-language discovery, and the
  Python composition gates were **not** run: they are manager-owned by this
  ticket's own scope statement. This environment has **no `cargo` and no
  `rustc`**, which is also why the two `install.sh` tests inside
  `tests/test_codefire_cli.py` cannot pass here at any revision.
* The 10-consecutive-run focused-suite stability figure on the card was not
  reproduced; the reviewer ran the focused suite once (44/44). A single green
  run cannot by itself refute or confirm a ~1-in-5 intermittent, so the
  candidate's flake fix is accepted on its documented root cause and code
  change, not on reviewer-observed repetition.

## 9. Integration Overlaps

* **Shared primitives.** `read_json_accounted`, `accounted_sorted_json_paths`,
  `revalidate_json_authority_capture`, `remote_project_gate`,
  `is_transport_os_error` and `_read_regular_bounded_at` are shared far beyond
  this route — `remote_project_gate` alone has 19 use sites (one definition at
  `codefire:2534`, fifteen plain `with`, three via `ExitStack.enter_context` or
  passed as a context manager, one binding). The gate signature changed from
  `yield` to `yield gate`; every use site was checked and only the MR-list site
  (`codefire:13544`) binds the value with `as gate`, so the other eighteen are
  unaffected. `accounted_sorted_json_paths`'s diagnostic changed from
  `authority directory is unavailable: …` to the typed family; the reviewer
  grepped the whole repo and found **no** remaining reference to the old string.
* **`is_transport_os_error` semantics changed globally** from an `isinstance`
  test to an attribute test. Any future caller that does not run inside
  `transport_boundary()` will now classify a genuine socket error as
  non-transport. This is the intended design, but it is a repo-wide behavioural
  contract change that Upload and MR-write successors must honour.
* **GET/POST asymmetry (item 10).** The two dispatch arms now share
  `classify_route_os_error`, `transport_boundary`, `record_secondary_failure`
  and `close_with_retry` — genuinely one composition-compatible interface for
  error origin and retryable cleanup. They do **not** share prepared-response
  discipline: `do_POST` closes its prepared owner in the error arm before
  re-preparing; `do_GET` publishes its prepared owner unconditionally. An Upload
  or MR-write successor that copies the `_HTTPReadResponseSlot` shape will
  inherit F2 and F3. This asymmetry should be resolved before the shape is
  reused.
* Acceptance of this candidate would not, and this rejection does not, disturb
  accepted Upload/MR-write bindings, shared-primitives preflight, Python
  composition, or its own fresh exact review.

## 10. Decision

`REQUEST_CHANGES`

`ACCEPT` requires all Must/Should/Could/residual items closed with
production-reachable counterexamples, zero debt/permits with one terminal
outcome, and no new same-root residual. Identity is exact, all seven prior
findings are closed, the oracles are non-vacuous and demonstrably finer than the
rejected ones, and every required gate is independently green apart from two
out-of-scope Cargo/Install tests that fail the same way at the task base — this
is a substantially stronger candidate than either predecessor. It is
nevertheless returned because:

1. Required item 5 asks the reviewer to *prove* no fallible work remains between
   the final fence and the first byte. The reviewer disproved it: five fallible
   operations run there (**F1**).
2. A genuine filesystem-origin `OSError` in that window is silently swallowed —
   no publication, no log, `terminal_reason=complete` — contradicting the typed
   family contract the candidate itself added (**F2**).
3. The new `require_publishable_state` guard raises its exact diagnostic and the
   `200` publishes anyway with live ownership debt, so the documented guarantee
   "a success can never publish while route ownership survives" is not enforced
   (**F3**).

`ACCEPT_WITH_NOTES` is forbidden by this ticket, and F1–F3 are not notes: two of
them falsify contracts written into `codefire` and the source-architecture
document by this very candidate.

### Required next action

Smallest sufficient change, all inside the already-declared write set:

1. In `do_GET`, do not publish a prepared owner when an error arm has selected a
   different response. Either close `read_slot` in each `except` arm before
   falling through (the `do_POST` shape), or gate the publish on
   `read_slot.state_proved`. This closes F2 and F3 together.
2. Either move the gate/lock teardown before the publishable proof so the
   publication point is genuinely last, or narrow the three "no fallible work"
   claims in `codefire`, `source-architecture.md` and `change-playbook.md` to
   "no authority-dependent, callback-capable, or input-proportional work",
   which is what is actually true and provable.
3. Extend the ordering oracle to observe `fcntl.flock`/`os.close` between the
   fence and the publish, and add one regression driving a real `OSError` at
   that boundary plus one driving the publishable-state guard, asserting the
   response is not a `200`.
4. Record in the candidate's evidence that three new tests pass on `38da898`,
   with the reasons established in §7.

Acceptance, when earned, remains scoped to the immutable MR-list candidate. It
would not authorize integration, nor bypass accepted Upload/MR-write bindings,
shared-primitives preflight, Python composition, or its fresh exact review.
