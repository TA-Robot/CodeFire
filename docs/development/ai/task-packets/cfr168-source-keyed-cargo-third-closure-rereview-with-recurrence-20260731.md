# CFR-168 Source-keyed Cargo Gate — Third Closure Rereview under AIA-047

Status: Prepared, not launched — launch is blocked until the AIA-047 gate
independent review (`cfr168-review-recurrence-gate-independent-review-20260731.md`)
returns `OVERALL_VERDICT: ACCEPT` and the manager accepts that verdict. No
reviewer may be dispatched against this packet before then. This document is
frozen pre-review evidence, not a dispatch.

Provider/model: Claude Opus 5 (`claude-opus-5`), fresh session.

Role: exact independent report-only reviewer. Review generation 3.

Pilot: AIA-047 successor review recurrence
(`docs/development/ai/review-recurrence/README.md`). This is the Cargo arm of
the two mandatory pilot runs.

## Objective

Decide, independently and report-only, whether candidate
`74cd37d5e9204c45ac23bf31aa6973d6004cc0f0` discharges the five frozen
closure contracts R1–R5 below, and separately whether the repair's own delta
— including the surfaces the pre-repair packet did not forecast — is sound.

Prior-obligation closure and repair-created delta are two distinct passes.
Passing the first does not excuse skipping the second.

There is no finding quota and no target report length. Coverage is judged by
obligation-and-surface saturation: every frozen obligation, every Git-derived
changed path, every actual changed surface, every required production route,
and every negative control listed here needs a disposition, and unavailable
evidence must be named as unavailable. Counts are diagnostic outputs, never
targets. Do not restate one effect under several observation rows to inflate
a count; do not omit a hard surface because the easy ones are done.

## Immutable subject — recompute every value before analysis

| Field | Exact value |
|---|---|
| review base (candidate parent) | `4e5c0e7f058cd406cb7a183ecda8b3a07f3ee912` |
| candidate | `74cd37d5e9204c45ac23bf31aa6973d6004cc0f0` |
| candidate tree | `b0aef9e41da5c7493e8ae9449ca81188dccc77cd` |
| base tree | `6417400cb7b5a3507f85f38d66c0573992ad4007` |
| additive commits in `base..candidate` | 1 |
| merge commits in `base..candidate` | 0 |
| canonical binary patch SHA-256 | `98ac645724493b10a761f52c635438962d2aba8804481f393085fd07445d9a2c` |
| numstat | 653 inserted, 23 deleted, 0 binary paths |
| shortstat | `6 files changed, 653 insertions(+), 23 deletions(-)` |
| predecessor (generation-2 subject) | `aaa39b61b37df273fe4d57e274ed8e1321e788ad` |
| checkpoint | `bfbbc2b2bdc22f88679e5e1dc794665d863304a1` |

The canonical patch digest is over exactly:

```bash
git diff --no-ext-diff --no-textconv --no-renames --binary --full-index \
  --src-prefix=a/ --dst-prefix=b/ \
  4e5c0e7f058cd406cb7a183ecda8b3a07f3ee912 \
  74cd37d5e9204c45ac23bf31aa6973d6004cc0f0 -- | sha256sum
```

Any other argv produces a different digest and is not the bound identity.

### Prior generations, immutably bound

| Generation | Review commit | Report path | Report SHA-256 | Decision |
|---|---|---|---|---|
| 1 | `b5898640e7d5d6b8442d36805cebdfb981e2cc06` | `docs/development/ai/swarm-results/aia-source-keyed-cargo-gate-closure-rereview-claude-20260731/review.md` | `212d391c8fdb23c9178e6f8ddbb68cfc7087a50950049f137d19d342a5e2e259` | `REQUEST_CHANGES` |
| 2 | `130878d321f6be166f554ac2c68a59e2b36afa88` (parent `4e5c0e7`) | `docs/development/ai/swarm-results/aia-source-keyed-cargo-gate-second-closure-rereview-claude-20260731/review.md` (49,965 bytes) | `aad067b2b8feb80830c41db1608dd31f0e7e7d7ca1f37ab01d040336f788b4fd` | `REQUEST_CHANGES` |

Do not edit, amend, replace, or re-derive either report. The generation-1
report contains one figure this lineage has already refuted (12 checkpoint
failures, an artifact of a non-Git `git archive` extraction); the correct
figure is 10, and restoring 12 is a regression, not a fix.

### Pre-repair packet, content-addressed before implementation

