# AIA-046 Source-keyed Cargo Gate — Third Closure Repair (Claude)

Date: 2026-07-31

Implementer: Claude Opus 5 (`claude-opus-5`), original-session repair
implementer, session `f7692f47-4af1-4f84-a186-d57ddd4b083e`

Batch: `aia-source-keyed-cargo-gate-third-closure-repair-20260731`

**This candidate is not accepted until a fresh independent report returns
`ACCEPT`.** Nothing below claims acceptance.

> **Read §10 before §4.** The commit this report described, `74cd37d`, was
> reviewed and returned `REQUEST_CHANGES`. §10 is an in-place correction
> record added on 2026-08-02; three claims in §4 and one command count in
> §6 are superseded there and are marked at the point of use.

## 1. Recomputed identities

| Field | Recomputed | Match |
|---|---|---|
| Reviewed candidate | `git rev-parse HEAD` → `4e5c0e7f058cd406cb7a183ecda8b3a07f3ee912` | yes |
| Candidate tree | `git rev-parse HEAD^{tree}` → `6417400cb7b5a3507f85f38d66c0573992ad4007` | yes |
| Predecessor is ancestor | `git merge-base --is-ancestor aaa39b6 HEAD` → true | yes |
| Checkpoint is ancestor | `git merge-base --is-ancestor bfbbc2b HEAD` → true | yes |
| Review commit | `130878d321f6be166f554ac2c68a59e2b36afa88` (parent `4e5c0e7`) | yes |
| Review SHA-256 | `git show 130878d:…/review.md \| sha256sum` → `aad067b2b8feb80830c41db1608dd31f0e7e7d7ca1f37ab01d040336f788b4fd` | yes |
| Worktree at start | `git status --porcelain` empty | yes |

The review report was read in full (691 lines) and was not edited, replaced
or amended.

## 2. Causal classification — confirmed, with one qualification

The brief's working classification is
`BOUND_FINDING_NOT_CLOSED + REPAIR_INTRODUCED_REGRESSION` with a missing
claim-to-oracle verification step. **Confirmed** against exact bytes:

```text
$ git show 4e5c0e7:docs/development/history.md | sed -n 16p | grep -o '候補は161件全pass'
候補は161件全pass

$ git log -1 --format=%B aaa39b6 | grep -n '157'
   history.md said 157 for a 159-test candidate; corrected.   # ← in 4e5c0e7's message
```

Ground truth re-derived here (section 4, R1): `aaa39b6` defines 159 tests,
`4e5c0e7` defines 168. `161` is neither. The bound finding `SHOULD-2` was
therefore not closed, a new wrong value was introduced by its own fix, and
two documents asserted a closure the bytes did not contain.

It is **not** an initial-review omission: the prior report named the defect
precisely. It is **not** primarily communication ambiguity: the required
value was unambiguous.

**Qualification, offered as evidence rather than as a defence.** The missing
step is narrower than "no verification". The `157 → 159` reasoning was
correct and the checkpoint half was independently re-derived and corrected.
What was absent is that the number written into the file was never read back
and compared with the number the reasoning produced — the edit landed while
the suite was mid-repair at 161 tests, and no oracle covered the permanent
record. That is exactly the gap R1's new control closes, and it is why the
control derives its expected value from fixed historical blobs rather than
from the running suite. Per the brief, this single episode is not
generalized into a project-wide root cause.

## 3. Changed paths and per-path purpose

| Path | Purpose |
|---|---|
| `docs/development/history.md` | R1: bind the row to `aaa39b6`/159 and `4e5c0e7`/168; correct a stale "always exact" reclaim claim in the same cell |
| `docs/development/tools/run_source_keyed_cargo_gate.py` | R2: marker branch consults `seen_multilink_inodes` |
| `docs/development/ai/source-keyed-cargo-gates.md` | R3: step 9 survivor clause bounded to the declared `--pass-fds` |
| `docs/development/tools/test_ai_tools.py` | R1/R2/R3/R4/R5 oracles: history consistency, marker overcount, survivor wording, unshare sequence equality + key binding, forward-guard rename |
| `…/aia-source-keyed-cargo-gate-second-closure-repair-20260731/result.md` | R1/R5: correct the two false claims, in place, marked as corrections |
| `…/aia-source-keyed-cargo-gate-third-closure-repair-20260731/result.md` | this report |

