# AIA-046 Source-keyed Cargo Gate — Second Closure Rereview (Claude)

Review date: 2026-07-31

Reviewer: Claude Opus 5 (`claude-opus-5`)

Batch: `aia-source-keyed-cargo-gate-second-closure-rereview-claude-20260731`

Subject: candidate `4e5c0e7f058cd406cb7a183ecda8b3a07f3ee912`, additive over predecessor
`aaa39b61b37df273fe4d57e274ed8e1321e788ad`, which is itself additive over checkpoint
`bfbbc2b2bdc22f88679e5e1dc794665d863304a1`.

Immutable prior report:
`b589864:docs/development/ai/swarm-results/aia-source-keyed-cargo-gate-closure-rereview-claude-20260731/review.md`.

**Verdict: `REQUEST_CHANGES`.**

Six of the seven bound findings are genuinely closed, and every one of them is closed against a
property rather than an enumeration — I re-derived each closure with my own oracle, run against the
predecessor tree first to prove the oracle is not vacuous. `MUST-1`, `MUST-2`, `MUST-3`, `SHOULD-1`,
`NIT-1` and `NIT-2` are real closures, and `MUST-1` in particular is now stronger than the prior
report's own recommendation. The implementer's dispute of the prior report's checkpoint failure
count is correct and I confirm it: in an actual Git worktree the checkpoint fails **10** of 148, not
12.

The change is not acceptable as written for one reason: `SHOULD-2` — "the permanent record misstates
both test counts" — is **not closed**. The repair corrected the checkpoint number (and correctly
disputed it) and then wrote a *third* wrong candidate number into the same row: `history.md:16` now
says `候補は161件全pass` for a predecessor with 159 tests and a candidate with 168. The commit
message asserts "history.md said 157 for a 159-test candidate; corrected," and `result.md` asserts
"`history.md` … records the candidate count." Both statements are false against the committed bytes.
This is exactly the claim-versus-implementation class this lineage exists to close, re-created inside
the fix for it. Four `NIT`s and one pre-existing `NIT` follow.

This verdict was derived independently. Every implementer-reported figure was treated as a claim and
re-run. No subagent, Codex or Fugu session was used; no implementation, test or management file was
modified; no Cargo, `rustc`, Clippy or `rustfmt` process was started, and the disjoint product CLI
suite under `tests/` was not run, per the review scope.

## 1. Recomputed subject identity and clean state

| Claim | Recomputed | Match |
|---|---|---|
| Candidate | `git rev-parse HEAD` → `4e5c0e7f058cd406cb7a183ecda8b3a07f3ee912` | yes |
| Candidate tree | `git rev-parse HEAD^{tree}` → `6417400cb7b5a3507f85f38d66c0573992ad4007` | recorded |
| Predecessor is the parent | `git rev-parse HEAD^` → `aaa39b61b37df273fe4d57e274ed8e1321e788ad` | yes |
| Checkpoint is an ancestor | `git merge-base --is-ancestor bfbbc2b HEAD` → true | yes |
| Delta | `git diff --shortstat aaa39b6 4e5c0e7` → `6 files changed, 818 insertions(+), 92 deletions(-)` | yes |
| Delta binary-patch SHA-256 | `git diff --binary aaa39b6 4e5c0e7 \| sha256sum` → `3aecdf035e3717f9931fad6987cdf417a2cb369d1713c5ea526d92704a31a2ec` | recorded |
| Worktree state at review start | `git status --porcelain` empty | yes |

Changed paths, all six reviewed in full:

```text
docs/development/ai/source-keyed-cargo-gates.md
docs/development/ai/swarm-results/aia-source-keyed-cargo-gate-second-closure-repair-20260731/result.md
docs/development/history.md
docs/development/tools/readonly_source_exec.py
docs/development/tools/run_source_keyed_cargo_gate.py
docs/development/tools/test_ai_tools.py
```

File digests at candidate HEAD:

```text
0aea6caaaa554fb8b74d4afaddda95a1c2bc406757e0b04c4e21932680fc8263  docs/development/tools/run_source_keyed_cargo_gate.py
1b1940e666904ef8834d4c3ebbff2c9f1f5bcdd2d806275e58d43a63ffcf0666  docs/development/tools/readonly_source_exec.py
c0194c7a7ed8b84f2ee48e30deac91d7e5191a8ae426c8fe70062e33413caf1a  docs/development/tools/test_ai_tools.py
43c4d3333bd24d2b0c9c6ff0d84fd22a3f80c682d1f05446b019c6b64592527a  docs/development/ai/source-keyed-cargo-gates.md
```

Nothing in the repository was modified during this review. All adversarial work ran from `/tmp`
(`probe_fdset.py`, `probe_hardlink2.py`, `probe_mountinfo.py`, `probe_identity.py`, `probe_caps.py`,
`probe_analyzer.py`) and against two throwaway detached `git worktree` checkouts under `/tmp`.
`git status --porcelain` is empty apart from this report.

Sources read in full before any finding: the complete candidate delta and the surrounding
implementation regions each hunk depends on; the implementer's `result.md` (294 lines); the immutable
prior review (361 lines); `docs/development/ai/source-keyed-cargo-gates.md` at candidate HEAD; the
root-cause audit
`abfee6e:docs/development/ai/swarm-results/cfr168-review-recursion-root-cause-audit-claude-20260731/audit.md`,
sections `I1`, `I2` and `I4`, whose obligations this review discharges explicitly in sections 2, 6
and 7.

## 2. `I1` — predecessor and checkpoint behaviour re-derived in a real Git worktree

The audit's `I1` requires predecessor behaviour to come from the predecessor tree, not from the
candidate's account of it, and the prior report's checkpoint figure is exactly the kind of number
`I1` exists to protect. I reconstructed both trees with `git worktree add --detach` — real Git
worktrees, with `.git` linkage intact — and additionally reproduced the prior report's own method to
find the discrepancy.