| Field | Exact value |
|---|---|
| repair packet path | `docs/development/ai/task-packets/cfr168-source-keyed-cargo-third-closure-repair-20260731.md` |
| repair packet commit | `8c51b712f1e3937fd462a95c70b8de79687b9289` |
| repair packet SHA-256 | `7464f158cc04373af2cb5e0eacb37212233d920dc9233bf220e553361a43adc7` (8,747 bytes) |
| repair packet commit time | `2026-07-31T12:47:02+00:00` |
| candidate commit time | `2026-07-31T13:08:06+00:00` |
| `merge-base --is-ancestor 8c51b71 74cd37d` | **false** — merge base is `893cbcad2b1f2a8455a5c256246c8110d3fcbdd6` |

The repair packet is therefore **not** in the candidate's ancestry: it was
committed on the canonical integration lane while the repair ran in the
implementer's own lane. Its pre-repair status rests on a content-addressed
dispatch event, and the final record must declare
`repair_packet_timing: "dispatch_event_attested"` with both attestation
fields populated. Declaring `ancestor_of_candidate` here is false and the
gate rejects it.

| Field | Exact value |
|---|---|
| dispatch event ref | `8c51b712f1e3937fd462a95c70b8de79687b9289:docs/development/ai/swarm-batches/cfr168-recovery-execution-ledger-20260730.md`, section `## 2026-07-31 12:44 UTC Cargo second-closure review completed and causally split` |
| dispatch event SHA-256 | `404a4f98402dc6cd45fef301f04243e5f0f020ae9c2c39e01665de3dede9fe45` (166,302 bytes) |

That ledger section states the repair packet's SHA-256, the R1–R5 owner
table, the provisional causal split, the routed parser NIT, and the
no-quota rule, in a commit whose committer time precedes the candidate's by
1,264 seconds. Its trust limit is explicit and must be reported as such: a
committer timestamp is mutable metadata, so the attestation proves the
*content* of the pre-repair contract but bounds its *chronology* only as
strongly as the canonical lane's commit sequence. If the reviewer finds any
evidence that the R1–R5 table was authored or altered after the candidate
existed, that is a blocking finding, not a note.

### Third-closure repair result, as committed

| Field | Exact value |
|---|---|
| path | `docs/development/ai/swarm-results/aia-source-keyed-cargo-gate-third-closure-repair-20260731/result.md` |
| SHA-256 at candidate | `3780afd8e8e36c942a4fd4ccc171d955a78fd94609d1593574252f13ebfc3a02` (20,228 bytes) |

Treat every figure in it as an unverified claim.

### File digests at the candidate

```text
056445f5284c685c8722b35c740968fd3c7dfc1bc7413ddc44958170a58f4f61  docs/development/history.md
b76bbc0a1aab56cf484fb52fb42e3babb5a6025bcd591fd1b67a2b5d53b7b373  docs/development/ai/source-keyed-cargo-gates.md
aec771414a388dab45f1b3760842fc1f79a7b898e831f6ab612bb8a15598dd9f  docs/development/tools/run_source_keyed_cargo_gate.py
5fb42a01fcec0c43990f7e31095082c4818d3b02e60c3854d2501e49c1e852d0  docs/development/tools/test_ai_tools.py
bb10c3a479713ef36ccc8ecfbc4fc71d6dd0ef0cf5696de135621ee6fb0aa6ea  docs/development/ai/swarm-results/aia-source-keyed-cargo-gate-second-closure-repair-20260731/result.md
3780afd8e8e36c942a4fd4ccc171d955a78fd94609d1593574252f13ebfc3a02  docs/development/ai/swarm-results/aia-source-keyed-cargo-gate-third-closure-repair-20260731/result.md
```

## Scope

### Frozen prior obligation closure contracts

These five contracts were frozen by the manager in the pre-repair packet
before implementation. They are the communication boundary: the reviewer
judges the candidate against these words, not against the implementer's
restatement of them. Stable IDs are `R1`–`R5` and must be reused verbatim in
the final record's `prior_findings[].id`.

#### R1 — permanent record states a test count matching no tree

