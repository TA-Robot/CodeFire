# CFR-168 Python HTTP MR Mutation Zero-Residual Rereview (Fugu 1.1 Ultra)

Review date: 2026-07-30

Reviewer: Fugu 1.1 Ultra

Review mode: independent adversarial, exact committed tree, report only
Decision: **`REQUEST_CHANGES`**

This report does not authorize Integration, Close, Install, or Dogfood.

## Exact subject

All production and test line references in this report refer to the committed candidate tree at
`6dd4527f23135388b53fc5c6d637c4105cc5d659`, before this report-only commit.

| Role | Exact revision |
|---|---|
| Task base | `3afef8fda0978a73ea798d3c6fa6ecaf741117cb` |
| Prior rejected target | `21fe75175e997cc7c6f8c3ef2a7558144d850f94` |
| Product repair commit | `b1c29fc97a9f6190c726b31d465b872558ce2808` |
| Review candidate, including focused tests | `6dd4527f23135388b53fc5c6d637c4105cc5d659` |

The ancestry is linear:

```text
21fe75175e997cc7c6f8c3ef2a7558144d850f94
  -> a4b21fdec3dafd344396447c1c02a52e767c16d6 (prior review report)
     -> dd23a63db20f8ad7c08617c558dff999add8cacc (repair task registration)
        -> 3afef8fda0978a73ea798d3c6fa6ecaf741117cb
           -> b1c29fc97a9f6190c726b31d465b872558ce2808
              -> 6dd4527f23135388b53fc5c6d637c4105cc5d659
```

The exact product/test delta `3afef8f..6dd4527` is limited to `codefire` and
`tests/test_python_http_mr_mutation_decision_work.py` (`1516` insertions, `295` deletions).
The broader repair delta `21fe751..6dd4527` also contains the registered task packet and prior
independent report, which were read as review inputs and were not modified.

## Method

The review used the following method.

1. Read `AGENTS.md`, the Markdown and JSON zero-residual repair packets, the prior rejected review,
   ADR-020, CFR-168, `docs/08_object_store.md`, `docs/10_merge_remote_server.md`, and the four
   canonical source documents.
2. Compared the exact diffs `21fe751..6dd4527` and `3afef8f..6dd4527`, then read every changed
   production and test path in the committed candidate.
3. Traced the real socket entry point through `make_http_handler`, `do_POST`, the shared response
   owner, all three MR mutation handlers, the lane, sealed sessions, signature and ancestry owners,
   authority gates, publication reducer, response handoff, observer, and cleanup.
4. Did not accept commit messages or test comments as evidence. Every claimed owner was followed to
   its physical read, allocation, canonicalization, hash, write, cleanup, and retry call site.
5. Used provider-independent local filesystem fixtures and failure injection for the accounting,
   object reread, prepublication seal, partial-copy, cleanup retry, and replay-nonce counterexamples.
6. Reused the completed focused validation state described in the Commands section. No Cargo,
   rustc, build, install, demo, broad/full Python suite, or target artifact was run or created.

## Summary

The repair is substantial but not zero-residual. It removes the nested `local_v1` validation
ledgers, adds one `SealedCommitValidationSession` per store, moves deterministic legacy publication
preflight ahead of escrow materialization, adds typed grow-tamper evidence, and makes apply
admission goal-directed. Those are real closures.

The exact request, however, still does not own all decision-equivalent work. Authority and recovery
JSON, project identity, publication canonicalization/hashing, projection/copy lifetimes, and
signature/ancestry commit rereads escape the claimed single-session accounting. The ledger is never
prepublication-sealed. Escrow copying remains a separate, partially visible effect outside the
publication reducer. Cleanup can lose its retry scalar and suppress the already-durable response.
Replay nonce publication is durable but not atomic with replay confirmation. The required boundary
and mutation matrix is also incomplete, and canonical documentation contradicts the now-wired MR
response owner.

Under the packet's zero-residual rule, all of these require repair.

## Findings

### F1 — Blocking: request accounting still excludes authority, identity, publication, and projection work

ADR-020 requires one mutable ledger for the complete request and requires charging before the
corresponding read, allocation, parse, validation, hash, or output append
(`adr/ADR-020-decision-equivalent-work-budget.md:50`). The repair packet explicitly includes policy,
nonce, every authority JSON, descriptor, projection, canonical encoding, publication, and cleanup
(`docs/development/ai/swarm-batches/cfr168-python-http-mr-mutation-zero-residual-repair-20260730.md:16`).

