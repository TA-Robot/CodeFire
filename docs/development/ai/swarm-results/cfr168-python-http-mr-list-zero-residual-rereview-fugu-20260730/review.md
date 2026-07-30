# CFR-168 Python HTTP MR-List Zero-Residual Independent Fugu Rereview

Review date: 2026-07-30
Reviewer: Fugu 1.1 Ultra
Review mode: independent adversarial review of the exact committed tree; report-only
Decision: **`REQUEST_CHANGES`**

The candidate closes the prior multi-fault precedence regression, direct authority-file symlink
failure, escrow re-traversal, and the `stable_project_id` stat/reopen defect. It does not satisfy the
batch's zero-residual rule. Five product residuals remain: parent-directory authority failures are
treated as absence and can silently change stale status; filesystem `TimeoutError` is mistaken for a
client transport failure on both GET and POST; cleanup release failures are swallowed after local
ownership is zeroed; mode/link-count changes after the final read's last metadata check pass the
fence; and authority can change after every fence but before response preparation/publication. Two
test-oracle residuals also remain. Every Must/Should/Could residual blocks acceptance in this batch.

This report does **not** authorize integration, issue Close, Install, or Dogfood.

## Exact Review Subject

| Role | Revision |
|---|---|
| Exact reviewed tree / report parent | `38da8988ac8bc1fcdfcb82c62e7aa0d28791500c` |
| Task base | `1695cc86664640cd3df94d0786839fdf3cff196f` |
| Prior reviewed product target | `37f7427d46d2cf3ad66de3c1885ef4c0527cae70` |
| Legacy behavior baseline | `7327e2a8e72f6af84b0eadd8bb6b16233f86882a` |
| Adjacent MR-mutation candidate, not in this tree | `b1c29fc97a9f6190c726b31d465b872558ce2808` |
| Merge base with adjacent MR-mutation candidate | `cab3126f216c595c0d2211174b364e8532a310e2` |

The reviewed candidate is the following linear four-commit series:

| Commit | Parent | Purpose |
|---|---|---|
| `20b5080a550403018c0f6c6bae0037d1ef4c4723` | `1695cc86664640cd3df94d0786839fdf3cff196f` | register repair packet |
| `29c421fc68ef445d8116693789a214fc72c83a95` | `20b5080a550403018c0f6c6bae0037d1ef4c4723` | implementation repair |
| `535515ca30333d0d5c82c070923e2441ae7871a5` | `29c421fc68ef445d8116693789a214fc72c83a95` | focused evidence and mutation tests |
| `38da8988ac8bc1fcdfcb82c62e7aa0d28791500c` | `535515ca30333d0d5c82c070923e2441ae7871a5` | canonical source-doc update |

The exact task-base-to-subject changed paths are:

```text
M  codefire
A  docs/development/ai/swarm-batches/cfr168-python-http-mr-list-zero-residual-repair-20260730.json
M  docs/development/change-playbook.md
M  docs/development/command-source-trace.md
M  docs/development/source-architecture.md
M  docs/development/source-data-model-catalog.md
M  tests/test_python_http_mr_list_decision_work.py
```

Diff size from the task base is `codefire` `+473/-99`, the focused test module `+1082/-6`, the
repair packet JSON `+32`, and the four canonical source documents `+80` total. Every candidate
commit and its complete diff was inspected. The live `make_http_handler` GET/POST dispatch, shared
project gate, MR-list operation, sealed-validation session finalization, response preparation and
publication, and relevant legacy helpers were traced in the exact subject tree.

## Sources Read

