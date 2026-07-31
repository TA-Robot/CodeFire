# CFR-168 Recurrence Gate v2 Repair Contract — Final Preflight Adjudication (`41824af`)

Reviewer: Claude Opus 5 (`claude-opus-5`), same independent contract-preflight
lane as `review.md` and `rereview.md`, report-only. Date: 2026-07-31.
Third and final contract adjudication. Subject is the **contract**; no v2
implementation exists at this revision and this lane wrote no schema, tool,
test, product, packet, or prior-report byte.

## 1. Exact manager-frozen subject

| Item | Value | How established |
|---|---|---|
| Canonical commit | `41824afaace1d172ec12f26d98fa0ae306b4ddcd` | `git cat-file -t` → `commit` |
| Subject / time | `docs(ai): root recurrence v2 authority and closure contracts`, 2026-07-31 14:44:32 +0000 | `git log -1` |
| Commit write set | exactly the packet path (`+179 / −43`) | `git show --stat` |
| Packet path | `docs/development/ai/task-packets/cfr168-review-recurrence-gate-v2-repair-20260731.md` | — |
| Packet mode | `100644` | `git ls-tree 41824af -- <path>` |
| Packet blob | `f5314904335f8fed4ed7ec65c6f82b590fef1a42` | `git ls-tree` |
| Packet SHA-256 | `e08e736cbdb7f48fc131ed2b542db951fbff32ebe1b29d399bfe83172bdea6f6` | `git show 41824af:<path> \| sha256sum` — **matches the frozen value exactly** |
| Length | 816 lines (prior revisions: 456 at `a79acc0`, 680 at `d491690`) | `wc -l` |
| Prior contract revision | `d4916906ccc17a56c587c345c3b2818c8eeca9b9`, SHA-256 `aa3faca4…` | `rereview.md` §1 |

The subject was read only through `git show 41824afaace1d172ec12f26d98fa0ae306b4ddcd:<path>`
and Git object commands. Nothing was cherry-picked, rebased, merged, or edited,
and my worktree copy of the packet was not consulted for any adjudication —
`git log --oneline d491690..41824af` shows the two intervening commits
(`e837404`, `41824af`), neither of which is on my branch.

### 1.1 Prior-report integration verified

| Report | Integrated commit | Recomputed SHA-256 | Matches what I committed |
|---|---|---|---|
| `review.md` | `102fc1232b9a5140c72be94b8add51b5a58c5634` | `ccd5585c79acb19e7c412b336e40930dd984ea4e26217486f659a1132f6bec0b` | yes |
| `rereview.md` | `e8374040ed177f915b3c3debed20cd4ffa21b17a` | `af5dab388cd2926da22ae77914a26605e067d5e884dae350f263123ad16e3dfb` | yes |

Both are single-path report-only commits, and the packet's **Evidence that
governs this repair** now binds both by commit, mode and digest with their
`REQUEST_CHANGES` verdicts. My prior reports were read in full from this result
directory before adjudicating.

### 1.2 Newly frozen identities, independently recomputed

Every identity this revision adds or retains was recomputed. All pass.

| Binding | Contract requirement | Result |
|---|---|---|
| Cargo port packet | `…/task-packets/cfr168-source-keyed-cargo-third-closure-repair-20260731.md` at `8c51b712f1e3937fd462a95c70b8de79687b9289`, SHA-256 `7464f158…`, absent at `4e5c0e7f` | **verified**: mode `100644`, digest matches, absent at the predecessor tree; source commit subject `docs(cfr168): bind Cargo third closure repair` |
| KGI/KPI port packet | `…/task-packets/cfr168-development-flow-kgi-kpi-r4-repair-20260731.md` at `a22fad0698b1623d8898c47699e7ba4e61294b5f`, SHA-256 `bbd6ef47…`, absent at `753c8db1` | **verified** on all four counts; source commit subject `docs(cfr168): bind KGI round-four repair`. The source commit is also the rejected v1 candidate's base — checked, not a paste error: that commit genuinely introduces this packet |
| Both bootstrap authority roots | sole parent = predecessor, write set = exactly the report path, mode `100644`, declared digest | **re-verified** (all five conditions per arm, as in `rereview.md` §1.1) |
| Both frozen inventory packets | `0304c0e1` / `8437a8e0` with declared digests | **re-verified**, mode `100644` |
| Four governing evidence artifacts | commit + mode + digest | **re-verified** |
| Packet-section document gate | `TASK_PACKET_SECTIONS` = `Status:`, `## Objective`, `## Scope`, `## Non-Goals`, `## Validation`, `## Handoff` | **all six present in the frozen blob**; the gate's test is a plain substring check (`check_ai_acceleration_docs.py:400-402`), so dropping the section numbers is compatible |

