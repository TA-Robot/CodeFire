# CFR-168 C14-LIVE-A2/B3 manager-correction — fresh independent preflight

Date: 2026-08-03

Lane: fresh independent Claude `claude-opus-5` contract reviewer, report-only,
tier `high`. Excluded from the packet-authoring, rebind-authoring, and
manager-adjudication lanes.

Subject: commit `0df71426d36de54d0fc9d5fe0539bce095fa7e8f`
(`docs(cfr168): correct C14 A2 launch contract`).

This report is documentation-only. It changed no product, test, canonical source
document, ADR, schema, packaging, issue, install, or dogfood state, and it did
not edit the corrected packets, the manager adjudication, or any prior report.
Every identity, digest, count, and topology fact below was recomputed from Git
objects and live source. No value was taken from the launch ticket, the packet's
self-report, the historical rebind result, or the historical preflight.

## 1. Verdict summary

Every acceptance-critical predicate named in the launch ticket reproduces
exactly, and every one of the manager's six audit closures `C1`-`C6` withstood an
independent attempt to falsify it. The two original observations that were
factual claims rather than hardening notes — `N1` (AST convention) and `N2` (the
owner-group table) — are now independently confirmed correct on live source, not
merely asserted: the packet's newly normative AST convention reproduces all seven
bound digests, the convention that produced `N1`'s false `BLOCK` demonstrably
does not, and the corrected `class plus eight methods / nine units` row now
matches a full recomputation in which all 49 changed units are accounted for and
zero is A2-owned.

Five observations remain. All five are hardening or traceability notes on
mechanisms that are already fail-closed; none is a required correction, none
makes implementation ambiguous or unsafe, and none was repaired in this lane.
No acceptance-critical predicate is unresolved.

Verdict: accept the corrected rebind contract.

## 2. Environment, exact commands, and results

| Item | Value |
|---|---|
| Python | `Python 3.10.12 (main, Mar  3 2026, 11:56:32) [GCC 11.4.0]` at `/usr/bin/python3.10`, always run as `PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3.10 -B` |
| Git | `git version 2.34.1` |
| Worktree at entry | `git status --porcelain` empty |
| Worktree at finish | clean after committing this one report |

No Cargo, rustc, Rustfmt, Clippy, product Python test, `demo.sh`, install,
dogfood, network workaround, or product/candidate mutation was run. Their
results are **unavailable, not passing**.

| # | Command | Exit | Duration |
|---:|---|---:|---|
| 1 | `git rev-parse HEAD` / `HEAD^{tree}` / `HEAD^`; `git rev-list --parents -n 1 HEAD` | 0 | <0.1 s |
| 2 | `git diff-tree --no-commit-id --name-only -r HEAD` | 0 | <0.1 s |
| 3 | `git diff --full-index --binary HEAD^ HEAD \| sha256sum` | 0 | <0.1 s |
| 4 | `git rev-parse HEAD:<path>` + `sha256sum <path>` for all three launch paths | 0 | <0.1 s |
| 5 | `git cat-file -t`, `git merge-base --is-ancestor P1 HEAD`, `git rev-list --parents P1..HEAD`, `git rev-list --merges P1..HEAD` | 0 | <0.1 s |
| 6 | `git diff --name-only P1 HEAD`; same piped to `sort \| sha256sum` | 0 | <0.1 s |
| 7 | `git rev-parse` + `sha256sum` for the historical result and historical preflight; `tail -1` on the historical preflight | 0 | <0.1 s |
| 8 | six-line allow-list extraction + trailing-newline SHA-256 (Python) | 0 | 0.05 s |
| 9 | strict JSON parse with duplicate-key-rejecting `object_pairs_hook` + canonical-form comparison | 0 | 0.06 s |
| 10 | seven-surface text/AST digest recomputation at `P0` and `P1` (Python `ast`) | 0 | ~9 s |
| 11 | full `codefire` AST-unit classification at `P0` and `P1` (1331/1337 units) | 0 | ~55 s |
| 12 | 144-hunk geometry / A2-span non-intersection analysis | 0 | ~2 s |
| 13 | callee, fence-label, and selector call-line extraction inside `handle_http_upload` at `P1` | 0 | ~4 s |
| 14 | Markdown/JSON digest-symmetry, ordered write-set, disallowed-path, and ID-uniqueness cross-check | 0 | 0.1 s |
| 15 | `bash -n` on all six Markdown shell blocks; fence-balance check on packet and adjudication | 0 | 0.2 s |
| 16 | `git diff --check` (worktree) and `git diff --check HEAD^ HEAD` | 0 | <0.1 s |
| 17 | `PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3.10 -B docs/development/tools/check_ai_development.py --root . --skip-cli --timeout 600 --json` | 0 | 62.712 s |
| 18 | `PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3.10 -B docs/development/tools/check_project_docs.py` (standalone, **after** writing this report) | 0 | 0.713 s |

Command 17 was run **exactly once**, with the explicit sufficient timeout the
manager adjudication mandates. Result: `"ok": true`, and all seven stages
`ok: true` with `returncode: 0` — `acceleration_docs`, `contracts`,
`project_docs` (715 documents inventoried), `source_docs`, `runtime`,
`tool_tests`, `validation_capabilities`. SHA-256 of the captured JSON report:
`26bd4b6f4446b672c8c90f49ec6613f61258432f3433ec57acbc9e20d154c9ec`. The
`project_docs` payload records `cfr` ledger totals `175` with `6 investigating`,
consistent with `cfr-168.md` remaining open. The aggregate's AI tool tests ran;
no CodeFire product test was executed.

