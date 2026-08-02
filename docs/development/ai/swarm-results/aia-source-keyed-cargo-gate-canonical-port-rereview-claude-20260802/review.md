# AIA-046 Source-Keyed Cargo Control — Canonical Semantic Port, Fresh Exact Review (Claude, 2026-08-02)

Role: fresh independent exact-candidate reviewer. Report-only. Provider/model:
Claude Opus 5 (`claude-opus-5`), fresh session and fresh worktree. No Codex,
Fugu, Cargo, rustc, Clippy, rustfmt, product CLI, install, dogfood, network, or
external write was used. The product/control candidate was never mutated.

**Overall verdict: `ACCEPT`.**

This `ACCEPT` means only that commit `fb891b0` is a faithful, non-weakening
semantic port of the source-keyed Cargo control onto canonical `41dea14`, and
is therefore fit for canonical integration and for supplying evidence to held
Rust reviews. It does **not** close CFR-168, CFB-136, CFB-138, any product
candidate, Install, Dogfood, or any lifecycle stage. It is not evidence about
AIA-047, review-bottleneck improvement, or product readiness. Section 9 names
two manager preconditions that must be discharged in separate authorized
commits before or alongside integration; neither is repairable inside this
candidate's frozen path boundary.

---

## 1. Exact subject — recomputed before work and again before verdict

Every value below was recomputed in this worktree with `git`, twice (start and
end of the review). All matched the manager-supplied values exactly. No
mismatch was found; the mismatch-is-BLOCK condition did not trigger.

| Item | Supplied | Recomputed | Match |
|---|---|---|---|
| Canonical base / candidate parent | `41dea1488bf04c8a2bc1c1d1ad3df7ac9b2570f6` | `41dea1488bf04c8a2bc1c1d1ad3df7ac9b2570f6` | yes |
| Candidate HEAD | `fb891b01f45e4d1963c48f06b827f8335150dace` | `fb891b01f45e4d1963c48f06b827f8335150dace` | yes |
| Candidate tree | `2dadc3088c14fb624fb30d8060eba7466297959a` | `2dadc3088c14fb624fb30d8060eba7466297959a` | yes |
| Additive commits in `base..candidate` | 1 | 1 | yes |
| Merge commits in `base..candidate` | 0 | 0 | yes |
| Changed paths in `base..candidate` | 21 | 21 | yes |
| Binary patch SHA-256 (`git diff --binary base candidate`) | `ba67a827…c6b1f` | `ba67a8278a20e3624b5cb98a99a1725456b92e112ef3b6a34f7b118afa3c6b1f` | yes |
| Implementation result SHA-256 | `1825f399…bb24d4` | `1825f3996dfc085244827f9d803e4c51144ffb703d6815617f0fdd9546bb24d4` | yes |
| Candidate worktree clean | clean at freeze | `git status --porcelain` empty at start and at end | yes |

Source authority, recomputed:

| Item | Supplied | Recomputed | Match |
|---|---|---|---|
| Source base (full identity) | `893cbcad` | `893cbcad2b1f2a8455a5c256246c8110d3fcbdd6` (commit, resolves uniquely) | yes |
| Accepted source HEAD | `1bbcfde2acc60476b5e29136a30f7c808fb6bb4e` | present, commit object | yes |
| Source head tree | `753c44216e0c1145fd97f33d3863c111f49eff2c` | `git rev-parse 1bbcfde^{tree}` → identical | yes |
| Source-range binary patch SHA-256 | `03b54705…47c56` | `03b54705f09cc83b75598560b42fecd972650e2d2e08de0f0d25c152547c2e56` | yes |
| Source range commits / paths | — | 5 commits (`bfbbc2b`, `aaa39b6`, `4e5c0e7`, `74cd37d`, `1bbcfde`), 20 paths | — |

The candidate's 21 paths are exactly the 20 source-range paths plus
`docs/development/ai/swarm-results/aia-source-keyed-cargo-gate-canonical-port-20260802/result.md`.
Set equality was computed, not eyeballed.

The result's "required correction to the stated base identity" is confirmed:
`git cat-file -t 41dea14b8e3bcb66de9d74fb9aee51d6ee9506ad` fails
(`could not get object info`), and `git rev-parse --disambiguate=41dea14`
yields the single object `41dea1488bf04c8a2bc1c1d1ad3df7ac9b2570f6`. The port
is built on the correct object; the 40-character value in the earlier task
authority was a transcription error.

**Unavailable evidence — the source `ACCEPT` report.** I was instructed to
locate and read the full immutable report/transcript authority for the source
`ACCEPT` on `1bbcfde`. It is not present in this repository. See §9/O1.

## 2. Method

Every claim below was derived here. The implementation result was read and then
treated as a set of claims to falsify, not as evidence. Checkers were run, but
no acceptance is inferred from a checker passing: each control that matters was
independently reversed and required to fail closed.

1. Recomputed all identities and the path set from Git objects.
2. Recomputed the B/S/C classification for all 20 source paths from blob OIDs.
3. Verified all 12 ADD/BASE_EQUAL candidate blobs against the source-HEAD blob
   OIDs, and verified BASE_EQUAL equality at `B` and `C` *before* the copy.
4. For the 8 SEMANTIC paths, constructed the three-way view (`B→S`, `B→C`,
   `C→candidate`) and checked, per path, that every source addition is present
   and every canonical-only addition survives. For the two Python modules this
   was done mechanically (AST top-level symbol sets, per-definition byte
   comparison, per-statement import union), not by reading prose.
5. Enumerated qualified `Class.test_method` identifiers by AST at B, S, C and
   the candidate; computed `P = (C ∪ (S \ B)) \ (B \ S)`; compared counts,
   membership and a sorted-list SHA-256; checked for duplicate test and class
   definitions.
6. Ran the required validation sequentially in this worktree, in the foreground.
7. Built a throwaway out-of-tree mirror at `/tmp/cfr168rev/mirror`
   (`git archive` of the candidate tree, fresh `git init`) whose tree hash was
   verified equal to `2dadc308…`, and ran reversals and direct behavioral
   probes there. Every reversal was reverted before the next; the mirror was
   `git status`-clean between reversals. The candidate worktree was never
   modified. Scratch removed at the end.

