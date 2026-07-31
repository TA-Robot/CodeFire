# CFR-168 Python HTTP Shared-Primitives Adversarial Preflight

Captured: 2026-07-31

Reviewer: Claude Opus 5 (`claude-opus-5`), independent architecture reviewer,
report-only lane.

Control input: the injected scheduler envelope and ticket text of
`cfr168-python-http-shared-primitives-preflight-claude-v2-20260731`. The
snapshot-local copies of an older, blocked Fugu instruction set for this path
and its linked JSON packet were not opened, quoted, or followed. Evidence is
frozen at `2c439d8b483c14919633036741af43be221e98cc`.

Declared write set, and the only repository write of this lane:

```text
docs/development/ai/swarm-results/cfr168-python-http-shared-primitives-preflight-claude-20260731/preflight.md
```

No product, test, canonical source document, packet, manifest, card, existing
report, issue, ledger, or lifecycle byte was modified. Reviewer probes lived
under `/tmp` and created no repository artifact.

---

## 0. Decision

```text
BLOCK
```

The accepted Doctor/Branch and MR-list contracts, the three immutable Upload
specifications, the MR-write F1–F7 problem statement, and the Rust Storage
invariant families **do** describe one internally consistent
request/session/response/publication lifecycle at the level of invariants.
Six of the seven named primitives resolve to `reuse` or `adapt`, and only one
(`RetryableCleanup`) is legitimately `shape-pending` on the open Upload
residual.

`READY` is nevertheless forbidden, because three of the ticket's `READY`
preconditions are not met at exact revisions. Each failure is a revision-level
or ownership-level fact, not a matter of degree, and each would force the next
MR-write implementer to guess:

| # | Failed `READY` precondition | Exact conflict |
|---|---|---|
| **B1** | *no unresolved specification conflict … must be guessed by the implementer*; *surface-local replacement is `BLOCK` unless an exact semantic incompatibility is demonstrated* | The frozen MR-write repair packet's `base_revision` `fcb2324145962e79d7c6ee173b9cf545733b957d` carries `codefire` **byte-identical to the rejected candidate** `6dd4527f…` (blob content SHA-256 `74a6cd8f9ae78de10d1042db84c89d2035abc6cda5ebc993cc42015cb7c9d59b`) and contains **none** of the fourteen accepted primitive symbols the repair must reuse. Reuse is physically impossible on that base; replacement is forbidden by the same rule. Choosing or creating the base is manager-owned composition and an explicit Non-Goal of this preflight. |
| **B2** | *no … incompatible accepted behavior must be guessed by the implementer* | The accepted MR-list tree `b16404676639336f705feffd9af4857d4a1fcc69` contains, on its two live `handle_http_upload` routes, the exact `_AccountedUploadCommitSource` cleanup defect that report `3df1db92…` holds **OPEN** as `HIGH-2` (`b164046:6994`–`6999`), plus an inverted primary/cleanup error precedence (`b164046:7047`–`7049`). Accepted product bytes and a frozen rejecting specification therefore contradict each other about the same symbol, and no frozen packet owns the accepted tree's copy. |
| **B3** | *every shared primitive has one compatible ownership/lifetime contract* (for `PreparedResponse` after the first authoritative effect) | `_PreparedHTTPMutationResponses` is **byte-identical in all five bound trees** (class-body SHA-256 prefix `104465a6dd8658f2`). A reviewer probe on the live base shows a rejected-owner release fault records `selection="success"` while leaving both owners live, after which the handler's own `except BaseException` arm destroys the already-selected payload: no response, `terminal_reason=None`, `retained_bytes=30` stranded. The two frozen specifications give **directly opposed** rules for this window (accepted GET: publish nothing on disposal failure; MR-write `F5`: return the selected success/durable bytes), and no accepted or rejecting report states the phase index that reconciles them. The accepted MR-list report puts the POST successor explicitly outside its write set; the disposition index routes it to a "shared-primitives / POST-precedence successor" that exists as no frozen packet. |

No weaker third model was invented to force `READY`. Where a reconciliation is
genuinely derivable from the frozen corpus it is stated below and marked
`frozen (synthesis)` with every source digest; where it is not derivable it is
recorded as a blocking conflict above.

This report authorizes no product byte, no candidate acceptance, no
integration, no Close, no Install, and no Dogfood. It does **not** authorize
the MR-write repair manifest.

---

## 1. Recomputed identity

### 1.1 Evidence base and worktree

| Item | Recomputed value |
|---|---|
| Freeze base / worktree `HEAD` | `2c439d8b483c14919633036741af43be221e98cc` |
| Worktree status at review start and end | clean |
| Live `codefire` bytes / lines | `1,341,034` / `33,455` |
| Live `codefire` content SHA-256 | `efd124fd442addf1c7f385a73d0fbbe959e3455d609ab3d5eb09dc82fb54fa10` |
| Pre-composition management/product base | `16d012938730fdb2b092b7f8c5cd601b53b81a1f` |
| `16d0129:codefire` content SHA-256 | `efd124fd442addf1c7f385a73d0fbbe959e3455d609ab3d5eb09dc82fb54fa10` — **identical to `HEAD`** |

**Material fact.** The evidence snapshot's product tree *is* the
pre-composition base named in the accepted Doctor/Branch report
(`c57215e2`, "Pre-composition management/product base"). The live dispatcher
carries **no** accepted Doctor/Branch composition, **no** accepted MR-list
protocol, **no** Upload repair, and **no** MR-write candidate work. Every
accepted contract in this preflight exists only inside an unintegrated
candidate tree. All live-base line references below are at `2c439d8`.

### 1.2 Accepted inputs — recomputed

| Input | Bound revision | Recomputed | Report path | Report SHA-256 (recomputed) |
|---|---|---|---|---|
| Doctor + Branch product | `701469f91dd0c4c52c5628fb3e9d9e9d0ab0c714` | resolves; `test: update branch-list deadline expectation`, 2026-07-30 | — | `codefire` = `1f2fa57d41e0908c683a7c53fe7d5087199b4718a6604aefa8218c865d86695a` |
| Doctor + Branch accepted report | `c57215e243839b224a641251fc9e3bf092f7e116` | resolves; `review: accept corrected Doctor branch composition` | `…/cfr168-python-http-doctor-branch-composition-rereview-20260730/review.md` | `c7a1665fdf4a6e1104bbab18ec8051c048ace1d25e865952bbbcd35a06eb19c0` |
| MR-list candidate | `b16404676639336f705feffd9af4857d4a1fcc69` | resolves | — | `codefire` = `a7515751bc5d0001aed3d168ab385213400500e19503e7f5f88da4fe6fc2335b`, `1,437,333` bytes |
| MR-list accepted report | `ee6517a780e56e8e79630688533a04e64d4c1ae5` (`ee6517a`) | resolves | `…/cfr168-python-http-mr-list-final-publication-closure-rereview-claude-20260731/review.md` | `5c952a9a5c162bfbb93522bb644fcba44ce32ddf3628501e6f94308899fe760e` — **matches the ticket** |

The ticket supplied no SHA-256 for the Doctor/Branch report. The recomputed
digest `c7a1665f…` independently matches the value the snapshot-local
disposition index records for the same commit, and the MR-list candidate's
`codefire` content digest and byte count independently match row 1 of the
accepted report's own changed-path table (`a7515751…`, `1437333`). Identity is
exact.

### 1.3 Rejecting specifications — recomputed

Every SHA-256 the ticket supplies recomputes **exactly**:

| Specification | Report commit | Report SHA-256 (recomputed) | Ticket match | Candidate subject | Candidate `codefire` SHA-256 |
|---|---|---|---|---|---|
| Upload generation 1 | `f835f64abb50fecaea28673ffa5dd829f1d3fc70` | `7632552eb0ca60addca0d83fe3ba53ead642a7cbee82937b1b7b37754f6cb2cf` | yes | `76baff84cb1192e11b26dcc669fb800f7b13fa30` | — (superseded) |
| Upload generation 2 | `8bdc076bcb38f9eb23217c848b95f391f2700488` | `57d6792106f09c17a1d154aedb5a64d342e5d19c23db7e9963c46061c868c132` | yes | `467b0c27715e424ae18ffb5664da053b55915e2d` | — (superseded) |
| Upload generation 3 (latest) | `b3de0d74dacf5b33295dd95e04bcc8205f7a8aca` | `3df1db92d6adcef9d318e23182fb9c62519f6eacb091266ced9616626f64cf1f` | yes | `02e04c7e6458035d9c7d58de4d30b4a12e7cd594` | `63a9be232d064afa022432537f0686e57d47845b1f5654b929a69acec7a872fa`, `1,408,820` bytes |
| MR-write | `84175bd2de8c92ee1eb4f64fd366e5d9b3560204` | `214e42fc8f36108f45e17b13e4343bfc2b56cc89b72be5c1258f5b497c9ec6ef` | yes | `6dd4527f23135388b53fc5c6d637c4105cc5d659` | `74a6cd8f9ae78de10d1042db84c89d2035abc6cda5ebc993cc42015cb7c9d59b` |
| Rust Storage | `8ac6741d218e1869fef91d96d29ac0d18d9d674d` | `6438df445b2d05510895fd23f73a4e71ba49238da15b00e86abf1a477b5ab960` | yes | `cdcb9bdefe7d25a118a9dc14171864d75f5ad304` | — (Rust; no Python design import) |

Candidate identities are recomputed **only** to bind the counterexample
subject. None is treated as accepted. Upload generation 3's `codefire` content
digest and byte count independently match its own report's changed-path table.

### 1.4 Lineage — where each contract physically lives

```text
16d0129  pre-composition base  ── product tree == 2c439d8 (this snapshot)
   │
   ├── 5e3b0c7 → 0703f71b → 7e87665b → 701469f   ACCEPTED Doctor+Branch (+ adjacent upload slice)
   │                                      │
   │                                      └── … → 06e519e4 → b164046   ACCEPTED MR-list
   │
   ├── 5254e3dc → … → b19c4346 → 467b0c2 → 909bbea → 02e04c7   REJECTED Upload gen 3
   │
   └── cab3126 → … → 21fe751 → 6dd4527 → b86a782 → fcb2324   REJECTED MR-write + frozen repair base
```

Verified relations: `701469f` **is** an ancestor of `b164046`; `merge-base(701469f, b164046) = 701469f`.
`merge-base(b164046, 02e04c7) = 5254e3dc`; `merge-base(b164046, 6dd4527) = cab3126`;
`merge-base(02e04c7, 6dd4527) = 5254e3dc`. `fcb2324:codefire == 6dd4527:codefire`.
`2c439d8` is a descendant of `16d0129` and is **not** a descendant of `701469f`.

