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
> audit of `a7e283e` found one hard contract contradiction. The four canonical
> source docs pass through **two sequential stages**: the step-5 three-way union,
> and the step-7 hand-authored `O1` citation, which by construction edits two of
> those same four files. `a7e283e`'s single `I5` check nevertheless required the
> **final** committed files to match all four **pre-citation** union blobs, which
> is unsatisfiable for `docs/development/source-architecture.md` and
> `docs/development/command-source-trace.md` and made `I5` ambiguous at best. This
> successor correction separates the two stages, publishes both blob sets as full
> 40-character values, and splits `I5` into `I5-UNION` and `I5-FINAL`.
>
> Correction 2 changes **nothing else**: the 18 + 1 + 1 topology, the routed-
> residual owner map, the `B1`/`B2`/`B3` adjudication, the accepted inputs, the
> product behaviour, the scope, the read/write sets and every other invariant and
> gate are carried through unchanged. `806f6c8` and `a7e283e` both remain
> immutable history and neither is amended, rebased or reset. The published
> composition preflight is also unchanged and remains valid **as historical
> intermediate-union evidence** — its four blob prefixes are the step-5 union
> stage, never the final committed candidate bytes. Because no commit can name
> itself, this correction carries no SHA of its own; it simply extends the
> manager-integrated Plan/FREEZE chain, and the launch base `L` is the final
> integrated docs-only `HEAD` that results.

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
manager-integrated Plan/FREEZE chain head, producing an exact **three-commit
linear chain**: candidate `C` of exactly 18 paths, writer result `W` of exactly
one result path, and reviewer report `R` of exactly one review path. The
aggregate writer write set is 19 paths, split 18 + 1 across `C` and `W`. It is
**not** one commit, and the writer result is **not** part of the candidate.

`C` alone is the immutable review subject. The product base of `C`'s content
remains `a972cde890e0d621aaced6aca4b45835871da975` at blob level.

This packet authorizes **composition of already-accepted bytes only**. It does
not authorize MR-write, integration of any of `C`, `W` or `R`, Install, Dogfood,
any Cargo/Rust work, any new product behaviour, or any lifecycle exit.

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
- Nothing must be guessed: every source blob, three-way base, expected target blob and omission is named in this packet and in the bound preflight.

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
- `B3` does **not** prevent composition of already-accepted bytes, for one reason only: the composition must not change or claim the `B3` surface. `_PreparedHTTPMutationResponses`, the handler `POST` arm, and the request-loop `finally` are carried through byte-identical to their accepted sources; the new observer test only **reads** them. A composition that altered that surface, or that claimed a phase index for it, would be outside this authorization.

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
| 1 | `codefire` | `C` | three-way semantic union, mode preserved `100755` |
| 2 | `tests/test_codefire_cli.py` | `C` | clean apply from `b164046` (base-equal path) |
| 3 | `tests/test_python_http_doctor_decision_work.py` | `C` | `ADD` from `b164046` |
| 4 | `tests/test_python_http_branch_decision_work.py` | `C` | `ADD` from `b164046` |
| 5 | `tests/test_python_http_mr_list_decision_work.py` | `C` | `ADD` from `b164046` |
| 6 | `tests/test_python_http_upload_accounted_traversal.py` | `C` | `ADD` from `0f43415`, duplicate omitted, newest wins |
| 7 | `docs/development/source-architecture.md` | `C` | three-way union + `O1` citation |
| 8 | `docs/development/command-source-trace.md` | `C` | three-way union + `O1` citation |
| 9 | `docs/development/change-playbook.md` | `C` | three-way union |
| 10 | `docs/development/source-data-model-catalog.md` | `C` | three-way union |
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
`git reset --hard` and no `git checkout --` over user changes; no integration of
`C`, `W` or `R`; no `Close` on any root, issue or cycle; no edit to the original
shared-primitives preflight or the published composition preflight; no self-review
by the writer; no implementation of `C14-LIVE-A1`, `C14-LIVE-A2` or
`C14-SUCCESSOR-OWNER`.

## Exact semantic procedure

Semantic apply throughout. Both candidate lineages carry management docs that
would clobber the canonical governance and Cargo lineage, so no range may be
cherry-picked.