Command 18 is disclosed separately and is **not** a second aggregate run: it is
the standalone ordinary-Python documentation checker, re-run once after this
report existed so that a self-inflicted Markdown defect in this very file could
not be left behind. It passes with **716** Markdown files inventoried — the 715
counted inside the aggregate plus this report — and reports no findings.

Commands 10 and 11 read `codefire` from Git objects into `/tmp` (1 506 968 bytes
at `P0`, 1 530 770 bytes at `P1`) rather than from the working tree, so no
tracked path was touched and no bytecode was written.

## 3. Exact launch subject — every identity reproduced

| Claim | Recomputed | Match |
|---|---|:---:|
| Launch commit | `0df71426d36de54d0fc9d5fe0539bce095fa7e8f` | yes |
| Launch tree | `cabb00ee8ee2dd7c4faff61e748cc4b0ddc40f35` | yes |
| Sole parent | `75ba005cbc4e7e96c3173b495f9da676e3b1c8b2` | yes |
| Non-merge | `git rev-list --parents -n 1 HEAD` yields exactly two ids (commit + one parent) | yes |
| Changed path count | exactly 3 | yes |
| Full-index binary patch SHA-256 | `e23ba6fcbe059151e52f9a7048d77a49f31eb8ae2a9b12ba1120bb9e95b8fc6a` | yes |

The three changed paths are exactly, and only:

```text
docs/development/ai/swarm-results/cfr168-c14-live-a2-b3-a1-base-rebind-preflight-claude-20260803/manager-adjudication.md
docs/development/ai/task-packets/cfr168-c14-live-a2-b3-plan-freeze-20260802.md
docs/development/ai/task-packets/json/cfr168-c14-live-a2-b3-plan-freeze-20260802.json
```

| Path | Git blob | Raw SHA-256 | Match |
|---|---|---|:---:|
| corrected Markdown packet | `26408013c06699f8c3d8323e8c8bbb281aa1cb91` | `7693f1881a2ddd29516a8319e8814af5d4fbc795ab3ed1789bb0bc2c3e3680b2` | yes |
| corrected JSON packet | `6875b119185f370d2d705329a6dd320df5f49e6d` | `29a048cc92b5fe034a315a226e278fb5d2bf3ee6cd48c940ad7ec0b7540957b6` | yes |
| manager adjudication | `a75cf572e7aa31aa887402ae63a8e6e2e5e52dd9` | `564463b15a48e1aae94eb22ae123e5035386e7d30800e4782e4c89e893d34b93` | yes |

## 4. `P1..launch` authority chain — exact

`P1` = `1e8dd545f27bd1f22879b067ad0e1b3f4e2ecbc4` exists and is a **strict**
ancestor of the launch commit (`git merge-base --is-ancestor` exits 0; the ids
differ). `P1..launch` is the five-commit linear chain

```text
1e8dd545 -> 615da7ae -> e25695e0 -> 926ba31b -> 75ba005c -> 0df71426
```

Every commit in the range has exactly one parent and
`git rev-list --merges P1..HEAD` is empty (0 merges). The range changes exactly
these five paths and no other:

```text
docs/development/ai/swarm-results/cfr168-c14-live-a2-b3-a1-base-rebind-claude-20260803/result.md
docs/development/ai/swarm-results/cfr168-c14-live-a2-b3-a1-base-rebind-preflight-claude-20260803/manager-adjudication.md
docs/development/ai/swarm-results/cfr168-c14-live-a2-b3-a1-base-rebind-preflight-claude-20260803/preflight.md
docs/development/ai/task-packets/cfr168-c14-live-a2-b3-plan-freeze-20260802.md
docs/development/ai/task-packets/json/cfr168-c14-live-a2-b3-plan-freeze-20260802.json
```

Five-line path digest (`git diff --name-only P1 HEAD | sort | sha256sum`):
`d9067fbe7a17b4b9a59777efeede9bce8de9b01b579ba0fa0e042425a11b8acf` — exact.

| Historical record | Git blob | Raw SHA-256 | Match |
|---|---|---|:---:|
| rebind result `…-a1-base-rebind-claude-20260803/result.md` | `f73bab403e0c7b9d5b1e59b3499bfb16011fde88` | `d0b4619bfe3382673dc98583bc811986d21ace9d6789fbd02a4910b652866d9a` | yes |
| accepted rebind preflight `…-a1-base-rebind-preflight-claude-20260803/preflight.md` | `9900c48258af35236cef1cf313034725ae977d67` | `e5162299592e5128b06d80d86fe438c597a05e02c4f9ba491f2503af92d2e6a8` | yes |

The historical preflight's last line is exactly
`PREFLIGHT_VERDICT: ACCEPT_REBIND_CONTRACT`. Both the corrected packet
(lines 662-664) and the manager adjudication (lines 39-42) state that this
historical ACCEPT is evidence but not execution authorization; this report
concurs and does not treat it as one.

## 5. The six-line canonical future allow-list — independently verified

Extracted verbatim from the heredoc at packet lines 802-807
(`L_MANAGEMENT_PATHS_EXPECTED`), the block contains exactly six lines, is
**already in `LC_ALL=C` sorted order**, and hashes — as six lines joined by `\n`
plus exactly one trailing newline — to