| Field | Frozen content |
|---|---|
| effect episode | `EFF-HISTORY-TEST-COUNT`, origin generation 1 (`b589864` `SHOULD-2`), continuity `continuous_open` through generation 2 (`130878d` `MUST-1`) |
| acceptance criterion | `docs/development/history.md` row 16 states, for each commit it names, that commit's own defined-test count, and no count that matches no tree in the lineage. Ground truth: `bfbbc2b` = 148 defined / 10 genuine failures in a real Git worktree; `aaa39b6` = 159, green; `4e5c0e7` = 168, green. A mechanical control derives its expected values from the named commits' own committed blobs and fails on `157` and on `161`, and does not become self-referential when this repair adds tests. |
| authorized boundary | `docs/development/history.md` row 16; the new control in `docs/development/tools/test_ai_tools.py`; the false closure sentence in `.../aia-source-keyed-cargo-gate-second-closure-repair-20260731/result.md`. |
| required live routes | control fails on `157`; control fails on `161`; control fails on a count matching no tree; control is unaffected by adding a test to the current suite; static count agrees with the measured runtime count for all three commits. |
| anticipated regressions | a fourth wrong number; a row that names no commit so no count can be wrong; an oracle that reads the *running* suite and so drifts as tests are added; an oracle that hard-codes 148/159/168 as literals and therefore proves nothing about the named blobs; publishing this repair's own HEAD count into `history.md`; restoring the refuted `12`. |
| forbidden shortcuts | prose-only correction with no oracle; asserting the numbers by re-running historical suites in-test; deleting the row instead of repairing it; weakening the row to omit counts; claiming closure in a commit message or result document that the committed bytes do not support. |

#### R2 — marker hard-link overcount reported as zero

| Field | Frozen content |
|---|---|
| effect episode | `EFF-MARKER-OVERCOUNT`, origin generation 2 (`130878d` `NIT-1`), created by `4e5c0e7` |
| acceptance criterion | For a namespace containing a hard link to the namespace marker wholly inside the namespace, `reclaimed_allocated_bytes_counted_more_than_once` reports the real overcount and `removed_hardlinked_entry_count` counts the marker, so that `removal-walk allocated − counted_more_than_once == du -s --block-size=1`; or the payload wording is narrowed to exclude the marker. The payload and its exactness wording agree for that case. |
| authorized boundary | the marker branch of `remove_tree_contents_at` in `docs/development/tools/run_source_keyed_cargo_gate.py`, plus one control in `test_ai_tools.py`. |
| required live routes | primary `cleanup_namespace_locked` path; `recover_incomplete_namespaces` recovery path with `expected_marker=None`; unlinked-marker case; directory entries with `st_nlink >= 2`; link whose other end is outside the namespace. |
| anticipated regressions | double-counting an unlinked marker; charging a directory; changing any `unlink`/`rmdir`/deadline/entry-cap decision; loosening the 32 GiB aggregate budget or the `MEASURED_MAX_NAMESPACE_ALLOCATED_BYTES` basis; changing the conservative per-entry convention; making the accumulator grow with files in a walk bounded only by `TREE_CLEANUP_SECONDS`. |
| forbidden shortcuts | switching to inode de-duplication (loosens a budget whose sizing basis was derived under the per-entry convention and cannot be re-derived without running Cargo); a symbol-name-only reversal instead of a property reversal; deleting the exactness claim without fixing the field while leaving the field named "exact" elsewhere. |

#### R3 — step 9's survivor sentence was unconditional

| Field | Frozen content |
|---|---|
| effect episode | `EFF-STEP9-SURVIVOR-CLAIM`, origin generation 2 (`130878d` `NIT-3`), created by `4e5c0e7` |
| acceptance criterion | Step 9 of `docs/development/ai/source-keyed-cargo-gates.md` states the survivor set conditionally on the descriptors *declared* in `--pass-fds`, so that the documented claim is true of the implemented complement closure. |
| authorized boundary | the step 9 prose; assertions in `test_descriptor_closure_is_described_as_a_complement`. |
| required live routes | declared-set complement closure as implemented; the undeclared-descriptor divergence (`PROBE_DECLARE_ALL=0` shape); the shared `pinned["pass_fds"]` origin of both the `--pass-fds` string and the kernel `pass_fds=` argument; `run_timed_gate.gate_runner` forwarding for `pid_namespace_isolated=True`. |
| anticipated regressions | weakening the implemented complement closure; claiming a general filesystem sandbox; asserting the divergence is impossible rather than currently unreachable; dropping the pre-existing `assertNotIn` that guards the original false invariant. |
| forbidden shortcuts | deleting the sentence rather than qualifying it; a substring assertion that a rewrap or a shortened claim would still satisfy; asserting the wording without asserting absence of the old false invariant. |

#### R4 — the unshare documentation oracle could not fail in MUST-2's direction