1. **Verify start state, abort on any mismatch.** `HEAD` equals the launch base `L` named by the manager; `git rev-parse HEAD^{tree}`; `git rev-parse HEAD:codefire` = `61214a21fd866efc0d9a8888dfbfc8e707039d48`; `git status --porcelain` empty; every step-3 condition of the execution-base rule proven, including linear docs-only descent from `a972cde`; all `ADD` paths absent.
2. **Product fast-forwards from `b164046`.** `b164046` supplies `codefire`, `tests/test_codefire_cli.py`, and the Doctor, Branch and MR-list decision-work tests. `701469f` is subsumed and is not replayed. The base blob is identical on `tests/test_codefire_cli.py`, so only ours changed it and the apply is clean.
3. **Merge the Upload successor onto `codefire`.** Three-way `git merge-file -p --diff3` with ours `b164046:codefire` `f30575a697c1b525653c13738987ccdb5c0151ea`, base `cb13da0:codefire` `03966af23dd860c5235832320a8f1977ecd6495a`, theirs `0f43415:codefire` `b984c1e7e393af658a618c96270d33d39bf8eeaa`. Not `git apply` against the working tree: the working `codefire` is now ours and the delta's context is the base's. Expect exit 0, **zero** conflict markers, exactly additive line arithmetic, mode preserved `100755`, and target blob **`f5e0b6db7c15aeadd3025ae4f038e301a7634caf`**. Recompute with `git hash-object codefire` — note the bound preflight's `W2` pseudocode contains the typo `codefier`; use `codefire`.
4. **Upload test module: newest wins, duplicate omitted.** Take `0f43415`'s 36-test module, blob `829836bc0411bf82030286e6f1b0e3d58e574e14`, **not** `b164046`'s 10-test duplicate `141dcbd4ee1f8727fbe146faef481cf746c8c73f`. Ours equals base on this path, so this is a clean fast-forward, not a merge.
5. **Stage one — three-way union the four canonical source docs**, with `cb13da0` as base, ours `b164046`, theirs `0f43415`. This is an **intermediate** stage: two of these four files are edited again in step 7, so these blobs are *not* the committed candidate bytes. Reconstruct them, in a scratch location or in the working tree before any hand edit, with `git merge-file -p --diff3 <ours> <base> <theirs>`, expecting exit 0 and zero conflict markers on each. The four **pre-citation union blobs**, in full:

   | Path | Pre-citation union blob (stage one) |
   |---|---|
   | `docs/development/source-architecture.md` | `d8d7aa9881b5f96bcfa13b78168403be9ae62512` |
   | `docs/development/command-source-trace.md` | `0a8d2a886c883a051fe8c0e74bf550da3821b977` |
   | `docs/development/change-playbook.md` | `9a3afd7dd932bd3db7f33909e3cda40fb867abfc` |
   | `docs/development/source-data-model-catalog.md` | `2b68347eaa357f187f794c2a76992401743929eb` |

   These are the values the bound preflight abbreviates as `d8d7aa9881b5…`, `0a8d2a886c88…`, `9a3afd7dd932…` and `2b68347eaa35…`; the preflight is correct and unchanged, and it describes **this stage only**. Expect zero conflicts, zero heading collisions, zero duplicate headings. Preserve the Upload-side scoping sentence that bounds the synchronous drain guarantee to the two commit-source routes and to nothing else.