```text
git worktree add --detach /tmp/ckpt-wt bfbbc2b   # real Git worktree
  python3 -m unittest test_ai_tools → Ran 148 tests — FAILED (failures=4, errors=6)   # 10

git archive bfbbc2b | tar -x -C /tmp/ckpt-arch   # non-Git extraction (prior report's method)
  python3 -m unittest test_ai_tools → Ran 148 tests — FAILED (failures=4, errors=8)   # 12
```

The set difference is exactly two, and exactly the two the implementer names:

```text
ERROR: RustBuildProvenanceTests.test_provenance_destination_is_ignored_untracked_target_only
ERROR: RustBuildProvenanceTests.test_source_manifest_is_stable_and_covers_workspace_inputs
```

Both fail on `git ls-files` with `fatal: not a git repository`. They are artifacts of the
reproduction environment, not checkpoint defects.

**Adjudication: the implementer's correction is right and the prior report's `12` is wrong.** The
checkpoint has **10** genuine failures out of 148. The ten, from the real worktree, are:

```text
FAIL  SourceKeyedCargoGateTests.test_gate_uses_locked_source_namespace_and_source_keyed_log
FAIL  RustBuildProvenanceTests.test_provenance_publish_rolls_back_on_parent_fsync_failure
FAIL  SourceKeyedCargoGateTests.test_result_is_reserved_before_gate_and_survives_publish_error
FAIL  SourceKeyedCargoGateTests.test_unexpected_failure_is_durably_recorded_as_block
ERROR SourceKeyedCargoGateTests.test_artifact_is_not_published_before_final_pass_predicates
ERROR SourceKeyedCargoGateTests.test_block_decision_is_atomically_written_when_result_path_is_safe
ERROR SourceKeyedCargoGateTests.test_candidate_change_after_running_transition_is_cleaned
ERROR SourceKeyedCargoGateTests.test_candidate_change_during_gate_invalidates_success
ERROR SourceKeyedCargoGateTests.test_incomplete_namespace_over_entry_cap_is_recovered
ERROR CapabilityProvenanceTests.test_valid_release_provenance_marks_only_release_binary_bound
```

Predecessor `aaa39b6` in its own real worktree: 159 tests defined, suite green. Candidate: 168 tests
defined, suite green. These three counts (10 / 159 / 168) are the ground truth against which
`SHOULD-2` is judged in section 5.

## 3. Independent verification of the three MUST closures

### MUST-1 — descriptor closure now achieves the stated invariant

`readonly_source_exec.py:928`–`:975`; contract text at `source-keyed-cargo-gates.md:175`–`:185`.

I did not evaluate the candidate's own control. I wrote `/tmp/probe_fdset.py`, which drives the real
helper through the real `unshare -Urnmpf --mount-proc --kill-child=SIGKILL --propagation private`
launch and the real `setpriv` drop, dumps the exec'd command's **complete** `/proc/self/fd` map, and
adjudicates it in the *parent* against an oracle I state independently of the implementation:

```text
open(exec) == {0,1,2} ∪ {primary_fd} ∪ { N : some environment value == "/proc/self/fd/N" }
```

The oracle never calls `descriptors_retained_by_environment` and never reads any candidate variable;
the retained set is computed from the environment dictionary the probe itself constructs. The probe
pins a strictly larger leak surface than the candidate's own harness: `unshare`, `mount`, `setpriv`,
the pinned Python, the helper, the parent-death helper, the policy supervisor, **`run_timed_gate.py`**,
**four** unselected toolchain drivers, two selected drivers named through `RUSTC`/`RUSTDOC`, seven
directory descriptors, three control descriptors, and **two arbitrary extra pass-fds that nothing
names**. It also plants two shapes the retention rule must not be fooled by
(`/proc/self/fd/notanumber`, `/proc/self/fd/<n>/child`).

Candidate `4e5c0e7`:

```json
{"verdict": "RETAINED-SET-EXACT",
 "expected_retained": [0, 1, 2, 8, 16, 17],
 "observed_open": {"0": "/dev/null", "1": "pipe:[…]", "2": "pipe:[…]",
                   "8": "/usr/bin/python3.10", "16": "/usr/bin/true", "17": "/usr/bin/true"},
 "leaked": [], "missing_retained": []}
```

Predecessor `aaa39b6`, same probe, same oracle — **ten** leaks, a superset of the prior report's
five:

```text
LEAK [3,  'unshare',                   '/usr/bin/unshare']
LEAK [9,  'pdeath-helper',             '.../pdeath_exec.py']
LEAK [10, 'policy-supervisor',         '.../run_source_keyed_cargo_gate.py']
LEAK [11, 'timed-gate',                '.../run_timed_gate.py']
LEAK [12..15, four unselected toolchain drivers, '/usr/bin/true']
LEAK [18, 'extra-a'], LEAK [19, 'extra-b']   # arbitrary extras nothing names
```

`missing_retained` is empty on the candidate, so nothing the child needs was closed — including the
two environment-named toolchain descriptors, and including the case where the *same* file is pinned
twice under different names. No descriptor is closed twice: the loop iterates a `set` union, guards
with an accumulating `closed` set, skips `{0,1,2}` without recording them, and tolerates `EBADF`
only. `pinned_fd()` has already `fstat`ed every `--pass-fds` member at parse time
(`readonly_source_exec.py:774`), so the `EBADF` arm is defensive rather than load-bearing.

The doc's step 9 was rewritten from the false "retaining only the descriptors the child environment
itself names" to a description of the mechanism actually implemented. I confirmed that the
descriptors this change *newly* closes relative to `aaa39b6` — `directory:runtime-home`,
`directory:runtime-scratch`, `directory:runtime-tmp`, `isolation:policy-supervisor`,
`isolation:pdeath-helper`, `isolation:unshare`, and the unselected toolchain drivers — are not needed
by the gated command: `HOME` and `TMPDIR` are projected as ordinary paths
(`run_source_keyed_cargo_gate.py:7304`–`:7308`), and the only environment values of the form
`/proc/self/fd/<n>` are the selected toolchain drivers (`:7297`–`:7302`).

**Closed, and the closure is stronger than the prior report's prescription.** See NIT-3 for the one
remaining directional gap in the *prose*.

### MUST-2 — one authoritative flag sequence feeds argv, identity, both keys and the result

