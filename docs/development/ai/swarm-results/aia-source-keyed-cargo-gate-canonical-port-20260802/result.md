# AIA-046 Source-Keyed Cargo Control — Canonical Semantic Port (2026-08-02)

Role: bounded implementer. Scope: semantically port the accepted AIA-046
source-keyed Cargo gate/control work onto canonical `41dea14` without
overwriting later canonical governance, recurrence-v2, or CFR-168
closure-rebaseline content.

**Fresh exact port review required; not yet accepted or integrated.** The
independent `ACCEPT` verdict recorded below applies to the *source* candidate
`1bbcfde` only. This canonical port is a new artifact with new semantic
decisions and has received no review.

## Exact identity

| Item | Exact value |
|---|---|
| Canonical base (HEAD parent) | `41dea1488bf04c8a2bc1c1d1ad3df7ac9b2570f6` |
| Canonical base tree | `47b6bc83350fff67386ec92c273f74c75a2d9471` |
| Canonical base branch | `integration/cfr168-p0-canonical` |
| Accepted source range | `893cbcad2b1f2a8455a5c256246c8110d3fcbdd6..1bbcfde2acc60476b5e29136a30f7c808fb6bb4e` |
| Accepted source head tree | `753c44216e0c1145fd97f33d3863c111f49eff2c` |
| Source-range binary patch SHA-256 | `03b54705f09cc83b75598560b42fecd972650e2d2e08de0f0d25c152547c2e56` |
| Independent review verdict on `1bbcfde` | `ACCEPT` (source candidate only) |
| Port worktree branch | `claude-agent/cfr168_cargo_gate_canonical_port_20260802` |

All four source-side values were recomputed in this worktree and matched the
manager-supplied values exactly. `git diff --binary 893cbcad 1bbcfde |
sha256sum` reproduced `03b54705…`; `git log -1 --format=%T 1bbcfde` reproduced
`753c4421…`; the recomputed source path set is exactly the 20 paths listed
below.

### Required correction to the stated base identity

The task statement gave the canonical base as
`41dea14b8e3bcb66de9d74fb9aee51d6ee9506ad (abbreviated 41dea14)`. That full
object **does not exist** in this repository:

```
$ git cat-file -t 41dea14b8e3bcb66de9d74fb9aee51d6ee9506ad
fatal: git cat-file: could not get object info
$ git rev-parse --disambiguate=41dea14
41dea1488bf04c8a2bc1c1d1ad3df7ac9b2570f6
$ git rev-parse integration/cfr168-p0-canonical
41dea1488bf04c8a2bc1c1d1ad3df7ac9b2570f6
```

The abbreviation `41dea14` is unique and resolves to
`41dea1488bf04c8a2bc1c1d1ad3df7ac9b2570f6`, which is also the tip of
`integration/cfr168-p0-canonical` and the subject
`docs(cfr168): rebaseline closure critical path`. The port was built on that
commit. The stated 40-character value is a transcription error in the task
authority and must be corrected before this port is reviewed against it.

The worktree was clean at launch and its HEAD equalled the canonical base.

## Three-way classification (recomputed, not assumed)

`B` = `893cbcad` (source base), `S` = `1bbcfde` (source head), `C` = `41dea14`
(canonical base).

### ADD — copied accepted source blob at `1bbcfde` (6)

Verified absent at both `B` and `C` before copying.

| Path | Source blob |
|---|---|
| `docs/development/ai/source-keyed-cargo-gates.md` | copied verbatim |
| `docs/development/ai/swarm-results/aia-source-keyed-cargo-gate-second-closure-repair-20260731/result.md` | copied verbatim |
| `docs/development/ai/swarm-results/aia-source-keyed-cargo-gate-third-closure-repair-20260731/result.md` | copied verbatim |
| `docs/development/tools/pdeath_exec.py` | copied verbatim |
| `docs/development/tools/readonly_source_exec.py` | copied verbatim |
| `docs/development/tools/run_source_keyed_cargo_gate.py` | copied verbatim |

### BASE_EQUAL — canonical still equalled source base, copied source blob (6)

Each was verified blob-identical at `B` and `C` before the copy, so no
canonical content was displaced.