- `AGENTS.md`
- `docs/development/ai/swarm-batches/cfr168-python-http-mr-list-zero-residual-repair-20260730.md`
- `docs/development/ai/swarm-batches/cfr168-python-http-mr-list-zero-residual-repair-20260730.json`
- `docs/development/ai/swarm-results/cfr168-python-http-mr-list-review-claude-20260730/review.md`
- `docs/development/code-review-issues/cfr-168.md`
- `docs/development/ai/task-packets/cfr-168-decision-work-budget.md`
- `docs/development/ai/task-packets/json/cfr-168-decision-work-budget.json`
- `adr/ADR-020-decision-equivalent-work-budget.md`
- `docs/development/source-architecture.md`
- `docs/development/command-source-trace.md`
- `docs/development/source-data-model-catalog.md`
- `docs/development/change-playbook.md`
- `codefire`
- `tests/test_python_http_mr_list_decision_work.py`
- relevant legacy MR/request-list cases in `tests/test_codefire_cli.py`
- adjacent MR-mutation commit `b1c29fc97a9f6190c726b31d465b872558ce2808`

The committed contract is explicit: the route must preserve legacy first-error order, publish one
typed filesystem-authority response for every filesystem failure, retain exact zero-debt ownership,
reprove mode/link count/content before publication, poll per bounded work quantum, and close every
Must/Should/Could residual (`source-architecture.md:715`, `command-source-trace.md:221`,
`source-data-model-catalog.md:479`, `change-playbook.md:315`).

## Methods and Commands

All Python commands were run with `PYTHONDONTWRITEBYTECODE=1`. No Cargo, rustc, rustfmt, clippy,
build, install, demo, broad Python discovery, or full-suite command was run. No product, test,
canonical issue, or existing report file was edited.

Primary commands:

```bash
git status --short --branch
git rev-parse HEAD 1695cc8 37f7427 7327e2a b1c29fc
git show --format=fuller --stat 20b5080 29c421f 535515c 38da898
git diff --name-status 1695cc8..38da898
git diff --name-status 37f7427..38da898
git diff --unified=25 37f7427..38da898 -- codefire
git show --unified=40 29c421f -- codefire
git show --unified=5 535515c -- tests/test_python_http_mr_list_decision_work.py
git show --unified=20 38da898
git merge-tree "$(git merge-base 38da898 b1c29fc)" 38da898 b1c29fc
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  tests.test_python_http_mr_list_decision_work -v
PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile \
  codefire tests/test_python_http_mr_list_decision_work.py tests/test_codefire_cli.py
PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/check_source_docs.py
PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/check_project_docs.py
git diff --check
```

Bounded inline Python probes imported the existing focused live-socket harness and changed only its
temporary fixture tree or production namespace monkeypatches. They covered:

1. replacing the `branches/` parent with a directory symlink after fixture creation;
2. injecting one `_sessions_retained` ledger-release failure;
3. injecting session-close cleanup failure on an early validation error;
4. changing branch-record mode or adding a hardlink from inside the final digest call, after the
   fence's last `fstat`;
5. mutating an MR record inside `prepare_json`, after operation and project-gate fences;
6. injecting filesystem-origin `TimeoutError(110, ..., "/storage/authority.json")` from GET and
   POST route handlers;
7. instrumenting descriptor `os.open`, `_read_regular_bounded_at`, `read_json_accounted`, and final
   authority-revalidation calls;
8. measuring the committed 1/8/24/80/200 shared-head counter and poll curves;
9. testing stable pre-existing hardlinks on an MR record, branch record, and project descriptor.

## Findings, Severity Ordered

### MUST-1 — Parent-directory filesystem failure is recorded as absence and silently corrupts status

`read_json_accounted` catches every `OSError` from opening the parent directory and records the
target file as absent (`codefire:1478-1484`). It does not distinguish `ENOENT` from parent symlink,
permission denial, I/O failure, or metadata failure. The final fence repeats the same fail-open
decision: if the capture was marked absent, any parent-open `OSError` is accepted as continued
absence (`codefire:1629-1635`).

The MR-list route then consumes that false absence. `consulted_branch` retains `None` as the branch
record (`codefire:12507-12529`), and `validate_records` changes an MR to `stale` only when the branch
record is truthy (`codefire:12628-12637`). The route therefore publishes an authoritative-looking
`200 OK` with a wrong status instead of the required typed filesystem-authority failure.