6. **Add only the exact listed evidence files** — aggregate paths 11 to 17. Do not import the five canonical-newer management docs, any agent state, or any other candidate doc.
7. **Stage two — hand-authored edits inside `C`.** Add the `O1` **relative** citation to the existing in-tree shared-primitives preflight at the `C14`/`B2` mentions in paths 7 and 8 — a repo-relative link to `ai/swarm-results/cfr168-python-http-shared-primitives-preflight-claude-20260731/preflight.md`, resolving from `docs/development/`. This citation, and nothing else, is what moves paths 7 and 8 off their stage-one union blobs. Paths 9 and 10 carry no citation and are therefore **byte-identical to their stage-one union blobs** in the committed candidate. Register `C14-LIVE-A1`, `C14-LIVE-A2` and `C14-SUCCESSOR-OWNER` as OPEN in path 18, using the ownership map above verbatim in effect and implementing none of them.

   The four **final candidate blobs** committed in `C`, in full:

   | Path | Final blob in `C` | Relation to stage one |
   |---|---|---|
   | `docs/development/source-architecture.md` | `5a1755ebd7e0eff8f55f92ae940557badb3ca567` | union `d8d7aa98…` + `O1` citation only |
   | `docs/development/command-source-trace.md` | `72953356fca0982596bbb81a939fd4e2f2584a75` | union `0a8d2a88…` + `O1` citation only |
   | `docs/development/change-playbook.md` | `9a3afd7dd932bd3db7f33909e3cda40fb867abfc` | unchanged from stage one |
   | `docs/development/source-data-model-catalog.md` | `2b68347eaa357f187f794c2a76992401743929eb` | unchanged from stage one |

   The stage-one → stage-two delta on paths 7 and 8 must contain **only** the `O1`
   citation and the minimal reflow it forces. No other sentence, claim, heading or
   ordering may change, and in particular the `C14`/`B2` statements keep their
   existing meaning: the citation names where the authority lives, it does not
   restate, narrow or resolve it.
8. **Freeze `C`.** One clean additive commit containing exactly aggregate paths 1-18 and nothing else: `git rev-list --count L..C` = 1; `git rev-list --merges L..C | wc -l` = 0; `git diff --name-status L..C` exactly the 18 paths; `git status --porcelain` empty. `C` contains no reference to its own identity.
9. **Commit `W` on top of `C`.** Exactly one `ADD` at aggregate path 19, recording `C`'s SHA, tree, `git diff --binary L..C | sha256sum`, the six product blob ids and `codefire` mode, **both** canonical source-doc blob stages in full — the four stage-one pre-citation union blobs *and* the four stage-two final candidate blobs, with the two that differ identified and their `O1`-citation-only delta shown — the omitted duplicate patch-id, the test counts, every gate and reversal result, and every item not run marked not run. `git diff --name-status C..W` is exactly that one path; `git diff C..W -- <the 18 candidate paths>` is empty. `W` must state truthfully that `C` is composed-not-accepted, that it closes neither CFR-168, CFB-136, CFB-138, global `C14`, `B3`, MR-write, integration, Install nor Dogfood, and that `C14-LIVE-A1`, `C14-LIVE-A2` and `C14-SUCCESSOR-OWNER` are registered OPEN and unimplemented.
10. **Then, and only then, the fresh report-only review starts** and produces `R` on top of `W`.

### Anti-false-closure rule