The candidate accounts the policy and primary nonce-cache read, but the live route still performs
the following work outside the handler ledger and deadline:

- Project descriptors are opened, parsed, and projected by `remote_project_stable_id` without a
  ledger or deadline (`codefire:2195`). The helper has a local 16 KiB bound, but it is not request
  charged. Create reads the semantic source and target IDs before the ordered gates
  (`codefire:13076`, `codefire:13077`). `ordered_remote_merge_gates` later compares the gated IDs to
  a second pair that it read itself (`codefire:2313`), not to the IDs already embedded in the
  request semantic. A descriptor replacement between those reads is therefore not a final fence on
  the semantic identity.
- Branch and MR authorities still enter through compatibility readers. `http_remote_branch` calls
  `read_remote_branch_record` (`codefire:11644`), and review/apply call
  `read_remote_merge_request_path` (`codefire:13303`, `codefire:13598`). The latter is a plain
  `read_json` (`codefire:2363`), which performs an unbounded `Path.read_text` plus `json.loads`
  (`codefire:1429`). The same plain reader is used again inside the review/apply gates
  (`codefire:13329`, `codefire:13764`) and for apply generation authority (`codefire:13770`).
- Replay/recovery reads pending and receipt authority through plain `read_json` in
  `peek_remote_publication_recovery_result` (`codefire:12231`) and again inside
  `execute_remote_json_publication` (`codefire:3294`, `codefire:3351`, `codefire:3361`). These reads
  are not represented in the request counters and are not bounded by the request deadline.
- Publication preimage validation reopens every authority with plain `read_json`
  (`codefire:2926`, `codefire:3076`). Request hashing, intent hashing, and write-set hashing perform
  complete canonical encodings without a request reservation or iterative deadline poll
  (`codefire:2412`, `codefire:3082`, `codefire:3113`). Publication preflight then constructs
  `authority_bytes`, `before_bytes`, `after_bytes`, three parsed clones per write, hashes, a write
  plan, and a cloned result outside a ledger owner (`codefire:3182`, `codefire:3198`,
  `codefire:3208`, `codefire:3273`).
- The nonce postimage is fully constructed before its retained ownership is acquired
  (`codefire:12396`, `codefire:12409`). The preimage is owned, but the new dict/list/string graph is
  already allocated when the charge happens.
- Object-set projections are appended as new record dictionaries while only a 64-byte temporary
  slot is charged (`codefire:12637`, `codefire:12641`). The slots are released before the returned
  projection is hashed, so the projection outlives its accounting owner. Escrow copy planning has
  the same shape: `object_record_core` creates a retained dictionary (`codefire:12752`), but only a
  64-byte slot is retained for the plan entry (`codefire:12771`). `write_json` later canonicalizes
  that dictionary in the effect window with no reserved temporary or retained owner.

Concrete counterexample: a provider-independent fixture installed a structurally valid,
status-inadmissible MR record of `2,098,457` bytes and sent a real socket apply request. The route
fully read and parsed the record, returned `409`, reported `aggregate_bytes=0`, and terminated the
ledger as `complete`. The request therefore performed over 2 MiB of decision work that no HTTP
limit or deadline observed.

The focused recorder does not detect this. `unowned_json_reads` only flags policy, nonce-cache, and
`/objects/` paths (`tests/test_python_http_mr_mutation_decision_work.py:461`). MR, branch,
generation, descriptor, pending, receipt, transaction, and publication-preimage reads evade the
test. The test also records which helper was called, not whether an accounted helper charged before
allocation or retained the resulting object for its whole lifetime.

Required repair: route every live authority, descriptor, recovery, receipt, pending, generation,
and publication read through a request-owned bounded reader; carry exact raw/parsed/projection/hash
ownership until its last consumer; charge before construction; add deadline polls to iterative
canonicalization; and final-fence the project IDs embedded in the semantic against the identities
held by the acquired gates.

### F2 — Blocking: apply rereads sealed commits outside the one sealed session per store

The packet requires one sealed session per store and forbids physically rereading a closure through
another helper. `AccountedRemoteCommitBorrow` instead reopens object files through
`read_json_accounted` (`codefire:11968`, `codefire:12016`). Signature enforcement creates one of
these borrows (`codefire:12541`, `codefire:12563`), and apply ancestry creates another
(`codefire:12814`). These are request-ledger reads, but they are not borrows from the retained
`SealedCommitValidationSession` snapshot and they do not increment `object_records`.