## 3. Three-way classification (recomputed)

`B` = `893cbcad`, `S` = `1bbcfde`, `C` = `41dea14`, `K` = candidate `fb891b0`.
Blob OIDs abbreviated to 8 hex.

| Path | `B` | `S` | `C` | `K` | Class | `K == S`? |
|---|---|---|---|---|---|---|
| `ai/source-keyed-cargo-gates.md` | — | `6a6af154` | — | `6a6af154` | ADD | yes |
| `ai/swarm-results/…second-closure-repair-20260731/result.md` | — | `0350ae2d` | — | `0350ae2d` | ADD | yes |
| `ai/swarm-results/…third-closure-repair-20260731/result.md` | — | `a6462a1c` | — | `a6462a1c` | ADD | yes |
| `tools/pdeath_exec.py` | — | `0afec5bc` | — | `0afec5bc` | ADD | yes |
| `tools/readonly_source_exec.py` | — | `5a7e8757` | — | `5a7e8757` | ADD | yes |
| `tools/run_source_keyed_cargo_gate.py` | — | `31e3194c` | — | `31e3194c` | ADD | yes |
| `ai/failure-cookbook.md` | `d823c68f` | `8c34f433` | `d823c68f` | `8c34f433` | BASE_EQUAL | yes |
| `ai/stage5-manual-control-runbook.md` | `6dd6486b` | `51700ed4` | `6dd6486b` | `51700ed4` | BASE_EQUAL | yes |
| `tools/ai_contracts.py` | `3ded0930` | `a53cd9cc` | `3ded0930` | `a53cd9cc` | BASE_EQUAL | yes |
| `tools/build_source_bound_rust.py` | `f30cac14` | `0283bde3` | `f30cac14` | `0283bde3` | BASE_EQUAL | yes |
| `tools/check_rust_execution_profile.py` | `b1313eb8` | `c85837e9` | `b1313eb8` | `c85837e9` | BASE_EQUAL | yes |
| `tools/probe_validation_capabilities.py` | `96115fa1` | `62f17a47` | `96115fa1` | `62f17a47` | BASE_EQUAL | yes |
| `ai/README.md` | `aa061c96` | `050d085a` | `e389b931` | `3849262a` | SEMANTIC | no (union) |
| `ai/acceleration-master-plan.md` | `fb137892` | `f054de8c` | `63724d34` | `c35d21a3` | SEMANTIC | no (union) |
| `ai/swarm-batches/…file-doctor-…-rereview-claude-20260730.md` | `8a31d969` | `0d6769a6` | `850ce81e` | `f7bc2637` | SEMANTIC | no (union) |
| `ai/swarm-batches/…http-mr-preflight-…-rereview-claude-20260730.md` | `7f7f1446` | `1d01e936` | `eea642a1` | `31e0bf8e` | SEMANTIC | no (union) |
| `development/history.md` | `660898a0` | `8eb292e1` | `9c746d0a` | `70e79d08` | SEMANTIC | no (union) |
| `tools/check_ai_acceleration_docs.py` | `8bb8155e` | `cad7220e` | `bc1b4a0a` | `2f4cad18` | SEMANTIC | no (union) |
| `tools/run_timed_gate.py` | `cfb8996a` | `0a87e9dc` | `67cf076a` | `57751c00` | SEMANTIC | no (union) |
| `tools/test_ai_tools.py` | `906a448d` | `07b017ec` | `38ccf166` | `ce6bb9e7` | SEMANTIC | no (union) |

**Totals: 6 ADD, 6 BASE_EQUAL, 8 SEMANTIC — matches the result exactly.**

- All 6 ADD paths are absent at both `B` and `C` and byte-exact to `S` at `K`.
- All 6 BASE_EQUAL paths really were blob-identical at `B` and `C` before the
  copy (columns 2 and 4 above), so no canonical content was displaced, and are
  byte-exact to `S` at `K`.
- All 12 ADD/BASE_EQUAL blob OIDs equal the `1bbcfde` blob OIDs. This is a
  stronger check than a content diff: the OIDs are identical objects.
- No SEMANTIC path was wholesale-copied from the source.

### 3.1 Semantic union — per-path verification

**`ai/README.md` (+4/−3).** Canonical-only additions all survive untouched
(they do not appear in the `C→K` diff at all): governance authority map, the
AIA-047 checkpoint section, the 14-step lifecycle, the batch preflight step,
the `review-recurrence/README.md` row and the `check_review_recurrence.py` row.
Source rows ported: `build_source_bound_rust.py`,
`check_rust_execution_profile.py`, and the new
`run_source_keyed_cargo_gate.py` / `readonly_source_exec.py` row. The
`run_timed_gate.py` row was rewritten by both branches; the merged row is a
true union — it carries the source's *trusted non-Cargo scope, bounded
same-descriptor log identity, optional live resource guard, rollback-safe
exclusive result publication, wall-clock-discontinuity evidence and the
Rust/shell rejection sentence* **and** canonical's *complete external log
digest, clean/index-state evidence and post-command source identity*. Nothing
from either side was dropped.

**`ai/acceleration-master-plan.md` (+5/−4).** Canonical's
`Stage 5 … Active 2026-07-31` heading, the AIA-047 row and next-item 9 are all
absent from the `C→K` diff, i.e. byte-unchanged. AIA-021/040/042/044 take their
source versions (canonical still carried base text for all four). The new
AIA-046 row is inserted immediately before AIA-047. No text was added that
makes AIA-047 a CFR-168 product prerequisite; canonical's explicit
"not a product source prerequisite" framing is intact.

**The two rereview packets (+6/−1, +8/−3).** The `C→K` diffs touch **only** the
shared-profile code block and the paragraph immediately after it. Canonical's
rewritten titles, `Status:` lines, exact bound subject tables, candidate
`cd130e5` binding, its pre-launch acceptance-profile block, the
`92fc8103`/`4bb2880f` binding, the manager rollover deviations section and the
candidate-card references are all untouched. The source hunks applied verbatim:
the `RUSTUP_TOOLCHAIN=1.97.0-x86_64-unknown-linux-gnu` export and the
"Do not invoke Cargo directly … logical argv after `--` to
`run_source_keyed_cargo_gate.py`" requirement, both retaining the "do not
create a worktree-local target or another Rustup/Cargo home" clause.

