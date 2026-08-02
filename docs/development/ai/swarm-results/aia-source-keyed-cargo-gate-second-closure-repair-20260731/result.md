# AIA-046 Source-keyed Cargo Gate — Second Closure Repair (Claude)

Date: 2026-07-31

Implementer: Claude Opus 5 (`claude-opus-5`)

Batch: `aia-source-keyed-cargo-gate-second-closure-repair-20260731`

Repairs the `REQUEST_CHANGES` rereview at
`b589864:docs/development/ai/swarm-results/aia-source-keyed-cargo-gate-closure-rereview-claude-20260731/review.md`.

**This document does not claim acceptance.** It reports what was reproduced,
what was changed, and what was measured, for a fresh independent reviewer.

## 1. Subject identity

| Field | Value |
|---|---|
| Base (reviewed candidate) | `aaa39b61b37df273fe4d57e274ed8e1321e788ad` |
| Base tree | `a99d1703be115d7008fad884f977d5506b927db0` |
| Checkpoint ancestor | `bfbbc2b2bdc22f88679e5e1dc794665d863304a1` |
| Immutable review | `b589864:docs/development/ai/swarm-results/aia-source-keyed-cargo-gate-closure-rereview-claude-20260731/review.md` |
| Head (this repair) | recorded in section 8 |

Changed paths:

```text
docs/development/ai/source-keyed-cargo-gates.md
docs/development/ai/swarm-results/aia-source-keyed-cargo-gate-second-closure-repair-20260731/result.md
docs/development/history.md
docs/development/tools/readonly_source_exec.py
docs/development/tools/run_source_keyed_cargo_gate.py
docs/development/tools/test_ai_tools.py
```

Additive only. No file outside this list was touched, no scheduler or product
work was done, no Cargo/`rustc`/Clippy/`rustfmt` process was started, and no
Codex or subagent was used.

## 2. Every finding reproduced before editing

### MUST-1 — descriptor closure did not achieve its stated invariant

Reproduced by rewriting the live control to read the exec'd command's
*entire* inherited descriptor set from `/proc/self/fd` and require equality
with the retained set, then running it against the base:

```text
AssertionError: inherited descriptors beyond the retained set:
  [(8, '/usr/bin/unshare')]
```

The base control asserted `len(closed) == 4` over four enumerated
descriptors, so it passed while the invariant was violated. After extending
the harness to pin the same surface the real manager pins (policy
supervisor, parent-death helper, unselected toolchain drivers), the
pre-fix behaviour reproduces the reviewer's exact five leaks:

```text
AssertionError: inherited descriptors beyond the retained set:
  [(8,  '/usr/bin/unshare'),
   (14, '.../docs/development/tools/pdeath_exec.py'),
   (15, '.../docs/development/tools/run_source_keyed_cargo_gate.py'),
   (16, '/usr/bin/true'),          # unselected toolchain driver
   (17, '/usr/bin/true')]          # unselected toolchain driver
```

### MUST-2 — recorded isolation identity declared the pre-fix flags

```text
recorded unshare_flags: ['-Urnmpf', '--mount-proc', '--kill-child=SIGKILL']
launched argv:           -Urnmpf --mount-proc --kill-child=SIGKILL --propagation private
```

### MUST-3 — both halves of the reclaim claim

(a) No charge-back consumer. `grep` across `docs/development/tools/*.py` for
`reclaimed_allocated_bytes` finds only the two production sites and the
test; nothing reads it into an admission decision.

(b) Not exact. One 4 MiB payload with a second link, as Cargo's uplift
produces:

```text
st_nlink:                         2
du -s --block-size=1 (dedups):    4194304
namespace_usage allocated_bytes:  8388608
overstatement:                    4194304
```

### SHOULD-1 — capability drop is not a durable capability boundary

Reproduced independently inside the gate's own launch, not copied from the
review:

```text
after setpriv          : CapEff 0000000000000000  CapBnd 0000000000000000  NoNewPrivs 1
unshare(NEWUSER|NEWNS) rc: 0
after nested unshare   : CapEff 000001ffffffffff  CapBnd 000001ffffffffff  NoNewPrivs 1
```

Values match the review exactly.

### SHOULD-2 — test counts, with one correction to the review

The candidate count in `history.md` was wrong (157, actual 159 at review
time). **The checkpoint count in the review is an artifact of its
reproduction method**, and I did not adopt it:

```text
checkpoint bfbbc2b, extracted with git archive, NOT a git repo:
  Ran 148 tests — FAILED (failures=4, errors=8)   # 12
checkpoint bfbbc2b, same tree, git init'd:
  Ran 148 tests — FAILED (failures=4, errors=6)   # 10
```

The two extra errors are `RustBuildProvenanceTests.test_source_manifest_is_stable_and_covers_workspace_inputs`
and `test_provenance_destination_is_ignored_untracked_target_only`, both
failing with `fatal: not a git repository` from `git ls-files`. They are
environment artifacts of running the suite outside a Git worktree, not
checkpoint defects. `history.md` now records 10 and states the 12 and why it
appears.