| Field | Frozen content |
|---|---|
| effect episode | `EFF-UNSHARE-DOC-ORACLE-DIRECTION`, origin generation 2 (`130878d` `NIT-4`), created by `4e5c0e7` |
| acceptance criterion | The documentation control detects the actual failure direction: launched argv, recorded isolation identity, source key, artifact key, published result, and the documented sequence must all remain equal and in order. Shortening `UNSHARE_ISOLATION_FLAGS` to its pre-fix three elements must fail the control, and shortening the doc and the constant together must also fail. |
| authorized boundary | `docs/development/tools/test_ai_tools.py` controls only; no change to `UNSHARE_ISOLATION_FLAGS`, `isolation_identity`, or `isolated_command`. |
| required live routes | documented sequence vs constant; recorded identity vs constant; launched argv slice vs constant; source key and artifact key under a mutated flag list; published result JSON `readonly_isolation_identity`. |
| anticipated regressions | replacing one weak assertion with another weak assertion; an oracle that passes when order is permuted; an oracle that passes when length changes; losing the argv-slice length pin (`launch[14+len] == "--"`); losing the published-result equality. |
| forbidden shortcuts | containment assertions; comparing the constant against itself; asserting only the symbol's presence; claiming coverage of argv and result while asserting neither anywhere in the suite. |

#### R5 — a forward guard was filed as a regression control

| Field | Frozen content |
|---|---|
| effect episode | `EFF-FORWARD-GUARD-LABEL`, origin generation 2 (`130878d` `NIT-5`), created by `4e5c0e7` |
| acceptance criterion | The no-consumer control is retained and labelled as a forward guard rather than as evidence that a predecessor regression was closed; the prior repair result's table and any matching claim are corrected; the predecessor-failing control for the same claim is named. |
| authorized boundary | the test's name and docstring in `test_ai_tools.py`; the table heading and row in `.../aia-source-keyed-cargo-gate-second-closure-repair-20260731/result.md`. |
| required live routes | the guard's behaviour on predecessor `aaa39b6` (passes, by construction); the behaviour of `test_reclaim_is_not_described_as_exact_or_charged_back` on `aaa39b6` (must fail, or the replacement claim is also false); the guard's four-literal search scope. |
| anticipated regressions | deleting a sound test to resolve a labelling error; renaming without correcting the prior result; asserting a new false claim about which control is predecessor-failing; silently widening the search scope claim. |
| forbidden shortcuts | claiming a behavioural negative control for a labelling change; correcting the label in the new result only and leaving the prior result's table wrong. |

### Routed, not an obligation of this repair — pre-existing `mount_points` parser

`docs/development/tools/readonly_source_exec.py:21`–`:27`. `mount_points`
skips malformed lines and searches for `-` unanchored, the fail-open
direction, while the adjacent `mount_propagation_tags` now rejects every
malformed shape. Function-region SHA-256 is
`db8f09559030749090514aafc037ce583de67c9b38654cc62d19d85415fb2dda` at
`bfbbc2b`, `aaa39b6`, `4e5c0e7`, and — to be confirmed by the reviewer — at
`74cd37d`. `readonly_source_exec.py` is not in the candidate's changed-path
set.

This item is **routed, not closed**. Record it in the final record's
`new_findings` with `attribution: "pre_existing"`, `blocking: false`,
`disposition: "routed_open"`, and causal classes
`["pre_existing_adjacent", "shared_primitive_or_composition_miss"]`. It must
never appear in `prior_findings`, must never carry
`repair_introduced_regression`, must never be counted toward closure — local
or global — and must not be repaired in this lineage. Its route is:

- owner: CFR-168 source-keyed Cargo gate lane, next separately admitted
  parser-hardening task packet;
- release condition: not required for this candidate's integration; required
  before any claim that the helper's mountinfo handling is uniformly
  fail-closed;
- route authority: this packet, section "Routed, not an obligation of this
  repair", manager `t.asakura`.

If the reviewer finds the bytes are *not* identical at `74cd37d`, the
routing is void and it becomes a blocking scope violation.

### Exact Git-derived changed-path coverage

The final record's `changed_path_coverage` must equal this set exactly —
`check_review_recurrence.py` recomputes it from Git and blocks on an omitted
or extra row. Every row's `required_by` must name obligation IDs from R1–R5.

