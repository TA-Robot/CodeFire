# CFR-168 C14-LIVE-A2/B3 accepted-A1 base rebind — independent preflight

Date: 2026-08-03

Lane: fresh independent Claude `claude-opus-5` contract reviewer, report-only.

Subject: commit `e25695e0baa5c5172408cce42bd6af5803b0d663`
(`docs(cfr168): rebind C14 A2/B3 packet to accepted A1 base`).

This report is documentation-only. It changed no product, test, source doc, ADR,
schema, packaging, issue, install, or dogfood state, and it did not edit the
packet or its typed companion. Every value below was recomputed from Git objects
and live source; no packet self-report was trusted.

## 1. Verdict summary

Every `BLOCK` trigger named in the ticket — identity, ancestry, path,
dirty-state, JSON/Markdown, and claimed-digest mismatch — was checked and is
clear. All 7 A2-owned text digests, all 7 AST digests, all 5 bound evidence
digests, both historical packet digests, and the R5 report digest reproduce
exactly. All spans, byte counts, the uniform `+475` shift, the unit counts, the
test-unit counts, the callee lines, and the three fence lines reproduce exactly.

The rebind is sound and implementation-ready. Six non-blocking observations are
recorded in §10; none is a required correction and none was fixed in this lane.

## 2. Environment and exact commands

| Item | Value |
|---|---|
| Python | `Python 3.10.12 (main, Mar 3 2026, 11:56:32) [GCC 11.4.0]` at `/usr/bin/python3.10`, run as `PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3.10 -B` |
| Git | `git version 2.34.1` |
| Worktree at finish | `git status --porcelain` empty except this one report |

Only read-only Git/object/source analysis and the ordinary Python documentation
checkers were run. No product test, Cargo, Rust, demo, install, packaging, or
network work was performed.

| Command | Exit |
|---|---:|
| `git merge-base --is-ancestor 333e4108 1e8dd545` | 0 |
| `git rev-list --parents --topo-order 333e4108..1e8dd545` | 0 |
| `git rev-list --count 333e4108..1e8dd545` → `11` | 0 |
| `git rev-list --merges 333e4108..1e8dd545` → empty | 0 |
| `git cat-file -p` / `git ls-tree` / `git rev-parse` identity set (§3) | 0 |
| `git diff --name-status 333e4108 1e8dd545` | 0 |
| `git diff -U0 333e4108 1e8dd545 -- codefire` (hunk analysis) | 0 |
| segment recomputation script (§5) | 0 |
| AST-convention search script (§5.2) | 0 |
| unit-classification script (§6) | 0 |
| test-unit-count script (§6.2) | 0 |
| Markdown/JSON cross-check script (§8) | 0 |
| `check_project_docs.py` → `Project docs check passed; 712 Markdown files inventoried.` | 0 |
| `check_source_docs.py` → `Source docs check passed: 44 modules, 34 commands, 12 object kinds.` | 0 |
| `check_ai_contracts.py` → `AI contract check passed.` | 0 |
| `check_ai_acceleration_docs.py` → `AI acceleration docs check passed.` | 0 |
| `check_ai_development.py` (default `--timeout 60`) → `FAIL tool_tests` | 1 |
| `check_ai_development.py --timeout 300` → all seven stages `PASS`, `AI development gate passed.` | 0 |
| `git diff --check` on this report | 0 |

All four documentation checkers were re-run with this report present and pass:
`Project docs check passed; 713 Markdown files inventoried.`, plus the source-doc,
contract, and acceleration-doc checks, and the full gate at `--timeout 300`. One
self-inflicted defect was found and fixed during that pass: a code span in §10
`N3` that placed a bracketed subscript immediately before a parenthesised
ellipsis parsed as a Markdown inline link and tripped `broken-local-link`; the
prose was reworded and the checker now passes. No other file was touched.

The default-timeout failure is an environmental artifact, not a subject defect;
see observation `N6` in §10. The subject commit touches no tool or test path
(`git show --name-only e25695e` lists only the two packet paths).

## 3. Subject identity, ancestry, and write set — all exact