The focused test explicitly accepts the mismatch. It asserts that apply `object_records` equals the
escrow closure size, while `record_reads` additionally includes one reopened record per ancestry
edge (`tests/test_python_http_mr_mutation_decision_work.py:660`,
`tests/test_python_http_mr_mutation_decision_work.py:667`). That contradicts both the production
docstring claiming every physically read record is charged exactly once
(`tests/test_python_http_mr_mutation_decision_work.py:608`) and the packet's one-session requirement.

Concrete exact-limit counterexample: with `object_records` limited to the measured exact value `10`,
apply returned `200`; the ledger charged `object_records=10`, while instrumentation observed
`record_reads=13` and three additional physical object-file reads. The exact object-record limit is
therefore not the physical-read limit.

With a seven-record base closure, the measured 1x/3x/10x curves are:

| Route | Scale | `object_records` | `record_reads` | `graph_nodes` |
|---|---:|---:|---:|---:|
| create | 1x | 15 | 15 | — |
| create | 3x | 17 | 17 | — |
| create | 10x | 24 | 24 | — |
| review | 1x | 8 | 8 | — |
| review | 3x | 10 | 10 | — |
| review | 10x | 17 | 17 | — |
| apply | 1x | 8 | 9 | 12 |
| apply | 3x | 10 | 13 | 18 |
| apply | 10x | 17 | 27 | 39 |

The growing apply gap is not a stable single-read curve. It proves that a second loader, not the
sealed-session memo, governs signature/ancestry consumption.

Required repair: expose purpose-specific, single-use commit handles from the already retained sealed
session, permit access only to validated documents, bind their fingerprint to the session snapshot,
and make signature and ancestry consume those handles. No live MR mutation may reopen an object
record after the session has retained it, and `object_records` must equal physical object-record
reads at exact, minus-one, and plus-one limits.

### F3 — Blocking evidence gap: exact/minus/plus and mutation non-vacuity remain incomplete

The production-path tests are real socket tests through `make_http_handler`, and the focused module
does fail against both historical trees. That establishes broad non-vacuity, but it does not satisfy
the exact zero-residual evidence contract.

`run_boundary` advertises exact/minus-one/plus-one (`tests/test_python_http_mr_mutation_decision_work.py:1005`),
but almost every caller passes only `(0, -1)`. Only response-body bytes exercise `+1`
(`tests/test_python_http_mr_mutation_decision_work.py:1065`). Header bytes, retained bytes,
temporary bytes, `object_records`, `aggregate_bytes`, graph nodes/edges, depth, frontier, snapshot
revalidation records/bytes, and signed authorization/nonce bytes cover exact and minus-one only
(`tests/test_python_http_mr_mutation_decision_work.py:1082`,
`tests/test_python_http_mr_mutation_decision_work.py:1092`,
`tests/test_python_http_mr_mutation_decision_work.py:1102`,
`tests/test_python_http_mr_mutation_decision_work.py:1113`,
`tests/test_python_http_mr_mutation_decision_work.py:1122`,
`tests/test_python_http_mr_mutation_decision_work.py:1209`,
`tests/test_python_http_mr_mutation_decision_work.py:1225`). No boundary exists for descriptor,
MR/branch/generation, recovery/pending/receipt, publication canonicalization/hash, projection, copy
postimage, or cleanup-retry owners because those owners do not yet exist.

The required mutation matrix is also only partially killing:

| Required mutation class | Result | Review |
|---|---|---|
| Restore nested sealed-validation ledger | Killed | Good |
| Restore hidden collection/copy helper | Killed for both variants | Good |
| Remove an iterative deadline poll | Killed | Good |
| Move deterministic publication preflight after legacy materialization | Killed | Good |
| Drop the replay nonce entirely | Killed | Good, but only checks eventual persistence |
| Omit the policy lifetime owner while leaving the read path | **Survives** | Recorder sees the helper, not lifetime accounting |
| Omit the nonce-cache/postimage lifetime owner while leaving the read path | **Survives** | Same gap |

There is no reversal that kills the cleanup scalar loss, sealed-session borrow reread, missing
ledger prepublication seal, non-atomic replay nonce/reducer window, or mid-copy recovery defect. The
current object-read assertion would defend F2 rather than kill it.