Across three rounds I have now checked sixteen manager-frozen bindings in this
contract and found no error in any of them.

## 2. Verdict summary

**R-1 through R-10 are all CLOSED.** Each has an exact clause, a named machine
reader where one was demanded, and a matching counterexample in **Validation**
(items 24–30 are new and map one-to-one onto R-1, R-2, R-3, R-5, R-6, R-10 and
R-8/R-9). Two closures exceed what I asked for: the timed-runner clause solves a
problem I had not raised (log/result writes would themselves dirty the checkout,
so they are written outside the worktree and copied in afterwards), and the
`CLOSED_PROVEN` clause adds a rule I did not propose but which closes the
obvious next hole ("A new finding with no pre-repair manifest mapping cannot be
closed in the same generation").

I found six new items. None is a bypass of a stated mechanism. The most serious,
N-1, is an *unspecified* role rule that the contract's own mandatory
pre-implementation design table forces an implementer to surface before writing
any implementation byte; the rest are one-sentence specification or sequencing
gaps, and N-3 concerns the canonical ports, which by construction happen after
v2 is implemented and accepted.

That is a materially different situation from the two prior rounds, which
contained live bypasses (author-chosen predecessor identity; unconstrained
`no_surface_change`; a nonblocking state with no evidence obligation). This
revision contains none that I can construct. On the standard set for this
adjudication — whether this exact packet is coherent enough to dispatch a
separate implementation agent — the answer is yes, and I record the six items as
mandatory design-table resolutions rather than as contract blockers, because the
contract already requires each authority-bearing artifact to be mapped to "its
chronology, mode, digest, and reader" and requires a stop-and-return instead of
a silent choice.

## 3. R-1 … R-10 disposition

| Finding | Exact clause in the frozen blob | Adversarial result | Disposition |
|---|---|---|---|
| R-1 unrooted predecessor chain | **Predecessor authority and candidate interval**, ¶4: "A prior v2 record is not trusted as a standalone JSON blob. The named machine reader is the `resolve_predecessor_authority_chain` path… walks the record's `predecessor_authority` chain transitively until one of the manager-fixed legacy roots… verifies generation continuity, candidate identity, packet/candidate/control/report/evidence/retention topology, exact per-commit write sets, modes, blob digests, record/result digest equality, and the exported successor-obligation set… Cycles, missing links, a discontinuous generation, or a chain that does not terminate at a manager-fixed root produce `BLOCK`. Ref existence is never a substitute for this walk." + Validation(24) | The synthesis trace in `rereview.md` R-1 no longer runs: a fabricated record must now terminate at a pinned legacy root, and every link's topology, write set, mode, digest, record/result equality and exported obligation set is re-derived. A rooted forgery would have to perform an entire real generation (real packet at a pinned digest, real repair series, real clean-checkout timed results); what remains is manifest-content discretion, which is N-3, not chain forgery | **CLOSED** |
| R-2 vacuous `CLOSED_PROVEN` | **I5 live experiment**: "`CLOSED_PROVEN` is not a free label. A prior obligation may use it only when it binds every closure-validation ID frozen for that obligation in the manifest, that mapping is non-empty, and every corresponding committed timed v2 result passes… A new finding with no pre-repair manifest mapping cannot be closed in the same generation." + manifest rule "Except for an obligation whose predecessor state is `DEFECT_INJECTION_ONLY`, every prior-obligation row has at least one closure-validation ID" + "Every referenced closure-validation ID must exist in the required validation set" + Validation(25) | The "label everything closed" trace is dead: an empty mapping is rejected, mapped results must exist and pass, closure validations must be manifest-frozen pre-repair, and same-generation closure of new findings is barred. The one remaining alternative is the injection-only non-reachability artifact, whose permitted revision is unstated — N-1 — and whose exemption predicate has no named source — N-2 | **CLOSED** |
| R-3 arbitrary-ref reachability | **Evidence-bound validations**, final ¶: "There is no global 'reachable from any ref' predicate. Contract-pinned bootstrap artifacts are verified by their exact commit/write set/mode/digest; packet, candidate, review, report, and evidence roles are verified by their declared topology; prior records use the rooted chain walk; routing/scope roles use candidate ancestry. A local branch or ref pointing at an object is never evidence, and a role-incompatible or unrooted commit fails even when `git cat-file` can read it." + Validation(26) | Both readings I showed to fail are gone: the strict-ancestry reading no longer applies to the pinned bootstrap reports (they are verified by pinned commit + write set + mode + digest), and the any-ref reading is explicitly refused. Verified consistent with the fact that all six bootstrap-arm objects are reachable only from lane branches | **CLOSED**; two roles still unnamed (N-1) |
| R-4 unstated sibling topology | **Repair packet and manifest precede the presented repair**, ¶2: "Steady state is an explicit fork. The predecessor candidate has one child on its completed review chain and a sibling packet commit for the successor repair. The manifest may therefore embed obligation data emitted later on the sibling review chain; the prior record is verified through the rooted authority-chain walk…, not by pretending it is a candidate ancestor. The topology proves graph relationships only, not when the manifest was authored." | Realizable: two children of one commit with disjoint write sets (exactly one review-packet path vs exactly the two packet paths), so role resolution is unambiguous. The shape already exists in this repository at `4e5c0e7f` (report child `130878d3`, repair child `74cd37d5`). The content-ordering paradox I raised is resolved explicitly rather than denied | **CLOSED** |
| R-5 dirty-checkout provenance | **Evidence-bound validations**, ¶3: "The runner captures `git status --porcelain=v1 -z --untracked-files=all` immediately before and after the command, emits both cleanliness states, and v2 acceptance requires both empty. Timed logs/results are first written outside the source worktree and later copied byte-identically into the evidence commit… A command that mutates and restores bytes during its own execution remains outside the provenance proof and is stated as such." + the boundary ¶ now disclaims "absence of mutate/revert activity inside the command" + Validation(27) | Closed and self-consistent, including the log-writes-would-dirty-the-tree problem. Residual limit is stated, not hidden. One narrow index-level evasion remains — N-5 | **CLOSED** |
| R-6 bootstrap `ALLOW` contradiction | **Predecessor authority and candidate interval**: "A bootstrap generation always records `predecessor_obligation_completeness: unavailable` and therefore cannot produce machine `ALLOW`; its retained v2 record seeds the second generation, where `ALLOW` first becomes reachable. No other legacy binding may serve as a root at all without a new manager contract and independent preflight." + Validation(28) | Wording contradiction removed and the two-generation cost is now explicit. Second generation is genuinely reachable: its authority is the bootstrap record, whose own authority is the pinned legacy root, generation continuity holds, and nothing requires a predecessor record to be non-blocking | **CLOSED** |
| R-7 unread retention invariant | Same ¶4 as R-1: "it verifies that the retention commit is the direct child of the evidence commit and changes exactly the declared record and result paths" + record/result digest equality + Validation(24), (9) | The named reader is the successor generation's authority resolution, exactly as recommended. Consequence to note for the manager, not a defect: retention integrity is therefore verified *retroactively*, so a manager citing generation `N` before `N+1` runs relies on the operator obligation | **CLOSED** |
| R-8 ambiguous seed collections | **Prior obligations**: "For the two bootstrap arms, all three manifest collections are manager-derived from the exact frozen packets below: prior-obligation rows from their frozen row inventories; parity/closure claims from their closure-contract and predecessor-failure sections; required validation IDs/argv from their validation and independent-negative-control tables." + Validation(30) | Ambiguity gone; all three collections are named with their source sections | **CLOSED**; anchor location is N-3 |
| R-9 unstated discretion boundary | **Repair packet and manifest precede the presented repair**: "The manifest is the remaining discretion boundary. Its parity/closure claims and required validations are manager-reviewed and content-frozen with the dispatch revision before an implementer starts. Set equality prevents later weakening but cannot prove that the manager chose a complete set; independent packet preflight judges that completeness without a count target." + Status line "freezes its manifest/dispatch revision" + **Prior obligations**: "The gate verifies the frozen manifest bytes and downstream set equality, not the manager's semantic extraction from Markdown." | Stated exactly, with the machine/judgment split explicit and no count target introduced | **CLOSED** |
| R-10 unexecutable KGI port | Port table (both arms, path + source commit + digest + "absent at" predecessor) + "KGI/KPI's historical `swarm-batches/…-r3-20260731.md` and `…-r4-20260731.md` remain byte-for-byte in the ported repair patch and are ordinary reviewed repair surfaces, not precedence packets… The proof is scoped to the one declared Markdown packet and its manifest; every other instruction artifact remains visible through exact changed-path/surface coverage." + Validation(29) | All four identities recomputed and correct, including both "absent at predecessor" claims, so the two-path write set holds for both arms. The in-patch `swarm-batches` artifacts are inside `repair_surface` (= changed paths − the two packet paths) and therefore must each carry a `reviewed` surface row under **Changed paths and surfaces**; coverage is continuous | **CLOSED** |

No PARTIAL and no OPEN. C-1 … C-16 and F-1 … F-14 remain closed: this revision
only adds, both disposition tables are intact, and every mechanism I credited at
`d491690` is present verbatim or strengthened.

## 4. Specifically requested verifications

| Requested check | Result |
|---|---|
| Transitive rooted predecessor authority; named successor-generation machine reader | **Satisfied.** Reader named (`resolve_predecessor_authority_chain`), rooting mandatory, ref existence explicitly disqualified |
| Cycles, generation continuity | **Satisfied.** "Cycles, missing links, a discontinuous generation… produce `BLOCK`" |
| Retention parent / write set / modes / digests; record-result equality; successor obligation sets | **Satisfied**, all five enumerated in the walk; `successor_obligation_ids` set equality also enforced in **Prior obligations** with derived continuity and "The author cannot choose a weaker label" |
| `CLOSED_PROVEN` cannot be vacuous; non-empty manifest mappings; injection-only alternative | **Satisfied** for the mapping path (non-empty, manifest-frozen, all mapped results must pass, no same-generation closure of new findings). The injection-only alternative is where N-1 and N-2 sit |
| Role-specific validity never depends on arbitrary refs | **Satisfied**; two roles (non-reachability evidence, prior-closure evidence) are named in the common definition but omitted from the role→revision list — N-1 |
| Steady-state sibling topology realizable | **Satisfied**; empirically realizable and already exemplified at `4e5c0e7f` |
| Clean-before/after timed-v2 scope and stated mutate/revert limit | **Satisfied**, with the out-of-worktree log/result staging that makes it self-consistent; index-level evasion is N-5 |
| First-generation bootstrap `BLOCK`, second-generation first `ALLOW` | **Satisfied** and now stated in the contract |
| All three manager-frozen manifest collections; manifest discretion boundary | **Satisfied** in text; the bootstrap manifest's own freeze anchor is N-3 |
| Exact Cargo/KGI port packet identities; continued changed-surface coverage of historical in-patch instruction artifacts | **Satisfied**; all four identities recomputed, coverage continuous |
| Fixed task-packet headings vs repository document gate | **Compatible**; all six `TASK_PACKET_SECTIONS` markers present in the frozen blob |
| New contradictions introduced by the revision | Two found: N-2 (exemption predicate with no named source) and N-6 (a Validation item referring to a field this revision removed) |

## 5. New findings, ordered by severity

None creates or requires a finding, checklist, field, or delta threshold, and I
propose none.

### N-1 (Medium-High) — two artifact roles are named in the common definition but omitted from the role→revision list, leaving `CLOSED_PROVEN`'s only alternative resolvable inside the reviewer's own evidence commit

**Review, evidence, record, and result ordering** lists the roles the common
artifact-reference definition covers, including "prior-closure evidence" and
"non-reachability evidence". **Evidence-bound validations** then derives the
permitted revision per role, and its list names packet/manifest, candidate,
review/report/evidence, prior v2 record, the two legacy bootstrap reports, and
routing/scope-decision artifacts — **not** those two.

**Trace.** An obligation whose predecessor state is `DEFECT_INJECTION_ONLY` may
use an empty closure-validation mapping "only when it binds the independently
accepted non-reachability boundary". With no role rule, the most permissive
reading is the adjacent one — "review/report/evidence artifacts resolve at their
exact ordering commit" — under which the boundary artifact is a blob the
reviewing lane writes in its own evidence commit. That is the single path around
R-2's closure-validation requirement, and it would be self-authored. The
contract's own intent points the other way ("independently accepted", "after an
independent reviewer accepts that boundary", Validation(23)'s "later
`CLOSED_PROVEN` row"), and routing/scope artifacts already have the correct shape
of rule ("at or before the candidate on its ancestry and never inside review
control, report, or evidence commits"). The same omission applies to
prior-closure evidence, which **Prior obligations** requires
`recurred_after_proven_closure` to bind.

**Minimum edit.** Add both roles to the role→revision list with the
routing/scope shape: resolvable at or before the candidate on its ancestry, or at
a rooted prior generation's report/record commit, and never inside this review's
own control, report, or evidence commits.

**Why this does not block dispatch.** **Pre-implementation self-review** requires
the implementer to map "every authority-bearing artifact to its chronology, mode,
digest, and reader" before editing implementation bytes, and **Non-Goals**
requires a stop-and-return rather than a silent widening. An implementer
following the contract must surface this gap in the design table; the manager can
then answer it in one sentence.

### N-2 (Medium) — the closure-validation exemption depends on a predecessor state that no named artifact is required to carry

The manifest rule reads: "Except for an obligation whose predecessor state is
`DEFECT_INJECTION_ONLY`, every prior-obligation row has at least one
closure-validation ID", and the I5 rule repeats the predicate. But the manifest
"contains exactly" nine listed items with `additionalProperties: false`, and none
is a predecessor state; the exported `successor_obligation_ids` is described as
IDs only ("every prior obligation plus every new finding from that record"); and
the chain walk's enumerated checks do not include obligation states.

**Trace.** For generation ≥ 2 the state is *available* — the walk loads the
predecessor record — but no clause requires the tool to source the exemption from
there, so an implementer may take the exemption from the manifest author's
assertion, which is precisely the discretion R-2 removed. At bootstrap the state
does not exist in machine-readable form at all (predecessor states are
manager-derived from legacy Markdown), so the predicate is undecidable there —
harmless in effect, since a bootstrap generation cannot `ALLOW`, but the rule as
written has no subject.

**Minimum edit.** Name the source: the exemption is read from the predecessor v2
record's state for that obligation ID via the chain walk, and empty mappings are
inadmissible in a bootstrap manifest.

### N-3 (Medium) — the bootstrap manifest has no machine-checkable freeze anchor, so Validation(30) has no subject and a rooted-but-thin chain remains possible

The contract pins the port packets, the bootstrap authority roots and the frozen
inventory packets by commit and digest. It does not pin the *manifests*, which do
not yet exist. Validation(30) nevertheless requires a test that "all three
bootstrap manifest collections are set-equal to the independently preflighted
manager freeze; thinning any one collection is rejected". Set equality between
record and manifest is testable; equality between a manifest and "the manager
freeze" needs a pinned manifest identity that the gate can read, and none is
named.

**Trace.** This is also the residual of R-1: chain rooting stops a fabricated
chain, but a *rooted* chain whose bootstrap manifest is thin — fewer obligations,
fewer required validations, fewer parity claims — passes every downstream set
equality, because every downstream set is defined by that manifest. The only
barrier is human preflight, which the contract honestly labels as judgment.

**Minimum edit and why it is post-acceptance.** Once each port's manifest exists,
record its path, commit and SHA-256 in this packet (or a manager-owned pin the
gate reads) before the arm is dispatched — the same treatment the port packets
already receive. The ports happen after v2 is implemented and accepted, so this
cannot block dispatching the implementation; it must land before either live arm
runs.

### N-4 (Medium) — the deciding gate's own identity is not bound into the v2 result, so retained decisions and the chain walk are not reproducible across gate revisions

The v2 result carries the record digest, the candidate revision/tree,
clean-before/after states and the timed evidence, and the retained record/result
pair becomes the authority a later generation walks. Nothing requires the result
to record which revision of `check_review_recurrence.py` and which schema blobs
produced it. v2 is explicitly experimental and will be revised during the pilot,
and the chain walk re-verifies historical links with whatever logic is current,
so a rule change silently re-adjudicates retained decisions.

**Minimum edit.** Emit the gate script's revision/blob digest and the v2
schema/manifest schema digests in the result, and have the chain walk record the
gate identity that validated each link. Additive to a schema already in scope; no
new operator field.

### N-5 (Low) — `git status --porcelain` can be defeated by index bits, so the clean-checkout proof has one narrow evasion

`assume-unchanged` and `skip-worktree` suppress modification reporting, so a
checkout at the exact candidate revision can report empty status while executing
modified bytes. This is the same class as the dirty-checkout hole R-5 closed, one
level down, and it is inside what the boundary paragraph disclaims ("Do not claim
that the result proves honest execution"). Cheapest mitigations, either of which
is a sentence: require acceptance runs in a freshly created checkout, or have the
runner also emit `git ls-files -v` entries whose status is not `H` and require
that set to be empty.

### N-6 (Low) — Validation(13) refers to a field this revision removed

**Changed paths and surfaces** now states "The v2 schema has no `out_of_scope`
field; `additionalProperties: false` rejects it", which is the right resolution of
F-13. Validation(13) still reads "contradictory unavailable/out-of-scope state",
which cannot be constructed against a schema that rejects the property. Restate
it as two tests: an `out_of_scope` property is rejected, and an `unavailable`
state inconsistent with its observations is rejected.

### Nits (no severity)

- "The predecessor candidate has **one** child on its completed review chain and
  **a** sibling packet commit" reads as a uniqueness claim about children. A lane
  with an abandoned attempt would have three; nothing breaks, because the record
  declares its packet commit and the write sets are disjoint, but the sentence
  claims more than the mechanism needs.
- Retention integrity is verified only by the successor generation (R-7's
  accepted design). Worth one sentence in the operating documentation so a
  manager citing a fresh decision knows retention has not yet been machine-checked.

## 6. Operator burden

The burden governance from `d491690` is unchanged and remains the right
instrument: **Pre-implementation self-review** requires minimal and realistic
required-leaf estimates, a comparison against v1's measured 76-minimal /
approximately-320-realistic shape, a machine-verified versus human-judgement
split, and a stop for manager adjudication if the design is unexpectedly larger.

This revision adds authored surface in two places and removes none: the
manifest's prior-obligation rows now carry an ID→closure-validation-ID mapping
(rather than bare IDs), and `CLOSED_PROVEN` items must bind those mapped
validations in the record. Both are small relative to what earlier revisions
removed (`outcomes`, stopping booleans, `closure_contract`/`oracle` prose,
`out_of_scope`), and both replace judgment with a machine check, which is the
trade the whole contract is built on. The dominant remaining cost is still one
scope-dispositioned surface row per changed path, including documentation and
packet paths — a disclosed consequence of deleting `no_surface_change`.

I name no threshold, no target, and no acceptable leaf count. The §12 measurement
with its stop condition is the correct place for that judgment, and the manifest
freeze is where set size is decided by the manager rather than by the gate.

## 7. Machine proof versus judgment, as the frozen contract now draws it

Recorded because acceptance depends on this boundary being honest rather than
generous.

| Machine-proved by the gate | Left to manager or reviewer judgment (and said so) |
|---|---|
| Predecessor identity equality; rooted transitive chain with per-link topology, write sets, modes, digests, record/result equality, generation continuity, cycle rejection | Whether the manager's extraction from legacy Markdown is complete (`predecessor_obligation_completeness: unavailable`; independent packet preflight) |
| Packet/manifest two-path direct-parent topology; Git-derived non-empty `repair_surface`; per-commit inspection | When any text was authored; any rewritten history that presents a conforming topology |
| Candidate revision/tree equality of timed v2 results; clean-before/after emptiness; result/log digests and byte counts; manifest↔record validation set equality | Honest execution; semantic oracle quality; nonzero test selection; mutate/revert inside a command |
| Exact changed-path set equality; path→surface linkage; one nonblocking state derived from enums plus mapped passing validations | Complete semantic symbol enumeration; whether a bound oracle semantically proves closure; whether the manifest set is complete; human authorship of any artifact |
| `finding_target_policy: "none"` and six named prohibited quota fields | Semantic proxy detection beyond those names |

Every row on the right is stated in the contract itself, in the section where the
corresponding claim is made. I found no place where a judgment is presented as a
machine proof, which was the original independent review's core complaint about
v1.

## 8. Explicitly unavailable evidence

- **No v2 implementation exists** and the contract forbids one. Every
  adjudication here is of text, against artifacts the text binds. I executed no
  gate, no counterexample, and no test of gate behaviour.
- **The document gate set could not be run against the frozen tree.** My worktree
  is a sibling review branch whose packet copy predates `ada80f7`, and creating a
  checkout of `41824af` would add repository state, so I declared it unavailable
  rather than doing it. Instead I extracted `TASK_PACKET_SECTIONS` from
  `check_ai_acceleration_docs.py` and tested all six markers directly against the
  frozen bytes — stronger evidence for the specific question than a whole-tree
  run. `git diff --check`, `check_ai_contracts.py` and `check_project_docs.py`
  were run in this worktree and pass; `check_ai_acceleration_docs.py` fails here
  on the five section markers missing from *my worktree's older packet copy*,
  which is a pre-existing condition of this branch and not a property of the
  frozen subject.
- **I did not check out, build, or run anything on either bootstrap or candidate
  lane branch.** All port and bootstrap verification is Git object inspection
  (parentage, write sets, modes, digests, presence/absence, ancestry). Whether a
  ported candidate's validations actually pass on a clean checkout is exactly what
  the live arms must measure.
- **`successor_obligation_ids`, the v2 result, the manifest schema and
  `resolve_predecessor_authority_chain` do not exist yet.** Their shapes,
  uniqueness rules and interactions are unverifiable at this revision; N-2 and
  N-4 are consequences of that, not observations of code.
- **Whether the frozen inventories at `0304c0e1` / `8437a8e0` faithfully
  enumerate their reports' obligations is not decidable** and the contract says so.
  I did not re-derive either arm's obligation set from Markdown.
- **Operator burden for v2 remains unmeasured**; no schema exists. v1's 76 / ≈320
  are the original independent review's measurements.
- **Rust predecessor-build availability** remains unavailable; the contract
  converts that into an explicit fail-closed limit I could not test.
- **Manager intent on N-1's two roles is unrecorded**, so it is stated as the gap
  it is rather than resolved by assumption.
- **I changed no packet, schema, tool, test, product byte, prior report, or
  recorded verdict**, and did not merge, rebase, cherry-pick, or edit the subject.

## 9. Repository status

- Review branch:
  `claude-agent/cfr168-review-recurrence-v2-contract-preflight-claude-20260731`,
  head `6513a1c` before this report's commit.
- Worktree clean at start; this report is the only added path.
- Subject branch untouched; `41824af` is not an ancestor of this branch and was
  read only via `git show`.
- Read-only checks run in this worktree: `git diff --check` (exit 0),
  `check_ai_contracts.py` (exit 0), `check_project_docs.py` (exit 0). See §8 for
  the one gate that is unavailable here and how its packet requirement was
  verified against the frozen bytes instead.

## 10. Disposition

This exact packet is internally consistent, machine-testable where it claims to
be, honest where it is not, proportionate under a measurement-with-stop rather
than a threshold, and sufficient against F-1 … F-14, C-1 … C-16 and R-1 … R-10.
Its sixteen frozen identities all verify. The six new items are one-sentence
specification and sequencing gaps; N-1 and N-2 must be answered in the mandatory
pre-implementation design table before implementation bytes, N-3 must be answered
before either live arm runs, and N-4 … N-6 can land with the same pass.

This accepts the contract for implementation dispatch only. It does not accept
any implementation, schema, tool, record, product candidate, or live-arm result,
and it does not authorize the canonical ports or the Cargo/KGI comparison. The
implementation must still pass a separate Claude Opus 5 adversarial review, and
the manager still owns the dispatch and adoption decisions.

CONTRACT_VERDICT: ACCEPT_FOR_IMPLEMENTATION