Concrete live-route reproduction:

```text
normal branches/ parent:
  HTTP/1.0 200 OK
  statuses = [open, open, stale, approved, open, open, open, open]

branches/ replaced by a symlink to the same directory:
  HTTP/1.0 200 OK
  statuses = [open, open, open, approved, open, open, open, open]
  current = {retained_bytes: 0, temporary_bytes: 0}
  open permits = 0
  terminal = complete
```

The pre-fix route reads the branch through `read_remote_branch_record` after validating each MR
(`codefire@7327e2a:12114-12120`) and preserves the stale result for this fixture. The candidate is
therefore neither typed-fail-closed nor legacy result compatible. Direct file-symlink tests at
`tests/test_python_http_mr_list_decision_work.py:1590` do not cover parent-directory capability
failure.

Required repair: parent-open `FileNotFoundError` may represent absence only when the authoritative
parent capability was proved available and the leaf alone is absent. Every other parent open,
permission, metadata, and read failure must use the typed filesystem-authority family. The final
absent fence must prove the parent and leaf state, not accept an unreadable parent as absence. Add
live GET tests for missing parent, symlinked parent, permission denial, and injected parent-open I/O
failure, including a stale MR whose status would otherwise be downgraded.

### MUST-2 — Type-only transport classification drops filesystem `TimeoutError` on GET and POST

`is_transport_os_error` classifies every `TimeoutError` and every `ConnectionError` as client
transport solely by Python exception class (`codefire:1439-1441`). GET rethrows such an `OSError`
instead of producing the typed body (`codefire:15913-15921`); POST similarly sets `authority=None`
and rethrows (`codefire:16128-16163`). The outer request handler catches `TimeoutError`, logs it, and
closes the connection (`codefire:15792-15794`).

Filesystem operations on remote/network filesystems can raise `TimeoutError`/`ETIMEDOUT`. A bounded
injection from each production dispatcher path produced:

```text
GET handle_http_request_list:
  TimeoutError(110, "Connection timed out", "/storage/authority.json")
  wire bytes = 0, current = {retained_bytes: 0}, permits = 0, terminal = complete

POST handle_http_request_merge:
  TimeoutError(110, "Connection timed out", "/storage/authority.json")
  wire bytes = 0, current = {retained_bytes: 0, temporary_bytes: 0}, permits = 0,
  terminal = complete
```

This violates the batch's required complete typed response for every filesystem read failure. The
existing dispatcher mutation test injects only generic `OSError(ELOOP)` and then separately injects
`ConnectionResetError` (`tests/test_python_http_mr_list_decision_work.py:1646-1696`); it does not
prove origin-sensitive classification. Its `MemoryError` control correctly remains uncaught, and
`RequestDeadlineExceeded` remains in its typed 503 path, so the repair must preserve those outcomes.

Required repair: distinguish transport at the actual socket write/read boundary, not by an exception
class also used by filesystem calls. Route execution must convert every filesystem-origin `OSError`,
including `ETIMEDOUT`, while actual client disconnect/timeout continues to abandon the connection.
Add GET and POST mutation tests for filesystem `TimeoutError`, real socket timeout/disconnect, generic
filesystem `OSError`, `MemoryError`, request-deadline cancellation, and a primary error followed by a
cleanup error.

### MUST-3 — Cleanup failures are swallowed after ownership is locally erased

`_HTTPMergeRequestListOperation._release` reads an amount, sets its owner attribute to zero, calls
the ledger release, and swallows every `BaseException` (`codefire:13061-13069`). If release fails,
the operation has discarded the only retry amount while the ledger still owns the bytes. Session
close also swallows every `BaseException` and then clears the session map
(`codefire:13104-13110`). Similar inner escrow/frontier cleanup blocks swallow release failures
(`codefire:12795-12801`, `codefire:12905-12925`).