`run_source_keyed_cargo_gate.py:457`–`:463` (`UNSHARE_ISOLATION_FLAGS`), `:2602`
(`isolation_identity`), `:2967` (`isolated_command`).

`/tmp/probe_identity.py` builds the launch argv through the real `isolated_command` with synthetic
descriptors and compares three things it obtains by three different routes:

```json
{"constant":                       ["-Urnmpf","--mount-proc","--kill-child=SIGKILL","--propagation","private"],
 "recorded_in_isolation_identity": ["-Urnmpf","--mount-proc","--kill-child=SIGKILL","--propagation","private"],
 "launched_argv_slice":            ["-Urnmpf","--mount-proc","--kill-child=SIGKILL","--propagation","private"],
 "all_three_equal": true, "propagation_private_present": true,
 "identity_sha256": "2742ef85c2f92e97b7f3b76d0cb5063ac9831e9133bd680b73c6cdd8c6818eb7"}
```

The single `isolation = isolation_identity()` call at `:6969` is the only producer, and it feeds all
three consumers: the source key payload (`build_source_identity`, `:3070`), the artifact key payload
(`build_artifact_identity`, `:3096`), and the published result JSON (`:8007`). There is no second
literal of these flags anywhere in the gate module. `build_source_bound_rust.py:496` and
`run_timed_gate.py:1051` keep their own flag lists, but those launchers are not described by
`readonly_isolation_identity`, so they are correctly out of scope — `run_timed_gate`'s list is a
different one (`-Urpf`) for a different purpose.

**Closed.** The recorded identity, the launched argv, both keys and every published result now answer
"was propagation pinned?" with the post-fix answer.

### MUST-3 — the accounting contract is internally consistent, conservative, and honestly reported

`run_source_keyed_cargo_gate.py:5012`–`:5029` (docstring), `:5110`–`:5132` (walk),
`:5220`–`:5224` (marker), `:5978`–`:6008` (payload); doc at `source-keyed-cargo-gates.md:365`–`:386`.

The contract the candidate chose — document the conservative per-entry convention rather than
de-duplicate — is the right call, and its rationale holds: de-duplication makes measured allocation
smaller, which loosens a 32 GiB budget whose `MEASURED_MAX_NAMESPACE_ALLOCATED_BYTES = 3_204_700_000`
sizing basis was derived under the counting convention and cannot be re-derived without running
Cargo. I verified the four properties that make that choice safe.

**(a) Cannot loosen the budget.** A repository-wide grep across `*.py`, `*.md`, `*.json` and `*.rs`
finds `reclaimed_allocated_bytes` only at its two production sites
(`cleanup_namespace_locked:5997`, `cleanup_snapshot:5831`), in `test_ai_tools.py`, and in the
implementer's own `result.md`. No admission path reads it. Every budget comparison
(`:6154`, `:6162`, `:7409`, `:7528`, `:7661`, `:7770`) re-measures the aggregate through
`inspect_registry`/`namespace_usage_at`, which uses the identical per-entry convention. The only
Python outside `docs/development/tools/` is `setup.py` and the disjoint `tests/` suite; neither
touches gate result JSON. The removed `resource_accounting_complete` /
`resource_accounting_source` keys have no consumer in `schemas/`, `crates/`, `codefire/` or `docs/`,
so dropping them breaks nothing.

**(b) Internally consistent, upper-bounded, and correct across the shapes the task names.**
`/tmp/probe_hardlink2.py` runs seven independent namespace shapes through the real
`namespace_usage` and the real `remove_tree_contents_at`, and compares both against `du -s
--block-size=1` (which de-duplicates by inode):

| case | du | aggregate scan | removal walk | hardlinked entries | counted more than once | walk − dup == du |
|---|---|---|---|---|---|---|
| two links inside | 1,052,672 | 2,101,248 | 2,101,248 | 2 | 1,048,576 | yes |
| three links inside | 1,052,672 | 3,149,824 | 3,149,824 | 3 | 2,097,152 | yes |
| five links inside | 528,384 | 3,149,824 | 3,149,824 | 6 | 2,621,440 | yes |
| reverse encounter order | 1,052,672 | 2,101,248 | 2,101,248 | 2 | 1,048,576 | yes |
| link outside namespace | 1,052,672 | 1,052,672 | 1,052,672 | 1 | 0 | yes |
| directories only (`st_nlink ≥ 2`) | 8,192 | 8,192 | 8,192 | **0** | 0 | yes |
| hard link to the namespace marker | 4,096 | 8,192 | 8,192 | 1 | **0** | **no** |

In every case the removal walk equals the aggregate scan (the property that keeps the budget
self-consistent) and is ≥ `du` (the upper-bound property). Directories are correctly excluded despite
carrying `st_nlink ≥ 2`. Reversing which link the walk meets first changes nothing, which confirms
the `(st_dev, st_ino)` repeat detection is doing the work rather than an `st_nlink` reading that
would be invalidated by the walk's own unlinking. The out-of-namespace case reports `0` overcount and
`1` hard-linked entry, which is exactly the lower-bound behaviour the payload comment declares.

The last row is the one exception and is reported as NIT-1.

**(c) Makes no charge-back or exactness claim.** "charged back against the aggregate budget" and
"exact" are gone from the doc, the docstring and the payload. The payload now carries
`reclaimed_measurement` with `convention`, `covers_every_removed_entry`,
`hardlinked_inodes_counted_once_per_link`, `is_upper_bound_on_freed_allocation` and
`charged_back_against_aggregate_budget: false`. The doc states the convention, the direction of the
error, the reason the convention is deliberate, and the precondition for changing it. That is an
honest overcount report.

**(d) No behavioural regression in removal.** The added code is pure accounting: it sits between the
`st_blocks` accumulation and the `child_relative` computation and touches no `unlink`, `rmdir`,
deadline or entry-cap decision. `remove_tree_contents_at` remains deadline-bounded only, with the
entry cap applied solely when a caller passes one, so closure 2 is intact.

**Closed, with one narrow inexactness (NIT-1).**

## 4. Independent verification of SHOULD-1, NIT-1 and NIT-2