| Exact path | Required by | Insert/delete | Mandatory review focus |
|---|---|---|---|
| `docs/development/history.md` | R1 | 1 / 1 | row 16 states `aaa39b6`/159 and `4e5c0e7`/168 and no count matching no tree; the row's new "upper bound, corrected in `4e5c0e7`" clause is factually true of the accounting contract; no other ledger row was disturbed; this repair's own HEAD count (173) does not appear |
| `docs/development/tools/run_source_keyed_cargo_gate.py` | R2 | 19 / 5 | marker branch consults `seen_multilink_inodes` before falling back to `st_nlink`; no removal, deadline, entry-cap or budget decision changed; unlinked marker not double-counted; directories still excluded; accumulator growth bound |
| `docs/development/ai/source-keyed-cargo-gates.md` | R3 | 12 / 2 | step 9 survivor clause is conditional on the declared `--pass-fds`; the added reachability and control-strength claims are true of `run_timed_gate.py` and `readonly_source_exec.py` as committed; no sandbox claim; no weakening of the complement closure |
| `docs/development/tools/test_ai_tools.py` | R1, R2, R3, R4, R5 | 332 / 11 | every added or modified oracle asserts the property its name claims; the deleted substring oracle's coverage is not lost; expected values come from independent sources; no oracle is self-referential against the running suite |
| `docs/development/ai/swarm-results/aia-source-keyed-cargo-gate-second-closure-repair-20260731/result.md` | R1, R5 | 26 / 4 | both corrections are marked as corrections, are true, and do not overwrite the original claim without stating what it was; no other claim in the document silently changed |
| `docs/development/ai/swarm-results/aia-source-keyed-cargo-gate-third-closure-repair-20260731/result.md` | R1, R2, R3, R4, R5 | 263 / 0 | every figure, command, exit code, count and duration is a claim to re-derive; the non-acceptance statement is present; rejected-claims table is honest |

`docs/development/tools/readonly_source_exec.py` is **not** in the set;
confirm that mechanically rather than by reading the result document.

### Actual changed surfaces — planned versus unplanned

This inventory is the manager's recomputation of what the repair actually
changed, not the implementer's summary. Each row needs a `reviewed`,
`blocked`, or `unreviewed` disposition, and `planned: true` rows need an
exact `plan_ref` into the pre-repair packet at
`8c51b712f1e3937fd462a95c70b8de79687b9289`.

| Surface | Required by | Planned? | Kind | Change | Review focus |
|---|---|---|---|---|---|
| `docs/development/history.md`: row 16 count clause bound to `aaa39b6`/159 and `4e5c0e7`/168 | R1 | yes — packet §R1 | claim | modified | matches the named blobs; no unbound count remains |
| `docs/development/tools/test_ai_tools.py`: `HistoryTestCountConsistencyTests` (3 tests) | R1 | yes — packet §R1 "property-based consistency control" | test_or_oracle | added | fails on 157/161/no-tree; static `ast` count over the named commit's own blob; not self-referential |
| `docs/development/tools/test_ai_tools.py`: `defined_test_count` static helper, `import ast`, `import re` | R1 | yes — packet §R1 | private_symbol | added | counts what it claims to count for all three commits; no execution of historical trees |
| `docs/development/tools/run_source_keyed_cargo_gate.py`: marker branch `marker_inode` / `seen_multilink_inodes` / `duplicate_allocated_bytes` | R2 | yes — packet §R2 | private_symbol | modified | property reversal, not symbol reversal; unlinked marker; directory case; no budget or removal change |
| `docs/development/tools/test_ai_tools.py`: `test_hardlinked_marker_overcount_is_reported` | R2 | yes — packet §R2 "add a reversal that fails on `4e5c0e7`" | test_or_oracle | added | `walk − counted_more_than_once == du`; `du` is an implementation-independent expectation |
| `docs/development/ai/source-keyed-cargo-gates.md`: step 9 declared-`--pass-fds` qualifier | R3 | yes — packet §R3 | claim | modified | true of the implementation |
| `docs/development/ai/source-keyed-cargo-gates.md`: added claims that the gate cannot reach the divergence today (shared `pinned["pass_fds"]`, timed-runner forwarding) and that the in-child control is stronger than the mechanism | R3 | **no** | claim | added | each is a *new* factual assertion about `run_timed_gate.py:1022`/`:1056` and `readonly_source_exec.py`; verify independently, because a false unreachability claim is worse than the wording it replaced |
| `docs/development/tools/test_ai_tools.py`: two added assertions in `test_descriptor_closure_is_described_as_a_complement` | R3 | yes — packet §R3 | test_or_oracle | modified | old `assertNotIn` guard retained; restoring the prior sentence fails both new assertions |
| `docs/development/tools/test_ai_tools.py`: `test_documented_unshare_flags_match_the_launched_argv` | R4 | yes — packet §R4 "replace or strengthen" | test_or_oracle | deleted | nothing it covered is now uncovered |
| `docs/development/tools/test_ai_tools.py`: `test_documented_unshare_sequence_equals_the_constant` | R4 | yes — packet §R4 | test_or_oracle | added | sequence equality fixes contents *and* order; adjacency of `--propagation private` |
| `docs/development/tools/test_ai_tools.py`: `test_unshare_sequence_binds_identity_argv_and_both_keys` | R4 | yes — packet §R4 | test_or_oracle | added | mutating the flag list makes all three `(source_key, artifact_key)` pairs distinct; verify the claim that argv and published result equality "remain asserted" elsewhere is true |
| `docs/development/tools/test_ai_tools.py`: `test_no_consumer_charges_reclaimed_bytes_back` → `test_forward_guard_no_consumer_charges_reclaimed_bytes_back` + docstring | R5 | yes — packet §R5 | test_or_oracle | modified | docstring's three claims (passes on predecessor; named predecessor-failing control; four-literal scope) are each true |
| `.../second-closure-repair.../result.md`: two correction blocks and heading change | R1, R5 | yes — packet §R1, §R5 | claim | modified | corrections are true and self-identifying |
| `docs/development/history.md`: replacement of the stale "reclaimed bytes … 常にexactにした" clause with the upper-bound wording | R1 | **no** | claim | modified | in-cell factual correction the packet did not forecast; verify it agrees with the accounting contract at `4e5c0e7` and does not extend beyond the R1 cell |
| `.../third-closure-repair.../result.md`: R2 reachability re-scoping (`validate_marker_at`/`read_regular_at` rejects `st_nlink != 1`; only `recover_incomplete_namespaces` with `expected_marker=None` reaches the marker branch) | R2 | **no** | claim | added | narrows the review's own failure scenario; if false, R2's severity and the review's scenario are both misstated |
| `.../third-closure-repair.../result.md` | R1–R5 | yes — packet "Required result structure" | claim | added | structure and every figure |