> **Correction, added by the third closure repair.** The next clause of this
> paragraph originally read "and records the candidate count". That was
> false against the committed bytes: `4e5c0e7` wrote `161` into the row,
> which is neither the predecessor's 159 nor this candidate's 168, and the
> commit message of `4e5c0e7` separately claimed the count had been
> corrected to 159. Both statements were wrong. The row is repaired in the
> third closure repair, which binds it to `aaa39b6`/159 and `4e5c0e7`/168
> and adds `HistoryTestCountConsistencyTests` so a count matching no tree
> cannot be committed again. See
> `../aia-source-keyed-cargo-gate-third-closure-repair-20260731/result.md`,
> R1.

### NIT-1 — short malformed mountinfo line silently accepted

```text
mount_propagation_tags("24 20 0:1 / / shared:1\n") -> []
prove_private_propagation(...)                     -> ACCEPTED
```

A `len(fields) < 7: continue` guard short-circuited the reject rule the
function's own docstring states.

### NIT-2 — reuse wording overstated what is verified

Confirmed by reading `_preload_policy_module`: reuse admits a live module
whose `__file__` equals the pinned path and digests the file's *current*
bytes, not the code the interpreter executed at the earlier import.

## 3. Fixes

**MUST-1** (`readonly_source_exec.py`). The close set is now the complement
of `retained` over the already-parsed `--pass-fds` set (`inherited`) unioned
with the named descriptors, skipping stdio and tolerating `EBADF`. The
control asserts the property: it reads the whole inherited set from
`/proc/self/fd` and requires it to equal `{stdio} ∪ {primary} ∪
{environment-named}`, and the harness pins the same leak surface the real
manager does.

**MUST-2** (`run_source_keyed_cargo_gate.py`). Added
`UNSHARE_ISOLATION_FLAGS` as the single source of truth; `isolation_identity`
and `isolated_command` both read it. The launch test asserts the launched
argv slice, the published
`result["readonly_isolation_identity"]["unshare_flags"]`, and the constant
are all equal, so a recorded copy cannot drift again.

**MUST-3** — contract chosen: **document the conservative per-entry
convention accurately**, not inode de-duplication.

Rationale for the choice: de-duplicating would make measured aggregate
allocation *smaller*, loosening a 32 GiB budget whose
`MEASURED_MAX_NAMESPACE_ALLOCATED_BYTES = 3_204_700_000` sizing basis was
derived under the counting convention and cannot be re-derived without
running Cargo, which is out of scope here. The current convention errs
toward blocking earlier and the two walks agree with each other, so the
budget stays internally consistent.

Implemented consistently: the false "charged back" and "exact" claims are
gone from the doc, the docstring, and the payload. `resource_accounting_complete`
is removed. The payload now carries `reclaimed_measurement` stating the
convention, `covers_every_removed_entry`, `is_upper_bound_on_freed_allocation`
and `charged_back_against_aggregate_budget: false`, plus
`removed_hardlinked_entry_count` and
`reclaimed_allocated_bytes_counted_more_than_once`.

That last field quantifies the overstatement rather than gesturing at it.
Two subtleties surfaced while building it, both caught by the new tests
rather than by inspection:

- directories carry `st_nlink >= 2` and cannot be hard linked, so counting
  them inflated the figure; they are excluded;
- `st_nlink` *drops as the walk unlinks*, so a link count read later is not
  comparable with one read earlier. Repeats are therefore detected by
  `(st_dev, st_ino)`, which is stable. For links wholly inside the namespace
  — the Cargo uplift case — this yields the exact overstatement, verified as
  `reclaimed - counted_more_than_once == du`.

**SHOULD-1.** New explicit-boundary bullet with the measured before/after
capability table, stating that what holds the masks is the kernel's locking
of mounts copied into a child user namespace, not the capability drop.

**SHOULD-2.** `history.md` corrected as described above.

**NIT-1.** Every malformed shape now takes the same reject path; blank lines
are still tolerated and real `/proc/self/mountinfo` still parses.

**NIT-2.** Doc narrowed to "after verifying the pinned file and the module's
origin path", with an explicit paragraph on what reuse does not verify and
on the `__main__` second-instance case the mechanism cannot address.

## 4. New regressions and guards

> **Correction, added by the third closure repair.** This table was
> originally headed "New non-vacuous regressions". One row —
> `test_no_consumer_charges_reclaimed_bytes_back` — does not belong under
> that heading: it passes unchanged on the predecessor `aaa39b6`, so it
> demonstrates no closed regression. It is a forward guard. The heading and
> that row are corrected below; the test itself is sound and is retained,
> renamed to say what it is. See
> `../aia-source-keyed-cargo-gate-third-closure-repair-20260731/result.md`,
> R5.

