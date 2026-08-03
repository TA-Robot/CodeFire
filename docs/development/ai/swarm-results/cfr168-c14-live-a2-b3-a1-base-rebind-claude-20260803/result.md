# CFR-168 C14-LIVE-A2/B3 — A1-base rebind authoring result

Date: 2026-08-03

Role: contract author and evidence writer (`docs_synchronizer`). Documentation
only.

Batch / task: `cfr168-c14-a2-a1-base-rebind-claude-20260803` /
`c14-a2-a1-base-rebind`.

This result changes no product, test, canonical source document, ADR, schema,
packaging, issue state, install state, or dogfood state. It does not implement
A2/B3, accept a review, freeze a candidate, integrate anything, or close any
issue or lifecycle stage.

## 1. Lane identity and environment

| Item | Value |
|---|---|
| Model | Claude Opus 5, exact id `claude-opus-5` |
| Effort tier | `high` |
| Interpreter | `/usr/bin/python3.10`, always `PYTHONDONTWRITEBYTECODE=1 … -B` |
| Launch base at entry | `615da7ae917d2a17d3bc97b00b8b0604092ab6ca`, `git status --porcelain` empty |
| Branch | `claude-agent/cfr168-c14-a2-a1-base-rebind-claude-20260803-c14-a2-a1-base-rebind` |

Not run, by ticket prohibition: Cargo/Rust, product Python discovery,
`demo.sh`, install, packaging, network work, and the AI aggregate
(`check_ai_development.py`). Their absence is a scope bound, not a pass.

## 2. Exact launch subject — recomputed

| Node | Commit | Tree | Parent |
|---|---|---|---|
| `P0` (superseded) | `333e41082ffc35c73c02a420511ae2f16839c5b8` | `c0ab3a09e4731ca27d43023be72e577487b11600` | `808513a65dbfbbffff8123792d59d9eadfd875b4` |
| Historical A2 packet | `e4d85f2158a3f4e353042ab352b749fa2a69eed3` | `0230e537abcd410ecd4fe582ad431417e4618570` | `333e4108…` |
| `P1` (manager adjudication) | `1e8dd545f27bd1f22879b067ad0e1b3f4e2ecbc4` | `fa59fe69ba69bd199ef2cf3757679c98668a0812` | `7bd4594f01db6d8298ca242db04b0f71ce34f040` |
| Launch | `615da7ae917d2a17d3bc97b00b8b0604092ab6ca` | `54ea713028b84cc1b36b795486072b38b35de263` | `1e8dd545…` |
| Candidate `C` | `e25695e0baa5c5172408cce42bd6af5803b0d663` | `2c6a9dd57b6d4ec01f67dc88a37d2cc5648b0580` | `615da7ae…` |

Launch cleanliness and scope: `git status --porcelain` empty at entry; launch
adds exactly the two historical packet paths (796 + 356 lines).

### 2.1 Cherry-pick byte equality

| Path | Blob at `e4d85f21…` | Blob at launch | Mode | Equal |
|---|---|---|---|---|
| `…/cfr168-c14-live-a2-b3-plan-freeze-20260802.md` | `7c1b9640aa1006a62ca653a54cbb3071dd2670ea` | `7c1b9640aa1006a62ca653a54cbb3071dd2670ea` | `100644` | yes |
| `…/json/cfr168-c14-live-a2-b3-plan-freeze-20260802.json` | `eb4be319786ecf0a98260667c1c3c12911177b7b` | `eb4be319786ecf0a98260667c1c3c12911177b7b` | `100644` | yes |

Historical file SHA-256:
`be8438198330d99e5ec35a72dc03df227a147a18a114846605eed8e9d9712563` (Markdown),
`1636cd49057def6303cd8c6fb85b2ee6065d2c7aadf977956e61939b3ee0af6c` (JSON).

### 2.2 A1 chain — every node recomputed here

```text
P0..P1                       count = 11   merges = 0
merge-base --is-ancestor P0 P1            -> true
```

