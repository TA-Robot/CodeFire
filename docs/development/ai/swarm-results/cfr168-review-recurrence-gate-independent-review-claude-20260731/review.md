# CFR-168 AIA-047 Successor Review Recurrence Gate — Independent Review

Reviewer: Claude Opus 5 (`claude-opus-5`), independent report-only lane.
Packet: `docs/development/ai/task-packets/cfr168-review-recurrence-gate-independent-review-20260731.md`.
Date: 2026-07-31.

## 1. Recomputed Immutable Identity

Every identity below was recomputed in this worktree before analysis. All match
the packet.

| Item | Packet value | Recomputed | Match |
|---|---|---|---|
| Base revision | `a22fad0698b1623d8898c47699e7ba4e61294b5f` | `git rev-parse` → same | yes |
| Candidate revision | `1cae0225e488ff918389700a7de0f94e9fd067b3` | `git rev-parse` → same | yes |
| Candidate tree | `9c0a46a2c27ed3d54f3e1aa475df8ab655caad3a` | `git rev-parse 1cae022^{tree}` → same | yes |
| Canonical binary patch SHA-256 | `9b52caec0a67e3c74b431a68486cb01abb4a38e04df71e7934a06e468a6a06a4` | `git diff --full-index --binary a22fad0 1cae022 \| sha256sum` → same | yes |
| Candidate commits | `5fa48c9`, `bf284b7`, `1cae022` | `git log --oneline a22fad0..1cae022` → same three, in that order | yes |
| Changed-path count | 11 | `git diff --name-only a22fad0 1cae022 \| wc -l` → 11 | yes |

The digest is only reproducible with `--full-index`; plain `git diff --binary`
yields `950c03b6b1f7884c43c57967c76377dce9e131bd86b8d91d10e9a6dc5b66dd08`. This
matches `ai_contracts.canonical_git_patch`, which passes `--binary
--full-index --no-ext-diff --no-textconv --no-renames --src-prefix=a/
--dst-prefix=b/`, so the packet digest and the tool's own canonical form agree.

Numstat over the 11 paths: `+3090 / -0`, no binary paths. Largest additions:
`review-recurrence-control.v1.schema.json` (+1052),
`check_review_recurrence.py` (+1300), `test_ai_tools.py` (+518).

Review-control revision at review start: `7f7a72df939cf8d46ef9637307bc5ae26d33dc96`
(`docs(ai): bind recurrence gate independent review`, sole change = the packet).
Worktree clean at start (`git status --porcelain` empty).

## 2. Verdict Summary

The Git-binding half of this gate is real and strong. The chronology and
evidence halves are not: two of the mechanism's own documented guarantees are
falsified by executable counterexamples, and the flagship exact-changed-path
guarantee is defeated by changing one unconstrained field. I recommend
REQUEST_CHANGES with a small, bounded repair list — the architecture is sound
and none of the repairs below require redesign.

**Strongest root-cause fit.** The `1cae022` addition of exact changed-path
coverage (`repair-path-coverage-mismatch`) is the single most valuable part of
the candidate. It converts "did you look at the repair?" from a prose claim
into a set equality against `git diff --name-only -z`, and it holds against
binary paths and Git names containing newlines (verified: a candidate touching
`bin.dat` and `weird\nname.txt` produces `ALLOW` only when `binary_paths`,
`inserted_lines`, and every coverage row match Git exactly). Combined with the
report-commit write-set check (`report_changed != {report_path}`) and the
single-parent-of-control ancestry check, the "initial review scope miss" and
"repair-introduced regression went unlooked-at" causes are genuinely
constrained for the first time. Thirty-nine separate fail-closed
counterexamples all returned `BLOCK` with exit 3.

**Strongest bypass / false-closure concern.** `subject.base_revision` is
unconstrained. It must only be an ancestor of the candidate. There is no field
anywhere in the schema recording the predecessor generation's candidate or
accepted revision, so nothing pins the reviewed interval to the actual repair.
Moving the base forward past the risky repair commit shrinks the Git changed-path
set, and the coverage set-equality check then dutifully confirms "exact"
coverage of the narrowed set. The hard surface disappears from the record with
zero findings emitted. This defeats the mechanism's headline guarantee with a
one-field edit and is the finding I would fix first.