**`development/history.md` (+3/−0).** Verified mechanically by multiset
comparison: 549 → 552 lines, **0 canonical lines missing from the candidate**,
and each of the 3 source rows present **exactly once**. Canonical's three top
rows (2026-08-02 closure rebaseline, 2026-07-31 governance repair, 2026-07-31
AIA-047 integration) remain first, in that order; the three 2026-07-31 source
rows follow them and precede the 2026-07-30 block — correct reverse-chronology.
**No history row was added for this port and no row claims it is accepted.**

**`tools/check_ai_acceleration_docs.py` (+4/−0).** Pure union. Canonical's
`review-recurrence-control.v1.schema.json`, `review-recurrence/README.md` and
`check_review_recurrence.py` required-file entries are preserved; the source's
`source-keyed-cargo-gates.md`, `run_source_keyed_cargo_gate.py` and
`readonly_source_exec.py` entries plus the `RUSTUP_TOOLCHAIN` marker are added.
No entry was removed.

**`tools/run_timed_gate.py` (+1475/−56).** AST top-level symbol union computed
mechanically:

| Set | Count |
|---|---|
| symbols at `B` | 8 |
| symbols at `S` | 49 (41 added by source) |
| symbols at `C` | 10 (2 added by canonical: `TIMED_V2_LIMITS`, `run_gate_v2`) |
| expected union `(C ∪ (S\B)) \ (B\S)` | **51** |
| symbols at candidate | **51** |
| missing / invented | **0 / 0** |

Per-definition byte comparison: `TIMED_V2_LIMITS` and `run_gate_v2` are
**byte-identical** to canonical (SHA-256 `7bb9e176…` and `c4ba854f…` on both
sides). Every source top-level definition is byte-identical to `S` except
exactly two — `parse_args` and `main` — which are precisely where canonical's
`--emit {v1,v2}` option and the v1/v2 runner dispatch had to be grafted.
`run_gate_v2` calls the source `run_gate` unmodified, and the source `run_gate`
still exposes the four positional and three keyword parameters it passes.
Critically, in the merged `main()` the source's `protected_rust_dispatch`
rejection runs **before** the emission dispatch, so the canonical v2 path is
protected too — confirmed behaviorally in §7 (C2).

**`tools/test_ai_tools.py` (+5307/−72).** Preamble unioned per import
statement: canonical's `check_review_recurrence`, `measure_recurrence_v2_burden`
and all six `recurrence_v2_*` module imports survive; the source's
`run_source_keyed_cargo_gate` and `readonly_source_exec` imports plus
`run_gate_v2`, `open_exclusive_output`, `protected_rust_dispatch` and
`verify_output_identity` are added. Per-method byte comparison found **zero**
source test methods altered in the candidate, and exactly **one**
candidate-authored change to a canonical test method (§4.1). Six further
canonical-vs-candidate differences are cases where canonical still carried base
text and the source strengthened the test; all six are byte-identical to `S`.

## 4. Test-set proof

Qualified `Class.test_method` identifiers extracted by AST from each tree's own
`test_ai_tools.py`. Independently enumerated; the result's figures were not
assumed.

| Set | Definitions | Unique | Classes | Duplicate tests | Duplicate classes |
|---|---:|---:|---:|---|---|
| `B` = `893cbcad` | 65 | 65 | 20 | none | none |
| `S` = `1bbcfde` | 174 | 174 | 24 | none | none |
| `C` = `41dea14` | 109 | 109 | 21 | none | none |
| `K` = candidate | **218** | **218** | **25** | **none** | **none** |

| Quantity | Value |
|---|---:|
| `\|S \ B\|` (added by source) | 110 |
| `\|C \ B\|` (added by canonical) | 44 |
| `\|B \ S\|` (deleted by source) | 1 |
| `\|(S\B) ∩ (C\B)\|` | 0 |
| `\|P\|` = `\|(C ∪ (S\B)) \ (B\S)\|` | **218** (25 classes) |
| `\|K\|` | **218** |
| `P \ K` (missing) | **0** |
| `K \ P` (unexpected) | **0** |

- `218 = 109 + 110 − 1`. Arithmetic confirmed.
- `B \ S` is exactly `RustExecutionProfileTests.test_shared_low_memory_process_passes`,
  deleted by the accepted source. It is **not** in `C \ B`, so removing it
  discards no canonical work.
- **Sorted-identifier SHA-256, newline-joined:**
  - computed `P`: `b83e0b41c383b9265f0c9a6c32ec71ee2c2a92e049205e9502025f48f389de62`
  - committed candidate `K`: `b83e0b41c383b9265f0c9a6c32ec71ee2c2a92e049205e9502025f48f389de62`
  - **identical.** This independently reproduces the result's hash.
- Every test definition occurs exactly once; no class is defined twice.
- Canonical `ReviewRecurrenceTests`: 43 at `C`, **43** at the candidate, and the
  two sets are **equal**, not merely equinumerous. All 43 canonical recurrence
  tests are present.
- **All accepted source tests are present:** `S \ K = ∅`.
- `TimedGateTests` = 23 = the source's 22 plus canonical's
  `test_v2_cli_requires_an_explicit_result_path`.

### 4.1 The one adapted canonical fixture — is it the minimal correct adaptation?

The sole candidate-authored change to a canonical test body is in
`ReviewRecurrenceTests.test_v2_timed_runner_marks_in_tree_outputs_and_index_flags_nonpassing`:
a five-line comment plus `(repo.root / "evidence").mkdir()`.

Cause: the base `run_gate` silently ran
`log_path.parent.mkdir(parents=True, exist_ok=True)`. The accepted source
replaced that with `open_exclusive_output`, which requires an existing,
symlink-free parent and creates the log `O_EXCL|O_NOFOLLOW`.

**Assessment — minimal and correct; the oracle is not weakened.**

- Production behaviour is untouched. The hardening was kept, not reverted. I
  verified directly (§7, P1–P6) that the runner still rejects a symlinked
  parent, a symlink at the output path, a missing parent, an existing regular
  file, a hardlinked path (`nlink != 1`), and `..`/`.`/`""` filenames.