| Claim | Recomputed | Match |
|---|---|:---:|
| Subject commit | `e25695e0baa5c5172408cce42bd6af5803b0d663` | yes |
| Subject tree | `2c6a9dd57b6d4ec01f67dc88a37d2cc5648b0580` | yes |
| Sole parent | `615da7ae917d2a17d3bc97b00b8b0604092ab6ca` (single parent) | yes |
| Subject write set | exactly `M` the Markdown packet and `M` the typed JSON companion | yes |
| `P1` | `1e8dd545f27bd1f22879b067ad0e1b3f4e2ecbc4`, tree `fa59fe69ba69bd199ef2cf3757679c98668a0812`, parent `7bd4594f01db6d8298ca242db04b0f71ce34f040`, subject `docs(cfr168): accept C14 A1 source package` | yes |
| `P0` | `333e41082ffc35c73c02a420511ae2f16839c5b8`, tree `c0ab3a09e4731ca27d43023be72e577487b11600`, parent `808513a65dbfbbffff8123792d59d9eadfd875b4`, subject `docs(cfr168): synchronize accepted source state` | yes |
| Rebind launch commit | `615da7ae…`, tree `54ea713028b84cc1b36b795486072b38b35de263`, parent `1e8dd545…`, adds exactly the two packet paths | yes |
| Historical packet | `e4d85f2158a3f4e353042ab352b749fa2a69eed3`, tree `0230e537abcd410ecd4fe582ad431417e4618570`, parent `333e4108…`, adds exactly the two packet paths | yes |

### 3.1 Blob and mode table (§"Immutable source authority `P1`")

Every cell reproduces, including both modes:

| Path | `P1` | `P0` |
|---|---|---|
| `codefire` | `15c7ebf5e2c53216c1aaa1add12070cffc12cbdd` `100755` | `73b1f368f9c1591be6fd9219b839351878b82b9a` `100755` |
| `tests/test_codefire_cli.py` | `b23c43ced6ccb69c0bb553f9856c1a7b2145e770` | identical |
| `source-architecture.md` | `ea54f24b70e3c2ba36896e620d00c597c6e0a6bb` | `fe2b3b74f2beaf30218333f14e05b6e4e6d4e699` |
| `command-source-trace.md` | `0c1f24c25ab981de304591e7ab8dd9aed7c47ecf` | `361a5de8f9f881fbedf35ed5a833baade9529e38` |
| `source-data-model-catalog.md` | `d687883f1630508319b29ff98d20b8e164d474bf` | `2b68347eaa357f187f794c2a76992401743929eb` |
| `change-playbook.md` | `8f460e1315b5dc51b53325e7d450aaf7845d2005` | `18d882813e02ed8cd795aa1e46d8cac6a09be4ef` |
| `cfr-168.md` | `29df10886dab2d5950df907d31238e8ea5ce1a8d` | identical |

`tests/test_python_http_post_effect_response.py` and
`adr/ADR-021-python-http-mutation-response-lifecycle.md` are absent at `P0`,
`P1`, the rebind launch commit, and the subject. Both correctly remain
create-paths.

## 4. Ticket item 1 — ancestry, topology, and byte-for-byte packet carriage

**Proved.**

- `git merge-base --is-ancestor 333e4108 1e8dd545` exits 0, and the two ids are
  distinct, so `P0` is a **strict** ancestor of `P1`.
- `P0..P1` is exactly **11 commits with zero merges**. Every commit in the range
  has exactly one parent; the chain is `333e4108` → `0f28f477` → `85c29974` →
  `4d24460d` → `46ed2a95` → `8ff33fd9` → `19010f74` → `f29019ad` → `46d98466` →
  `c57ac240` → `7bd4594f` → `1e8dd545`.
- The four bound A1 lineage nodes occur in the claimed order at chain positions
  6 (`19010f74` C4), 8 (`46d98466` O1), 10 (`7bd4594f` R5), 11 (`1e8dd545` P1).
- `615da7ae…` carries both historical packet blobs **byte-for-byte**: blob ids
  `7c1b9640aa1006a62ca653a54cbb3071dd2670ea` (Markdown) and
  `eb4be319786ecf0a98260667c1c3c12911177b7b` (JSON) are *identical object ids*
  at `e4d85f21` and `615da7ae`, and the raw SHA-256 values
  `be8438198330d99e5ec35a72dc03df227a147a18a114846605eed8e9d9712563` and
  `1636cd49057def6303cd8c6fb85b2ee6065d2c7aadf977956e61939b3ee0af6c` reproduce
  at both commits. This is stronger than the packet's cherry-pick claim: the
  blobs are the same objects, not merely equal content.