Three unplanned surfaces are declared here. All three are claim-class, none
is a public API, state machine, classifier, or persisted shape, and none was
stopped for a scope decision by the implementer. The manager's pre-decision:
they are admitted as `scope_expansion: "manager_approved"` with authority
"this packet, section 'Actual changed surfaces', manager `t.asakura`",
**conditional** on the reviewer confirming each is (a) true and (b) confined
to a file and cell the corresponding obligation already authorized. If any
one fails either test, the manager records `scope_expansion: "blocked"`,
which cannot be integrated. If the reviewer finds a fourth unplanned
surface, that is a reportable communication finding regardless of whether the
surface itself is sound.

### Negative controls the reviewer must run

Reading the implementer's account of these is not a disposition. Run each in
a throwaway detached worktree, record the exact failure text, and restore
identity afterwards. `killed` means the oracle fails; `survived` on any row
is a blocking finding.

| ID | Deliberate mutation | Oracle that must fail |
|---|---|---|
| `NC-R1-157` | `history.md` row 16 count → `157` | `HistoryTestCountConsistencyTests` |
| `NC-R1-161` | row 16 count → `161` | `HistoryTestCountConsistencyTests` |
| `NC-R1-NOTREE` | row 16 count → `173` (a count matching no named tree) | `HistoryTestCountConsistencyTests` |
| `NC-R1-DRIFT` | add one trivial test to the current suite, leave `history.md` correct | control must **not** fail — proves non-self-referentiality |
| `NC-R2-NLINK` | marker branch reverted to `4e5c0e7`'s `st_nlink`-only form | `test_hardlinked_marker_overcount_is_reported` |
| `NC-R3-SENTENCE` | restore the pre-`74cd37d` unconditional survivor sentence | both added assertions in `test_descriptor_closure_is_described_as_a_complement` |
| `NC-R4-SHORT` | `UNSHARE_ISOLATION_FLAGS` → pre-fix three elements | `test_documented_unshare_sequence_equals_the_constant` and `test_unshare_sequence_binds_identity_argv_and_both_keys` |
| `NC-R4-BOTH` | shorten the constant **and** the doc line together | at least one of the two new controls (order/adjacency) |
| `NC-R4-PERMUTE` | permute the flag order without changing length | `test_documented_unshare_sequence_equals_the_constant` |
| `NC-R5-ENTAIL` | none — labelling change | not applicable, with evidence: instead prove `test_reclaim_is_not_described_as_exact_or_charged_back` fails on `aaa39b6`, or R5's replacement claim is also false |

For `R5` the record's `oracle.negative_control.status` may be
`not_applicable`, but only with exact evidence refs; a bare
`not_applicable` is rejected.

### Required production routes

A closure row that names no exercised route is not a closure. At minimum:

- normal route: the gate's own cleanup path (`cleanup_namespace_locked`) and
  the recovery path (`recover_incomplete_namespaces`, `expected_marker=None`);
- error route: unreadable/hard-linked marker rejected by
  `read_regular_at`'s single-link check;
- unwind route: `remove_tree_contents_at`'s deadline expiry with a partially
  removed tree, and the `finally` that closes target and registry
  descriptors;
- composition route: descriptor closure under the real
  `unshare -Urnmpf --mount-proc --kill-child=SIGKILL --propagation private`
  launch and the real `setpriv` drop, with `run_timed_gate` forwarding;