Required repair: add plus-one coverage for every charged dimension and each newly introduced owner;
make policy and nonce owner omissions fail; add reversals for F2, F4, F5, and F6; and keep the
historical failures against both `21fe751` and `e2a218e`.

### F4 — Blocking: the ledger is not prepublication-sealed, and escrow copy is outside reducer recovery

`DecisionWorkLedger.seal_prepublication` exists and is the transition that forbids ordinary
post-effect `apply` (`codefire:862`). `_HTTPMergeRequestLane.start_effect` never calls it; it only
seals and marks the response slot (`codefire:12829`). A direct probe after effect start observed
`ledger._prepublication_sealed == False`; an ordinary `ledger.apply(...)` still succeeded, and the
route returned `200`. The response owner is sealed, but the decision-work ledger is not.

Create and legacy review then perform escrow writes after `start_effect` and before the publication
reducer:

```text
create: start_effect -> execute_copy -> execute_remote_json_publication
        codefire:13193  codefire:13197  codefire:13198

review: start_effect -> execute_copy -> execute_remote_json_publication
        codefire:13522  codefire:13524  codefire:13527
```

`execute_copy` writes each planned object directly with `write_json` (`codefire:12707`). It has no
transaction marker, progress record, rollback, or durable reducer state. The publication reducer
does not create `pending_transaction.json` or a transaction directory until after copy completes
(`codefire:3371`).

Concrete counterexample: a legacy review needed four missing escrow records. Injecting failure on
the second `write_json` returned `503 retry_same_request`, left one of the four missing objects in the
target object store, left MR and branch authority unchanged, and left neither a pending marker nor a
transaction directory. No reducer can distinguish, roll back, or prove that partial object effect.

This is not cured by moving deterministic preflight earlier. That prior precedence defect is closed,
but the first visibility point is still a multi-file effect without durable recovery ownership.

Required repair: call `seal_prepublication` exactly once on the handler ledger after every fallible
raw revalidation and before the first effect; reserve permits for all legal post-seal work; and move
escrow materialization into the same durable reducer protocol as authority publication, or into a
separately durable staged transaction with exact progress, rollback/retry ownership, and response
recovery. No partial object effect may exist without a durable record that the exact retry can
resume.

### F5 — Blocking: cleanup loses retry ownership and can suppress a durable response

`_HTTPMergeRequestLane.close` clears each scalar before attempting the corresponding ledger release
(`codefire:12890`, `codefire:12899`). If `release` fails, `_closed` remains false, but the amount is
already zero. A second `close` therefore marks the lane closed without retrying the release.

Direct counterexample: an injected release failure on a lane retaining `37` bytes left ledger debt
`37`. The second `close()` completed and marked the lane closed while the debt remained.

Live counterexample: after MR authority had durably published, an injected lane cleanup release
failure prevented any HTTP response from reaching the client. The committed MR authority remained,
one observer snapshot had `terminal_reason=None`, and retained debt `2` remained. The outer handler's
`BaseException` cleanup suppresses secondary slot/response cleanup errors (`codefire:16543`), but it
cannot reconstruct the scalar already discarded by the lane.

This violates all four required postconditions: preserve the primary typed/durable outcome, return
the selected success or durable-retry bytes, emit one terminal observation, and finish with zero
debt and zero permits.

Required repair: retain every scalar until its release succeeds; make close retry the exact failed
owner without double-dropping payloads; preserve and chain the first error; keep the selected
prepared response recoverable after publication; and add release-fault tests for every lane owner,
the sealed session, fence, borrows, response selection, observer, and handler cleanup.

### F6 — Medium: replay nonce publication is not atomic with replay confirmation

The original replay bug silently discarded a fresh accepted nonce. The candidate now writes it, but
`record_nonce_lease` writes the nonce cache directly (`codefire:12461`) before the replay reducer is
confirmed. Create, review, and apply recovery call it before `execute_remote_json_publication`
(`codefire:13194`, `codefire:13525`, `codefire:13716`).

Concrete counterexample:

1. The original signed request returned `200` and created a receipt.
2. A replay with a fresh valid nonce reached `record_nonce_lease`; an injected reducer failure then
   returned `503 retry_same_request`.
3. The fresh nonce was already durable.
4. Retrying the exact signed request returned `400 nonce has already been used`.
5. Only a third nonce could recover the original recorded `200` result.