```text
53fdfbef54fb68808eb4707fd9417f553634cb1459fe946589c84011271f6644
```

matching the launch ticket's required value. The trailing-newline convention is
not merely compatible with the packet's own script, it is *forced* by it: the
`$(cat <<'EOF' … EOF)` command substitution at line 801 strips trailing
newlines, and line 812 re-adds exactly one via `printf '%s\n'` before hashing.
The same stripped form is compared against
`git diff --name-only "$P1".."$L" | LC_ALL=C sort` at lines 810-811, so the
actual-versus-expected comparison and the digest comparison operate on
consistent bytes. Line 825 states there is no optional "other management
record"; no eleventh or seventh management path is admitted anywhere.

The rule is demonstrably satisfiable, not merely argued: `P1..launch` is already
five of the six paths, so a single commit adding only the sixth
(`…-manager-correction-preflight-claude-20260803/preflight.md`) — that is, this
report's commit — makes `P1..L` exactly the canonical six. Predicate 9 is a
worktree-cleanliness precondition, satisfied here. I did not invent the future
review commit, tree, or blob; the manager binds them.

## 6. Challenge of the six original observations `N1`-`N6`

### `N1` — AST convention: CLOSED, and independently reproduced

Packet lines 138-143 now make the convention normative and explicitly forbid the
three wrong readings ("Hashing a reparsed `Module` wrapper, a text rendering, or
an AST produced by another Python minor version is not comparable evidence").
`INV-REBIND-DELTA` (JSON line 245) and `VAL-A2-SEGMENTS` (JSON line 353) carry
the identical algorithm, so the Markdown and typed contract cannot diverge.

I executed the stated algorithm from scratch at both `P0` and `P1` — parse the
complete UTF-8 `codefire` source with `ast.parse`, select the unique
`ClassDef`/`FunctionDef` node, `ast.dump(node, annotate_fields=True,
include_attributes=False)`, UTF-8 encode with no added newline, SHA-256 — and
every bound value reproduces:

| Surface | occurrences | text `P0`=`P1`=claim | AST `P0`=`P1`=claim | bytes | `P0` span | `P1` span | shift |
|---|---:|:---:|:---:|---:|---|---|---:|
| `_PreparedHTTPResponseOwner` | 1 | yes | yes | 1748 | 16037-16089 | 16512-16564 | +475 |
| `_PreparedHTTPMutationResponses` | 1 | yes | yes | 5769 | 16092-16243 | 16567-16718 | +475 |
| `_HTTPMutationResponseSlot` | 1 | yes | yes | 2635 | 16256-16325 | 16731-16800 | +475 |
| `handle_http_upload` | 1 | yes | yes | 21581 | 14720-15226 | 15195-15701 | +475 |
| `publish_prepared` | 1 | yes | yes | 1168 | 17221-17245 | 17696-17720 | +475 |
| `handle_one_request` | 1 | yes | yes | 7834 | 17264-17419 | 17739-17894 | +475 |
| `do_POST` | 1 | yes | yes | 11265 | 17635-17867 | 18110-18342 | +475 |

Both endpoints of every span move by exactly `+475`. Each name resolves to
exactly one node at each of `P0` and `P1`, so "the exact bound `ClassDef` or
`FunctionDef` node" is unambiguous in practice for all seven — including the
three methods nested inside `make_http_handler.CodeFireHTTPHandler`, whose spans
begin at the `def` keyword per the `ast.get_source_segment` convention stated at
packet lines 131-133.

The correction is load-bearing, not cosmetic: recomputing
`_PreparedHTTPResponseOwner` under the reparsed-`Module` reading that produced
`N1`'s false `BLOCK` yields a digest different from the bound value. Naming the
convention is therefore exactly what prevents a spurious base violation.

Disposition: `duplicate` of `N1`, now closed. Owner/next action: none.

### `N2` — owner-group table: CLOSED, and factually correct on live source

Packet line 169 now reads
"`AccountedRemoteSignaturePolicyAuthority` and 8 methods | 9 changed".

Full independent recomputation over qualified AST unit names: `codefire` holds
**1331** units at `P0` and **1337** at `P1` — **6 added, 0 removed, 43
AST-changed, 0 text-only-changed**, aggregate **49**. The six added units are
exactly those named at packet lines 167-168. **Zero** of the 49 is A2-owned
under a test of every qualified-name component against the seven A2-owned names.

Every group row is now exact, and the 43 changed units are fully accounted with
no remainder:

| Packet row | Claimed | Recomputed | Match |
|---|---:|---:|:---:|
| new exception helpers (+ `capture`) | 4 added | 4 | yes |
| two `close.attempt` units | 2 added | 2 | yes |
| `AccountedRemoteSignaturePolicyAuthority` and 8 methods | 9 | class + `__init__`, `_open_absolute_directory`, `_open_subject`, `_read_policy`, `_revalidate_policy`, `_revalidate_project`, `close`, `prepare_policy` = 9 | yes |
| `AccountedCommitSignaturePolicy` and 6 methods | 7 | 7 | yes |
| `AccountedServerVerification` and 10 methods, `run_server_verification` | 12 | 11 + 1 = 12 | yes |
| `AccountedHTTPUploadAncestry` and 4 methods, +2 functions | 7 | 5 + 2 = 7 | yes |
| `_AccountedUploadCommitSource` and 2 methods, +2 functions | 5 | 3 + 2 = 5 | yes |
| `SealedCommitValidationSession(.close)`, `validate_sealed_commit` | 3 | 2 + 1 = 3 | yes |
| **Units column sum** | **49** | `4+2+9+7+12+7+5+3 = 49` | yes |

The corrected column now agrees with the packet's own aggregate at line 162-164
and with the JSON evidence ref
`changed-product-units-P0-to-P1:1331-to-1337-units-6-added-0-removed-43-ast-changed-0-text-only`
(JSON line 86). The historical rebind result's table (result.md line 112) retains
the pre-correction `+ 9 methods | 10 changed` row; that file is immutable
historical evidence, was correctly left untouched by the correction commit, and
is not current authority.

Corroborating geometry, recomputed here: the `P0..P1` `codefire` diff has
**144 hunks** (33 of them pure insertions), old-side changed range
**5003-32673**. **No** hunk intersects any A2-owned `P0` span, and none touches
the claimed gap **14720-17867** — checked for both overlapping hunks and
zero-length insertions falling inside a span. Note for the record: `5003` is the
minimum old-side hunk start over all 144 hunks including the leading
`@@ -5003,0 @@` insertion of the A1 helpers; excluding zero-length hunks the
first *modified* old line is 5395. The packet's stated range is correct under the
natural all-hunks reading.

Disposition: `duplicate` of `N2`, now closed. Owner/next action: none.

### `N3` — two A1-touching test modules: CLOSED by `C3`, two-phase and exact

The corrected packet gives both modules mandatory-read status and a *separately
identified* two-phase evidence obligation, without making either a candidate
write path:

- read set, packet lines 653-657: both
  `tests/test_python_http_upload_accounted_traversal.py` and
  `tests/test_python_external_production.py` at `L`, with the explicit statement
  that "both must receive an explicit pre-edit baseline and post-candidate
  focused regression run even though neither is a candidate write path";
- clean-`L` baseline, packet lines 828-840: a dedicated section whose block
  asserts `test "$(git rev-parse HEAD)" = "$L"` and
  `test -z "$(git status --porcelain)"` **before** running the two modules, and
  requires count, exit, duration, output digest, and both test-file blobs
  recorded "separately from post-candidate evidence";
- immutable-`C` regression, packet lines 885-889: the same two modules rerun
  after `test "$(git rev-parse HEAD)" = "$C"`;
- typed contract: `VAL-A1-TOUCHING-BASELINE-L` (JSON lines 366-371) and
  `VAL-A1-TOUCHING-REGRESSION-C` (JSON lines 407-412), the latter requiring
  proof that "both test-file blobs equal the `L` baseline" — which is precisely
  what forbids their becoming candidate paths;
- handoff obligation: JSON line 456 requires separate clean-`L` and frozen-`C`
  records.

Verified independently: both modules exist at `P1`
(`c609e2a84fbcd5f3dca3791cfa106abca369a374` and
`b036f900fdec48b7a83d5bdd97e5b821ac2c6f7c`); neither appears in the ten-path
`write_set`; both appear in the JSON `read_set` (lines 143-144) and
`context.additional_files` (lines 57-58). The two-run design cannot be collapsed
into one, because the two blocks assert different `HEAD` values.

Disposition: `duplicate` of `N3`, now closed. Owner/next action: none.

### `N4` — tenth candidate path: CLOSED, with a concrete truth obligation

`docs/development/known-limitations.md` is candidate path **10** at packet
line 589 and `write_set[9]` in the typed companion (JSON line 156), with "no
optional eleventh path" (line 577). The truth obligation is specific, not
gestural: packet lines 630-632 require it to "distinguish the A2 live
response-slot lifecycle repaired by this candidate from the still-frozen
successor composition, publication-proof, and all-route wiring" and to not
"leave the pre-A2 pair limitation stated as current after the production A2
lifecycle changes". `INV-DOCS` (JSON line 330) now names the file and carries a
matching falsification test ("leave the pre-A2 live pair limitation stated as
current").

I confirmed the obligation has a real subject. At `P1`,
`docs/development/known-limitations.md` line 181 states the current Python
mutation-response pair limitation in the present tense
(「現行のPython mutation response pairは…このため単純接続はFreezeし…」), and the
neighbouring lines separately state that successor production wiring and
publication proof remain `Freeze`/未接続. Both halves of the required distinction
are therefore locatable, and the file genuinely does go stale if the A2 repair
lands — so authorizing it as the tenth path removes a predictable late blocker
rather than widening scope speculatively. It was also already a changed
`P0..P1` path (11th of the 22), so A1 precedent exists for touching it.

Disposition: `duplicate` of `N4`, now closed. Owner/next action: none.

### `N5` — base-rule proof block: CLOSED as to mechanism; residual noted as `O1`/`O2`

The proof block is now machine-asserting rather than prose-comparing. Packet
lines 810-812 assert the sorted actual path set against the packet's own
embedded canonical text and then assert the manager-supplied digest against a
hash of that same embedded text; lines 813-818 assert all six `L`-side Git blobs;
lines 794-800 fail closed when any binding variable is unset; line 819 requires
a clean worktree. The name-only-print-and-eyeball step `N5` objected to is gone.

Two residual hardening gaps survive and are recorded separately as `O1` and `O2`
in §8. Neither is acceptance-critical.

Disposition: `duplicate` of `N5`, closed as to the objection raised; residual
carried forward as `O1`/`O2`.

### `N6` — aggregate timeout: CLOSED and corroborated by this lane's own run

The manager adjudication (lines 34-36) routes `N6` to AIA-052 workload timeout
routing and rules that until that control is accepted, "any applicable aggregate
run uses an explicit sufficient timeout and does not reinterpret the
default-timeout failure as a C14 product defect". That is the correct
disposition, and it is corroborated rather than assumed here: my single run at
`--timeout 600` completed in **62.712 s with exit 0** and all seven stages
`ok: true`, including `tool_tests`. A 60-second-per-check default is simply
below the `tool_tests` stage's runtime on this machine; nothing in the subject
commit touches a tool or test path (§3 shows three documentation paths only).

The corrected packet correctly contains **no** `check_ai_development.py`
validation entry — I grepped both packet files and found zero references — so no
implementer action can be gated on, or misread as, that environmental failure.
The `N6` rule is a lane-hygiene control carried in one of the six mandatory-read
`L` paths, which is the right place for it.

One traceability gap remains, recorded as `O4`: `AIA-052` appears nowhere in the
repository except the manager adjudication itself.

Disposition: `duplicate` of `N6`, now closed; correctly *not* a C14 product
defect. Residual carried forward as `O4`.

## 7. Challenge of the six manager-audit closures `C1`-`C6`

Each closure was treated as a claim to be falsified, per the adjudication's own
instruction at lines 64-65.

**`C1` — preserved design versus superseded rules.** Held. Packet lines 33-41
retain the historical packet as "exact prior evidence", keep `D1`-`D9`,
`T01`-`T15`, hazards and non-goals "in force", and supersede it as *base* and
"only where this corrected packet says so" for five enumerated axes, closing with
"The historical nine-path/no-tenth boundary is not current authority." I searched
for a stale nine-path authority claim and found none: every remaining occurrence
of "nine" in the packet is either explicitly historical/superseded (lines 12, 40)
or a *factual* statement about A1's nine changed `tests/test_python_*.py`
modules (lines 125, 176-177, 719) — which I confirmed is exactly right, since
`git diff --name-only P0 P1` yields 22 paths of which 9 match
`^tests/test_python_`. `known-limitations.md` is the 11th entry in that list, as
line 126 says.

**`C2` — two distinct preflight roles.** Held. `L` predicate 2 (lines 213-214)
binds the *historical accepted rebind preflight* path; predicate 5 (lines
219-222) binds the *fresh correction preflight* path and requires it to end
exactly `PREFLIGHT_VERDICT: ACCEPT_CORRECTED_REBIND_CONTRACT`. The paths differ
and are spelled in full. Read-set lines 659-664 restate the asymmetry
explicitly: "The historical preflight proves the original rebind identities but
does not authorize implementation; only the fresh correction preflight's exact
terminal ACCEPT unlocks manager selection of `L`." Lifecycle stages 0-2 (lines
994-999), `INV-BASE` (JSON line 240), the `disallowed_actions` entry at JSON
line 189, and the `implementation-prohibited-until-…` evidence ref at JSON
line 111 all agree. The exact terminal literal appears twice in the Markdown and
five times in the JSON with no variant spelling.

**`C3` — split A1-touching validations.** Held; see `N3` above. The two runs are
bound to different `HEAD` values and different recording obligations, so they
cannot be conflated, and blob equality proves neither module was written.

**`C4` — typed context/read contract.** Held. The JSON `read_set` (28 entries)
and `context.additional_files` (27 entries) both carry all four management
authority records — rebind result, historical preflight, manager adjudication,
and fresh correction preflight (JSON lines 124-127 and 38-41) — plus both
A1-touching test modules and `docs/development/known-limitations.md`. The eight
already-existing candidate paths all appear in `read_set`; the two create-paths
(`tests/test_python_http_post_effect_response.py`,
`adr/ADR-021-python-http-mutation-response-lifecycle.md`) correctly do not,
and I confirmed both are absent at `P1`.

**`C5` — canonical allow-list replaces the digest-only assertion.** Held; see §5.
Sorted, six lines, self-consistent hashing convention, digest
`53fdfbef…f6644` reproduced, six blob bindings, and an explicit prohibition on
optional management paths.

**`C6` — the validate-before-commit cycle is removed.** Held. Packet lines
842-853 order it unambiguously: "After the clean-`L` baseline and all
implementation edits are complete, the implementer must first verify the proposed
exact ten-path diff and then create one clean, non-merge candidate commit `C`…
This commit happens before any candidate-scoped positive gate or reversal is
relied upon. All remaining validation runs against those immutable committed
bytes." Repair policy is explicit ("Do not amend, squash, rebase, merge, or stack
a repair onto `C`. If any gate requires a byte change, reject that candidate and
construct a new one-commit candidate from `L`; then rerun every candidate-scoped
gate"), reversals are disposable-copy-only (lines 896-906: "Never apply a
reversal to `C`"), and the manager Candidate Freeze is record-only after
validation (lines 923-945: "This record does not create, amend, squash, rebase,
merge, or add evidence to `C`"; "Any byte change after freeze creates a new
candidate with a new identity. It does not amend `C`"). `INV-CANDIDATE-ORDER`
(JSON lines 334-337) encodes the same order with a falsification test that fires
if "a candidate gate runs on uncommitted or dirty bytes" or "`C` is committed
only after its claimed validation". I found no residual cycle: no gate in the
Markdown or the typed validation list requires evidence about `C` before `C`
exists.

## 8. Observations

Five observations. Each is stated with exact evidence, impact, confidence,
must-fix trigger, one canonical disposition, owner, and next action. None is a
required correction; none was repaired in this lane. Ordered by usefulness to the
manager.

### `O1` — three of the six management blob assertions bind only the manager's own record, not the accepted bytes

- **Evidence.** Packet lines 794-800 require six blob variables to be non-empty;
  lines 813-818 compare each `L`-side blob to the corresponding
  manager-supplied variable. The packet pins no literal blob value, and I
  confirmed by grep that none of `f73bab403e0c7b9d5b1e59b3499bfb16011fde88`,
  `9900c48258af35236cef1cf313034725ae977d67`, or
  `a75cf572e7aa31aa887402ae63a8e6e2e5e52dd9` appears in either packet file.
- **Impact.** Three of the six blobs are already immutable and knowable at
  correction time — the rebind result and historical preflight are from earlier
  commits, and the manager adjudication is a sibling file in the launch commit —
  so they *could* have been pinned literally. As written, an `L` carrying
  different bytes at those three paths would still pass the block, provided the
  manager records the matching blobs. The fourth (correction preflight) and the
  two packet blobs genuinely cannot be pre-pinned; the launch ticket forbids
  inventing them.
- **Why it is nonetheless fail-closed.** The path-set binding is *not* vacuous:
  line 811 compares the actual sorted `P1..L` set against text embedded in the
  packet, and line 812 hashes that same embedded text, so the digest cannot be
  satisfied by a manager record that disagrees with the packet. Predicate 8
  forbids any product/test/source-doc/ADR/schema/packaging/issue difference from
  `P1`, and the six-path allow-list admits nothing else, so the residual freedom
  is confined to management-record bytes. The instructions being executed are
  themselves read from `L`, and a historical-packet `L` would fail line 811
  outright.
- **Confidence.** High that the mechanism gap exists; low that it produces harm.
- **Must-fix trigger.** Not met. No frozen invariant is violated and no
  reproduced failure exists; `INV-BASE`'s falsification test still fires on a
  path, digest, topology, or dirty-state mismatch.
- **Disposition.** `nonblocking_improvement`.
- **Causal class.** Not required (not a `required_correction`).
- **Owner / next action.** Manager: optionally pin the three known literals in
  the launch record or a future packet revision. No implementer action.

### `O2` — the correction preflight's exact terminal line is required but not machine-asserted

- **Evidence.** `L` predicate 5 (packet lines 219-222) requires the report to end
  exactly `PREFLIGHT_VERDICT: ACCEPT_CORRECTED_REBIND_CONTRACT`, and
  `VAL-BASE-P1` (JSON line 347) repeats "require the fresh report terminal
  ACCEPT_CORRECTED_REBIND_CONTRACT". The proof block asserts only that path's
  blob (line 818); no line inspects the file's last line, though a one-line
  `tail -n 1` assertion was available.
- **Impact.** The unlock gate rests on manager diligence rather than on the
  script the implementer runs. A `BLOCK`-terminated report at the right path
  would satisfy the shell block.
- **Confidence.** High (direct reading of the block).
- **Must-fix trigger.** Not met. The predicate is stated in prose, in the typed
  validation, in `INV-BASE`, and in the lifecycle order; the manager, not the
  implementer, owns naming `L`, and `INV-BASE`'s falsification test explicitly
  lists "non-ACCEPT correction terminal".
- **Disposition.** `nonblocking_improvement`.
- **Owner / next action.** Manager: add the terminal assertion to the launch
  record, or verify it during integration as the lifecycle already requires.

### `O3` — the broad-gate block omits the `HEAD == C` guard the focused block carries

- **Evidence.** Packet lines 910-916 (`check_source_docs.py`,
  `check_project_docs.py`, `unittest discover`, `./demo.sh`,
  `git diff --check "$L"..HEAD`) contain no
  `test "$(git rev-parse HEAD)" = "$C"`, whereas the focused block asserts it at
  line 885 before rerunning the A1-touching modules.
- **Impact.** Cosmetic asymmetry only. `VAL-SOURCE-DOCS`, `VAL-PROJECT-DOCS`, and
  `VAL-FULL-PYTHON` each say "On clean committed `C`" (JSON lines 421, 428, 435),
  `INV-CANDIDATE-ORDER` binds every candidate-scoped gate to the immutable
  commit, and line 915's `git diff --check "$L"..HEAD` presumes `HEAD` is the
  candidate.
- **Confidence.** High.
- **Must-fix trigger.** Not met.
- **Disposition.** `nonblocking_improvement`.
- **Owner / next action.** Manager, at its discretion; no implementer action
  beyond obeying the typed validation entries.

### `O4` — `AIA-052` is referenced only by the manager adjudication and exists nowhere else

- **Evidence.** Manager adjudication line 34 routes `N6` to "AIA-052 workload
  timeout routing". A repository-wide grep over `*.md` and `*.json` returns
  exactly one file — that adjudication itself.
- **Impact.** Traceability only. The operative rule in the same sentence
  ("uses an explicit sufficient timeout and does not reinterpret the
  default-timeout failure as a C14 product defect") is self-contained and
  executable without the ticket existing, and the corrected packet contains no
  aggregate validation entry, so no implementer action depends on `AIA-052`. My
  own run (§2, command 17) confirms the environmental reading.
- **Confidence.** High.
- **Must-fix trigger.** Not met.
- **Disposition.** `nonblocking_improvement`.
- **Owner / next action.** Manager: create or route `AIA-052`, or drop the
  identifier in favour of the inline rule.

### `O5` — lazy Markdown continuation in the read-set bullet

- **Evidence.** Packet lines 656-657 lose the two-space continuation indent
  inside the `tests/test_python_external_production.py` list item
  ("post-candidate focused regression run even though neither is a candidate" /
  "write path.").
- **Impact.** None on rendering — CommonMark lazy continuation keeps both lines
  in the same list-item paragraph — and none on tooling: `check_project_docs`
  passes inside the aggregate with 715 documents inventoried, and
  `git diff --check HEAD^ HEAD` is clean.
- **Confidence.** High.
- **Must-fix trigger.** Not met.
- **Disposition.** `nonblocking_improvement`.
- **Owner / next action.** Manager, at its discretion. No implementer action.

## 9. The ten required decisions — adjudicated

| # | Required decision | Verdict | Primary evidence |
|---:|---|---|---|
| 1 | Python 3.10.12 exact AST-node/dump/encoding digest convention | unambiguous and executable | packet 138-143; `INV-REBIND-DELTA` JSON 245; `VAL-A2-SEGMENTS` JSON 353; all 7 digests reproduced §6/`N1` |
| 2 | class-plus-eight-methods / nine-unit row and 49 aggregate | factually correct | packet 169, 162-164; full 1331→1337 recomputation §6/`N2`; column sum 49 = 6 added + 43 changed |
| 3 | separate clean-`L` baseline and immutable-`C` regression for both A1-touching modules, neither a candidate path | unambiguous | packet 653-657, 828-840, 885-889; `VAL-A1-TOUCHING-BASELINE-L`, `VAL-A1-TOUCHING-REGRESSION-C`; blob-equality proof |
| 4 | exact ten-path candidate set incl. `known-limitations.md` and its live-repair vs successor-Freeze truth obligation | unambiguous, with a real subject | packet 575-589, 630-632; `INV-DOCS`; `known-limitations.md` line 181 at `P1` |
| 5 | historical vs fresh preflight roles and exact paths; only this report's exact ACCEPT unlocks `L` | unambiguous | packet 213-222, 659-664, 994-999; `INV-BASE`; JSON 111, 189 |
| 6 | canonical sorted six-line `P1..L` allow-list, trailing-newline digest, six blob bindings, no optional management path | verified; digest `53fdfbef…f6644` | packet 801-826; independent recomputation §5; residual `O1`/`O2` |
| 7 | clean-`L` before edits, one immutable single-commit `C` before candidate gates, disposable-only reversals, new candidate on byte change, record-only Freeze | no residual cycle | packet 828-840, 842-853, 896-906, 923-945; `INV-CANDIDATE-ORDER` |
| 8 | Markdown/JSON/context/read/write/non-goal/handoff/lifecycle consistency | consistent | §10 below |
| 9 | exact ten-path scope, no stale nine-path authority, no product or issue closure claim | satisfied | packet 40, 575-589, 628, 746-777, 1018-1033; `project_docs` shows `cfr-168` still open |
| 10 | `N6` explicit sufficient aggregate timeout pending AIA-052, not a false C14 defect | correct, and corroborated | adjudication 34-36; aggregate exit 0 in 62.712 s at `--timeout 600`; zero aggregate entries in the packet; residual `O4` |

## 10. Markdown / JSON / lifecycle consistency — recomputed

| Check | Result |
|---|---|
| strict JSON parse with duplicate-key-rejecting hook | passes; **no duplicate keys** at any level |
| canonical formatting | byte-identical to `json.dumps(obj, indent=2, ensure_ascii=False) + "\n"` |
| 64-hex digest symmetry | 22 in Markdown, 22 in JSON, **zero asymmetry in either direction** |
| 40-hex id containment | all 13 JSON ids appear in the Markdown; the Markdown carries 29 (a superset, not a mismatch) |
| ten-path write set | Markdown list (lines 580-589, items 1-10) equals JSON `write_set` **as an ordered list**, length 10 |
| `write_set` ∩ `disallowed_paths` | empty; and no write path lies under a disallowed prefix |
| invariant / validation ids | 21 invariants and 15 validations, all ids unique; every invariant has a `falsification_test`; every validation has `command`, `purpose`, `required` |
| new typed entries | `INV-CANDIDATE-ORDER` added (20→21); `VAL-A1-TOUCHING-BASELINE-L` and `VAL-A1-TOUCHING-REGRESSION-C` added (13→15); 52 evidence refs (was 51) |
| `base_revision` | `1e8dd545f27bd1f22879b067ad0e1b3f4e2ecbc4` = `P1`; `packet_id`, `schema`, `role` unchanged; `status` `ready` |
| Markdown shell blocks | 6 blocks, all `bash -n` exit 0; 22 fences, balanced |
| `git diff --check` | clean on the worktree and on `HEAD^..HEAD` |
| lifecycle order | stages 0-12 (packet 989-1011) place the fresh correction preflight at stage 1 and manager naming of `L` at stage 2, ahead of the clean-`L` baseline (3), candidate (4), Freeze (5), fresh review (6); consistent with `L` predicates 1-9 and with `INV-CANDIDATE-ORDER` |
| handoff | packet 1035-1090 separates packet-author, rebind-author, and implementer handoffs; JSON `handoff.required_outputs` (15 entries) covers the six-path allow-list, both preflight roles, the exact ACCEPT terminal, separate `L`/`C` A1-touching records, and the negative-space statement |
| callee / fence coordinates at `P1` | `validate_sealed_commit` 15309, `enforce_remote_commit_signature_policy_accounted` 15364, `run_server_verification` 15374, `is_ancestor_from_objects_accounted` 15481; `fence_first_effect` at 15401 / 15455 / 15620 with labels `"upload final subject fence"`, `"upload layout bootstrap"`, `"upload publication effect"`; `select_success` 15657/15666, `select_durable` 15668 — all exact, and `handle_http_upload` spans 15195-15701 |

The JSON `disallowed_paths` list includes `docs/development/ai/swarm-results`,
where this report is written. That is not a contradiction: the list bounds
candidate `C`, and the packet independently requires preflight and review reports
to live outside `C` (lines 592-595, 959-960).

## 11. Preserved negative space

This report is **only** an independent contract preflight of commit
`0df71426d36de54d0fc9d5fe0539bce095fa7e8f`. It is explicitly **not**, and does
not imply, any of the following:

- A2/B3 implementation, or any product, test, ADR, source-doc, schema,
  packaging, install, dogfood, or issue-state change;
- a Candidate Freeze, candidate review, or source acceptance;
- integration of the correction or of A2/B3 into the mainline — the manager owns
  final integration;
- naming the execution base `L`, which remains a manager act, or inventing the
  future review commit, tree, or blob;
- Install, Dogfood/Evaluate, Retrospect, or final Close of CFR-168, CFB-136,
  CFB-138, global C14, MR-write, or the active batch; or
- a reopening, re-adjudication, or widening of the accepted `C14-LIVE-A1` source
  package, which is treated here as settled authority and examined only as the
  base of this correction.

A2/B3 itself remains open. No observation in §8 was repaired in this lane, and
none is a required-correction trigger. The historical
`PREFLIGHT_VERDICT: ACCEPT_REBIND_CONTRACT` is treated as evidence only and is
not read as authorizing implementation.

## 12. Unavailable evidence and limitations

- No Cargo, rustc, Rustfmt, Clippy, product Python test, `demo.sh`, install,
  dogfood, or network work was run, by ticket prohibition. Those results are
  unavailable, not passing. `T01`-`T15` were therefore reviewed as a **contract
  only**; this report makes no claim about their runtime outcome, which the
  packet correctly assigns to the implementer on `L`.
- `docs/development/tools/test_ai_tools.py` executed once, as a stage of the
  single allowed `check_ai_development.py` aggregate. It is a tools suite, not a
  CodeFire product test.
- `L` does not yet exist. Every `L`-bound predicate remains an obligation
  verified for *satisfiability and fail-closedness* (§5), not an observation.
  This report's own commit, tree, and blob are not self-bindable; the manager
  binds them.
- The seven segment digests, the 1331→1337 unit classification, the 144-hunk
  geometry, and the callee/fence coordinates were verified at `P0` and `P1`. The
  packet's obligation to re-derive them on `L` remains the implementer's and is
  correctly stated as a current-source obligation.
- `AIA-052` could not be located in the repository (`O4`), so the downstream
  timeout-routing control was reviewed only as described in the adjudication.
- The runtime behavior of A1's four changed callees was not executed; the
  no-contradiction reading rests on the static analysis above plus the helpers'
  construction, which is exactly what `VAL-BASELINE-RECOMPUTE` defers to `L`.

## 13. Conclusion

Every acceptance-critical predicate reproduces exactly from Git objects and live
source: the launch commit, tree, sole parent, non-merge status, three-path write
set, full-index patch digest `e23ba6fc…8fc6a`, and all three final blob/raw
digest pairs; `P1` strict ancestry, five-commit linear non-merge `P1..launch`,
the exact five changed paths and their digest `d9067fbe…b8acf`; both historical
management blobs and the historical `ACCEPT_REBIND_CONTRACT` terminal; and the
canonical sorted six-line future allow-list hashing to `53fdfbef…f6644` with a
hashing convention forced by the packet's own script.

The two substantive original observations are now correct as *facts*, not merely
addressed as text: the normative AST convention reproduces all seven bound
digests while the reading that generated `N1`'s false `BLOCK` provably does not,
and the corrected owner-group row matches a complete recomputation in which
6 added + 43 AST-changed = 49 units are fully accounted for, all A1-owned, none
A2-owned, with no diff hunk intersecting any A2-owned span. `N3`-`N6` and all six
manager closures `C1`-`C6` withstood independent falsification attempts. Scope is
single-valued at exactly ten candidate paths with the historical nine-path
boundary explicitly demoted, the two preflight roles are distinct and exactly
pathed, the candidate ordering cycle is gone, Markdown and JSON agree with zero
digest asymmetry and no duplicate keys, and no product, issue, or lifecycle
closure is claimed anywhere.

The five remaining observations are hardening and traceability notes on
mechanisms that already fail closed. None is an identity, ancestry, path,
dirty-state, JSON/Markdown, digest, ordering, or scope mismatch; none makes
implementation ambiguous or unsafe; and none is a required correction. There is
no unresolved acceptance-critical predicate, so no `BLOCK` condition arose.

PREFLIGHT_VERDICT: ACCEPT_CORRECTED_REBIND_CONTRACT
