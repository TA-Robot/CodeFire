# CFR-168 Python HTTP accepted-input composition — writer result

Packet: `docs/development/ai/task-packets/cfr168-python-http-accepted-input-composition-20260802.md`
Typed companion: `docs/development/ai/task-packets/json/cfr168-python-http-accepted-input-composition-20260802.json`
Bound preflight: `docs/development/ai/swarm-results/cfr168-python-composition-exact-preflight-claude-20260802/preflight.md` (unchanged; authoritative for the step-5 union stage only)
Original shared-primitives authority: `docs/development/ai/swarm-results/cfr168-python-http-shared-primitives-preflight-claude-20260731/preflight.md` (unchanged; verdict still `BLOCK`)
Role: composition writer (implementer). Provider/model: Claude Opus 5.

> **Evidence attribution.** Every result below is labelled either
> **[writer-executed]** — run by this lane in this worktree — or
> **[manager-executed]** — produced by a manager lane, whose artifacts this lane
> independently re-hashed and bound but did **not** itself run. Nothing a manager
> lane produced is claimed as this lane's own execution. Anything not executed at
> all is recorded as **not run**, never as `PASS`.

---

## 1. Identities

| Role | Value |
|---|---|
| Immutable product base | `a972cde890e0d621aaced6aca4b45835871da975`, tree `76a7342031f560a116916cfe978979674841c75f` |
| Launch base `L` | `6db7944b26399c3bc43867f12ee9b3f260988621`, tree `69b9f7414ef0104a5782493f8731e1cd3636795f` |
| Candidate `C` | `cf1ac6a7d61d32cb8138c127e649151c5c545196` |
| `C` tree | `531d07fe121a1ac49c5a49462f0ce99f4270c595` |
| `C` parent | `6db7944b26399c3bc43867f12ee9b3f260988621` — exactly `L` |
| `git diff --binary L..C \| sha256sum` | `ca496cbe48714c4f97f35c2ad30d01f9c07e82b670d0db315349a8e86f53c68a` |
| `W` parent | `C` |

### Chain from `L` back to the immutable product base

`a972cde` → `806f6c8` → `a7e283e` → `808280e` → `677f649` → `9737ce3` → `6db7944` (= `L`) → `C`

| Commit | Subject |
|---|---|
| `806f6c8033905cd44e22bf12d42c62dba42c47a8` | freeze the accepted-input Python composition plan and adjudicate B1/B2/B3 |
| `a7e283e5eb5a9f54765c46f10057313a4e858f38` | correct the composition topology, owner map and canonical lifecycle authority |
| `808280e` | separate the composition doc union and O1 citation blob stages |
| `677f649` | freeze the required C18 error-origin composition bridge |
| `9737ce3` | remove stale two-stage composition assertions |
| `6db7944` | correct remaining composition stage authority labels (= `L`) |

`I2` **[writer-executed]**: `git merge-base --is-ancestor a972cde L` succeeds; `git rev-list --merges a972cde..L | wc -l` = **0**; `git diff --name-only a972cde..L` is **docs-only** (4 paths: the bound preflight, the packet, the typed companion, the burn-down) with **0** product/runtime/test/packaging paths.

---

## 2. Accepted inputs and the omitted duplicate

| Input | Candidate | Tree | Report SHA-256 |
|---|---|---|---|
| Doctor + Branch | `701469f91dd0c4c52c5628fb3e9d9e9d0ab0c714` | `dd63dd560514aef08925cc8b1dbe815bafcceef4` | `c7a1665fdf4a6e1104bbab18ec8051c048ace1d25e865952bbbcd35a06eb19c0` |
| MR-list | `b16404676639336f705feffd9af4857d4a1fcc69` | `36185e320338b67e1610d82fcc81cba310c36026` | `5c952a9a5c162bfbb93522bb644fcba44ce32ddf3628501e6f94308899fe760e` |
| Upload live-route cleanup | `0f4341561ce95935600bf1398b8a5fbdb206daba` | `da356c2a041e57eea9192ab917f3a293239c60e7` | `46e7c3e41ac1b132766657e445e017b392ce6a0b72eb46f909b13755fc8a9b3c` |

`merge-base(701469f, b164046)` = `701469f` **[writer-executed]** — Doctor/Branch is a strict ancestor of MR-list and was not replayed separately.

**`I7` omitted-duplicate patch-id [writer-executed].** `git show 0703f71 | git patch-id --stable` and
`git diff 5254e3d cb13da0 | git patch-id --stable` both yield
**`9889888cedeba7344fb987ff5891cf728d9c7e36`**. The `b164046` copy is kept; the
Upload lineage's own `5254e3d..cb13da0` limb is omitted. On the one path both
copies touch, the blob is `141dcbd4ee1f8727fbe146faef481cf746c8c73f` at
**both** `cb13da0` and `b164046`, so the omission is a no-op there and the
newest successor blob was taken whole.

Three-way merge inputs for `codefire`: ours `b164046` `f30575a697c1b525653c13738987ccdb5c0151ea`, base `cb13da0` `03966af23dd860c5235832320a8f1977ecd6495a`, theirs `0f43415` `b984c1e7e393af658a618c96270d33d39bf8eeaa`. `git merge-file -p --diff3` exit **0**, zero conflict markers, line arithmetic exactly additive: base 33,633 + ours 35,791 + theirs 35,129 → merged **37,287** = 35,791 + (35,129 − 33,633).