| Node | Commit | Tree | Exact scope recomputed |
|---|---|---|---|
| C4 | `19010f742d358c5c03e19d091c4249420f9e53b9` | `a0ec4ee256f63c7fe7de438eb5c978c918b3d22f` | `M codefire`, `M tests/test_python_http_upload_accounted_traversal.py` |
| O1 | `46d98466c702f111b58a3c1ea080518f75a59580` | `626b28d1989ef21bbc23f94288845be5b0082c90` | `M tests/test_python_existing_terminal_owner.py` only |
| W5 | `c57ac240cc6dc2011f78ba6148123804b9aa4258` | `daee74c0f8630aaef0c92e779bcc965fc75bd03c` | `A …/result.md` only |
| R5 | `7bd4594f01db6d8298ca242db04b0f71ce34f040` | `d3bd3a2c60fab7e34e25726305733bf91a2fe134` | `A …/review.md` only |
| `P1` | `1e8dd545f27bd1f22879b067ad0e1b3f4e2ecbc4` | `fa59fe69ba69bd199ef2cf3757679c98668a0812` | `A …/manager-adjudication.md` only |

R5 raw report SHA-256 recomputed:
`2facd0caef19f993f7e33e5be07a6f4e80ee4b7855756f6d1b49d584565d1a08` — matches
the value the manager adjudication binds.

`codefire` blob/mode: `73b1f368f9c1591be6fd9219b839351878b82b9a` `100755` at
`P0`; `15c7ebf5e2c53216c1aaa1add12070cffc12cbdd` `100755` at `P1`. File
SHA-256 `459d01a1…` (`P0`, 1 506 968 bytes) and `dc40d2c0…` (`P1`,
1 530 770 bytes).

## 3. Semantic-delta result — recomputed, not inherited from R5

### 3.1 A2-owned surfaces

Extraction used `ast.get_source_segment`, which reproduces all seven historical
`P0` hashes exactly. All seven surfaces are byte-identical **and**
AST-identical at `P1`, moved by exactly `+475` lines.

| Surface | `P0` lines | `P1` lines | Bytes | Text SHA-256 (`P0` = `P1`) | AST SHA-256 (`P0` = `P1`) |
|---|---|---|---:|---|---|
| `_PreparedHTTPResponseOwner` | 16037-16089 | 16512-16564 | 1748 | `82d4ca7b…63c04` | `12e71966…6fce3` |
| `_PreparedHTTPMutationResponses` | 16092-16243 | 16567-16718 | 5769 | `16a4501d…3cf25c` | `2aa732ef…2ea6ca` |
| `_HTTPMutationResponseSlot` | 16256-16325 | 16731-16800 | 2635 | `3c3fae78…43d9b9` | `0435a770…368a7` |
| `handle_http_upload` | 14720-15226 | 15195-15701 | 21581 | `b9d032b6…073732` | `7647655f…dd42c` |
| `publish_prepared` | 17221-17245 | 17696-17720 | 1168 | `99af445f…2d1940` | `1ddc1122…e178c` |
| `handle_one_request` | 17264-17419 | 17739-17894 | 7834 | `af8a6a83…3eaf007` | `8c6ede01…5dd60b` |
| `do_POST` | 17635-17867 | 18110-18342 | 11265 | `ff31cb1f…e301e1a` | `3668ac02…5edcdbed` |

Full digests are bound in the rebound packet. Historical byte counts reproduce
exactly for all seven rows, including the three nested methods whose spans
begin at the `def` keyword.

Hunk-level confirmation: the `P0..P1` `codefire` diff has 144 hunks over the
old-side range 5003-32673, and **none** intersects any A2-owned `P0` span.

### 3.2 Classification of every changed `P0..P1` product unit

`codefire` AST units: 1331 at `P0`, 1337 at `P1` — 6 added, 0 removed, 43
AST-changed, 0 text-only-changed. Every one of the 49 is A1-owned.

| A1 owner group | Units | A2-owned |
|---|---:|---|
| `_detach_effective_exception_tracebacks`, `_link_primary_cleanup_exception` (+ `capture`), `_retain_first_cleanup_failure` | 4 added | no |
| `AccountedServerVerification.close.attempt`, `SealedCommitValidationSession.close.attempt` | 2 added | no |
| `AccountedRemoteSignaturePolicyAuthority` + 9 methods | 10 changed | no |
| `AccountedCommitSignaturePolicy` + 6 methods | 7 changed | no |
| `AccountedServerVerification` + 10 methods, `run_server_verification` | 12 changed | no |
| `AccountedHTTPUploadAncestry` + 4 methods, `is_ancestor_from_objects_accounted`, `_detach_ancestry_exception_chain` | 7 changed | no |
| `_AccountedUploadCommitSource` + 2 methods, `_drained_commit_source_cleanup_error`, `enforce_remote_commit_signature_policy_accounted` | 5 changed | no |
| `SealedCommitValidationSession` + `.close`, `validate_sealed_commit` | 3 changed | no |

### 3.3 Classification of every changed `P0..P1` test unit

