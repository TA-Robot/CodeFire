# AIA-046 source-keyed Cargo gate — durable publication of the source `1bbcfde` `ACCEPT`

## Publication metadata (manager-authored; not reviewer text)

This section is written by the portfolio manager who published this artifact.
It is not part of the review. Everything below the horizontal rule is the
complete formal report from the original Claude review response, reproduced
**verbatim, byte-for-byte**, with no paraphrase, silent correction,
abridgement, reordering or added commentary.

Why this file exists: the accepted exact canonical-port review
`docs/development/ai/swarm-results/aia-source-keyed-cargo-gate-canonical-port-rereview-claude-20260802/review.md`
(report SHA-256
`4dc9cd3c5b1cdbfb8938b744ac06730e56d99aafa832a12d7dab70cdf6b51eea`) raised one
manager-owned integration precondition, its observation **O1**: the source
`ACCEPT` for `1bbcfde` existed only as an out-of-tree agent session and was not
locatable from canonical, so canonical could not evidence its own source
authority. This file discharges the publication half of that precondition by
making the verdict durable, with content digests, inside this accepted-chain
candidate repository artifact. It is not canonical yet, and this file does not
make it so: it becomes canonical only after this repair is independently
reviewed and the manager fast-forwards canonical over the whole accepted chain.

### Original review session

| Field | Value |
|---|---|
| Agent | `aia-cargo-final-oracle-rereview-claude-20260802` |
| Session id | `9fac2a66-27ff-4be6-9551-b83e662fd31c` |
| Response timestamp | `2026-08-02T14:40:04.301989+00:00` |
| `is_error` | `false` |
| Reviewer | Claude Opus 5, read-only exact-candidate mode, no worktree, no subagent |
| Terminal verdict | `ACCEPT` |

### Historical capture location (external, read-only, outside canonical)

The session was captured under a gitignored `.claude-second-agent/` agent
directory that is not part of this repository and is not reachable from any
canonical ref. It is recorded here only as the historical provenance of the
bytes, not as a live dependency:

```text
/workspace/worktrees/aia-source-keyed-cargo-gate-base/.claude-second-agent/309658cdb004fc0b743a2ee3c1e3041f34fa1a14523a3abf8ca12716a6a3aff5/worktrees/aia-source-keyed-cargo-gate-closure-claude/.claude-second-agent/29425718ec327306ac4d282b304c5b0e4143891956565a1746eba33b3776d09f/agents/aia-cargo-final-oracle-rereview-claude-20260802/logs/transcript.jsonl
```

That location may disappear. This published file, and the digests below, are
the durable authority from this point forward.

### Content digests (recomputed by the publisher before writing)

| Artifact | SHA-256 |
|---|---|
| Capture `transcript.jsonl` (whole file, 1 JSON line) | `2ec8b17a4f4bf33a2b5c71c5d335e5291a6d61a998f3371bc759b3bbc80acd0a` |
| Sibling `events.jsonl` (same directory, not reproduced here) | `18d89bfb1647a4ce71b3d79aa7819d237ac50857dbb6a28a5a83861b3714603b` |
| Exact response-field bytes, `jq -j .response` | `69bc03a6270e38a960d0e6191186faab2ec7e38c5e4d4d78efeb3620462f58f3` |
| Extracted formal-report body (the verbatim text below the rule) | `7b81cf2ab150d553cbd80a74c151fbaefcb5d90ec3201fd7d7b25da1e04dd9bd` |

The sibling `events.jsonl` is digest-bound only. Raw JSONL and interactive
progress chatter are deliberately not embedded in this repository artifact.

### Reviewed subject identities (recomputed by the publisher)

| Field | Value |
|---|---|
| Source candidate head | `1bbcfde2acc60476b5e29136a30f7c808fb6bb4e` |
| Source candidate tree | `753c44216e0c1145fd97f33d3863c111f49eff2c` |
| Source candidate parent | `74cd37d5e9204c45ac23bf31aa6973d6004cc0f0` |
| Two-path `parent..head` binary patch SHA-256 | `de42c57eaad99307fef9ccb133d262123307aa5519ef53918cf4460dafc145c8` |
| Changed paths | `docs/development/ai/swarm-results/aia-source-keyed-cargo-gate-third-closure-repair-20260731/result.md`, `docs/development/tools/test_ai_tools.py` |
| Terminal verdict | `ACCEPT` |

### Extraction boundary