- The test's subject is unchanged. Its assertions —
  `log_inside_source_worktree`, `result_inside_source_worktree`,
  `index_flags_clean` false, and `facts.passing` false — are all still present
  and still asserted. Nothing was deleted, relaxed, skipped or `assertRaises`-d
  around.
- The two alternatives are both worse: restoring the implicit `mkdir` would
  revert accepted production hardening, and moving the log out of the tree
  would destroy the very condition (`log_inside == True`) the test exists to
  detect.
- `evidence/` is already in the fixture's `.gitignore`, so creating it does not
  perturb the clean-state assertions.
- Non-vacuity confirmed by reversal R2: the test still fails closed when the
  canonical control it guards is removed.

The adaptation is one `mkdir` — there is no smaller correct change.

## 5. Validation — sequential, foreground, in this worktree, no Cargo

No Cargo, rustc, Clippy, rustfmt, product CLI, install or dogfood command was
executed. Durations are wall-clock from this session.

| # | Command | Result | Duration |
|---|---|---|---|
| V1 | `python3 -m unittest test_ai_tools.{SourceKeyedCargoGateTests,SourceKeyedGateDocumentationTests,HistoryTestCountConsistencyTests,PolicyModulePreloadTests,TimedGateTests,ReviewRecurrenceTests,RustExecutionProfileTests,RustBuildProvenanceTests,CapabilityProvenanceTests}` | **OK — Ran 163 tests**, rc 0 | 66.99 s |
| V2 | `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest docs/development/tools/test_ai_tools.py` | **OK — Ran 218 tests**, rc 0 | 61.58 s |
| V3 | `PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile docs/development/tools/*.py` | rc 0, 34 files | 0.26 s |
| V4 | **`python3 docs/development/tools/check_ai_development.py --timeout 180`** | **rc 0 — PASS ×7** (`acceleration_docs`, `contracts`, `project_docs`, `source_docs`, `runtime`, `tool_tests`, `validation_capabilities`); `AI development gate passed.` | **65.56 s** |
| V5 | `python3 docs/development/tools/check_ai_development.py` (bare default 60 s) | **rc 1 — `FAIL tool_tests`**, other six PASS | 63.64 s |
| V6 | `python3 docs/development/tools/check_ai_contracts.py` | rc 0 — `AI contract check passed.` | 0.49 s |
| V7 | `python3 docs/development/tools/check_project_docs.py` | rc 0 — `683 Markdown files inventoried` | 1.80 s |
| V8 | `python3 docs/development/tools/check_source_docs.py` | rc 0 — 44 modules, 34 commands, 12 object kinds | 0.34 s |
| V9 | `python3 docs/development/tools/check_ai_acceleration_docs.py` | rc 0 — `AI acceleration docs check passed.` | 0.61 s |
| V10 | `python3 docs/development/tools/validate_ai_runtime.py --json` | rc 0 | 3.74 s |
| V11 | `python3 docs/development/tools/probe_validation_capabilities.py --json` | rc 0 | 0.54 s |
| V12 | `git diff --check` | rc 0, no output | — |
| V13 | `git status --porcelain` | empty (clean) at start and at end | — |

V1's selection is non-vacuous: 163 > 0 tests were selected and executed across
all nine named classes.

## 6. Reversal and non-vacuity results

All in the throwaway mirror `/tmp/cfr168rev/mirror`, whose tree hash was
verified equal to the candidate tree `2dadc3088c14fb624fb30d8060eba7466297959a`.
**The candidate was never modified.** Each reversal was reverted and the mirror
confirmed `git status`-clean before the next.

| # | Reversal | Expected | Observed |
|---|---|---|---|
| R1 | `protected_rust_dispatch` → always `None` | source Rust-dispatch control fails | **43 failures** across `test_direct_and_known_indirect_rust_dispatch_are_rejected` (42 argv subtests) + `test_symlink_alias_to_protected_tool_is_rejected` — reproduces the result exactly |
| R2 | Drop the canonical index-flag union in `run_gate_v2` | canonical recurrence-v2 control fails | **2 failures**: `test_v2_timed_runner_marks_in_tree_outputs_and_index_flags_nonpassing`, `test_v2_timed_runner_tracks_head_switch_and_transient_index_flags`. (Two additional `ERROR`s on `test_v2_canonical_port_packet_identities_match_git_objects` also occur at the *unmodified* mirror baseline — a mirror artifact, not a reversal effect.) |
| R3 | Ported history row `159 test全pass` → `161 test全pass` | history-count guard fires | **2 failures**: `test_history_states_no_test_count_matching_no_tree` (`named=('bfbbc2b','aaa39b6','4e5c0e7')`), `test_history_states_the_right_count_for_each_named_commit` (`revision='aaa39b6'`) |
| R5 | Delete `check_review_recurrence.py` (canonical) | acceleration checker blocks | rc 1, `missing-ai-file` |
| R6 | Delete `source-keyed-cargo-gates.md` (source) | acceleration checker blocks | rc 1, `missing-ai-file` |
| R7 | Delete `run_source_keyed_cargo_gate.py` (source) | acceleration checker blocks | rc 1, `missing-ai-file` |
| R8 | Delete `readonly_source_exec.py` (source) | acceleration checker blocks | rc 1, `missing-ai-file` |
| R9 | Delete `review-recurrence/README.md` (canonical) | acceleration checker blocks | rc 1, `missing-ai-file` |
| R10 | `validate_provenance_payload` → `return []` | artifact/result binding control fails | **1 failure**: `test_review_verifies_provenance_payload_source_and_binary` |
| R11 | `audit_recorded_command` → `return []` | recorded source-gate control fails | **42 failures** in `RustExecutionProfileTests` |
| R12 | `normalize_cargo_argv` → identity | source-keyed gate argv/harness control fails | **27 failures/errors** in `SourceKeyedCargoGateTests` |
| R13 | Delete `pdeath_exec.py` | — | checker **passes** — see O4 |
| — | Unmodified mirror baseline | pass | acceleration checker rc 0; the two R2 `ERROR`s present at baseline |

**R1 and R2 together are the decisive pair:** the source's Cargo/dispatch
control and canonical's recurrence-v2 Git-identity control each still fail
closed when reversed, in the same tree. Neither branch's control was made
vacuous by the merge.