| Test | Property |
|---|---|
| `test_readonly_helper_mounts_snapshot_and_drops_namespace_caps` (rewritten) | full inherited FD set at exec equals the retained set; harness pins supervisor, pdeath helper and unselected drivers |
| `test_gate_uses_locked_source_namespace_and_source_keyed_log` (extended) | launched argv slice == published `unshare_flags` == `UNSHARE_ISOLATION_FLAGS` |
| `test_hardlinked_reclaim_is_an_upper_bound_and_says_so` | uplift double-count reproduced; `reclaimed - counted_more_than_once == du`; payload states convention and `charged_back == false` |
| `test_forward_guard_no_consumer_charges_reclaimed_bytes_back` | **forward guard, not a regression control** — passes on `aaa39b6`; guards against a future charge-back consumer. The predecessor-failing control for that claim is `test_reclaim_is_not_described_as_exact_or_charged_back` |
| `test_unparseable_mountinfo_line_is_not_read_as_private` (extended) | four malformed shapes incl. the short `shared:` line all reject |
| `test_blank_mountinfo_lines_are_tolerated`, `test_real_mountinfo_parses` | strict reject path does not reject valid input |
| `SourceKeyedGateDocumentationTests` (5 tests) | the corrected doc claims cannot silently regress |

Negative control for MUST-1: reverting only `*inherited` from the close set
reproduces all five leaks; the control fails. Recorded in section 2.

## 5. Gates

```text
python3 docs/development/tools/test_ai_tools.py        Ran 168 tests — OK
python3 -m py_compile docs/development/tools/*.py      OK
python3 docs/development/tools/check_ai_development.py PASS x7:
    acceleration_docs, contracts, project_docs, source_docs,
    runtime, tool_tests, validation_capabilities
python3 docs/development/tools/check_ai_contracts.py       passed
python3 docs/development/tools/check_project_docs.py       passed (606 files)
python3 docs/development/tools/check_source_docs.py        passed (44/34/12)
python3 docs/development/tools/check_ai_acceleration_docs.py passed
git diff --check                                            clean
```

No Cargo, `rustc`, Clippy or `rustfmt` invocation.

## 6. Preserved boundaries

The five independently verified closures are untouched except where a
finding required strengthening:

1. policy-module reuse — logic unchanged; doc wording narrowed (NIT-2);
2. deadline-bounded cleanup — logic unchanged; accounting fields added;
3. propagation — strengthened (single constant, stricter parser);
4. candidate capture / dirty recapture — unchanged;
5. global execution lock — unchanged, still taken unconditionally before
   the registry lock at `cargo_registry_locks`; release prerequisites
   unchanged.

Descriptor closure (the sixth, previously partial) is now closed against the
property. All ten fail-closed safety classes adjudicated in review section 4
remain as adjudicated; nothing here relaxes a check.

## 7. Candid residuals

- **The severity of MUST-1 remains low, as the review said.** `/usr` is not
  masked inside the namespace, so `/usr/bin/unshare` was already reachable
  by path. The repair removes inherited descriptors, not path reachability.
  It fixes a false published invariant and a control that could not fail.
- **Reclaim remains an upper bound, deliberately.** Inode de-duplication is
  still the more accurate convention and is still not implemented. Adopting
  it requires re-measuring the sizing basis under Cargo first.
- **`reclaimed_allocated_bytes_counted_more_than_once` is exact only for
  links wholly inside the namespace.** If the other link lives outside, the
  entry is counted once and no overstatement is recorded, so the field is a
  lower bound in that case. Stated in the code comment and the payload
  semantics.
- **`prove_private_propagation` is strict about the whole namespace.** This
  host's *outer* namespace carries `master:341` on `/var/lib/docker` and
  `/vscode`; `--propagation private` clears them inside the gate, which is
  why the live test passes. On a host where the gate's own namespace
  retained a slave mount, the gate would block. That is the intended
  fail-closed direction but it is a real availability edge worth knowing.
- **The project CLI suite (`tests/`) was not completed.** It was started in
  the previous session, exceeded ten minutes, and was stopped by harness
  teardown without producing a result; it was not re-run here. It is
  disjoint from every changed path — `git status --short -- codefire tests/`
  is empty and nothing under `tests/` imports `docs/development/tools` — so
  it is reported as not-run rather than as passing.
- **No Rust evidence exists for this lane.** Cargo gates remain held for
  manager release; nothing here should be read as Rust validation.
- **`check_ai_development.py` and the capability probe create**
  `/workspace/.cache/codefire-gate-evidence` and `codefire-source-snapshots`
  (mode `0700`) if absent. Pre-existing checkpoint behaviour, not introduced
  here, but it means the doc's "create the three roots once" instruction is
  partly automated already.

## 8. Head

A commit cannot contain its own hash, so this file names its parent instead.
The head to review is the single commit whose parent is
`aaa39b61b37df273fe4d57e274ed8e1321e788ad` and which adds this file:

```bash
git log --oneline -1 --format=%H aaa39b61b37df273fe4d57e274ed8e1321e788ad..HEAD
```

Return that commit to a fresh independent reviewer. No acceptance is claimed.