| Test module | Added units | AST-changed units | A2-owned symbols in added/removed lines |
|---|---:|---:|---:|
| `test_python_ancestry_borrow_handoff.py` | 0 | 4 | 0 |
| `test_python_ancestry_owner.py` | 7 | 1 | 0 |
| `test_python_existing_terminal_owner.py` | 0 | 3 | 0 |
| `test_python_external_production.py` | 1 | 3 | 0 |
| `test_python_external_verification.py` | 13 | 9 | 0 |
| `test_python_http_upload_accounted_traversal.py` | 62 | 1 | 0 |
| `test_python_signature_authority.py` | 8 | 1 | 0 |
| `test_python_signature_owner.py` | 3 | 2 | 0 |
| `test_python_signature_verification_handoff.py` | 0 | 4 | 0 |
| **Total** | **94** | **28** | **0** |

Across all nine modules, zero added or removed lines reference
`_PreparedHTTPResponseOwner`, `_PreparedHTTPMutationResponses`,
`_HTTPMutationResponseSlot`, `handle_http_upload`, `publish_prepared`,
`handle_one_request`, `do_POST`, `select_success`, or `select_durable`.
`tests/test_codefire_cli.py`, which carries the two bound A2 focused oracles,
is byte-identical at `P1` (`b23c43ce…`).

### 3.4 A2 path and evidence status at `P1`

Changed A2 write-set paths: `codefire`, `source-architecture.md`
(`ea54f24b…`), `command-source-trace.md` (`0c1f24c2…`),
`source-data-model-catalog.md` (`d687883f…`), `change-playbook.md`
(`8f460e13…`). Unchanged: `tests/test_codefire_cli.py`, `cfr-168.md`
(`29df1088…`). Absent at `P0`, `P1` and launch:
`tests/test_python_http_post_effect_response.py`,
`adr/ADR-021-python-http-mutation-response-lifecycle.md`.

All five bound evidence digests recomputed at `P1` and unchanged from `P0`:
`ca1a1451…`, `a3835f37…`, `02a759c4…`, `913b0a0b…`, `4566de10…`.

### 3.5 Verdict

**No A2-owned semantic surface differs, and no A1 exception behavior
contradicts `D1`-`D9` or `T01`-`T15`.** R5's claim is confirmed by independent
recomputation.

## 4. New observation

### O-R1 — A1 changed four callees reachable from `handle_http_upload`

`handle_http_upload` is byte-identical at `P1` but is not behaviorally frozen,
because four functions it calls changed under A1.

| Changed callee | Call line at `P1` | Position vs `fence_first_effect` at 15401 / 15455 / 15620 |
|---|---:|---|
| `validate_sealed_commit` | 15309 | before every fence — strictly pre-effect |
| `enforce_remote_commit_signature_policy_accounted` | 15364 | before every fence — strictly pre-effect |
| `run_server_verification` | 15374 | before every fence — strictly pre-effect |
| `is_ancestor_from_objects_accounted` | 15481 | after the layout-bootstrap fence, before the publication-effect fence and before `select_success`/`select_durable` at 15657-15668 |

A1's new helpers return the exact primary exception object and cannot alter its
type or `str()`; they set `__traceback__` to `None` and rewrite
`__cause__`/`__context__`/`__suppress_context__` on the retained chain. That is
consistent with `D9`'s pre-`M` row and `T01`'s "primary identity preserved"
oracle. The *observable chain and traceback shape* in the pre-`M` arms `T01`
and `T13` exercise does change, and `is_ancestor_from_objects_accounted` sits
in the region where `D1`/`H01`/`H02` require the implementer to fix `M`.

| Axis | Value |
|---|---|
| Impact state | `affected` — `development_flow` and `correctness` (oracle baselines), not a frozen-decision violation |
| Frozen requirements | unaffected: `D1`-`D9`, `T01`-`T15`, hazards, write set and non-goals all hold unchanged |
| Affected surfaces | `handle_http_upload` pre-`M` arms; `T01`, `T13`, `T14`, `T15` baselines; `D1` `M` placement |
| Reachability / likelihood | reachable on every Upload request; certain to matter to a future implementer |
| Blast radius | bounded to A2 baseline derivation; no A1 or product change implied |
| Reversibility | fully reversible — a documentation obligation |
| Evidence confidence / availability | high / `available`; recomputed from Git objects and AST in this lane |
| Must-fix trigger | none proved. `MF-1` not met: no frozen clause is violated. `MF-5` not met: the required evidence is obtainable on `L` and is now mandated |
| Disposition | `nonblocking_improvement` — carried into the rebound packet as a binding recompute obligation, not a repair |
| Causal class | null (only `required_correction` carries one) |
| Actionability | actionable by the future A2 implementer at stage 2; no action for A1 or for this lane |
| Owner | manager (commissioning) → future A2 implementer (execution) |

