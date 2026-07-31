# CFR-168 Recurrence Gate v2 Repair Contract — Independent Adversarial Preflight

Reviewer: Claude Opus 5 (`claude-opus-5`), independent contract-preflight lane
`cfr168-review-recurrence-v2-contract-preflight-claude-20260731`, report-only.
Date: 2026-07-31.
Subject: the **contract**, not an implementation. No implementation of v2 exists
at this base, and this lane wrote no schema, tool, test, product, or prior report
byte.

## 1. Exact identity

| Item | Value | How established |
|---|---|---|
| Task base / review head | `a79acc0acc50f96868860fd7db5eef051488e420` | `git rev-parse HEAD` |
| Branch | `claude-agent/cfr168-review-recurrence-v2-contract-preflight-claude-20260731` | `git rev-parse --abbrev-ref HEAD` |
| Worktree at start | clean (`git status --porcelain` empty) | recomputed |
| Contract under review | `docs/development/ai/task-packets/cfr168-review-recurrence-gate-v2-repair-20260731.md` | — |
| Contract SHA-256 | `15931ddbdeae51806cfff619ce4a86e241216b8fcb7993569a46a194f9f3f138` | `sha256sum` |
| Rejected candidate | `1cae0225e488ff918389700a7de0f94e9fd067b3` | `git cat-file -t` → `commit` |
| Rejected candidate base | `a22fad0698b1623d8898c47699e7ba4e61294b5f` | `git cat-file -t` → `commit` |
| Rejected candidate tree | `9c0a46a2c27ed3d54f3e1aa475df8ab655caad3a` | `git cat-file -t` → `tree` |

Governing evidence named by §2 of the contract, digests recomputed here:

| Artifact | Recomputed SHA-256 | Declared in contract? |
|---|---|---|
| `…/cfr168-review-recurrence-gate-independent-review-claude-20260731/review.md` | `02bc2030728fd2c471237dd1a5da8eca268c07836c7d5c494bb73b2050ffb05b` | **no digest declared** |
| `…/cfr168-review-recursion-root-cause-audit-claude-20260731/audit.md` | `2392450e957c8437b323ded492fc7597dc77f5147b74f3056c97c0efd9329780` | **no digest declared** |
| `…/cfr168-review-intervention-retroactive-pilot-claude-20260731/pilot.md` | `527b800cd29384ff5f2a1e213e27226eb794e10d247329a088d85679994ab553` | declared, **matches** |
| `…/swarm-batches/cfr168-review-intervention-retroactive-pilot-20260731.md` | (read in full) | path only |

The audit digest also equals the value the pilot packet binds for it, so the audit
is byte-unchanged since `abfee6e`. The independent review's verdict line reads
`OVERALL_VERDICT: REQUEST_CHANGES`, as §2 states. All four artifacts were read in
full. I additionally read, read-only, `AGENTS.md`,
`docs/development/tools/check_review_recurrence.py` (constants and result
emission), `run_timed_gate.py`, `check_ai_contracts.py`,
`check_ai_acceleration_docs.py`, `check_ai_development.py`,
`ai_contracts.py` (encoding helpers) and
`docs/development/ai/review-recurrence/README.md`, because several contract
clauses are only decidable against the artifacts they will bind.

## 2. Verdict summary

The contract is a real improvement on the rejected v1 surface in four places, and
those four are worth stating before the findings, because they should survive any
revision: the free `base_revision` is gone (§4.1), `dispatch_event_attested` is
**removed** rather than documented (§4.2), the review-control commit is confined
to a single path by a direct-parent rule (§4.3), and the transcribed `outcomes`
block is replaced by derived output (§8.3). §5 also states its own provenance
boundary honestly — "It proves that the manager record, timed result, and
retained bytes agree" — which is exactly the sentence v1's documentation was
missing.

It is nevertheless not yet implementable as written, and it is not sufficient
against F-1 through F-14. Three classes of defect:

1. **One F-finding is closed by renaming, which §9 of the contract itself
   forbids.** F-1's interval floor moved from `base_revision` to
   `predecessor_candidate_revision` and is still constrained only by ancestry, so
   the narrowing bypass survives intact — and the contract's own required
   counterexample §11(1) cannot be produced against §4.1 as specified (C-1).
2. **The two chronology mechanisms depend on a path taxonomy the contract never
   defines**, and the only enumeration that exists in the repository
   simultaneously blocks the contract's own first candidate and admits a
   control-plane repair before its packet — which is the class of candidate all
   three named live subjects belong to (C-2).
3. **The evidence layer binds bytes but not provenance or authorship.** Every
   "exact evidence" and "exact committed manager authority artifact" obligation is
   dischargeable by a blob the reviewing lane writes in the evidence commit that
   the record itself declares, and the one accept-eligible evidence state is the
   one whose authority is unverifiable (C-3, C-4). `no_surface_change` is an
   unconstrained escape hatch that satisfies §8.2 with zero reviewed surfaces
   (C-5).

On the two live experiments: I1 is faithful on the point the pilot corrected
(derivation, not differential) but widened beyond the measured obligation and
stripped of the pilot's Rust caveat (C-10). I5 is faithful on vocabulary and on
both properties the pilot required be preserved — `DEFECT_SOURCE_PROVEN` is added
for the exact gap the pilot measured, `CLOSURE_UNPROVEN` never authorizes, and
`ACCEPT_WITH_NOTES` is refused — but its authority rules keep five of the six
generations the pilot measured as I5's entire benefit blocking, so the metric §7
asks for cannot replicate the pilot's `6 / 18` and must be redefined before the
Cargo/KGI comparison is run (C-9). Neither infidelity is a reason to weaken
integration authority; both are reasons to state the change of measurement
explicitly.

CONTRACT_VERDICT is `REQUEST_CHANGES`, with a bounded edit list (§7). None of the
edits requires redesign; C-1, C-2, C-4 and C-5 are the four that must land before
any implementation byte, because each defeats a headline guarantee in §1.

## 3. Findings

Severity is the severity of the *contract* defect: High = a §1 guarantee is
false, unimplementable, or bypassable as written.

### C-1 (High) — §4.1 renames F-1's field without constraining it, and §11(1) then demands a counterexample §4.1 cannot produce