| Path | `B` blob | `C` blob | equal? |
|---|---|---|---|
| `docs/development/ai/failure-cookbook.md` | `d823c68f` | `d823c68f` | yes |
| `docs/development/ai/stage5-manual-control-runbook.md` | `6dd6486b` | `6dd6486b` | yes |
| `docs/development/tools/ai_contracts.py` | `3ded0930` | `3ded0930` | yes |
| `docs/development/tools/build_source_bound_rust.py` | `f30cac14` | `f30cac14` | yes |
| `docs/development/tools/check_rust_execution_profile.py` | `b1313eb8` | `b1313eb8` | yes |
| `docs/development/tools/probe_validation_capabilities.py` | `96115fa1` | `96115fa1` | yes |

All 12 ADD/BASE_EQUAL files were confirmed byte-identical to their `1bbcfde`
blobs after copying (`git hash-object` compared against
`git rev-parse 1bbcfde:<path>`).

### SEMANTIC UNION — hand-merged, canonical never overwritten (8)

The range was never cherry-picked and none of these eight paths was
wholesale-copied from the source.

## Semantic conflict decisions

**1. `docs/development/ai/README.md`** (+4/−3)

Retained the canonical governance authority map, the AIA-047 checkpoint
section, the 14-step lifecycle, the batch preflight steps, the
`review-recurrence/README.md` row, and the `check_review_recurrence.py` row.
Ported the source Local Tools rows for `build_source_bound_rust.py` and
`check_rust_execution_profile.py` (canonical still had the base text for
both), and added the source-keyed gate / `readonly_source_exec.py` row.

The `run_timed_gate.py` row was changed by **both** branches. Merged into one
row that keeps the source's trusted-non-Cargo scope, bounded same-descriptor
log identity, live resource guard, rollback-safe exclusive publication and
rejection language **and** the canonical clean/index-state evidence and
post-command source identity.

**2. `docs/development/ai/acceleration-master-plan.md`** (+5/−4)

Kept the canonical `Stage 5 … Active 2026-07-31` heading, the canonical
AIA-047 row verbatim, and canonical next-item 9 verbatim. Replaced the
AIA-021/AIA-040/AIA-042/AIA-044 rows with their source versions (canonical
still carried the base text for all four). Inserted the new AIA-046 row
**immediately before** AIA-047. Nothing was added that makes AIA-047 a
CFR-168 product prerequisite.

**3–4. The two rereview packets** (+6/−1 and +8/−3)

Canonical had rewritten the title/status/subject table, added the bound
candidate `cd130e5`, its zero-test-and-provenance pre-launch blocker, and the
candidate-card reference. All of that is preserved untouched. The source
hunks are disjoint from the canonical hunks and were applied as-is: the
`RUSTUP_TOOLCHAIN=1.97.0-x86_64-unknown-linux-gnu` export and the
source-wrapper-only requirement (Cargo must be the logical argv after `--` to
`run_source_keyed_cargo_gate.py`). The only canonical lines replaced are the
three-and-one prose lines the source rewrote in place, which retain their
"do not create a worktree-local target or another Rustup/Cargo home"
requirement.

**5. `docs/development/history.md`** (+3/−0)

Canonical's three top rows (2026-08-02 closure rebaseline, 2026-07-31
governance repair, 2026-07-31 AIA-047 integration) are unchanged and remain
first in that order. The three accepted source rows (Cargo MUST closure,
provider-readiness retry evidence, Cargo/disk safety — all 2026-07-31) were
inserted verbatim after them and before the 2026-07-30 block, which is their
correct reverse-chronological position. Verified: 549 → 552 lines, every
canonical line still present, each source row present exactly once.

**No history row was added for this port**, and no row claims it is accepted.

**6. `docs/development/tools/check_ai_acceleration_docs.py`** (+4/−0)

Pure union; the two branches' additions are disjoint. Canonical's
`review-recurrence-control.v1.schema.json`, `review-recurrence/README.md` and
`check_review_recurrence.py` requirements are preserved. Added the source's
`source-keyed-cargo-gates.md`, `run_source_keyed_cargo_gate.py` and
`readonly_source_exec.py` required files and the
`RUSTUP_TOOLCHAIN=1.97.0-x86_64-unknown-linux-gnu` profile marker. See the
limitation on that marker below.