Consequence: `b164046` is the **only** tree that carries both accepted product
contracts. The Upload and MR-write lanes are siblings of it, not descendants.

### 1.5 Cross-tree primitive-symbol census

Counted as definition sites (`class X` / `def X`) in each tree's `codefire`.

| Symbol | live base `2c439d8` | accepted `701469f` | accepted `b164046` | rejected Upload `02e04c7` | rejected MR-write `6dd4527` |
|---|---:|---:|---:|---:|---:|
| `DecisionWorkLedger` | 1 | 1 | 1 | 1 | 1 |
| `HTTPRequestDeadline` | 1 | 1 | 1 | 1 | 1 |
| `SealedCommitValidationSession` | 1 | 1 | 1 | 1 | 1 |
| `remote_project_gate` | 1 | 1 | 1 | 1 | 1 |
| `_PreparedHTTPResponseOwner` | 1 | 1 | 1 | 1 | 1 |
| `_PreparedHTTPMutationResponses` | 1 | 1 | 1 | 1 | 1 |
| `_HTTPMutationResponseSlot` | 1 | 1 | 1 | 1 | 1 |
| `seal_prepublication` (**call sites**) | **0** | **0** | **0** | **0** | **0** |
| `read_route_phase` | 0 | **1** | **1** | 0 | 0 |
| `_HTTPReadResponseSlot` | 0 | 0 | **1** | 0 | 0 |
| `require_publishable_state` / `prove_and_take` | 0 | 0 | **1** / **1** | 0 | 0 |
| `displace_prepared_response` | 0 | 0 | **1** | 0 | 0 |
| `prove_pinned_project_state` | 0 | 0 | **1** | 0 | 0 |
| `os_error_origin` / `classify_route_os_error` | 0 | 0 | **1** / **1** | 0 | 0 |
| `route_filesystem_boundary` / `mark_transport_origin` | 0 | 0 | **1** / **1** | 0 | 0 |
| `_release_exact` / `close_with_retry` | 0 | 0 | **1** / **1** | 0 | 0 |
| `record_secondary_failure` / `cleanup_diagnostic` | 0 | 0 | **1** / **1** | 0 | 0 |
| `_AccountedUploadCommitSource` | 0 | 1 | 1 | 1 | 0 |
| `_AccountedUploadSourceMemory` | 0 | 0 | 0 | **1** | 0 |
| `RemoteJSONPublicationRootProof` | 0 | 0 | 0 | **1** | 0 |
| `_charged_allocation_boundary` | 0 | 0 | 0 | **1** | 0 |
| `_HTTPMergeRequestLane` / `AccountedRemoteCommitBorrow` | 0 | 0 | 0 | 0 | **1** / **1** |

Three facts drive the decision:

1. `DecisionWorkLedger.seal_prepublication` (live `codefire:862`) has **zero
   call sites in every one of the five bound trees**. It is dead code
   everywhere, in the accepted trees as well as the rejected ones. This
   independently corroborates and widens MR-write `F4`.
2. The fourteen accepted-primitive symbols in the `b164046` block exist in
   **no other tree** — in particular, not in the MR-write repair base.
3. `_AccountedUploadCommitSource` is present in both accepted trees, where its
   cleanup still carries the open `HIGH-2` defect shape.

---

## 2. Per-clause register

Evidence state uses the ticket's vocabulary: `behavior-open`,
`behavior-closed`, `evidence-unproven`, `claim-only`. `frozen` means the exact
shape is fully determined by the cited source; `shape-pending` means the
invariant and its rejecting counterexample are fixed but the exact
implementation shape still depends on an open residual.

| # | Clause | Source report SHA-256 | Evidence state | Marker |
|---|---|---|---|---|
| C1 | One `DecisionWorkLedger` + one `HTTPRequestDeadline` per HTTP request, created in `handle_one_request`, passed by reference, no nested ledger | `c7a1665f…` (Doctor/Branch, ACCEPT) + `5c952a9a…` §8 row "exact handler ledger/deadline" | behavior-closed on Doctor/Branch/Branch-list/MR-list/bundle; behavior-open on all three MR-write routes and GC | `frozen` |
| C2 | One combined route-phase helper names every response boundary and every per-route deadline flag is reset per request | `c7a1665f…` conflict-resolution items 1–6 | behavior-closed for `bundle`/`doctor`/`branch list`; **absent** for mutation suffixes (`701469f:14193`–`14201` returns `None`, mutation phases stay literal) | `frozen`; extension to mutation suffixes required |
| C3 | One retained `SealedCommitValidationSession` per object store per request; no live route physically rereads a record the session has retained | `214e42fc…` `F2`; `c7a1665f…` (Branch: one session, `finalize(terminate=False)`) | behavior-open — `6dd4527`'s `AccountedRemoteCommitBorrow` reopens records; apply exact `object_records=10` vs `record_reads=13` | `frozen` |
| C4 | Purpose-bounded, single-use commit handles vended by the retained session; fingerprint bound to the session snapshot | `214e42fc…` `F2` required repair | evidence-unproven — no tree implements it | `frozen` |
| C5 | Last complete project/session/subject observation is a named fence; nothing authority-dependent follows it | `5c952a9a…` §4 `F1` (three fences, `gate.fence()`, exit fence skipped when fenced) | behavior-closed on the accepted GET route only; behavior-open at live base (exit fence runs after the route body, `codefire:2287`–`2305`) | `frozen` |
| C6 | The project identities embedded in a request semantic are compared to the identities the acquired gates hold — not to a second independent read | `214e42fc…` `F1` | behavior-open — live `ordered_remote_merge_gates` re-reads both ids itself (`codefire:2316`, `codefire:2328`) via uncharged `remote_project_stable_id` (`codefire:2195`) | `frozen` |
| C7 | Every authority / descriptor / recovery / receipt / pending / generation / publication read is charged before the read and bounded by the request deadline | `214e42fc…` `F1`; ADR-020 `adr/ADR-020-decision-equivalent-work-budget.md:42`, `:50` | behavior-open — live MR routes use plain `read_json` (`codefire:1429`, `codefire:2363`), `read_remote_branch_record` (`codefire:20605`), `read_json(remote_generation_path(...))` (`codefire:11965`) with no ledger at all | `frozen` |
| C8 | The success/durable response pair is prepared, sealed, and effect-marked **before** the first authoritative effect; exactly one first-byte publication point | `5c952a9a…` §3 timeline; `214e42fc…` `F4`; live upload `codefire:12178`–`12182` | behavior-closed on upload; behavior-open on all three MR routes (response is encoded at `codefire:14673`–`14675`, after the effect) | `frozen` |
| C9 | Publishability is proved **after** every fallible teardown and is the only transition into publication | `5c952a9a…` §3, §4 `F1`/`F3`, `require_publishable_state` → `prove_and_take` | behavior-closed on the accepted GET route; **absent** from the mutation slot in all five trees (`_HTTPMutationResponseSlot.take`, live `codefire:13475`, has no proof) | `frozen` |
| C10 | Disposal precedence **before** the first authoritative effect: dispose the older prepared response first, refuse to select when disposal fails, never publish a success carrying debt or permits | `5c952a9a…` §4 `F2`/`F3`/§4.5 | behavior-closed on the accepted GET route; behavior-open on POST | `frozen` |
| C11 | Disposal precedence **after** the first authoritative effect: the already-selected success/durable bytes must still publish; the cleanup fault is recorded as secondary and never substitutes for the primary; the terminal class is a cleanup/publication class, never `complete` | `214e42fc…` `F5` (four postconditions) + `5c952a9a…` §4.5 (secondary recording, never substitution) + `6438df44…` `R3` (publication failure must not be terminal `Complete`) + `6438df44…` `R4` (no second visible document) | behavior-open in all five trees; **no single source states the phase index** | **conflict — see B3**; the synthesis is stated in §6.4 as `frozen (synthesis)` and must be *assigned an owner* before it is frozen for an implementer |
| C12 | An owner never erases its claim, scalar, handle, or payload accounting before the corresponding release has actually succeeded | `3df1db92…` `HIGH-2`; `57d67921…` `HIGH-2`; `214e42fc…` `F5`; `5c952a9a…` §4.5/§4.7 | behavior-closed for `_PreparedHTTPResponseOwner` (live `codefire:13237`–`13244` keeps the counts), `_release_exact`/`close_with_retry` (`b164046` only), and `6dd4527`'s `AccountedRemoteCommitBorrow`; behavior-open for `_AccountedUploadCommitSource` (`b164046:6994`–`6999`) and for the selected/rejected pair (§6.4) | `frozen` invariant |
| C13 | The still-owed reservation is reachable for retry from a production surface — public retry API **or** idempotent finalizer **or** non-consuming handles | `3df1db92…` `HIGH-2` minimal closure (three alternatives, none accepted) | behavior-open; the Upload lane is actively repairing exactly this | **`shape-pending`** (licensed by the ticket) |
| C14 | Cleanup faults never displace the primary decision, including on constructor unwind, in every build | `3df1db92…` `HIGH-2` supporting item 3; `6438df44…` `R2`; `5c952a9a…` §4.5 | behavior-open — `b164046:7047`–`7049` raises the cleanup error with the primary as `__cause__` on a live upload route | `frozen` |
| C15 | The handler ledger is prepublication-sealed exactly once, after every fallible raw revalidation and before the first effect; every legal post-seal work item holds a reserved permit | `214e42fc…` `F4` | evidence-unproven — **zero call sites in all five trees**; `apply` (`codefire:525`–`526`) and `reserve_into` (`codefire:760`–`761`) refuse after the seal while `release` (`codefire:882`) and `terminate` (`codefire:896`) stay legal; `seal_prepublication` is neither idempotent nor publicly observable | `frozen` spec, unimplemented; needs the two product seams in §7 row 3 |
| C16 | No partial object effect exists without a durable record an exact retry can resume; escrow materialization shares the reducer's durability | `214e42fc…` `F4` | behavior-open — live create writes objects at `codefire:11673` before the reducer at `codefire:11674`; the same order is live on upload (`codefire:12337` → `codefire:12338`) | `frozen` for MR-write; the upload instance is `claim-only` (no accepted report covers it) |
| C17 | An accepted replay nonce is published atomically with the durable replay/recovery transition; every `503 retry_same_request` stays retryable with the same signed bytes | `214e42fc…` `F6` | behavior-open | `frozen` |
| C18 | One typed, exhaustive error-origin taxonomy shared by GET and POST: filesystem, transport, deadline, budget, integrity, post-commit cleanup — classified at the boundary that raised it, never by message text | `5c952a9a…` §4 `F2` (`os_error_origin`, `classify_route_os_error`, `route_filesystem_boundary`, `mark_transport_origin`); `6438df44…` `R11` (no wildcard, no text matching) | behavior-closed inside `b164046`; **absent** from every other tree including the MR-write repair base | `frozen` |
| C19 | Transport origin abandons the connection; a deadline arm precedes the other arms; `MemoryError` is not absorbed by the `OSError` arm | `5c952a9a…` §4 `F2` (probes P3, P11) | behavior-closed on the accepted GET route | `frozen` |
| C20 | A publication-root proof is atomically single-use with an explicit `available/consuming/consumed/revoked` state machine | `3df1db92…` (`HIGH-3` CLOSED, preserved); `57d67921…` `HIGH-3` | behavior-closed **only inside rejected candidate `02e04c7`**; the symbol exists in no accepted tree | `frozen` invariant; **out of scope for MR-write** — MR-write must not introduce a competing proof object |
| C21 | Counters are charged before the corresponding read/allocation/parse/validation/output append; a probe that finds nothing is not a logical unit | ADR-020 `:42`; `6438df44…` `R1` | behavior-open on the live MR routes (zero charges) | `frozen` |
| C22 | Container capacity, frontier entries, and re-open authority are owners in the simultaneous peak | `6438df44…` `R5`; `7632552e…` formula table | behavior-closed for Upload's sealed owner; evidence-unproven for MR-write's projections and copy plans (64-byte slots for real dicts, `214e42fc…` `F1`) | `frozen` |
| C23 | Every charged dimension has exact, minus-one, **and plus-one** boundary evidence, and every owner omission is detected | `214e42fc…` `F3`; `6438df44…` §8 | behavior-open — plus-one exists only for response body bytes at `6dd4527`; two owner-omission mutations survive | `frozen` |
| C24 | Canonical source documents describe exact live behavior | `214e42fc…` `F7`; `6438df44…` `R14`; `3df1db92…` "Documentation Accuracy" | behavior-open — the data-model catalog still calls the Python response owner upload-only | `frozen` |
| C25 | Blanket or self-witnessing formulas are replaced by exact simultaneous-owner derivations; slack must not hide a structural undercount | `6438df44…` `R7`–`R10`; `3df1db92…` non-blocking `_control_charge_witness` note | evidence-unproven for Python HTTP; recorded as a cross-runtime family, no Rust design import | `frozen` invariant |