## 7. Direct behavioral probes (out-of-tree)

Reversals show the tests are non-vacuous; these probes show the *production*
code is. All executed against the mirror.

| # | Probe | Result |
|---|---|---|
| P1 | `open_exclusive_output` with a symlinked parent directory | rejected — `output parent must be a real directory without symlinks` |
| P2 | Pre-existing symlink at the output path | rejected — `FileExistsError` (O_EXCL\|O_NOFOLLOW), symlink not followed |
| P3 | Missing parent directory (the §4.1 subject) | rejected — `FileNotFoundError`; no implicit `mkdir` |
| P4 | Pre-existing regular file at the output path | rejected; original content **preserved** (no clobber) |
| P5 | Hardlinked output path (`nlink > 1`) | rejected |
| P6 | Filenames `..`, `.`, `""` (traversal) | all rejected; `..` by the explicit lexical guard |
| P7 | Dispatch matrix (14 argv forms) | `cargo`/`/usr/bin/cargo`/`env … cargo`/`timeout … cargo`/`nice -n 5 cargo`/`taskset -c 0 cargo`/`rustup run 1.97.0 cargo` → **`cargo`**; `rustc` → `rustc`; `clippy-driver` → `clippy-driver`; `sh -c`, `bash -lc`, `python3 -c …execvp('cargo')` → **`unresolved-wrapper`** (fail closed); `/bin/echo`, `git status` → `None` (correctly admitted, so the control is not trivially "reject everything") |
| P8 | Process-group timeout with a grandchild (`sleep 987`) | gate exit **124**, `timed_out: true`, duration 3.03 s, **0 leaked descendants** |
| P9 | Log file mode after the gate | `-r--------` (0400), sealed |
| C1 | CLI `run_timed_gate.py … -- cargo test` (v1) | **exit 2** — `cargo dispatch is not admitted; Cargo gates must use run_source_keyed_cargo_gate.py` |
| C2 | CLI `… --emit v2 -- env cargo build` | **exit 2**, same rejection — the canonical v2 path is protected, confirming the ordering in `main()` |
| C3 | CLI `--emit v2` without `--result` | **exit 2** — canonical's guard intact |
| C4 | CLI benign `--emit v2` run | rc 0; `schema: codefire.timed_gate.v2`; `source.revision`/`source.tree` **derived correctly** and equal to the mirror's real `git rev-parse HEAD`/`HEAD^{tree}`; `log_inside`/`result_inside` false; `clean_before`/`head_unchanged` true; log digest and byte count bound; 3 limits emitted |
| C5 | CLI `--emit v2` with a symlinked log parent | **exit 2** — hardening reaches the canonical v2 path |
| C6 | `run_source_keyed_cargo_gate.py --help` under ordinary Python | **exit 1** — `source gate requires exact Python flags: -I -S -B -X pycache_prefix=/dev/null`; refuses to bootstrap |

No path traversal, command injection, shell-wrapper bypass, TOCTOU/symlink/
hardlink weakness, unsafe cleanup, or process-tree leak was found. The error
path of `open_exclusive_output` unlinks only when the observed `(dev, ino)`
still equals the file it created — it cannot delete a file substituted by
another party.

## 8. Adjudication of the two explicit risk items

### Risk A — stale aggregate-timeout evidence in the immutable result

**Recomputed independently; the manager's post-freeze claim was not trusted.**

- `python3 docs/development/tools/check_ai_development.py --timeout 180` →
  **rc 0, all seven components PASS, 65.56 s** (V4). This corroborates the
  manager's independent remeasurement (rc 0, seven PASS, 65.067 s) to within
  normal run-to-run variance.
- The bare default-60 invocation → **rc 1, `FAIL tool_tests`** (V5). The
  result's limitation 1 is accurate, including its warning that reviewers
  running the bare command will see a failure.
- The `--timeout 600` PASS the result records is consistent with V4: 600 > 180
  > the ~62–73 s the suite needs.

**Classification: incomplete, not false; low impact; not actionable inside this
candidate's boundary.** Every figure the result does record reproduced here.
What it omits — the exact authoritative `--timeout 180` run — I have now
supplied at V4, and it passes. The result is immutable and must not be
rewritten. `check_ai_development.py` is not one of the 21 frozen paths, so the
port could not have raised the default without exceeding its authority.

One real consequence is separated out as **O2b** below: the port does make the
*documented default-form* command fail on canonical. That is a genuine,
reachable workflow regression, but it fails **closed** (a visible failure, not
a silent pass), it is honestly disclosed, and its repair boundary lies outside
the frozen path set.

### Risk B — is the ported `RUSTUP_TOOLCHAIN` marker vacuous?

**Finding: currently inert against the live tree, but provably armed and not
bypassable. It does not fail its purpose, and it is not a must-fix.** This was
determined by construction, not assertion.

*Current reachability, recomputed.* `check_active_rust_packet_profiles` selects
a packet only when all three hold: `^Status: (?:blocked|ready)\b`,
`## Validation` present, and the literal `cargo ` present. Across all 22
`swarm-batches/cfr168-rust-*.md` files at the candidate tree, exactly three
match the Status regex, and each fails one of the other two filters:

| Packet | Status matches | `## Validation` | `cargo ` | Selected |
|---|---|---|---|---|
| `…source-keyed-recovery-wave-binding-20260731.md` | yes (`blocked`) | **no** | yes | no |
| `…storage-review-closure-resume-preflight-20260731.md` | yes (`blocked`) | **no** | yes | no |
| `…http-mr-preflight-evidence-closure-repair-20260730.md` | yes (`ready`) | yes | **no** | no |
| both `…rereview-claude-20260730.md` packets | **no** (canonical rewrote both Status lines) | yes | yes | no |

So the count is zero, exactly as the result discloses. Note this is **not a
regression against the canonical base**: at `41dea14` the same three packets
matched the Status regex and the same filters excluded them, so live selection
was already zero. It is a reduction only against source head `1bbcfde`, where
the two rereview packets still carried their pre-governance `Status: blocked…`
lines — and restoring that would require overwriting canonical governance text,
which is outside this port's authority. The port made the correct call.