Injecting one failure only for the `_sessions_retained` release reproduced a successful published
response with permanent debt:

```text
release amount = 209
HTTP/1.0 200 OK
wire bytes = 1296
current = {retained_bytes: 209, temporary_bytes: 0}
open permits = 0
terminal_reason = None
```

The request-level finalizer notices nonzero current work only after publication and closes the
connection (`codefire:15795-15812`); it cannot recover the lost amount or retract the 200. On an
early validation error, injecting `MemoryError` from session cleanup was attempted twice, swallowed
twice, and the original typed 400 was still published. Preserving a primary diagnostic is correct,
but silently losing cleanup failure and retry ownership is not.

The committed cleanup test exercises budget, deadline, subject-change, and validation failures
without injecting any cleanup failure (`tests/test_python_http_mr_list_decision_work.py:2455-2510`).
It therefore cannot prove the batch requirement that every path reaches zero debt under fallible
cleanup.

Required repair: do not zero or detach retry ownership before a successful ledger release. Cleanup
must be retryable/idempotent, preserve the primary error deterministically, and prevent publication
when success cleanup cannot establish request-header-only/open-zero state. Add independent faults for
every retained/temporary scalar release, session close, projection close, prepared-response close,
and primary-error-plus-cleanup-error ordering.

### MUST-4 — Mode and link-count changes after the last final-read metadata check pass the fence

The authority fence checks regular-file type, link count, and mode before reading
(`codefire:1675-1680`). After the read it checks only device, inode, and size
(`codefire:1694-1698`), then hashes the already-owned buffer (`codefire:1699-1700`). A mode or link
change after that second `fstat` is invisible to the fence.

The live probe changed the consulted branch record from mode `0600` to `0644`, or added a second
hardlink, from inside `_sha256_hexdigest_polling` during final authority revalidation. Both mutations
occurred after the fence's last metadata observation:

```text
late mode change:  mutated=true, final mode=0644, HTTP/1.0 200 OK, terminal=complete
late nlink change: mutated=true, final nlink=2,  HTTP/1.0 200 OK, terminal=complete
```

Both requests ended at zero recorded debt, which makes the stale success look fully valid. This
contradicts the canonical claim that the final fence reproves mode and link count
(`source-data-model-catalog.md:481-490`). Existing same-byte/metadata tests mutate before
`finalize_authority` begins (`tests/test_python_http_mr_list_decision_work.py:1773-1848`) and do not
exercise a mutation inside the final read after its last metadata check.

Stable pre-existing hardlinks were also checked explicitly. MR and branch authority files with
`st_nlink == 2` currently return `200 OK`; the project descriptor is rejected as
`bounded authority file is invalid`. The committed generic authority model records an exact link
count rather than declaring an exactly-one-link rule for MR/branch files, so stable multi-link input
is not classified here as a separate compatibility finding. It does, however, provide an alias from
which the late mutation can occur. The definite defect is that a changed captured link count/mode is
not reproved through the end of the final read.

Required repair: repeat all captured metadata checks after the final byte read and after any
interruptible digest work, or otherwise bind the digest and terminal metadata observation into one
safe protocol. Add mutations for mode, hardlink creation/removal, symlink replacement, size, identity,
and content at each boundary: before open, during read, after EOF probe, during digest, and immediately
before publication.

### SHOULD-1 — Authority can change after all fences but before response preparation/publication

The operation's authority and sealed-session fences finish before returning the payload
(`codefire:13133-13138`). The shared project gate performs its last root/locks/descriptor check while
exiting `handle_http_request_list` (`codefire:2454-2475`). Only afterward does `do_GET` call
`send_json` (`codefire:15922`), which prepares and publishes the response
(`codefire:15647-15685`). No authority or session owner survives to a final pre-write check.

Mutating an MR authority record from inside `prepare_json`, after every operation and project-gate
fence returned, produced:

```text
mutation occurred = true
authority bytes changed = true
HTTP/1.0 200 OK
merge_requests = 8
terminal = complete
current = {retained_bytes: 0, temporary_bytes: 0}
```

The mutation was not observed to enter the already-built four-field projection. That limits the
immediate semantic effect, but it does not meet the explicit review requirement of no post-fence
publication gap, and the canonical documentation says the final fence is "before publication"
(`source-architecture.md:727-731`). Under this batch's zero-residual rule, this Should remains a
blocker unless the disk-independence/publication-point contract is formally narrowed and proven.

Required repair: prepare/encode the complete response under its accounted owner first, then run the
last authority/session/project fence immediately before the first response byte, with no fallible or
authority-dependent work between fence and publication. Add mutation hooks after projection, after
encoding, after every current finalizer, and immediately before `prepared.publish`.

### COULD-1 — The project-descriptor "read once" oracle measures only one helper

The repaired `stable_project_id` path does perform one accounted read from owned bytes and memoizes
the resulting stable ID (`codefire:12531-12589`), so the prior stat-then-reopen implementation defect
is closed at that helper boundary. The test named
`test_project_descriptor_is_read_once_from_the_bytes_it_charges` records only calls to
`read_json_accounted` (`tests/test_python_http_mr_list_decision_work.py:590-601`,
`tests/test_python_http_mr_list_decision_work.py:2061-2072`).

Whole-route instrumentation observed:

```text
descriptor-related os.open calls                  = 5
_read_regular_bounded_at("remote_project.json")  = 3
read_json_accounted(descriptor)                   = 1
final authority revalidation                      = 1
```

The three bounded reads belong to project-gate acquisition/recheck/release
(`codefire:2390`, `codefire:2408-2418`, `codefire:2465-2475`). They may be independently required,
but the current test cannot support its unqualified "never reread" docstring or a whole-route
one-physical-read claim. The batch wording scopes the repair to `stable_project_id`; the oracle and
documentation must state that scope exactly or instrument every physical descriptor read.

Required repair: rename/narrow the assertion to the `stable_project_id` accounted-read boundary, or
replace it with an oracle that records all descriptor opens/reads and asserts the intended exact
route protocol. Under the zero-residual rule, an overclaimed test is not accepted as evidence.

### COULD-2 — Poll mutation proves phase-helper ownership, not necessity of each production poll

The candidate materially improves polling: shared-head poll counts are affine, the escrow traversal
is performed once, and a long traversal cancels at many deterministic boundaries. However,
`test_removing_a_production_poll_breaks_its_phase_cancellation` replaces an entire phase helper such
as `_validation_poll` or `_revalidation_poll` with a no-op
(`tests/test_python_http_mr_list_decision_work.py:2388-2449`). Every call site using that helper is
removed together.

That mutation proves that each phase has polls and that the named helper owns them. It does not prove
that each individual production poll is necessary, nor would it fail if two adjacent polls were
redundant. This is weaker than the packet's required "prove each production poll is required by
silencing it" evidence.

Required repair: identify production poll sites or bounded-work-quantum boundaries independently and
mutate them one at a time. Each removal must either lose cancellation at a unique admitted-work
boundary or violate the documented maximum quantum; redundant sites should be removed.

## Prior-Finding Disposition

