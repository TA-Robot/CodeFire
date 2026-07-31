# CFR-168 Recurrence Gate v2 Repair Contract — Preflight Addendum (revision `d491690`)

Reviewer: Claude Opus 5 (`claude-opus-5`), same independent contract-preflight
lane as `review.md`, report-only. Date: 2026-07-31.
Addendum to `review.md` (`ccd5585c…`); that report stands unchanged.
Subject: the **revised contract**, not an implementation. No v2 implementation
exists at this revision, and this lane wrote no schema, tool, test, product,
packet, or prior report byte.

## 1. Exact revised identity

| Item | Value | How established |
|---|---|---|
| Revised contract commit | `d4916906ccc17a56c587c345c3b2818c8eeca9b9` | `git cat-file -t` → `commit` |
| Commit subject / time | `docs(ai): close recurrence v2 contract preflight findings`, 2026-07-31 14:18:56 +0000 | `git log -1` |
| Commit write set | exactly the contract path (`+359 / −136`) | `git show --stat` |
| Contract path | `docs/development/ai/task-packets/cfr168-review-recurrence-gate-v2-repair-20260731.md` | — |
| Contract SHA-256 | `aa3faca47550904949ea23dc28be539bb46d973c6fe6c6896b54344abb3eea20` | `git show d491690:<path> \| sha256sum` — **matches the value given for verification** |
| Prior contract revision | `a79acc0acc50f96868860fd7db5eef051488e420`, SHA-256 `15931ddb…` | `review.md` §1 |
| Revised length | 680 lines (prior: 456) | `wc -l` on the extracted blob |
| My review, as integrated | commit `102fc1232b9a5140c72be94b8add51b5a58c5634`, mode `100644`, SHA-256 `ccd5585c79acb19e7c412b336e40930dd984ea4e26217486f659a1132f6bec0b` | recomputed; **byte-identical to what I committed** |
| Worktree at start | clean | `git status --porcelain` empty |

The contract blob was inspected via `git show d491690:<path>` only. I did not
merge, rebase, cherry-pick, or edit it, and my worktree remains on
`claude-agent/cfr168-review-recurrence-v2-contract-preflight-claude-20260731`
at its own head.

Intervening commits between `a79acc0` and `d491690`, all single-path:

| Commit | Path touched | Note |
|---|---|---|
| `8437a8e` | `…/task-packets/cfr168-development-flow-kgi-kpi-r4-rereview-with-recurrence-20260731.md` | now a §8.1 frozen inventory source |
| `ada80f7` | the contract | heading rename only (`## 1. Objective` → `## Objective`, `## 10. Scope` → `## Scope`, adds `## Non-Goals`, `## 11.…` → `## Validation`, `## 13.…` → `## Handoff`), satisfying `check_ai_acceleration_docs.py`'s `TASK_PACKET_SECTIONS`; no clause changed |
| `102fc12` | my preflight report | integrated byte-identically |
| `d491690` | the contract | the revision under review |

Because of `ada80f7`, the contract's own section numbers are now
non-contiguous (`Objective`, `2`–`10`, `Scope`, `Non-Goals`, `Validation`, `12`,
`Handoff`). This addendum cites headings by name where the number is gone, and
uses `§Validation(n)` for the numbered counterexample list that was §11.

### 1.1 Independently verified manager-fixed bindings

Every identity the revision newly fixes was recomputed here. All pass.

| Binding | Requirement in the contract | Result |
|---|---|---|
| §2 independent review `2bec2241` | commit, mode `100644`, digest `02bc2030…` | **verified**; write set is exactly the report path |
| §2 root-cause audit `abfee6ef` | commit, mode `100644`, digest `2392450e…` | **verified**; write set is exactly the audit path (so "report commit" is exact, not approximate) |
| §2 replay result `94f2ab4b` | commit, mode `100644`, digest `527b800c…` | **verified**; write set is exactly the pilot path |
| §2 contract preflight `102fc123` | commit, mode `100644`, digest `ccd5585c…` | **verified**; write set is exactly the report path |
| §4.1 Cargo bootstrap | legacy report-only commit `130878d3`, sole parent = predecessor `4e5c0e7f`, write set exactly the report path, mode `100644`, digest `aad067b2…` | **all five verified** |
| §4.1 KGI/KPI bootstrap | legacy report-only commit `12f9d0b5`, sole parent = predecessor `753c8db1`, write set exactly the report path, mode `100644`, digest `23b59e01…` | **all five verified** |
| §8.1 Cargo frozen inventory | packet commit `0304c0e1`, digest `db0d58cc…` | **verified**, mode `100644` |
| §8.1 KGI/KPI frozen inventory | packet commit `8437a8e0`, digest `4b5e6603…` | **verified**, mode `100644` |
| §4.2 port feasibility | existing candidates `74cd37d5` / `9ca01d9a` are off-lineage from their packets | **verified for the off-lineage claim**: each is a *direct child* of its manager-fixed predecessor with no packet commit between, so the non-conforming topology is real and a port is genuinely required. The port itself is executable as written for Cargo and **not** for KGI/KPI — see R-10 |

This is the first artifact in this lineage whose declared identities I could
check exhaustively and find nothing wrong. It is worth recording as a positive
result, because the same standard produced eight defects last round.

## 2. Adjudication summary