The `jq -j .response` bytes are 238 lines. Lines 1–54 are the reviewer's
interactive progress narration and are excluded. The published body is exactly
lines **55–238**: it begins at the first line
`# AIA-046 Source-keyed Cargo Gate — Fourth-Closure Candidate, Fresh Independent Exact-Candidate Review (Claude)`
and ends at the final `**Why not BLOCK.**` paragraph, which is also the last
line of the response. That extracted range is 184 lines and 16 130 bytes; the
response's final line carries no terminating newline, so this file appends
exactly one line-feed after it as the file terminator. The digest
`7b81cf2a…1e04dd9bd` above is over the extracted range without that appended
terminator.

### Publisher scope

The publisher recomputed identity and content digests only: the transcript,
events and response digests above; the source head, tree, parent, changed-path
set and two-path binary patch digest; and the byte-equality of the published
body against the extracted range. The publisher **did not** rerun the
reviewer's validation suite, run any product test, or start any Cargo or Rust
process. Every validation figure, measurement and finding in the body is the
original reviewer's, reported as the reviewer wrote it.

Note on numbering: the body's own §4 observation `O1` is the reviewer's
informational NIT about `COUNT_FORMS`. It is unrelated to the canonical-port
review's observation `O1` that this publication discharges.

This publication records the source verdict. It is not an integration record.
The accepted chain is not merged into canonical; see
`docs/development/ai/task-packets/cfr168-source-keyed-cargo-canonical-port-20260731.md`
for the remaining manager actions.

---

# AIA-046 Source-keyed Cargo Gate — Fourth-Closure Candidate, Fresh Independent Exact-Candidate Review (Claude)

Review date: 2026-08-02 · Reviewer: Claude Opus 5 (`claude-opus-5`) · Mode: read-only, exact-candidate, no worktree, no subagent

## 0. Subject identity — recomputed before and after

| Field | Recomputed (before) | Recomputed (after) | Matches brief |
|---|---|---|---|
| Head | `1bbcfde2acc60476b5e29136a30f7c808fb6bb4e` | identical | yes |
| Tree | `753c44216e0c1145fd97f33d3863c111f49eff2c` | identical | yes |
| Parent | `74cd37d5e9204c45ac23bf31aa6973d6004cc0f0` | identical | yes |
| Changed paths | `…third-closure-repair-20260731/result.md`, `docs/development/tools/test_ai_tools.py` | identical | 2 paths |
| Two-path patch SHA-256 | `de42c57eaad99307fef9ccb133d262123307aa5519ef53918cf4460dafc145c8` | identical | **yes** |
| `git status --porcelain` | empty | empty | clean |
| Worktrees created by me | none (`git worktree list \| grep ^/tmp` → 0) | none | — |
| Scratch | `/tmp/rv5scratch` only | removed, `ls` → absent | — |

Subject: `fix(ai): correct the R1 negative-control claim and two closure oracles`. Diffstat `2 files changed, 305 insertions(+), 67 deletions(-)`.

Prior review binding: transcript located at the stated path (repo-relative, under the gitignored `.claude-second-agent/`); `response` field SHA-256 = `28cff83ca8dbbe0717b8b4fd5ba7efaa9c976f4a0f22edfdf1dd0cc593e02b3c` — **matches**. Bound finding set: **F1 (MUST), F2 (SHOULD), F3/F4/F5(a,b)/F6 (NIT)**. No quota targeted.

Read in full: `AGENTS.md`, the complete two-path diff, the updated `result.md` (411 lines) including the new §10, both commit messages, and every implementation region the falsifiers depend on.

## 1. Method

Every implementer figure was treated as a claim and re-measured. All mutation work ran in throwaway non-repository directories (`/tmp/rv5scratch`, plus `git archive` extractions of the candidate and parent trees), or via out-of-tree drivers that subclass the oracle and redirect only `HISTORY` / `CLAIMED` / `COUNT_FORMS`, or monkeypatch the two identity builders in the imported module. No repository file was written at any point. No `git worktree add`. No Cargo, `rustc`, Clippy, `rustfmt`, product CLI suite, Codex, Fugu or subagent.

## 2. Required validation, re-run here