**SHOULD-1 — capability-drop boundary.** `/tmp/probe_caps.py` builds the gate's own shape (private
namespace, read-only bind over a source tree, tmpfs mask), drops with the gate's exact `setpriv`
argument list, then attempts regain and mount subversion. Measured here, independently of both the
prior report and the implementer:

```text
BEFORE  CapPrm 0000000000000000  CapEff 0000000000000000  CapBnd 0000000000000000  NoNewPrivs 1
unshare(CLONE_NEWUSER|CLONE_NEWNS) rc 0
AFTER   CapPrm 000001ffffffffff  CapEff 000001ffffffffff  CapBnd 000001ffffffffff  NoNewPrivs 1
umount mask                EINVAL
umount mask MNT_DETACH     EINVAL
umount source              EINVAL
remount read-only bind rw  EPERM,  subsequent write EROFS
```

Every figure in the new boundary bullet at `source-keyed-cargo-gates.md:436`–`:454` matches what I
measured, including the attribution: what holds the masks is the kernel's locking of mounts copied
into a child user namespace, not the capability drop. **Closed and accurate.**

**NIT-1 (prior) — malformed mountinfo.** `/tmp/probe_mountinfo.py` runs 21 shapes through
`mount_propagation_tags` and `prove_private_propagation`, plus the host's live
`/proc/self/mountinfo`. On the candidate every shape is correct: all four malformed families reject
(short-with-`shared:`, short-without, single token, long-without-separator), a separator appearing
before index 6 rejects, a mount point literally named `-` and a root field named `-` both still
surface their `shared:` tag, `unbindable` and a filesystem type spelled `shared:1` after the
separator are correctly *not* propagation tags, blank and whitespace-only lines are tolerated, and
real kernel mountinfo parses. On the predecessor the same probe reports `INCORRECT:
['short-with-shared', 'short-no-shared', 'single-token']`. **Closed.**

**NIT-2 (prior) — policy-module reuse wording.** Reading `_preload_policy_module`
(`run_source_keyed_cargo_gate.py:300`–`:365`), reuse admits a live module when `existing.__file__`
equals the pinned lexical path, and the digest recorded in the isolation identity is of the pinned
file's current bytes. The new doc text at `:236` and `:241`–`:248` says exactly that — "after
verifying the pinned file and the module's origin path", "not of the code the interpreter actually
executed" — and records both the same-UID precondition and the `__main__` second-instance case the
mechanism cannot address. **Closed.**

## 5. Findings

### MUST-1 (this review) — `SHOULD-2` is not closed: the permanent record carries a third wrong test count, and two documents claim it was corrected

`docs/development/history.md:16`. Created by `4e5c0e7`.

The row now reads, in the same cell the prior report faulted:

```text
checkpoint `bfbbc2b`はGit worktree内で148 test中10件fail（failures=4, errors=6）。
非Git展開（`git archive`）で走らせるとprovenance test 2件が`git ls-files`不能で
追加errorとなり12件に見えるが、これは再現環境の産物でcheckpointの欠陥ではない。
候補は161件全pass。
```

The checkpoint half is now right, and its dispute of the prior report's `12` is right — I confirmed
both in section 2, and the explanation of where `12` comes from is a genuine improvement over what
the prior report asked for. The candidate half is wrong. Measured in this review:

```text
tests defined at aaa39b6 (predecessor, the candidate this row describes):  159
tests defined at 4e5c0e7 (this candidate):                                 168
history.md:16 states:                                                      161
```

`161` is neither. The row does not say which candidate it means, and no reading makes it correct.
The prior report's `SHOULD-2` was "the permanent record misstates both test counts"; one of the two
is still misstated, by a different wrong number.

Two documents affirmatively claim otherwise. The commit message of `4e5c0e7` says "history.md said
157 for a 159-test candidate; corrected." It was not corrected to 159. `result.md` §2 says
"`history.md` now records 10, states the 12 and why it appears, **and records the candidate count**"
— the first two are true, the third is false against the committed bytes.

Failure scenario: a reader auditing this lineage from the permanent record reconciles `161` against
either tree, finds no match, and cannot tell whether the row describes `aaa39b6` (159), `4e5c0e7`
(168), or an uncommitted intermediate state. The closure claim in the commit message compounds it:
the one artifact that survives the lineage says the number was fixed.