**All sixteen prior findings C-1 … C-16 are closed.** Not one is closed by
renaming: each has a named mechanism, and in the four cases where I could test
the mechanism against repository facts (predecessor authority, bootstrap
topology, port feasibility, timed-gate consumers) the facts support it. Three
of the closures are stronger than the edits I asked for — `no_surface_change`
was deleted rather than corroborated, `SCOPE_BOUNDARY` now blocks at the machine
gate rather than being merely derivable, and `CONTROL_PATH_PREFIXES` is
explicitly excluded by name rather than replaced by a new prefix list.

Two **new** High residuals were created by the revision's own new mechanisms,
and both are one clause each:

1. **R-1 — the predecessor-authority chain is unrooted after bootstrap.** §4.1
   roots generation `N` in a manager-fixed artifact only for the two bootstrap
   arms. From generation `N+1` onward the authority is "a committed v2 record",
   whose entire content — including `candidate_revision`, `generation`, and the
   `successor_obligation_ids` that seed the next manifest — is author-written
   bytes. A lane that can commit can synthesize the whole review chain, which
   reopens C-1's interval narrowing and C-6's obligation thinning one generation
   later.
2. **R-2 — `CLOSED_PROVEN` is the sole nonblocking state and carries no
   evidence obligation.** The blocking derivation is now exact (C-4 closed), but
   it is exact over a label the author chooses freely. §8.3 removes "unchecked
   'all done' stopping booleans"; the same object returns as an enum value.

A third finding is an operability defect rather than a bypass: **R-10 — the
canonical port instruction cannot be executed as written for the KGI/KPI arm**,
because that arm's historical repair dispatch artifacts live in a different
namespace and are inside the candidate commit's own write set, i.e. they were
committed together with the repair bytes — the exact condition §4.2 makes
non-`ALLOW`. Cargo's port is clean. The remaining seven residuals are
clarifications: sentences that must say which of two readings is intended.

Verdict is therefore `REQUEST_CHANGES` again, but on a materially different
footing: last round three of my four blocking findings were *bypasses of stated
guarantees*; this round the two blocking findings are *unbound inputs to
otherwise sound mechanisms*, and the fix for each is a clause, not a design.
I am not moving the goalposts — R-1 and R-2 attach to mechanisms that did not
exist at `a79acc0` (predecessor-record chaining, and a nonblocking state), so
neither could have been raised in `review.md`.

## 3. C-1 … C-16 disposition