---

## 3. Production call graphs — live base `2c439d8`

Every path below was read in the live `codefire` at the freeze base. All nine
live HTTP entry points are enumerated; the five focused modules are marked.

```text
ReusableThreadingHTTPServer  (codefire:12621)
└─ _ProcessHTTPConnectionAdmission / _HTTPConnectionPermit  (12495 / 12422)
   └─ CodeFireHTTPHandler.handle_one_request                (14241)
      ├─ decision_work = DecisionWorkLedger(http_v1, "http request")   (14242)  ← the one ledger
      ├─ request_deadline = HTTPRequestDeadline(30_000 ms, monotonic)  (14245)  ← the one deadline
      ├─ flags reset: _bundle_deadline_active, _mutation_deadline_active (14250-14251)
      │              (accepted 701469f additionally resets _doctor_/_list_)
      ├─ reserve("request_headers")  → _BoundedHTTPHeaderReader        (14270, 12822)
      ├─ parse_request → do_GET | do_POST                              (14313)
      └─ finally: cancel permits, release header storage, terminate, observer (14332-14380)

do_GET  (14391)                                          [no response slot at base]
├─ GET /v1/projects/{org}/{app}/branches           → handle_http_list        (11477)  ▲ Branch
├─ GET …/doctor[?quick=1]                          → handle_http_doctor      (11491)  ▲ Doctor
├─ GET …/branches/{b}/bundle                       → handle_http_bundle      (11524)  — only route
│                                                     given the ledger+deadline at base
├─ GET …/merge-requests                            → handle_http_request_list(11705)  ▲ MR-list
└─ send_json(status, payload)                                                  (14432)
      └─ prepare_json → prepare_json_owned → _PreparedHTTPResponseOwner (14187, 14017, 13211)
         then publish_prepared → wfile.write                                   (14214)
   ⇒ encoding and publication happen AFTER the route returned and after
     remote_project_gate's exit fence (2287-2305) has already run.

do_POST  (14470)
├─ body: reserve request_body (14504) → read → json.loads (14546)
│        → canonical_json_size shrink-to-exact retained (14555-14567)
├─ POST …/branches/{b}/upload   → handle_http_upload(..., response_slot,
│                                   decision_work, deadline)          (12017)  ▲ Upload
│                                 prepared = mutation_slot.take()             (14611)
├─ POST …/gc                    → handle_http_gc(storage_root,...)    (11502)  predecessor-gated
├─ POST …/merge-requests        → handle_http_request_merge(...)       (11616)  ▲ MR-write create
├─ POST …/merge-requests/{id}/review → handle_http_request_review(...) (11740)  ▲ MR-write review
├─ POST …/merge-requests/{id}/apply  → handle_http_request_apply(...)  (11867)  ▲ MR-write apply
├─ except BaseException: mutation_slot.close() / prepared.close() in
│                        try/except pass, release request retained, re-raise (14638-14660)
├─ if prepared is None: prepared = self.prepare_json(status, payload)         (14673-14675)
└─ publish_prepared(prepared)                                                 (14689)
```

Live MR-write signatures carry **no** ledger, deadline, or response slot:

```text
handle_http_request_merge (storage_root, org, app, request)                  11616
handle_http_request_review(storage_root, org, app, mr_id, request)           11740
handle_http_request_apply (storage_root, org, app, mr_id, request)           11867
handle_http_request_list  (storage_root, org, app)                           11705
handle_http_list          (storage_root, org, app)                           11477
```

Accepted signatures, for contrast:

```text
701469f:11841 handle_http_list  (storage_root, org, app, ledger, deadline=None)
701469f:11863 handle_http_doctor(storage_root, org, app, quick=False, ledger=None, *, …)
b164046:13593 handle_http_request_list(storage_root, org, app, ledger, deadline=None, *,
                                       response_slot: _HTTPReadResponseSlot | None = None)
              -> tuple[int, dict[str, Any] | None]
```

### 3.1 Physical owner and lifetime table — live base

Column key: **Create** = creating function and request ledger; **Point** =
allocation/read/open/acquire point; **Formula** = retained (R) / temporary (T);
**Permit** = permit or descriptor identity; **Borrow** = borrowers and physical
rereads; **Terminal** = transfer/selection/publication/release/retry and final
terminal owner; **Precedence** = error precedence when release also fails.

| Owner | Create · ledger | Point | Formula | Permit / descriptor | Borrowers · physical rereads | Terminal owner | Precedence when release also fails |
|---|---|---|---|---|---|---|---|
| Request headers | `handle_one_request` · `http_v1` "http request" | `reserve("request_headers")` `14270` before `_BoundedHTTPHeaderReader` `14275` | R `2 × min(64 KiB, retained/2)`; charge `request_header_bytes` | permit `request_headers` | `parse_request` | `finish_request_headers` `14227` / `release_request_header_storage` `13587` | swallowed (`14336` runs before the terminal decision); non-zero `current` forces `close_connection` `14339` |
| Raw request body | `do_POST` · same | `reserve("request_body")` `14504` before `rfile.read` `14524` | T `2 × Content-Length` | permit `request_body` | `json.loads` | released at `14551` after the AST exists | `except BaseException` `14580`–`14587` swallows |
| Parsed request AST | `do_POST` · same | `reserve` `14535` before `json.loads` `14546` | R `request_canonical_upper(body)`, shrunk to `canonical_json_size` `14560` | permit `request_body` | every route | released `14669` / `14684` | `14588`–`14594` swallows |
| **Project descriptor identity** | `remote_project_stable_id` `2195` · **none** | `_open_directory_nofollow` `2198` | locally bounded 16 KiB, **uncharged** | raw fd, closed in `finally` `2205` | `ordered_remote_merge_gates` `2316`, `2328` — **second independent read** | fd closed | n/a — no ledger |
| Pinned project gate | `remote_project_gate` `2214` · **none** | `_open_directory_nofollow` `2216`, `_open_directory_at` `2225` | uncharged | `project_fd`, `locks_fd`, `rebound_fd`, `rebound_locks_fd`, `(st_dev, st_ino)` pairs | route body under `yield` `2285` | exit fence `2287`–`2305` then `finally` closes 4 fds `2307`–`2309` | exit-fence `CFError` **replaces** the route result; `finally` close errors surface last |
| Project gate lock | `stable_remote_lock` `990` · **none** | `flock` after open | uncharged | lock fd + dup'd `owned_directory_fd` | gate | `LOCK_UN` + 2 closes `1034`–`1039` | fault after route body escapes as raw `OSError` at base — no origin taxonomy (C18) |
| Branch authority | `read_remote_branch_record` `20605` · **none** | `read_json` `1429` — unbounded `read_text` + `loads` | uncharged | none | review `11789`, apply `11962`, list `11730` | GC'd | n/a |
| MR authority | `read_remote_merge_request_path` `2363` · **none** | `read_json` `1429` | uncharged | none | review `11753`/`11776`, apply `11880`/`11918`/`11959`, list `11713` | GC'd | n/a |
| Generation authority | `read_json(remote_generation_path)` `11965`, `current_remote_generation` `6834`, `next_remote_generation` `6842` · **none** | `read_json` | uncharged | none | apply | GC'd | n/a |
| Signature / nonce records | `verify_remote_request_signature` `3924`, `record_request_nonce` `3759`, `request_nonce_cache_postimage` `3776` · **none on MR routes** | `read_json` | uncharged | none | `authorize_remote` `6787` | written by `write_json` | n/a |
| Sealed object snapshot | `SealedCommitValidationSession` `20735` · caller ledger where supplied | session construction; control upper acquired first | R `SEALED_SESSION_CONTROL_UPPER` = `11,456` B + per-document frames (`7632552e…` formula table) | session registry key = objects root | `validate_sealed_commit` `28779`, signature policy, ancestry | `session.close` / `finalize(terminate=False)` | `02e04c7` makes the scalar release retryable; the `b164046` wrapper defeats it (`6994`–`6999`) |
| Escrow object copies | `copy_object_graph` `8799` / `write_object_records` `10351` · **none on MR routes** | `write_json` per object | uncharged; `214e42fc…` `F1` shows 64-byte slots for real dicts | none | none | none — **no transaction marker, no rollback** | mid-copy failure leaves a partial effect with no reducer state (`F4`) |
| Publication pre/post-images | `RemoteJSONPublicationWritePreflight` `2895`, `RemoteJSONPublicationPreflight` `2955`, `preflight_remote_json_publication` `3220` · **none** | `read_json` reopen `2926`, `3076`; `authority_bytes`/`before_bytes`/`after_bytes` built in `3175`–`3273` | uncharged, no deadline poll in the canonical-encoding loops | none | `execute_remote_json_publication` `3285` | plan/result dropped | n/a |
| Pending / recovery / receipt / audit | `preflight_remote_publication_recovery` `2547`, `remote_publication_recovery_exists` `2531`, `validate_remote_publication_receipt` `2450`, `recover_remote_pending_quarantine` `2719` · ledger only when the caller supplies one | `read_json` | uncharged on MR routes | transaction dir names `2603` | `execute_remote_json_publication` `3294`/`3351`/`3361` | `remove_remote_transaction_dir_quarantine` `2689` | `2406` `raise cleanup_error` |
| Prepared response owner | `prepare_json_owned` `14017` / `prepare_mutation_json` `13899` · handler ledger | `reserve(response_body)` `14040` / `13903` **before** `freeze_response_body` `13793` | R exact header + body bytes; charge `response_bytes`, `response_header_bytes` | permits `response_body`, `response_headers` | `_PreparedHTTPMutationResponses`, slot | `publish` `13246` → `finally: close()` `13253` | `close` `13237` drops payload refs first but **keeps the counts** until the release succeeds — retryable ✔ |
| Success/durable response pair | `_PreparedHTTPMutationResponses` `13262` · handler ledger | constructed by `prepare_mutation_json` | R both bodies + both headers; charges use `max(success, durable)` `13332`, `13344` | both permits, released in `seal_for_effect` `13319` | `_HTTPMutationResponseSlot` | `_select` `13293` closes the rejected owner `13314` | **`selection` is set at `13313` before the fallible `13314`; on failure both pointers stay live and the subsequent `slot.close()` destroys the selected payload** — see §6.4 |
| Mutation response slot | `_HTTPMutationResponseSlot` `13426` · handler ledger | `do_POST` `14600` (upload only) | none of its own | none | route | `take()` `13475` — **no publishability proof** | `close()` `13480` closes both `responses` and `selected` unconditionally |
| Cleanup debt | `DecisionWorkLedger._current` · handler ledger | `release` `882` | underflow is a typed `CFError` `889`; **no over-release guard at base** (`_release_exact` exists only in `b164046`) | n/a | every owner | `handle_one_request` `finally` `14337`–`14349` | non-zero `current` ⇒ `close_connection`, `terminate` skipped, `terminal_reason` stays `None` |