The response's retry instruction is false in this window. The nonce effect and replay confirmation
are two independent durable operations.

Required repair: publish the accepted replay nonce atomically with a durable replay/recovery
transaction or receipt transition, or reject the fresh nonce before selecting the recorded result.
Every `503 retry_same_request` must remain exactly retryable with the same signed bytes.

### F7 — Low, mandatory under zero-residual rules: canonical source documentation is stale

The canonical data-model catalog still describes the Python response owner as upload-only and says
other routes/deadline coverage are not implied (`docs/development/source-data-model-catalog.md:89`,
`docs/development/source-data-model-catalog.md:91`). The candidate now wires the same response owner
into all three MR mutation routes through the real handler.

`check_source_docs.py` and `check_project_docs.py` pass because they validate structure, not this
semantic contradiction. The task requires canonical source docs to change with a major owner and
route boundary. Under the zero-residual rule, this Low finding still requires repair.

## Prior finding disposition

| Prior item | Candidate disposition | Exact review |
|---|---|---|
| Prior F1: hidden nested ledger and unaccounted validation/authorization/materialization | **Not closed** | Nested `local_v1` and the named compatibility collection/copy helpers are removed, but F1 and F2 leave material authority/publication/projection work and sealed-record rereads outside the claimed owner/session. |
| Prior F2: boundary and non-vacuity evidence | **Not closed** | Coverage is much stronger and historical failures are real, but most dimensions omit plus-one, two owner-omission mutations survive, and the object-read test accepts the physical-read mismatch. |
| Prior F3: deterministic legacy preflight after escrow materialization | **Closed narrowly; zero-residual effect requirement not closed** | Deterministic publication/sealed preflight now precedes materialization and preserves the original 4xx. Mid-copy failure still leaves untracked partial escrow after effect start (F4). |
| Prior F4: replay accepts and silently discards a fresh nonce | **Closed narrowly; replaced by F6** | The nonce is now persisted and cannot be reused without bound, but persistence is not atomic with replay confirmation, so exact durable retry is broken. |
| Prior F5: same-path grow tamper loses typed fence evidence | **Closed** | `_PinnedAuthorityChanged` is translated to `merge_request_fence_changed` with expected/actual evidence, and focused grow/tamper tests pass. |
| Prior F6: GC nonce timing | **Correctly predecessor-gated** | `POST .../gc` remains on the separate legacy path. The candidate does not claim uniform POST nonce timing. Do not integrate or close the successor on this report. |
| Goal-directed apply admission | **Closed/accepted** | Shape/version corruption remains before status/staleness; status and stale rejections occur before full sealed graph, signature history, and ancestry work (`codefire:13601`, `codefire:13613`). This is the deliberate required precedence change. |
| Cleanup, terminal observation, zero debt/permits | **Not closed** | F5 gives both direct and live counterexamples. |

## Ownership audit

| Work/lifetime | Required owner | Candidate state | Disposition |
|---|---|---|---|
| Handler request, deadline, response slot | One exact `http_v1` ledger, one deadline, one prepared response owner | Correctly threaded through all three route arms | Pass locally |
| Sealed validation per store | One request-bound retained session reused through final revalidation | One session is created, but signature/ancestry reopen records | Fail: F2 |
| Policy read and parsed policy | Request ledger from pre-read through last permission/signature/branch use | Primary read is bounded/accounted and reused | Partial: post-allocation/lifetime mutation coverage is insufficient |
| Nonce preimage/postimage | Request ledger, charged before materialization, atomically published | Preimage is accounted; postimage is built before ownership; replay write is separate | Fail: F1, F6 |
| Branch, MR, generation authority | Request-owned bounded raw/parsed owner and final fence | Compatibility `read_json` readers remain | Fail: F1 |
| Stable project descriptor/identity | Request-owned bounded descriptor and equality to gated final identity | Locally bounded but uncharged; semantic IDs are not compared to final gated IDs | Fail: F1 |
| Object collection projection | Retained projection owner through digest/materialization | 64-byte temporary slots are released before digest | Fail: F1 |
| Escrow copy plan | Exact retained record/postimage owner through durable reducer | 64-byte slots represent copied dicts; direct writes precede reducer | Fail: F1, F4 |
| Signature history and ancestry commits | Purpose borrow from sealed-session snapshot | `AccountedRemoteCommitBorrow` reopens files | Fail: F2 |
| Publication hashes, preimages, cloned write/result plan | Request ledger plus deadline; no ordinary work after seal | Canonicalization, hashing, parsed clones, and plain rereads are outside the ledger | Fail: F1, F4 |
| Success/durable response | Prepared slot sealed before effect and recoverable through cleanup | Preparation is correct; cleanup failure can suppress handoff | Fail: F5 |
| Cleanup scalar and retry | Scalar retained until successful release; primary error preserved | Scalar cleared before release | Fail: F5 |