| Finding | Revised mechanism (heading) | Verified against | Disposition |
|---|---|---|---|
| C-1 predecessor still author-chosen | §4.1: `predecessor_authority` = prior v2 record **or** one of two manager-fixed legacy report-only commits; authority resolved *before* the predecessor field is accepted; exact equality; "A later ancestor cannot be substituted merely because it is on candidate ancestry"; §Validation(1) restated as an interval narrowed to a later ancestor | both bootstrap arms verified on all five conditions | **CLOSED**; residual R-1 for generations ≥ 2 |
| C-2 undefined manager-control path set | §4.2: packet commit = direct child of predecessor, write set exactly `{Markdown packet, JSON manifest}`; "The admissible manager-control path set in this pre-repair window is therefore exactly those two paths; it is not `CONTROL_PATH_PREFIXES`"; `repair_surface = changed_paths − {packet, manifest}`, non-empty; per-commit inspection; "no product/control taxonomy" | port feasibility verified; `CONTROL_PATH_PREFIXES` exclusion is explicit | **CLOSED**; residual R-4 (topology statement) |
| C-3 evidence bytes without provenance | §3 + §5: new `codefire.timed_gate.v2` derives Git root/`HEAD`/`HEAD^{tree}`; gate requires result revision+tree == candidate; validation IDs and argv set-equal to the manifest; role→revision table for every artifact reference; "Object existence or a dangling/throwaway-branch commit is insufficient" | no other consumer parses the v1 schema string; no out-of-scope doc pins the result shape | **CLOSED**; residuals R-3 (reachability basis), R-5 (clean checkout) |
| C-4 blocking underivable / self-authorable authority | §7: `CLOSED_PROVEN` + `next_action: NONE` is the only nonblocking state; `SCOPE_BOUNDARY` "remains blocking at this machine gate"; owner/release/pre-candidate authority "retained for routing evidence only and cannot change the v2 blocking derivation"; "Git binds its bytes and ancestry but cannot attest human authorship"; §Validation(5) | — | **CLOSED**; residual R-2 |
| C-5 `no_surface_change` escape | §8.2: escape deleted; every changed path — "including packet, test, schema, tool, and documentation paths" — is `reviewed` with ≥1 linked surface row; every surface carries the same exact scope disposition; `kind` diagnostic only and "cannot change validation or blocking"; §Validation(7) rejects an attempted `no_surface_change` property | — | **CLOSED**, both halves of F-7 |
| C-6 no obligation-set anchor | §8.1: manifest prior-obligation IDs set-equal to the predecessor record's emitted `successor_obligation_ids`, including items recorded closed; continuity **derived** (`recurred_after_proven_closure` / `continuous_open`); "The author cannot choose a weaker label"; bootstrap completeness explicitly `unavailable` and cannot produce machine `ALLOW` | frozen inventory digests verified | **CLOSED**; residuals R-1, R-6 |
| C-7 record unplaced / mode contradiction | §4.3: five-step order; `--record` deliberately uncommitted at evaluation; gate emits canonical record SHA-256 in the result; retention commit = direct child of evidence adding exactly record + result under named namespaces, both `100644`, committed record digest == result-emitted digest | — | **CLOSED**; residual R-7 |
| C-8 inverted injection-only closure | §7: blocks unless an exact independently reviewable **non-reachability** boundary is committed and bound; "The validator proves artifact identity only, not the semantic non-existence claim"; closure recorded as `CLOSED_PROVEN` rather than silent de-blocking; §Validation(23) | — | **CLOSED** |
| C-9 I5 metric incomparable | §7: derived `product_repair_required`; metric defined as product-repair generations avoided with full denominator and separate unavailable count; "deliberately not the replay's `6 / 18` … and must not be compared as though it were" | matches pilot §6.5/§9 | **CLOSED** |
| C-10 I1 widened / Rust caveat dropped | §6: derivation rows set-equal to **manifest parity/closure claim IDs**; "surface count does not create additional I1 obligations"; outputs labelled row coverage; "no Rust arm may use v2 `ALLOW` until a predecessor build is available … the explicit fail-closed consequence of the replay's five unavailable Rust rows, not an inference that I1 was validated on Rust" | matches pilot §4.5/§5.2/§9 | **CLOSED**; residual R-8 |
| C-11 chronology overclaim | §Objective(2) reworded to "in the presented Git topology"; §4.2 states Git "cannot detect a rewritten history that presents a different topology"; the port "must not be described as proof of the original lane's authorship time" | — | **CLOSED** |
| C-12 §2 fails its own evidence standard | §2 binds all four governing artifacts by commit + mode + digest; probe tally quoted verbatim with "only the bypasses the report identifies by finding/reproducer are treated as false `ALLOW`" | all four recomputed and matching | **CLOSED** |
| C-13 schema-registration scope trap | §3 + Scope: `timed-gate-result.v2.schema.json`, `run_timed_gate.py`, and `check_ai_contracts.py` "limited to schema registration" are in scope; "its only permitted change is the corresponding `SCHEMAS` entries and tests required for them" | verified no hidden v1 consumer and no out-of-scope doc pins the shape | **CLOSED** |
| C-14 F-10 position unspecified | §9 F-10 row + §Validation(10): detect **before schema validation** and JSON-escape the offending bytes in a structured finding | consistent with the encoder asymmetry (`canonical_json_bytes` raises; the result writer escapes) | **CLOSED** |
| C-15 five non-testable instructions | §4.3 direct-parent "mandatory" + per-commit for every other interval; artifact roles structural; single `next_action` with an exact enum; `CLOSED_PROVEN` named; §8.3 names six prohibited quota fields and states "Semantic proxy detection beyond those named fields remains an independent review judgement" | — | **CLOSED** (all five) |
| C-16 validation set / v1 before-evidence | §Validation pins v1's before behaviour to `1cae0225` or a labelled synthetic fixture ("deleting v1 authorization at HEAD is not evidence of its old behaviour") and restores `check_ai_development.py` | — | **CLOSED** |

## 4. F-1 … F-14 re-adjudication

Every F-level mechanism is now present and machine-testable. Four carry a
residual, and in each case the residual is an R-finding against a *new*
mechanism rather than a surviving v1 bypass.

| Finding | Mechanism at `d491690` | Verdict |
|---|---|---|
| F-1 base narrowing | authority-resolved predecessor with mandatory equality (§4.1) | **CLOSED** for the bootstrap arms; carries R-1 from generation ≥ 2 |
| F-2 post-repair packet | two-path packet commit as direct parent of the repair series, per-commit inspection, rewrite boundary stated (§4.2) | **CLOSED**; carries R-4 |
| F-3 fake dispatch event | `dispatch_event_attested`, branch-external packets and timestamp-only assertions cannot produce `ALLOW` (§4.2) | **CLOSED** |
| F-4 fake evidence/log digests | candidate-bound timed v2 result + manifest-anchored validation set + committed result/log bytes (§5) | **CLOSED**; carries R-5 |
| F-5 dead severity/blocking | one nonblocking state, derived from exact enums (§7) | **CLOSED** as a derivation; carries R-2 on the label |
| F-6 dead continuity | obligation set equality + derived continuity, no weaker label (§8.1) | **CLOSED**; carries R-1 |
| F-7 one surface covers many paths | ≥1 scope-dispositioned surface per exact changed path; no escape; `kind` diagnostic (§8.2) | **CLOSED**, both halves |
| F-8 control rewrites gate | direct-child, single-path review control; explicit no-rewrite (§4.3) | **CLOSED** |
| F-9 impossible ordering | five-step order with named namespaces and retention commit (§4.3) | **CLOSED**; carries R-7 |
| F-10 invalid UTF-8 path | detect before schema validation, escape in the finding (§9, §Validation(10)) | **CLOSED** |
| F-11 symlink artifacts | mode `100644` for every role in the common artifact-reference definition (§4.3) | **CLOSED** |
| F-12 mutate/revert chain | direct parents mandatory; every other interval per-commit; repair-series paths must be a subset of the net `repair_surface`, so a mutate/revert pair fails | **CLOSED**, and tighter than requested |
| F-13 contradictory unavailable/out-of-scope | consistency enforced or field removed (§8.1, §8.2) | **CLOSED** |
| F-14 self-referential patch fixture | pinned byte-level patch digest fixture (§9, §Validation(14)) | **CLOSED** |