| Prior finding | Disposition at `38da898` | Evidence / residual |
|---|---|---|
| MUST-1: whole-snapshot capture changes multi-fault first-error precedence | **Closed** | Deferred per-record capture faults are retained at `codefire:12418-12424` and raised in filename order at `codefire:12595-12608`. Real-baseline, both-order fixtures and eager-capture mutation are present. |
| MUST-2: symlinked authority can yield zero-byte reply | **Partially closed; still open overall** | Direct MR/branch/descriptor symlinks receive typed responses. Parent-directory `OSError` still fails open (MUST-1), and filesystem `TimeoutError` is still abandoned (MUST-2). |
| SHOULD-1: escrow set/digest recomputed per MR | **Closed** | Per-operation memo key is absolute pinned object root plus NUL plus `source_head` (`codefire:12783-12806`); ownership is retained/released by the operation; session finalization fences reused documents. |
| SHOULD-2: evidence overclaims precedence and omits symlink authority | **Improved but open** | Multi-fault and direct symlink coverage were added, but parent authority, cleanup failure, late metadata, publication gap, descriptor scope, and individual-poll oracles remain. |
| COULD-1: `stable_project_id` stats then reopens | **Implementation closed; oracle residual open** | One accounted owned-byte read now replaces the helper's stat/reopen. Whole-route physical read count is not what the current test claims (COULD-1). |
| COULD-2: excessive deadline polling | **Implementation improved; proof incomplete** | Poll density is affine and traversal remains interruptible, but the mutation removes whole helpers, not individual polls (COULD-2). |

## Requirement and Residual Matrix

| Required property | Result | Independent disposition |
|---|---|---|
| Legacy multi-fault precedence under deferred per-record authority faults | **PASS** | Deferred capture preserves filename-order validation; eager-capture mutation is non-vacuous. |
| Typed non-empty filesystem handling | **FAIL** | Parent-directory failures become absence; filesystem `TimeoutError` is treated as transport. |
| Do not swallow transport failure, `MemoryError`, or cancellation | **PASS with classifier defect** | Generic `MemoryError`, actual connection reset, and request deadline remain outside the filesystem arm; type-only `TimeoutError` cannot distinguish origin. |
| Preserve primary error while completing cleanup | **FAIL** | Cleanup `BaseException` is swallowed and retry ownership can be erased. |
| Absent/appeared final authority revalidation | **PARTIAL** | Leaf absence/appearance test passes; unreadable parent is accepted as continued absence. |
| Same-metadata byte replacement | **PASS at tested boundary** | Digest catches same-inode/size/mtime byte replacement before final fence begins. |
| Symlink authority | **PASS for leaf; FAIL for parent** | Direct file symlinks typed-fail; symlinked parent directory can produce 200 with wrong status. |
| Hardlink/mode/nlink | **FAIL** | Stable MR/branch hardlinks are accepted by the current exact-link-count model; mode/nlink changes after the last `fstat` are missed. |
| Deterministic authority/order and no later fault precedence | **PASS except filesystem fail-open** | Filename order and deferred capture hold; false absence changes status rather than error order. |
| No post-fence publication gap | **FAIL** | Authority mutation in `prepare_json` after all fences still publishes 200. |
| One descriptor read from bytes actually charged | **PASS only at `stable_project_id` scope** | Helper defect is fixed; whole-route test wording/oracle overclaims and records only one helper. |
| Escrow memo key and no cross-request reuse | **PASS** | Memo is operation-local; key binds absolute object root and source head; next request constructs a new operation. |
| Escrow ownership/accounting/lifetime | **PASS absent cleanup fault** | Ordered IDs and memo entry are retained and released by the operation; cleanup fault handling remains MUST-3. |
| Escrow final revalidation | **PASS** | Reused session documents are finalized once per pinned root after projection. |
| Deadline density and interruptibility | **PASS implementation / incomplete mutation proof** | Affine poll curve and long traversal cancellation pass; individual poll necessity remains COULD-2. |
| Exact phase/diagnostic/status/order/response bytes | **FAIL overall** | Main success/parity cases pass, but parent failure changes stale output and filesystem timeout yields no response. |
| Cleanup zero debt and zero permits | **FAIL** | Injected release failure leaves 209 retained bytes and no terminal reason after 200 publication. |
| Asymptotic/counter claims | **PASS for escrow claim** | Object traversal/validation counters remain constant; only per-record work grows. |
| Production-path evidence | **PASS for exercised cases** | Tests drive the live handler over sockets with the handler ledger/deadline. Missing adversarial boundaries are listed above. |
| Mutation-test non-vacuity | **PARTIAL** | Eager capture and memo removal are strong; descriptor and poll mutations do not prove their full claim. |
| Shared GET/POST isolation | **PARTIAL** | Flags/reset and direct symlink cases pass; shared timeout classifier and adjacent-lane conflicts remain. |