No composition text may apply `Close`, `Closed`, `complete`, `zero-residual` or
`ACCEPT` to CFR-168, CFB-136, CFB-138, global `C14`, MR-write or the cycle. Only
two narrow claims are licensed by evidence: `HIGH-2` is closed on all three limbs
**for candidate `0f43415`** per report SHA-256 `46e7c3e4…`; and the `B2` product
contradiction between `b164046` and the shared-primitives preflight is
**measured as eliminated on the composed tree**, which is a candidate-scoped
measurement and not a repository-level resolution of `B2`.

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
- Introducing any new product behaviour, refactor, rename or optimisation not present in an accepted source blob.
- Authoring a new `C14`/`B2` authority document; the `O1` remediation is a citation only.
- Treating the bound preflight's four abbreviated doc blobs as the final candidate bytes, or requiring the committed architecture and trace files to match their pre-citation union blobs; they are the stage-one intermediate values and the `O1` citation necessarily moves those two paths off them.
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
| `INV-BLOBS` | `codefire` is `f5e0b6db7c15aeadd3025ae4f038e301a7634caf` at mode `100755`, and the upload traversal test is `829836bc0411bf82030286e6f1b0e3d58e574e14`. | `git hash-object` or `git ls-tree` mismatch on either path. |
| `INV-DOC-STAGES` | The four canonical source docs have exactly two stages. Stage one, the pre-citation three-way union reconstructed from `cb13da0`/`b164046`/`0f43415`, is `d8d7aa9881b5f96bcfa13b78168403be9ae62512`, `0a8d2a886c883a051fe8c0e74bf550da3821b977`, `9a3afd7dd932bd3db7f33909e3cda40fb867abfc`, `2b68347eaa357f187f794c2a76992401743929eb`. Stage two, the final blobs committed in `C`, is `5a1755ebd7e0eff8f55f92ae940557badb3ca567`, `72953356fca0982596bbb81a939fd4e2f2584a75`, `9a3afd7dd932bd3db7f33909e3cda40fb867abfc`, `2b68347eaa357f187f794c2a76992401743929eb`, in the path order architecture / trace / playbook / catalog. The first two differ between stages **only** by the required relative `O1` citation; the last two are identical across both stages. | Any stage-one reconstruction that misses its union blob; any final blob in `C` other than the four stage-two values; a stage-one → stage-two delta on paths 7 or 8 containing anything beyond the `O1` citation; or any change at all to paths 9 and 10 after stage one. |
| `INV-DUP` | The initial-Upload patch appears exactly once in `codefire`, and `tests/test_python_http_upload_accounted_traversal.py` has 36 test methods — not 10 and not 46. | A doubled hunk, or a method count other than 36. |
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
| `I3` | `git hash-object codefire` | `f5e0b6db7c15aeadd3025ae4f038e301a7634caf` |
| `I4` | `git ls-tree C -- codefire` | mode `100755` preserved through the three-way merge |
| `I5-UNION` | Reconstruct stage one independently — for each of the four canonical source docs, `git merge-file -p --diff3 <b164046 copy> <cb13da0 copy> <0f43415 copy>` into a scratch file, then `git hash-object` it | exit 0 and zero conflict markers on each, and exactly `d8d7aa9881b5f96bcfa13b78168403be9ae62512` (architecture), `0a8d2a886c883a051fe8c0e74bf550da3821b977` (trace), `9a3afd7dd932bd3db7f33909e3cda40fb867abfc` (playbook), `2b68347eaa357f187f794c2a76992401743929eb` (catalog). These are **intermediate** values and are *not* expected to match the committed files for architecture and trace |
| `I5-FINAL` | `git hash-object` on the four canonical source docs as committed in `C`, plus `diff` of each stage-one scratch file against its committed counterpart | exactly `5a1755ebd7e0eff8f55f92ae940557badb3ca567` (architecture), `72953356fca0982596bbb81a939fd4e2f2584a75` (trace), `9a3afd7dd932bd3db7f33909e3cda40fb867abfc` (playbook), `2b68347eaa357f187f794c2a76992401743929eb` (catalog). The playbook and catalog diffs are **empty**; the architecture and trace diffs contain **only** the required relative `O1` citation and the reflow it forces |
| `I6` | `git hash-object tests/test_python_http_upload_accounted_traversal.py` | `829836bc0411bf82030286e6f1b0e3d58e574e14`, not the 10-test duplicate `141dcbd4ee1f…` |
| `I7` | `git show 0703f71 \| git patch-id --stable` and `git diff 5254e3d cb13da0 \| git patch-id --stable` | both `9889888cedeba7344fb987ff5891cf728d9c7e36` |
| `I8` | `git diff --binary L..C \| sha256sum` | recorded in `W`; the bound report digests `46e7c3e4…` and `756bf200…` reconfirmed |
| `I9` | AST proof on merged `codefire`: units changed by ours vs theirs, overlap, drift, qualified-name collisions, module-level union and evaluation order | 0 overlap, 0 non-identical accepted units, 0 drift, 0 collisions, merged equals the union |
| `I10` | `grep -c '<<<<<<<\|>>>>>>>\|=======' codefire` and the four canonical source docs, at both the stage-one union reconstruction and the stage-two committed files | `0` everywhere, at both stages |
| `I11` | `git diff --check` and `git status --porcelain` at each of `C`, `W` and `R` | clean and empty at all three |

### Focused Python