## 5. I1 / I5 fidelity at `d491690`

| Replay input | Contract | Fidelity |
|---|---|---|
| I1 capture is derivation, not differential | §6 binds derivation from the predecessor tree; §Validation(15) uses the byte-identical case | faithful |
| I1 burden measured per parity claim, not per surface | derivation rows set-equal to manifest claim IDs; surface coverage emitted as a *separate diagnostic* | faithful — this is now closer to the pilot than my own suggested edit |
| I1 Rust arm blocked until a predecessor build exists | stated, with the fail-closed consequence and the explicit denial that Rust was validated | faithful |
| I1 metric is row presence, not capture | "These measure row presence and artifact binding, not derivation quality or historical capture" | faithful |
| I5 vocabulary incomplete without a source-proven state | `DEFECT_SOURCE_PROVEN` present with the reason attributed to the replay | faithful |
| I5: `CLOSURE_UNPROVEN` never accepted; no `ACCEPT_WITH_NOTES` | both preserved; §Validation(17) | faithful |
| I5 benefit is generation removal (`6 / 18`) | redefined as product-repair generations avoided, with the incomparability stated in the contract | faithful **and** explicit; C-9 closed |
| I3 `REVISE_AND_REPLAY`: no size limit | no line, file, surface, or delta threshold anywhere; delta remains output | faithful |
| I4 `REVISE_AND_REPLAY`: no E1–E9 checklist | none installed; §5 binds candidate identity without claiming nonzero test selection — the honest half of `E1` without the checklist | faithful |
| Replay's refusal to fix counts/depth | §2 closing paragraph, §8.3, §12 ("measurements with no acceptance threshold") | faithful; this addendum invents no count, field, or size threshold either |

Residual R-8 is the only fidelity gap, and it concerns the bootstrap arms only.

## 6. Bootstrap Cargo/KGI operability

Verified feasible end to end, with one unstated cost.

- Both manager-fixed predecessors and both legacy report commits satisfy every
  §4.1(2) condition exactly (§1.1).
- Both existing candidates are direct children of their predecessors, so the
  §4.2 canonical port is in principle a single-patch replay onto a two-path
  packet commit, and the ported candidate's tree would differ from the reviewed
  candidate's tree only by the packet and manifest — which §4.2 already requires
  be re-identified ("recompute all candidate/review identities").
- For the **Cargo** arm the port is executable: no Cargo packet exists in the
  predecessor tree's `task-packets/` namespace, and `74cd37d5`'s write set
  contains no dispatch artifact, so the two-path packet commit is achievable.
- For the **KGI/KPI** arm it is not, as written — R-10.
- The frozen obligation inventories exist at the named commits with the named
  digests.
- **Unstated cost (R-6).** §8.1 makes a bootstrap record carry
  `predecessor_obligation_completeness: unavailable` and states it "cannot
  produce machine `ALLOW`"; §3 makes unavailable required evidence `BLOCK`; and
  §Validation(21) requires a test proving it. So neither live arm's *first* v2
  generation can reach `ALLOW` under any content. The `ALLOW` path first becomes
  reachable at each arm's second v2 generation, whose predecessor authority is
  the retained v2 record from the first. That is coherent and conservative — but
  it means the Cargo/KGI live comparison needs two reviewed generations per arm
  before the machine authorization is exercised at all, and the contract never
  says so. §4.1's sentence "No other legacy bootstrap may produce `ALLOW`
  without a new manager contract and independent preflight" reads as though
  these two can, which contradicts §8.1 and §Validation(21).

## 7. Residual findings

Ordered by severity, not by number: R-10 was found last and appears after R-4.
Every one attaches to a mechanism introduced by this revision; none is a
restatement of C-1 … C-16.

### R-1 (High) — the predecessor-authority chain is unrooted after bootstrap, reopening C-1 and C-6 one generation later

§4.1 admits two authority forms. Form 2 is manager-fixed and verified. Form 1 —
"a committed v2 record whose bound `candidate_revision` is the predecessor and
whose generation is exactly `N-1`" — is a blob of author-written JSON. Nothing
in §4.1, §4.3, §5 or §8.1 requires that record's own authority chain to
terminate at a §4.1 manager-fixed binding.

**Trace.** A lane preparing generation `N+1` wants the interval floor moved past
a risky commit, or a thinner obligation seed. It forks from the *real*
predecessor candidate `P` and commits, on that fork: a review-packet commit
(single path, `100644`), a report commit (single path), an evidence commit
(paths under `…/review-recurrence/evidence/<control_id>/`), then a retention
commit adding a record and a result under `…/records/` and `…/results/` whose
digests agree. Every §4.3 direct-parent relation, every write set, every mode,
and the record-digest-equals-result-digest check are satisfiable, because the
forger writes all of them. The synthesized record declares
`candidate_revision = P'` (any commit it likes, including a later ancestor) and
a `successor_obligation_ids` set omitting the inconvenient item. Generation
`N+1` then resolves that record as its `predecessor_authority`, and §4.1's
equality check passes against the forged value. C-1's narrowing and C-6's
thinning are both back, one generation downstream of the arms the contract
pinned by hand.