## Counter and Complexity Evidence

The required shared-head cardinalities were independently measured on the exact subject. One
sealed-validation session was used and no path recorded by `read_json_accounted` was duplicated.

| MRs | accounted reads | object records | memo misses | memo hits | unique nodes | semantic validations | graph edges | bytes hashed | fence records | polls |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 3 | 22 | 22 | 90 | 22 | 31 | 142 | 45,496 | 25 | 14,135 |
| 8 | 10 | 22 | 22 | 104 | 22 | 31 | 142 | 63,101 | 32 | 16,830 |
| 24 | 26 | 22 | 22 | 136 | 22 | 31 | 142 | 103,341 | 48 | 22,990 |
| 80 | 82 | 22 | 22 | 248 | 22 | 31 | 142 | 244,181 | 104 | 44,550 |
| 200 | 202 | 22 | 22 | 488 | 22 | 31 | 142 | 545,981 | 224 | 90,750 |

`object_records`, `memo_misses`, `unique_nodes_discovered`, `semantic_validations`, graph nodes, and
graph edges stay constant. Accounted authority reads, final authority fence records, subject hashing,
projection work, and polls grow affinely per MR. Removing the escrow memo restores a larger linear
traversal/hash slope, so the memo/counter proof itself is non-vacuous.

## Test Oracle and Vacuity Audit

| Oracle | Classification |
|---|---|
| Multi-fault precedence vs real legacy baseline, both orders | **Non-vacuous**; eager `_defer` mutation restores the prior divergence. |
| Direct symlinked authority response | **Non-vacuous for leaf files**, but not parent-directory capability failure. |
| Shared-route escaping generic `OSError` | **Non-vacuous for generic filesystem errors**, but misses filesystem `TimeoutError` origin ambiguity. |
| Escrow memo constancy | **Non-vacuous**; memo removal increases edges, revisits, hashes, and polls. |
| One descriptor read | **Overclaimed**; records only `read_json_accounted`, not all descriptor opens/reads. |
| Every production poll is required | **Overclaimed**; removes an entire phase helper, not individual sites. |
| Cleanup zero debt | **Vacuous for cleanup faults**; no release/close fault is injected. |
| Final authority fence | **Incomplete**; mutations occur before fence entry, not after the last metadata observation or after all fences. |

## Unexplained Early Focused-Module Failure

Classification: **unexplained, non-reproduced evidence**.

- The implementer reported one unidentified failure in an early output-truncated focused-module run,
  followed by 12 clean full focused-module reruns and 14 clean targeted stress reruns.
- No failing test name, traceback, wire capture, ledger snapshot, seed, temporary fixture, or other
  artifact is committed or present in the worktree. Commit messages and notes contain no additional
  evidence.
- The supplied prior review state recorded three clean 33/33 focused runs in `74.792s`, `72.382s`,
  and `73.197s`.
- This rereview ran the exact module once more: **33/33 passed in `70.079s`**.
- The module has real socket server threads, subprocess-based class fixture setup, 30-second socket
  and join bounds, and temporary filesystem trees. Deadline tests use deterministic injected clocks,
  there are no wall-clock sleeps, and no random source controls the assertions. OS scheduling or
  socket/filesystem timing is a plausible class of nondeterminism, but there is no evidence tying the
  unidentified failure to it.

Therefore the early observation is not called a proven flake, and the later passes are not used to
erase it or to claim a definitive full pass. A future occurrence must preserve full output and the
temporary fixture or at least the failing test ID, traceback, wire bytes, observer snapshot, and
phase/poll trace.