§4.1 removes `base_revision` and requires `predecessor_candidate_revision` plus
`candidate_revision`, with the tool required to "resolve both to canonical
commits" and "require the predecessor to be an ancestor of the candidate". That
is the same predicate v1 applied to `base_revision`. The reviewed interval floor
is still whatever revision the record author names, and **no clause anywhere in
the contract binds `predecessor_candidate_revision` to the revision generation
`N-1` actually reviewed.**

**Counterexample (the v1 bypass, unchanged).** Generation `N-1` reviewed
candidate `X`. The repair series is `X → MID` (the risky commit) `→ CAND`. The
author declares `predecessor_candidate_revision = MID`,
`candidate_revision = CAND`. `MID` is a canonical commit and an ancestor of
`CAND`, so §4.1(1) and (2) pass. §4.1(3) then computes tree, patch, numstat,
changed paths and **every repair coverage obligation** over `MID..CAND`, so the
risky surface is absent from the Git-derived set, §8.2's set equality confirms
"exact" coverage of the narrowed set, and no finding is emitted. §4.1(4) is
satisfied — there is no *second* field, because the *first* field does all the
narrowing. This is the independent review's F-1 reproducer with one identifier
substituted.

**Contradiction trace.** §9 states: "No finding may be marked 'fixed' only
because its field was renamed." §1(1) claims v2 "cannot narrow the reviewed
repair interval past the predecessor candidate" — which is a tautology, since the
interval floor *is* the declared predecessor candidate; the property that matters
is that the declared predecessor candidate is the one the previous generation
reviewed, and no clause asserts it. §11(1) then requires a counterexample in
which "base moved forward past a risky commit" is blocked by v2. Under the strong
reading (the reviewed interval is narrowed) §4.1 **allows** it, so the required
test cannot be written; under the weak reading (a stray `base_revision` key is
rejected by `additionalProperties: false`) the test passes but exercises the
field name rather than the bypass. Both readings are dead ends, which is itself
proof that §4.1 is incomplete.

**Minimum edit.** Require the predecessor identity to be resolved from a
committed, Git-bound prior-generation authority artifact — the predecessor
generation's own v2 record where one exists, otherwise that generation's
committed independent review report — bound by exact revision, path, SHA-256 and
mode `100644`, with the tool requiring equality between the artifact's declared
candidate revision and `predecessor_candidate_revision`. Because the first v2
generation has no predecessor v2 record (and §3 makes a v1 record `BLOCK` at the
entry point), the contract must also state the bootstrap explicitly: for the
first generation, name the exact predecessor artifact and revision **in the
contract text**, so the identity is manager-fixed rather than author-chosen. This
is the one edit whose absence makes every other Git guarantee in §4 conditional.

### C-2 (High) — §4.2's "explicitly enumerated manager-control paths" is undefined, and the only enumeration in the repository both blocks v2's own first candidate and admits a control-plane repair before its packet

§4.2(5) requires that when the packet revision is after the predecessor, "every
commit through and including that packet revision changes only the exact repair
packet path or explicitly enumerated manager-control paths". The enumeration is
never given. Two arms, both verified against this repository:

**Arm A — narrow enumeration blocks v2's own first candidate.** For the next
generation, `predecessor_candidate_revision` is the rejected candidate
`1cae0225` and the repair packet revision is `a79acc0`. That window contains
seven commits:

| Commit | Sole path touched | Namespace |
|---|---|---|
| `7f7a72d` | `…/task-packets/cfr168-review-recurrence-gate-independent-review-20260731.md` | task-packets |
| `c62ab97` | `…/swarm-results/cfr168-python-http-upload-third-repair-rereview-claude-20260731/review.md` | swarm-results |
| `6792f86` | `…/swarm-results/cfr168-python-http-shared-primitives-preflight-claude-20260731/preflight.md` | swarm-results |
| `0304c0e` | `…/task-packets/cfr168-source-keyed-cargo-third-closure-rereview-with-recurrence-20260731.md` | task-packets |
| `2bec224` | `…/swarm-results/cfr168-review-recurrence-gate-independent-review-claude-20260731/review.md` | swarm-results |
| `94f2ab4` | `…/swarm-results/cfr168-review-intervention-retroactive-pilot-claude-20260731/pilot.md` | swarm-results |
| `a79acc0` | the repair packet itself | task-packets |

Five of the seven are neither the repair packet path nor within
`CONTROL_PATH_PREFIXES` (`check_review_recurrence.py:41-45`:
`docs/development/ai/task-packets/`, `docs/development/ai/schemas/`,
`docs/development/tools/` — `swarm-results/` is absent). Two of the five are the
independent review and the pilot, i.e. the very artifacts §2 binds as governing
evidence. So on the narrow reading the gate `BLOCK`s its own first candidate for
containing its own evidence. This is F-9's operability trap reproduced in a new
place. It also generalises: in steady state, generation `N-1`'s review-control,
report-only and evidence commits mandated by §4.3 necessarily sit inside
generation `N`'s §4.2(5) window, so the window rule and the chain rule are in
direct tension for every generation after the first.

**Arm B — the natural broad enumeration reopens F-2 for exactly the candidates v2
will judge.** The one existing constant an implementer will reach for,
`CONTROL_PATH_PREFIXES`, contains `docs/development/ai/schemas/` and
`docs/development/tools/` — which is precisely v2's own implementation surface
per §10, and precisely where the two named live subjects live (the source-keyed
Cargo gate is tool files; KGI/KPI is `ai_contracts.py`). Compounding it, §4.2(6)
speaks of "the first candidate/product/test change" without defining that class.
For a control-plane candidate there is no product code and the repaired bytes are
tool bytes. **Counterexample:** commit the repaired `readonly_source_exec.py`
before the repair packet, classify `docs/development/tools/` as a manager-control
path, then §4.2(5) passes (only enumerated control paths were touched), §4.2(6)
passes (no `codefire`/`crates`/`tests` byte ever changes), and §4.2(7) passes
(the packet commit carries no "repair byte" under that taxonomy). F-2 is defeated
on the class of candidate the gate exists to judge, and §1(2)'s claim that v2
"proves that the repair packet existed before the first repair change" is false
for that class.