Note the mitigation the design does have, and which should be preserved: a
false `CLOSED_PROVEN` in generation `N` is partly self-punishing, because §8.1
derives `recurred_after_proven_closure` when that item is nonclosed at `N+1`.
That constrains careless labelling, not deliberate synthesis.

**Minimum edit.** Require the `predecessor_authority` chain to be walked
transitively to a §4.1 manager-fixed binding, verifying at every link the
topology (`candidate → control → report → evidence → retention` direct
parents), each commit's exact write set, each blob's mode and digest, and the
record/result digest equality; a chain that does not terminate at a manager-fixed
root cannot produce `ALLOW`. If walking is judged too costly, the equivalent is
to require the manager's frozen dispatch revision for each generation to be
named in the contract or a manager-owned ledger, so the root is external to the
lane. Either is one clause; the second matches the Status line's existing
"manager freezes its dispatch revision" practice.

### R-2 (High) — `CLOSED_PROVEN` is the only nonblocking state and no clause requires it to bind closure evidence

§7 fixes the blocking derivation exactly, then makes everything turn on one
enum value for which the contract states no evidence obligation.
`CLOSED_PROVEN` appears at lines 370, 385, 393, 402 of the revised blob and in
the §9/§10 tables; none of those occurrences requires a bound artifact, a
passing validation, or a reversal. §8.1 binds prior-closure evidence for
`recurred_after_proven_closure` — a *predecessor*-derived label — not for
asserting closure now.

**Trace.** A generation-`N+1` record with a legitimate predecessor authority
records every obligation and every finding as
`evidence_state: CLOSED_PROVEN`, `next_action: NONE`. Its manifest-required
validations run against a clean candidate checkout and pass, so §5 is satisfied.
Its surfaces cover every changed path with a scope disposition and evidence
references that resolve at the evidence commit — a location §5's role table
explicitly sanctions for evidence artifacts, and whose bytes the reviewing lane
authors. Its derivation rows are set-equal to the manifest claim IDs. The
blocking derivation finds no blocking item, and the gate returns
`integration_decision: ALLOW`. §8.3 removed "unchecked 'all done' stopping
booleans"; this is the same object with an enum's type.

**Minimum edit.** Require that any item whose predecessor state was a defect,
oracle, claim, closure-unproven, scope, or requirement state and which this
record records as `CLOSED_PROVEN` must bind at least one manifest-required
validation ID whose committed timed v2 result is passing (or, for
injection-only, the independently accepted non-reachability boundary §7 already
defines), and state the boundary exactly as §5 does: the gate proves the
binding, not the semantic closure. This adds no minimum finding count and no
target — it attaches an existing anchor to the one state that currently has
none.

### R-3 (Medium-High) — "reachability" has no defined basis, and the contract's own bootstrap artifacts fail the strict reading

§5 closes C-3b with a role→revision table plus: "Object existence or a
dangling/throwaway-branch commit is insufficient. Reachability and the
role-to-revision relationship are verified." The basis of reachability — from
the candidate, from a named ref, from any ref — is never stated, and the two
available readings fail in opposite directions. Verified facts:

- Every artifact in both bootstrap arms (`4e5c0e7f`, `130878d3`, `74cd37d5`,
  `753c8db1`, `12f9d0b5`, `9ca01d9a`) is **not** an ancestor of `d491690` or of
  this lane's `HEAD`. Under an ancestry-based reading the gate rejects the
  bindings the contract itself fixes.
- All six **are** reachable from lane branches under
  `refs/heads/claude-agent/…` (198 refs exist in this repository). Under an
  "any local ref" reading they resolve — but so does any commit an author
  fabricates and then points a branch at, which is precisely the
  throwaway-branch case §5 and §Validation(22) exist to reject.

**Minimum edit.** Define the basis per role rather than globally: roles whose
revision is pinned by the contract or by the Git-bound manifest (bootstrap
reports, packet/manifest, candidate-relative artifacts) need no reachability
test at all — the pinned commit plus write set plus digest is the stronger
check; roles that are not pinned (the prior v2 record, per R-1) require a
verified topological relation (descendant of the predecessor candidate, chain
inspected per commit) rather than ref reachability. State that ref existence is
never evidence.

### R-4 (Medium-High) — the steady-state topology is a fork the contract never states, and the manifest must embed data that is created later on a sibling branch

Two direct-parent rules meet at the same commit. §4.2: the packet commit is the
direct child of the predecessor candidate. §4.3: review control is the direct
child of the candidate. For generation `N+1`, the predecessor candidate is
generation `N`'s candidate, which must therefore have two children on divergent
lines — generation `N`'s review chain, and generation `N+1`'s packet commit.
The repository already works this way and it is worth recording as support, not
as an objection: `4e5c0e7f` has exactly this shape, with the report commit
`130878d3` on one branch and the next repair `74cd37d5` on another.

The consequence the contract does not address is ordering of *content*. §8.1
requires the manifest — inside the topologically earliest commit of the interval
— to carry prior-obligation IDs set-equal to the predecessor v2 record's emitted
`successor_obligation_ids`. That record is created after the predecessor
candidate, on the sibling review branch, so the pre-repair manifest must embed
data produced later and off-lineage. This is realizable (Git position is not
authoring time, as §4.2 now says) but it means the predecessor record can never
be an ancestor of the candidate under review — which is exactly the role R-3
leaves without a basis, and it is the contradiction §12 asks the implementer to
find while giving them no authority to choose the topology.