| Command | Result |
|---|---|
| Focused: `HistoryTestCountConsistencyTests` + `SourceKeyedGateDocumentationTests` | **Ran 10 — OK**, 0.689 s |
| `python3 -m unittest test_ai_tools` | **Ran 174 — OK**, 21.1 s, exit 0 |
| `python3 -m py_compile docs/development/tools/*.py` | exit 0 |
| `check_ai_development.py` | exit 0, **PASS ×7** |
| `check_ai_contracts.py` | exit 0 |
| `check_project_docs.py` | exit 0, **608** Markdown files |
| `check_source_docs.py` | exit 0, 44/34/12 |
| `check_ai_acceleration_docs.py` | exit 0 |
| `git diff --check` | exit 0 |

Every §10 validation figure reproduces. One process note on my side, not a candidate defect: `check_ai_acceleration_docs.py` is cwd-relative and exits 1 when invoked from the tools subdirectory; from the repository root it exits 0. Pre-existing behaviour, identical at the parent, out of scope here.

## 3. Re-falsification of the six required items

### Item 1 — exact Japanese historical forms extracted and rejected for the permanent-row reason · **CONFIRMED**

First I verified the pins are authentic rather than plausible. `git show aaa39b6:docs/development/history.md` line 16 contains verbatim `checkpoint時点でtest 148中10件failだったものを157件全passへ。`; `git show 4e5c0e7:…` line 16 contains verbatim `候補は161件全pass。`. Both strings pinned in `test_the_historical_wrong_count_forms_are_recognised` are exact substrings of the real historical rows.

Re-injecting each into a copy of the live row, through an out-of-tree driver:

| Mutation | Candidate extract | Surplus | Candidate result | Parent (`74cd37d`) result |
|---|---|---|---|---|
| live row, unmodified | `{148,159,168}` | ∅ | 4 tests OK | OK |
| `…`aaa39b6`で157件全passへ` | `{148,157,168}` | `{157}` | **2 failures** — set-difference *and* row regex | row regex only |
| live row + `候補は161件全pass。` | `{148,159,161,168}` | `{161}` | **1 failure** — set-difference alone | **`ok` — 161 admitted** |

The parent result in the last row is measured against the parent's own test class with `ROOT` repointed at a working object database, so the `ok` is the guard genuinely admitting `161`, not an incidental `CalledProcessError`. That is the exact defect state F1 named, and it is closed.

The rejection reason is the intended one: `AssertionError: Items in the first set but not the second: {157}` / `{161}`, `admitted [148, 159, 168]` — i.e. the permanent row states a count matching no tree it names, not an incidental regex miss.

Non-vacuity of the new control, by mutating `COUNT_FORMS` only:

| `COUNT_FORMS` variant | Failures |
|---|---|
| as shipped | 0 |
| Japanese alternative dropped (= `74cd37d`) | 1 — `test_the_historical_wrong_count_forms_are_recognised` |
| widened to bare `件` | 2 — the exact-form control **and** the set-difference guard |

Both directions bite, so the `全pass` suffix is genuinely load-bearing exactly as the comment claims, and the Japanese branch cannot be silently dropped.

### Item 2 — `git archive` / fresh non-Git extraction · **CONFIRMED**

`git archive <rev> | tar -x` into `/tmp`, verified not a Git repository (`fatal: not a git repository`, no `.git`).

| | Parent `74cd37d` | Candidate `1bbcfde` |
|---|---|---|
| History class | 7 errors | **Ran 4 — OK (skipped=1)** |
| Full suite | `Ran 173 — FAILED (errors=9)` | `Ran 174 — FAILED (errors=2, skipped=1)` |
| `CalledProcessError` occurrences | 14 | **0** |

The candidate's 2 remaining errors are exactly the two long-standing `git ls-files` provenance artifacts (`RustBuildProvenanceTests.test_source_manifest_is_stable_and_covers_workspace_inputs`, `test_provenance_destination_is_ignored_untracked_target_only`) this lineage has documented since `bfbbc2b`. The non-Git baseline is restored; no new `CalledProcessError` anywhere.

Crucially, the two row guards are **active, not vacuously passing**, inside the extraction — re-injecting the wrong forms there still fails them while re-derivation skips:

```
archive + reinject-161-jp: run=4 FAIL=1 skip=1  [test_history_states_no_test_count_matching_no_tree]
archive + reinject-157-jp: run=4 FAIL=2 skip=1  [+ test_history_states_the_right_count_for_each_named_commit]
```

I also tested the **partial**-reachability case the docstring names (shallow / single-branch clone) — which neither the candidate nor the prior review measured. Making one revision unreachable while drifting another literal:

```
partial: bfbbc2b unreachable                  fail=0 skip=1
partial + drifted literal (4e5c0e7 -> 999)    fail=1 skip=1   AssertionError: 168 != 999
```