`http_v1` limits at base (`codefire:84`–`109`): `retained_bytes` 33,554,432
concurrent; `temporary_bytes` 16,777,216 concurrent; `request_body_bytes` and
`response_bytes` 16,777,216 cumulative; `request_header_bytes` and
`response_header_bytes` 65,536 cumulative; `request_deadline_ms` 30,000. These
match the values recorded independently in `7632552e…`.

---

## 4. Effect and response timelines

### 4.1 Reference timeline — live upload (`handle_http_upload`, `codefire:12017`)

The only live route that already prepares its response before its effect.

| Step | Live site | Fallible? | Authority requirement |
|---|---|---|---|
| request parse | `14478`–`14567` | yes | pre-effect; typed 400/413 |
| authority/session capture | `authorize_remote` `6787`; staging `12076` | yes | pre-effect |
| response pair preparation | `response_slot.prepare(200, result)` `12178` | yes | **byte-identical** — the frozen success bytes must be the bytes published later |
| prepublication seal (response work) | `seal_for_effect()` `12181` | yes | permits consumed; `apply` still legal because the **ledger** is never sealed (C15) |
| effect-start mark | `mark_effect_started()` `12182` | yes | after this point no ordinary response may replace the committed outcome |
| final authority fence | `remote_project_gate(project_root)` `12184`; publication preflight seal `12322` | yes | last complete observation |
| first authoritative effect | `write_object_records` `12337` | yes | **durably recoverable** — currently outside the reducer (C16) |
| reducer confirmation / recovery | `execute_remote_json_publication` `12338` | yes | already committed once pending state exists |
| response selection | `select_success()` `12362` / `select_durable()` `12373` | yes | transactional; must not be able to replace a committed outcome |
| first byte | `publish_prepared` `14689` → `_PreparedHTTPResponseOwner.publish` `13246` | yes | transport-abandoned on failure |
| cleanup / terminal observation | `publish`'s `finally: close()` `13253`; `handle_one_request` `finally` `14337`–`14380` | yes | must not suppress the published outcome |

### 4.2 Required timeline — MR-write create / review / apply

Live order today, per route:

```text
create  (11616): authorize_remote 11630
                 → ordered_remote_merge_gates 11666        [gate; ids re-read 2316/2328]
                 → copy_object_graph 11673                 ← FIRST AUTHORITATIVE EFFECT
                 → execute_remote_json_publication 11674   ← reducer AFTER the effect
                 → return 200 11702
                 → do_POST prepare_json 14675              ← RESPONSE ENCODED AFTER THE EFFECT
                 → publish_prepared 14689

review  (11740): authorize_remote 11745 → read MR 11753
                 → gate 11771 / ordered gates 11773
                 → re-read MR 11776, branch 11789, staleness 11796
                 → execute_remote_json_publication 11838 or 11854
                 → return 200 11864 → prepare_json 14675 → publish 14689

apply   (11867): authorize_remote 11872 → read MR 11880 → validate 11883
                 → validate_merge_request_subject 11890
                 → recovery branch: gate 11913, generation 11914, branch 11915, MR 11918,
                   execute_remote_json_publication 11919 → return 200 11943
                 → normal: gate 11958, re-read MR 11959, branch 11962,
                   generation_before 11965, execute_remote_json_publication 11986
                 → return 200 12014 → prepare_json 14675 → publish 14689
```

Required order after repair, with the fallibility contract per step:

| Step | Required site | Authority contract |
|---|---|---|
| request parse | `do_POST` body owners | pre-effect, typed |
| authority/session capture | route-owned bounded readers on the handler ledger (C7); one sealed session per store (C3) | pre-effect, typed |
| response pair preparation | `response_slot.prepare(success)` + fixed `HTTP_MUTATION_RETRY_PAYLOAD` durable owner, before any effect (C8) | **byte-identical** to what publishes |
| publication plan / postimages | charged preimage/postimage/plan owners (C7, C22) | pre-effect |
| prepublication seal | `slot.seal_for_effect()` **and** `ledger.seal_prepublication()` exactly once (C15) | after every fallible raw revalidation |
| final authority fence | `gate.fence()` + semantic-vs-gated identity comparison (C5, C6) | last complete observation |
| first authoritative effect | escrow materialization **inside** the durable reducer or a staged transaction with progress/rollback (C16) | **durably recoverable** |
| reducer confirmation / recovery | `execute_remote_json_publication`, with the replay nonce atomic with the replay transition (C17) | already committed |
| response selection | `select_success` / `select_durable` — transactional, cannot replace the committed outcome | already committed |
| first byte | one publication point after the publishability proof (C9) | transport-abandoned |
| cleanup / terminal observation | retryable release of every owner (C12, C13); one terminal observation; post-effect precedence per C11 | must not suppress the published outcome |

**Invariant that must hold at every fallible step:** no response may be
selected after an effect in a way that replaces the committed outcome. At the
live base this is violated structurally on all three MR routes, because the
response does not exist until `codefire:14675` — after the effect — so the
committed outcome and the published outcome are chosen by two different pieces
of code with no transactional link.

---

## 5. Shared primitive decisions

Surface-local replacement is `BLOCK` unless an exact semantic incompatibility
is demonstrated. None of the seven required a replacement.

### 5.1 `RequestContext` — **adapt**

One handler ledger plus deadline plus route phase.

*Reuse:* `DecisionWorkLedger` (`codefire:403`) and `HTTPRequestDeadline`
(`codefire:320`) created once in `handle_one_request` (`14242`, `14245`).
*Adapt:* extend the accepted `read_route_phase` (`701469f:14193`–`14201`) to
return mutation suffixes instead of `None`, and reset one flag per live route
per request (the accepted composition already resets four; MR-write adds
`create`/`review`/`apply` phases or one `_mutation_route` label). Thread the
exact ledger and deadline into all three MR signatures and into every authority
reader they call (C7).
*Incompatible semantics:* none found. The live `_mutation_deadline_active`
single flag cannot distinguish three MR phases, so deadline diagnostics would
be indistinguishable across routes — an extension, not a conflict.

### 5.2 `SealedSnapshotSession` — **adapt**

One physical capture with purpose-bounded borrows.

*Reuse:* `SealedCommitValidationSession` (`codefire:20735`), one per objects
root per request, `finalize(terminate=False)` for the final bounded
revalidation (accepted Branch behavior, `c7a1665f…`).
*Adapt:* vend purpose-specific, single-use handles from the retained session
(C4) and make signature-history and ancestry consume them, so `object_records`
equals physical object-record reads at exact, minus-one and plus-one
(`214e42fc…` `F2`). Adopt `6dd4527`'s release ordering from
`AccountedRemoteCommitBorrow` — `_release_current` keeps `_retained` until the
release succeeds, `_closed` is set only after it, and a `__del__` exists — and
reject its *reopen* behavior.
*Incompatible semantics:* the accepted tree's wrapper
(`_AccountedUploadCommitSource`) and the rejected MR-write tree's borrow
(`AccountedRemoteCommitBorrow`) are two different shapes over the same session,
with opposite retry correctness. The accepted one is the defective one. This is
the **B2** conflict: the shape the implementer must converge on is owned by the
Upload lane, not by MR-write.

### 5.3 `AuthorityFence` — **adapt**

Last complete project/session/subject observation.