Recorded in the packet as `INV-BASELINE-RECOMPUTE`, `VAL-BASELINE-RECOMPUTE`,
and the *Mandatory baseline recomputation on `L`* section.

## 5. Rebind content — what changed and what did not

Byte-identical after the rebind (SHA-256 over each section):

| Preserved section | Status |
|---|---|
| Frozen design `D1`-`D9` | identical `92dd2b9f…` |
| Execution routing | identical `4e5d52a0…` |
| Nine-path write set + allowed product surface | identical `1e578683…` |
| `T01`-`T15` matrix rows | identical `8cefb04a…` |
| Hazards `H01`-`H15` | identical `57e2d2d7…` |
| Non-Goals (Markdown) | identical `40c720f0…` |

Changed, per the ticket's seven required items:

1. Header marked `2026-08-03 A1-base rebind`; new *Superseded base authority —
   retained, not erased* table binds the historical packet commit, tree,
   parent, both blobs and both file SHA-256 values, plus the launch commit.
2. `P` replaced by `P1` `1e8dd545…`; identity table now carries `P1` and `P0`
   columns with recomputed tree, parent, subject, blobs and modes; the
   `P0`/`P1` segment table carries current line spans and text/AST hashes.
3. New *A1 product/test lineage bound by `P1`* section binds C4, O1, R5 and the
   manager decision with exact scope, and states precisely which paths and AST
   units changed and which A2-owned units are byte/AST identical.
4. Execution-base rule `L` replaced: six predicates requiring the rebound
   packet, typed companion, accepted independent rebind preflight, explicitly
   named management-only records, linear non-merge descent from `P1`, and no
   product/test/source-doc/ADR/schema/packaging/issue-detail difference from
   `P1`. `P0` is explicitly never a valid `L`.
5. New *Mandatory baseline recomputation on `L`* section (5 clauses) plus a
   `P0`-coordinates warning at the head of *Reproduced root cause* and in the
   segment paragraph.
6. Typed companion updated: `base_revision`, objective, risk rationale,
   `additional_files`, `read_set`, 51 `evidence_refs` (authorities, protected
   segment identities, dependency lineage, semantic delta, negative space), 6
   new `disallowed_actions`, 2 new non-goals, `INV-BASE` rewritten, new
   `INV-REBIND-DELTA` and `INV-BASELINE-RECOMPUTE`, new `VAL-BASE-P1`,
   `VAL-A2-SEGMENTS`, `VAL-BASELINE-RECOMPUTE`, and 4 new handoff outputs. No
   behavioral oracle was weakened; every historical invariant and validation
   entry is retained verbatim.
7. New *What the 2026-08-03 rebind is not* section plus lifecycle stages 0 and
   1, and a matching `negative-space:` evidence ref.

Self-check table gained five rows binding single product authority, base-rule
satisfiability, A2-surface identity, historical-coordinate status, and the
changed-callee obligation.

## 6. Markdown / JSON consistency

| Bound value | In Markdown | In JSON |
|---|---|---|
| `P1` commit / tree / parent / `codefire` blob | yes | yes |
| Superseded `P0` | yes | yes |
| C4, O1, R5 report SHA-256 | yes | yes |
| Historical packet commit, launch commit | yes | yes |
| All 7 A2-owned segment text hashes | yes | yes |

`base_revision` equals `P1`. `packet_id` unchanged
(`cfr168-c14-live-a2-b3-plan-freeze-20260802`), `status` `ready`, `role`
`manager`, `schema` `codefire.ai_task_packet.v1`. `write_set` is the same nine
paths in the same order and matches the Markdown list. `D1`-`D9`, `T01`-`T15`
and `H01`-`H15` all present. No `write_set`/`disallowed_paths` overlap.

## 7. Validation