All six are inside the allowed write set. `readonly_source_exec.py` was
**not** modified — verified by `git diff --name-only`.

## 4. Finding table

### R1 — bound finding re-created: `history.md` states a count matching no tree

| Field | Content |
|---|---|
| identity | R1 / review `MUST-1` (`130878d`); base `4e5c0e7`; `docs/development/history.md:16`, `docs/development/tools/test_ai_tools.py`, prior `result.md` |
| prior property | The permanent record must state the test counts of the commits it describes. |
| pre-edit observation | Four observations, all reproduced. `git worktree add --detach /tmp/r3wt/ckpt bfbbc2b` → `Ran 148 tests — FAILED (failures=4, errors=6)` = **10**. `git archive bfbbc2b \| tar -x` → `Ran 148 tests — FAILED (failures=4, errors=8)` = **12**, the two extras being `RustBuildProvenanceTests.test_source_manifest_is_stable_and_covers_workspace_inputs` and `test_provenance_destination_is_ignored_untracked_target_only`, both `fatal: not a git repository` from `git ls-files`. `aaa39b6` worktree → `Ran 159 tests — OK`. `4e5c0e7` → `Ran 168 tests — OK`. Committed row said `161`. **Reproduces.** |
| causal attribution | `BOUND_FINDING_NOT_CLOSED` + `REPAIR_INTRODUCED_REGRESSION`, with an oracle gap. Section 2. |
| repair | Row now reads `本行が記述する候補は`aaa39b6`で159 test全pass。後続の`4e5c0e7`は168 test全pass。` — each count bound to the commit it describes. Smallest sufficient boundary: the row's subject is the six-MUST closure, i.e. `aaa39b6`, so binding + `161→159` is the minimum; `4e5c0e7`/168 is added because the row now names that commit. The new HEAD's own count is deliberately **not** published, per the brief. |
| oracle | **Corrected 2026-08-02 — see §10/F1 and F2.** As committed at `74cd37d` this cell read: "`HistoryTestCountConsistencyTests`, 3 tests. Expected values come from `git show <commit>:…/test_ai_tools.py` parsed with `ast` — the *named commit's own blob*, never the running suite … Validated against runtime: static 148/159/168 == measured 148/159/168." Two parts of that were wrong. (a) The count extractor recognised only the ASCII `<N> test` spelling, and **both** wrong counts were written in the Japanese `<N>件全pass` spelling, so the guard was blind to the exact recurrence it is named for. (b) Deriving the expected value from `git show` inside the *row* assertions made the permanent record's guard depend on the historical objects being reachable. The class is now 4 tests: the two row assertions compare against literal `CLAIMED` counts and recognise both spellings; a separate re-derivation test re-earns those literals from each named commit's own blob via `git show` + `ast` and is the only test permitted to skip. Scope is unchanged: history rows naming a lineage commit; other ledger rows (product CLI, algorithm lab) are out of scope by construction. |
| negative control | **Corrected 2026-08-02 — see §10/F1.** As committed at `74cd37d` this cell claimed: "Substituting `157`, then `161`, for the row's `159`: both fail, on both assertions." That claim holds only for the ASCII spelling actually exercised. Re-measured against the exact historical bytes: with the row rewritten to `aaa39b6`で157件全passへ (aaa39b6's own wording) the set-difference assertion fails with `Items in the first set but not the second: 157 … admitted [148, 159, 168]` and the row-count regex fails too; appending `候補は161件全pass。` (4e5c0e7's own wording) beside the correct `159 test全pass` fails the set-difference assertion **alone**, with `161`, isolating the intended reason. Against the `74cd37d` ASCII-only extractor that second mutation **passed**. Both restored → `OK`. |
| remaining boundary | Closed for the ASCII spelling at `74cd37d`; the Japanese spelling was closed on 2026-08-02 (§10/F1) and the reachability dependency on 2026-08-02 (§10/F2). |

Also corrected in the same cell, and reported rather than done silently: the
row still asserted `reclaimed bytesはremoval walk自身で実測して常にexactにした`
("always exact"), the pre-`MUST-3` claim that `4e5c0e7` corrected in the doc
and payload but not here. Left as-is, the permanent record would contradict
the accounting contract. This is a factual correction inside the cell R1
already repairs, not a scope expansion into another file.

### R2 — marker hard-link overcount reported as zero