**Minimum edit.** Enumerate, in the contract, three disjoint path sets and their
roles: (i) the repair-packet path; (ii) the manager-control paths admissible in
the pre-repair window — and admit the report/evidence/record namespaces so the
steady state is satisfiable; (iii) the **repair surface** for the candidate under
review, defined as the candidate's own changed paths minus (i) and (ii), so that
"first repair change" is computed from Git rather than from a product/test
taxonomy that does not apply to control-plane candidates. State explicitly that
`CONTROL_PATH_PREFIXES` is not reusable as (ii) and that (ii) must exclude every
path in the candidate's changed-path set.

### C-3 (High) — §5 binds evidence bytes but not evidence provenance: no source identity, and no ancestry requirement for artifact references

Two independent gaps.

**(a) The timed-gate result carries no candidate identity.** `run_timed_gate.py`
(`run_gate`, lines 119-140) emits exactly `schema`, `argv`, `cwd`,
`timeout_seconds`, `timed_out`, `exit_code`, `status`, timing evidence, and
`log.{path,bytes,sha256,complete}`. There is **no revision, tree, source key, or
selected/executed count**, and `cwd` and `log.path` are absolute resolved paths.
§5 therefore cannot bind the run to the candidate, and the contract's checks
(schema string, digest equality, status/exit/timeout/log agreement) all pass for a
result produced in a different worktree, at a different revision, or against a
tree in which the assertion under test was never selected. That is not
hypothetical: it is the pilot's only `E1` instance and the corpus's one proven
`VALIDATION_PROVENANCE_FALSE_GREEN` — `cargo test … --locked live_file_doctor`,
exit 0 in 0.267 s, **0 selected / 794 filtered**, from a binary belonging to a
different Rust worktree (pilot §5.3; audit §5.5). §5's boundary sentence covers
honesty of execution; it does not cover *which tree was executed*, and §9's F-4
disposition ("Exact committed result/log/artifact binding") reads as though it
does.

Note that closing this is not "installing E1–E9 as a mechanical checklist", which
§2 forbids. `E1` is the single rule the pilot scored as mechanically decidable,
zero-noise, and *the reason a generation-3 review never launched*; §5's mechanism
is already the place it belongs.