*Reuse:* `remote_project_gate` (`codefire:2214`) with the accepted MR-list
`gate.fence()` / `prove_pinned_project_state("holding")` addition, which makes
the fence explicit and suppresses the redundant exit fence when already fenced
(`5c952a9a…` §4 `F1`, `b164046:2628`–`2632`).
*Adapt:* replace the second independent identity read in
`ordered_remote_merge_gates` (`codefire:2316`, `2328`) with a comparison
against the identity the acquired gate already holds, and against the ids
embedded in the request semantic (C6). Route `remote_project_stable_id`
(`codefire:2195`) through a request-owned bounded reader (C7).
*Incompatible semantics:* none. Note the ordering consequence: at the live base
the gate's exit fence at `2287`–`2305` runs *after* the route body but *before*
encoding, so a descriptor swap during the route currently converts a would-be
`200` into a `CFError`. With the accepted GET protocol the response is already
frozen before the fence, and the fence failure displaces it — a strictly
stronger order, and the one MR-write must adopt.

### 5.4 `PreparedResponse` — **adapt**, with one unassigned clause

Success/durable pair, transactional selection, one first-byte publication point.

*Reuse:* `_PreparedHTTPResponseOwner` (`codefire:13211`) unchanged — its
`close` (`13237`) already drops payload references before the release while
keeping `header_retained`/`body_retained` until the release succeeds, so it is
retryable, and `publish` (`13246`) has exactly one write pair.
*Reuse:* the pre-effect half of the accepted GET protocol — `prepare` →
fences → cleanup → teardown → `require_publishable_state` → `take` →
`publish`, with `displace_prepared_response` at the head of every arm
(`5c952a9a…` §2, §3, §4 `F2`/`F3`).
*Adapt:* give `_HTTPMutationResponseSlot` (`codefire:13426`) the accepted
publishability proof and state machine (C9), and make the mutation pair's
selection transactional (see below).

**The demonstrated defect** — reproduced by reviewer probe **P1** on the live
base (§7 row 7, appendix A):

`_PreparedHTTPMutationResponses._select` (`codefire:13293`) sets
`self.selection = requested_selection` at `13313` and only then calls
`rejected.close()` at `13314`, clearing both pointers at `13315`–`13316`. If
the rejected owner's ledger release fails, the exception propagates with
`selection` already recorded, `self.success` and `self.durable` both still
live, and `slot.selected` still `None`. `do_POST`'s `except BaseException` arm
(`14641`–`14645`) then calls `mutation_slot.close()`, which closes **both**
owners (`13480`–`13488`) — destroying the payload that was already selected.

Measured, live base, one-shot fault:

```text
select_success raised: OSError [Errno 5] injected release fault
pair.selection            = success
slot.selected             = None
success payload after slot.close() = None      ← the selected response is gone
ledger current after      = {'retained_bytes': 0}
open permits = False   terminal_reason = None
```

Measured, live base, persistent fault:

```text
slot.close() raised: OSError [Errno 5] injected release fault
success payload after slot.close() = None
ledger current after      = {'retained_bytes': 30}   ← stranded debt
open permits = False   terminal_reason = None
```

This class is **byte-identical in all five bound trees** (class-body SHA-256
prefix `104465a6dd8658f2`): the accepted Doctor/Branch composition, the
accepted MR-list candidate, the rejected Upload candidate and the rejected
MR-write candidate all inherit it unrepaired. It is the live-base root of
MR-write `F5`'s "cleanup can suppress the already-durable response".

**Required shape** (derivable): clear each pointer only after that owner's
release has succeeded, and record `selection` only once the transfer is
complete, so `_select` is exactly retryable; and never let a generic
`except BaseException` arm close a slot whose selection has already been
recorded.

**The unassigned clause — B3.** For the post-effect window the two frozen
specifications disagree:

| Source | Rule for "disposal/release failed" |
|---|---|
| `5c952a9a…` §4 `F2`/§4.5 (accepted GET) | publish **nothing**; `b""` on the wire; refuse to select when disposal fails |
| `214e42fc…` `F5` (MR-write) | **return** the selected success or durable-retry bytes; preserve the primary durable outcome; emit one terminal observation |

Both are satisfiable only under a phase index at the first authoritative
effect. The reconciliation stated in §6.4 is forced by the corpus, but **no
source states it**, the accepted report puts the POST successor outside its
write set (`5c952a9a…` §7 note 3, §9 required next action), and the snapshot's
disposition index routes it to a "shared-primitives / POST-precedence
successor" that exists as no frozen packet. Assigning that owner is a manager
act, not an implementer guess.

### 5.5 `RetryableCleanup` — **adapt**, **`shape-pending`**

An unreleased amount remains owned until retry succeeds.

*Invariant (frozen, unanimous across four reports):* no owner erases a claim,
scalar, handle, or payload accounting before the corresponding release has
actually succeeded (C12); a release that may partially commit compares before
and after and commits ownership only when the ledger actually changed
(`57d67921…` minimal closure); an over-release fails closed with a typed error
and changes nothing (`5c952a9a…` §4.7 `_release_exact`); cleanup faults are
recorded as secondary and never substitute for the primary (C14).

*Rejecting counterexample (frozen):* one `close(2)` returning `EIO`, with no
ledger fault injected anywhere, inside `_AccountedUploadCommitSource.close`
strands `11,456` retained bytes on the live request ledger with
`terminal_reason = None`, makes `source.close()` a silent no-op, and escapes
`handle_http_upload` as a raw `OSError` rather than a typed HTTP status
(`3df1db92…`). The same shape is present verbatim in the **accepted** tree at
`b164046:6994`–`6999`.

*`shape-pending`:* the exact mechanism that makes the still-owed reservation
reachable is open. `3df1db92…` names three alternatives — non-consuming
handles, a public retry surface, or an idempotent finalizer in the shape of
`AccountedHTTPBundle.__del__` / `_SealedSessionDescriptor.__del__` — and the
Upload lane is actively repairing it. MR-write must not freeze a competing
shape. The invariant and counterexample above are fixed and may be implemented
now; the wrapper repair may not.

### 5.6 `PublicationTxn` — **adapt**

Nonce, object, escrow, pending, recovery, receipt, audit, reducer confirmation,
replay.

*Reuse:* `execute_remote_json_publication` (`codefire:3285`) and the whole
pending/receipt/transaction/quarantine helper family
(`2450`, `2508`, `2531`, `2547`, `2603`–`2719`, `2895`, `2955`, `3220`), plus
the accepted recovery handoff shape — the caller's exact ledger and deadline
are passed into `recover_remote_pending_quarantine` and every recovered root
is validated on the request ledger (`57d67921…` `BLOCKER-1` CLOSED).
*Adapt:*
1. wire `ledger.seal_prepublication()` exactly once per mutating request
   (C15). Its semantics are already frozen by the base: `apply` (`525`–`526`)
   and `reserve_into` (`760`–`761`) refuse after the seal, while `release`
   (`882`) and `terminate` (`896`) remain legal. Every legal post-seal work
   item therefore needs a permit reserved before the seal;
2. bring escrow materialization inside the reducer's durability, or give it a
   separately durable staged transaction with exact progress and
   rollback/retry ownership (C16);
3. make the accepted replay nonce durable atomically with the replay/receipt
   transition, or reject the fresh nonce before selecting the recorded result
   (C17);
4. do **not** introduce a second publication-root proof: `RemoteJSONPublicationRootProof`
   exists only in the rejected Upload tree, where its
   `available/consuming/consumed/revoked` machine is CLOSED (C20). MR-write
   must leave that surface to the Upload lane.
*Incompatible semantics:* none between the reports. Two components are
unimplemented everywhere (`seal_prepublication` wiring; escrow-inside-reducer),
and one exists only in a rejected tree (root proof). None of these is a
contradiction; they are bounded work with a fixed specification, except that
`seal_prepublication` is neither idempotent nor publicly observable, so
"exactly once" needs the two product seams in §7 row 3.

### 5.7 `ErrorOrigin` — **adapt**

Filesystem, transport, deadline, budget, integrity, post-commit cleanup
precedence.

*Reuse:* the accepted taxonomy — `os_error_origin`,
`classify_route_os_error`, `route_filesystem_boundary`,
`mark_transport_origin` — which `5c952a9a…` §4 `F2` records as *shared* across
GET and POST with no second route-local protocol, and which independent probes
falsified for filesystem, transport, budget, `CFError`, deadline and
`MemoryError` origins (C18, C19).
*Adapt:* add the post-commit cleanup origin required by C11/C14 and Storage
`R3`/`R4`: after the first attempted response byte, no second document may be
emitted and the terminal class must be a publication/cleanup class rather than
`complete`.
*Incompatible semantics — one, in the accepted tree:*
`b164046:7047`–`7049` does

```python
    if cleanup_error is not None:
        if run_error is not None:
            raise cleanup_error from run_error
```

so the cleanup fault becomes the surfaced exception and the primary decision is
demoted to `__cause__`. That is the exact inversion `6438df44…` `R2` and
`3df1db92…` (constructor-unwind precedence) forbid, on a live upload route
(`b164046:14001`). Its live-base predecessor
`enforce_remote_commit_signature_policy` (`codefire:6587`) has no cleanup at
all and no accounting, so the defect is introduced by the accepted
composition — part of **B2**.

### 5.8 Decision summary

| Primitive | Decision | Marker | Blocking dependency |
|---|---|---|---|
| `RequestContext` | adapt | `frozen` | — |
| `SealedSnapshotSession` | adapt | `frozen` (borrow shape) | B1 (base), B2 (wrapper owner) |
| `AuthorityFence` | adapt | `frozen` | B1 (base) |
| `PreparedResponse` | adapt | `frozen` pre-effect; post-effect clause **unassigned** | **B3** |
| `RetryableCleanup` | adapt | **`shape-pending`** | Upload `HIGH-2` |
| `PublicationTxn` | adapt | `frozen`; two components unimplemented everywhere | — |
| `ErrorOrigin` | adapt | `frozen` | B1 (base), B2 (precedence inversion) |

No primitive is `incompatible`. No surface-local replacement is authorized.

---

## 6. Reconciliations

### 6.1 MR-write `F1`–`F7` → live production counterexample and repair seam