- oracle route: each new control run against the predecessor tree
  `aaa39b6` in a **real Git worktree** (not `git archive`), so that
  `git ls-files`-dependent tests do not produce phantom errors;
- documentation route: the whitespace-collapsed contract text the
  documentation controls actually read.

Any route that cannot be exercised must be reported `unavailable` with the
reason, not silently skipped.

### Causal classification rules

Use only the schema enum. `mixed` is not a value; multiple classes may
apply; do not invent a project-wide root cause from this episode.

| Item | Frozen classification | Rule |
|---|---|---|
| R1 | `bound_finding_not_closed` + `repair_introduced_regression` + `oracle_gap` | Confirm or refute with exact bytes. It is **not** `initial_review_scope_miss` (generation 1 named the defect precisely) and **not** primarily `communication_ambiguity` (the required value was unambiguous). Refutation requires cited bytes, not preference. |
| R2, R3, R5 | `repair_introduced_regression` | Created by `4e5c0e7`, the fix for generation-1 findings. |
| R4 | `repair_introduced_regression` + `oracle_gap` | The control's assertion direction was orthogonal to the defect it was named for. |
| generation-1 `12`-failure figure | `reviewer_artifact` | Reproduction-environment error from a non-Git extraction; already adjudicated, do not re-litigate and do not restore `12`. |
| `mount_points` parser | `pre_existing_adjacent` + `shared_primitive_or_composition_miss` | Never `repair_introduced_regression`; never in `prior_findings`. |
| three unplanned surfaces | `requirement_or_scope_expansion` (+ `communication_ambiguity` if the implementer had to guess the boundary) | Applies to the communication defect, not automatically to the surface's soundness. |

An observation that only rewords an already-recorded effect shares that
effect's `effect_episode_id` and does not create a new one. A prior effect
episode ID must never reappear in `new_findings`; a bound obligation that is
still open belongs in `prior_findings` with `disposition: "open"`, never in
`new_findings`.

### Terminal evidence requirements

The final record is written by the manager after terminalization, and the
gate enforces this shape:

- the report is the reviewer's **only** tree change, committed as a
  single-parent direct child of the review-control commit, changing exactly
  one path;
- the review-control commit must have candidate `74cd37d` as its **sole
  parent** and may touch only paths under
  `docs/development/ai/task-packets/`, `docs/development/ai/schemas/`, or
  `docs/development/tools/`. This packet must be present at that commit with
  the SHA-256 recorded in the launch preconditions below, byte-identical to
  the copy committed on the preparation lane;
- the reviewer process and every child process must be stopped, and
  `child_processes_active` must be 0, before the record is written;
- `terminalization.state` must be `complete` with a content-addressed event
  log (`event_log_ref` + `event_log_sha256`);
- every validation row carries an exact command, a terminal flag, the real
  exit code, a log reference, and a complete-log SHA-256. Background
  progress, truncated output, a different tree, a different argv, a
  different environment, or a mutable name is never `passed`;
- prefer `docs/development/tools/run_timed_gate.py` and report its monotonic
  duration and wall-clock-discontinuity field rather than shell wall time.

### AIA-047 metrics to capture

| Metric | Derivation |
|---|---|
| review generation | 3 |
| final decision | the report's single `OVERALL_VERDICT` line, mirrored into `review_decision` |
| prior obligations closed / open / routed / unavailable | counted from `prior_findings[].disposition`; must equal the `outcomes` fields exactly |
| prior observations vs prior effect episodes | distinct `observation_id` count vs distinct `effect_episode_id` count — divergence is the duplication signal the pilot exists to expose |
| new observations vs new effect episodes | same rule over `new_findings` |
| candidate-created findings | `new_findings` with `attribution: "created_by_candidate"`; each must carry `repair_introduced_regression` |
| unplanned surfaces | `changed_surfaces` with `planned: false`; three are declared here, so a different number is itself a finding |
| `repair_service_ms` | candidate committer time minus dispatch-event commit committer time = `1785503286 − 1785502022` = **1,264,000 ms**, recorded with the explicit note that it is bounded by commit metadata, not by an execution clock |
| `review_service_ms` | reviewer start to `OVERALL_VERDICT`, from the event log; `null` plus an `unavailable_metrics` entry if the log cannot establish it |
| `verdict_to_report_ms` | `OVERALL_VERDICT` to report commit, from the event log; same null rule |
| unavailable evidence | enumerated explicitly in the report and mirrored in the record |
| operator time | manager-side minutes to produce and validate the record, for the Cargo-versus-KGI comparison |