### 4.1 A1 lineage scope claims

| Node | Recomputed tree | Recomputed scope | Match |
|---|---|---|:---:|
| C4 `19010f74` | `a0ec4ee256f63c7fe7de438eb5c978c918b3d22f` | `M codefire`, `M tests/test_python_http_upload_accounted_traversal.py` — two paths | yes |
| O1 `46d98466` | `626b28d1989ef21bbc23f94288845be5b0082c90` | one path, `tests/test_python_existing_terminal_owner.py`, exactly two changed expected literals (`partial verification close`→`partial verification construction`, `partial ancestry close`→`partial ancestry construction`), zero product bytes | yes |
| R5 `7bd4594f` | `d3bd3a2c60fab7e34e25726305733bf91a2fe134` | adds only `review.md`; raw SHA-256 `2facd0caef19f993f7e33e5be07a6f4e80ee4b7855756f6d1b49d584565d1a08`; terminal `REVIEW_VERDICT: ACCEPT` | yes |
| P1 `1e8dd545` | `fa59fe69ba69bd199ef2cf3757679c98668a0812` | adds only `manager-adjudication.md`; terminal `MANAGER_ADJUDICATION: ACCEPT_A1_SOURCE_PACKAGE` | yes |

The A1 manager adjudication independently directs this rebind: *"A2/B3 must be
rebound to the resulting manager adjudication descendant and must recompute its
response/wire tests against A1 exception behavior."* It also reserves A2/B3,
Install, Dogfood/Evaluate, final Close, and Retrospect, and forbids whole-file
`codefire` selection — all three constraints are faithfully carried into the
rebound packet.

## 5. Ticket item 2 — the seven A2-owned segments recomputed at `P0` and `P1`

**Proved, using `ast.get_source_segment` as the packet specifies.** Each surface
occurs exactly once at each of `P0` and `P1`.

| Surface | `P0` span | `P1` span | Shift | Bytes | Text SHA-256 `P0`=`P1`=claim | AST SHA-256 `P0`=`P1`=claim |
|---|---|---|---:|---:|:---:|:---:|
| `_PreparedHTTPResponseOwner` | 16037-16089 | 16512-16564 | +475 | 1748 | yes | yes |
| `_PreparedHTTPMutationResponses` | 16092-16243 | 16567-16718 | +475 | 5769 | yes | yes |
| `_HTTPMutationResponseSlot` | 16256-16325 | 16731-16800 | +475 | 2635 | yes | yes |
| `handle_http_upload` | 14720-15226 | 15195-15701 | +475 | 21581 | yes | yes |
| `publish_prepared` | 17221-17245 | 17696-17720 | +475 | 1168 | yes | yes |
| `handle_one_request` | 17264-17419 | 17739-17894 | +475 | 7834 | yes | yes |
| `do_POST` | 17635-17867 | 18110-18342 | +475 | 11265 | yes | yes |

Both the start and the end line of every surface move by exactly `+475`. Raw
segment text is byte-identical `P0`→`P1` for all seven, so byte counts and text
digests are necessarily equal — and each equals the packet's claimed value.

### 5.1 Line movement is consistent with the diff geometry

The `P0..P1` `codefire` diff has **144 hunks**, old-side changed range
**5003-32673** — exactly as claimed. **No hunk intersects any A2-owned `P0`
span**, and **no hunk touches the claimed gap 14720-17867** (checked for both
overlapping hunks and zero-length pure insertions falling inside a span). The
`+475` shift is therefore fully attributable to insertions strictly below line
14720, and the byte-identity result is corroborated geometrically as well as by
digest.

### 5.2 The AST-hash convention is unstated but recoverable

The packet fixes the segment convention (`ast.get_source_segment`) and the text
digest, but never states how the **AST** digest is computed. My first reasonable
choice — `sha256(ast.dump(ast.parse(segment), include_attributes=False))`, i.e.
hashing the re-parsed `Module` wrapper — mismatched **all seven** claimed
values. I then searched 20 conventions per surface. Exactly one family
reproduces all seven claimed digests simultaneously:

```
sha256(ast.dump(node, annotate_fields=True, include_attributes=False))
```