**7. `docs/development/tools/run_timed_gate.py`** (+1475/−56)

The source rewrote this module (326 → 1610 lines: pinned FD-delivered
launchers, parent-death arming, PID-namespace isolation, protected Rust
dispatch rejection, bounded JSON, exclusive no-follow output creation,
runtime resource guard, atomic-temporary recovery). Canonical had added a
purely additive recurrence-v2 layer on top of the unchanged base.

The merge takes the source module and grafts the canonical layer verbatim:
`TIMED_V2_LIMITS`, `run_gate_v2`, the `--emit {v1,v2}` CLI option, the
`--emit v2 requires --result` guard, and the v1/v2 runner dispatch in
`main()`. The source's `protected_rust_dispatch` rejection in `main()` is
kept and runs **before** the emission dispatch. Canonical's `run_gate_v2`
calls the source `run_gate` unmodified — the source kept the four positional
parameters and the three keyword parameters `run_gate_v2` passes.

Top-level symbol union verified mechanically: 51 symbols, zero missing from
either branch, zero invented.

> **Conflict resolved in favour of the source hardening.** The base
> `run_gate` silently created the log's parent directory
> (`log_path.parent.mkdir(parents=True, exist_ok=True)`). The source replaced
> that with `open_exclusive_output`, which requires an existing, symlink-free
> parent and creates the log `O_EXCL|O_NOFOLLOW`. The hardening was kept. One
> canonical test incidentally depended on the implicit `mkdir`; its fixture
> now creates the in-tree `evidence/` directory itself. The test's actual
> subject — `log_inside_source_worktree`, `result_inside_source_worktree`,
> `index_flags_clean` — is unchanged and still asserted.

**8. `docs/development/tools/test_ai_tools.py`** (+5307/−72)

Preamble imports unioned per statement. `TimedGateTests` was changed by both
branches: the merged class is the source's 22 methods (including the source's
strengthened `test_gate_uses_monotonic_duration_and_binds_complete_log`) plus
canonical's `test_v2_cli_requires_an_explicit_result_path`. Canonical's
`ReviewRecurrenceTests` is retained whole — **43 tests, matching the recorded
count at the base**. The four new source classes
(`PolicyModulePreloadTests`, `HistoryTestCountConsistencyTests`,
`SourceKeyedGateDocumentationTests`, `SourceKeyedCargoGateTests`) were added,
and `CapabilityProvenanceTests`, `RustBuildProvenanceTests` and
`RustExecutionProfileTests` take their source versions (canonical still had
the base text for all three).

## Test-set formula, count and hash

```
P = (C ∪ (S \ B)) \ (B \ S)
```

Qualified `Class.test_method` identifiers, extracted by AST from each tree's
own `test_ai_tools.py`:

| Set | Count |
|---|---|
| `B` = `893cbcad` | 65 (20 classes) |
| `S` = `1bbcfde` | 174 (24 classes) |
| `C` = `41dea14` | 109 (21 classes) |
| `S \ B` (added by source) | 110 |
| `C \ B` (added by canonical) | 44 |
| `B \ S` (deleted by source) | 1 |
| `(S \ B) ∩ (C \ B)` | 0 |
| **`P` (computed)** | **218 (25 classes)** |

`B \ S` is exactly
`RustExecutionProfileTests.test_shared_low_memory_process_passes`, deleted by
the accepted source and therefore absent from the port. It is not in
`C \ B`, so removing it discards no canonical work.

218 = 109 + 110 − 1.

- **`P` SHA-256** (newline-joined sorted identifiers):
  `b83e0b41c383b9265f0c9a6c32ec71ee2c2a92e049205e9502025f48f389de62`
- The committed module's identifier set was re-extracted and compared to the
  computed `P`: **exact match, 0 missing, 0 unexpected, 0 duplicates**. Each
  test definition occurs exactly once.
- The predicted count was **not** taken on trust; it was calculated here and
  then confirmed against the runtime count below.
- Per-class: `ReviewRecurrenceTests` 43, `TimedGateTests` 23.

## Commands and results

Sequential, foreground, in this worktree. No Cargo, rustc, Clippy, rustfmt or
product CLI test was run.