**Minimum edit.** State the topology once: name the fork, state that the
predecessor generation's review chain and the successor generation's packet
commit are sibling children of the predecessor candidate, and pair it with R-3's
role-specific basis. The simpler alternative, which matches how the canonical
documentation line in this repository actually accumulates packets and reports,
is to relax §4.2 to "the packet commit is the first commit after the predecessor
generation's retention commit, and every commit in the pre-repair window is
inspected per commit with a write set restricted to the two packet paths" —
which keeps one linear history, keeps the predecessor record on ancestry, and
loses nothing, because the per-commit write-set rule already does the work the
direct-parent rule was added for.

### R-10 (Medium-High) — the canonical port is unnamed and, for the KGI/KPI arm, cannot be executed as written

§4.2 requires the port to "commit the byte-identical historical repair packet
plus a new manifest" in a commit whose write set is exactly two paths, both
"in `docs/development/ai/task-packets/`". Unlike every other identity this
revision fixes, neither arm's historical repair packet is named by path, commit,
or digest, so the implementer must choose the artifact — and for KGI/KPI every
available choice fails one clause.

**Trace (verified).** The KGI/KPI candidate `9ca01d9a`'s own write set contains
`docs/development/ai/swarm-batches/development-flow-kgi-kpi-zero-residual-repair-r3-20260731.md`
and `…-r4-20260731.md`. So that arm's historical repair dispatch artifacts (i)
live in `swarm-batches/`, not the `task-packets/` namespace §4.2 mandates, and
(ii) were historically committed **inside the same commit as the repair bytes** —
precisely the condition §4.2 declares cannot produce v2 `ALLOW`. The port
therefore cannot commit them byte-identically at their historical paths, and if
the implementer instead reaches for the one KGI packet already in the
predecessor's `task-packets/` namespace,
`development-flow-kgi-kpi-zero-residual-repair-20260730.md`, that file is
**byte-identical at `753c8db1` and `9ca01d9a`** (digest recomputed at both), so
re-committing it changes nothing and the two-path write set degenerates to one
path — also non-conforming. Cargo has neither problem: `74cd37d5` changes no
dispatch artifact, and no Cargo packet exists at `4e5c0e7f`.

**Second-order point.** Because `repair_surface` is
`changed_paths − {repair_packet_path, repair_manifest_path}`, any *other*
dispatch or instruction artifact committed inside the repair series is inside the
permitted surface and invisible to the precedence proof. §4.2's guarantee is
therefore about the one declared packet, not about every artifact that governed
the repair. §8.2 at least forces such paths to appear as reviewed surfaces, so
they are visible in the record; the contract should say that the proof is
declared-packet-scoped.

**Minimum edit.** Name, by exact path, commit, and SHA-256, the artifact each
arm's port must commit as its Markdown repair packet — the same treatment §4.1
and §8.1 already give the predecessor authorities and frozen inventories. State
that the chosen artifact must be absent or different at the predecessor tree so
the two-path write set holds; state whether the ported repair patch retains or
drops any historical dispatch artifact it carried; and state that §4.2's
precedence proof is scoped to the declared packet.

### R-5 (Medium) — "This closes a result borrowed from another worktree" overclaims: `HEAD^{tree}` is the committed tree, not the executed bytes

§5 has `run_timed_gate.py` derive the Git root, canonical `HEAD`, and
`HEAD^{tree}`, and the gate require equality with the candidate. `HEAD^{tree}`
is the tree of the commit, so a worktree checked out at the exact candidate with
modified files — a stubbed test, a patched assertion — emits a v2 result whose
revision and tree both equal the candidate while the executed bytes differ. That
is still "a result borrowed from another worktree", so the sentence is wrong at
the margin even though §5's boundary paragraph ("Do not claim that the result
proves honest execution") covers the general case.

**Minimum edit.** Since `run_timed_gate.py` is now in scope, require it to
derive and emit worktree cleanliness (porcelain status empty, or the index/
worktree digest) and require the gate to fail closed on a dirty acceptance run;
otherwise narrow the claim to "a result produced from a different revision or
tree".

### R-6 (Medium) — the bootstrap arms can never reach `ALLOW`, which contradicts §4.1's wording and doubles the live-comparison cost

See §6. §4.1's "No other legacy bootstrap may produce `ALLOW` without a new
manager contract and independent preflight" presupposes that the two named arms
may; §8.1 and §Validation(21) say no bootstrap record can.