## Adjacent GET/POST and MR-Mutation Overlap

The adjacent MR-mutation candidate is not part of the reviewed tree and is not accepted here.
`git merge-tree` reports `codefire` as changed in both and emits conflict markers in at least two
large regions.

| Overlap | Evidence | Required integration handling |
|---|---|---|
| MR-list/MR-mutation authority helpers | The adjacent lane inserts `_PinnedAuthorityChanged`, `_read_pinned_authority_bytes`, and `AccountedHTTPMergeRequestFence` immediately after the MR-list handler region where this candidate adds the operation and list handler. | Reconcile one authority/fence model; do not keep duplicate error taxonomies or inconsistent absence/link rules. |
| Shared `do_POST` dispatcher and response slot | Both lanes modify the `BaseException`/filesystem arm, mutation-slot cleanup, prepared-response ownership, and phase flags. | Preserve origin-sensitive filesystem handling, actual transport abandonment, primary-error ordering, and retryable cleanup in one dispatcher. |
| Shared `do_GET` dispatcher | This candidate adds the generic filesystem arm and MR-list phase flag; adjacent read-route owners depend on the same handler state. | Re-run mixed GET/POST isolation and every adjacent route's filesystem fault after manual conflict resolution. |
| Project gate and descriptor authority | MR-list reads/fences the descriptor while mutation owners also pin mutation authority and project state. | Define exact read/fence ordering and one final pre-publication point across the composed tree. |

Because the merge base is `cab3126f216c595c0d2211174b364e8532a310e2`, neither candidate contains
the other's shared-dispatcher choices. Integration cannot be inferred from either isolated focused
suite.

## Validation Results

| Command / check | Result |
|---|---|
| Exact revision, parent chain, changed paths, and candidate diffs | **PASS**; subject is exactly `38da8988ac8bc1fcdfcb82c62e7aa0d28791500c` before report creation. |
| `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tests.test_python_http_mr_list_decision_work -v` | **PASS — 33 tests, 70.079s** |
| Focused adversarial inline probes | **FAIL contract** with the exact reproductions in MUST-1 through SHOULD-1. |
| `PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile codefire tests/test_python_http_mr_list_decision_work.py tests/test_codefire_cli.py` | **PASS** |
| `PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/check_source_docs.py` | **PASS — 44 modules, 34 commands, 12 object kinds** |
| `PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/check_project_docs.py` | **PASS — 543 Markdown files inventoried, including this report** |
| Adjacent `git merge-tree` | **CONFLICT** in shared `codefire` authority/dispatcher regions |
| Broad/full Python discovery | **NOT RUN — prohibited by rereview constraint** |
| Cargo/rustc/rustfmt/clippy/build/install/demo | **NOT RUN — prohibited by rereview constraint** |

## Decision and Required Next Action

**Decision: `REQUEST_CHANGES`.**

Before another zero-residual review, repair every finding in this report:

1. distinguish true leaf absence from unavailable/symlinked/denied parent authority and make stale
   status fail closed rather than silently downgrade;
2. classify filesystem versus client transport failures at their actual boundary, including
   filesystem `TimeoutError`, on both GET and POST;
3. make cleanup release/close retryable without erasing ownership, swallowing `BaseException`, or
   publishing success with residual debt;
4. reprove mode and link count after final read/digest work and cover hardlink mutations at every
   boundary;
5. move the final authority/session/project fence to the immediate pre-publication boundary;
6. replace the descriptor and poll overclaims with exact, non-vacuous oracles;
7. manually reconcile the shared `codefire` conflicts with
   `b1c29fc97a9f6190c726b31d465b872558ce2808` and rerun the composed GET/POST isolation, typed
   filesystem, cleanup, fence, and mutation evidence.

Do not integrate, Close CFR-168, Install, or Dogfood on the basis of this candidate or report.