*Is it vacuous?* No. Two constructed reversals settle it:

| # | Construction | Result |
|---|---|---|
| RB1a | Transition the file-doctor rereview packet's Status to `Status: blocked pending …` (the typed transition that would precede a review launch) | checker **passes** — and it passes *precisely because this port added `RUSTUP_TOOLCHAIN` to that packet* |
| RB1b | Same transition, then delete the `RUSTUP_TOOLCHAIN` export from the packet | checker **fails**, rc 1: `active-rust-packet-missing-shared-profile: … missing exact shared low-memory marker(s): RUSTUP_TOOLCHAIN=1.97.0-x86_64-unknown-linux-gnu` |

The marker is therefore **armed-on-transition**, and both held Rust review
packets are already marker-complete, so the transition will pass rather than
block a launch. The port's packet edits are the thing that makes that true.

*Can the accepted control be bypassed?* No. The packet-profile check is a
documentation-consistency control, not the execution control. The execution
control is separately enforced and armed **today**, with zero dependence on
packet Status text — proved by direct construction against the live tree:

| Recorded command | Audit codes |
|---|---|
| The live `Status: ready` packet's exact documented profile + `cargo test -p codefire-cli --locked` | `recorded-direct-rust-not-source-gated`, `recorded-missing-rustup-toolchain`, `recorded-missing-source-key-gate-key`, `recorded-missing-source-key-gate-provenance`, `recorded-wrong-target-dir` |
| Same, with `RUSTUP_TOOLCHAIN` added (marker fully satisfied) | still **`recorded-direct-rust-not-source-gated`** + 3 more |
| A complete, self-attested profile string + `cargo build --release` | still **`recorded-direct-rust-not-source-gated`** + 3 more |
| A non-Rust command (`python3 -m unittest discover`) | none (correctly admitted) |

A complete profile string never self-attests: only the exact source-gate argv
form is admitted. Reversing this audit costs 42 test failures (R11). Add
`protected_rust_dispatch` (R1: 43 failures; P7/C1/C2: every direct, `env`,
`timeout`, `nice`, `taskset`, `rustup run`, shell and interpreter form rejected)
and the gate's own refusal to bootstrap without `-I -S -B` (C6), and there is no
route by which a Cargo gate escapes the source-keyed path because a packet's
Status line does not match a regex.

*Are current release packets falsely reported covered?* No. The checker emits
findings only; it makes no coverage claim, prints no "covered" statement, and
`check_ai_development.py` does not aggregate one. The result's limitation 2
states the situation accurately and does not overclaim.

**Conclusion on Risk B: deliberately latent until a typed status transition,
with the source-gate path separately and demonstrably enforced. Not a must-fix.**
Three residual weaknesses fall out of the analysis and are recorded as O3a–O3c;
all three are inherited from the accepted source range and none is repairable
within the 21 frozen paths.

## 9. Observations

No finding count was targeted. Style preferences and non-impacting
implementation variance are excluded by design and are listed in §10.

### O1 — The source `ACCEPT` report for `1bbcfde` is not locatable, and canonical's own critical path still names `74cd37d` as unreviewed

- **Evidence.** I searched the candidate tree and every ref in the repository.
  No review report or verdict artifact exists for `1bbcfde`, `74cd37d`, or
  `4e5c0e7`. The only two source-keyed rereview reports in the tree —
  `aia-source-keyed-cargo-gate-closure-rereview-claude-20260731/review.md` and
  `…second-closure-rereview-claude-20260731/review.md` — both return
  **`REQUEST_CHANGES`**, against earlier candidates. The in-tree port packet
  `task-packets/cfr168-source-keyed-cargo-canonical-port-20260731.md` is
  `Status: blocked`, binds source range `893cbcad..4e5c0e7` and **19** paths,
  and records that its reviewer session `d089c8ae-…` "stopped on Claude quota
  before writing a report or verdict"; it instructs that the packet "must be
  rebound to the accepted successor". `task-packets/…third-closure-rereview-with-recurrence-20260731.md`
  is "Prepared, not launched", bound to `74cd37d`, and blocked on an AIA-047
  `ACCEPT` that canonical itself records as reopened. Canonical's own frozen
  critical path at `current-cycle-burn-down-plan.md:88` — rebaselined
  2026-08-02, one day before this port — still reads: "independently review
  source-keyed Cargo control candidate `74cd37d`; only an accepted semantic
  canonical port may supply evidence for held Rust reviews."
- **Affected user/workflow/surface.** The manager's integration decision, and
  every held Rust review that will cite this port as its Cargo evidence.
- **Reachability/likelihood.** Certain, at integration time.
- **Impact/severity.** High for the *integration decision*; **zero** for port
  faithfulness. Faithfulness is verifiable against the `1bbcfde` objects
  themselves, and §3–§4 verify it byte-exactly and set-exactly regardless of
  where the source verdict is recorded.
- **Candidate-created or inherited.** Inherited/upstream. The port's result
  does not overclaim: it labels itself "Fresh exact port review required; not
  yet accepted or integrated", scopes the `ACCEPT` explicitly to the source
  candidate only, adds no history row, and adds no acceptance claim anywhere.
  This is a manager record-keeping gap, not a candidate defect.
- **Must-fix trigger.** Not triggered for this candidate: the repair paths
  (`current-cycle-burn-down-plan.md`, the canonical-port packet, a published
  source review artifact) all lie outside the 21 frozen paths, and amending the
  port to touch them would break the frozen path set and the patch digest.
- **Actionability.** `required_precondition` on the manager, not
  `required_correction` on the candidate.
- **Repair/routing.** Before or alongside integration, in a separate authorized
  commit: (a) publish the immutable `1bbcfde` `ACCEPT` report or its
  session/transcript authority with a content digest; (b) rebind
  `cfr168-source-keyed-cargo-canonical-port-20260731.md` from `4e5c0e7`/19
  paths to `1bbcfde`/20 paths and clear its admission blocker; (c) update
  burn-down item 1 from `74cd37d` to the accepted `1bbcfde`. Until (a) exists,
  integration should not be read as discharging burn-down item 1.

### O2 — The immutable result omits the exact `--timeout 180` aggregate run (Risk A)