---

## 3. This is not a byte-only union — the `C18` bridge

The error-origin taxonomy exists **only** in `b164046`; the synchronous
commit-source cleanup drain exists **only** in `0f43415`; base `cb13da0` has
neither. Their union is the first tree where both exist, and it is **not closed**
under either input's accepted behaviour.

**Measured pre-bridge defect [writer-executed].** On the pre-bridge composed
tree, `python3 -m unittest ...test_request_loop_accounts_a_drained_cleanup_fault`
failed both subtests — `route='ancestry'` and `route='signature-history'` — with
`AssertionError: 0 != 1` on `len(errors)`. The standalone cleanup `OSError`
returned by `_drained_commit_source_cleanup_error` reached `do_POST`'s
`except BaseException` arm unmarked, where `classify_route_os_error` converted it
into a typed filesystem `400`, consuming the fault instead of letting it escape.

**Authority.** Frozen in the shared-primitives preflight, quoted not
reinterpreted, and that report is **not** edited: the `C18` per-clause register
row ("one typed, exhaustive error-origin taxonomy shared by GET and POST:
filesystem, transport, deadline, budget, integrity, **post-commit cleanup** —
classified at the boundary that raised it, never by message text"); §5.7
`ErrorOrigin` **adapt** ("add the post-commit cleanup origin"); and the §9.2
conflict-map error-origin row ("reuse the accepted taxonomy; add only the
post-commit cleanup origin").

**The resolution, as landed.** One shared taxonomy extended by exactly one origin:

1. `COMMIT_CLEANUP_ORIGIN_ATTRIBUTE = "codefire_commit_cleanup_origin"` — one new module constant.
2. `mark_commit_cleanup_origin` / `is_commit_cleanup_os_error` — the new marker and predicate.
3. `os_error_origin` is exhaustive over **transport**, **commit-cleanup**, **filesystem** (falling through to `unclassified` for an unmarked error, exactly as before).
4. The mark is applied **at the cleanup owner**, inside `_drained_commit_source_cleanup_error`, at the point the drain produced the error — never by route, dispatcher, errno, message or exception subclass.
5. `classify_route_os_error` **refuses** the cleanup origin exactly as it refuses transport, so the error reaches `if authority is None: raise`, escapes `do_POST`, and escapes `handle_one_request` after the request loop has already accounted the drained ledger.

**Preserved unchanged, and proved by reversal:** a generic **unmarked** route
`OSError` — including one raised by a patched `handle_http_upload` — still
becomes the typed filesystem `400` (`X12`); transport-origin still abandons the
connection; `MemoryError` stays outside the `OSError` arm; and `C14` precedence
over a cleanup **cause** is unchanged (`X4`, `r5a`/`r5b`/`r6a`/`r6b`).

`mark_filesystem_origin` and `is_filesystem_os_error` are **byte-identical to the
pre-bridge checkpoint** and are deliberately outside the bridge's changed set.
Cleanup precedence is achieved by ordering inside `os_error_origin` and by the
refusal inside `classify_route_os_error`, not by weakening filesystem
classification.

**Scope of what the bridge closes.** On a fresh exact `ACCEPT` of `C` in `R`, and
not before, this bridge closes **only** the `C18` composition incompatibility
between the two accepted inputs. It adds the post-commit cleanup **origin**; it
does **not** implement the §5.7 post-effect publication clause, which remains
`B3` / `C14-LIVE-A2` work.

---

## 4. Candidate write set — exactly 18 paths (`T2` [writer-executed])

`git diff --name-status L..C`, count **18**:

| # | Op | Path | Blob in `C` | Mode |
|---:|---|---|---|---|
| 1 | `M` | `codefire` | `73b1f368f9c1591be6fd9219b839351878b82b9a` | `100755` |
| 2 | `M` | `tests/test_codefire_cli.py` | `b23c43ced6ccb69c0bb553f9856c1a7b2145e770` | `100644` |
| 3 | `A` | `tests/test_python_http_doctor_decision_work.py` | `0f993edd6c6a57ee5ca0d15abc04e2260c3d48cd` | `100644` |
| 4 | `A` | `tests/test_python_http_branch_decision_work.py` | `c0269ee80e00265d0144520ab975ce7e4bc9b7e3` | `100644` |
| 5 | `A` | `tests/test_python_http_mr_list_decision_work.py` | `0d49841a2e682e661d169274e571e198f3408aa7` | `100644` |
| 6 | `A` | `tests/test_python_http_upload_accounted_traversal.py` | `aba7ac9fb4f73bfc1257da00d2a4ddbcbf6f55a4` | `100644` |
| 7 | `M` | `docs/development/source-architecture.md` | `fe2b3b74f2beaf30218333f14e05b6e4e6d4e699` | `100644` |
| 8 | `M` | `docs/development/command-source-trace.md` | `361a5de8f9f881fbedf35ed5a833baade9529e38` | `100644` |
| 9 | `M` | `docs/development/change-playbook.md` | `18d882813e02ed8cd795aa1e46d8cac6a09be4ef` | `100644` |
| 10 | `M` | `docs/development/source-data-model-catalog.md` | `2b68347eaa357f187f794c2a76992401743929eb` | `100644` |
| 11 | `A` | `…/cfr168-python-http-upload-live-route-cleanup-rereview-claude-20260802/review.md` | `216ee2680872c43dcd7b657e3a8d4312dbd9b1e9` | `100644` |
| 12 | `A` | `…/swarm-batches/cfr168-python-http-upload-live-route-cleanup-repair-20260802.md` | `250703ae50afcf842359abebc813cea717679c6a` | `100644` |
| 13 | `A` | `…/swarm-batches/cfr168-python-http-upload-zero-residual-repair-20260730.md` | `46254b17badce37fdbec130967a0329bbb9afac7` | `100644` |
| 14 | `A` | `…/swarm-batches/cfr168-python-http-upload-zero-residual-repair-20260730.json` | `4b4c6bfffcd2ab77f1e27d1345d543f58ddea143` | `100644` |
| 15 | `A` | `…/cfr168-python-http-upload-high2-third-repair-20260731/result.md` | `7cfa8bbe441b243e28fb8340fa249a9ed53a5005` | `100644` |
| 16 | `A` | `…/cfr168-python-http-upload-zero-residual-review-closure-repair-20260731/result.md` | `7100966bf76b1ada44d797e8787eb8377a18f6fd` | `100644` |
| 17 | `A` | `…/cfr168-python-http-upload-zero-residual-review-closure-repair-20260731/result-2.md` | `7169efbf1a0eb578aa92a71797f54abe9eb0299d` | `100644` |
| 18 | `M` | `docs/development/code-review-issues/cfr-168.md` | `b3a543979d04ecca3c12014a5d144a08dc72c784` | `100644` |

Evidence-path SHA-256 reconfirmed **[writer-executed]**: path 11 `46e7c3e41ac1b132…` (the Upload terminal `ACCEPT`), path 12 `756bf200138f6066…` (the repair packet the `ACCEPT` binds to).

Aggregate path 19 — this file — is **not** in `C` and does not appear in
`git diff --name-status L..C`.

---

## 5. The three canonical source-doc blob stages, in full

| Path | Stage 1 — pre-citation union | Stage 2 — after `O1` citation | Stage 3 — committed in `C` |
|---|---|---|---|
| `docs/development/source-architecture.md` | `d8d7aa9881b5f96bcfa13b78168403be9ae62512` | `5a1755ebd7e0eff8f55f92ae940557badb3ca567` | `fe2b3b74f2beaf30218333f14e05b6e4e6d4e699` |
| `docs/development/command-source-trace.md` | `0a8d2a886c883a051fe8c0e74bf550da3821b977` | `72953356fca0982596bbb81a939fd4e2f2584a75` | `361a5de8f9f881fbedf35ed5a833baade9529e38` |
| `docs/development/change-playbook.md` | `9a3afd7dd932bd3db7f33909e3cda40fb867abfc` | `9a3afd7dd932bd3db7f33909e3cda40fb867abfc` | `18d882813e02ed8cd795aa1e46d8cac6a09be4ef` |
| `docs/development/source-data-model-catalog.md` | `2b68347eaa357f187f794c2a76992401743929eb` | `2b68347eaa357f187f794c2a76992401743929eb` | `2b68347eaa357f187f794c2a76992401743929eb` |

**Delta between each pair, identified [writer-executed]:**

- **Stage 1 → stage 2** changes **architecture and trace only**, and only by the required relative `O1` citation to `ai/swarm-results/cfr168-python-http-shared-primitives-preflight-claude-20260731/preflight.md` plus the reflow it forces. Measured: architecture 12 changed lines, trace 7 changed lines. Playbook and catalog diffs are **empty**. No other sentence, claim, heading or ordering changed; the `C14`/`B2` statements keep their existing meaning — the citation names where the authority lives, it does not restate, narrow or resolve it.
- **Stage 2 → stage 3** changes **architecture, trace and playbook only**, and only by the frozen `C18` bridge truth: each of the three now names all three origins, none still asserts that only client transport failures abandon the connection, and none asserts origin is decided by exception class. The catalog diff is **empty**.
- **`docs/development/source-data-model-catalog.md` is unchanged across all three stages** and is still `2b68347eaa357f187f794c2a76992401743929eb` in `C`.

`I5-UNION` was reconstructed **independently** from `cb13da0` / `b164046` /
`0f43415` with `git merge-file -p --diff3` into scratch files, exit **0** and
**zero** conflict markers on each, and every hash matched the frozen value.

---

## 6. Pre-bridge and final identities of `codefire` and aggregate path 6

| Artifact | Pre-bridge checkpoint | Committed in `C` |
|---|---|---|
| `codefire` | `f5e0b6db7c15aeadd3025ae4f038e301a7634caf` (mode `100755`) | `73b1f368f9c1591be6fd9219b839351878b82b9a` (mode `100755`) |
| `tests/test_python_http_upload_accounted_traversal.py` | `829836bc0411bf82030286e6f1b0e3d58e574e14` (36 methods) | `aba7ac9fb4f73bfc1257da00d2a4ddbcbf6f55a4` (37 methods) |

Both pre-bridge checkpoints were reached and recorded before the bridge, exactly
as the packet freezes them. Neither is the blob committed in `C`; the delta from
each is exactly the frozen bridge.

`tests/test_python_http_mr_list_decision_work.py` is `0d49841a2e682e661d169274e571e198f3408aa7`
in `C` — **byte-identical to its accepted source** at `b164046`, preserved as the
unmodified `C18` control oracle.

### `I9-BRIDGE` changed-unit set — `C`'s `codefire` vs the pre-bridge checkpoint

Exactly five function units, plus exactly one module-level constant, and nothing else:

| Unit | Change |
|---|---|
| `mark_commit_cleanup_origin` | **ADDED** |
| `is_commit_cleanup_os_error` | **ADDED** |
| `os_error_origin` | modified — third origin, checked before filesystem |
| `classify_route_os_error` | modified — refuses the cleanup origin as it refuses transport |
| `_drained_commit_source_cleanup_error` | modified — marks the standalone `OSError` at the owner |
| `COMMIT_CLEANUP_ORIGIN_ATTRIBUTE = "codefire_commit_cleanup_origin"` | the sole added module-level statement |

Zero units deleted, zero renames, zero duplicate qualified definitions, zero
qualified-name collisions, no module-level statement removed, and the complete
pre-bridge top-level order is a subsequence of the final order. `mark_filesystem_origin`
(unit SHA-256 `cc2bd19ffa2abb793725e1a796aa836dd1671748f86d72fbb81bd3d040442009`) and
`is_filesystem_os_error` (`3b169be24b40aa21d567714a816ab5d47e240d222ea408307e6fbac8910b97b1`)
are byte-identical pre-bridge and final.

The bridge changed **no** unit outside the named taxonomy symbols and their two
live call sites, and introduced **no** product change beyond it.

---

## 7. Gate results

### Topology and write-set exactness

| ID | Result | Evidence |
|---|---|---|
| `T1` | **PASS** [writer-executed] | `git rev-list --count L..C` = `1`; `git rev-list --merges L..C \| wc -l` = `0` |
| `T2` | **PASS** [writer-executed] | `git diff --name-status L..C` = exactly the 18 candidate paths in §4 |
| `T3` | **PASS** [writer-executed] | `git rev-list --count C..W` = `1`; merges = `0`; `git diff --name-status C..W` = exactly one `ADD` of aggregate path 19 |
| `T4` | **PASS** [writer-executed] | `git diff C..W` over the 18 candidate paths is **empty** |
| `T5` | **not run** — reviewer report `R` does not exist and this lane must not create it |
| `T6` | **partial, honestly scoped** — `L..W` is `2` commits with `0` merges [writer-executed]; the `3` / `0` form requires `R` and is **not run** |
| `T7` | **PASS** [writer-executed] | `git grep -F` over the 18 candidate paths for `C`'s SHA `cf1ac6a7…`, tree `531d07fe…` and patch digest `ca496cbe…` → **0 matches each** |

`T3`/`T4` were measured after committing `W`; the values are restated in §10.

### Identity and exactness

| ID | Result | Evidence |
|---|---|---|
| `I1` | **PASS** [writer-executed] | `C` `cf1ac6a7d61d32cb8138c127e649151c5c545196`, tree `531d07fe121a1ac49c5a49462f0ce99f4270c595`, `C^` = `6db7944b…` = `L` |
| `I2` | **PASS** [writer-executed] | linear descent, 0 merges, docs-only, 0 product/test/packaging paths |
| `I3` | **PASS** [writer-executed] | pre-bridge `f5e0b6db7c15aeadd3025ae4f038e301a7634caf`; in `C` `73b1f368f9c1591be6fd9219b839351878b82b9a`, delta exactly the frozen bridge |
| `I4` | **PASS** [writer-executed] | `git ls-tree C -- codefire` → mode `100755` |
| `I5-UNION` | **PASS** [writer-executed] | all four stage-one blobs matched exactly; exit 0, 0 markers each |
| `I5-CITED` | **PASS** [writer-executed] | all four stage-two blobs matched exactly; playbook/catalog diffs empty; architecture/trace diffs contain only the `O1` citation |
| `I5-FINAL` | **PASS** [writer-executed] | catalog unchanged with an empty diff; architecture/trace/playbook diffs contain only the `C18` bridge truth |
| `I6` | **PASS** [writer-executed] | pre-bridge `829836bc…` (not the 10-test duplicate `141dcbd4ee1f…`); in `C` `aba7ac9f…`, differing only by the one new oracle; MR-list module byte-identical to its accepted source |
| `I7` | **PASS** [writer-executed] | both patch-ids `9889888cedeba7344fb987ff5891cf728d9c7e36` |
| `I8` | **PASS** [writer-executed] | `ca496cbe48714c4f97f35c2ad30d01f9c07e82b670d0db315349a8e86f53c68a`; bound report digests `46e7c3e4…` and `756bf200…` reconfirmed |
| `I9` | **PASS** [writer-executed] | see below |
| `I9-BRIDGE` | **PASS** [writer-executed] | §6 |
| `I10` | **PASS** [writer-executed] | `0` line-anchored conflict markers in `codefire` and all four docs at **all three** stages |
| `I11` | **PASS** [writer-executed] | `git diff --check` clean and `git status --porcelain` empty at `C` and at `W` |

**`I9` AST disjointness on the pre-bridge merged `codefire` [writer-executed].**
Units: base 1088, ours 1163, theirs 1136, merged 1211. Changed by OURS **101**
(computed symmetrically over `union(base, side)`, so the accepted refactor's
deletions of `AccountedHTTPBundle.close` and `AccountedHTTPBundle.__del__` count
as OURS changes rather than false untouched drift); changed by THEIRS **78**;
**overlap 0**; every OURS-changed and THEIRS-changed unit byte-identical in
merged; **0 drift across 1034 units untouched by both**; **0** qualified-name
collisions; **0** duplicate qualified definitions. Module-level statements were
compared as **ordered sequences with multiplicity**, never as sets: base 84,
ours 94, theirs 92, merged 102; merged additions are exactly OURS+THEIRS as a
multiset; nothing removed; base, ours and theirs orders are each preserved as a
subsequence of merged; and every module-bound name used at import time is
defined before its use.

A manager-supplied independent proof `/tmp/cfr168ast/astproof3.py`, SHA-256
`f617c3c083394d0da5cba3630985f775c69e8effda748ee255639e6c79628fb6`, was
re-hashed and executed by this lane on the committed bytes and returned
`RESULT=ALL_PROOFS_PASS`, exit `0` **[writer-executed run of a manager-authored
script]**.

**`INV-B3` surface, byte-identity proved [writer-executed]:**

| Symbol | Identical to pre-bridge | Accepted source |
|---|---|---|
| `_PreparedHTTPMutationResponses` (sha256 `16a4501d35455d48…`) | yes | matches `0f43415` |
| `make_http_handler.CodeFireHTTPHandler.do_POST` (`ff31cb1ffc988883…`) | yes | matches `b164046` |
| `make_http_handler.CodeFireHTTPHandler.handle_one_request` (`af8a6a832d3845a2…`) | yes | matches `b164046` |

The bridge changes `classify_route_os_error`'s verdict for one origin, not the
arm that consults it. No phase index is claimed and no post-effect clause is
implemented.

### Focused and broad Python

| ID | Expected | Result | Attribution |
|---|---|---|---|
| `V1` | `PASS` | **PASS**, exit `0` — `py_compile` of `codefire` + the five bound test modules | [writer-executed] |
| `V2` | 37/37 | **PASS**, `Ran 37 tests`, `OK`, exit `0` — both subtests of `test_request_loop_accounts_a_drained_cleanup_fault` pass | [writer-executed] |
| `V3` | 49/49 | **PASS**, `Ran 49 tests`, `OK`, exit `0`, including `test_dispatcher_publishes_every_route_filesystem_failure` | [writer-executed] |
| `V4` | 32/32 | **PASS**, `Ran 32 tests` (19 + 13), `OK`, exit `0` | [writer-executed] |
| `V5` | 59/59 | **PASS**, `Ran 59 tests`, `OK`, exit `0` | [writer-executed] |
| `V6` | 539/539 in 34 modules | **PASS**, `Ran 539 tests`, `OK`, exit `0`; 34 `test_python_*` modules counted | [writer-executed] |
| `V7` | 228/228 | **PASS**, `Ran 228 tests in 1343.779s`, `OK`, `TEST_EXIT=0` | **[manager-executed]** |
| `V8` | 767/767 in 35 modules | **PASS**, `Ran 767 tests in 1804.878s`, `OK`, `TEST_EXIT=0`; 35 test modules counted | **[manager-executed]** |
| `V9` | `./demo.sh` | **PASS**, exit `0`, CodeFire demo completed | **[manager-executed]** |

Composed test-count arithmetic: base 421 `test_python_*` + 19 Doctor + 13 Branch
+ 49 MR-list + 36 Upload = 538 pre-bridge, **+1** new `C18` oracle = **539** in 34
modules; **539 + 228 = 767** in 35 modules.

**`V7`/`V8` binding [manager-executed, independently re-hashed by this lane].**
Both were run by a manager lane against a genuine source-bound Rust binary after
this lane froze `C`. This lane re-computed every digest below and confirms each
matches:

| Artifact | SHA-256 | Verified |
|---|---|---|
| final binary `…/cfr168_aia046_artifact_export_hardlink_repair_20260802/target/release/codefire-rs` | `09d9fa2561de4b687200baa4b9fe54a4159e43def1a7a3cd28dc05c2741741e4` | re-hashed: match; `bytes=14457880 mode=700 nlink=1` re-`stat`ed: match |
| provenance `target/codefire-ai/rust-build-provenance.json` | `d4f34a6bb81fdd7ead8d0fb76751d3335f393e2167c8c3a9d071da0dd29123d0` | re-hashed: match |
| gate result `…/rust-release-build-ddcfad8a1d7b4b94fa4daf94-0.json` | `19a79f1be1b2938d9b82b3fd5385aa1a1a320246dc5b6267816ff79e813f573f` | re-hashed: match |
| gate log `…-0.log` | `cf322384704f81e003aa33c1978b72210a1a2cab1859d298b888561b7722bb29` | re-hashed: match |
| `V7` log `/tmp/cfr168-python-composition-v7-20260802.log` | `976b7a50a03fbf4140f902be3b5b26a1d1855fe8d45fbfc0ee163845d69cbaa3` | re-hashed: match |
| `V8` log `/tmp/cfr168-python-composition-v8-20260802.log` | `6ce6ad311b61839089de022d244ccda84e2f47c886a2ae79d037169a1367c662` | re-hashed: match |

Inspected inside those artifacts by this lane: provenance `ok` true, base
`54028a136943a9d5f9608e19d3855c0edae46adf` tree
`e4057e1b6f16b12fed45ad0b50bc4c6014ca7ebd`, source key
`cd0c9abd072a843c6289ed83c5b604adf456f6312a307affbe797c6fb8bf6b25`, artifact key
`9c92559f4f4d51f270d27bcc5fff888936bda373dcb9150435607e6d33c861c6`, build
command `cargo build --release --locked -p codefire-cli --bin codefire-rs`. Gate
`decision` = **`PASS`**, `blockers` = **`[]`**, `candidate_unchanged` = **true**,
candidate and `candidate_after` both `dirty: false`. Artifact export `decision` =
**`EXPORTED`**, `source_nlink` 2, destination `export_nlink` 1 `export_mode` 448
(`0700`), and `bytes` / `sha256` / `source_reread_bytes` / `source_reread_sha256`
all equal the final binary. Both test logs record `HEAD=cf1ac6a7d61d32cb8138c127e649151c5c545196`
— exactly `C` — the binary digest **identical before and after** each run, and an
empty `WORKTREE_STATUS_BEGIN`/`END` block.

**`54028a1` scope, independently verified by this lane [writer-executed]:**
`merge-base(L, 54028a1)` = `L`, so it descends linearly from `L`;
`git diff --name-only L..54028a1` is exactly four AI-gate docs/tools paths
(`docs/development/ai/source-keyed-cargo-gates.md`,
`docs/development/tools/build_source_bound_rust.py`,
`docs/development/tools/run_source_keyed_cargo_gate.py`,
`docs/development/tools/test_ai_tools.py`); and
`git diff --name-only L..54028a1 -- codefire tests/ crates/ Cargo.toml Cargo.lock install.sh pyproject.toml setup.py`
is **empty**, so the product and Rust source are the same canonical base this
Python-only candidate composes against. None of those four paths is in `C`.

Final binary smoke, **[manager-executed]**: `codefire foundation` → 1; `--help`
exit 0; completion `bash` and `zsh` exit 0.

### Checkers

| ID | Result | Attribution |
|---|---|---|
| `D1` | **PASS**, exit `0` — `44 modules, 34 commands, 12 object kinds` | [writer-executed] |
| `D2` | **PASS**, exit `0` — `694 Markdown files inventoried`; `L` has 688 tracked `.md` and `C` adds exactly the 6 Markdown paths in its write set (688 + 6 = 694) | [writer-executed] |
| `D3` | **PASS**, exit `0` — AI contract check passed | [writer-executed] |
| `D4` | **PASS**, exit `0` — AI acceleration docs check passed | [writer-executed] |
| `D5` | **PASS**, exit `0` — `--timeout 180`, all **7/7** including `tool_tests` | [writer-executed] |
| `D6` | **PASS** — `git diff L..C` over `source-keyed-cargo-gates.md` and `run_source_keyed_cargo_gate.py` is **empty** | [writer-executed] |
| `D7` | **PASS** — every `shared-primitives` / `C14` / `B2` mention in paths 7 and 8 resolves to the in-tree authority path | [writer-executed] |
| `D8` | **PASS** — paths 7, 8 and 9 each name all three origins; none still asserts that only client transport failures abandon the connection, or that origin is decided by exception class | [writer-executed] |

### Non-vacuity and reversals — `X1`-`X13`

Every reversal below was **applied to a pristine snapshot, run, then reverted,
with byte-identity re-proved by SHA-256 after each one** [writer-executed].
Final `codefire` and `tests/test_codefire_cli.py` digests equal their pristine
values; no reversal left any residue in `C`.

| ID | Result | Observed |
|---|---|---|
| `X1` `r1` | **FAILED as required** | ancestry retry removed → 3 failures: cleanup-survives, request-loop-accounts, never-displaces, all `route='ancestry'` |
| `X1` `r2` | **FAILED as required** | signature retry removed → the same 3, all `route='signature-history'` |
| `X1` `r3` | **FAILED as required** | `__del__` renamed away → 1 failure, `test_commit_source_finalizer_settles_an_abandoned_retry` |
| `X1` `r4` | **FAILED as required** | helper retries twice → 6 failures across both routes |
| `X1` `r5a` | **FAILED as required** | ancestry inversion restored → `errors=1` on the C14 test, `route='ancestry'` |
| `X1` `r5b` | **FAILED as required** | signature inversion → `errors=1`, `route='signature-history'` |
| `X1` `r6a` | **FAILED as required** | ancestry `from cleanup_error` → `from None` → 1 failure on the C14 test |
| `X1` `r6b` | **FAILED as required** | signature same → 1 failure |
| `X1` `m2` | **FAILED as required** | erase-before-release restored → 2 failures (cleanup-survives, finalizer) |
| `X2` | **PASS** | `test_request_loop_accounts_a_drained_cleanup_fault` `Ran 1 test`, `OK`, exit `0` — real socket `POST`, one real `close(2)` fault, accounted through `make_http_handler`'s `_decision_work_observer` in `handle_one_request`'s `finally` |
| `X3` | **PASS** — reproduced independently | an independent probe drove the production drain with **8 armed `close(2)` faults**: the faulting source was closed **exactly 2** times, the **first** armed error was returned by object identity (not the retry's), it carried the cleanup origin, and three non-faulting sources were each closed **exactly 1** time returning `None` |
| `X4` | **PASS** — reproduced independently | `test_live_cleanup_fault_never_displaces_the_operation_decision` `Ran 1 test`, `OK`, exit `0`; plus an independent identity assertion: the outer decision is the **same object** and the first cleanup error is its `__cause__` by identity, and a non-`OSError` outer decision is still refused by the shared classifier |
| `X5` | **FAILED as required** ×3 | `x5a` silence the publication-state proof → `test_unpublishable_prepared_response_never_reaches_the_wire`; `x5b` restore a silent `_release_exact` → `test_owner_over_release_fails_closed`; `x5c` classify by exception class instead of boundary → `test_error_origin_is_classified_at_the_boundary_that_raised_it`, 6 failures |
| `X6` | **FAILED as required** | reverting the corrected branch-list deadline expectation to the pre-fix `200 OK` / zero-poll form fails `test_python_http_bundle_deadline_boundaries_and_poll_phases`, proving the corrected expectation is load-bearing |
| `X7` | **PASS** | `grep -n 'raise cleanup_error from run_error' codefire` → **exactly 1** occurrence, inside `run_server_verification`; **0** inside `enforce_remote_commit_signature_policy_accounted` |
| `X8` | **PASS** | `grep -c '_drained_commit_source_cleanup_error' codefire` → **5**; both live call sites carry the drain helper intact |
| `X9` | **PASS** | three consecutive runs of the three new Upload live-route tests → `Ran 3 tests` `OK` exit `0` ×3, no port, PID or timing flake |
| `X10` | **FAILED as required** | removing the cleanup-origin mark at the drain owner → `test_request_loop_accounts_a_drained_cleanup_fault` fails on **both** routes with `0 != 1`, the exact measured pre-bridge defect |
| `X11` | **FAILED as required** | restoring `classify_route_os_error` to its accepted form with the mark left in place → the same oracle fails on both routes; the **refusal**, not merely the mark, is load-bearing |
| `X12` | **FAILED as required** | making the classifier treat a generic **unmarked** `OSError` as cleanup origin → `test_dispatcher_publishes_every_route_filesystem_failure` fails on 3 routes **including its patched `handle_http_upload` subtest**; the bridge did not weaken filesystem classification |
| `X13` | **FAILED as required** | replacing the marker check with an errno test → the new `C18` cross-input oracle fails, because its two `OSError`s share errno, message and filename and differ only by the production mark |

### Explicitly not run

- **Cargo/Rust by this lane** — `cargo fmt`, `cargo clippy`, `cargo test`, and any `cargo build`: **not run** by the writer. The release build recorded in §7 is **[manager-executed]** and is bound here by digest only.
- `install.sh`, packaging, dogfood, Docker, network: **not run**.
- `check_swarm_admission.py`, `check_review_terminalization.py`, `check_review_recurrence.py`: **not run** — each requires mandatory arguments and gates lane admission rather than this composition's content.
- `T5`: **not run** — requires reviewer report `R`, which this lane must not create.
- `T6`: only the `L..W` half was measured (`2` commits, `0` merges); the full `L..R` = `3` form is **not run**.
- `V7`, `V8`, `V9`: **not executed by this lane**; recorded above as **[manager-executed]** with independently re-hashed evidence.
- The MR-list `ACCEPT`'s fault-injection probe rows (its reviewer `P2`/`P3`/`P6`-`P13` style injections) were **not** re-run as such; `X5` reproduces that report's reversals as three single source edits and nothing broader is claimed for them.
- An earlier interrupted battery in this session lacked `pipefail`; **none of its output is relied on**. Every `V`/`D` result above was re-run afterwards with the exit code captured directly from the Python process.

---

## 8. Routed residual ownership — registered OPEN, implemented nowhere

Candidate `C` registers three handles as **OPEN** in
`docs/development/code-review-issues/cfr-168.md`. **Registration is bookkeeping
only.** No handle is implemented, narrowed, repaired or closed. Every routed
locus has exactly one owner and appears in no second row.

| Handle | Scope rule | State |
|---|---|---|
| `C14-LIVE-A1` | reachable **pre-effect** loci: the inherited `RetryableCleanup` constructor/traceback debt in `_open` and `validate_sealed_commit`; `F6` persistent-teardown non-recovery at 11,456 retained bytes; `F7` `_open` handle orphaning; every reachable pre-effect cleanup owner on the two commit-source wrappers; and `AccountedServerVerification` / `run_server_verification`, including the one surviving inversion there | **OPEN** |
| `C14-LIVE-A2` | `B3` **post-effect / response / wire** loci: `_PreparedHTTPMutationResponses`' post-effect contract; the handler `POST` arm; the request-loop `finally` and the wire publication window; promoting `C14`/`B2` to a durable in-tree contract; and `RetryableCleanup`'s `shape-pending` state | **OPEN** |
| `C14-SUCCESSOR-OWNER` | **only** production-unwired or deferred global loci owned by neither `A1` nor `A2`: `AccountedHTTPUploadAncestry`, `AccountedHTTPUploadSealedAncestry`; planner/stage/sealed factories; signature and ancestry owners outside the two commit-source wrappers, **excluding** `AccountedServerVerification` and `run_server_verification`; `F9` `_control_charge_witness`; `O2`; `O3` | **OPEN** |

The surviving `raise cleanup_error from run_error` in `run_server_verification`
is **pre-existing on both lineages**, is **not** composition-introduced, and
routes to `C14-LIVE-A1`. It was not repaired.

The `O1` observation is discharged by **citation only** — a relative link to the
existing in-tree shared-primitives preflight at the `C14`/`B2` mentions in paths
7 and 8. No new authority document was authored. The `C14-LIVE-A2` contract half
stays OPEN.

---

## 9. Truthful scope

- **`C` is composed, not accepted.** It is a candidate. It has **not** been reviewed.
- **`C` is accepted-input composition plus exactly one preflight-prescribed `C18` bridge — not a byte-only union.** The bridge is authorised by the shared-primitives preflight's `C18` per-clause register row, its §5.7 `ErrorOrigin` **adapt** clause, and its §9.2 conflict-map error-origin row. Every byte outside that frozen bridge comes from an accepted source blob.
- **`C` closes nothing.** It does not close CFR-168, CFB-136, CFB-138, global `C14`, `C18` repository-wide, `B3`, MR-write, integration, Install or Dogfood.
- **`B2` remains OPEN**, pending a fresh exact `ACCEPT` of `C` in `R`. The composition is its assigned resolution *action*, not a repository-level resolution.
- **`B3` remains OPEN** and routed to `C14-LIVE-A2`; it continues to block MR-write and global closure. No phase index is claimed for the post-first-effect `PreparedResponse` window.
- **`B1` remains blocked for MR-write's own base** `fcb2324145962e79d7c6ee173b9cf545733b957d`. No MR-write base was rebound.
- **MR-write remains blocked** until both a fresh exact `ACCEPT` of `C` and the later required `C14` resolution land.
- **The three `C14` handles are registered OPEN and unimplemented.**
- **Root row 6** (`CFR-168 + CFB-136 + CFB-138`) stays `Implement in progress`. `docs/development/current-cycle-burn-down-plan.md` is in no writer write set and was not touched. No `Close` was performed on any root, issue or cycle.
- **The original shared-primitives preflight's verdict is still `BLOCK`** with `B1`, `B2`, `B3`. It is byte-identical at `L` and in `C` and was never edited, and it is not restated as `READY`.
- **Licensed narrow claims, and only these three:** (a) `HIGH-2` is closed on all three limbs **for candidate `0f43415`**, per report SHA-256 `46e7c3e41ac1b132766657e445e017b392ce6a0b72eb46f909b13755fc8a9b3c`; (b) the `B2` product contradiction between `b164046` and the shared-primitives preflight is **measured as eliminated on this composed tree** — a candidate-scoped measurement, not a repository-level resolution; (c) the `C18` **composition incompatibility** between `b164046`'s taxonomy and `0f43415`'s cleanup drain is resolved in candidate `C` — candidate-scoped, durable only on a fresh exact `ACCEPT` in `R`.

### Negative space verified [writer-executed]

- The five canonical-newer management docs — `README.md`, `current-cycle-burn-down-plan.md`, `history.md`, `codefire-improvement-cycle.md`, `ai/swarm-batches/cfr168-recovery-execution-ledger-20260730.md` — are **byte-identical to `L`** across `L..C`.
- The AIA-046 Cargo/O1 governance surface is byte-identical to `L` across `L..C`.
- Both immutable preflights and this packet with its typed companion are byte-identical to `L` across `L..C`.
- No `.claude-second-agent`, `.claude-worktrees`, `.codex-second-agent`, `.fugu-*` or `.dev-swarm` path, and no agent state, log or worktree debris, was imported. No untracked file exists in the worktree.
- The superseded `…upload-third-repair-rereview-claude-20260731/review.md` `REQUEST_CHANGES` report and the three stale open-audit artifacts were left untouched and none was deleted.
- No `git cherry-pick`, no `git merge`, no merge commit, no amend/rebase/reset, no squash, no `git reset --hard`, no `git checkout --` over user changes.

---

## 10. Topology of this result commit `W`

| Property | Value |
|---|---|
| Parent | `C` = `cf1ac6a7d61d32cb8138c127e649151c5c545196` — direct, no merge |
| Write set | exactly one `ADD`: `docs/development/ai/swarm-results/cfr168-python-http-accepted-input-composition-20260802/result.md` |
| Candidate bytes changed | **none** — `git diff C..W` over the 18 candidate paths is empty |

`W` records `C` exactly and is writer evidence only. It is **not** part of the
review subject. The review subject is `C` alone.

Reviewer report `R` was **not** created by this lane, `C..W` was **not**
integrated, and canonical was **not** fast-forwarded. Next step is the fresh
exact report-only review producing `R` on top of `W`, by a fresh agent, session
and worktree.