## Counter audit

The candidate's create and review curves are linear for the sealed-session reads they count. Apply is
not a one-read curve because the ancestry/signature borrow path grows independently of
`object_records`.

| Claim | Evidence | Disposition |
|---|---|---|
| One handler ledger | One ledger observed in production-path focused tests | Pass |
| Zero nested ledgers | No `local_v1` creation on the three live routes | Pass |
| One sealed session per store | Session registry is one-per-path, but reopen borrows bypass it | Fail |
| `object_records` equals physical reads | Apply exact `10` versus `13` reads | Fail |
| `aggregate_bytes` covers all authority bytes | 2,098,457-byte MR read with charge `0` | Fail |
| Stable 1x/3x/10x counters | Create/review stable for counted work; apply gap grows `1,3,10` | Fail for whole-request claim |
| Exact/minus/plus | Plus-one only for response body | Fail |

No counter from this candidate should be promoted as a whole-request or cross-runtime golden until
F1-F3 are repaired.

## Precedence audit

| Scenario | Required first result | Candidate result | Disposition |
|---|---|---|---|
| Malformed/version-invalid MR versus status/stale | Record shape corruption first | Shape validation precedes goal admission | Pass |
| Well-formed but unapproved apply | `409` before off-goal graph/signature/ancestry work | Implemented at `codefire:13613` | Pass, deliberate change |
| Well-formed stale apply | `409` before off-goal graph/signature/ancestry work | Implemented at `codefire:13619` | Pass, deliberate change |
| Deterministic legacy publication preflight failure | Original typed 4xx, before escrow write | Preflight now precedes copy planning/execution | Pass |
| Escrow write failure | Durable, recoverable atomic effect or no effect | `503`, partial object, no reducer marker | Fail: F4 |
| Replay reducer failure after fresh nonce acceptance | Exact same signed request remains retryable | `503`, then same request gets nonce `400` | Fail: F6 |
| Cleanup release failure after durable publication | Preserve selected success/durable bytes and terminal observation | No response, debt remains, terminal unset | Fail: F5 |

## Effect and authority audit

| Failure point | Objects | MR/branch/generation | Nonce | Pending/receipt | Response | Result |
|---|---|---|---|---|---|---|
| Status/stale/non-fast-forward rejection | Unchanged | Unchanged | Unchanged unless a prior durable request exists | Unchanged | Typed 409 | Pass |
| Response encoding limit before effect | Unchanged | Unchanged | Unchanged | Unchanged | Typed 413 | Pass |
| Final fence or sealed snapshot tamper | Unchanged | Unchanged | Unchanged | Unchanged | Typed subject/fence failure | Pass |
| Deterministic legacy preflight rejection | Unchanged | Unchanged | Unchanged | Unchanged | Original 4xx | Pass |
| Injected second escrow copy write | **One object added** | Unchanged | Unchanged | **No marker/transaction** | `503 retry_same_request` | Fail |
| Injected replay reducer failure | Unchanged | Existing authority remains | **Fresh nonce added** | Existing receipt not reconfirmed | `503`; exact retry becomes 400 | Fail |
| Injected cleanup release failure after publication | Durable effect remains | Durable authority changed | As published | Reducer may be complete | **No HTTP response** | Fail |

The no-authority/no-partial claims therefore hold for typed pre-effect rejection but not for every
failure after `start_effect`.

## Boundary audit