**Minimum edit.** Require each timed-gate reference to bind the exact revision or
tree the gate executed against, and require the tool to verify that identity
against `candidate_revision`. Because `codefire.timed_gate.v1` cannot express it
today, the contract must say which route it takes — extend the emitter (a shared
tool outside §10's declared surfaces; see C-13), or require a separate committed
identity artifact per validation — rather than leaving the implementer to
discover the impossibility mid-implementation and trip §10's stop-and-return
rule. Also state, since it is not currently anywhere: `log.path` and `cwd` are
absolute and machine-specific, so only the digest links a committed log blob to
its result.

**(b) Artifact references need not be reachable.** §5 requires that all other
evidence and authority references "bind exact revision, path, SHA-256, and
regular-file mode". Nothing requires the revision to be on candidate ancestry, or
reachable at all. **Counterexample:** create the authority artifact on a
throwaway branch (or as a dangling commit), record its revision, path, digest and
mode `100644`; `git rev-parse`, `git ls-tree` and `git cat-file` all succeed, so
every stated check passes. This is the F-3 branch-external vector, closed for the
repair packet in §4.2 and left open for every other reference in §5 — including
`route_authority`, which is the field §9's F-5 disposition relies on.

**Minimum edit.** Require every artifact reference to resolve **on candidate
ancestry** (or at one of the exact commits in §4.3's chain) and require the tool
to verify reachability rather than object existence.

### C-4 (High) — §7 leaves `SCOPE_BOUNDARY` blocking underivable, and every "committed manager authority artifact" is authorable by the reviewing lane

§7(3) requires that whether an item blocks integration be "derived by the
validator from state, closure, and exact authority rather than copied from an
unchecked boolean". §7 then gives an explicit blocking rule for
`DEFECT_REPRODUCED`, `DEFECT_SOURCE_PROVEN`, `DEFECT_INJECTION_ONLY`,
`CLOSURE_UNPROVEN`, `ORACLE_VACUOUS`, `CLAIM_FALSE` and
`REQUIREMENT_UNSATISFIABLE` — and **none for `SCOPE_BOUNDARY`**, only a routing
precondition ("may be routed only through an exact committed manager authority
artifact with owner and release condition"). The validator therefore has no rule
from which to derive blocking for the one state the audit's §12.2 marks
non-blocking. As written, §7 is unimplementable at exactly the point where
integration authority is decided.

Worse, the authority artifact is self-authorable. §4.3 requires the evidence
commit to "change exactly the evidence paths declared by the record", and the
record is written by the reviewing lane after that commit. Git records no
verifiable manager identity (no signature requirement anywhere in the contract,
and author/committer fields are trivially set), so an artifact with `owner:
manager:portfolio` and a prose release condition, committed by the reviewer in
their own evidence commit, satisfies every stated check.

**Counterexample.** An `accept` record in which every prior obligation and every
finding carries `evidence_state: SCOPE_BOUNDARY`, each pointing at
`…/swarm-results/<this-lane>/authority.md` — a file the reviewer wrote and
committed in the evidence commit, with an owner line and a release condition.
Chronology, modes, digests, path coverage and set equality all hold; blocking is
underivable, so it is not derived; `integration_decision: ALLOW`. The same shape
closes `REQUIREMENT_UNSATISFIABLE`, which §7 says "remains blocking until that
exact decision is committed" — the reviewer commits it.

**Minimum edit.** (i) State `SCOPE_BOUNDARY`'s blocking rule explicitly. (ii)
Require any authority artifact that changes a blocking derivation to exist **at
or before the candidate revision** on ancestry — a manager decision that predates
the review cannot be minted by the review — and state that Git cannot attest
authorship, so an artifact created inside the review chain is not authority. The
cheapest fully-consistent option, and the one that matches §3's "`ALLOW` is the
sole machine authorization": keep every state blocking at the gate, record the
routing, and let the manager's own decision live outside the machine
authorization entirely.

### C-5 (High) — §8.2's `no_surface_change` disposition is unconstrained, so §1(4) is satisfiable with zero reviewed surfaces

§8.2 gives each changed path exactly one disposition: `reviewed` with at least one
linked surface row, or `no_surface_change` "with a non-empty justification and
exact evidence". §8.3 forbids any minimum, target, or proxy target for surfaces.
Nothing requires *any* path to be `reviewed`, and nothing constrains what the
`no_surface_change` evidence must show.

**Counterexample.** An 11-path candidate whose real repair is in one tool file.
Every path is declared `no_surface_change`, justification "no reviewable
behavioural surface", evidence pointing at a reviewer-authored note committed in
the evidence commit. Changed-path set equality holds exactly (the strongest part
of the design). There are zero surface rows, so §6's I1 derivation obligation
("for each reviewed repair surface") has an empty domain and is discharged
vacuously. The record is `accept`, and F-7 — the finding whose reproducer was one
surface row covering a whole candidate — is bypassed more completely than in v1,
where `changed_surfaces` at least had `minItems: 1`.

Note also that "non-empty justification" is the exact pattern §5 outlaws two
sections earlier ("must be structured Git artifact references, not non-empty
strings").

**Minimum edit, count-free.** Make `no_surface_change` admissible only where the
tool can corroborate it independently — for example, the path's per-commit diff
over the reviewed interval contains no added or removed non-blank content outside
comment/text-only regions, or the path lies in a namespace the Git-bound
pre-repair repair packet declared non-implementation. Where the tool cannot
corroborate, `reviewed` is required. This adds no minimum count and no target; it
removes an author-elective escape.

Second half of F-7, unaddressed: §8.2 says "A self-selected kind may not bypass
that disposition; add cross-checkable signals where practical and state the
human-judgement boundary where not." That is an instruction without a mechanism —
the review's F-7 reproducer (cases 49 vs 50: relabel `public_api` as
`private_symbol`, identical records, `ALLOW` vs `BLOCK`) survives it, and §9's
F-7 row does not mention kind at all. The cheap fix is to stop making the
disposition depend on a self-declared kind: require the exact scope disposition
for **every** surface row, so the label carries no authority and there is nothing
to gain by relabelling.

### C-6 (High) — §8.1 has no set anchor for prior obligations, so recurrence remains author-elective

§8.2 gets its strength from set equality against Git. §8.1 has no equivalent:
the prior-obligation set is enumerated by the record author, and nothing detects
omission. `recurred_after_proven_closure` is required to "bind exact prior
closure evidence and block while open" — but only when the author selects that
value. Recurrence is never *derived* from the predecessor generation, because
(per C-1) no predecessor authority artifact is bound.

**Counterexample.** Obligation `O` was proven closed in generation `N-1` and
recurs in `N`. The author omits `O` from the record entirely, or records it with a
non-recurrence continuity value and no prior-closure reference. Nothing fires.
Combined with C-4: record `O` as `SCOPE_BOUNDARY` against a self-committed
authority artifact and the recurrence rides an `accept`. §8.3's claim that
"acceptance is derived from exact set coverage and terminal dispositions" is true
for changed paths and false for obligations — for a gate whose subject is
*recurrence*, that is the wrong half to leave unanchored.

**Minimum edit.** Derive the prior-obligation set from the predecessor
generation's bound authority artifact (the same anchor C-1 requires) and require
set equality against it, with an explicit `unavailable` state retained in the
denominator for the bootstrap generation where no machine-readable predecessor
record exists.

### C-7 (Medium-High) — §4.3's chain has no place for the record itself, contradicting the mode-`100644` authority rule; the evidence namespace is unenumerated

§4.3 pins three commits by direct-parent rules (review control → report →
evidence) and then says the manager record "is evaluated after the evidence
commit. It need not be present in an earlier commit to authorize its own
creation." That correctly kills the self-authorization paradox, but leaves the
record itself unplaced, while the same section requires that "any other
authority-bearing artifact must be mode `100644`". The record is the most
authority-bearing artifact in the system — it carries every input to
`integration_decision` — and at evaluation time it is a mutable worktree file
with no mode, no digest and no commit. §5 meanwhile makes "mutable worktree-only
file" a fail-closed condition for evidence. The contract owes one sentence
separating *evaluation-time input* from *retained authority artifact*.

Related, and unfixed from F-9: the contract never enumerates where evidence
paths or the record may live. `docs/development/ai/review-recurrence/README.md`
places final records in `docs/development/ai/review-recurrence/`, which is absent
from `CONTROL_PATH_PREFIXES`; §4.3's "evidence paths declared by the record" are
unconstrained as to namespace. Since §4.2(5) (C-2) and §4.3 both turn on path
membership, the namespaces must be named in the contract.

**Minimum edit.** State that the record is evaluated as an uncommitted `--record`
input, that the tool emits the record's own digest in its result, and that the
record plus result are committed as a child of the evidence commit before the
manager cites them; exclude the record from the evaluation-time `100644`
obligation and enumerate the evidence and record namespaces.

### C-8 (Medium-High) — §7's `DEFECT_INJECTION_ONLY` closing condition is inverted, and unverifiable in its strong reading

§7: "`DEFECT_INJECTION_ONLY` blocks unless exact, independently reviewable
**reachability** evidence closes it". Reachability evidence *strengthens* the
finding. The audit's §12.2 states the intended condition — blocking "yes, unless
the reviewer states reachability is impossible" — i.e. closure requires a
**non**-reachability boundary. As written the rule is semantically backwards, and
under its strong reading ("independently reviewable" proof of non-reachability) it
demands a negative-existence proof that neither Git nor the gate can decide; the
pilot scored the corresponding `E8` as only *partly* mechanically decidable ("no
production caller" is a call-graph query; "unreachable by construction" is
judgement).

**Minimum edit.** Rewrite as: blocks unless an exact, independently reviewable
non-reachability boundary is committed and bound; the tool verifies the artifact
binding only, and must not represent the semantic claim as machine-proved — the
same boundary sentence §5 already sets for timed evidence.

### C-9 (Medium-High) — I5 fidelity: §7's authority rules negate five of the six generations the pilot measured, so the metric §7 mandates cannot mean what the pilot measured

The pilot's entire measured benefit for I5 is generation removal: "6 of 18
successor generations … were driven only by states the extended vocabulary would
route rather than block" (pilot §6.5, §9). Those six decompose as `DB-g2`
(`CLAIM-FALSE` only), `RH-g2` and `RH-g3` (`ORACLE-VACUOUS` + `CLAIM-FALSE` +
`SCOPE-BOUNDARY`), `FD-g2` (`ORACLE-VACUOUS` ×2 + `SCOPE-BOUNDARY` ×6), `FD-g3`
(`SCOPE-BOUNDARY` ×3) and the source-keyed Cargo gate (`CLAIM-FALSE` ×5 +
`ORACLE-VACUOUS`).

§7 keeps `ORACLE_VACUOUS` and `CLAIM_FALSE` blocking until closed. Applied to the
pilot's own rows, five of the six generations still block; only `FD-g3` is
composed purely of `SCOPE_BOUNDARY`, and §7 admits that only through an authority
artifact (C-4). §7 then requires: "The result must emit counts by evidence state
and next action so the Cargo and KGI live experiments can measure whether routing
changes generations."

This is not an argument for weakening the rules — §7's last bullet ("must not
reduce integration evidence") and §3's sole-authorization rule are right, and the
conservative choice is the correct one. It is an argument that the **measurement
has silently changed**: the contract redefines the benefit as fewer
*product-repair* generations while the pilot measured fewer *candidate*
generations, and a live experiment reporting a numerator near zero against the
pilot's `6 / 18` will be misread as I5 failing to replicate.

**Minimum edit.** State in §7 that the emitted metric is product-repair
generations avoided (with its denominator and its `unavailable` count), that it is
**not** comparable to the pilot's `6 / 18`, and that the pilot's generation-removal
benefit is deliberately not sought at the gate.

### C-10 (Medium) — I1 fidelity: the obligation is widened beyond the measured basis, the pilot's Rust caveat is dropped, and the emitted metric measures row presence

Three separable points.

1. **Faithful where it matters.** §2's table and §6 bind *derivation* from the
   predecessor tree and state that a byte differential is insufficient. That is
   exactly the pilot's correction of the audit's framing (pilot §9: capture comes
   from derivation, 2 of 2 executed derivation cases versus 1 of 5 differential),
   and §11(15)'s byte-identical counterexample is the right test. Credit where
   due.
2. **Widened.** The pilot measured burden as "1 reviewer obligation **per parity
   claim**", with 0 new author artifacts (pilot §4.5, §7.2). §6 binds a full
   six-field derivation for **each reviewed repair surface whose locus existed in
   the predecessor**. Combined with C-5's per-path surface requirement, the
   obligation now scales with the candidate's changed-path count rather than with
   the record's parity claims. That is a defensible manager choice and an
   undefensible silent one; the pilot explicitly refused to derive obligation
   depth (prerequisite 4 "not met").
3. **Rust caveat dropped, while `unavailable` is fatal.** §6 makes an `accept`
   fail when "the derivation evidence is unavailable". The pilot's I1 decision
   carries the caveat "the Rust arm is blocked until a predecessor build is
   available", and it scored 5 Rust rows `[U]` for exactly that reason. §6 thus
   converts a measured `[U]` into a permanent `BLOCK` for every Rust candidate.
   That may be the right fail-closed choice — the two named live subjects are
   Python-side tooling — but it is a scope consequence the manager should decide,
   and it appears nowhere in the contract.
4. **Metric.** §6 requires emitting "I1 coverage and unavailable counts derived
   from rows", and correctly refuses to claim prose semantics are machine-proved.
   The consequence is that the emitted coverage measures *row presence*, not
   derivation quality. Say so where the metric is defined, or the Cargo/KGI
   comparison will read 100% row presence as 100% capture.

**Minimum edit.** Bind the derivation obligation to the record's parity/closure
claims (with per-surface coverage emitted as output), state the Rust consequence
of fatal `unavailable`, and label the emitted metric as row coverage.

### C-11 (Medium) — §1(2) and §9's F-2 row overclaim: ancestry is graph position, not chronology

§4.2 correctly refuses timestamp-only and branch-external assertions, which
leaves topological ancestry as the sole chronology evidence. Ancestry is
rewritable: `rebase`, `commit --amend` and `filter-branch` can place a packet
commit before repair commits that were authored after it, and the result is
indistinguishable at the gate. §1(2) nevertheless claims v2 "proves that the
repair packet existed before the first repair change", and §9's F-2 row says
"Verify packet precedes first repair change".

§5 already models the honest form of this sentence. §4.2 needs the same one:
what is proved is that in the history presented, the packet commit is an ancestor
of the first repair change and no repair byte accompanies it; history rewriting
is outside the gate's evidence.

### C-12 (Medium) — §2 fails the contract's own evidence standard for two of its three governing artifacts, and overstates the review's `ALLOW` tally

§2 binds the pilot result by SHA-256 (verified above, matches) but gives the
independent review and the root-cause audit by path only, with no digest and no
commit revision — in a contract whose thesis is that authority-bearing artifacts
must be bound by exact revision, path, digest and mode. Both digests are supplied
in §1 of this report, so the edit is mechanical.

Separately, §2 describes "51 executable counterexamples, comprising 39 expected
`BLOCK` results and 12 expected false `ALLOW` results". The review's own wording
is "39 `BLOCK` as expected, 12 `ALLOW` as expected; `unexpected=0`" — "as
expected" means matching the probe's prediction, not that all twelve are
bypasses; a probe set of this shape needs at least one legitimately-`ALLOW`
control, and the review attributes named `ALLOW` bypasses to nine identified
cases (40, 41, 42, 43, 44, 45, 46, 48, 49). Since §11 instructs the implementer
to "turn the independent report's bypasses into failing tests", the inflated
gloss invites a hunt for twelve bypasses that the report does not enumerate.
Quote the report's own tally.

### C-13 (Medium) — §10's surface list omits the only place a schema identifier is registered, so §3 forces either an unregistered `$id` or a stop-and-return

§3 requires "a new schema identifier and file". Schema identifiers are registered
in `check_ai_contracts.py`'s `SCHEMAS` map (lines 28-34), which validates each
file's `$id` against a fixed URL. That file is **not** in §10's expected
implementation surfaces, and §10 ends with "If a shared helper change is
genuinely necessary, stop and return the exact reason". Verified consequences: the
map is an allowlist and does not scan the directory, so §11's
`check_ai_contracts.py` stays green either way — but the v2 schema's `$id` is then
validated by nothing and free to drift, which is the class of gap v1 was
criticised for. `check_ai_acceleration_docs.py`'s `REQUIRED_AI_FILES` (lines
17-49) is existence-only and needs no edit, and `check_ai_development.py` simply
re-runs `test_ai_tools.py`, so those two are clear.

**Minimum edit.** Either add `docs/development/tools/check_ai_contracts.py` to
§10's expected surfaces for the single `SCHEMAS` entry (and say that is the only
permitted change to it), or state that the v2 `$id` is deliberately unregistered
and why.

### C-14 (Low-Medium) — F-10's disposition names the outcome but not its position, and the position is the whole fix

§9's F-10 row requires "Structured fail-closed finding, not a codec traceback",
and §11(10) requires the counterexample. Neither says **where** the check runs.
The failure is raised inside schema validation, by
`ai_contracts.canonical_json_bytes` (`ensure_ascii=False`, line 38) reached
through `uniqueItems` fingerprinting (line 176), before any Git binding — so a
check placed anywhere after schema validation leaves the exit-2 codec error
intact. The gate's own result writer uses `json.dumps(..., indent=2,
sort_keys=True)` with default `ensure_ascii=True`, so a surrogate-escaped path
*can* be rendered in the emitted finding without re-raising; that is worth
stating, because it is the difference between a fixable finding and a second
traceback. Neither point requires a shared-helper change (the review's smallest
boundary is inside the new gate), so this is cheap to pre-decide and should not be
left to discovery.

### C-15 (Low) — five instructions that are not machine-testable as written

Each needs one sentence, not a redesign:

1. §4.3: "Direct-parent rules are **preferred** to net-diff assertions." A
   preference is not a rule. §9's F-12 row states the disjunction correctly
   ("Direct child **or** per-commit write-set verification"); make §4.3 match, and
   require per-commit inspection for every interval that is not a single parent
   edge.
2. §4.3: "any other authority-bearing artifact must be mode `100644`" — a
   semantic category, not a path or field set. Enumerate the artifact roles the
   rule covers, or bind it to the artifact-reference definition §5 introduces
   (which already carries `mode`), so membership is structural.
3. §7(2): "`next_action` **or** `routing_state`" offers two names for one
   machine-checked field. Pick one.
4. §7: "an exact closed/no-defect state **where needed** for prior obligations"
   leaves an enum member unnamed while §7(3) requires blocking to be derived from
   the enum. Name it.
5. §11(19): "no finding-count target or proxy appears in schema, tool, or tests" —
   "proxy" is semantic and cannot be asserted by a test. v1's honest form was
   `const: "none"` plus `additionalProperties: false`; require that shape plus a
   named prohibited-field list, and state the human-judgement boundary.

Also §8.2's "add cross-checkable signals where practical and state the
human-judgement boundary where not" (covered under C-5) belongs in this class.

### C-16 (Low) — the validation set narrows against the predecessor lane, and the v1 "before" half becomes unreproducible at HEAD

Two small consistency points. (a) §11 omits
`python3 docs/development/tools/check_ai_development.py`, which the independent
review ran and recorded at exit 0 (`PASS source_docs / runtime / tool_tests /
validation_capabilities`); dropping it reduces exit-code comparability between
generations at no saving, since it mostly re-runs a suite §11 already runs. (b)
§3 requires that v2 "never retain the rejected v1 `ALLOW` path", while §11 asks
the implementer to "demonstrate old-v1 false behaviour or an equivalent fixture".
Once the v1 path is deleted, the "before" half is unreproducible at HEAD; say
that the v1 behaviour is demonstrated at the rejected candidate revision
`1cae0225` (which resolves in this object database) or by an explicitly synthetic
fixture, so the demonstration does not quietly become v2-only.

## 4. Clause-by-clause F-1 … F-14 coverage

`Testable` = the mechanism as written can be decided by the tool from Git and the
record. `Verdict`: `CLOSED` = no residual bypass found; `PARTIAL` = mechanism
present, bypass or unimplementability remains; `GAP` = the bypass survives.

| Finding | v2 clause(s) | Mechanism | Testable | Residual bypass / defect | Verdict |
|---|---|---|---|---|---|
| F-1 base narrowing | §4.1(1)–(4); §9 row | replace free base with `predecessor_candidate_revision`; ancestry; single interval | partly | interval floor still author-chosen; §11(1) unproducible (C-1) | **GAP** |
| F-2 post-repair packet | §4.2(1)–(7) | packet blob mode/digest/ancestry + pre-repair write-set window | no, as written | "manager-control paths" undefined; control-plane taxonomy admits repair-before-packet; blocks own first candidate (C-2); ancestry ≠ chronology (C-11) | **PARTIAL** |
| F-3 fake dispatch event | §4.2 preamble | `dispatch_event_attested` removed; branch-external/timestamp-only ⇒ `BLOCK` | yes | none for the packet path; the same branch-external vector persists for §5 references (C-3b) | **CLOSED** (in scope) |
| F-4 fake evidence/log digests | §5 | committed `codefire.timed_gate.v1` result + log blobs, digests, byte count, status agreement, fail-closed list | yes for bytes | no source identity (C-3a); references need not be reachable (C-3b); required-validation *set* unanchored | **PARTIAL** |
| F-5 dead severity/blocking | §7(1)–(3) + authority rules | separate `evidence_state` / next action; blocking derived | 7 of 8 states | `SCOPE_BOUNDARY` blocking underivable; authority self-authorable; `REQUIREMENT_UNSATISFIABLE` self-closable (C-4); injection rule inverted (C-8) | **PARTIAL** |
| F-6 dead continuity | §8.1 | `recurred_after_proven_closure` binds prior closure evidence and blocks while open | yes, when set | obligation set has no anchor; recurrence not derived from predecessor (C-6) | **PARTIAL** |
| F-7 one surface covers many paths | §8.2 | per-path disposition; surface belongs to exactly one path | yes for linkage | `no_surface_change` unconstrained ⇒ zero reviewed surfaces admissible; self-declared `kind` relabelling unaddressed (C-5) | **PARTIAL** |
| F-8 control rewrites gate | §4.3 | review control = direct child of candidate, changes exactly the review packet path; explicit no-rewrite rule | yes | none found | **CLOSED** |
| F-9 impossible/undocumented ordering | §4.3 | explicit chain; record evaluated after evidence, need not pre-exist | yes for the three commits | record's own location/mode contradiction; evidence/record namespaces unenumerated (C-7); chain collides with §4.2(5) (C-2 Arm A) | **PARTIAL** |
| F-10 invalid UTF-8 Git path | §9 row; §11(10) | structured fail-closed finding | yes if placed correctly | position unspecified; must precede schema validation, and the path's rendering in the result is undecided (C-14) | **PARTIAL** |
| F-11 symlink report/packets | §4.3 mode list; §5 symlink fail-closed | require mode `100644` | yes | artifact set is a semantic category, not an enumeration (C-15.2) | **CLOSED** |
| F-12 mutate/revert control chain | §4.3 final bullet; §4.2(5); §9 row | direct-parent rules or per-commit write sets | yes | "preferred" is not a rule (C-15.1) | **CLOSED** pending wording |
| F-13 contradictory unavailable/out-of-scope | §8.1; §8.2 | `unavailable` constrains observations and cannot ride `accept`; `out_of_scope` removed or disjoint | yes | none found | **CLOSED** |
| F-14 self-referential patch fixture | §9 row; §11(14) | pin a byte-level patch digest fixture | yes | none found; note the pin must be a fixed fixture repository, not this candidate's own digest | **CLOSED** |

Tally: 5 `CLOSED`, 1 `CLOSED` pending wording, 7 `PARTIAL`, 1 `GAP`. The
contract is therefore **not** sufficient to prevent all F-1 … F-14 bypasses as
written, which by §9's own standard ("The implementation and its tests must
explicitly disposition every finding") is a pre-implementation defect rather than
an implementation risk.

## 5. I1 / I5 fidelity to the historical replay

| Replay decision | Contract instantiation | Fidelity |
|---|---|---|
| I1 `PROMISING_FOR_LIVE_EXPERIMENT`; capture is **derivation**, not differential | §2 table ("A byte differential is insufficient"); §6 binds predecessor-tree derivation; §11(15) byte-identical counterexample | **faithful on mechanism**; widened from per-parity-claim to per-surface; Rust `[U]`→`BLOCK` consequence unstated; emitted metric is row presence (C-10) |
| I1 caveat: 3 of 21 reports already discharge it, control group 18 lanes | absent | omitted; harmless to the gate, relevant to the live comparison |
| I1 caveat: Rust arm blocked until a predecessor build exists | absent, while `unavailable` derivation is a hard `accept` failure | **unfaithful by omission** (C-10.3) |
| I5 `PROMISING_FOR_LIVE_EXPERIMENT`; record the two states independently without changing integration authority | §7(1)–(3); §3's sole-authorization rule; §7's "must not reduce integration evidence" | **faithful** |
| I5 condition 1: the §12.2 vocabulary is incomplete — a state is missing for *source-proven by a lane that executed nothing* | §7 adds `DEFECT_SOURCE_PROVEN`, with the reason stated | **faithful**, and correctly attributed |
| I5 condition 2: `CLOSURE_UNPROVEN` never accepted; `ACCEPT_WITH_NOTES` not invented | §1(7), §3, §7 ("Do not add `ACCEPT_WITH_NOTES`"), §11(17) | **faithful** |
| I5 measured benefit: 6 of 18 generations routable rather than blocking | §7 keeps `ORACLE_VACUOUS`/`CLAIM_FALSE`/`DEFECT_INJECTION_ONLY` blocking ⇒ 5 of the 6 still block; §7 still asks the live experiment to "measure whether routing changes generations" | **metric infidelity** (C-9): conservative on authority, incomparable on measurement |
| I3 `REVISE_AND_REPLAY`: no line/file/surface/size limit; delta as output only | §2 table; §8.3; §13(6) measurements only | **faithful**; no threshold appears anywhere in the contract |
| I4 `REVISE_AND_REPLAY`: do not install E1–E9 as a mechanical checklist | §2 table; §5 binds concrete evidence without claiming completeness | **faithful**. Consequence to record: the pilot's measured hole (4 of 175 oracle rows whose fixture never enters the state the invariant concerns) stays open by design, and `E1` — the one rule that is mechanically decidable, zero-noise, and the reason a generation-3 review never launched — is *not* covered by §5 today (C-3a) |
| Replay's negative constraint: no review-item count, checklist length, template depth, field count, or delta threshold | §2 closing paragraph; §8.3; §12; §13(6) | **faithful**. I found no count, quota, or proxy target in the contract, and this report invents none |

## 6. Operator burden and proportionality

The contract does the two things the independent review's §6 asked for: it deletes
the transcribed `outcomes` block and the unchecked stopping booleans (§8.3), and
it moves unverified `closure_contract`/`oracle` prose out of the record unless the
tool consumes it (§8.1). Both are real reductions against the review's measured
v1 cost (76 required leaves minimal, ≈320 for the rejected candidate's own 11-path
shape).

Against that, v2 adds obligations whose growth is *multiplicative in references*
rather than additive in fields: every evidence and authority reference becomes a
structured artifact reference with at least revision, path, digest and mode (§5)
where v1 accepted one string; every changed path acquires a disposition and at
least one surface row (§8.2, and C-5's fix makes that binding); every reviewed
surface with a pre-existing locus acquires a six-part derivation with its own
references (§6); every required validation acquires a result path, a log path,
two digests, a byte count, argv, exit code, status and terminal state (§5). The
direction is therefore not obviously downward for an 11-path candidate, and
§13(6) only measures it **after** the bytes are written.

I deliberately name no target and no acceptable leaf count. The proportionate
edit is a measurement obligation, not a threshold: require §12's
pre-implementation design table to carry the minimal and realistic required-leaf
estimate **and** the v1 comparison (76 / ≈320, from the independent review §6),
so the manager can stop an over-budget shape before implementation instead of
reading it in the return contract. That is also the cheapest defence against the
review's central proportionality finding — "the structure is not the problem; the
unverified surface attached to it is" — because it forces the ratio of verified to
unverified leaves to be stated up front.

One burden asymmetry worth flagging: §4.3's rigid three-commit chain plus the
record commit means four commits per generation with pinned write sets, and any
mistake in ordering is unrecoverable without rewriting history (which C-11 shows
the gate cannot see). Combined with C-2 Arm A, an operator who commits the report
and evidence in the natural order can find the *next* generation blocked by the
previous generation's own compliance. That interaction is the single most likely
cause of a wasted live-experiment cycle and should be resolved in text, not
discovered in a lane.

## 7. Minimum contract edits required before implementation

Ordered by dependency. Each is an edit to the contract, not to an
implementation.

1. **§4.1 + §11(1) — bind predecessor identity** to a committed prior-generation
   authority artifact (revision, path, SHA-256, mode `100644`), require equality
   with `predecessor_candidate_revision`, and fix the first-generation bootstrap
   by naming the exact predecessor artifact in the contract. Restate §11(1) so the
   counterexample is a narrowed *interval*, not a stray field. (C-1)
2. **§4.2 — enumerate three disjoint path sets**: repair-packet path;
   manager-control paths admissible in the pre-repair window (including the
   report/evidence/record namespaces, so the steady state is satisfiable);
   repair surface computed from the candidate's changed paths. State that
   `CONTROL_PATH_PREFIXES` is not reusable, and that the admissible set must
   exclude every path in the candidate's changed-path set. Replace
   "candidate/product/test change" with the Git-derived repair surface. (C-2)
3. **§7 — state `SCOPE_BOUNDARY`'s blocking rule**, require any
   blocking-changing authority artifact to exist at or before the candidate
   revision on ancestry, and state that Git cannot attest authorship. (C-4)
4. **§8.2 — constrain `no_surface_change`** to dispositions the tool can
   corroborate independently, and require the exact scope disposition for every
   surface row so a self-declared `kind` carries no authority. Count-free. (C-5)
5. **§5 — require provenance**: each timed-gate reference binds the revision or
   tree it executed against (and the contract states which route closes the
   `codefire.timed_gate.v1` gap); every artifact reference must resolve on
   candidate ancestry with verified reachability. Anchor the required-validation
   set to the Git-bound pre-repair repair packet's declared commands, or state
   that the scope is author-chosen. (C-3)
6. **§8.1 — anchor the prior-obligation set** to the predecessor authority
   artifact with set equality and an explicit `unavailable` bootstrap state. (C-6)
7. **§4.3 — place the record**: evaluation-time input versus retained authority
   artifact, record digest emitted in the result, record+result committed after
   the evidence commit, evidence/record namespaces enumerated. (C-7)
8. **§7 — invert the `DEFECT_INJECTION_ONLY` closing condition** to a committed
   non-reachability boundary, with the machine/judgement boundary stated. (C-8)
9. **§7 — redefine the emitted I5 metric** as product-repair generations avoided,
   with denominator and `unavailable`, and state that it is not comparable to the
   pilot's `6 / 18`. (C-9)
10. **§6 — bind the I1 obligation to parity/closure claims**, state the Rust
    consequence of fatal `unavailable`, and label the emitted coverage as row
    coverage. (C-10)
11. **§1(2) and §9 F-2 — restate the chronology claim** as ancestry-in-presented-
    history, with the rewrite boundary named as §5 names its own. (C-11)
12. **§2 — bind the review and audit by SHA-256** (digests in §1 above) and quote
    the review's `ALLOW` tally verbatim. (C-12)
13. **§10 — resolve the schema-registration scope trap** (`check_ai_contracts.py`
    `SCHEMAS`) explicitly, one way or the other. (C-13)
14. **§9 F-10 / §11(10) — specify the position** (before schema validation) and
    the rendering of the offending path. (C-14)
15. **§4.3, §7, §11(19), §8.2 — replace the five non-testable instructions**
    listed in C-15 with exact rules.
16. **§11 — restore `check_ai_development.py`** and pin the v1 "before"
    demonstration to `1cae0225` or to an explicitly synthetic fixture. (C-16)

Edits 1–5 are the blocking set: each defeats a numbered guarantee in §1.

## 8. Explicitly unavailable evidence

- **No v2 implementation exists**, and the contract forbids one, so every finding
  here is a defect in text reasoned against the artifacts the text binds. I
  executed no counterexample against a v2 tool and claim no executed reproducer.
- **I ran no gate, test, or probe against v1 either.** I read
  `check_review_recurrence.py`, `run_timed_gate.py`, `ai_contracts.py`,
  `check_ai_contracts.py`, `check_ai_acceleration_docs.py` and
  `check_ai_development.py` to decide feasibility of specific clauses; the
  behavioural claims about v1 in this report are the independent review's `[R]`
  results, not mine. The structural claims (`CONTROL_PATH_PREFIXES` membership,
  `SCHEMAS` allowlist semantics, `timed_gate.v1` field set, `ensure_ascii`
  asymmetry, the seven commits in `1cae0225..a79acc0`) are recomputed here.
- **The manager's dispatch disposition is unrecorded** at this base (the packet's
  own Status line makes implementation contingent on it), so I cannot know whether
  a narrow or broad manager-control enumeration is intended. C-2 is stated as a
  dichotomy for that reason.
- **No predecessor-generation machine-readable record exists** for the next v2
  candidate, so C-1's and C-6's recommended anchor cannot be exhibited — only
  specified. Whether a prior generation's report reliably declares its exact
  candidate revision in a machine-readable position is unverified beyond the
  three reports I read.
- **Operator burden for v2 is unmeasured.** No v2 schema exists, so no required-
  leaf count can be computed; v1's 76 / ≈320 are the independent review's
  measurements, not recomputed here. I name no target.
- **Rust predecessor-build availability** (I1's Rust arm) is unavailable, per the
  pilot's own `[U]` rows and threat 5; I could not test whether §6's fatal
  `unavailable` is tolerable for any Rust lane.
- **The Cargo and KGI live-experiment packets do not exist** at this base, so I
  could not verify that §6's and §7's obligations are dischargeable by those two
  lanes, nor that their candidates' path shapes survive C-2's window rule.
- **Provenance of execution is out of reach by construction**, as §5 itself
  states. Nothing in this report claims that a timed-gate result proves honest
  execution; C-3a is about binding the *tree*, which is decidable, not about
  honesty, which is not.
- **I did not review** the v1 schema's bytes clause-by-clause, the rejected
  candidate's tests, or any product candidate, and I changed no packet, schema,
  tool, test, prior report, or recorded verdict.

## 9. Non-authorization

This report is an independent contract-preflight disposition. It does not
authorize implementation, adopt I1 or I5, accept or reject any product candidate,
change any recorded verdict or index, fix any checklist length, field count, or
repair-size threshold, or authorize the Cargo/KGI live comparison. The manager
owns the dispatch decision. The commit carrying this report adds exactly one
path:

```text
docs/development/ai/swarm-results/cfr168-review-recurrence-v2-contract-preflight-claude-20260731/review.md
```

CONTRACT_VERDICT: REQUEST_CHANGES