| # | Command | Exit | Duration | Log digest |
|---:|---|---:|---:|---|
| 1 | Git identity/topology/path/blob/digest checks for `P0`, A1 chain, `P1`, launch, candidate, result | 0 | — | inline in §2 |
| 2 | AST/source comparison of all seven A2-owned surfaces `P0` vs `P1`, plus classification of every changed product/test unit | 0 | ~35 s | inline in §3 |
| 3 | `docs/development/tools/check_ai_acceleration_docs.py` | 0 | 0.28 s real | `c5acd996…62a33d` |
| 4 | `docs/development/tools/check_ai_contracts.py` | 0 | 0.20 s real | `b04cb4dd…83d3e85` |
| 5 | `docs/development/tools/check_project_docs.py --json` | 0 | 0.72 s real | `59b1deae…b73c8808` |
| 6 | `docs/development/tools/check_source_docs.py --json` | 0 | 0.15 s real | `8942f30e…82445ff9b` |
| 7 | `git diff --check` | 0 | — | empty |

Digests above were measured at candidate `C`, before this result file existed.
All four are identical to the pre-edit launch baseline, and both JSON reports
carry `"ok": true` (712 documents inventoried; `check_source_docs` schema
`codefire.source_docs_check.v1`).

Re-run after writing this result, all still exit `0` with `ok: true`. Three
digests are unchanged; `check_project_docs.py --json` becomes
`e6229bb30a2fb4cb9a1e7093a649734e6b6aa54a41c0c45d790b1120520bf58a` because the
inventory grows from 712 to 713 documents — this file. That is the only
difference, and it produces no finding.

All Python ran as `PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3.10 -B`.

### 7.1 Commit topology and scope

```text
615da7ae..e25695e0   count = 1   merges = 0
e25695e0^            = 615da7ae917d2a17d3bc97b00b8b0604092ab6ca
git diff --name-only 615da7ae..e25695e0
  docs/development/ai/task-packets/cfr168-c14-live-a2-b3-plan-freeze-20260802.md
  docs/development/ai/task-packets/json/cfr168-c14-live-a2-b3-plan-freeze-20260802.json
```

Candidate `C` = `e25695e0baa5c5172408cce42bd6af5803b0d663`, tree
`2c6a9dd57b6d4ec01f67dc88a37d2cc5648b0580`, parent exact `615da7ae…`, two paths,
`+364 / -60` lines.

| Candidate path | Blob | Mode | File SHA-256 |
|---|---|---|---|
| `…/cfr168-c14-live-a2-b3-plan-freeze-20260802.md` | `3dcbe74a94298ff3277de079d1e1ca1a690d147e` | `100644` | `6a9d7f767eee87dfce3b0ace9f50e636981f79657647aa0963526a75c9e6c36a` |
| `…/json/cfr168-c14-live-a2-b3-plan-freeze-20260802.json` | `988573ee412cc8c90494c81d831848c24a315008` | `100644` | `331175f4b65478f4662ac3f964dd3b091ea6fa4c965c16eb0d3456f6133fe455` |

Canonical binary patch SHA-256 for `615da7ae..e25695e0`:
`ea791c614ca0df389f16cf3a4e7c221f0d8b31630da42c147d0988170ccde1c7`.

This result is the sole path in a single result-only child of `C`. Its own
commit and tree are not self-bindable and are reported in the lane handoff.

## 8. Unavailable facts and limitations

- `L` does not exist yet. Every `L`-bound predicate in the rebound packet is an
  obligation, not a verified observation.
- The result child's own commit/tree cannot be bound inside itself.
- Runtime behavior of the four changed callees was **not** executed. The
  no-contradiction verdict rests on static AST/source analysis plus the A1
  helpers' construction, not on running the pre-`M` arms; that runtime proof is
  exactly what `VAL-BASELINE-RECOMPUTE` requires on `L`.
- Product Python discovery, `demo.sh`, install, packaging, Cargo/Rust, network
  work and the AI aggregate were not run, by ticket prohibition. Their results
  are unavailable, not passing.
- The `+475` uniform line shift is a property of `P0..P1` only; a future `L`
  containing additional records could shift it again, which is why symbol
  resolution on `L` is mandated over line numbers.
- R5's A2-surface claim was recomputed rather than trusted; no conclusion here
  depends on R5's or W5's arithmetic.

## 9. Disposition

The rebind is complete and authority is single-valued: `P1` is the only product
base, the rebound `L` rule is the only execution-base rule, and the packet is
implementable without an authority choice. No required update changed A2
behavior, so no BLOCK condition arose. The historical design is intact.

The next step is a fresh, independent exact rebind preflight against `P1` by a
lane excluded from this authoring lane. Neither this result nor that preflight
is A2 implementation, review acceptance, integration, Install, Dogfood, or
Close, and A2/B3 remains open.

RESULT_DISPOSITION: READY_FOR_REBIND_PREFLIGHT