| Field | Content |
|---|---|
| identity | R2 / review `NIT-1`; base `4e5c0e7`; `run_source_keyed_cargo_gate.py` marker branch and payload semantics |
| prior property | `reclaimed_allocated_bytes_counted_more_than_once` is "exact for links wholly inside the namespace". |
| pre-edit observation | `/tmp/probe_marker.py`, one hard link to `.codefire-source-key.json` inside the namespace root: `du` 8192, aggregate scan 12288, removal walk 12288, `removed_hardlinked_entry_count` **1**, `counted_more_than_once` **0**, `walk − dup == du` **False**. **Reproduces.** |
| causal attribution | Repair-introduced (`4e5c0e7` created both the field and its exactness claim). The marker branch read `st_nlink` directly; by the time it runs the walk has unlinked the sibling, so the count has already fallen to 1. |
| repair | The marker branch now applies the walk's own `(st_dev, st_ino)` repeat test against `seen_multilink_inodes` before falling back to `st_nlink`. Smallest sufficient boundary: three lines mirroring logic that already exists eleven lines above; the per-entry convention, the 32 GiB budget and every removal decision are untouched. |
| oracle | `test_hardlinked_marker_overcount_is_reported`: asserts the arithmetic property `walk − counted_more_than_once == du`, plus `hardlinked == 2` and `walk > du`. Expected value from `du -s --block-size=1`, which de-duplicates by inode — independent of the implementation. Not a symbol-name assertion. |
| negative control | Reverting only the marker branch to `4e5c0e7`'s `st_nlink`-only form: `AssertionError: 1 != 2`. Restored → `OK`. Post-fix probe: `du` 8192, walk 12288, hardlinked 2, overcount 4096, `walk − dup == du` **True**. |
| remaining boundary | **Closed**, with a reachability qualification below. |

**Reachability is narrower than the review's failure scenario, and this is
decision-relevant.** The review's scenario is a build script linking the
marker, then "the next cleanup". On the primary path that cleanup is already
fail-closed: `cleanup_namespace_locked` calls `validate_marker_at`, whose
`read_regular_at` rejects `st_nlink != 1` with
`… must be a single-link regular file`. I hit that first and had to look
further. The reachable surface is `recover_incomplete_namespaces`
(`run_source_keyed_cargo_gate.py:5495`–`:5507`), which passes
`expected_marker=None` precisely when the marker could not be read — which
is what a hard-linked marker is — so removal proceeds without validation and
reaches the marker branch. The finding is real and now fixed; the exposure
is one recovery path, not every cleanup.

### R3 — step 9's survivor sentence was unconditional

| Field | Content |
|---|---|
| identity | R3 / review `NIT-3`; base `4e5c0e7`; `source-keyed-cargo-gates.md:184` |
| prior property | "What survives is stdio, the primary tool, and the descriptors the child environment itself names." |
| pre-edit observation | Read against `readonly_source_exec.py`: the close set is the complement over the **parsed `--pass-fds` string**, so a descriptor the kernel inherits but the string omits survives. The review reproduced this with `PROBE_DECLARE_ALL=0`. Confirmed by reading; not independently re-probed, because the fix is to the sentence and the mechanism is unchanged. **Reproduces (by construction).** |
| causal attribution | Repair-introduced claim-vs-implementation; the leak class is currently unreachable in the gate. |
| repair | The clause is now "Of the descriptors declared in `--pass-fds`, what survives is …", followed by why the qualifier is load-bearing, why the gate cannot reach the divergence today (both the `--pass-fds` string and the kernel `pass_fds=` argument come from the same `pinned["pass_fds"]` tuple), and the note that the in-child control is stronger than the mechanism. Smallest sufficient boundary: wording only. The implemented complement closure is unchanged and no filesystem-sandbox claim is made. |
| oracle | Two added assertions in `test_descriptor_closure_is_described_as_a_complement`, over whitespace-collapsed contract text. |
| negative control | The prior sentence lacks both required strings, so restoring it fails both assertions. The pre-existing assertion that the *false* invariant is absent (`assertNotIn`) still guards the original regression. |
| remaining boundary | **Closed.** |

### R4 — the unshare documentation oracle could not fail in MUST-2's direction