Product severity is nil — no code, gate, key or decision depends on this figure, and I say so
plainly. I classify it MUST rather than SHOULD, above the prior report's rating of the same defect,
for two reasons that are about this repair rather than about the underlying number: it is a bound
finding re-created inside its own fix (`BOUND_FINDING_NOT_CLOSED` compounding
`REPAIR_INTRODUCED_REGRESSION` in the audit's taxonomy), and both the commit message and the result
document assert a closure that the bytes do not contain. That is precisely the standard this lineage
sets for itself and the standard on which `MUST-1`, `MUST-2` and `MUST-3` of the prior report were
raised.

Smallest correct fix: replace `161` with `159` and name the commit it describes, or add a second row
for `4e5c0e7` recording `168`. One number, one line.

### NIT-1 — `reclaimed_allocated_bytes_counted_more_than_once` is not exact for a hard link to the namespace marker, though that link is wholly inside the namespace

`run_source_keyed_cargo_gate.py:5220`–`:5224`; payload semantics at `:6003`–`:6007`. Created by
`4e5c0e7`.

The payload comment states the field is "Exact for links wholly inside the namespace, which is the
Cargo uplift case; a lower bound when the other link is outside it." There is a second case, inside
the namespace, where it is a lower bound. The marker is skipped by the scandir walk (`:5089`–`:5093`)
and removed last in a separate branch that increments `hardlinked_entries` when
`marker.st_nlink > 1` but never consults `seen_multilink_inodes` and never adds to
`duplicate_allocated_bytes`. Because `st_nlink` drops as the walk unlinks, by the time the marker
branch runs its link count has already fallen to 1, so it is not even counted as hard-linked.

Reproduced (`/tmp/probe_hardlink2.py`, case `marker-hardlinked-inside`: one 4 KiB marker with a
second link named `marker-alias` inside the namespace root):

```text
du -s --block-size=1                              4096
namespace_usage allocated_bytes                   8192
remove_tree_contents_at allocated_bytes           8192
removed_hardlinked_entry_count                       1
reclaimed_allocated_bytes_counted_more_than_once     0   ← should be 4096
```

Failure scenario: a build script running inside the gate creates a hard link to
`.codefire-source-key.json` in the namespace root — the namespace root is writable by design, and
`link(2)` needs write permission on the destination directory, not on the source file. The next
cleanup reports an overcount of `0` while the per-entry convention double-counted the marker's
allocation, so a consumer subtracting the field to recover the de-duplicated figure gets a number
4,096 bytes too high. No budget decision consumes the field, so the consequence is confined to
evidence accuracy.

Smallest correct fix: in the marker branch, test `(marker.st_dev, marker.st_ino)` against
`seen_multilink_inodes` exactly as the walk does, or narrow the payload comment to say the field
excludes the marker.

### NIT-2 — `mount_points` still silently skips the malformed line that `mount_propagation_tags` was just taught to reject

`readonly_source_exec.py:21`–`:27`. **Not created by `4e5c0e7`** — byte-identical across `bfbbc2b`,
`aaa39b6` and `4e5c0e7` (`sha256` of the function region: `db8f0955…`).

`mount_propagation_tags` now opens with "Every rule below is `reject`, never `skip`", and the sibling
function eleven lines above it does the opposite:

```python
if len(fields) >= 7 and "-" in fields:
    result.append(Path(unescape_mount_field(fields[4])))
```

A malformed line is skipped, so its mount point is absent from the list the nested-mount rejection at
`readonly_source_exec.py:981`–`:995` iterates — the fail-open direction, and the same shape the prior
report's NIT-1 was raised on. The `"-" in fields` search is also unanchored, which is the exact hazard
`mount_propagation_tags`'s `fields.index("-", 6)` was written to avoid: a mount point or root field
literally named `-` satisfies the guard on a line that has no separator at all.

Failure scenario: a mountinfo line that `mount_points` cannot parse hides a mount nested below the
read-only source, snapshot, toolchain, admitted or evidence root from the nested-mount check.
Unreachable in practice — the kernel's own format always has at least ten fields — which is the same
argument the prior report used for its NIT-1, and the reason that one was worth one line. I raise
this because the repair created an explicit consistency rule in the adjacent function and left its
sibling on the other side of it, not because I found a reachable path.

### NIT-3 — step 9's survivor sentence is unconditional, but the mechanism is bounded by the *declared* `--pass-fds`

`docs/development/ai/source-keyed-cargo-gates.md:184`. Created by `4e5c0e7`.

The first clause is exact — "closes the complement of the retained set over the whole inherited
`--pass-fds` set" — and the second is not: "What survives is stdio, the primary tool, and the
descriptors the child environment itself names." The close set is the complement over the *parsed
`--pass-fds` string*, so a descriptor the kernel inherits but the string omits survives.

Reproduced (`/tmp/probe_fdset.py` with `PROBE_DECLARE_ALL=0`, which passes an extra descriptor
through `subprocess(pass_fds=…)` while omitting it from `--pass-fds`):

```text
MISMATCH
LEAK [19, 'extra-b', '/tmp/.../extra-b.bin']
```

This is currently unreachable in the gate. Both the `--pass-fds` string
(`run_source_keyed_cargo_gate.py:2949`) and the kernel-level `pass_fds=` argument (`:7435`) read the
same `pinned["pass_fds"]` tuple, and `run_timed_gate.gate_runner` forwards it unmodified because the
source-keyed gate passes `pid_namespace_isolated=True` (`run_timed_gate.py:1022`); the branch at
`:1056` that unions in extra launcher descriptors is taken only by the non-isolated callers. So the
divergence requires a future edit, which is why this is a NIT and not a MUST.

It is worth noting because the candidate's own control is *stronger* than the mechanism: the in-child
oracle reads the whole of `/proc/self/fd`, so a future divergence would be caught by the test even
though the implementation would leak. Enumerating `/proc/self/fd` in the close loop instead of the
parsed string would make the survivor sentence unconditionally true and cost about four lines.

### NIT-4 — the new documentation oracle for the unshare flags is a substring assertion, so it cannot fail in the direction MUST-2 actually failed

`docs/development/tools/test_ai_tools.py:2690`–`:2696`. Created by `4e5c0e7`.

```python
self.assertIn(" ".join(("unshare", *UNSHARE_ISOLATION_FLAGS)), self.contract)
```

`MUST-2`'s defect was that the *recorded* list was a strict prefix of the launched one. This oracle
compares the constant against the doc by containment, so if `UNSHARE_ISOLATION_FLAGS` were shortened
to its pre-fix three elements, the doc line at `source-keyed-cargo-gates.md:156` — which spells the
full five — would still contain the shorter join and the test would still pass.

Demonstrated: running the candidate's whole 168-test suite against the predecessor implementation
with `UNSHARE_ISOLATION_FLAGS` shimmed to the pre-fix three-element list, this test **passes**, while
`test_gate_uses_locked_source_namespace_and_source_keyed_log` fails. The launch test is the one doing
the real work: it pins `launch[14 : 14+len]`, then `launch[14+len] == "--"`, which fixes the length,
and asserts the published `readonly_isolation_identity["unshare_flags"]` equals the constant. So the
property that matters is covered; the doc oracle is weaker than its name suggests. An equality
against the extracted doc line, rather than a containment, would close the gap.

### NIT-5 — one new test is a forward guard, not a regression control, and `result.md` files it as the latter

`docs/development/tools/test_ai_tools.py:3653`–`:3675`;
`.../aia-source-keyed-cargo-gate-second-closure-repair-20260731/result.md:208`. Created by
`4e5c0e7`.

`test_no_consumer_charges_reclaimed_bytes_back` **passes unchanged on `aaa39b6`** — measured, in the
168-test run against the predecessor implementation. That is correct behaviour: no charge-back
consumer existed there either; the prior `MUST-3(a)` defect was the *doc claiming* one, and that
defect is caught by `test_reclaim_is_not_described_as_exact_or_charged_back`, which does fail on the
predecessor. But `result.md` §4 lists it in a table headed "New non-vacuous regressions", which
overstates what it demonstrates. Its search scope is also four literal patterns over
`docs/development/tools/*.py`, so a read via an intermediate variable, or from outside that
directory, would not be seen — immaterial today, since the only Python outside that directory is
`setup.py` and the disjoint `tests/` suite, neither of which touches gate result JSON.

By the audit's `I4` conditions this is admissible as an oracle — it states a property, and deleting
the property's guard would change its result — but not as evidence that a predecessor defect was
closed. The label, not the test, is what needs correcting.

## 6. `I2` — diff-scoped review of every new symbol, separate from closure

Computed mechanically as `git show aaa39b6:<path> | grep -c <symbol>` versus the same at `4e5c0e7`.
Every symbol below is absent from the predecessor tree and reviewed here as unreviewed surface in its
own right, independently of whether it closes a bound finding.

| New symbol | Kind | Independent review |
|---|---|---|
| `UNSHARE_ISOLATION_FLAGS` | module constant, 4 references | A tuple, module-level, never mutated; `isolation_identity` copies it with `list(...)` so the recorded payload cannot alias it; `isolated_command` splats it. No import cycle, no ordering hazard. Sound. |
| `seen_multilink_inodes` | local set in `remove_tree_contents_at` | Correct key choice: `(st_dev, st_ino)` is stable under the walk's own unlinking while `st_nlink` is not, which is the subtlety the reverse-order case in section 3(b) confirms. Only inodes with `st_nlink > 1` are inserted, so it holds the uplift set rather than one entry per file. It is nevertheless a **new unbounded accumulator** in a walk that closure 2 deliberately freed from an entry cap; it is bounded only by `TREE_CLEANUP_SECONDS = 300`. At ~130 B per entry a walk that unlinks a million hard-linked files before the deadline would hold ~130 MB. Not a finding — the pre-existing `events` list has the same shape for directories, and the deadline binds both — but it is new memory growth proportional to files rather than directories, and worth knowing. |
| `hardlinked_entries`, `duplicate_allocated_bytes` | locals + two new `remove_tree_contents_at` return keys | Pure accumulators; adjudicated across seven shapes in section 3(b). Directory exclusion via `stat.S_ISDIR` is correct and necessary. One inexactness at the marker branch (NIT-1). |
| `reclaimed_measurement` (6 sub-keys) | new payload block | Each sub-key is a literal that matches the implemented behaviour: I verified `covers_every_removed_entry`, the `convention` string, `hardlinked_inodes_counted_once_per_link`, `is_upper_bound_on_freed_allocation` and `charged_back_against_aggregate_budget: false` against measurement rather than against the code that emits them. |
| `removed_hardlinked_entry_count`, `reclaimed_allocated_bytes_counted_more_than_once` | new payload keys | Semantics as measured; see NIT-1 for the one case where the second understates. |
| *removed*: `resource_accounting_complete`, `resource_accounting_source` | payload keys deleted | Breaking change to the cleanup payload shape. No consumer anywhere in `schemas/`, `crates/`, `codefire/`, `docs/` or the tools outside the test that asserts their absence. Safe. |
| `SourceKeyedGateDocumentationTests` (5 tests) | new test class | Asserts against the contract document with whitespace collapsed, so a rewrap cannot break it. Four of the five fail on the predecessor doc. The fifth is NIT-4. |
| `test_hardlinked_reclaim_is_an_upper_bound_and_says_so` | new test | Errors on the predecessor (missing payload keys); its central assertion `reclaimed − counted_more_than_once == du` is the same equality my independent probe checks, and it holds. |
| `test_no_consumer_charges_reclaimed_bytes_back` | new test | Passes on the predecessor; see NIT-5. |
| `test_blank_mountinfo_lines_are_tolerated`, `test_real_mountinfo_parses` | new tests | Pass on the predecessor by construction — they guard the *new* strict parser against over-rejection, a failure mode that can only exist on the candidate side. Correctly directed. |
| rewritten in-child FD oracle | rewritten control | Section 3, MUST-1. Bounded by the property; fails on the predecessor with ten leaks under my superset harness. |

Only four implementation functions and one constant changed in the gate module
(`isolation_identity`, `isolated_command`, `remove_tree_contents_at`, `cleanup_namespace_locked`),
plus `mount_propagation_tags` and the exec-time close loop in the helper. The `100` inserted and `21`
deleted lines in the gate module contain no removed safety logic: the only deleted non-comment lines
are the two duplicated flag literals and the two retired payload keys.

## 7. `I4` — admissibility of every new oracle, judged by the property it claims

Measured by running the candidate's complete 168-test suite against the **predecessor
implementation** in a real Git worktree, with `UNSHARE_ISOLATION_FLAGS` shimmed to the pre-fix
three-element list so the module imports and every other oracle can be judged. Result on the
predecessor: `Ran 168 tests — FAILED (failures=9, errors=2)`.

| Oracle | Property claimed | Fails on `aaa39b6`? | Shares the implementation's blind spot? |
|---|---|---|---|
| `test_readonly_helper_…_drops_namespace_caps` (rewritten) | exec-time inherited set == retained set | **yes** | No. Reads the whole of `/proc/self/fd` inside the exec'd command and derives the expectation from the child's own environment, not from `descriptors_retained_by_environment`. It is strictly stronger than the mechanism (NIT-3). My own probe, written without reading it, reaches the same verdict on both trees. |
| `test_gate_uses_locked_source_namespace_and_source_keyed_log` (extended) | constant == launched argv == published identity | **yes** | No. Pins the slice length by asserting `launch[14+len] == "--"`, so a shortened constant cannot slip through. |
| `test_hardlinked_reclaim_is_an_upper_bound_and_says_so` | overcount reproduced and quantified; payload states the convention | **yes** (error) | Partly: it exercises the two-link uplift case only, which is the same case the implementation was written against. It does **not** cover three-plus links, reverse encounter order, out-of-namespace links, or the marker. I covered all four independently; the first three hold, the fourth is NIT-1. |
| `test_cleanup_measures_every_entry_over_the_premeasure_cap` (renamed) | reclaim covers every entry when the pre-measure bound refuses | **yes** (error) | No. Pre-existing control, strengthened; the rename is honest ("covered, not exact"). |
| `test_reclaim_is_not_described_as_exact_or_charged_back` | doc states the convention and no charge-back | **yes** | No. |
| `test_descriptor_closure_is_described_as_a_complement` | doc no longer publishes the false invariant | **yes** | No. |
| `test_capability_drop_is_not_claimed_as_a_durable_boundary` | boundary section states the regain | **yes** | No. |
| `test_policy_module_reuse_claim_is_narrowed` | reuse wording matches what is verified | **yes** | No. |
| `test_unparseable_mountinfo_line_is_not_read_as_private` (extended) | every malformed shape rejects | **yes**, 3 of 4 subtests | No. My 21-shape probe finds no shape the candidate gets wrong. |
| `test_blank_mountinfo_lines_are_tolerated`, `test_real_mountinfo_parses` | strict path does not over-reject | no — by design | No. Correctly directed at the candidate-side failure mode. |
| `test_documented_unshare_flags_match_the_launched_argv` | doc quotes the real launch line | no | **Yes, directionally** — see NIT-4. |
| `test_no_consumer_charges_reclaimed_bytes_back` | no tool reads the figure into a decision | no | See NIT-5: admissible as a forward guard, mislabelled as a regression. |

Against the audit's admissibility conditions: the asserted axis observably moves for every oracle in
the "yes" rows; each assertion fires at the boundary it names; the name/docstring claim is entailed
by the assertions in every case except NIT-4 and NIT-5; reversal is behavioural rather than
symbol-absence for the FD, identity and hard-link oracles; and the FD oracle is explicitly not a
frozen counterexample corpus — it is a set equality over whatever the process actually holds.

## 8. Preserved closures, the ten fail-closed classes, and lock ordering

**The five previously verified closures.** Confirmed preserved by diff scope — only the six functions
named in section 6 changed:

1. **policy-module reuse** — `_preload_policy_module` byte-identical; doc wording narrowed only.
2. **deadline-bounded destructive cleanup** — the removal walk's control flow is untouched; the added
   lines are accumulators. Entry cap still applied only when a caller supplies one
   (`remove_tree_contents_at`), still enforced where it belongs (`namespace_usage_at:3756`).
3. **private propagation** — strengthened: one constant, a stricter parser, and the three
   `prove_private_propagation` calls (before the first mount, in the post-drop phase, after the mount
   phase) all now run through the reject-everything parser.
4. **cleanliness separated from unchangedness** — untouched.
5. **global execution lock** — `cargo_registry_locks:3338`–`:3347` byte-identical: the global
   execution lock is still taken unconditionally *before* the registry lock, and the call site at
   `:7101` is unchanged. Lock order remains evidence → global → registry → namespace → provenance.

**The ten fail-closed classes.** Every code path the prior report's section 4 adjudicated is
byte-identical at `4e5c0e7`. I re-ran the evidence class directly rather than relying on that
(`/tmp/probe_analyzer.py`, driving the real `CargoTestLogAnalyzer`):

```text
genuine-single-harness      PASS   executed_tests=3
forged-summary-only         BLOCK  test-summary-without-running-harness
forged-then-zero-harness    BLOCK  test-summary-without-running-harness, zero-selected-target-harness
zero-selected-harness       BLOCK  zero-selected-target-harness
harness-outside-target      BLOCK  cargo-test-harness-outside-target, running-count-without-target-harness
dotdot-escape               BLOCK
running-summary-mismatch    BLOCK  test-summary-running-count-mismatch
second-unexpected-harness   BLOCK  unexpected-target-harness-count
```

The genuine single harness is the only `PASS`. The capability class was re-measured live in section 4
and the descriptor class in section 3; the mount-propagation, cleanup-TOCTOU, accounting,
partial-write/process-group, executable-binding and key-separation classes rest on code this commit
does not touch. Nothing here relaxes a check; the descriptor and propagation classes are strictly
tighter than at `aaa39b6`.

**Cleanup and resource ordering.** `cleanup_namespace_locked`'s only change is its return payload; the
descriptor open/close ordering, the `finally` that closes the target and registry descriptors, and
the post-cleanup aggregate re-measurement under the registry lock are all unchanged.

## 9. Adjudication of the candid residuals under the zero-residual objective

`result.md` §7 declares seven residuals. Three are product-behavioural and are the ones the
zero-residual objective has to rule on; I adjudicate all seven for completeness and do not expand
scope beyond them.

| Residual | Adjudication |
|---|---|
| **1. Reclaim remains an upper bound; inode de-duplication not implemented.** | **Accepted, and correctly reasoned.** De-duplication makes measured allocation smaller and therefore loosens a budget whose sizing basis was derived under the counting convention; re-deriving it requires running Cargo, which is out of scope. The doc states the convention, the direction of the error, why it is deliberate, and the precondition for changing it. Verified internally consistent and non-loosening across seven shapes (section 3(b)). Not an open residual. |
| **2. `…_counted_more_than_once` is exact only for links wholly inside the namespace.** | **Under-declared — this is NIT-1.** The declaration covers the out-of-namespace case, which I confirmed behaves as stated. It omits a second case that *is* wholly inside the namespace: a hard link to the namespace marker, where the field reports `0` against a real 4,096-byte overcount. Open. |
| **3. `prove_private_propagation` is strict about the whole namespace; a host slave mount would block the gate.** | **Accepted.** I reproduced the host condition (`master:` tags outside the gate, cleared inside it by `--propagation private`, live test green). The failure direction is fail-closed, the availability edge is real, and it is disclosed rather than hidden. Not an open residual. |
| **4. MUST-1 severity remains low; `/usr` is unmasked so those programs were already reachable by path.** | **Accepted as a boundary statement, not a residual.** Consistent with the existing explicit-boundary section, and the repair removes inherited descriptors rather than claiming to remove path reachability. |
| **5. The project CLI suite under `tests/` was not run.** | **Out of scope by instruction**, and independently disjoint: it is not reachable from any changed path and nothing under `tests/` imports `docs/development/tools`. Correctly reported as not-run rather than as passing. |
| **6. No Rust evidence exists for this lane.** | **Out of scope by instruction** and consistent with the recovery execution ledger. Correctly reported. |
| **7. `check_ai_development.py` and the capability probe create the two cache roots if absent.** | **Accepted, pre-existing at `bfbbc2b`.** Disclosed accurately. |

One residual the implementer did not declare, and which the zero-residual objective should catch:
the permanent-record number in `MUST-1` of this review. `result.md` §2 asserts it was recorded; it
was not.

## 10. Finding classification and attribution

| Finding | Class | Created by `4e5c0e7`? |
|---|---|---|
| MUST-1 — `history.md:16` states `161` for a 159/168-test lineage; commit message and `result.md` claim it was corrected | `BOUND_FINDING_NOT_CLOSED` + `REPAIR_INTRODUCED_REGRESSION` (permanent record) | **Yes.** The predecessor said `157`; the repair replaced it with a different wrong number and published a closure claim. |
| NIT-1 — marker hard link is a wholly-inside-namespace case the overcount field misses | evidence accuracy | **Yes.** The field and its exactness claim are new in this commit. |
| NIT-2 — `mount_points` silently skips malformed lines and searches `-` unanchored | fail-open consistency | **No.** Byte-identical at `bfbbc2b`, `aaa39b6` and `4e5c0e7`. Surfaced by the new rule the repair wrote into the adjacent function. |
| NIT-3 — step 9's survivor sentence is unconditional; the mechanism is bounded by the declared `--pass-fds` | claim vs implementation | **Yes.** The sentence is new text; the leak class it describes is currently unreachable in the gate. |
| NIT-4 — doc-quote oracle is a substring assertion, blind to the direction MUST-2 failed in | oracle strength | **Yes.** New test. |
| NIT-5 — `test_no_consumer_charges_reclaimed_bytes_back` is a forward guard filed as a regression control | oracle labelling | **Yes.** New test and new table row. |

No finding in this review is a privilege escalation, an evidence-forgery path, or a relaxation of the
gate's fail-closed posture. The gate is strictly tighter at `4e5c0e7` than at `aaa39b6` on the
descriptor and mount-propagation surfaces, and no weaker anywhere.

## 11. What I ran

```text
git worktree add --detach /tmp/ckpt-wt bfbbc2b   # real Git worktree
  python3 -m unittest test_ai_tools              → Ran 148 tests — FAILED (failures=4, errors=6)
git archive bfbbc2b | tar -x -C /tmp/ckpt-arch   # non-Git extraction, prior report's method
  python3 -m unittest test_ai_tools              → Ran 148 tests — FAILED (failures=4, errors=8)
git worktree add --detach /tmp/pred-wt aaa39b6   # real Git worktree
  candidate suite against predecessor impl       → Ran 168 tests — FAILED (failures=9, errors=2)
python3 -m unittest test_ai_tools  (candidate)   → Ran 168 tests — OK (16.270s)
python3 -m py_compile docs/development/tools/*.py                        → OK
python3 docs/development/tools/check_ai_development.py                   → PASS ×7
    acceleration_docs, contracts, project_docs, source_docs,
    runtime, tool_tests, validation_capabilities
python3 docs/development/tools/check_ai_contracts.py                     → passed
python3 docs/development/tools/check_project_docs.py                     → passed
python3 docs/development/tools/check_source_docs.py                      → passed (44/34/12)
python3 docs/development/tools/check_ai_acceleration_docs.py             → passed
git diff --check                                                         → clean
/tmp/probe_fdset.py       # complete exec-time FD set vs an independently derived retained-set oracle
/tmp/probe_hardlink2.py   # seven hard-link / directory / ordering / marker accounting shapes
/tmp/probe_mountinfo.py   # 21 mountinfo shapes plus live /proc/self/mountinfo
/tmp/probe_identity.py    # constant == recorded identity == launched argv
/tmp/probe_caps.py        # setpriv drop, nested userns regain, umount/remount attempts
/tmp/probe_analyzer.py    # eight Cargo test-evidence spoof classes
```

No Cargo, `rustc`, Clippy or `rustfmt` invocation. The disjoint product CLI suite under `tests/` was
not run, per the review scope. No subagent, Codex or Fugu session. No implementation, test or
management file was modified; the two throwaway worktrees under `/tmp` are removed at the end of this
review and never touched the repository working tree.

## 12. Verdict

`REQUEST_CHANGES`.

Six of the seven bound findings are genuinely and independently closed, each against an oracle I
re-derived and proved non-vacuous on the predecessor tree. `MUST-1`'s descriptor closure now holds
against the complete inherited set including arbitrary extra pass-fds, with no descriptor closed
twice and nothing the child needs closed; `MUST-2` has one authoritative flag sequence feeding argv,
recorded identity, both keys and every result; `MUST-3`'s accounting contract is internally
consistent, cannot loosen the 32 GiB budget, reports its overcount honestly, and behaves correctly
across multiple links, directories, out-of-namespace links and removal-order changes; `SHOULD-1`'s
measured boundary table reproduces exactly; `NIT-1` and `NIT-2` are closed. The five previously
verified closures, the ten fail-closed classes, the global execution lock and the cleanup ordering
are all preserved, and the implementer's correction of the prior report's checkpoint count from 12 to
10 is right, which I confirmed by reconstructing the checkpoint in an actual Git worktree.

The change is not acceptable as written because `SHOULD-2` — the bound finding that the permanent
record misstates its test counts — is not closed. The row now carries a third wrong number, and both
the commit message and the result document affirmatively claim it was corrected. Under the
zero-residual objective, NIT-1 (an inside-the-namespace case the new exactness claim misses), NIT-3,
NIT-4 and NIT-5 should be taken in the same pass; NIT-2 is pre-existing and is recorded as such
rather than charged to this commit. Every one of these is a one-to-four-line change in a file this
commit already touches.