| Dimension/owner | Exact | Minus one | Plus one | Review |
|---|---:|---:|---:|---|
| Response body bytes | Yes | Yes | Yes | Complete |
| Response header bytes | Yes | Yes | No | Incomplete |
| Retained bytes | Yes | Yes | No | Incomplete |
| Temporary bytes | Yes | Yes | No | Incomplete |
| Object records | Yes | Yes | No | Incomplete and does not equal physical reads |
| Aggregate bytes | Yes | Yes | No | Incomplete and excludes authority reads |
| Graph nodes/edges | Yes | Yes | No | Incomplete |
| Logical depth/frontier | Yes | Yes | No | Incomplete |
| Snapshot revalidation records/bytes | Yes | Yes | No | Incomplete |
| Signed policy/nonce read set | Yes | Yes | No | Incomplete |
| Descriptor/MR/branch/generation/recovery/publication/projection/copy/cleanup | No owner | No owner | No owner | Missing |

## Mutation and test-vacuity audit

- The focused module contains 26 real-socket production-path tests and passes on `6dd4527`.
- The same current tests fail non-vacuously against `21fe751` with 23 failures and 4 errors, and
  against `e2a218e` with 33 failures and 27 errors.
- The seven required mutation classes are not all defended: nested ledger, hidden collection/copy,
  missing iterative poll, preflight reordering, and total nonce drop are killed; policy-owner and
  nonce-owner omissions survive.
- No mutation defends physical-read equality, ledger prepublication sealing, copy reducer recovery,
  cleanup retry ownership, or replay nonce/reducer atomicity.
- The test at `tests/test_python_http_mr_mutation_decision_work.py:667` makes the apply reread defect
  an expected value, so a correct single-session change would require rebaselining that assertion.

The suite is production-path non-vacuous, but the zero-residual mutation requirement is not met.

## `seen_at` float-width decision

Canonical `{"seen_at": ...}` encodings observed in the fixture space vary from 15 to 30 bytes;
realistic epoch floats occupy 24 to 30 bytes. The focused test explicitly rewrites timestamps to
integers before comparing exact aggregate counts (`tests/test_python_http_mr_mutation_decision_work.py:1675`).

Disposition: **not a separate blocker for this candidate, but only correctly predecessor-gated for
fixed persisted-byte fixtures**. A fixture that pins the exact nonce-cache bytes can support stable
counters. An unstabilized wall-clock float cannot support a cross-runtime golden aggregate-byte
claim and must not be promoted as one. F1-F3 already prevent promotion of the current counters, so
the float-width note does not add a distinct decision-changing finding.

## Residual and overlap audit

| Surface/owner | Overlap | Required integration treatment | Current disposition |
|---|---|---|---|
| Python upload response owner | MR uses `_HTTPMutationResponseSlot`, prepared success/durable responses, signature deferral, and publication preflight introduced by upload | Preserve one shared response owner; do not fork or weaken upload invariants | Required predecessor; candidate defects block composition |
| `do_POST` / `make_http_handler` | Upload, MR create/review/apply, GC, response encoding, deadline labels, observer, and slot handoff share adjacent hunks | Resolve to exactly one handler ledger/deadline and one slot transfer | High-conflict integration point; no integration authorization |
| Doctor | Doctor shares request creation, observer/terminal handling, and handler cleanup | Preserve Doctor's one-ledger deadline and release-fault semantics | Recheck after MR cleanup repair |
| Branch-session owner | Project gates, branch authority reads, and handler routing overlap | Use the branch/session-owned bounded authority reader rather than compatibility `read_json` | F1 should compose additively, not create a second branch owner |
| Read-route response owner | GET bundle/read routes share bounded freezer/header/write and terminal observation | Do not let MR cleanup or response-slot recovery regress read-route response semantics | Recheck response/observer integration after F5 |
| File transport | File and HTTP share project identity, nonce, branch/MR authority, and publication helpers | Add optional ledger/deadline/owner seams or purpose wrappers; preserve no-ledger file behavior | F1/F4 fixes must be additive and compatibility-safe |
| GC | `handle_http_gc` remains on separate authorization/nonce/publication timing | Keep separately predecessor-gated; do not claim uniform POST timing | Correctly outside this acceptance decision |
| Publication reducer | Upload and MR share pending/receipt/transaction helpers | Add request accounting and escrow progress without duplicating the reducer | F1/F4 require shared-owner repair |

## Commands and results

The following substantive validation was completed against the exact candidate. Inspection used
`git diff`, `git show`, `git rev-parse`, `rg`, and `nl -ba` on the committed tree.