| Finding | Live production entry point | Live counterexample seam | Expected repair seam | Evidence state |
|---|---|---|---|---|
| `F1` request accounting excludes authority/identity/publication/projection | `POST …/merge-requests`, `…/review`, `…/apply` | routes take no ledger (`11616`, `11740`, `11867`); `read_json` `1429`, `2363`; `read_remote_branch_record` `20605`; generation `11965`; `remote_project_stable_id` `2195`; publication reopen `2926`, `3076`; canonical hashing `2412`, `3082`, `3113` with no poll | request-owned bounded readers + charged preimage/postimage/plan owners + iterative deadline polls + semantic-vs-gated identity fence | behavior-open |
| `F2` apply rereads sealed commits outside the session | `POST …/apply` | live apply has no session at all; `6dd4527`'s `AccountedRemoteCommitBorrow` reopens via `read_json_accounted` | purpose-bounded single-use handles from the retained session (C4) | behavior-open |
| `F3` exact/minus/plus and mutation non-vacuity incomplete | all three | no boundary evidence at base; at `6dd4527` plus-one exists only for response body | plus-one for every dimension and every new owner; killing reversals for `F2`, `F4`, `F5`, `F6` | behavior-open |
| `F4` ledger not prepublication-sealed; escrow outside reducer recovery | `POST …/merge-requests`, `…/review` | `seal_prepublication` has zero call sites in all five trees; `copy_object_graph` `11673` precedes the reducer `11674` | `ledger.seal_prepublication()` once before the first effect, permits reserved for all post-seal work; escrow inside the reducer or a durable staged transaction | evidence-unproven (seal) / behavior-open (escrow) |
| `F5` cleanup loses retry ownership and can suppress a durable response | `POST …/upload` today, all three after wiring | reviewer probe P1 on `_PreparedHTTPMutationResponses._select` `13313`–`13316` + `do_POST` `14641`–`14645` (§5.4) | clear pointers only after each release succeeds; record `selection` after the transfer; never close a slot whose selection is recorded; post-effect precedence per C11 | behavior-open, **reproduced live** |
| `F6` replay nonce not atomic with replay confirmation | `POST …/apply` recovery, `…/merge-requests`, `…/review` | live `record_request_nonce` `3759` writes the cache with `write_json` independently of the reducer `3285` | one durable transition, or reject the fresh nonce before selecting the recorded result | behavior-open |
| `F7` canonical source documentation stale | `docs/development/source-data-model-catalog.md` | the catalog still scopes the Python response owner to upload while the repair wires it into three MR routes | update all four canonical documents with the owner/route boundary change | behavior-open |

### 6.2 Newer predecessor findings

| Source | Finding | Disposition for this preflight |
|---|---|---|
| `5c952a9a…` (accepted) | `F1` final fence / teardown / proof; `F2` post-route filesystem displacement; `F3` publishability + zero debt; response-slot state machine; disposal precedence; recovered cleanup diagnostics; exact over-release | All CLOSED for the **read** route. Adopted as C5, C9, C10, C18, C19, C12. **Not** discharged for POST. |
| `5c952a9a…` §7 notes | 1 redundant retained state; 2 unbounded per-fault text; 3 GET/POST displacement asymmetry; 4 source-contract precision | 1, 2, 4 are bounded successor items. 3 is **B3**. |
| `3df1db92…` (Upload gen 3) | `BLOCKER-1`, `HIGH-3`, `MEDIUM-4` CLOSED; `HIGH-2` OPEN; `_control_charge_witness` aggregate read; two documentation over-claims | `HIGH-2` → C13 `shape-pending` and **B2**. `HIGH-3` → C20 (out of scope for MR-write). `MEDIUM-4` → C22 pre-allocation boundary discipline. Witness note → C25. |
| `57d67921…` (Upload gen 2) | `BLOCKER-1`/`HIGH-3` closed; `HIGH-2`/`MEDIUM-4` open; recovery handoff shape | Recovery handoff (ledger+deadline into `recover_remote_pending_quarantine`) adopted in §5.6. |
| `7632552e…` (Upload gen 1) | Formula table, `http_v1` profile, scale curves, four mandatory reversals absent or vacuous | Formula/owner-lifetime discipline adopted as C22; profile numbers independently reconfirmed at base. |
| `6438df44…` (Storage) | `R1` phantom nodes/absent edges; `R2` debug-only precedence; `R3` failed publication terminal `Complete`; `R4` second visible document; `R5` container/reopen owners; `R7`–`R10` blanket formulas; `R11` non-exhaustive terminal class; `R12` final path/parent authority; `R13` writer sequences | Imported as **cross-runtime invariant families only**, no Rust design import: C21 (`R1`), C14 (`R2`), C11 (`R3`, `R4`), C22 (`R5`), C25 (`R7`–`R10`), C18 (`R11`), C5 (`R12`), C23 (`R13`). |
| `c7a1665f…` (Doctor/Branch) | `MUST-1` closed; six-surface `read_route_phase` resolution; per-request flag reset | C1, C2. Its residual risk 1 (unrestricted exact-target discovery) remains manager-owned. |

### 6.3 The `seal_prepublication` seam

Because `apply` and `reserve_into` refuse after the seal while `release` and
`terminate` stay legal, "seal once before the first effect" is only
implementable if every post-seal work item already holds a permit. The live
mutation pair already does this for the response (`seal_for_effect`
`13319`–`13351` consumes both permits before the effect). The MR-write repair
must extend the same treatment to reducer, receipt, audit, and cleanup work.
Two product seams are required so the property is observable by an oracle
rather than by reading a private attribute (as `214e42fc…` `F4` had to):

- make `seal_prepublication` idempotent-or-typed so a second call is a typed
  error rather than a silent re-seal;
- expose a public read-only `prepublication_sealed` accessor.

### 6.4 The post-effect response rule — `frozen (synthesis)`

Stated here so it is not guessed, and marked as a synthesis because no single
source states it. It must be **assigned to an owner** before an implementer
relies on it (that assignment is part of clearing **B3**).

> Let `E` be the first authoritative effect (the first durable byte of object,
> authority, nonce, or transaction state).
>
> **Before `E`.** A disposal or release failure means **nothing publishes**.
> Dispose the older prepared response before selecting any replacement; if
> disposal fails, publish `b""` and close the connection; never publish a
> success while any dimension is non-zero or any permit is open.
> — `5c952a9a…` §4 `F2`, §4 `F3`, §4.5.
>
> **At or after `E`.** The already-selected success or durable-retry owner
> **must** publish. A cleanup or release fault is attached to the primary via
> the secondary-failure channel and never substitutes for it; exactly one
> terminal observation is emitted; the terminal reason is a cleanup/publication
> class and never `complete`; and no second document may be written after the
> first attempted byte.
> — `214e42fc…` `F5` (four postconditions) + `5c952a9a…` §4.5 (secondary,
> never substitution) + `6438df44…` `R3` (failed publication is not `Complete`)
> + `6438df44…` `R4` (no second visible document).
>
> The two halves are not in conflict once `E` indexes them: before `E` there is
> no committed outcome to report, so silence is safe; at or after `E` silence
> would hide a durable effect from the only party that can retry it, which
> `F5`'s live counterexample shows is the actual failure.

---

## 7. Counterexample and mutation matrix

Every row names a real live production entry point, an independent observable
oracle, and a deliberate regression. Deleted whole helpers, constant labels,
callbacks that skip the production route, incomplete snapshots, and blanket
terminal sweeps do not count. Rows 1–12 are the ticket's mandatory classes.
"Status" records what this preflight established: `reproduced` = executed here;
`frozen` = executed in a bound report and reproduced structurally in live
source here; `required` = specified for the implementer.