- **Evidence.** The result records only the bare default-60 `FAIL` and a
  `--timeout 600` pass. I ran the exact authoritative command: rc 0, seven
  PASS, 65.56 s (V4).
- **Affected.** A reviewer or manager reading the result for the authoritative
  aggregate evidence.
- **Reachability.** Certain; **already discharged** by V4 in this report.
- **Impact/severity.** Low. Incomplete, not false — every recorded figure
  reproduced.
- **Candidate-created.** Yes, but the result is immutable and correctly so.
- **Must-fix trigger.** Not triggered.
- **Actionability.** Not actionable. Do not mutate the result.
- **Routing.** This report's V4 is the authoritative `--timeout 180` evidence.

### O2b — The port makes the documented default-form aggregate command fail on canonical

- **Evidence.** V5: `python3 docs/development/tools/check_ai_development.py`
  → rc 1, `FAIL tool_tests`. The unioned 218-test suite needs ~62–73 s against
  a 60 s per-check default. `ai/README.md` step 11 — canonical text this port
  preserved — instructs exactly that bare form.
- **Affected.** Anyone following the documented 14-step lifecycle after
  integration.
- **Reachability/likelihood.** High.
- **Impact/severity.** Moderate operationally; **fails closed** (a visible
  failure, never a silent pass), so no false evidence can be produced by it.
- **Candidate-created or inherited.** Candidate-created as a *combination*
  effect: the source suite (168) and the canonical suite (109) each fit under
  60 s; only the union does not.
- **Must-fix trigger.** Not triggered. The only repair boundary is
  `docs/development/tools/check_ai_development.py` (raise the default) or a
  suite-runtime reduction — both outside the 21 frozen paths. Requiring it here
  would force the candidate to exceed its authority and invalidate its frozen
  identity.
- **Actionability.** `follow_up`, manager-owned.
- **Repair/routing.** A separate authorized change to
  `check_ai_development.py` raising the default per-check timeout to ≥180 s (or
  splitting `tool_tests`). Until then, the authoritative invocation is
  `--timeout 180`, which passes.

### O3a — No test pins the contents of `ACTIVE_RUST_PROFILE_MARKERS`

- **Evidence.** Reversal RB1c: with the packet transitioned *and* the
  `RUSTUP_TOOLCHAIN` export deleted from it, additionally deleting the tuple
  entry makes the checker pass again. `RustPacketProfileTests` derives its
  fixture *from* the tuple, so removing an entry cannot fail it. This is the
  mechanism behind the result's honestly-recorded "R4/R9 did not fire".
- **Affected.** Future maintainers of the marker set.
- **Reachability.** Only via a future edit to the tuple.
- **Impact/severity.** Low–moderate: a future change could silently weaken the
  enforced profile.
- **Candidate-created or inherited.** Inherited — the same fixture derivation
  exists at accepted source head `1bbcfde`.
- **Must-fix trigger.** Not triggered.
- **Repair/routing.** Follow-up oracle hardening: assert the literal expected
  marker tuple against `DEFAULT_PROFILE` rather than deriving the fixture from
  the tuple under test.

### O3b — A live `Status: ready` packet documents a profile the ported audit now rejects

- **Evidence.** `swarm-batches/cfr168-rust-http-mr-preflight-evidence-closure-repair-20260730.md`
  is `Status: ready for exact original-implementer continuation` and its
  `## Validation` block exports six of the seven markers, omitting
  `RUSTUP_TOOLCHAIN`. Post-port, a recorded command built from that exact
  profile audits with `recorded-missing-rustup-toolchain` (§8, Risk B table,
  row 1).
- **Affected.** An implementer resuming that packet.
- **Reachability/likelihood.** Moderate — the packet is `ready` and actionable
  now.
- **Impact/severity.** Moderate; **fails closed** (the implementer is blocked,
  not silently admitted). It cannot produce false evidence.
- **Candidate-created or inherited.** **Inherited.** That packet's blob is
  `389df36a86ac9be710ffb7b937b3ea6ac91f5dbf` at `B`, `S`, `C` **and** the
  candidate — byte-identical at accepted source head, where the same
  inconsistency existed and was accepted.
- **Must-fix trigger.** Not triggered: the packet is not among the 21 frozen
  paths.
- **Repair/routing.** A separate authorized edit adding
  `export RUSTUP_TOOLCHAIN=1.97.0-x86_64-unknown-linux-gnu` and the
  source-gate-wrapper sentence to that packet, matching what this port applied
  to the two rereview packets.

### O3c — The packet selector's third filter is a fragile substring

- **Evidence.** The `Status: ready` packet in O3b escapes selection only
  because the literal token `"cargo "` never appears in it — all its Cargo
  references are `CARGO_HOME=`, `CARGO_TARGET_DIR=`, etc., and its prose says
  "strict Clippy" and "focused tests". Adding one sentence containing
  `cargo test` would immediately select it, and it would then error on the
  missing marker.
- **Affected.** The stability of the acceleration gate under ordinary packet
  edits.
- **Reachability.** Moderate.
- **Impact/severity.** Low; the failure direction is closed (an error, never a
  silent pass).
- **Candidate-created or inherited.** Inherited — the filter is unchanged from
  the base and from accepted source head.
- **Must-fix trigger.** Not triggered.
- **Repair/routing.** Follow-up: select on an explicit typed packet field
  rather than on incidental prose substrings. Fixing O3b first removes the
  latent error.

### O4 — `pdeath_exec.py` is added but not required by the acceleration checker

- **Evidence.** Reversal R13: deleting `docs/development/tools/pdeath_exec.py`
  leaves `check_ai_acceleration_docs.py` at rc 0, while deleting any of
  `run_source_keyed_cargo_gate.py`, `readonly_source_exec.py` or
  `source-keyed-cargo-gates.md` correctly blocks (R6–R8).
- **Affected.** Presence guarantees for the parent-death helper.
- **Reachability.** Low; the module's *behavior* is covered by the source's
  live unshare/parent-death tests, and the gate fails closed at runtime without
  it.
- **Impact/severity.** Low.
- **Candidate-created or inherited.** Inherited — `pdeath_exec.py` is absent
  from `REQUIRED_AI_FILES` at accepted source head too.