| Command | Result |
|---|---|
| `git rev-parse HEAD b1c29fc 3afef8f 21fe751 6dd4527` | Exact hashes matched the subject table |
| `git diff --name-status 21fe751..6dd4527` | Only the repair product/test paths plus registered packet/prior report |
| `git diff --name-status 3afef8f..6dd4527` | Only `codefire` and the focused test module |
| `PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile codefire tests/test_python_http_mr_mutation_decision_work.py tests/test_codefire_cli.py` | **OK** |
| `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tests.test_python_http_mr_mutation_decision_work -v` | **OK — 26/26, 67.786s** |
| The exact 18 neighboring legacy MR/signature/nonce/replay/escrow/branch-protection cases listed below | **OK — 18/18, 217.643s** |
| Same focused module with current tests and `codefire` extracted from `21fe75175e997cc7c6f8c3ef2a7558144d850f94` | **FAILED non-vacuously — 23 failures, 4 errors** |
| Same focused module with current tests and `codefire` extracted from `e2a218e015953f082245f36dbc6aee906b837e5a` | **FAILED non-vacuously — 33 failures, 27 errors** |
| `python3 docs/development/tools/check_source_docs.py` | **OK** |
| `python3 docs/development/tools/check_project_docs.py` | **OK** |
| `git diff --check 21fe751..6dd4527` | **OK** |
| `git diff --check 3afef8f..6dd4527` | **OK** |

The exact neighboring unittest selection was:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest -v \
  tests.test_codefire_cli.CodeFireCliTests.test_http_merge_request_escrow_survives_source_project_removal \
  tests.test_codefire_cli.CodeFireCliTests.test_http_remote_payload_tamper_writes_no_layout_or_nonce \
  tests.test_codefire_cli.CodeFireCliTests.test_http_remote_request_signature_rejects_replayed_nonce \
  tests.test_codefire_cli.CodeFireCliTests.test_http_remote_request_signature_required_on_upload \
  tests.test_codefire_cli.CodeFireCliTests.test_merge_request_subject_escrow_survives_source_removal_and_rejects_tamper \
  tests.test_codefire_cli.CodeFireCliTests.test_merge_request_subject_matches_cross_runtime_golden_vector \
  tests.test_codefire_cli.CodeFireCliTests.test_python_remote_nonce_capacity_preserves_live_replay_memory \
  tests.test_codefire_cli.CodeFireCliTests.test_python_remote_publication_recovers_authoritative_staging_and_replays_receipt \
  tests.test_codefire_cli.CodeFireCliTests.test_python_remote_request_signature_enforces_default_skew_in_utc \
  tests.test_codefire_cli.CodeFireCliTests.test_python_remote_request_signature_rejects_payload_field_tamper_and_key_windows \
  tests.test_codefire_cli.CodeFireCliTests.test_python_upload_signed_invalid_identity_does_not_consume_nonce \
  tests.test_codefire_cli.CodeFireCliTests.test_python_upload_nonce_race_stays_pre_effect_and_exactly_retryable \
  tests.test_codefire_cli.CodeFireCliTests.test_remote_branch_protection_gates_upload_and_apply \
  tests.test_codefire_cli.CodeFireCliTests.test_remote_request_signature_matches_shared_golden_vector \
  tests.test_codefire_cli.CodeFireCliTests.test_remote_request_signatures_gate_mutating_operations \
  tests.test_codefire_cli.CodeFireCliTests.test_remote_upload_receipt_replays_across_file_and_http_transports \
  tests.test_codefire_cli.CodeFireCliTests.test_request_list_and_review_reject_invalid_merge_request_refs \
  tests.test_codefire_cli.CodeFireCliTests.test_upload_list_remote_clone_and_request_merge
```

Provider-independent probes used local temporary projects and the real socket handler, with
`PYTHONDONTWRITEBYTECODE=1`; they created no repository artifact. Their exact measured outcomes are
recorded in F1, F2, F4, F5, and F6.

Not run and not claimed: Cargo/rustc, any build, install, demo, broad/full Python discovery, or a
target directory.

## Decision and required next action

**Decision: `REQUEST_CHANGES`.**

Required next action: repair F1-F7 on a new committed candidate; add request-owned bounded authority,
identity, publication, projection, and cleanup owners; remove every sealed-record reopen; seal the
ledger exactly once before effect; make escrow copy and replay nonce publication durably atomic and
recoverable; add complete exact/minus/plus and killing mutation evidence; update canonical docs; run
the same bounded validation; then request a new independent zero-residual rereview.

Integration, Close, Install, and Dogfood remain unauthorized.