| Command | Result |
|---|---|
| `python3 -m unittest test_ai_tools.{SourceKeyedCargoGateTests,SourceKeyedGateDocumentationTests,HistoryTestCountConsistencyTests,PolicyModulePreloadTests,TimedGateTests,ReviewRecurrenceTests,RustExecutionProfileTests,RustBuildProvenanceTests,CapabilityProvenanceTests}` | **OK — 163 tests, 68.0 s** |
| `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest docs/development/tools/test_ai_tools.py` | **OK — 218 tests, 72.7 s** (equals \|P\|) |
| `PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile docs/development/tools/*.py` | exit 0 (34 files) |
| `python3 docs/development/tools/check_ai_development.py` | **FAIL** at the default 60 s per-check timeout — see limitations |
| `python3 docs/development/tools/check_ai_development.py --timeout 600` | exit 0 — PASS ×7 |
| `python3 docs/development/tools/check_ai_acceleration_docs.py` | exit 0 |
| `python3 docs/development/tools/check_ai_contracts.py` | exit 0 |
| `python3 docs/development/tools/check_project_docs.py` | exit 0 — 682 Markdown files inventoried |
| `python3 docs/development/tools/check_source_docs.py` | exit 0 — 44 modules, 34 commands, 12 object kinds |
| `python3 docs/development/tools/validate_ai_runtime.py --json` | exit 0, `ok: true` |
| `python3 docs/development/tools/probe_validation_capabilities.py --json` | exit 0, `ok: true` |
| `git diff --check` | exit 0, no output |

Component checkers named by `check_ai_development.py` output —
`acceleration_docs`, `contracts`, `project_docs`, `source_docs`, `runtime`,
`tool_tests`, `validation_capabilities` — were each additionally run
standalone; all pass.

## Reversal and non-vacuity checks

All performed in a throwaway out-of-tree mirror at `/tmp/portrev/cand`
(candidate content copied out, `git init`, object alternates to the shared
store). **The candidate was never modified by any reversal.** Each reversal
was reverted before the next.

| # | Reversal | Expected | Observed |
|---|---|---|---|
| R1 | Make source `protected_rust_dispatch` always return `None` | source Rust-dispatch control fails | **43 failures** across `test_direct_and_known_indirect_rust_dispatch_are_rejected` (42 argv subtests) and `test_symlink_alias_to_protected_tool_is_rejected` |
| R2 | Drop the canonical index-flag union in `run_gate_v2` | canonical recurrence-v2 control fails | **2 failures**: `test_v2_timed_runner_marks_in_tree_outputs_and_index_flags_nonpassing`, `test_v2_timed_runner_tracks_head_switch_and_transient_index_flags` |
| R3 | Change the ported history row `159 test全pass` → `161 test全pass` | history-count guard fires on the exact historical recurrence | **2 failures**: `test_history_states_no_test_count_matching_no_tree`, `test_history_states_the_right_count_for_each_named_commit` |
| R5 | Delete `check_review_recurrence.py` | acceleration checker blocks | exit 1, 1 error |
| R6 | Delete `source-keyed-cargo-gates.md` | acceleration checker blocks | exit 1, 1 error |
| R7 | Delete `run_source_keyed_cargo_gate.py` | acceleration checker blocks | exit 1, 1 error |
| R8 | Delete `readonly_source_exec.py` | acceleration checker blocks | exit 1, 1 error |
| R4/R9 | Perturb / remove the `RUSTUP_TOOLCHAIN` profile marker | acceleration checker or a test blocks | **did not fire — see limitation 2** |
| — | Unmodified mirror baseline | pass | `Ran 27 tests … OK`; acceleration checker exit 0 |

R1 and R2 together show the port preserves *both* branches' controls
non-vacuously: the source Cargo/dispatch control and the canonical
recurrence-v2 Git-identity control each still fail closed when reversed.

## Limitations and open items

1. **`check_ai_development.py` now exceeds its default per-check timeout.**
   The unioned suite runs 218 tests in ~73 s; the gate's default is 60 s, so
   the bare invocation reports `FAIL tool_tests / timeout after 60.0s` even
   though the suite passes. With `--timeout 600` all seven components pass.
   The source suite (168 tests) and the canonical suite (109 tests) each fit
   under 60 s; only the union does not. `check_ai_development.py` is **not**
   in this port's 21 allowed paths, so I did not change its default. A
   follow-up with authority over that file must either raise the default or
   reduce suite runtime. **Reviewers running the bare command will see a
   failure.**