| # | Class | Live entry point | Deliberate regression | Independent observable oracle | Expected result | Status |
|---|---|---|---|---|---|---|
| 1 | Omit each physical owner before its first allocation/read/open | `POST …/merge-requests`, `…/review`, `…/apply`; `GET …/merge-requests` | remove one owner's `reserve`/`acquire` while leaving the read/allocation in place, for each of: descriptor, branch, MR, generation, policy, nonce preimage **and** postimage, pending, receipt, transaction, publication preimage/postimage/plan, projection, copy plan, reducer state | typed pre-allocation refusal observed **before** the allocation exists — the accepted `_charged_allocation_boundary` pattern (`02e04c7:21790`–`21831`), with the memo's `__iter__` or the reader's first `read`/`open` as the witness, plus a ledger snapshot showing `0 of N` proved | each omission raises a typed boundary error and the allocation never happens | required — `214e42fc…` `F3` records that the policy-owner and nonce-owner omissions **survive** at `6dd4527`; `3df1db92…` `MEDIUM-4` gives the working oracle shape |
| 2 | Restore one second physical read after sealed snapshot capture | `POST …/apply` (ancestry + signature history) | reintroduce a second loader after the session has retained the record | `object_records` must equal physical object-record reads at exact, minus-one **and** plus-one; instrument at the file-open boundary, not at helper names | exact `object_records = L` must fail when physical reads exceed `L` | frozen — measured at `6dd4527`: exact `object_records=10` returned `200` while `record_reads=13`; apply gap grows `1/3/10` (`214e42fc…` `F2`) |
| 3 | Move response preparation after escrow or publication | all three MR routes | keep the live base order (`prepare_json` at `codefire:14675`, after the effect) instead of preparing the pair before the effect | wire bytes captured over a real socket **and** the ledger's `response_bytes` phase order; the prepared success bytes must be byte-identical to the published bytes | preparing after the effect must fail: a post-effect encoding limit would otherwise convert a committed effect into a `413` with no durable-retry response | required — the base order is the regression, already live |
| 4 | Move one object/postimage allocation before its reservation | `POST …/merge-requests` (`copy_object_graph` `11673`), `POST …/upload` (`write_object_records` `12337`) | allocate the `object_record_core` dict / postimage before its `acquire` | the 64-byte-slot defect (`214e42fc…` `F1`): compare the charged amount with the canonical size of the dict actually built; oracle = ledger delta across the allocation, not a helper call count | charge must dominate the real dict, and the projection must not outlive its accounting owner | required |
| 5 | Mutate authority after the last observation | all three MR routes + `GET …/merge-requests` | after the final fence, replace branch, MR, generation, policy, signature, object record, project descriptor, mode/link/inode, or readable state — including same-metadata replacement and extension-only change | typed fence failure with expected/actual evidence; frozen projection unaffected; no partial `200` | mutation after the fence has no effect on the published bytes and, when observable, fails closed | frozen for the read route (`5c952a9a…` §8 rows 4, 5; `_PinnedAuthorityChanged` → `merge_request_fence_changed` closed at `6dd4527` per `214e42fc…` prior-`F5`); required for MR-write |
| 6 | Fail every first cleanup release, then retry the same exact owner | all three MR routes, `POST …/upload` | one-shot and persistent `ledger.release` faults; **and** a realistic single `close(2)` returning `EIO` with no ledger fault at all | after the retry: zero debt, zero permits, exactly one release per owner, primary error preserved; oracle = ledger `current` plus a per-owner release counter | retry drains the debt; nothing double-releases | frozen — `close(2)` variant strands `11,456` bytes with `terminal_reason=None` and a silent no-op retry (`3df1db92…`); `57d67921…` records the four public-API variants |
| 7 | Partially consume response/header/body permits and attempt durable fallback | `POST …/upload` today; all three MR routes after wiring | fail the rejected owner's release inside `_PreparedHTTPMutationResponses._select` (`codefire:13314`) after `selection` is recorded (`13313`) | wire bytes over a real socket; `pair.selection`; `slot.selected`; the selected owner's payload identity; ledger `current`; `terminal_reason` | the selected success/durable bytes must still publish, the fault must be recorded as secondary, and the terminal reason must not be `complete` | **reproduced** — see §5.4 and appendix A: selected payload destroyed, `b""` publishable, `retained_bytes=30` stranded under a persistent fault, `terminal_reason=None` |
| 8 | Fail the second of several object writes and then exact-retry | `POST …/merge-requests`, `POST …/review` | inject a failure on the second `write_json` inside the escrow copy | durable snapshot diff of the target object store, the pending marker, and the transaction directory; the exact same signed request replayed | after the failure a durable record must exist that the exact retry can resume; the retry must converge to the same committed outcome | frozen — at `6dd4527` this returns `503 retry_same_request` leaving one of four objects written, MR and branch unchanged, and **neither** a pending marker nor a transaction directory (`214e42fc…` `F4`) |
| 9 | Fail reducer confirmation before and after nonce persistence | `POST …/apply` recovery, `POST …/merge-requests`, `POST …/review` | inject a reducer failure (a) before `record_request_nonce` and (b) after it | replay the exact signed request; observe the nonce cache bytes and the receipt state | every `503 retry_same_request` must remain retryable with the same signed bytes | frozen — at `6dd4527` the post-nonce case makes the exact retry return `400 nonce has already been used`; only a third nonce recovers the recorded `200` (`214e42fc…` `F6`) |
| 10 | Alternate Doctor, Branch, Upload, MR-list and MR-write on one server | one `ReusableThreadingHTTPServer` (`codefire:12621`), sequential and pipelined | leave one per-route deadline flag unreset, or share one response-phase flag between two routes | per-request observer snapshot: route phase labels, `poll_count`, ledger `current`, permit count, cleanup debt, terminal reason, and the exact published bytes for each of the nine live entry points | no request flag, ledger, phase, debt, or response leaks into the next request; `HTTP/1.0` never serves a pipelined second request | frozen for the read routes (`c7a1665f…` conflict item 6; `5c952a9a…` §8 route-isolation tests incl. the eight-request sequence); required with the three MR routes added |
| 11 | Exact, minus-one and plus-one for every charged dimension | all nine live entry points | drive each dimension to `L`, `L-1`, `L+1` and prove the **intended** owner fails, not an earlier limit | typed `ResourceBudgetExceeded` naming the intended dimension and phase, `observed = limit + 1`, exit code `60`, zero debt, zero permits; plus a dominance check that no earlier limit binds first | the intended owner is the first to refuse at every boundary | frozen as a gap — at `6dd4527` only response-body bytes have plus-one; descriptor, MR/branch/generation, recovery/pending/receipt, publication, projection, copy and cleanup owners have **no owner at all** (`214e42fc…` `F3`, boundary audit) |
| 12 | 1x/3x/10x curves | all nine live entry points | scale logical records, physical reads, descriptors, bytes, retained, temporary, permits and deadline polls | measured curves per dimension with an independently hand-derived oracle, not a candidate-observed snapshot replayed | each curve is the intended logical curve; no phantom units; no dimension grows independently of its charged counterpart | frozen as a gap — `6dd4527` create/review are stable but apply's `record_reads` gap grows `1/3/10` (`214e42fc…` `F2`); Upload's 1x/3x/10x are measured and stable (`7632552e…`); Storage's node curve is defective by construction (`6438df44…` `R1`) |

Additional rows this preflight adds, each with a live entry point and an
independent oracle:

| # | Class | Live entry point | Deliberate regression | Oracle | Expected result | Status |
|---|---|---|---|---|---|---|
| 13 | Prepublication seal is absent | all three MR routes | keep the current state — no `seal_prepublication` call — and perform an ordinary `ledger.apply` after the effect starts | public `prepublication_sealed` accessor (§6.3) plus a post-effect `apply` attempt | an ordinary post-effect `apply` must raise `decision-work ledger is sealed for publication` (`codefire:525`–`526`) | **reproduced** as an absence: zero call sites in all five trees |
| 14 | Cleanup fault displaces the primary decision | `POST …/upload` in the accepted tree | keep `raise cleanup_error from run_error` (`b164046:7047`–`7049`) | the exception type/message the caller observes when the primary is a typed decision and the cleanup fault is unrelated | the caller must observe the primary decision; the cleanup fault is secondary | **reproduced** by source identity; measured for the constructor-unwind twin in `3df1db92…` (`p4` case 5) |
| 15 | Second identity read replaces the fence | `POST …/merge-requests` | keep `ordered_remote_merge_gates`' own re-read (`codefire:2316`, `2328`) instead of comparing against the gate-held identity | swap the descriptor between the semantic read and the gate read; observe whether the published semantic still names the old identity | the request must fail closed on the semantic identity, not silently proceed | frozen — `214e42fc…` `F1`; reproduced structurally in live source |
| 16 | Unbounded authority read escapes every limit | `POST …/apply` | keep the plain `read_json` reader (`codefire:1429`, `2363`) | install a structurally valid, status-inadmissible MR record of ~2 MiB and send a real socket apply; read `aggregate_bytes` and the terminal reason | the read must be charged and deadline-bounded | frozen — at `6dd4527` a `2,098,457`-byte record is fully read and parsed, returns `409`, reports `aggregate_bytes=0`, and terminates `complete` (`214e42fc…` `F1`) |

---

## 8. Focused validation plan and expected failures on `6dd4527`

Ticket order: focused first, then neighboring, then broad. Nothing broader than
the Python-pattern sweep can change a preflight decision, so nothing broader is
proposed here.

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile \
  codefire tests/test_python_http_mr_mutation_decision_work.py \
  tests/test_codefire_cli.py

PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  tests.test_python_http_mr_mutation_decision_work -v

# neighbouring: the exact 18 legacy MR/signature/nonce/replay/escrow/
# branch-protection ids enumerated in 214e42fc… "Commands and results",
# plus the 7 response-owner ids enumerated in 5c952a9a… §6 "Affected legacy CLI scope"

PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tests -p 'test_python_*.py'
python3 docs/development/tools/check_source_docs.py
python3 docs/development/tools/check_project_docs.py
git diff --check
```

Expected non-green results when the **repaired** focused module is run against
`6dd4527f23135388b53fc5c6d637c4105cc5d659`, with the exact reason each must
fail — these are the regression oracles the repair has to earn:

| Test intent | Expected outcome on `6dd4527` | Reason |
|---|---|---|
| `object_records` equals physical object-record reads at exact/−1/+1 on apply | FAIL | exact `10` charged vs `13` reads; `tests/…:667` currently encodes the mismatch as the expected value, so a correct single-session change requires rebaselining that assertion |
| handler ledger is prepublication-sealed exactly once before the first effect | FAIL | `seal_prepublication` has zero call sites; a post-effect ordinary `apply` still succeeds and the route still returns `200` |
| mid-copy escrow failure leaves a resumable durable record | FAIL | `503 retry_same_request` with one of four objects written and no pending marker or transaction directory |
| cleanup retries the exact failed owner without losing the scalar | FAIL | scalars are cleared before the release; a second `close()` marks the lane closed with debt `37` outstanding |
| a post-publication cleanup fault still publishes the selected bytes and one terminal observation | FAIL | no HTTP response reaches the client; `terminal_reason=None`; retained debt `2` |
| a `503 retry_same_request` after reducer failure stays retryable with the same signed bytes | FAIL | the fresh nonce is already durable, so the exact retry returns `400 nonce has already been used` |
| every authority read is charged and deadline-bounded | FAIL | a `2,098,457`-byte MR record is read with `aggregate_bytes=0` |
| plus-one boundary for every charged dimension and every new owner | FAIL | plus-one exists only for response-body bytes |
| omitting the policy owner / the nonce postimage owner is detected | FAIL | both omissions currently survive |
| canonical source docs match the wired owner boundary | FAIL | the data-model catalog still scopes the Python response owner to upload |

Historical non-vacuity must be preserved: the repaired module must remain
non-green against `21fe75175e997cc7c6f8c3ef2a7558144d850f94` and
`e2a218e015953f082245f36dbc6aee906b837e5a` (`214e42fc…` recorded 23F/4E and
33F/27E respectively). A missing symbol is an **error**, not proof of a defect,
and must be reported as such.

Environment note carried forward, not re-derived: `cargo` and `rustc` are
absent in this environment, so `tests.test_codefire_cli`'s
`test_completion_command_and_install_completion` and
`test_install_script_installs_codefire` fail identically at every base in this
chain with byte-identical `tests/test_codefire_cli.py` and `install.sh` blobs
(`5c952a9a…` §6). They are inherited environment failures and must not be
attributed to any candidate.

---

## 9. Bounded MR-write write set and conflict map

### 9.1 Proposed write set

The frozen packet
`docs/development/ai/swarm-batches/cfr168-python-http-mr-write-review-closure-repair-20260730.json`
(SHA-256 `eac34a87a4668d43ce8c278553f67ffa477fa8beb36f2b63aae2e48beb2457ee`;
its Markdown ticket `b075bb331095b5ce8790907a7a151b8899dc65cd4a3879077930de61a456a568`)
declares exactly seven paths, and the set is **bounded and sufficient** for
F1–F7:

| Path | Needed for |
|---|---|
| `codefire` | C1–C22 product work |
| `tests/test_python_http_mr_mutation_decision_work.py` | rows 1–16 of §7 |
| `tests/test_codefire_cli.py` | neighboring legacy MR/signature/nonce/replay/escrow/branch-protection and response-owner cases |
| `docs/development/source-architecture.md` | C24 |
| `docs/development/command-source-trace.md` | C24 |
| `docs/development/source-data-model-catalog.md` | C24 / `F7` |
| `docs/development/change-playbook.md` | C24 |

No addition is required. Explicitly **outside** the set: Rust, file transport,
packaging, devcontainer, dogfood, `POST …/gc`, the Upload wrapper repair, the
publication-root proof, and every other lane's report or packet.

The packet's `base_revision` is **not** usable as declared — see §9.3.

### 9.2 Conflict map — one `codefire` file, four lanes

| Surface | Lanes that touch it | Required resolution | Risk if unresolved |
|---|---|---|---|
| `do_POST` (`codefire:14470`) | MR-write (all three routes + slot), Upload (slot handoff), MR-list (POST asymmetry successor) | exactly one handler ledger/deadline, one slot transfer, one displacement precedence, phase-indexed per §6.4 | two competing displacement protocols in one function |
| `do_GET` (`codefire:14391`) | Doctor/Branch (`read_route_phase`), MR-list (`_HTTPReadResponseSlot`, arms) | adopt the accepted GET protocol verbatim; MR-write must not touch it | read-route regression |
| `_PreparedHTTPMutationResponses` / `_HTTPMutationResponseSlot` (`13262`, `13426`) | **all four lanes, byte-identical today** | one repair, once, owned by whoever lands first; the others rebase onto it | four divergent copies of the same class |
| `_PreparedHTTPResponseOwner` (`13211`) | all four | reuse unchanged | none identified |
| `SealedCommitValidationSession` (`20735`) | MR-write (`F2` handles), Upload (`HIGH-2` control release), Branch (`finalize`) | one session, one borrow shape, one release transition | three release protocols |
| `_AccountedUploadCommitSource` (`b164046:6946`) | Upload (repair), accepted tree (inherited defect) | Upload-lane repair, then MR-write consumes it | MR-write forks a second wrapper |
| `remote_project_gate` / `ordered_remote_merge_gates` (`2214`, `2313`) | MR-list (`gate.fence()`), MR-write (`F1` identity fence) | one fence API, additive | two fence protocols |
| Publication reducer family (`2450`–`3639`) | Upload, MR-write, GC, file transport | add accounting and escrow progress without duplicating the reducer; keep the no-ledger file path working | duplicated reducer |
| Error-origin taxonomy (`b164046` only) | MR-list (introduced), MR-write (`F5`/`F1`), Upload | reuse the accepted taxonomy; add only the post-commit cleanup origin | second route-local taxonomy |
| `read_route_phase` (`701469f:14193`) | Doctor/Branch (introduced), MR-write (mutation phases) | extend, do not replace | duplicated phase logic |
| `DecisionWorkLedger.seal_prepublication` (`862`) | MR-write (`F4`), all future publications | one wiring, plus the two seams in §6.3 | permanently dead invariant |
| Four canonical documents | all lanes | serialize; each lane states only its own live truth | contradictory canonical docs |
| `POST …/gc` (`11502`) | none in this lane | remains predecessor-gated; no uniform POST nonce-timing claim | false uniformity claim |

### 9.3 The base conflict — B1 in full

| Fact | Value |
|---|---|
| Packet `base_revision` | `fcb2324145962e79d7c6ee173b9cf545733b957d` |
| `fcb2324:codefire` | byte-identical to `6dd4527:codefire`, content SHA-256 `74a6cd8f9ae78de10d1042db84c89d2035abc6cda5ebc993cc42015cb7c9d59b` |
| Commits `6dd4527..fcb2324` | `b86a782` (the review report), `fcb2324` (the frozen packet) — no product change |
| Accepted primitives present in `fcb2324` | **none** of `read_route_phase`, `_HTTPReadResponseSlot`, `require_publishable_state`, `prove_and_take`, `displace_prepared_response`, `prove_pinned_project_state`, `os_error_origin`, `classify_route_os_error`, `route_filesystem_boundary`, `mark_transport_origin`, `_release_exact`, `close_with_retry`, `record_secondary_failure`, `cleanup_diagnostic` |
| Packet's declared ancestry | `e2a218e`, `21fe751`, `6dd4527`, `b86a782` — the accepted `701469f` and `b164046` are **not** named |

The packet defers composition explicitly ("The manager serializes the full
Python suite, composition gates, and install checks after all predecessor
candidates are accepted"; the handoff asks for "expected composition
conflicts"). That was defensible before this preflight. It is no longer, because
this preflight's own rule — surface-local replacement is `BLOCK` — cannot be
satisfied on a base where the primitives do not exist. The implementer would
have to either re-implement fourteen accepted symbols (forbidden) or silently
pick a different base (a guess).

Required manager action, stated so it is not a guess: rebind the MR-write
repair packet's `base_revision` to a revision whose product tree contains both
`701469f91dd0c4c52c5628fb3e9d9e9d0ab0c714` and
`b16404676639336f705feffd9af4857d4a1fcc69` — `b164046` itself is the only
currently existing such tree — or integrate those two accepted candidates
first and freeze the repair on the composed result. This is a product
composition decision and is outside this preflight's authority.

---

## 10. Residual predecessor gates

| Gate | Owner | State |
|---|---|---|
| Upload `HIGH-2` closure on an additive successor of `02e04c7` + fresh independent acceptance | original Upload session | **open** — blocks C13 and, through `b164046:6994`–`6999`, **B2** |
| Owner for `_AccountedUploadCommitSource` as it exists in the **accepted** trees `701469f` / `b164046` | unassigned | **open** — **B2** |
| Owner for the POST displacement-precedence successor (§6.4) | unassigned; `5c952a9a…` §7 note 3 puts it outside the MR-list write set | **open** — **B3** |
| MR-write repair `base_revision` rebinding | manager | **open** — **B1** |
| Manager integration decision for accepted `b164046` | manager | open; `5c952a9a…` required next action |
| Manager integration decision for accepted `701469f`, and its residual unrestricted exact-target Python discovery | manager | open; `c7a1665f…` residual risk 1 |
| MR-list non-blocking observations 1, 2, 4 | manager-routed successor | open; not preconditions |
| `seal_prepublication` wiring plus the idempotence/accessor seams | MR-write repair (spec frozen) | open |
| Python composition review after all candidates are accepted | manager | open |
| `POST …/gc` nonce/authorization timing | separate predecessor gate | correctly outside |
| Cargo/Rust toolchain | environment | absent; two inherited `test_codefire_cli` failures explained at base |

---

## 11. Next action

1. **Manager, B1.** Rebind the MR-write closure repair packet's
   `base_revision` from `fcb2324145962e79d7c6ee173b9cf545733b957d` to a
   revision containing both `701469f9…` and `b164046…`, or integrate those two
   accepted candidates and freeze the repair on the composed tree. Record the
   chosen revision and its `codefire` content digest in the packet.
2. **Manager, B2.** Assign an owner for `_AccountedUploadCommitSource` as it
   exists in the accepted trees — either fold it into the Upload `HIGH-2`
   repair and require the accepted trees to be rebased onto the result, or
   open a bounded shared-primitive repair packet for
   `b164046:6994`–`6999` and `b164046:7047`–`7049`. Until then the accepted
   product bytes contradict report `3df1db92…` about the same symbol.
3. **Manager, B3.** Assign an owner for the POST displacement-precedence
   successor and adopt §6.4 as its frozen contract, or fold §6.4 into the
   MR-write packet as an explicit required clause with all four citations. It
   must not remain an implementer inference.
4. **Then** re-run this preflight against the rebound base. Sections 2–9 are
   reusable as written: the primitive decisions, the clause register, the
   owner/lifetime table, the timelines, the counterexample matrix, the
   validation plan, and the write set do not depend on the base choice — only
   B1's reuse feasibility does.
5. Only after a committed `READY` from that re-run, with this report's
   successor commit and SHA-256 bound into the MR-write closure packet, may the
   MR-write implementation manifest be created or started.

`READY` would have authorized only the MR-write repair packet. This `BLOCK`
authorizes nothing at all: no Upload integration, no candidate acceptance, no
rejected product byte, no integration, Close, Install, or Dogfood.

---

## Appendix A — reviewer probe

One executable probe was run, against the live base `codefire` at `2c439d8`,
to convert the `PreparedResponse` post-effect claim from a source reading into
a measured counterexample. It drives the production classes
`_PreparedHTTPResponseOwner`, `_PreparedHTTPMutationResponses` and
`_HTTPMutationResponseSlot` loaded with `runpy.run_path`, the same way the
checked-in suite loads them, and reproduces the exact sequence `do_POST`
performs at `codefire:14611` and `14641`–`14645`.

| Probe | SHA-256 | Fault |
|---|---|---|
| `/tmp/probe_select.py` | `8b8b2e915d358e7c2b240201bca06dd825fcc04d5cd3054402500f60a980ffcd` | one-shot `release` fault on the rejected owner |
| `/tmp/probe_select_persistent.py` | `d2db80408b943e19713ca781ce925bd9c2dba150a2bd3c0913e4a14f7ecbb4cb` | persistent `release` fault |

Measured output is quoted verbatim in §5.4. The probe is a **recorder plus
oracle**: its oracle is the identity of the selected owner's payload after the
handler's own cleanup arm runs, which is independent of any method name.

Limitation stated honestly: this probe exercises the production classes
directly rather than over a real socket, because the live base has no MR-write
route that prepares a response pair at all — the class is only reachable from
`handle_http_upload` today. Once the pair is wired into the MR routes the same
regression must be driven over a real socket, as §7 row 7 requires.

## Appendix B — commands

All read-only unless noted. `git` 2.34.1, `python3` 3.10.12, Linux
6.6.87.2-microsoft-standard-WSL2. `cargo` and `rustc` absent.

| Command class | Result |
|---|---|
| `git rev-parse` / `git log -1` on all 14 bound revisions | all resolve; subjects and dates recorded in §1 |
| `git merge-base --is-ancestor` over the 5×5 tree matrix and the four lane bases | §1.4 relations confirmed |
| `git show <report-commit>:<review.md> \| sha256sum` ×7 | all six ticket-supplied digests match; the Doctor/Branch digest recomputed as `c7a1665f…` |
| `git show <candidate>:codefire \| sha256sum` ×6 | `b164046` and `02e04c7` match their own reports' changed-path tables; `HEAD` == `16d0129` |
| symbol census over the five exported `codefire` trees | §1.5 |
| class-body digest of `_PreparedHTTPMutationResponses` in five trees | identical, prefix `104465a6dd8658f2` |
| `python3 -c` JSON read of the frozen MR-write packet; `sha256sum` of packet `.json`/`.md` | §9.1, §9.3 |
| `PYTHONDONTWRITEBYTECODE=1 python3 /tmp/probe_select*.py <live codefire>` | appendix A; §5.4 |
| `git status --porcelain` at start and before handoff | clean / only the declared report |

Not run and not claimed: any Cargo/Rust command, any build, install, `demo.sh`,
dogfood, Docker, unrestricted all-language discovery, the focused MR-write or
MR-list unittest modules, the broad Python-pattern sweep, and any product or
test mutation. None of those could change this decision, whose three blocking
conditions are revision-level and ownership-level facts.