## 3. Prior Guarantees That Are Actually Established

These are executably confirmed (51 cases in probe 1, plus probes 2–4; all
exercised the real `check_review_recurrence.main` entry point against
throwaway repositories outside this tree).

1. **Non-`accept` and `--no-git-verify` never authorize integration.**
   `request_changes` on an internally consistent record returns
   `record_valid: true` with `integration_decision: BLOCK` and exit 3 — the
   packet's required separation of record validity from integration authority
   holds exactly. `block` and `--no-git-verify` likewise `BLOCK`.
2. **Candidate Git identity.** Mismatched or unsafe declared changed paths,
   candidate tree, patch digest, control tree, control patch digest, repair
   packet digest, review packet digest, report digest, and numstat
   inserted/deleted/binary sets each produce a distinct `BLOCK`.
3. **Exact changed-path coverage.** Omitting one coverage row →
   `repair-path-coverage-mismatch`. Extra or unsafe declared path →
   `candidate-changed-paths-mismatch`.
4. **Report/control confinement.** A report commit carrying an extra product
   change → `report-commit-write-set-mismatch`. A merge-commit report →
   `report-git-binding-failed` ("must be a single-parent direct child of review
   control"). A control revision touching product code →
   `review-control-touches-non-control-path`. A report path outside
   `docs/development/ai/swarm-results/` → `report-git-binding-failed`.
5. **Disposition consistency.** Blocking findings cannot be routed; open,
   unavailable, and unrouted-but-authority-claiming prior findings cannot ride
   an `accept`; unreviewed and blocked repair paths and surfaces cannot;
   `public_api`/`state_machine`/`classifier`/`persisted_shape` surfaces need an
   explicit scope decision; unplanned surfaces need one too.
6. **Validation terminality.** `status: passed` with `terminal: false`, a
   non-zero exit code, a null `log_ref`, or a null `log_sha256` each `BLOCK`;
   a required validation that is `blocked` or `unavailable` `BLOCK`s.
7. **Terminalization.** Live worker, non-zero active children, uncommitted or
   missing report, and missing event-log ref/digest each `BLOCK`.
8. **Derived counts cannot be fabricated.** All 15 count fields in `outcomes`
   are recomputed from the record's own rows; observation IDs and effect
   episode IDs are counted distinctly and cannot overlap across sections, so
   reworded duplicates are visible rather than rewarded.
9. **No finding quota, direct or indirect.** `coverage.finding_target_policy`
   is `const: "none"`, `additionalProperties: false` rejects a smuggled
   `target_finding_count`, and no threshold on any count appears in the
   validator. This part of the design is honest: I found no count proxy.

## 4. Findings

### F-1 (High) — `base_revision` is unconstrained, so "exact changed-path coverage" is exact only over a manager-chosen interval

`check_review_recurrence.py:660-733` recomputes the changed-path set from
`canonical_git_patch(root, subject.base_revision, subject.candidate_revision)`
and requires `changed_path_coverage` to equal it. `canonical_git_patch`
(`ai_contracts.py:355-362`) only requires the base to be an *ancestor* of the
head. The schema has no `predecessor_candidate_revision` or equivalent, so
there is no field that could bind the base to the previous generation.

**Reproducer** (probe 2). Repair series of two commits on top of
`TRUE_BASE`: `MID` adds `risky.txt`, `CAND` adds `cosmetic.txt`.

- `base_revision = TRUE_BASE` → coverage `['cosmetic.txt', 'risky.txt']` → `ALLOW`.
- `base_revision = MID` → coverage `['cosmetic.txt']` → `ALLOW`, zero findings.

`risky.txt` never appears in the record. Every "exact", "must equal the Git
diff", and "omission blocks integration" claim in
`validation-report-template.md` and `operating-protocol.md` is true only
relative to the base the record author chose.

**Smallest repair boundary.** Add `subject.predecessor_candidate_revision`
(the revision the previous generation reviewed) to the schema; in
`verify_git_subject`, resolve it and require `base_revision ==
predecessor_candidate_revision` and that it is an ancestor of the candidate.
Roughly one schema field and eight lines in the tool.

### F-2 (High) — `ancestor_of_candidate` does not establish that the repair contract preceded the repair

`check_review_recurrence.py:922-952` tests
`git merge-base --is-ancestor repair_packet_revision candidate`. That predicate
is satisfied by every commit in `base..candidate` including the candidate
itself (a commit is its own ancestor), so a repair packet first written *after*
the code is accepted as the pre-repair contract.

**Reproducer** (probe 1, case 42). Branch: `ROOT` → commit A adds
`product.txt` + `other.txt` (the repair) → commit B adds
`docs/development/ai/task-packets/repair.md` ("retro-fit the repair
contract"). Set `base_revision = ROOT`, `candidate_revision = B`,
`repair_packet_revision = B`, `repair_packet_timing = "ancestor_of_candidate"`.
Result: `ALLOW`, exit 0, zero findings — with a repair contract that
demonstrably did not exist when the code was written.

This directly falsifies `operating-protocol.md`: "A post-candidate review
prompt therefore cannot masquerade as the pre-repair communication contract",
and `review-recurrence/README.md` step 1: "Its commit either precedes the
candidate in Git ancestry".

**Smallest repair boundary.** Require
`merge-base --is-ancestor repair_packet_revision base_revision` (not
`candidate`) and read the packet blob at `base_revision` rather than at
`repair_packet_revision`. Two lines changed.

### F-3 (High) — `dispatch_event_attested` performs no timing check at all, and admits the post-candidate review packet as the pre-repair contract

`check_review_recurrence.py:953-960` requires only that
`repair_dispatch_event_ref` and `repair_dispatch_event_sha256` be non-empty
strings. Neither is resolved, hashed, compared, or ordered against anything.
Choosing this timing value also skips the ancestry check entirely, so the
repair packet may be a strict *descendant* of the candidate.

**Reproducer** (probe 1, case 43). Point `repair_packet_path` /
`repair_packet_revision` at the review packet committed in the review-control
revision — i.e. the post-candidate review prompt itself — set
`repair_packet_timing = "dispatch_event_attested"`,
`repair_dispatch_event_ref = "events:not-a-real-file"`,
`repair_dispatch_event_sha256 = "b"*64`. Result: `ALLOW`, exit 0, zero
findings. This is the exact scenario `operating-protocol.md` claims is
impossible.

The candidate's own test suite codifies this: `test_ai_tools.py` (added lines,
`test_git_binding_covers_candidate_control_report_and_task`) sets a fabricated
`"b"*64` digest for an off-branch packet and asserts
`validate_recurrence_control(...) == []`. So the hole is intended behaviour,
but it is documented as a proof and it is not one.

**Smallest repair boundary.** Either (a) resolve the dispatch event through
`ai_contracts.read_repository_artifact`, re-hash it, and require a recorded
dispatch timestamp strictly before the first commit in `base..candidate`; or
(b) keep it as an unverified operator assertion, state that limit precisely in
`review-recurrence/README.md` and `operating-protocol.md`, and require a
separate manager-authority field before `ALLOW`. Option (b) is a documentation
change plus one accept-path condition.

### F-4 (High) — no evidence reference or log digest is ever resolved; "content-addressed evidence" is a format check only

`evidence_refs` is inspected in exactly three places
(`check_review_recurrence.py:198`, `202`, `214`) and only for
`len(...) == 0`. `log_ref`, `log_sha256`, `event_log_ref`, and
`event_log_sha256` are checked only by `nonempty()` plus the schema's
`^[0-9a-f]{64}$` pattern. `ai_contracts.read_repository_artifact` exists for
exactly this purpose and has no consumer anywhere in
`docs/development/tools/`.

**Reproducer.** Every `ALLOW` in probe 1 uses `log_sha256 =
event_log_sha256 = "a"*64` with `log_ref = "log:focused"`. Probe 1 case 48
reduces every prose array to the single character `"x"` — `declared_surfaces`,
`required_production_routes`, `anticipated_regressions`,
`forbidden_shortcuts` — and still returns `ALLOW`, exit 0, zero findings.

Consequences for the packet's stated root causes: the "weak oracle" cause is
not addressed at all. `oracle.property`, `independence_basis`,
`assertion_boundary`, `positive_control_refs`, and the negative control's
`rationale` are unverified prose; the only enforced fact about a killed
negative control is that a non-empty string array exists. `production_routes_checked`
is a boolean, and `closure_contract.required_production_routes` is never
cross-checked against `validations`, `changed_paths`, or anything else.

**Smallest repair boundary.** `run_timed_gate.py` already emits
`codefire.timed_gate.v1` with `exit_code`, `log.path`, and `log.sha256`. Add
`validations[].timed_gate_result_path`; in the tool, `load_object` it (already
imported), then require its `exit_code` and `log.sha256` to equal the declared
ones and re-hash the log via `read_repository_artifact`. About 30 lines, and it
converts the largest self-attested block in the record into machine-checked
evidence.

### F-5 (Medium) — `severity` is dead: a `severity: "block"` finding routes into an `ALLOW`

`severity` appears twice in the schema and zero times in
`check_review_recurrence.py` (`grep -n severity check_review_recurrence.py`
→ no match). Only the separate, freely-authored `blocking` boolean gates. The
`route_authority` that supposedly licenses the route is an unvalidated string.

**Reproducer** (probe 1, case 41; probe 3 confirms without Git verification).
New finding with `severity: "block"`, `blocking: false`,
`attribution: "created_by_candidate"`,
`causal_classes: ["repair_introduced_regression"]`,
`disposition: "routed_open"`,
`route_authority: "manager:self-asserted"`. Result: `ALLOW`, exit 0, zero
findings — a block-severity, repair-introduced regression integrated by
setting one boolean to `false`. `review-recurrence/README.md`'s "A blocking
finding ... cannot produce `ALLOW`" is technically true of the boolean and
misleading about the record.

**Smallest repair boundary.** Add `severity == "block" ⇒ blocking is True` and
`severity in {"block", "high"} ⇒ disposition != "routed_open"` to
`validate_new_findings`. Four lines. Separately, require `route_authority` to
name a committed artifact path plus revision so the tool can resolve it.

### F-6 (Medium) — `continuity` is dead, including `recurred_after_proven_closure`

`continuity` never appears in `check_review_recurrence.py`. For a gate whose
entire purpose is *recurrence*, the enum value that names a recurrence after a
previously proven closure has no effect on any disposition or on the decision.

**Reproducer** (probe 3). Set
`prior_findings[0].continuity = "recurred_after_proven_closure"` on an
otherwise valid accept record → zero findings. Same for `"unknown"`.

**Smallest repair boundary.** In `validate_prior_findings`, require
`continuity == "recurred_after_proven_closure"` to carry `blocking: true` and
an evidence reference to the generation that previously proved closure, and
forbid `routed_open` for it. Six lines plus one schema field.

### F-7 (Medium) — per-path coverage is exact; per-symbol coverage is not required at all

`repair_delta.changed_surfaces` has `minItems: 1` and each surface's path must
appear in `changed_path_coverage`, but no rule requires the converse. One
surface row satisfies a candidate of any size, so every changed path but one
can be marked `reviewed` with no declared symbol.

**Reproducer** (probe 1, case 40). Candidate changes `product.txt` and
`other.txt`; both covered and `reviewed`; `changed_surfaces` declares only
`product.txt:final_fence`. Result: `ALLOW`, exit 0. Applied to this
candidate's own 11 paths and ~3,090 added lines, one surface row would satisfy
the gate. Relatedly, `newSurface.kind` is self-declared, so relabelling a
public API as `private_symbol` avoids the scope-approval path entirely (probe
1, cases 49 vs 50: identical records, `ALLOW` vs `BLOCK`).

**Smallest repair boundary.** Require each `changed_path_coverage` row with
`disposition: "reviewed"` to be referenced by at least one `changed_surfaces`
row, or to carry an explicit `no_surface_change` justification with evidence.
About twelve lines and one schema field.

### F-8 (Medium) — the review-control revision may rewrite the gate implementation and its own schema

`CONTROL_PATH_PREFIXES` (`check_review_recurrence.py:42-46`) admits
`docs/development/tools/` and `docs/development/ai/schemas/`, so the control
commit that parents the report may modify `check_review_recurrence.py`,
`ai_contracts.py`, and `review-recurrence-control.v1.schema.json` themselves.

**Reproducer** (probe 1, case 45). Control commit adds the review packet *and*
replaces `docs/development/tools/check_review_recurrence.py` with
`print('ALLOW')` and the schema with `{}`. Result: `ALLOW`, exit 0, zero
findings. The gate cannot attest that it and its schema were unmodified between
the candidate and the report.

**Smallest repair boundary.** Exclude `check_review_recurrence.py`,
`ai_contracts.py`, and `review-recurrence-control.v1.schema.json` from the
permitted control write set, or record and verify their blob digests at the
candidate revision. Six lines.

### F-9 (Medium) — the documented record namespace cannot be committed in the candidate→report window, and no evidence artifact can be either

`review-recurrence/README.md` places final records in
`docs/development/ai/review-recurrence/`, which is absent from
`CONTROL_PATH_PREFIXES`. Combined with the requirement that the report
commit's write set be exactly `{report_path}`, this means: (a) committing the
record alongside the review packet yields
`review-control-touches-non-control-path`; (b) *no* log, transcript, or
evidence artifact can be committed anywhere between the candidate and the
report, so the digests in F-4 can never refer to an artifact that was in the
repository when the record froze the report. The mandatory ordering
(candidate → control → report → record) is nowhere stated.

**Smallest repair boundary.** State the ordering explicitly in
`review-recurrence/README.md`, and either add
`docs/development/ai/review-recurrence/` to the control prefixes or add a
dedicated evidence prefix so logs referenced by `log_ref` can be committed
before the report.

### F-10 (Low) — a non-UTF-8 Git path name makes the gate exit 2 with an opaque codec error

`canonical_git_patch` decodes Git path bytes with `surrogateescape`
(`ai_contracts.py:383-387`), and the new schema puts those paths in
`stringArray` with `uniqueItems: true`, which routes them through
`canonical_json_bytes` — `json.dumps(..., ensure_ascii=False).encode("utf-8")`
(`ai_contracts.py:36-37`). A lone surrogate raises `UnicodeEncodeError`, a
`ValueError` subclass, caught by `main`'s handler.

**Reproducer.** Record containing
`subject.changed_paths = ["latin\udcff.txt"]` →
`Review recurrence gate error: 'utf-8' codec can't encode character '\udcff'
in position 6: surrogates not allowed`, exit 2. It fires during schema
validation, before any Git binding.

This is fail-closed (exit 2 is not `ALLOW`), but no decision can ever be
produced for such a candidate and the operator sees a codec error rather than a
finding. The root behaviour is pre-existing in `ai_contracts`; the candidate is
the first consumer that feeds it Git-derived surrogate-escaped paths, so the
smallest repair boundary is in the new gate: reject non-UTF-8 changed paths
with an explicit finding before schema validation. Binary paths and names
containing newlines are handled correctly (verified `ALLOW` on a candidate with
`bin.dat` and `weird\nname.txt`), so this is narrowly the invalid-UTF-8 case.

### F-11 (Low) — the report may be a symlink

`git_output(root, "show", f"{report_commit}:{report_path}")` returns the link
target for a mode-120000 entry, and `git diff --name-only` lists the symlink
path, so both the digest and write-set checks pass.

**Reproducer** (probe 1, case 44). Commit `report_path` as a symlink to
`../../../../../../../../etc/hostname`; `git ls-tree` confirms mode `120000`.
Result: `ALLOW`, exit 0. The "committed report" is 30 bytes of link target.

**Smallest repair boundary.** Read the tree entry mode with
`git ls-tree -z --format=... report_commit -- report_path` and require
`100644`. Three lines. The same applies to the two packet paths.

### F-12 (Low) — an intermediate commit between candidate and control can mutate and revert product code invisibly

Control-path confinement is checked against the *net* diff
`candidate..control`. A commit that replaces `product.txt` followed by one that
restores it leaves an empty net diff.

**Reproducer** (probe 1, case 46) → `ALLOW`, exit 0. Low severity: the final
tree is byte-identical to the candidate, so nothing integrable changed. It does
mean the record cannot attest that the candidate tree was never disturbed
during review. `git rev-list candidate..control` plus a per-commit path check
would close it.

### F-13 (Low) — `disposition: "unavailable"` never constrains the observations

`validate_prior_findings` constrains `after.status` for `closed`, `open`, and
`routed_open`, but not for `unavailable`. So `disposition: "unavailable"` with
`after.status: "verified"` is a self-contradictory record that reports
`record_valid: true` under `request_changes` (probe 3). Harmless for `ALLOW`
(the accept path rejects `unavailable`), but it weakens the record as
management evidence. Add the same `after.status in {"open", "unavailable"}`
check used for `open`. Two lines.

`coverage.out_of_scope` is likewise never cross-checked against
`changed_paths`, so a path can be simultaneously declared out of scope and
`reviewed` (probe 2, case C → `ALLOW`).

### F-14 (Low) — the Git-binding test asserts self-consistency, not the canonical patch definition

`test_git_binding_covers_candidate_control_report_and_task` computes the
expected `patch_sha256` and `changed_paths` with the same
`canonical_git_patch` the validator calls, so changing the canonical diff flags
would keep the test green. The mutation assertions in that test (coverage
omission, off-branch packet, numstat, report digest) are genuine and each
targets a distinct code path; this is specifically about the digest definition.
In practice the definition is right — the packet's independent digest
`9b52caec…` matches `--full-index --binary`, which I recomputed. Pinning one
byte-level expected digest for a fixed fixture repository would close the gap.

## 5. Contract / Tool / Test / Document Agreement

- **Schema vs validator.** Every `disposition` value in every enum has a
  validator branch, with two exceptions: `changedPathCoverage.disposition` and
  `newSurface.disposition` treat `blocked` and `unreviewed` identically, so a
  `blocked` path is reported as `accept-with-unreviewed-repair-path` (cosmetic).
  Two enums are entirely inert — `severity` (F-5) and `continuity` (F-6) — and
  one disposition leaves its observations unconstrained (F-13). All other
  required fields are either validator-enforced or deliberately
  schema-only-enforced (`record_status`, `finding_target_policy`,
  `stoppingRule.method`), which is consistent.
- **Tests.** `ReviewRecurrenceTests` — 11 tests, all pass (exit 0). Full
  `test_ai_tools` — 76 tests, all pass (exit 0). The mutation tests do fail for
  the mutations they claim: I re-derived each one independently through the CLI
  rather than the shared helper. F-14 is the one place a shared helper masks a
  possible drift. There is no test for F-1, F-2, F-5, F-6, F-7, F-8, or F-11,
  and the test for F-3 asserts the permissive behaviour.
- **Documents.** `review-recurrence/README.md` is admirably honest about
  promotion ("The pilot is not evidence that the bottleneck is solved"), and
  `schemas/README.md`'s "A recurrence record is valid management evidence, not
  automatically an approval" is exactly right and executably true. Two claims
  are not established by the code: `operating-protocol.md`'s "A post-candidate
  review prompt therefore cannot masquerade as the pre-repair communication
  contract" (F-2, F-3) and the repeated "content-addressed"/"exact evidence"
  framing for log and evidence references that are never resolved (F-4).
  `acceleration-master-plan.md` correctly claims no speedup yet.
- **Pilot operability.** The Cargo and KGI successor pilots are operationally
  possible without fabricating evidence, provided the repair packets are
  written before those repairs start — which is the case, since neither repair
  has begun. `run_timed_gate.py` can supply real exit codes and log digests
  today. F-9 is the one genuine operability trap: the record and any evidence
  artifact must be committed strictly after the report commit, and that
  ordering is undocumented.

## 6. Operator Burden

Measured from the schema's own `required` closure: **76 required leaf fields**
for the smallest possible record (1 prior obligation, 1 coverage row, 1
surface, 1 validation). Per-item costs: `subject` 23, `outcomes` 19,
`coverage` 11, `terminalization` 7, each prior obligation 31, each validation
10, each surface 9, each coverage row 5.

For a record describing *this* candidate — 11 changed paths, say 3 prior
obligations, 6 surfaces, 5 validations — that is roughly **320 hand-authored
fields**. Of those, the machine-checked subset is: 8 Git identities, 4 blob
digests, the coverage-path set equality, the numstat/binary triple, control
path confinement, report parentage and write set, and the cross-field
disposition rules. Everything else — every `closure_contract` array, the whole
`oracle` object, `declared_surfaces`, `out_of_scope`, `review_focus`, every
`evidence_refs`, every `log_sha256`, all six `stopping_rule` booleans, every
`severity`, `kind`, `continuity`, and `planned` flag — is unverified prose or a
self-set boolean.

Two burdens are pure make-work:

- **`outcomes` (19 fields).** The tool recomputes 15 of them from the record's
  own rows and then requires the author to have transcribed them identically.
  This detects only the author's own transcription slips; it adds no guarantee,
  because the tool already holds the ground truth. It should emit them, not
  demand them.
- **`closure_contract` (5 prose arrays × every obligation) and `oracle` (4
  prose fields × every obligation).** 31 required fields per obligation, of
  which the validator reads 6.

The stopping rule is not a saturation proof. `validate_coverage` checks six
booleans (`check_review_recurrence.py:122-143`). Setting all six to `true` is
the entire cost of "coverage saturation"; the only real saturation constraint
in the whole design is the changed-path set equality, and F-1 shows that is
relative to a chosen base. So the honest answer to the packet's question — "can
an author set booleans and omit the difficult surface?" — is yes, via F-1 and
F-7.

**Smallest credible alternative with the same established guarantees.** Roughly
a third of the size:

1. **Keep** the `subject` Git block (add `predecessor_candidate_revision`),
   `changed_path_coverage` as `{path, disposition, evidence_refs}`, the
   `validations` table, `terminalization`, `review_decision`, and the
   cross-field disposition rules. This is the entire verified core.
2. **Reduce** obligations to
   `{id, origin_generation, continuity, blocking, disposition, before.status +
   evidence, after.status + evidence, negative_control.status + evidence,
   owner, release_condition, route_authority}` — 13 fields instead of 31 — and
   move `closure_contract` and `oracle` prose into the repair packet and the
   review report, which is where prose belongs and where a human already reads
   it.
3. **Drop** `outcomes` entirely; have the tool emit the same 15 counts into its
   result JSON. Drop `coverage.declared_surfaces`, `out_of_scope`, and the six
   `stopping_rule` booleans; the changed-path set equality plus per-path
   surface linkage (F-7) is the only stopping rule that means anything.
4. **Spend the savings** on F-4: bind `validations` to
   `codefire.timed_gate.v1` results and resolve repository-relative
   `evidence_refs`. Trading ~130 unverified fields for ~30 verified ones is the
   right trade for a gate whose stated purpose includes closing a weak oracle.

Estimated result: ~110 fields for this candidate instead of ~320, with strictly
more machine-checked content. On the packet's proportionality question: the
~2,870 lines are not justified by the guarantees *currently* established, but
they would be by the guarantees the same structure could establish with F-1
through F-4 fixed. The structure is not the problem; the unverified surface
attached to it is.

## 7. Validation Performed

All commands run from this worktree. Focused counterexamples first, as the
packet requires. Every result below is a real terminal exit status, not
background progress.

| Command | Exit | Result |
|---|---|---|
| probe 1: 51 counterexamples through `check_review_recurrence.main` | — | 39 `BLOCK` as expected, 12 `ALLOW` as expected; `unexpected=0` |
| probe 2: base narrowing, `out_of_scope`, symlink mode | — | 3 `ALLOW`, mode `120000` confirmed |
| probe 3: dead-field probes without Git verification | — | 6 cases, all "NO FINDING" |
| probe 4: binary path, newline name, invalid-UTF-8 name | — | `ALLOW` / `ALLOW` / exit 2 codec error |
| `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest test_ai_tools.ReviewRecurrenceTests` | 0 | Ran 11 tests, OK |
| `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest test_ai_tools` | 0 | Ran 76 tests, OK |
| `python3 docs/development/tools/check_ai_contracts.py` | 0 | AI contract check passed. |
| `python3 docs/development/tools/check_ai_acceleration_docs.py` | 0 | AI acceleration docs check passed. |
| `python3 docs/development/tools/check_project_docs.py` | 0 | Project docs check passed; 651 Markdown files inventoried. |
| `python3 docs/development/tools/check_ai_development.py` | 0 | PASS source_docs / runtime / tool_tests / validation_capabilities |
| `git diff --check` | 0 | clean |

The unittest commands were run from `docs/development/tools`. The four gate
scripts were run from the repository root. Counterexample repositories were
created under `/tmp` and never inside this tree; no Cargo, Rust, Docker, or
unrelated product suite was run.

## 8. Explicitly Unavailable Evidence

- **Predecessor review reports** for the AIA-046 source-keyed Cargo gate repair
  and the development-flow KGI/KPI repair were not read (outside the packet's
  11-path scope). I therefore cannot judge whether the obligations those two
  pilots must bind are expressible in `closure_contract`, nor whether 31
  required fields per obligation is tolerable at their real obligation count.
- **Cost comparison against avoided rework** is unmeasurable here. No
  historical record of recurrence-loop cost exists in this tree, so the
  packet's "less costly than the rework it targets" question can only be
  answered on the burden side (§6), not on the benefit side.
- **The blind causal classification** that `review-recurrence/README.md` makes
  a promotion precondition is scheduled for on or after 2026-08-03 and has not
  occurred.
- **No live pilot run.** I did not exercise the gate against a real Cargo or
  KGI candidate; the packet forbids starting those reviews. All executable
  evidence comes from synthetic repositories.
- **`dispatch_event_attested` in real use.** I could not test it against a real
  dispatch event log, because the tool never reads one; F-3 is therefore a
  finding about absent verification, not about a specific log format.
- **Concurrency and filesystem races.** I did not test `atomic_write` under
  concurrent invocation or a read-only `--result` parent directory.

## 9. Recommendation

Fix F-1 through F-4 before either pilot run. They are small — one schema field
and eight lines for F-1, two lines for F-2, a documentation correction plus one
condition for F-3, and roughly thirty lines for F-4 reusing
`run_timed_gate.py` output that already exists. F-5 through F-9 are worth
folding into the same pass; F-10 through F-14 can follow.

Until F-2 and F-3 are resolved, `operating-protocol.md`'s claim that a
post-candidate prompt cannot masquerade as the pre-repair contract should be
removed, because a manager reading it would reasonably believe the gate
establishes chronology that it does not.

To answer the packet's framing question directly: this is not merely report
volume and another self-attested approval path — the Git-binding, write-set,
terminality, and count-reconciliation layers are real, and the refusal of a
finding quota is honest and complete. But it is not yet a gate that a known
recurrence would fail, because the reviewed interval, the pre-repair
chronology, and all evidence content remain author-chosen. Acceptance here
would authorize no promotion in any case; the manager owns the pilot decision.

OVERALL_VERDICT: REQUEST_CHANGES