**Minimum edit.** Replace that sentence with the actual rule ("no bootstrap
record produces machine `ALLOW`; no other legacy binding may serve as
predecessor authority at all without a new manager contract and independent
preflight"), and state the operational consequence: each live arm requires a
first v2 generation that blocks by construction and whose retained record seeds
a second generation in which `ALLOW` first becomes reachable.

### R-7 (Medium-Low) — the record/result retention invariant has no named machine reader

§4.3's retention rule is an operator obligation phrased around "Before a manager
cites the decision". The gate run that emits the result has already finished, so
nothing in the contract names the component that verifies the retention commit's
parentage, write set, modes, and record-digest equality. §5's role table says a
prior v2 record "resolves at its retained record commit" but does not require
the resolver to check the retention invariant.

**Minimum edit.** Make the successor generation's predecessor-authority
resolution the named reader: verify the retention commit is a direct child of
the evidence commit, that its write set is exactly the record and result paths,
and that the committed record digest equals the result's emitted digest. This
also supplies half of R-1's chain walk, so the two edits share a mechanism.

### R-8 (Low) — bootstrap manifest: which collections the frozen inventories seed is ambiguous

§4.2 says the manifest contains three collections: prior-obligation IDs, parity/
closure claim rows, and required validation IDs with argv. §8.1 says "For the two
bootstrap arms, the manifest derives its IDs from the exact frozen inventories"
at the two named packet commits. "Its IDs" is ambiguous between the
prior-obligation IDs alone and all three collections, and the I1 obligation is
scoped by the parity/closure claims (§6), so the ambiguity decides how much of
the bootstrap I1 experiment is manager-anchored versus author-chosen.

**Minimum edit.** Say which collections the frozen inventories seed, and if
parity/closure claims and validation IDs are author-authored for the bootstrap,
say that too.

### R-9 (Low) — the manifest is now the discretion boundary, and the contract does not say so

Anchoring the record to a pre-repair, Git-frozen, manager-visible manifest is the
right move, and it removes post-hoc weakening completely: §5 forbids the record
from adding, omitting, or weakening a required validation, and §8.1 fixes the
obligation IDs from the predecessor. What remains author-chosen is the manifest's
own parity/closure claims and required validations, authored at packet time. A
thin manifest therefore yields a thin gate, with every downstream set-equality
check passing exactly.

This must not be fixed with a minimum — the count-free rule is correct and I
propose no threshold. It should be *stated*, and tied to the Status line's
existing manager dispatch freeze so the manifest is reviewed when the dispatch
revision is frozen rather than treated as an implementation detail.

### Documentation nits (no severity)

The heading rename at `ada80f7` left the contract's section numbers
discontinuous (`Objective`, `2`–`10`, `Scope`, `Non-Goals`, `Validation`, `12`,
`Handoff`) while §2, §5, §9 and §10 still cross-reference numbered sections.
Internal cross-references remain correct; only the sequence reads oddly. Renumber
or drop the numbers when the next edit lands.

## 8. Operator burden at `d491690`

The burden governance I asked for is present and is stronger than my edit: §12
now requires a pre-implementation estimate of minimal and realistic required-leaf
counts, a comparison against v1's measured 76-minimal / ≈320-realistic shape, a
split between machine-verified and human-judgement leaves, and — the part I did
not ask for — "if the design is unexpectedly larger, stop for manager
adjudication rather than hiding the burden until handoff".

The direction of the burden is up relative to `a79acc0`, and honestly so:
deleting `no_surface_change` means every changed path, including documentation,
schema, packet and manifest paths, needs at least one surface row carrying a
scope disposition and evidence fields, and the manifest adds a second
authored artifact per generation. Against that, I1 obligations no longer scale
with surfaces, `outcomes` is derived, continuity is derived, and the validation
set is transcribed once into the manifest rather than re-asserted. I name no
threshold and no target; the §12 measurement is the correct instrument and it now
has a stop condition.

One consequence worth recording for the manager rather than the implementer:
because `kind` is now diagnostic and every surface carries the same scope
disposition, the discriminating signal v1 obtained from
`public_api`/`state_machine`/`classifier`/`persisted_shape` moves entirely into
human judgement. §8.2 states that boundary ("complete semantic symbol
enumeration remains a human review judgement and must not be described as
machine-proved"), so this is a disclosed trade, not a defect.

## 9. New-contradiction checks that came back clean

Stated because the ask was to look for them specifically, and a negative result
is evidence too.

- **Circular manifest/packet digests.** The manifest binds the Markdown packet's
  digest one-directionally; the packet does not bind the manifest. Their mutual
  binding is the two-path write set of a single commit. Non-circular, as §4.2
  claims.
- **Self-authorizing record.** The record is uncommitted at evaluation; the gate
  emits its canonical digest; retention commits the byte-identical record and
  result afterwards and compares digests. No artifact authorizes its own
  creation. (R-1 and R-2 concern the *authenticity of content*, not
  self-reference.)
- **Circular record/result digests.** The result carries the record digest, not
  its own; the retention check is one-directional. Non-circular.
- **Repair-series laundering.** Because intermediate commit paths must lie within
  the *net* `repair_surface`, a mutate/revert pair inside the repair series fails
  rather than disappearing. Stronger than F-12 required.
- **Hidden shared-helper breakage from timed v2.** No tool other than the future
  recurrence gate parses the `codefire.timed_gate.v1` schema string;
  `test_ai_tools.py` imports `run_gate`/`timing_evidence` directly and is in
  scope; the two out-of-scope documents mentioning the runner
  (`ai/README.md:173`, `stage5-manual-control-runbook.md:318`) describe behaviour
  and an invocation, not the result shape, so no out-of-scope documentation edit
  is forced.
- **Bootstrap packet-path collision.** Clean for Cargo only. The equivalent check
  fails for KGI/KPI, which is R-10.

## 10. Explicitly unavailable evidence

- **No v2 implementation exists**, and the contract forbids one. Every finding
  here is a defect in text, adjudicated against artifacts the text binds. I
  executed no gate, no counterexample, and no test.
- **I did not check out, build, or run anything on either bootstrap lane
  branch.** The bootstrap verification is Git object inspection only (parentage,
  write sets, modes, digests, ancestry). Whether the ported candidate's
  validations actually pass on a clean checkout is unavailable and is precisely
  what the live arms must measure.
- **`git rev-list` reachability was computed in this worktree only.** The
  198-ref set here may differ from the canonical repository or from a fresh
  clone, which is part of why R-3 matters; I cannot state what refs the gate will
  see at run time.
- **Whether the retained legacy reports' obligation inventories are complete is
  not decidable**, as §8.1 itself records. I did not attempt to re-derive either
  arm's obligation set from its Markdown, and I therefore cannot say whether the
  frozen inventories at `0304c0e1` / `8437a8e0` match those reports' findings —
  only that the packets' bytes are what the contract says they are.
- **Which artifact is each arm's "historical repair packet" is not determinable
  from the contract.** R-10 rests on what the candidates' own write sets contain
  and on the packet namespaces at the two predecessor trees, not on a
  reconstruction of either lane's dispatch history. If the manager has a specific
  artifact in mind for either arm, naming it is the fix; if the intended artifact
  is one I did not consider, R-10's second clause (absent-or-different at the
  predecessor) still applies and should still be stated.
- **`successor_obligation_ids` has no existing emitter.** It is a v2 result field
  that does not exist yet, so its shape, uniqueness rules, and interaction with
  the manifest are unverifiable at this revision.
- **Operator burden for v2 remains unmeasured**; no schema exists. v1's 76 / ≈320
  are the independent review's measurements. I name no target.
- **Rust predecessor-build availability** remains unavailable, per the replay's
  five `[U]` rows; §6 now converts that into an explicit fail-closed limit, which
  I could not test.
- **Manager intent on two open readings** — reachability basis (R-3) and steady-
  state topology (R-4) — is unrecorded, so both are stated as dichotomies with a
  recommended resolution rather than as single defects.
- **The document gate set could not be run against `d491690`'s tree.** This
  lane's worktree sits on the contract as of `a79acc0`, before `ada80f7`'s
  heading rename, so `check_ai_acceleration_docs.py` fails here on five missing
  `TASK_PACKET_SECTIONS` markers in the contract — a pre-existing condition of
  this worktree, unrelated to this addendum, and one the revised blob does not
  have (all five markers are present at `d491690`). `git diff --check`,
  `check_ai_contracts.py` and `check_project_docs.py` pass here.
- **I changed no packet, schema, tool, test, product byte, prior report, or
  recorded verdict**, and did not merge, rebase, or edit the contract.

## 11. Minimum edits before implementation

Blocking (each defeats a numbered `Objective` guarantee):

1. **§4.1/§5 — root the predecessor-authority chain** at a manager-fixed binding,
   verifying every link's topology, write set, mode, and digest; a chain that
   does not terminate at a §4.1 root cannot produce `ALLOW`. (R-1, and it
   subsumes R-7.)
2. **§7 — attach closure evidence to `CLOSED_PROVEN`** for any item whose
   predecessor state was a defect/oracle/claim/closure/scope/requirement state,
   using the manifest-required validation IDs (or the non-reachability boundary
   §7 already defines), with the binding-not-semantics boundary stated. (R-2)

Blocking for the KGI/KPI live arm specifically:

3. **§4.2 — name each arm's port packet artifact** by path, commit, and digest;
   require it to be absent or different at the predecessor tree; state the
   disposition of any historical dispatch artifact inside the repair patch; and
   scope the precedence proof to the declared packet. (R-10)

Required clarifications, all one or two sentences:

4. **§5 — define the reachability basis per role**; state that ref existence is
   never evidence. (R-3)
5. **§4.2/§4.3 — state the steady-state topology**, or relax the packet
   commit's direct-parent rule to "first commit after the predecessor retention
   commit" with the existing per-commit write-set rule. (R-4)
6. **§5 — derive worktree cleanliness** in timed v2, or narrow the
   borrowed-result claim. (R-5)
7. **§4.1 — fix the bootstrap `ALLOW` sentence** and state the two-generation
   consequence for the live comparison. (R-6)
8. **§8.1 — say which manifest collections the frozen inventories seed.** (R-8)
9. **§4.2/Status — name the manifest as the discretion boundary** and tie its
   review to the manager's dispatch freeze. (R-9)
10. Renumber or drop the discontinuous section numbers.

Nothing in this list requires redesign, and none of it touches the mechanisms
that closed C-1 … C-16. Edits 1, 2 and 3 are the set I would want landed before
an implementer starts; 4 … 10 can land in the same pass.

## 12. Non-authorization

This addendum is an independent contract-preflight disposition. It does not
authorize implementation, adopt I1 or I5, accept or reject any product
candidate, change any recorded verdict, index, or ledger, authorize the
Cargo/KGI live comparison or its canonical ports, fix any checklist length,
field count, or repair-size threshold, or modify the contract. The manager owns
the dispatch decision. The commit carrying this addendum adds exactly one path:

```text
docs/development/ai/swarm-results/cfr168-review-recurrence-v2-contract-preflight-claude-20260731/rereview.md
```

CONTRACT_VERDICT: REQUEST_CHANGES