## Non-Goals

- Do not edit the candidate, the two prior review reports, the repair
  packets, the schema, the validator, or any canonical management document.
- Do not repair the `mount_points` parser or otherwise touch
  `readonly_source_exec.py`.
- Do not reopen the six closures generation 2 independently proved, absent
  new exact evidence that one of them regressed at `74cd37d`.
- Do not restore the refuted `12` checkpoint-failure figure.
- Do not run Cargo, `rustc`, Clippy, `rustfmt`, Docker, or the disjoint
  product CLI suite under `tests/`.
- Do not use Codex, Fugu, a subagent, a scheduler, or any parallel lane.
- Do not rebase, amend, squash, cherry-pick, or perform any destructive Git
  operation; do not integrate anything.
- Do not generalize this episode into a project-wide root cause, and do not
  promote AIA-047 to permanent policy.
- Do not produce findings to reach a number, and do not treat report length
  as evidence of thoroughness.

## Validation

Run focused controls and negative controls first, then the broad checks.
Run the `unittest` commands from `docs/development/tools`.

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  test_ai_tools.HistoryTestCountConsistencyTests
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  test_ai_tools.SourceKeyedGateDocumentationTests
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest test_ai_tools
PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile docs/development/tools/*.py
python3 docs/development/tools/check_ai_development.py
python3 docs/development/tools/check_ai_contracts.py
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_source_docs.py
python3 docs/development/tools/check_ai_acceleration_docs.py
git diff --check
```

Record the complete exit status of each, and distinguish a terminal result
from background progress. Every negative control in the table above must be
run and reverted, with the restored tree re-verified green. The suite defines
173 tests at the candidate; report the number you measure rather than
repeating that figure.

## Handoff

Write one report at:

```text
docs/development/ai/swarm-results/aia-source-keyed-cargo-gate-third-closure-rereview-claude-20260731/review.md
```

Use the compact exact independent review shape plus the successor review
recurrence addendum in `docs/development/ai/validation-report-template.md`.
The report must contain the recomputed immutable identity table, one
disposition per frozen obligation R1–R5, the exact Git-derived changed-path
coverage table, the actual-surface table with planned/unplanned marked, the
negative-control results, the production routes exercised or reported
unavailable, causal classes from the schema enum, explicit unavailable
evidence, and the routed `mount_points` item recorded as routed rather than
closed. `ROUTED_OPEN` is never closure. End with exactly one line:
`OVERALL_VERDICT: ACCEPT` or `OVERALL_VERDICT: REQUEST_CHANGES`.
`ACCEPT_WITH_NOTES` is not permitted.

Commit only that report path, leave the worktree clean, stop every child
process, and return the report commit, tree, path, byte count, SHA-256, and
the exact validation results.

Acceptance of this report authorizes no integration. The manager then writes
the final `docs/development/ai/review-recurrence/` record against
`docs/development/ai/schemas/review-recurrence-control.v1.schema.json` with
`generation: 3`, `record_status: "final"`,
`coverage.finding_target_policy: "none"`,
`repair_packet_timing: "dispatch_event_attested"`, and runs:

```bash
python3 docs/development/tools/check_review_recurrence.py \
  --record <final-record.json> \
  --git-root .
```

`--no-git-verify` is diagnostic only and can never return `ALLOW`. A
structurally valid `request_changes` record is still `BLOCK`.

## Launch preconditions

1. The AIA-047 gate independent review
   (`cfr168-review-recurrence-gate-independent-review-20260731.md`, candidate
   `1cae0225e488ff918389700a7de0f94e9fd067b3`) has committed its report and
   returned `OVERALL_VERDICT: ACCEPT`, and the manager has accepted it. As of
   this packet, no report exists at
   `docs/development/ai/swarm-results/cfr168-review-recurrence-gate-independent-review-claude-20260731/review.md`,
   so **launch is blocked**. A `REQUEST_CHANGES` verdict there requires this
   packet to be re-frozen against the repaired gate before any dispatch.
2. Live resource admission grants a slot under the recovery execution
   ledger's floor.
3. A fresh reviewer session is created — not the implementer session
   `f7692f47-4af1-4f84-a186-d57ddd4b083e` — in a clean detached worktree at
   exactly `74cd37d5e9204c45ac23bf31aa6973d6004cc0f0`.
4. This packet is committed as the review-control commit whose sole parent is
   `74cd37d5e9204c45ac23bf31aa6973d6004cc0f0`, byte-identical to the copy on
   the preparation lane, and the manager records that commit, its tree, its
   canonical patch digest, its changed-path set, and this file's SHA-256 as
   the record's control-overlay identity.