| ID | Command | Expected |
|---|---|---|
| `V1` | `python3 -m py_compile codefire` and the five test modules | `PASS` |
| `V2` | `python3 -m unittest tests.test_python_http_upload_accounted_traversal -v` | **36/36** |
| `V3` | `python3 -m unittest tests.test_python_http_mr_list_decision_work -v` | **49/49** |
| `V4` | `python3 -m unittest tests.test_python_http_doctor_decision_work tests.test_python_http_branch_decision_work -v` | **32/32** (19 + 13) |
| `V5` | `python3 -m unittest tests.test_python_ancestry_owner tests.test_python_signature_owner tests.test_python_signature_terminal_freeze tests.test_python_sealed_production tests.test_python_external_production -v` | **59/59** adjacent, unchanged by composition |

### Broad Python

| ID | Command | Expected |
|---|---|---|
| `V6` | `python3 -m unittest discover -s tests -p 'test_python_*.py'` | **538/538** in 34 modules |
| `V7` | `python3 -m unittest tests.test_codefire_cli -v` | **228/228** — mandatory composition-only gate; `b164046`'s copy has never run against the merged `codefire`. A `-k upload` subset is a pre-check, never a substitute |
| `V8` | `python3 -m unittest discover -s tests -v` | **766/766** in 35 modules |
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

### Non-vacuity and reversals

Named `X1`-`X9` so the family cannot be confused with the reviewer report commit
`R`. This is the family `806f6c8` published as `R1`-`R9`; none was dropped.

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
4. The `O1` relative citation present in aggregate paths 7 and 8, and the four canonical source docs at their stage-two final blobs: architecture `5a1755ebd7e0eff8f55f92ae940557badb3ca567`, trace `72953356fca0982596bbb81a939fd4e2f2584a75`, playbook `9a3afd7dd932bd3db7f33909e3cda40fb867abfc`, catalog `2b68347eaa357f187f794c2a76992401743929eb`.

### Writer result `W` — required outputs

1. One clean additive commit whose parent is `C`, containing exactly one `ADD` of `docs/development/ai/swarm-results/cfr168-python-http-accepted-input-composition-20260802/result.md`, zero merges, clean tree, and no candidate byte changed.
2. `W` records `C` exactly: SHA, tree, parent `L` and the chain from `L` back to `a972cde`; the 18-path `L..C` name-status; `git diff --binary L..C | sha256sum`; the six product blob ids and `codefire` mode; **both canonical source-doc blob stages in full** — the four stage-one pre-citation union blobs from `I5-UNION` and the four stage-two final candidate blobs from `I5-FINAL`, stated as two distinct sets, with the architecture and trace paths identified as the two that differ and their `O1`-citation-only delta shown, and the playbook and catalog paths identified as identical across both stages; the omitted duplicate patch-id `9889888cedeba7344fb987ff5891cf728d9c7e36`; test counts `538` and `766` plus every focused count; every `T`, `I`, `V`, `D` and `X` result; and every item not run marked explicitly not run.
3. Truthful scoping: `C` is composed-not-accepted; it closes nothing; `B2` remains OPEN pending a fresh exact `ACCEPT` of `C`; `B3` remains OPEN and routed to `C14-LIVE-A2`; MR-write remains blocked; the three handles are registered OPEN and unimplemented; root row 6 unchanged.
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
final blobs in `C` per `I5-FINAL` and confirming the architecture and trace deltas
carry only the required relative `O1` citation while the playbook and catalog are
unchanged across stages; the
duplicate stable patch-id `9889888cedeba7344fb987ff5891cf728d9c7e36`; the binary
patch SHA-256s; the test counts `538` and `766` and every focused count; the
reversals `X1`-`X9`; the inversion census; the AST disjointness; the `B1`/`B2`/`B3`
state as adjudicated here; the single-owner ownership map; and the negative space
— zero product divergence between `16d0129` and `a972cde`, docs-only linear
descent from `a972cde` to `L`, the five canonical-newer management docs
byte-identical, the 18 already-identical docs unchanged, the Cargo/O1 governance
surface untouched, and no agent-state or worktree debris imported. It must confirm
the duplicate is omitted exactly once, must reproduce rather than infer `X2`,
`X3`, `X4` and `V7`, and must render an explicit scope verdict.

Any `ACCEPT` must state that it is scoped to the immutable candidate `C` only and
does not close CFR-168, CFB-136, CFB-138, all of Upload, global `C14`, `B3`,
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