2. **The ported `RUSTUP_TOOLCHAIN` marker is currently inert against the real
   tree.** `check_active_rust_packet_profiles` only inspects
   `swarm-batches/cfr168-rust-*.md` whose text matches
   `^Status: (?:blocked|ready)\b` *and* contains `## Validation` *and*
   `cargo `. At source head `1bbcfde`, the two rereview packets matched and
   carried the marker, so it was enforced there. On canonical `41dea14`,
   later governance work rewrote both packets' Status lines
   (`Status: exact candidate bound; …` and `Status: candidate-bound; …`), so
   **zero** packets in the ported tree satisfy all three filters. Separately,
   `RustPacketProfileTests` builds its fixture *from*
   `ACTIVE_RUST_PROFILE_MARKERS`, so removing an entry from the tuple cannot
   fail it. The marker's enforcement logic is tested; this specific marker's
   live coverage is not. I did **not** rewrite the canonical Status lines to
   restore matching — that would overwrite canonical governance content,
   which is outside this port's authority. Flagged for manager disposition.

3. `apply_patch` is not available in this harness. Edits were made with
   targeted string replacements and verified byte-exact splices of accepted
   source segments; no semantic path was wholesale-copied, and every union
   was verified mechanically (canonical-line-preservation checks, AST symbol
   and test-identifier set comparison).

4. No Cargo, rustc, Clippy, rustfmt or product CLI test was executed, per
   scope. The source-keyed gate's *live* Cargo behavior is therefore
   unexercised here; only its Python-level contracts and negative controls
   ran. Release provenance remains local non-hermetic evidence.

5. `docs/development/ai/swarm-batches/cfr168-rust-source-keyed-recovery-wave-binding-20260731.md`
   is `Status: blocked until an accepted Cargo canonical port is integrated`.
   This artifact does **not** satisfy that condition: it is an unreviewed
   candidate, not an accepted integration.

6. The port preserves canonical's AIA-047 status exactly as found (L1.5
   hardening reopened, L1.6 blocked). Nothing here is evidence about AIA-047,
   review-bottleneck improvement, or CFR-168 closure progress.

## Changed paths (21)

20 source paths (6 ADD, 6 BASE_EQUAL, 8 SEMANTIC UNION) plus this result
file. Verified that exactly these paths changed and no others.

```
A docs/development/ai/source-keyed-cargo-gates.md
A docs/development/ai/swarm-results/aia-source-keyed-cargo-gate-second-closure-repair-20260731/result.md
A docs/development/ai/swarm-results/aia-source-keyed-cargo-gate-third-closure-repair-20260731/result.md
A docs/development/ai/swarm-results/aia-source-keyed-cargo-gate-canonical-port-20260802/result.md
A docs/development/tools/pdeath_exec.py
A docs/development/tools/readonly_source_exec.py
A docs/development/tools/run_source_keyed_cargo_gate.py
M docs/development/ai/README.md
M docs/development/ai/acceleration-master-plan.md
M docs/development/ai/failure-cookbook.md
M docs/development/ai/stage5-manual-control-runbook.md
M docs/development/ai/swarm-batches/cfr168-rust-file-doctor-review-note-closure-rereview-claude-20260730.md
M docs/development/ai/swarm-batches/cfr168-rust-http-mr-preflight-evidence-closure-rereview-claude-20260730.md
M docs/development/history.md
M docs/development/tools/ai_contracts.py
M docs/development/tools/build_source_bound_rust.py
M docs/development/tools/check_ai_acceleration_docs.py
M docs/development/tools/check_rust_execution_profile.py
M docs/development/tools/probe_validation_capabilities.py
M docs/development/tools/run_timed_gate.py
M docs/development/tools/test_ai_tools.py
```

## Status

One additive commit on `claude-agent/cfr168_cargo_gate_canonical_port_20260802`,
parented directly on `41dea1488bf04c8a2bc1c1d1ad3df7ac9b2570f6`. The canonical
worktree was not modified and nothing was merged.

**Fresh exact port review required; not yet accepted or integrated.**