on the `ClassDef`/`FunctionDef` node itself (equivalently, `body[0]` of the
re-parsed segment) — the default `ast.dump` signature. Under that convention all
seven claimed AST values reproduce exactly at `P1` and equal their `P0` values.
`P0`==`P1` AST equality also holds under *every* convention I tried, so the
invariant's substance is robust; only the literal hex string is
convention-dependent. Recorded as non-blocking observation `N1`.

## 6. Ticket item 3 — classification of every `P0..P1` product and test change

**The claim that no A2-owned surface changed is not falsifiable; I tried and
failed to falsify it.**

`codefire` holds **1331** AST units at `P0` and **1337** at `P1` (counting all
nested `def`/`class` units by qualified name). Recomputed delta: **6 added, 0
removed, 43 AST-changed, 0 text-only-changed** — all four figures match, and
`6 + 43 = 49` matches the packet's aggregate. **Zero** of the 49 is A2-owned,
under a check that tests every qualified-name component against the seven
A2-owned names.

The six added units are exactly the ones named:
`_detach_effective_exception_tracebacks`, `_link_primary_cleanup_exception`,
`_link_primary_cleanup_exception.capture`, `_retain_first_cleanup_failure`,
`AccountedServerVerification.close.attempt`,
`SealedCommitValidationSession.close.attempt`.

All 43 AST-changed units are A1-owned pre-effect cleanup/authority units, in the
groups the packet names. Six of the eight group rows reproduce exactly; one row
overcounts by one — see observation `N2` in §10.

### 6.1 Changed `P0..P1` path set

22 paths differ, matching the packet exactly: `codefire`; nine
`tests/test_python_*.py` modules; four canonical source docs; the fifth doc
`docs/development/known-limitations.md`; and seven added A1 packet/result
documents. No other path differs.

### 6.2 Test-unit classification

Recomputed across the nine A1-changed modules: **94 added, 0 removed, 28
AST-changed** — both claimed figures exact.

| Module | Added | AST-changed |
|---|---:|---:|
| `test_python_ancestry_borrow_handoff.py` | 0 | 4 |
| `test_python_ancestry_owner.py` | 7 | 1 |
| `test_python_existing_terminal_owner.py` | 0 | 3 |
| `test_python_external_production.py` | 1 | 3 |
| `test_python_external_verification.py` | 13 | 9 |
| `test_python_http_upload_accounted_traversal.py` | 62 | 1 |
| `test_python_signature_authority.py` | 8 | 1 |
| `test_python_signature_owner.py` | 3 | 2 |
| `test_python_signature_verification_handoff.py` | 0 | 4 |
| **total** | **94** | **28** |

**Zero** added or removed test lines across all nine modules reference any of
the nine A2-owned symbols — verified against the raw `P0..P1` test diff. The
claim is exactly true as written. `tests/test_codefire_cli.py` is byte-identical
at `P1`, and both bound A2 focused oracles
(`test_python_upload_response_preflight_precedes_effect_and_retries_durable_result`,
`test_python_mutation_response_owner_rejects_foreign_and_retries_cleanup`) are
present there at `P1`.

Independently, no added or removed line in the `P0..P1` source-doc delta
references any A2-owned symbol either, so A1's canonical-doc edits do not
pre-empt the A2 documentation obligations.

## 7. Ticket item 4 — the four changed callees, the fences, and A1 exception behavior

**All coordinates and every position claim reproduce exactly at `P1`.**

| Changed callee | `P1` call line | Position relative to fences | Verified |
|---|---:|---|:---:|
| `validate_sealed_commit` | 15309 | before all three fences — strictly pre-effect | yes |
| `enforce_remote_commit_signature_policy_accounted` | 15364 | before all three fences — strictly pre-effect | yes |
| `run_server_verification` | 15374 | before all three fences — strictly pre-effect | yes |
| `is_ancestor_from_objects_accounted` | 15481 | after the layout-bootstrap fence (15455), before the publication-effect fence (15620) and before `select_success`/`select_durable` (15657-15668) | yes |

The three `fence_first_effect` call sites inside `handle_http_upload` are at
**15401** (`"upload final subject fence"`), **15455** (`"upload layout
bootstrap"`), and **15620** (`"upload publication effect"`). The phase labels
confirm the packet's fence naming, not merely the line numbers.

### 7.1 Exact traceback/cause/context behavior introduced by A1

Read at `P1`: `_detach_effective_exception_tracebacks` (5004-5011),
`_link_primary_cleanup_exception` (5014-5049), `_retain_first_cleanup_failure`
(5052-5078). The packet's characterisation is accurate and complete:

- **Primary object, type, and `str()` are preserved.**
  `_link_primary_cleanup_exception` receives `primary`, mutates only its dunder
  chain fields, and `return primary` — the same object. It never constructs a
  replacement, and it never touches `args`.
- **`__traceback__` is set to `None`** along the effective chain
  (`__cause__ or __context__` walk, with `id()`-keyed cycle guards in all three
  helpers).
- **`__cause__`/`__context__`/`__suppress_context__` are rewritten** on the
  retained chain: all links are first reset
  (`__cause__ = None`, `__context__ = None`, `__suppress_context__ = True`),
  then re-linked as a deduplicated `__cause__` spine ordered cleanup-chain then
  primary-prior-chain, with `primary.__cause__ = ordered[0]`.
- `_retain_first_cleanup_failure` keeps exactly one cleanup fault and protects
  the already-retained chain from being cleared.

This matches the `change-playbook.md` text A1 landed at `P1` ("Preserve the
exact operation exception as the outer object; append the first cleanup
exception as explicit cause… identity deduplication and no cycles").

### 7.2 The root-cause section's historical `P0` coordinates are all accurate

Every `codefire:NNNNN` in the *Reproduced root cause* section resolves correctly
at `P0`, and the section explicitly labels them historical:

| Claim | `P0` evidence | Verified |
|---|---|:---:|
| `_select` records `selection` at 16143 | `self.selection = requested_selection` | yes |
| then calls rejected `close()` at 16144 | `rejected.close()` | yes |
| slot generic close at 16310-16319 | `def close` calling `responses.close()` then `selected.close()` | yes |
| `do_POST` generic arm slot close at 17812-17817 | `if mutation_slot is not None: try: mutation_slot.close()` | yes |
| second generic slot close in route-finally at 17849-17857 | `finally:` … `mutation_slot.close()` | yes |
| `select_success` retried once at 15184-15192, `select_durable` once at 15193 | `except BaseException:` arm with `publication_succeeded` branch | yes |
| `publish_prepared` puts socket writes *and* accounting cleanup inside `transport_boundary` at 17227-17240 | `with transport_boundary(): prepared.publish(self.wfile)`; `publish` does `write(header)`, `write(body)`, then `finally: self.close()`, and `close()` calls `self.ledger.release("retained_bytes", …)` | yes |

The last row is the one requiring two hops to confirm, and it holds: the ledger
release genuinely executes inside the transport boundary, so the misclassifica-
tion the packet describes is real, not asserted.

## 8. Ticket items 5 and 7 — contradiction search and Markdown/JSON alignment

### 8.1 No A1 delta contradicts any frozen decision

I looked for a contradiction against each axis the ticket names and found none:

- **Pre-`M` silence and primary identity.** A1 preserves the exact primary
  object/type/`str()`, so `D9`'s pre-`M` row ("Existing route/business/budget/
  deadline/filesystem error" as primary; "Preserve and annotate/log cleanup")
  and `T01`'s "primary identity preserved" oracle both still hold. A1 appends
  cleanup *below* the primary as `__cause__`, which is annotation, not
  substitution — consistent with `H09` ("Diagnostic becomes new primary") and
  with `D4`'s "preserves the route's original primary exception".
- **What genuinely changed** is the *observable chain and traceback shape* in
  the pre-`M` arms `T01` and `T13` exercise. The packet states exactly this and
  does not overclaim behavioral freeze: it says `handle_http_upload` "is
  byte-identical but is **not** behaviorally frozen".
- **Marker `M` placement.** `is_ancestor_from_objects_accounted` (15481) sits
  between the layout-bootstrap and publication-effect fences, i.e. precisely in
  the region `D1`/`H01`/`H02` govern. The packet does not pre-decide the
  question; it requires the implementer to adjudicate before/at-or-after `M`
  with exact `L` line and fence evidence, and `INV-BASELINE-RECOMPUTE`'s
  falsification test fails the work if that adjudication is omitted. That is a
  correctly delegated open decision with an oracle, not an unresolved authority
  choice.
- **Success/durable precedence (`D5`), pair/slot state machines (`D2`/`D3`),
  wire borrow (`D6`), terminal vocabulary (`D8`).** None of the 49 changed A1
  units is A2-owned, and no A1 test line touches an A2 symbol, so no frozen
  transition is contradicted. The vocabulary `D8` depends on exists at `P1`:
  `cleanup_incomplete` (17854, 17986), `complete`, `budget_exceeded`,
  `deadline_exceeded`, `HTTP_MUTATION_RETRY_PAYLOAD` (16721), and
  `transport_boundary` (1705). The three forbidden variants
  `publication_failed`, `wire_failed`, `response_abandoned` occur **zero** times
  at `P1`, so `D8`'s "adds no seventh advertised terminal" starts from a clean
  base.
- **Nine-path scope.** No A1 change requires a tenth A2 path. The related
  residual risk is recorded as `N3`.

The two superseded-source claims the design leans on are both accurate. The
historical shared-primitives preflight is indeed a `BLOCK` that leaves `B3`
unassigned and `RetryableCleanup` `shape-pending`, and its `C11` does demand a
terminal class "never `complete`". The later manager freeze does supersede
exactly that literal: "`DecisionWorkTerminalReason::Complete` is the
ADR-020-conformant terminal value. This lane must not add a seventh
terminal-reason variant." `D8`'s scoped supersession is therefore justified, and
the packet correctly preserves the preflight's other requirements.

### 8.2 Markdown and JSON are semantically aligned

- The JSON parses, and it is **canonically formatted**: the file is
  byte-identical to `json.dumps(obj, indent=2, ensure_ascii=False) + "\n"`.
- **All 22** 64-hex digests in the Markdown appear in the JSON and vice versa —
  **zero asymmetry in either direction**.
- All 13 of the JSON's 40-hex ids appear in the Markdown. The Markdown carries
  16 additional 40-hex ids (source-doc blobs, C4/O1/R5 trees, historical packet
  blob ids, `P0`'s parent); the Markdown is a superset, which is not a mismatch,
  and I recomputed every one of those extra values independently (§3.1, §4.1).
- All seven Markdown segment rows have an exactly matching JSON
  `protected-identity-*` evidence ref, agreeing on text digest, AST digest,
  `P0` span, and `P1` span.
- `base_revision` = `1e8dd545…` = `P1`. `status`, `packet_id` aligned.
- The JSON `write_set` equals the Markdown's nine paths **as an ordered list**,
  not merely as a set. No tenth path.
- 20 invariants, 13 validations, 15 non-goals, 51 evidence refs; the three new
  invariants (`INV-BASE` rewritten, `INV-REBIND-DELTA`, `INV-BASELINE-RECOMPUTE`)
  and three new validations (`VAL-BASE-P1`, `VAL-A2-SEGMENTS`,
  `VAL-BASELINE-RECOMPUTE`) each have a stated, executable falsification test or
  command. The two new non-goals and three new handoff obligations are
  checkable statements.

The packet's own `disallowed_paths` lists `docs/development/ai/swarm-results`,
and this preflight report is written there. That is not a contradiction: that
list bounds candidate `C`, and the packet independently requires preflight and
review reports to live **outside** `C`. The two rules are consistent.

## 9. Ticket item 6 — the new execution-base rule is satisfiable and fail-closed

**Satisfiable — demonstrated concretely, not argued.** At the subject commit,
`git diff --name-only 1e8dd545 e25695e` returns exactly the two rebound packet
paths; `git rev-list --merges 1e8dd545..e25695e` is empty; `P1` is an ancestor of
the subject; the range is the linear pair `615da7a` → `e25695e`. A commit adding
this accepted preflight report on top of the subject therefore satisfies
predicates 1-5 by construction, and predicate 6 is a worktree-cleanliness
precondition. The rule is satisfiable **after** A1 acceptance, which is the
defect it was written to repair.

**The old rule really was unsatisfiable, not merely stale.** The historical rule
at `e4d85f21` required `L` to descend from `P` = `P0` *and* to contain "no
product, test, ADR, source-doc, packaging, schema, or issue-detail difference
from `P`". Any base carrying A1's accepted product and test changes necessarily
differs from `P0` in `codefire` and nine test modules. No commit can satisfy
both conjuncts. The packet's justification (§"Why the rebind is required") is
accurate.

**`P0` is excluded, twice over.** Predicate 3 requires `L` to descend linearly
from `P1`; `P0` is a strict *ancestor* of `P1` and so cannot descend from it.
The packet also states it explicitly ("`P0` satisfies no predicate above and is
never a valid `L`"), and the JSON adds a matching `disallowed_action` covering
`P0` as `L`, as product authority, **and** as an oracle baseline.

**Unnamed changes are excluded, fail-closed.** Predicate 4 restricts the `P1..L`
delta to the two packet paths, this preflight report, and management-only
records "each named in the launch prompt by exact path"; predicate 5 excludes
any product, test, source-doc, ADR, schema, packaging, or issue-detail
difference outright. The predicates are conjunctive and the rule ends "The
implementer must stop if any predicate fails", so an unrecognised path fails
closed rather than being tolerated. Editing the typed `base_revision` to widen
the base is separately forbidden in both files.

One hardening gap in the *proof block* rather than the rule is recorded as `N5`.

## 10. Non-blocking observations

None is a required correction; none was fixed in this lane. Ordered by
usefulness to the manager.

**`N1` — the AST-hash convention is unstated, and a reasonable reading yields a
false base violation.** `INV-REBIND-DELTA` and `VAL-A2-SEGMENTS` require the
implementer to recompute the seven AST digests on `L` and treat inequality as "a
base violation requiring a BLOCK return to the manager". The packet fixes the
segment convention and the text digest but never states the AST-digest
convention. Hashing the re-parsed `Module` wrapper — the first thing I tried —
mismatches all seven claimed values, which would manufacture a spurious `BLOCK`.
Suggested manager action: have the launch prompt name the convention, which I
recovered as `sha256(ast.dump(node, annotate_fields=True,
include_attributes=False))` on the `ClassDef`/`FunctionDef` node. Not a blocker:
the claimed values do reproduce under the standard default, the seven text
digests pin the same fact unambiguously, and `P0`==`P1` AST equality held under
every convention tested.

**`N2` — one A1 owner-group row overcounts by one; the group table sums to 50
against a correct aggregate of 49.** The row
"`AccountedRemoteSignaturePolicyAuthority` and 9 methods | 10 changed" is wrong:
the class has 32 units at `P0`, of which the class itself plus **8** methods
changed (`__init__`, `_open_absolute_directory`, `_open_subject`, `_read_policy`,
`_revalidate_policy`, `_revalidate_project`, `close`, `prepare_policy`) — **9**
changed units, not 10. Summing the table's Units column gives
`4+2+10+7+12+7+5+3 = 50`, while the packet's own aggregate and the JSON both
correctly say `6 added + 43 AST-changed = 49`. The other seven rows reproduce
exactly. Impact is descriptive only: the aggregate is right, the JSON is right,
and the load-bearing conclusion — none of the 49 is A2-owned — is independently
confirmed. No invariant or validation consumes the per-group numbers.

**`N3` — two A1-changed test modules drive `handle_http_upload` as their
production entry point but are not in the nine-path write set.** At `P1`,
`tests/test_python_http_upload_accounted_traversal.py` (which A1 grew by 62
units / +943 lines) and `tests/test_python_external_production.py` both call
`handle_http_upload` through `self.namespace`, and the former also reasons
about `handle_one_request`'s `finally`/`close_connection` semantics. A2 must
change `handle_http_upload` at the seal/`M`/selection/cleanup seams and
`handle_one_request`, so this is the most likely trigger of the packet's own
tenth-path stop rule. This is **not** a rebind defect and **not** a
counterexample to the packet: the 7 references are pre-existing at `P0` and A1
added none, so the packet's claim ("zero *added or removed* test lines reference
an A2-owned symbol") is exactly true as written, and the tension exists
identically under `P0`. The contract already fails closed — `VAL-FULL-PYTHON`
would surface the breakage and the stop rule forbids improvising a tenth path.
Suggested manager action: name this risk in the launch prompt so the implementer
returns a blocker promptly rather than discovering it late.

**`N4` — `docs/development/known-limitations.md` describes the mutation-response
pair but is unowned by A2.** Line 181 at `P1` documents the current Python
mutation-response pair limitation. A1 changed this file; A2's write set contains
the four canonical source docs and `cfr-168.md` but not this one, and `INV-DOCS`
does not cover it. If the A2 repair makes that entry stale, updating it would be
a tenth path. The stop rule is the fail-closed valve; flagged so the manager can
decide deliberately rather than by omission.

**`N5` — the base-rule proof block is not fully self-asserting.** Four of the
six shell lines are `test`-guarded, but `git diff --name-only "$P1".."$L"` is
printed for inspection against a prose criterion rather than asserted, and
nothing asserts that `L`'s two packet paths carry the *rebound* bytes — predicate
1 ("byte-for-byte") is checked by name, not by content. A blob-identity
assertion against the `L`-side blob ids would close this. Hardening only: the
criterion is stated precisely enough to evaluate deterministically, and
`INV-BASE`'s falsification test covers a missing packet or preflight.

**`N6` — `check_ai_development.py` fails `tool_tests` at its default timeout on
this machine.** The gate's default is 60 s per check; `docs/development/tools/
test_ai_tools.py` runs 266 tests in 60.557 s here, so the stage times out. Run
directly the suite is `OK` (exit 0), and the full gate passes all seven stages
at `--timeout 300`. Environmental and unrelated to the subject, which touches no
tool or test path. Recorded so the next lane does not misread it as a
regression.

## 11. Preserved negative space

This preflight is **only** an independent contract review of commit
`e25695e0…`. It is explicitly **not**, and does not imply, any of the following:

- A2/B3 implementation, or any product, test, ADR, source-doc, schema,
  packaging, or issue-state change;
- acceptance of a candidate, or a Candidate Freeze;
- integration of A2/B3, or of this rebind, into the mainline — the manager owns
  final integration;
- Install, Dogfood/Evaluate, Retrospect, or final Close of CFR-168, CFB-136,
  CFB-138, global C14, MR-write, or the active batch;
- a reopening, re-adjudication, or widening of the accepted C14-LIVE-A1 source
  package — A1's acceptance is treated here as settled authority and was
  reviewed only as the rebind's base; or
- naming the execution base `L`, which remains a manager act.

A2/B3 itself remains open. No finding in §10 was repaired in this lane, and no
finding is a required-correction trigger.

## 12. Unavailable evidence and limitations

- The AST-hash convention was **recovered by search**, not read from the packet
  (`N1`). All seven values reproduce under the recovered convention.
- No product test, Cargo, Rust, `demo.sh`, install, packaging, or network work
  was run, per the ticket. `T01`-`T15` behavior was therefore reviewed as a
  contract only; the packet correctly assigns their observation to the
  implementer on `L`, and this report makes no claim about their runtime
  outcome.
- `docs/development/tools/test_ai_tools.py` executed once as a side effect of
  the allowed `check_ai_development.py` documentation gate, and once directly to
  diagnose that gate's timeout (`N6`). It is a tools suite, not a product test.
- The `+475` uniform shift and byte-identity were verified at `P0` and `P1`. The
  packet's obligation to re-derive them on `L` remains the implementer's, and is
  correctly stated as a current-source obligation.

## 13. Conclusion

Every recomputable claim in the rebound packet and its typed companion
reproduces exactly from Git objects and live source: identities, trees, parents,
modes, ancestry, topology, byte-for-byte packet carriage, all 7 A2-owned text
digests, all 7 AST digests, all spans and byte counts, the uniform `+475` shift,
the diff-hunk non-intersection, the 1331→1337 unit classification, the 94/28
test-unit classification, the four callee lines, the three fence lines and their
phase labels, every historical `P0` root-cause coordinate, all 5 bound evidence
digests, and the full A1 lineage. Markdown and JSON are semantically aligned
with zero digest asymmetry. The rebound execution-base rule is satisfiable,
fail-closed, excludes `P0` twice over, and admits no unnamed product, test,
source-doc, ADR, schema, packaging, or issue change. The frozen `D1`-`D9`
design, `T01`-`T15` matrix, `H01`-`H15` hazards, nine-path write set, execution
routing, and non-goals are unchanged, and no A1 delta contradicts any of them.
The one behavioral consequence of the new base — A1's pre-effect exception-chain
normalization reaching the pre-`M` arms — is correctly identified, correctly
scoped as a recompute-on-`L` obligation rather than a design change, and backed
by an executable invariant and validation.

The six observations in §10 are documentation-precision, hardening, and risk
notes. None is an identity, ancestry, path, dirty-state, JSON/Markdown, or
claimed-digest mismatch; none makes implementation ambiguous or unsafe; and none
is a required correction.

PREFLIGHT_VERDICT: ACCEPT_REBIND_CONTRACT