`skipTest` inside `subTest` records a per-subtest skip and the loop continues, so a partially-reachable checkout still validates every literal it can reach. The design holds in the intermediate case, not only the two extremes.

### Item 3 — independent key binding · **CONFIRMED, and one cell stronger than the report claims**

Blinding each builder to `readonly_isolation_identity` independently, over the shipped original/shortened/reordered flag sweep:

| Mutation | distinct source | distinct artifact | distinct **pairs** | shipped (independent) | `74cd37d` (pair-uniqueness) |
|---|---|---|---|---|---|
| unmutated | 3 | 3 | 3 | PASS | PASS |
| **artifact-only constant** | 3 | **1** | 3 | **FAIL** `1 != 3 : … does not bind into the artifact key` | **PASS** |
| **source-only constant** | **1** | 3 | 3 | **FAIL** `1 != 3 : … does not bind into the source key` | **PASS** |
| both constant | 1 | 1 | 1 | FAIL | FAIL |

Each key independently binds all three flag orderings (3 distinct keys per key under 3 distinct flag lists), and each one-sided constant fails **its own** assertion with its own message. The old pair-uniqueness assertion passes under *both* single-key blindings — §10/F3 reported only the artifact-side half of that; the mirror is equally damning, so the strengthening is real and slightly better than claimed.

### Item 4 — 607 vs 608 · **CONFIRMED, temporally and factually correct**

Verified independently of the checker, by `git ls-tree -r --name-only <rev> | grep -c '\.md$'`:

| Revision | `.md` count | `…/result.md` present |
|---|---|---|
| `4e5c0e7` | **607** | absent |
| `74cd37d` | **608** | present |
| `1bbcfde` | **608** | present |

So the 607 invocation genuinely predates the result file, the exact reviewed tree `74cd37d` contains it and reports 608, and nothing regressed — one file was added. The live checker on the candidate reports 608. The annotation is exact.

The prior blanket commit-message claim **is** explicitly superseded, in the candidate's own commit message, quoting the superseded sentence and naming the replacement: *"That message said … 'Substituting 157 or 161 fails it on both assertions.' That is true only for the ASCII … spelling … The blanket negative-control claim is superseded by the per-spelling measurements recorded in the result."* `result.md` §4 carries the same correction marked at the point of use in both the oracle and negative-control cells. `74cd37d` is not amended (parent hash unchanged, still reachable, message intact).

### Item 5 — F2/F3 closed; F4–F6 dispositions · **CONFIRMED, no required material residual**

F2 and F3 are closed as measured in items 2 and 3. On the three not-changed dispositions I re-derived the underlying facts rather than accepting the table:

- **F4** — `4e5c0e7` did genuinely correct the "always exact" claim in both the contract doc (`aggregate budget is exact` → `reclaimed figure is … an upper bound`) and the gate payload. So `この表現は`4e5c0e7`で訂正済み` is *imprecise about which surface*, not false; no number is wrong. Residual is prose precision only. Correctly dispositioned.
- **F5(a)** — `grep '`unshare [^`]\+`'` over the contract returns exactly **one** match (line 156). The first-match extractor is still unambiguous; the fragility is latent, not live.
- **F5(b)** — contract document is byte-identical to `74cd37d`, so the readability residual persists exactly as disposed. No false claim.
- **F6** — the dead `if not stat.S_ISDIR(marker.st_mode)` guard is still present and unchanged; the directory case still raises `cleanup-marker-not-regular` upstream. Harmless, zero cost.

None of the three carries a must-fix trigger: no false statement in the permanent record, no oracle blind in the direction it is named for, no reproducibility defect.

### Item 6 — preserved boundaries · **CONFIRMED by digest**

`git rev-parse <rev>:<path>` blob comparison, `74cd37d` vs `1bbcfde`:

| Path | Result |
|---|---|
| `run_source_keyed_cargo_gate.py` | IDENTICAL |
| `readonly_source_exec.py` | IDENTICAL |
| `run_timed_gate.py` | IDENTICAL |
| `build_source_bound_rust.py` | IDENTICAL |
| `ai/source-keyed-cargo-gates.md` | IDENTICAL |
| `history.md` | IDENTICAL |

Per-function AST digest over the test module: **185 functions unchanged**, 5 changed in place (all inside `HistoryTestCountConsistencyTests`), 3 added, 1 removed. The R2 marker-accounting test, the R3 descriptor-complement test, the R4 doc-oracle test `test_documented_unshare_sequence_equals_the_constant` and the R5 forward-guard test are **untouched**. Since every implementation file is byte-identical, the six verified closures, the ten fail-closed classes, the global execution lock and cleanup/resource ordering are unchanged by construction. Nothing in this commit relaxes any check.

Suite arithmetic independently re-derived: 148 / 159 / 168 / 173 / **174** across `bfbbc2b` → `1bbcfde`. Exactly one net test added (the exact-form control); the other add/remove pair is the F3 rename. The `CLAIMED` literals `{148, 159, 168}` are confirmed against the blobs by my own recount, independent of the candidate's.

## 4. Observations

Only one item rises to the level of reporting, and it is informational.

### O1 — NIT (informational, no action) — `COUNT_FORMS` is safe only while it stays non-capturing

**Location:** `docs/development/tools/test_ai_tools.py:2675`, consumed at `:2683` and `:2813`.
**Evidence.** `stated_counts` does `re.findall(rf"(\d+)\s*{cls.COUNT_FORMS}", …)` and indexes group 1 implicitly. `COUNT_FORMS` is `(?:…)`, so `findall` yields plain strings today. A future editor adding a *capturing* alternative would make `findall` return tuples and `int(value)` raise.
**Impact axes.** Maintainability only; product severity nil. It fails **loudly** (`TypeError`, test error) rather than silently admitting a wrong count, so the guard cannot go quietly vacuous by this route.
**Must-fix trigger.** None — no false record claim, no blind oracle, no reproducibility defect.
**Actionability.** Optional one-word comment noting the group must stay non-capturing. Not worth a commit on its own; worth folding into any future edit of this class.

No other observation survived falsification. In particular I probed and rejected: a self-referentiality regression from moving to literals (the re-derivation test re-earns them in-repo, and the literals still never come from the running suite); `int | None` / `set[int]` syntax against the declared floor (`requires-python = ">=3.10"`, and 74 pre-existing `| None` uses in the gate module alone); row-scoping drift (`source_keyed_rows` resolves to exactly 1 lineage row, extracting `{148, 159, 168}`); and Markdown table integrity in the new §10 (0 malformed rows across 411 lines, `git diff --check` clean).

## 5. Verdict

**`ACCEPT`.**

- **Subject:** `1bbcfde2acc60476b5e29136a30f7c808fb6bb4e` — `fix(ai): correct the R1 negative-control claim and two closure oracles`
- **Tree:** `753c44216e0c1145fd97f33d3863c111f49eff2c`
- **Parent:** `74cd37d5e9204c45ac23bf31aa6973d6004cc0f0`
- **Two-path patch SHA-256:** `de42c57eaad99307fef9ccb133d262123307aa5519ef53918cf4460dafc145c8`

**Validation assessment: sound and fully reproduced.** 174/174 green, `py_compile` clean, seven `check_ai_development` components PASS, all five checkers exit 0 from the repository root, `git diff --check` clean, worktree clean before and after, all identity anchors recomputed identical at both ends. Every figure in §10 reproduces here, including the 608 inventory. Scope discipline holds: two admitted paths, no implementation byte moved, no Cargo/Rust process, no product CLI suite, no worktree mutation, no subagent/Codex/Fugu.

**Why ACCEPT.** The bound MUST is closed against the precise recurrence it was raised on: I measured the parent admitting `候補は161件全pass。` and the candidate rejecting it with the intended set-difference reason, using the verbatim historical bytes rather than a paraphrase. F2's closure is stronger than its own account — the row guards stay genuinely *active* in a non-Git extraction, the `CalledProcessError` count goes 14 → 0, the non-Git error baseline returns to the lineage's historical 2, and the partial-reachability case still catches a drifted literal. F3's strengthening is real: each one-sided key blinding defeats the new assertion and would have passed the old one, in both directions. The 607/608 correction is verified independently of the checker that produced it, and the superseded commit-message sentence is quoted and retired explicitly rather than quietly overwritten. F4–F6 were re-derived from source, not accepted from the table, and none carries a must-fix trigger.

**Why not REQUEST_CHANGES.** The single surviving observation is a latent maintainability note on a regex that fails loudly rather than silently, with no path to a false record claim.

**Why not BLOCK.** Product severity is nil across every observation. No fail-closed class is relaxed, no gate is weaker anywhere than at `74cd37d`, and every implementation and contract path is byte-identical to the parent.