| Field | Content |
|---|---|
| identity | R4 / review `NIT-4`; base `4e5c0e7`; `test_ai_tools.py` |
| prior property | `assertIn(" ".join(("unshare", *UNSHARE_ISOLATION_FLAGS)), contract)` — containment. |
| pre-edit observation | Shortened `UNSHARE_ISOLATION_FLAGS` to its pre-fix three elements — MUST-2's exact failure shape — and ran the oracle: **`OK`**. The doc spells five flags, so it still contains the three-flag join. **Reproduces.** |
| causal attribution | Repair-introduced oracle gap: the assertion's direction is orthogonal to the defect it is named for. |
| repair | Replaced with `test_documented_unshare_sequence_equals_the_constant` (extracts the documented token sequence with `re.search(r"`unshare ([^`]+)`")` and asserts **sequence equality**, fixing contents and order, plus `--propagation`/`private` presence and adjacency so shortening doc and constant together still fails) and a second test that asserts the recorded identity equals the constant and then mutates only the flag list — full, pre-fix three, reversed. **Corrected 2026-08-02 — see §10/F3.** At `74cd37d` that second test was named `test_unshare_sequence_binds_identity_argv_and_both_keys` and required the three `(source_key, artifact_key)` **pairs** to be distinct. Both were overstated: the test observes no launch, so it makes no argv claim, and a pair is already distinct when only one member moves, so a key that stopped binding the sequence entirely would still have passed. It is now `test_unshare_sequence_binds_identity_and_both_keys` and projects the two keys **independently**, asserting `len(set(source_keys)) == 3` and `len(set(artifact_keys)) == 3` as separate assertions. |
| oracle | Expected values: the doc's own extracted tokens, and per-key inequality under mutation. Scope: doc + constant + identity + each key separately. Launch argv and published result are asserted in `test_gate_uses_locked_source_namespace_and_source_keyed_log`, the only test that runs a gate; unchanged. |
| negative control | The same shortening that the old oracle passed fails the doc oracle: `Lists differ: [… '--propagation', 'private'] != [… '--kill-child=SIGKILL']`. For the key binding, measured 2026-08-02 against the corrected assertions: dropping `readonly_isolation_identity` from `build_artifact_identity`'s payload fails with `1 != 3 : … does not bind into the artifact key`, and dropping it from `build_source_identity`'s payload fails with `1 != 3 : … does not bind into the source key`. Under the first of those the `74cd37d` pair assertion would have **passed** — measured: 3 distinct pairs, 3 distinct source keys, 1 distinct artifact key. All scratch mutations restored → `OK`. |
| remaining boundary | Closed; the name and the pair-uniqueness claim were corrected on 2026-08-02 (§10/F3). |

### R5 — a forward guard was filed as a regression control

| Field | Content |
|---|---|
| identity | R5 / review `NIT-5`; base `4e5c0e7`; `test_ai_tools.py`, prior `result.md:201`/`:208` |
| prior property | Listed under "New non-vacuous regressions", implying it fails on the predecessor. |
| pre-edit observation | The review measured it passing unchanged on `aaa39b6` in the 168-test predecessor run. Accepted as reproduced: no charge-back consumer existed at `aaa39b6` either, so the test cannot fail there by construction. **Reproduces (by construction).** |
| causal attribution | Repair-introduced labelling error. The test is sound; the claim about what it demonstrates was not. |
| repair | Renamed to `test_forward_guard_no_consumer_charges_reclaimed_bytes_back` with a docstring stating that it passes on the predecessor, that the predecessor-failing control for the same claim is `test_reclaim_is_not_described_as_exact_or_charged_back`, and its four-literal-pattern scope limit. Prior `result.md` heading changed to "New regressions and guards", the row rewritten, both marked as corrections with a pointer here. |
| oracle | The name and docstring now entail only what the assertions demonstrate. |
| negative control | Not applicable — a labelling correction. Deleting the guarded property still changes the test's result, so it remains admissible as an oracle under `I4`. |
| remaining boundary | **Closed.** |

### Routed, not assigned — `mount_points` parser

Recorded as pre-existing, per the brief; `readonly_source_exec.py` was not
changed. Verified independently rather than taken on assertion — SHA-256 of
the `mount_points` function region:

```text
bfbbc2b   db8f09559030749090514aafc037ce583de67c9b38654cc62d19d85415fb2dda
aaa39b6   db8f09559030749090514aafc037ce583de67c9b38654cc62d19d85415fb2dda
4e5c0e7   db8f09559030749090514aafc037ce583de67c9b38654cc62d19d85415fb2dda
worktree  db8f09559030749090514aafc037ce583de67c9b38654cc62d19d85415fb2dda
```

Identical across the lineage, matching the review's `db8f0955…`. Needs a
separately admitted owner if promoted.

## 5. Claims considered and rejected

| Considered | Rejected because |
|---|---|
| Restore the prior review's checkpoint figure of `12` | Refuted by measurement in both the second review and here. The two extra errors are `git ls-files` failures from a non-Git extraction. The brief explicitly says not to blindly restore 12. |
| Publish this repair's own suite count in `history.md` | The brief forbids publishing the new HEAD's count before the final commit exists, and it would make the R1 oracle self-referential. Only fixed prior commits are recorded. |
| Fix `mount_points` while adjacent and cheap | Explicitly routed away; not this repair's owner. |
| Change the per-entry accounting convention to inode de-duplication | Out of scope and would loosen the 32 GiB budget; unchanged, as in the second repair. |
| Add findings beyond R1–R5 to look thorough | The brief sets no quota. Two genuinely observed items are recorded instead — the stale "always exact" clause in the R1 cell, and R2's narrower reachability — and nothing was invented. |
| Assert the R1 counts by running each historical suite in-test | Slow and environment-dependent. Static `ast` counting is validated against the runtime numbers for all three commits, giving the same evidence without executing historical trees. |

## 6. Commands, exit codes, counts, durations

```text
git rev-parse HEAD / HEAD^{tree}                                    exit 0
git merge-base --is-ancestor aaa39b6|bfbbc2b HEAD                   exit 0
git show 130878d:…/review.md | sha256sum                            exit 0  aad067b2…
git worktree add --detach /tmp/r3wt/ckpt bfbbc2b                    exit 0
  python3 -m unittest test_ai_tools    Ran 148 — FAILED (f=4, e=6)  17.5 s
git archive bfbbc2b | tar -x -C /tmp/r3arch                         exit 0
  python3 -m unittest test_ai_tools    Ran 148 — FAILED (f=4, e=8)  17.2 s
git worktree add --detach /tmp/r3wt/pred aaa39b6                    exit 0
  python3 -m unittest test_ai_tools    Ran 159 — OK                 18.0 s
4e5c0e7 (pre-edit)                     Ran 168 — OK                 18.6 s

selected-test runs during repair:
  HistoryTestCountConsistencyTests            Ran 3  — OK    1.1 s
  SourceKeyedGateDocumentationTests           Ran 6  — OK    0.4 s
  both classes together                       Ran 9  — OK    1.3 s
  test_hardlinked_marker_overcount_is_reported Ran 1 — OK    0.04 s

final validation, from the repaired tree:
python3 -m unittest test_ai_tools                    Ran 173 — OK   19.7 s  exit 0
PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile docs/development/tools/*.py exit 0
python3 docs/development/tools/check_ai_development.py       exit 0  PASS ×7
python3 docs/development/tools/check_ai_contracts.py         exit 0
python3 docs/development/tools/check_project_docs.py         exit 0  607 files
  ^ 607 was the count at that invocation, which ran before this result
    file existed. The exact reviewed tree 74cd37d contains it, and the
    checker reports 608 there. Re-measured 2026-08-02: exit 0, 608 files.
    Nothing regressed between the two numbers; one file was added.
python3 docs/development/tools/check_source_docs.py          exit 0  44/34/12
python3 docs/development/tools/check_ai_acceleration_docs.py exit 0
git diff --check                                             exit 0
```

Negative controls executed and reverted: `157`/`161` in `history.md`;
`st_nlink`-only marker branch; three-element `UNSHARE_ISOLATION_FLAGS`
(twice — once against the old substring oracle, once against the new ones).
Each was restored immediately and the suite re-run green.

No Cargo, `rustc`, Clippy or `rustfmt` process was started. The product CLI
suite under `tests/` was not run. No Codex, Fugu, subagent, scheduler,
canonical integration, rebase, amend, squash or destructive Git operation.
The two throwaway worktrees under `/tmp/r3wt` are removed at the end.

## 7. Preserved boundaries

All six independently verified closures, the ten fail-closed classes, the
global execution lock and the cleanup/resource ordering are unchanged.
Specifically: `_preload_policy_module`, `cargo_registry_locks`, the removal
walk's control flow, `namespace_usage_at`'s entry cap, the candidate
recapture sites and the descriptor complement closure are all untouched by
this repair. The only implementation change is three lines of accounting in
the marker branch of `remove_tree_contents_at`; it adds no `unlink`,
`rmdir`, deadline or entry-cap decision.

## 8. Remaining routed observations

- **`mount_points` parser** — pre-existing, routed, unowned here.
- **R2 reachability** — the marker case is fail-closed on the validated
  path and reachable only through `recover_incomplete_namespaces`. Fixed
  regardless; recorded so a reviewer is not misled about exposure.
- **Product CLI suite under `tests/`** — still not run in this lineage; it
  is disjoint from every changed path.
- **No Rust evidence** — Cargo gates remain held for manager release.

## 9. Head

A commit cannot contain its own hash. The head to review is the single
commit whose parent is `4e5c0e7f058cd406cb7a183ecda8b3a07f3ee912` and which
adds this file:

```bash
git log -1 --format=%H 4e5c0e7f058cd406cb7a183ecda8b3a07f3ee912..HEAD
```

That commit is `74cd37d5e9204c45ac23bf31aa6973d6004cc0f0`. It was reviewed
and returned `REQUEST_CHANGES`; §10 records what that review found and what
was done about it. The head to review now is the single additive successor
commit whose parent is `74cd37d5e9204c45ac23bf31aa6973d6004cc0f0`. `74cd37d`
itself is unchanged — not amended, rebased or rewritten.

Return that commit to a fresh independent reviewer. **This candidate is not
accepted until a fresh independent report returns `ACCEPT`.**

## 10. Corrections after the third-closure review (2026-08-02)

The review of `74cd37d` returned `REQUEST_CHANGES` with findings F1–F6.
This section is an in-place correction record, not a new result: no new
packet or result file was created, `74cd37d` was not amended, and the
successor commit touches exactly two paths —
`docs/development/tools/test_ai_tools.py` and this file. `history.md`, the
Cargo gate implementation, the contract document and all product code are
byte-identical to `74cd37d`.

| Finding | Disposition | What changed |
|---|---|---|
| F1 — the count guard was blind to the Japanese spelling | **Included** — the bound finding was still open in the spelling it was raised in | `HistoryTestCountConsistencyTests` now extracts counts through one `stated_counts` helper over `COUNT_FORMS`, which recognises the ASCII `<N> test`/`<N> tests` forms **and** the Japanese `<N>件全pass` form, plus a new exact-form control test |
| F2 — the permanent-row guard depended on historical objects being reachable | **Included** as a canonical-port reproducibility repair | Row assertions now compare against literal `CLAIMED` counts; `defined_test_count` returns `None` instead of raising; only the re-derivation test may skip |
| F3 — a test name claimed argv coverage it did not have, and pair-uniqueness was weaker than claimed | **Included** as a local labelling correction, with the assertion strengthened to match the name | Renamed to `test_unshare_sequence_binds_identity_and_both_keys`; the two keys are now projected and checked independently |
| F4, F5, F6 | **Not changed** | None of the three carries a must-fix trigger: no false claim in the permanent record, no oracle that cannot fail in the direction it is named for, and no reproducibility defect. Changing them would have widened the diff past the two admitted paths without closing anything |

### F1 — the exact recurrence, and the corrected oracle

Both wrong counts were written in Japanese, and the `74cd37d` extractor
matched only `(\d+)\s*tests?(?![A-Za-z])`:

```text
aaa39b6  …checkpoint時点でtest 148中10件failだったものを157件全passへ。
4e5c0e7  …候補は161件全pass。
```

Neither string contains `test` adjacent to its number, so `stated_counts`
returned `∅` for the wrong value and the set-difference guard admitted it.
Measured on 2026-08-02 against the `74cd37d` ASCII-only pattern with
`候補は161件全pass。` inserted into the live row:
`test_history_states_no_test_count_matching_no_tree` → `ok`. The guard
named for `157`/`161` could not see either number as written.

`COUNT_FORMS` is now `(?:tests?(?![A-Za-z])|件全pass)`, shared by both row
assertions and by `stated_counts`. The `全pass` suffix is load-bearing:
plain `件` counts other things in the same row (`MUST/CRITICAL 6件`,
`10件fail`, `12件に見える`, `test 3件`), and admitting those would make every
row state counts matching no tree.
`test_the_historical_wrong_count_forms_are_recognised`
pins the two historical strings verbatim, the ASCII and
CJK-adjacent forms, and the non-count `件` exclusions; it fails if the
Japanese alternative is dropped, which is what keeps it non-vacuous.

Re-adding either wrong count now fails for the set-difference reason:

```text
…`aaa39b6`で157件全passへ    Items in the first set but not the second: 157
                             admitted [148, 159, 168]  (+ row-regex failure)
…159 test全pass。候補は161件全pass。
                             Items in the first set but not the second: 161
                             admitted [148, 159, 168]  (set difference alone)
```

### F2 — permanent-row checks no longer depend on object reachability

At `74cd37d` both row assertions called `git show` with `check=True`, so a
checkout without the historical objects turned the permanent record's guard
into an error rather than a check. Measured in a `git archive HEAD`
extraction of `74cd37d` outside any repository:

```text
ERROR test_history_states_no_test_count_matching_no_tree
ERROR test_history_states_the_right_count_for_each_named_commit (×3 subtests)
  subprocess.CalledProcessError: 'git show bfbbc2b:…' exit 128
```

The counts are fixed historical facts, so they are held as literals in
`CLAIMED` and the row assertions use them directly. `CLAIMED` maps the
concise label the ledger row spells to `(full 40-hex revision, count)`:
`git show` is given the full identifier, since an abbreviation is unique
only until the object database grows into it, while rows are still matched
on the short label the ledger has always written. `defined_test_count` runs
with `check=False` and returns `None` when the object is absent, so no
`CalledProcessError` can escape any test. Only
`test_static_count_agrees_with_the_recorded_runtime_count` reacts to that
`None`, by skipping. Measured in the same archive extraction of the
corrected tree:

```text
test_history_states_no_test_count_matching_no_tree            ... ok
test_history_states_the_right_count_for_each_named_commit     ... ok
test_the_historical_wrong_count_forms_are_recognised          ... ok
test_static_count_agrees_with_the_recorded_runtime_count      ... skipped
Ran 4 tests — OK (skipped=1)
```

In the real repository all four run and the literals are re-earned from the
blobs: 148/159/168.

### F3 — name and assertion brought back to what the test demonstrates

The old name asserted argv coverage the body never had — nothing in it
launches or captures a process. Launch argv and the published result are
covered by `test_gate_uses_locked_source_namespace_and_source_keyed_log`,
the only test that runs a gate; the comment now says so instead of implying
this test shares that duty. The pair-uniqueness assertion was also weaker
than its message: three distinct `(source_key, artifact_key)` pairs are
guaranteed as soon as *one* member moves, so a key that stopped binding the
sequence entirely still passed. Demonstrated by dropping
`readonly_isolation_identity` from `build_artifact_identity`'s payload —
3 distinct pairs, 3 distinct source keys, **1** distinct artifact key, old
assertion `PASS`. The two projections are now separate assertions, and that
same mutation fails with `1 != 3 : … does not bind into the artifact key`;
the mirror mutation on `build_source_identity` fails with
`1 != 3 : … does not bind into the source key`. Both scratch mutations to
`run_source_keyed_cargo_gate.py` were reverted; the file is byte-identical
to `74cd37d`.

### F1–F3 validation

```text
HistoryTestCountConsistencyTests + SourceKeyedGateDocumentationTests
                                                     Ran 10 — OK    0.6 s
python3 -m unittest test_ai_tools                    Ran 174 — OK  18.8 s  exit 0
PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile docs/development/tools/*.py exit 0
python3 docs/development/tools/check_ai_development.py       exit 0  PASS ×7
python3 docs/development/tools/check_ai_contracts.py         exit 0
python3 docs/development/tools/check_project_docs.py         exit 0  608 files
python3 docs/development/tools/check_source_docs.py          exit 0  44/34/12
python3 docs/development/tools/check_ai_acceleration_docs.py exit 0
git diff --check                                             exit 0
```

Suite 173 → 174; the one added test is the exact-Japanese-form control.
No Cargo, `rustc`, Clippy or `rustfmt` process was started, and the product
CLI suite under `tests/` — disjoint from both changed paths — was not run.
Every scratch mutation used above (`history.md` ×2, the ASCII-only
`COUNT_FORMS`, both identity payloads) was restored and re-verified green.

**Nothing in this section claims acceptance.** The successor commit remains
a candidate until a fresh independent report returns `ACCEPT`.