- **Must-fix trigger.** Not triggered.
- **Repair/routing.** Optional follow-up: add it to `REQUIRED_AI_FILES`.

### O5 — Recorded Markdown inventory is 682; the actual count is 683

- **Evidence.** V7 reports 683; the result records 682. The difference is
  exactly one, and is fully explained by the checker having been run before the
  result file itself was written into the tree.
- **Impact/severity.** Negligible; no decision depends on it. Not false in
  substance, imprecise in sequencing.
- **Must-fix trigger.** Not triggered. **Actionability.** None; the result is
  immutable.

### O6 — The recurrence-v2 fixture adaptation is correct (verified, not a defect)

Recorded affirmatively because the task required an explicit determination. See
§4.1: it is the minimal correct adaptation, it does not weaken production
behavior, and it does not weaken the test's oracle. No action.

## 10. Explicitly not corrections

- The candidate's non-alphabetical import ordering in the `test_ai_tools.py`
  preamble is inherited verbatim from the accepted source block; a style
  preference, not an impact.
- Timing figures in the immutable result (68.0 s / 72.7 s) differ from mine
  (66.99 s / 61.58 s) by ordinary run-to-run variance on a shared host.
- Two `ERROR`s on `test_v2_canonical_port_packet_identities_match_git_objects`
  during R2 are present at the unmodified mirror baseline (the mirror's fresh
  Git history cannot contain the real canonical port-packet objects). They are
  a reproduction-environment artifact, not a candidate defect; the same test
  passes in the candidate worktree as part of the 218-test `OK` run (V2).

## 11. Checks explicitly performed and clean

Path traversal (P6), command injection and shell-wrapper bypass (P7, C1–C2),
source/worktree mutation (V13 and a final identity recheck; only `/tmp` was
written), result spoofing (the result self-labels as unreviewed, adds no
history row, and scopes the source `ACCEPT` correctly), stale identity reuse
(§1; the one stale packet binding is O1, outside the candidate), weak or
vacuous reversals (R1/R2/R11/R12 each produce large, named, targeted failure
sets), TOCTOU/symlink/hardlink behavior (P1–P5, C5, plus
`verify_output_identity`'s `(dev, ino)` re-verification and
`descriptor_log_identity`'s full stat-tuple re-check), unsafe cleanup (the
`open_exclusive_output` error path unlinks only a file whose `(dev, ino)`
matches what it created), timeout and process-tree leaks (P8: exit 124, zero
leaked descendants), false docs/history claims (§3.1 history multiset proof;
O5 is the only imprecision found), duplicated test definitions (§4: zero
duplicate tests and zero duplicate classes at all four trees), and unjustified
governance changes (none: canonical AIA-047 remains non-prerequisite, Stage 5 /
next-item 9 unchanged, recurrence-v2 intact, `cd130e5` and all blocker/status
text intact, the three top history rows intact and first).

No Cargo, rustc, Clippy, rustfmt or product compilation was executed. The
source-keyed gate's live Cargo behavior is therefore unexercised here; only its
Python-level contracts and negative controls ran. Release provenance remains
local, non-hermetic evidence.

## 12. Residual routing

| Item | Owner | Blocking for integration? |
|---|---|---|
| O1 — publish the `1bbcfde` `ACCEPT` authority; rebind the port packet (`4e5c0e7`/19 → `1bbcfde`/20) and burn-down item 1 (`74cd37d` → `1bbcfde`) | portfolio manager | **Yes — precondition.** Not repairable in this candidate. |
| O2b — raise `check_ai_development.py`'s default per-check timeout to ≥180 s, or reduce suite runtime | follow-up with authority over that file | No; use `--timeout 180` meanwhile |
| O3b — add `RUSTUP_TOOLCHAIN` + the source-gate-wrapper sentence to `cfr168-rust-http-mr-preflight-evidence-closure-repair-20260730.md` | manager / packet owner | No |
| O3a — pin `ACTIVE_RUST_PROFILE_MARKERS` with an independent literal assertion | follow-up | No |
| O3c — select Rust packets on a typed field rather than the `"cargo "` substring | follow-up | No |
| O4 — add `pdeath_exec.py` to `REQUIRED_AI_FILES` | follow-up | No |
| O2, O5 — immutable result imprecision | none; superseded by §5 of this report | No |
| Live Cargo/Rust gate execution | manager release of the Rust lane | Out of scope here |

## 13. Verdict

Every supplied identity recomputed exactly. The classification is exactly 6
ADD / 6 BASE_EQUAL / 8 SEMANTIC; all 12 ADD/BASE_EQUAL blobs are the identical
Git objects as at source head, and the 6 BASE_EQUAL paths genuinely were equal
at source base and canonical base before the copy. All eight semantic unions
preserve every canonical-only addition and carry every source addition: AIA-047
remains a non-prerequisite, recurrence-v2 is intact and byte-exact, Stage 5 and
next-item 9 are unchanged, `cd130e5` and the current blockers/status text are
untouched, the top closure/governance history rows remain first and unmodified,
and both v1 and v2 timed-gate behaviors work — with the source's Rust-dispatch
rejection correctly guarding the canonical v2 path too. The committed test set
is exactly `P`, 218 identifiers, hash
`b83e0b41c383b9265f0c9a6c32ec71ee2c2a92e049205e9502025f48f389de62`, with no
duplicates, all 43 canonical `ReviewRecurrenceTests`, and every accepted source
test. The single adapted fixture is the minimal correct response to accepted
hardening and weakens neither production behavior nor the oracle. All required
validation passes, including the exact authoritative
`check_ai_development.py --timeout 180` at rc 0. Reversals and direct probes
show both branches' controls fail closed, and Risk B's marker is latent but
armed, with the source-gate execution path separately and demonstrably
unbypassable.

No must-fix trigger fired inside the candidate's frozen 21-path boundary. Every
observation is either inherited from the accepted source range, honestly
disclosed by the immutable result, or repairable only outside that boundary.

**`OVERALL_VERDICT: ACCEPT`**

Fit for canonical integration and for supplying evidence to held Rust reviews,
subject to the O1 manager precondition in §12. This closes nothing else:
CFR-168, CFB-136, CFB-138, all product candidates, Install, Dogfood and the
lifecycle remain exactly as canonical records them.
