# CFR-168 Development-Flow KGI/KPI — Round-4 Rereview under AIA-047

Status: Prepared, not launched — launch is blocked on two independent
preconditions stated in "Launch preconditions" below. No reviewer may be
dispatched against this packet until both clear. This document is frozen
pre-review evidence, not a dispatch.

Provider/model: Claude Opus 5 (`claude-opus-5`), fresh session — **not** the
implementer session `34cdf037-2326-480e-8353-621713838feb`.

Role: exact independent report-only reviewer. Review generation 4.

Pilot: AIA-047 successor review recurrence
(`docs/development/ai/review-recurrence/README.md`). This is the KGI/KPI arm
of the two mandatory pilot runs; the Cargo arm is
`cfr168-source-keyed-cargo-third-closure-rereview-with-recurrence-20260731.md`.

## Objective

Decide, independently and report-only, whether candidate
`9ca01d9a457aae05adc7f7881b66ce066aacaff6` discharges the frozen A1–A8
closure contracts below, and separately whether the repair's own delta —
including the one persisted-shape surface the pre-repair packet told the
implementer to stop for — is sound.

Prior-obligation closure and repair-created delta are two distinct passes.
Neither substitutes for the other.

Two coverage rules bind this review and neither is a length target:

1. **No finding quota.** Coverage is judged by obligation-and-surface
   saturation: every frozen obligation row, every Git-derived changed path,
   every actual changed surface, every required production route, and every
   negative control listed here needs a disposition, and unavailable evidence
   must be named. Counts are diagnostic outputs. Do not restate one effect
   under several observation rows.
2. **No frozen-enumerated corpus.** This lineage's named root cause is
   treating a list of known counterexamples as the invariant. The candidate
   adds 16 A-row tests and six probes (`P24`–`P29`). Passing all of them is
   evidence about the implemented contract and **is not** closure evidence.
   Every disposition in this review must rest on a fixture, probe, or held-out
   shape the reviewer built, run against the predecessor tree first. A closure
   row whose only evidence is one of the candidate's own 16 tests is
   `UNREVIEWED`, not `CLOSED`.

## Immutable subject — recompute every value before analysis

| Field | Exact value |
|---|---|
| review base (candidate parent, = generation-3 reviewed candidate) | `753c8db103579d021795d3fe6a97c5d45d4a111a` |
| base tree | `213e16a033aeb2e8eb3599be4ffe725b1d7ad344` |
| candidate | `9ca01d9a457aae05adc7f7881b66ce066aacaff6` |
| candidate tree | `65959ae20381a2096f1f56d46ec406d4681e4931` |
| additive commits in `base..candidate` | 1 |
| merge commits in `base..candidate` | 0 |
| canonical binary patch SHA-256 | `0051161004a6c2763d9f997371c8defba3ce0749b2773fb4956c0ec3a0b1e7c0` |
| numstat | 1,123 inserted, 140 deleted, 0 binary paths |
| shortstat | `12 files changed, 1123 insertions(+), 140 deletions(-)` |
| changed paths | 12, enumerated below |
| generation-2 rejected predecessor | `0944bf3deba9e753132b3f71716b5c009a8fc30f` (ancestor of the candidate) |
| generation-3 repair commit | `d003a666bcbf7cbec804efcf4e321f0a0ae8e1dd` |
| defined test methods at candidate | 145 |

The canonical patch digest is over exactly:

```bash
git diff --no-ext-diff --no-textconv --no-renames --binary --full-index \
  --src-prefix=a/ --dst-prefix=b/ \
  753c8db103579d021795d3fe6a97c5d45d4a111a \
  9ca01d9a457aae05adc7f7881b66ce066aacaff6 -- | sha256sum
```

Any other argv — in particular omitting `--full-index` — yields a different
digest and is not the bound identity.

**The base is pinned, deliberately.** The AIA-047 gate's own independent
review (finding F-1, `df156088`) showed that `subject.base_revision` is
otherwise unconstrained: moving the base forward past a risky commit shrinks
the Git changed-path set, and the coverage set-equality check then confirms
"exact" coverage of the narrowed set. The base here **must** be
`753c8db103579d021795d3fe6a97c5d45d4a111a`, which is both the candidate's sole
parent and the exact revision generation 3 reviewed. The reviewer must verify
both facts (`git rev-parse 9ca01d9^` and the generation-3 report's subject
line) and report any divergence as blocking. Until F-1 is repaired, the tool
cannot enforce this; the packet does.

### Prior generations, immutably bound

| Generation | Review commit | Report path | Bytes / SHA-256 | Decision |
|---|---|---|---|---|
| 2 | `5760c984fb0aaee27b42dc48aa89fe1bacb0d590` | `docs/development/ai/swarm-results/development-flow-kgi-kpi-r2-rereview-claude-fallback-20260731/review.md` | SHA-256 `54a4244a8e563b48b2f086a51433ea858996c08b0104c0309f2c8445ce6706b2` (recomputed here) | `REQUEST_CHANGES` |
| 3 | `12f9d0b5c30ce2695ec6c96e5876703efd7b0d90` (sole parent `753c8db`) | `docs/development/ai/swarm-results/development-flow-kgi-kpi-r3-rereview-claude-20260731/review.md` | 63,420 bytes / `23b59e01825e3f66d2d38b754348e3d51457f3c465def578ac99ce52729a204f` | `REQUEST_CHANGES` |

Do not edit, amend, or replace either report. Generation 3 is 814 lines and
must be read in full before any disposition; its §3.2, §4, §11 and §14 are the
source of the A-rows below.

### Pre-repair packet, content-addressed before implementation

| Field | Exact value |
|---|---|
| repair packet path | `docs/development/ai/task-packets/cfr168-development-flow-kgi-kpi-r4-repair-20260731.md` |
| repair packet commit | `a22fad0698b1623d8898c47699e7ba4e61294b5f` |
| repair packet SHA-256 | `bbd6ef472903079238f24ad92bf567a9143fdbd1874975d5364ddd4eea8964ad` (11,676 bytes) |
| repair packet commit time | `2026-07-31T12:56:00+00:00` |
| candidate commit time | `2026-07-31T13:14:37+00:00` |
| `merge-base --is-ancestor a22fad06 9ca01d9a` | **false** — merge base is `9dd0da4f2ae55742ee51c1671aab6308994de24c` |

The repair packet is on the canonical integration lane, not in the candidate's
ancestry, so the final record must declare
`repair_packet_timing: "dispatch_event_attested"`. Declaring
`ancestor_of_candidate` here is false.

| Field | Exact value |
|---|---|
| dispatch event ref | `a22fad0698b1623d8898c47699e7ba4e61294b5f:docs/development/ai/swarm-batches/cfr168-recovery-execution-ledger-20260730.md`, section `## 2026-07-31 12:51 UTC KGI/KPI round-3 review completed without corpus retuning` |
| dispatch event SHA-256 | `78f879d5ead1d171d8c8897c938df410f0dcdb4d0271045cb8259cb588cf5785` (168,695 bytes) |

That section states the repair packet's SHA-256, the A1–A8 owner matrix, the
structural stop condition, the preservation-replay requirement, and the
explicit routing of `R1c-c` and the seal-topology policy — in a commit whose
committer time precedes the candidate's by 1,117 seconds.

**State the trust limit precisely, do not overstate it.** AIA-047 gate finding
F-3 established that `dispatch_event_attested` currently performs *no* timing
check: the tool checks only that the reference and digest strings are
non-empty. Neither is resolved, re-hashed, or ordered. So the attestation
above is, as far as automated verification goes, an **unverified operator
assertion**. Its evidential weight comes from the reviewer doing by hand what
the tool does not: resolve
`git show a22fad06:.../cfr168-recovery-execution-ledger-20260730.md`, re-hash
it, confirm the section is present, and confirm the packet digest it names
matches `bbd6ef47…`. Report the outcome. If the section is absent, altered, or
names a different digest, the pre-repair chronology fails and the record
cannot be `accept`.

### Round-4 result report, as committed

| Field | Exact value |
|---|---|
| path | `docs/development/ai/swarm-batches/development-flow-kgi-kpi-zero-residual-repair-r4-20260731.md` |
| SHA-256 at candidate | `b233dd33757b3c5279c93bc921e29e3b82894c20d408945ad5b257e6abe80877` (31,689 bytes, 303 lines) |

Treat every figure, code name, count, and duration in it as an unverified
claim. It correctly disclaims acceptance; that disclaimer is not evidence.

### File digests at the candidate

```text
28dc2767cb64b4507a61ff5f30c6fe7227d0f690a3b06ab5564b22c05eef3acd  docs/development/ai/development-flow-kgi-kpi.md
bcb7b25d41d6789ba6fb809a46b24e5c2b15c137e94bbacfa9cdd89b8e7749fb  docs/development/ai/flow-metrics/README.md
3abc9a435a544c5f79b5119253f3b4c613464514bf5eb129c3aa4c34b4706d45  docs/development/ai/flow-metrics/cfr168-recovery-20260730.v1.json
3ac1213dff58e59bb9fc946823a3e3a574bba59809ed39061f0f6ca8f8a45c38  docs/development/ai/schemas/README.md
9f9ab4181b486a523823f499bc4ba184e9935c76b8ebdeb7f5a65ca41be72e06  docs/development/ai/schemas/development-flow-record.v1.schema.json
67dceef1e687f4304345cbd098e8b87bde875458959bef5f264a72ff78fa7a98  docs/development/ai/swarm-batches/development-flow-kgi-kpi-zero-residual-repair-r3-20260731.md
b233dd33757b3c5279c93bc921e29e3b82894c20d408945ad5b257e6abe80877  docs/development/ai/swarm-batches/development-flow-kgi-kpi-zero-residual-repair-r4-20260731.md
4525db9fdc1a57fa7dbce293d9310038a668700b3e37ce3bc86e3e58681be8a1  docs/development/history.md
2a1a45e265ad3c27b99593fcd6d59085ffe3c92e08a9a4e91c5552ee7bb6d21b  docs/development/project-management.md
f013f2bf6832d76285fe4bfe36da1cf5f8ec416aede84e022f8a17dfb584f8c1  docs/development/tools/ai_contracts.py
a39e59fdd37b76cc83255392c43e822fd9ba1fe9a7e6250958adafbae5915bb1  docs/development/tools/check_ai_contracts.py
8578cd4592470c297353275d27d6b07fc2ac67439b149cfda1d99db0558ad95f  docs/development/tools/test_ai_tools.py
```

## Scope

### Frozen prior obligation row inventory

Twenty rows, each with a stable ID, a distinct effect episode, and an origin
generation strictly below 4. Reuse these IDs verbatim as
`prior_findings[].id` and `effect_episode_id`; the coverage tables below key
off them. The five substantive published-claim rows of generation-3 §11 ride
with the code rows whose behaviour they publish (`A1-SUBJECT`, `A2-DENOM`,
`A3-D1`/`A3-D5`, `A4-STAGE`, `A5-SCOPE`) and are part of those closure
contracts rather than separate episodes — generation 3 itself recorded them
that way. Their omission as separate rows is therefore not a coverage gap;
omitting the claim half of any of those five contracts **is**.

| ID | Effect episode | Origin gen | Source rows | Blocking at gen 3 |
|---|---|---:|---|---|
| `A1-SUBJECT` | `EFF-KGI004-DECLARABLE-BY-NAME` | 2 | §3 R1c-a, §6.1 F1/F11/F-S9/F-S13, §11 row 1, §12 R1c-a | yes (`MUST`) |
| `A1-PREREQ` | `EFF-MEASURED-ZERO-WITHOUT-RUNTIME-VERSION` | 3 | §13 item 2, §14 KGI/guardrail owner | no |
| `A2-DENOM` | `EFF-COMPLETENESS-DENOMINATOR-AUTHOR-CHOSEN` | 2 | §3 R1c-b, §11 row 2, §12 R1c-b | no (`SHOULD`) |
| `A3-D1` | `EFF-ARTIFACT-SPLIT-BY-COMMIT-PADDING` | 2 | §3 R6 D1, §11 row 3, §12 R6-D1 | no (`SHOULD`) |
| `A3-D5` | `EFF-ARTIFACT-SPLIT-BY-COMMITFREE-PADDING` | 2 | §3 R6 D5, §12 R6-D5 | no (`SHOULD`) |
| `A3-N1` | `EFF-TWO-DELIVERABLES-ONE-COMMIT-REJECTED` | 3 | §4 N1, §12 N1 | no (`SHOULD`) |
| `A4-STAGE` | `EFF-POLICY-CITATION-CLOSES-LIFECYCLE-STAGE` | 2 | §3 L, §11 row 4, §12 L | no (`SHOULD`) |
| `A5-SCOPE` | `EFF-INSTANT-BOUND-STATUS-AND-ROUTE-CONDITIONAL` | 3 | §3 R5′, §6.1 F6/F-S2, §11 row 5, §12 R5′ | no (`SHOULD`) |
| `A5-ECHO` | `EFF-INSTANT-ECHO-ABSENT-IN-JSON` | 3 | §2.2, §3 R5′, §11 row 5 | no |
| `A6-N2` | `EFF-STRUCTURAL-FLOOD-SUPPRESSES-SEMANTIC-LAYER` | 3 | §4 N2, §5.6, §12 N2 | no (`SHOULD`) |
| `A6-N4` | `EFF-COHORT-COST-TOTAL-CODE-UNREACHABLE` | 3 | §4 N4, §12 N4 | no (`COULD`) |
| `A7-C1` | `EFF-SEAL-DOCSTRING-OVERCLAIMS-BLOB-COMPARISON` | 3 | §8, §11, §12 C1 | no |
| `A7-C2` | `EFF-BOUND-WORDING-512-VS-513` | 3 | §5.6, §11, §12 C2 | no |
| `A7-C3` | `EFF-SHALLOW-CLONE-SEAL-CLAIM-FALSE` | 3 | §5.4, §11, §12 C3 | no |
| `A7-C4` | `EFF-PACKET-DOC-COUNT-565-VS-566` | 3 | §2.2, §12 C4 | no |
| `A8-C5` | `EFF-FIXTURE-SHARES-PRODUCTION-DIGEST-HELPER` | 3 | §8, §12 C5 | no |
| `A8-C6` | `EFF-COHORT-POSITIVE-CASE-ROOT-NONE` | 2 | §8, §12 C6 | no |
| `R1C-C-TRUST` | `EFF-COUNTED-INTEGRATION-EVENTS-UNTRUSTED` | 3 | §3 R1c-c, §12 R1c-c | no — **routed** |
| `N3-MERGE-SEAL` | `EFF-SEAL-FAILS-ON-SIMPLIFIED-MERGE-INTRODUCTION` | 3 | §4 N3, §5.4 S6, §12 N3 | no — **routed** |
| `P1-RENAME-SHALLOW` | `EFF-SEAL-UNRESOLVED-ON-RENAME-OR-SHALLOW-CLONE` | 3 | §5.4 S7/S10, §12 P1 | no — **routed, pre-existing** |

### Frozen closure contracts A1–A8

These were frozen by the manager in the pre-repair packet at `a22fad06`
before implementation. The reviewer judges the candidate against these words.

#### A1 — terminal guardrails must not be discharged by a name

Rows `A1-SUBJECT`, `A1-PREREQ`.

| Field | Frozen content |
|---|---|
| acceptance criterion | A zero-target guardrail becomes a claimed zero only when the act that observed it and the row that claims it bind the same immutable repository object, and only when the fact the guardrail's question depends on exists in the record. Adding arbitrary event names, `measurement:` strings, or unrelated repository citations must not make `KGI-DEV-004` terminal. `MET-AI-075` must not be `measured` while `runtime_version` is missing. An honest positive case whose evidence independently identifies the metric subject must still pass. |
| authorized boundary | the existing `check_zero_target` branch, the existing `_reference_is_repository_verifiable` helper, `development-flow-kgi-kpi.md`, `flow-metrics/README.md`, `test_ai_tools.py`. |
| required live routes | production entry point (record route) with `KGI-DEV-004` set to `achieved`; the same through the directory route; the committed exemplar unchanged; the honest shared-object case; `MET-AI-075` with and without complete `runtime_version`; deletion of the shared object from the exemplar. |
| anticipated regressions | a frozen allowlist of known event IDs; a general classifier over metric semantics; refusing the honest exemplar; making `doc:` references unusable where they legitimately are the evidence; making `prefix:` subjects trusted without verification; a shared-object rule so strict that no record can ever discharge a guardrail. |
| forbidden shortcuts | an allowlist of legitimate audit event IDs; a hardcoded policy-document path exclusion; requiring the event's `stage` to match a per-metric stage as a classifier; a new trust system; closing by correcting the published sentence alone. |

#### A2 — required-evidence completeness needs a non-author-chosen denominator

Row `A2-DENOM`.

| Field | Frozen content |
|---|---|
| acceptance criterion | Completeness is the share of the record's own claims that owe evidence which a repository-verifiable subject discharges. Removing a required citation cannot raise the metric; adding an optional unverifiable reference cannot make an otherwise honest record fail a claim it never made. Either the denominator is derived from the record's actual claims/deliverables, or the metric and the `KGI-DEV-001`/`KGI-DEV-002` target language no longer claim complete required evidence. |
| authorized boundary | one private counting method replaced in place (`evidence_completeness_counts` → `required_evidence_completeness_counts`), the metric table row, the `flow-metrics/README.md` sentence. No new field, no new metric. |
| required live routes | the committed exemplar's derived value; the stripped-citation shape of generation-3 §3 R1c-b; an optional `lane:` reference appended to an unrelated event; one lane's verifiable citation removed; both KGI target statements. |
| anticipated regressions | a fixed required-evidence checklist; a denominator that counts claims the record does not make; making an honest record's value drop because it cites *more*; changing the exemplar's value without saying so. |
| forbidden shortcuts | enumerating a required-evidence checklist per metric; leaving the KGI target sentence stronger than the computed quantity; improving the metric by narrowing what counts as a claim. |

#### A3 — artifact subject must resist padding without collapsing deliverables

Rows `A3-D1`, `A3-D5`, `A3-N1`.

| Field | Frozen content |
|---|---|
| acceptance criterion | One property closes all four observed shapes: (1) the generation-2 lane-padding case remains rejected; (2) a commit-bearing twin padded with another declared `commit:` cannot mint a second subject; (3) a commit-free artifact padded with `event:` cannot mint a second subject; (4) two genuinely distinct deliverables in one commit remain valid. Identity derives from the deliverable, not its citation list. The central negative test must not build its expected digest with the production digest helper. |
| authorized boundary | `ai_contracts.py` (`artifact_subject_digest` and one new derivation), `development-flow-record.v1.schema.json`, the exemplar record, `test_ai_tools.py`, both policy docs. **A new persisted concept required a stop and a design blocker, not an implementation** — see the scope judgment below. |
| required live routes | all four shapes above; padding with `prefix:`, `lane:`, `measurement:`, `cohort:` and `artifact:` schemes as well, not only the two named; two artifacts in different lanes touching the same file; an artifact whose declared path no commit touched; the five committed exemplar subjects. |
| anticipated regressions | over-strictness in a new direction (two artifacts in different lanes rejected); a required schema field that invalidates any other existing record; a path set the author can declare freely, making identity author-chosen again; per-commit path granularity presented as per-deliverable granularity. |
| forbidden shortcuts | hashing artifact content instead of the deliverable; a schema version replacement; building expected digests through `artifact_subject_digest`; declaring shape 4 closed while two deliverables touching identical paths still collapse. |

#### A4 — lifecycle stages require evidence of the stage

Row `A4-STAGE`.

| Field | Frozen content |
|---|---|
| acceptance criterion | A closing stage is discharged by something the stage produced that cannot be rewritten, not by a pointer to the document describing the obligation. A `final` record whose five closing events cite only the policy document must raise a stage finding. Ordering and prior-stage constraints already closed stay closed. |
| authorized boundary | the existing `check_lifecycle` branch reusing the A1 helper; the finding message; the policy row; `test_ai_tools.py`. |
| required live routes | `final` record with policy-doc-only closing events; the same with `commit:` subjects; the sealed-final fixture; the four-`decision`-events case that must still raise `lifecycle-complete-before-prior-stage`; an `active` record (completion must still require `final`). |
| anticipated regressions | an allowlist of the five historical event IDs; breaking the already-closed ordering or prior-stage rules; refusing the honest sealed-final fixture; a message that still claims more than the code does. |
| forbidden shortcuts | excluding the policy document by hardcoded path; correcting the message without changing behaviour; treating any repository-verifiable reference as stage evidence. |

#### A5 — validation instant applies consistently

Rows `A5-SCOPE`, `A5-ECHO`.

| Field | Frozen content |
|---|---|
| acceptance criterion | The `captured_at` lower bound is a property of the instant, not of the record's status or route: it applies to `active`, `final`, and `superseded` records on both the record and directory routes. The effective instant is echoed in both human and JSON output. Existing valid-instant behaviour is preserved. |
| authorized boundary | move the existing check out of the status-conditional branch; add the same comparison to the existing directory function; one key on the existing JSON payload; three docs; `test_ai_tools.py`. |
| required live routes | `1970-01-01T00:00:00Z` against each of the three statuses on both routes; `captured_at` exactly (must pass); `captured_at − 1 ns` (must fire); the environment-variable form `CODEFIRE_FLOW_VALIDATION_INSTANT`; `--json` with and without an injected instant; a future instant (the overdue findings must still appear). |
| anticipated regressions | reading the wall clock when no instant is injected; a JSON key whose value is `null` presented as an echo; double-reporting the bound on the directory route; breaking the existing overdue findings. |
| forbidden shortcuts | claiming the control is automatic; echoing in one mode only; applying the bound to two statuses and calling it every status. |

#### A6 — semantic diagnostics survive structural floods

Rows `A6-N2`, `A6-N4`.

| Field | Frozen content |
|---|---|
| acceptance criterion | A flood of one defect cannot suppress the report of any other defect, and the total stays bounded. A payload exceeding the structural allowance still emits the terminal `finding-limit-reached` marker **and** at least one independently constructed decision-critical semantic finding. `cohort-cost-total-not-bound` is removed or made reachable with a negative control. No second independent limit implementation. |
| authorized boundary | one shared bound predicate and its list form inside `ai_contracts.py`; `schemas/README.md`; `development-flow-kgi-kpi.md`; `test_ai_tools.py`. |
| required live routes | 600, 2,000 and 4,000 malformed events, each with an independently constructed semantic falsification; a record breaking eight distinct rules many times each; a record breaking **one** rule legitimately more than the per-code allowance; the committed exemplar (must stay clean); the existing bound test. |
| anticipated regressions | **a per-code reserve that suppresses legitimate repeated findings of one code** — if 300 events each carry a real distinct instance of one defect, reporting 8 is a new under-report, and the published wording must say so; a total bound that no longer holds; a marker that no longer appears; folding `cohort-cost-total-not-bound` into another code in a way that loses a real distinction. |
| forbidden shortcuts | a separate semantic-layer counter; removing the dead code without checking whether the case it named is now unreportable; asserting the property only through the flood fixture the implementer wrote. |

#### A7 — published claims must match enforcement

Rows `A7-C1`, `A7-C2`, `A7-C3`, `A7-C4`, plus the claim halves of `A1-SUBJECT`,
`A2-DENOM`, `A3-D1`/`A3-D5`, `A4-STAGE`, `A5-SCOPE`.

| Field | Frozen content |
|---|---|
| acceptance criterion | Every published sentence about this contract is entailed by the code, and every measured count in the lineage's own reports is the count the tool prints. Only claims tied to A1–A6 and the exact precision mismatches generation 3 named may be corrected. No clone, rename, or merge guarantee is claimed. |
| authorized boundary | `development-flow-kgi-kpi.md`, `flow-metrics/README.md`, `schemas/README.md`, `project-management.md`, the `check_final_provenance` docstring, and the generation-3 result report's two count corrections. |
| required live routes | each corrected sentence read against the code path it describes; `check_project_docs.py` on the reviewed tree and on this tree (the counts differ because the report itself is a Markdown file — verify the arithmetic rather than the assertion); the seal docstring against what `check_final_provenance` actually compares. |
| anticipated regressions | correcting a sentence into a *different* overstatement; editing an immutable review report; a docstring that now describes an obligation no test covers; a count corrected in one place and stale in another. |
| forbidden shortcuts | prose-only closure for a row whose code half is still open; claiming a behavioural negative control for a prose change; asserting the corrected count without running the tool. |

#### A8 — closure evidence must be independent of the mechanism

Rows `A8-C5`, `A8-C6`.

| Field | Frozen content |
|---|---|
| acceptance criterion | Every A1–A7 negative control constructs its expected value independently of the production helper under test, runs against `753c8db` first with the failure recorded, rejects stably on the repair, has one honest positive case, makes the asserted axis move, and states the exact boundary at which it fires. The cohort positive case uses a real repository root. Deleting the protected production check changes a test result. |
| authorized boundary | `test_ai_tools.py` only. |
| required live routes | all 16 A-row tests executed against a read-only checkout of `753c8db`; both cohort test classes with a real root; the "delete the protected production input" case. |
| anticipated regressions | an independent oracle that silently reimplements the production bug; a positive case that passes on both trees being counted as a regression control; a real root that makes tests depend on this repository's mutable state. |
| forbidden shortcuts | reusing `artifact_subject_digest` in a fixture; `root=None` in the central positive case; counting the 16 tests as the invariant. |

### Predecessor failure inventory — re-derive from the `753c8db` tree itself

Audit obligation I1: predecessor behaviour must come from the predecessor
tree, not from the candidate's tests or the result report. Materialise
`753c8db` as its own checkout, import `ai_contracts` **from that tree**, and
reproduce each row below with a fixture the reviewer built. A row that cannot
be reproduced is `UNAVAILABLE` with a reason — and if a row fails to reproduce
because the described defect never existed, that is a blocking finding against
the repair's premise.

| Row | Claimed pre-edit observation on `753c8db` |
|---|---|
| `A1-SUBJECT` | `MET-AI-075` + `MET-FLOW-037` `measured 0`, two `measurement:` strings on `event:mr-list-focused-first-pass-failure-20260730-091908`, `KGI-DEV-004` → `achieved`: **zero findings** |
| `A1-PREREQ` | `MET-AI-075` accepted as `measured` with no `runtime_version` anywhere |
| `A2-DENOM` | strip every non-`commit:`/`doc:` reference, declare `MET-FLOW-027 = 12/12 = 1.0`: **accepted, no completeness finding** |
| `A3-D1` | commit-padded twin: **no artifact finding** |
| `A3-D5` | commit-free twin padded with `event:`: **no artifact finding** |
| `A3-N1` | two honest deliverables in one commit: **`duplicate-artifact-subject`** (false rejection) |
| `A4-STAGE` | `final` record, five closing events citing only the policy doc: generation 3 reports no `lifecycle-stage-evidence-unverifiable` and no `lifecycle-complete-missing-stage`; the result report records `['missing-final-lifecycle-event']` only. **These two accounts use different code names — reconcile them and state which is the true observation.** |
| `A5-SCOPE` | backdated instant: `active` fires, `final` and `superseded` do not, directory route does not |
| `A5-ECHO` | `--json` payload has no instant key |
| `A6-N2` | 600 malformed events + `MET-FLOW-028` declared 3: generation 3 §4 reports "3 distinct codes, 513 findings, `throughput-not-derived` ABSENT"; the result report reports 513 with the marker present. **Reconcile the finding count against the distinct-code count.** |
| `A6-N4` | `cohort-cost-total-not-bound` unreachable in every constructible case, and absent from the frozen 55 |
| `A7-C1` | `check_final_provenance` docstring claims a working-tree-versus-sealing-commit comparison; the code compares against `HEAD` |
| `A7-C2` | `schemas/README.md` says "at 512 findings"; worst case observed 513 |
| `A7-C3` | "so the seal survives a clone or a `gc`" false for `--depth 1` |
| `A7-C4` | the generation-3 packet says 565 Markdown files; the tool printed 566 |
| `A8-C5` | `_pad_artifact_subject` builds its fixture with production `artifact_subject_digest` |
| `A8-C6` | cohort acceptance test runs `root=None`, leaving repository verification inert |

### Candidate positives to re-verify independently

A repair that closes a gap by refusing honest records has not closed it. Each
of these must be re-verified with the reviewer's own fixture, not the
candidate's:

- the committed exemplar record validates with **zero findings** on both the
  record route and the directory route;
- `MET-FLOW-037` bound to the real recovery event, both citing the same
  `commit:` subject, produces no guardrail finding;
- `MET-AI-075` becomes admissible once every agent lane declares
  `runtime_version`;
- the declared completeness pair is accepted when computed independently, and
  an appended optional `lane:` reference does not break it;
- two artifacts in one commit — a source patch delivering `codefire` and a
  test patch delivering its focused module — are two valid artifacts;
- all five committed exemplar `subject_sha256` values reproduce under the
  reviewer's own canonical-JSON encoding over the declared path sets;
- five closing events citing a real `commit:` close their stages; the
  sealed-final fixture stays clean;
- `--validation-instant` exactly equal to `captured_at` is admissible, and
  `--json` echoes it;
- an honest held-out cohort with a real repository root validates clean;
- the frozen `FLOW_SEMANTIC_MUTATIONS` (55) and `FLOW_STRUCTURAL_CONTROLS`
  (15) tables are byte-identical to `753c8db`, and all 13 generation-2
  executable residuals still reject.

The frozen-table integrity check is the single most important preservation
check in this lineage. If either table was weakened, reworded, or shortened,
that is blocking regardless of everything else.

### Independent negative controls the reviewer must run

Run each in a throwaway checkout or temporary repository outside this tree,
record the exact code or failure text, and restore identity afterwards.
`survived` on a `must fire` row is a blocking finding. **None of these may be
satisfied by citing one of the candidate's 16 A-row tests.**

| ID | Held-out control | Required outcome |
|---|---|---|
| `NC-A1-NAME` | reproduce the generation-3 declaration shape on the candidate | must fire (`zero-target-audit-subject-unshared`) |
| `NC-A1-PILE` | add more `measurement:` names and more `doc:` citations on top | refusal must persist |
| `NC-A1-DOC` | make the *only* shared subject a `doc:` path | must fire — a mutable pointer is not the shared object |
| `NC-A1-PREFIX` | make the shared subject an **unverified** `prefix:` reference | must fire; a `prefix:` scheme must not be trusted without verification |
| `NC-A1-HONEST` | honest shared `commit:` subject | must not fire |
| `NC-A1-RT` | `MET-AI-075` `measured` with one agent lane missing `runtime_version` | must fire (`zero-target-prerequisite-unmet`) |
| `NC-A2-DELETE` | raise completeness by deleting citations | must fire |
| `NC-A2-ADD` | append an optional unverifiable reference to an honest record | must **not** fire |
| `NC-A2-CLAIMLESS` | a record whose claims owe no evidence at all | denominator behaviour must be stated, not silently zero-divided |
| `NC-A3-PAD-ALL` | pad a twin with each of `commit:`, `lane:`, `event:`, `prefix:`, `measurement:`, `cohort:`, `artifact:` in turn — **held-out beyond the three shapes the candidate tests** | none may mint a second subject |
| `NC-A3-CROSS-LANE` | two artifacts in **different** lanes declaring the same path | must not fire — the lane scoping must not become a new false positive |
| `NC-A3-FAKE-PATH` | declare a path no bound commit touched | must fire (`artifact-deliverable-path-unbound`) |
| `NC-A3-UNSAFE-PATH` | declare `../escape` or an absolute path | must fire (`artifact-deliverable-path-unsafe`) |
| `NC-A3-SAME-PATHS` | two deliverables in one commit touching **exactly the same** paths | expected to still collapse — record it as the candidate's declared limitation, not as closure |
| `NC-A4-DOC` | policy-doc-only closing events on a `final` record | must fire |
| `NC-A4-COMMIT` | the same events citing a real commit | must not fire |
| `NC-A4-ORDER` | four `decision` events only | must still fire `lifecycle-complete-before-prior-stage` |
| `NC-A5-MATRIX` | backdated instant × {`active`, `final`, `superseded`} × {record, directory} | all six must fire |
| `NC-A5-EXACT` | instant exactly `captured_at`, then `captured_at − 1 ns` | pass, then fire |
| `NC-A5-ENV` | the same through `CODEFIRE_FLOW_VALIDATION_INSTANT` | identical behaviour |
| `NC-A5-JSON` | `--json` with and without an instant | key present in both; `null` must not be presented as an echo |
| `NC-A6-FLOOD` | 600 / 2,000 / 4,000 malformed events, each with the reviewer's own semantic falsification | marker **and** the semantic code present every time |
| `NC-A6-EIGHT` | eight distinct rules broken many times each | all eight reported |
| `NC-A6-LEGIT` | **one** code legitimately instantiated far more than the per-code allowance | quantify the new under-report and check the published wording admits it |
| `NC-A6-N4` | attempt to reach the removed `cohort-cost-total-not-bound` case | the folded code must name both bindings, or the distinction is lost |
| `NC-A7-DOCSTRING` | read `check_final_provenance` against its corrected docstring | entailed, or `OPEN` |
| `NC-A7-COUNT` | run `check_project_docs.py` on both trees | the arithmetic must explain both numbers |
| `NC-A8-PRED` | all 16 A-row tests against a read-only `753c8db` checkout | every test that claims a regression must fail there; a test that passes on both trees is a forward guard, not a regression control |
| `NC-A8-DELETE` | delete the protected production check | a test result must change |
| `NC-PRESERVE` | replay 55 semantic + 15 structural + all 13 generation-2 residuals | all pass, tables byte-identical |
| `NC-SCHEMA-BREAK` | validate any other existing flow record against the new required `deliverable_paths` | state the blast radius; only `cfr168-recovery-20260730.v1.json` exists in this tree, so confirm that and say so |

### Exact Git-derived changed-path coverage

The final record's `changed_path_coverage` must equal this set exactly. Every
row's `required_by` must name IDs from the frozen inventory above.

| Exact path | Required by | Insert/delete | Mandatory review focus |
|---|---|---|---|
| `docs/development/tools/ai_contracts.py` | `A1-SUBJECT`, `A1-PREREQ`, `A2-DENOM`, `A3-D1`, `A3-D5`, `A3-N1`, `A4-STAGE`, `A5-SCOPE`, `A6-N2`, `A6-N4`, `A7-C1` | 243 / 89 | every new symbol and every removed branch; that no already-closed rule was relaxed; that the per-code reserve is one predicate, not a second limiter; that `_immutable_subjects` cannot be satisfied by a mutable pointer |
| `docs/development/tools/test_ai_tools.py` | `A1-SUBJECT`, `A1-PREREQ`, `A2-DENOM`, `A3-D1`, `A3-D5`, `A3-N1`, `A4-STAGE`, `A5-SCOPE`, `A5-ECHO`, `A6-N2`, `A6-N4`, `A8-C5`, `A8-C6` | 453 / 5 | 16 A-row tests + `P24`–`P29`; the independent oracle really is independent; frozen 55/15 byte-identical; no test asserts only a symbol name |
| `docs/development/tools/check_ai_contracts.py` | `A5-ECHO` | 8 / 1 | the payload key is present in both output modes and its `null` case is honest |
| `docs/development/ai/schemas/development-flow-record.v1.schema.json` | `A3-D1`, `A3-D5`, `A3-N1` | 8 / 0 | `deliverable_paths` required, `minItems 1`, `maxItems 256`, unique; the compatibility consequence of a newly required field |
| `docs/development/ai/flow-metrics/cfr168-recovery-20260730.v1.json` | `A3-D1`, `A3-D5`, `A3-N1`, `A2-DENOM` | 25 / 6 | five path sets were **read from the bound commits**, not authored; five recomputed subjects; the added limitation; no metric value silently improved |
| `docs/development/ai/development-flow-kgi-kpi.md` | `A1-SUBJECT`, `A2-DENOM`, `A3-D1`, `A4-STAGE`, `A5-SCOPE`, `A6-N2`, `A7-C3` | 50 / 25 | each corrected sentence entailed by the code; no new overstatement |
| `docs/development/ai/flow-metrics/README.md` | `A1-SUBJECT`, `A2-DENOM`, `A3-D1`, `A5-SCOPE` | 16 / 9 | the "recomputed, never declared" claim now matches what is computed |
| `docs/development/ai/schemas/README.md` | `A7-C2`, `A6-N2` | 1 / 1 | per-code and total bound wording exact |
| `docs/development/project-management.md` | `A5-SCOPE` | 4 / 3 | freshness wording; the Phase-2 scorecard row still accurate |
| `docs/development/history.md` | `A7-C4` | 1 / 0 | the round-4 entry states no unearned closure and no count matching no tree. This is the weakest attribution in the table: the entry is ledger hygiene rather than a named obligation, and `A7-C4` is its closest owner only because A7 requires every published count in this lineage's own records to be the count the tool prints. If the reviewer judges the attribution wrong, say so — an unattributable changed path is a communication finding, not a licence to omit the row |
| `docs/development/ai/swarm-batches/development-flow-kgi-kpi-zero-residual-repair-r3-20260731.md` | `A7-C4` | 11 / 1 | this is the **implementer's own prior result report**, not an independent review — confirm no independent review report was edited; corrections dated and self-identifying |
| `docs/development/ai/swarm-batches/development-flow-kgi-kpi-zero-residual-repair-r4-20260731.md` | all A-rows | 303 / 0 | every claim re-derived; the non-acceptance statement present; the rejected-claims table honest |

No path outside the pre-repair packet's allowed write set may appear. Confirm
that mechanically. In particular no independent review report under
`docs/development/ai/swarm-results/` is touched.

### Actual changed surfaces — planned versus unplanned

| Surface | Required by | Planned? | Kind | Change | Review focus |
|---|---|---|---|---|---|
| `ai_contracts.py`: `deliverable_paths` on the artifact object; `artifact_subject_digest(deliverable_paths)` signature change; `development-flow-record.v1.schema.json` required field | `A3-D1` | **no — see the scope judgment** | persisted_shape | added | whether any rule over references alone could separate A3 shapes 2 and 4; whether the path set is evidence or a declaration; blast radius of a newly required field |
| `ai_contracts.py`: `git_commit_paths` | `A3-N1` | yes — packet §A3 "derive identity from the deliverable" | private_symbol | added | that declared paths are checked against what the bound commits touched; behaviour on a commit that is unreachable or absent |
| `ai_contracts.py`: `IMMUTABLE_EVIDENCE_SCHEMES`, `_immutable_subjects` | `A1-SUBJECT` | yes — packet §A1 | private_symbol | added | `doc:` excluded by principle, not by path; unverified `prefix:` not trusted |
| `ai_contracts.py`: `ZERO_TARGET_PREREQUISITES`, `_runtime_versions_complete` | `A1-PREREQ` | yes — packet §A1 "require the missing runtime-version fact" | private_symbol | added | a two-entry mapping is a registry, not a classifier; what happens for the other ten guardrails |
| `ai_contracts.py`: `required_evidence_completeness_counts` replacing `evidence_completeness_counts` | `A2-DENOM` | yes — packet §A2 | private_symbol | modified | the denominator is the record's claims, not its citations; deletion cannot raise it |
| `ai_contracts.py`: `finding_admitted`, `cap_findings`, `MAX_FINDINGS_PER_CODE = 8` | `A6-N2` | yes — packet §A6 "reserve diagnostic capacity … do not add a second independent limit" | private_symbol | added | one predicate for both layers; the per-code under-report quantified |
| `ai_contracts.py`: `cohort-cost-total-not-bound` removed, folded into `cross-provider-total-not-summed` | `A6-N4` | yes — packet §A6 | classifier | deleted | the folded message names both bindings; no real distinction lost |
| `ai_contracts.py`: new codes `zero-target-audit-subject-unshared`, `zero-target-prerequisite-unmet`, `artifact-deliverable-path-unbound`, `artifact-deliverable-path-unsafe` | `A1-SUBJECT`, `A1-PREREQ`, `A3-N1` | yes | classifier | added | each is reachable, distinct, and covered by a held-out control; none is in the frozen 55, so the replay does not cover them |
| `ai_contracts.py`: instant bound moved into `check_record_window` and added to the directory function | `A5-SCOPE` | yes — packet §A5 | private_symbol | modified | six-cell matrix; no double reporting |
| `check_ai_contracts.py`: `flow_validation_instant` payload key | `A5-ECHO` | yes — packet §A5 "echo the effective instant in both human and JSON output" | public_api | added | a CLI JSON payload is a consumed interface; `null` must not read as an echo |
| `ai_contracts.py`: `check_final_provenance` docstring | `A7-C1` | yes — packet §A7 | claim | modified | entailed by an existing obligation and its test |
| `test_ai_tools.py`: `independent_artifact_subject`, `independent_completeness`, 16 A-row tests, `P24`–`P29`, cohort classes moved to a real root | `A8-C5`, `A8-C6` | yes — packet §A8 | test_or_oracle | added/modified | independence is real; each claimed regression fails on `753c8db` |
| exemplar record: five `deliverable_paths` sets, five recomputed subjects, one added limitation | `A3-D1` | partly — the recomputation is entailed by A3; **the added limitation text is a new claim** | claim | modified | path sets read from commits; the limitation is true and not a substitute for closure |
| `…-repair-r3-20260731.md`: 565→566 and the demotion-count correction with a dated note | `A7-C4` | yes — packet §A7 | claim | modified | it is the implementer's own report; the correction is dated and does not rewrite history silently |

Two surfaces are declared **unplanned or partly unplanned**: the
`deliverable_paths` persisted shape, and the exemplar's new limitation
sentence. The record's `scope_expansion` must reflect the manager judgment
below; a fourth surface the reviewer finds unrecorded here is itself a
reportable communication finding.

### Manager scope judgment on `deliverable_paths` — explicitly not automatic closure

The pre-repair packet's structural boundary said: *"If closing A1–A8 requires
a new public API, new persisted schema concept, new state machine, new general
classifier, or a second decision engine, stop and return a design blocker
instead of implementing it."* The implementer did not stop. It recorded the
decision candidly, argued the field is an *attribute of an existing concept*
rather than a new concept, and wrote: *"A reviewer who reads 'new persisted
schema concept' more strictly than 'attribute of an existing concept' should
treat this one field as the design blocker; everything else in this repair is
independent of it."*

This is a manager scope judgment, and this packet makes it explicitly rather
than letting the closure of A3 carry it silently. The reviewer must not treat
A3 as closed on the strength of the field working. Required disposition, in
this order:

1. **Test the necessity claim.** Is it true that no rule over references alone
   can accept two honest deliverables in one commit while refusing a padded
   twin? Try at least: deriving the touched-path set from the bound commits
   *without persisting it*; keying identity on `(commit set, kind)`; keying on
   a declared deliverable identifier that is not a path. If any of these
   achieves the property, the stop condition was violated and the correct
   record is `scope_expansion: "blocked"`.
2. **Test the "attribute, not concept" claim.** A newly *required* field
   changes the validity of every existing record. Enumerate every record in
   the tree (there is exactly one: `cfr168-recovery-20260730.v1.json`) and
   state whether any external consumer or schema version boundary is crossed.
3. **Test whether the field is evidence or a declaration.** If a path set can
   be declared without a bound commit touching those paths, identity is
   author-chosen again and A3 is not closed.

Manager pre-decision, conditional and stated in advance: if all three tests
pass, the field is admitted as `scope_expansion: "manager_approved"` with
authority "this packet, section 'Manager scope judgment on
`deliverable_paths`', manager `t.asakura`". If test 1 or test 3 fails, the
record is `scope_expansion: "blocked"`, which cannot be integrated, and A3's
rows are `OPEN`. If only test 2 raises a compatibility concern, the field may
remain `manager_approved` with the concern routed. The reviewer reports the
three test results; the manager owns the resulting label.

### Routed, not closed

Three rows are routed. Routing is never closure, never satisfies a
`prior_findings` closure contract, and is valid only as
`disposition: "routed_open"` with `blocking: false` and all three routing
fields populated. Each is recorded in `prior_findings`, not `new_findings`,
because each is a bound obligation the repair narrowed without closing —
`bound_finding_not_closed` in `new_findings` is rejected by the gate.

| Row | Owner | Release condition | Route authority |
|---|---|---|---|
| `R1C-C-TRUST` — an authored `integration` event is bound to an auditable structure but carries no gate, commit, or trusted-signal requirement; a shared immutable object proves neither topical relevance nor that the counted integration occurred | event/telemetry foundation lane | automatic event provenance exists, so an event's occurrence is not an author's assertion; required before any claim that `MET-FLOW-028` measures *trusted* integration | pre-repair packet `a22fad06` §"Explicitly Routed, Not Hidden"; reaffirmed by this packet, manager `t.asakura` |
| `N3-MERGE-SEAL` — an honest seal fails when identical record content is introduced on two branches and merged and the seal names the simplified-away side commit | record-lifecycle owner | one explicit Git history-support policy covering merge simplification, rename following, and whether full history is required | same |
| `P1-RENAME-SHALLOW` — relocating a sealed record, or validating in a `--depth 1` clone, makes an honest seal `final-record-provenance-unresolved`; **pre-existing, reproduces identically on `0944bf3`** | record-lifecycle owner | same policy decision as `N3-MERGE-SEAL` | same |

The reviewer may verify the routing and must report it if any of the three was
in fact silently changed by this candidate — their absence from the write set
is not a closure claim, and a change to them without an owner decision is a
scope violation. `P1-RENAME-SHALLOW` must additionally be confirmed
pre-existing at `0944bf3`; if it is not, it becomes candidate-attributed.

The candidate additionally declares a residual of the new mechanism: two
deliverables touching exactly the same paths inside one commit still collapse
to one subject. That is a candidate-created observation, not a routed prior
obligation. It belongs in `new_findings` with
`attribution: "created_by_candidate"` and
`causal_classes` including `repair_introduced_regression`; for the record to
be `accept` it needs `disposition: "routed_open"`, `blocking: false`, and an
owner/release/authority the manager assigns after the reviewer reports it.
The manager does not pre-authorize routes for findings not yet observed.

### Causal and evidence-state rules

Use only the schema enum. `mixed` is not a value. Multiple classes may apply.
Do not generalize this episode into a project-wide dominant cause — generation
3's own conclusion was that interaction, not one cause, dominates.

| Row | Frozen classification | Rule |
|---|---|---|
| `A1-SUBJECT` | `bound_finding_not_closed` + `oracle_gap`; the false published sentence is `repair_introduced_regression` at generation 3 | Generation 2 required subject-bound guardrails; generation 3 bound the name only. Not `initial_review_scope_miss` — generation 2 named it. |
| `A1-PREREQ` | `bound_finding_not_closed` (waived-then-consequential) | Generation 3 §13 item 2 waived it and generation 3 §14 made it required. |
| `A2-DENOM` | `bound_finding_not_closed` + `repair_introduced_regression` | Declarability inherited; the omission incentive was created by generation 3's reference-share derivation. |
| `A3-D1`, `A3-D5` | `bound_finding_not_closed` | Generation 3 closed only the non-commit shape. |
| `A3-N1` | `repair_introduced_regression` | Created by generation 3; a false rejection of an honest record. |
| `A4-STAGE` | `bound_finding_not_closed` + `oracle_gap`; the contradicting message is `repair_introduced_regression` | A `doc:` reference satisfied the very check whose message denied it. |
| `A5-SCOPE`, `A5-ECHO` | `initial_review_scope_miss` + `repair_introduced_regression` | Both the check and the over-broad claim are generation-3 additions. |
| `A6-N2`, `A6-N4` | `repair_introduced_regression` | The layer-ordered early return and the dead code are generation-3 artifacts. |
| `A7-C1`–`A7-C4` | `communication_ambiguity` + `repair_introduced_regression` | Claim/precision defects introduced at generation 3. `A7-C4` is documentation-only. |
| `A8-C5` | `oracle_gap` + `repair_introduced_regression` | A fixture built with the helper it validates. |
| `A8-C6` | `oracle_gap` + `pre_existing_adjacent` | Inherited fixture convention, not created at generation 3. |
| `R1C-C-TRUST` | `bound_finding_not_closed` (narrowed) | The count is a real tightening; trust is not established. |
| `N3-MERGE-SEAL` | `repair_introduced_regression` | Created by generation 3's `earliest_final_publication`. |
| `P1-RENAME-SHALLOW` | `pre_existing_adjacent` | Must never carry `repair_introduced_regression`; the *claim* about shallow clones is `A7-C3` and is separate. |
| `deliverable_paths` | `requirement_or_scope_expansion` + `communication_ambiguity` | Applies to the unstopped scope decision, not automatically to the field's soundness. |

Evidence-state rules, applied per row:

- `reproduced` requires the reviewer's own fixture on the `753c8db` tree;
  quoting the result report is not reproduction.
- `verified` requires the property to hold under a held-out shape, not only
  under a candidate-authored test.
- `not_applicable` requires a stated reason and exact evidence refs; a bare
  `not_applicable` is rejected by the gate.
- `unavailable` requires the reason, an owner, and a release condition, and
  must not claim route authority.
- A test that passes on both `753c8db` and the candidate is a **forward
  guard**, not a regression control, and must be labelled as such — the
  generation-2/3 Cargo lineage failed exactly this way.
- A reworded observation shares its effect episode ID. Prior effect episode
  IDs must never reappear in `new_findings`.

### Terminal evidence requirements

- The report is the reviewer's only tree change, committed as a single-parent
  direct child of the review-control commit, changing exactly one path under
  `docs/development/ai/swarm-results/`.
- The review-control commit must have candidate
  `9ca01d9a457aae05adc7f7881b66ce066aacaff6` as its **sole parent** and may
  touch only paths under `docs/development/ai/task-packets/`,
  `docs/development/ai/schemas/`, or `docs/development/tools/`. This packet
  must be present at that commit byte-identical to the copy committed on the
  preparation lane, at the digest recorded in the launch preconditions.
- AIA-047 gate finding F-12 notes that an intermediate commit between
  candidate and control can mutate and revert product code invisibly. The
  control commit must therefore be the *immediate* child of the candidate, and
  the reviewer must report `git rev-list --count 9ca01d9a..<control>` as 1.
- The reviewer process and every child process must be stopped, with
  `child_processes_active` at 0, before the record is written.
- `terminalization.state` must be `complete` with a content-addressed event
  log. AIA-047 finding F-4 established that the gate does not resolve or
  re-hash any evidence reference or log digest, so the manager must resolve
  and re-hash the event log and every required validation log **by hand** and
  say so in the record's rationale.
- Every validation row carries an exact command, a terminal flag, the real
  exit code, a log reference, and a complete-log SHA-256. Background progress,
  truncated output, a different tree, a different argv, or a different
  environment is never `passed`.
- Prefer `docs/development/tools/run_timed_gate.py` and report its monotonic
  duration and wall-clock-discontinuity field rather than shell wall time.

### AIA-047 recurrence metrics

| Metric | Derivation |
|---|---|
| review generation | 4 |
| final decision | the report's single `OVERALL_VERDICT` line, mirrored into `review_decision` |
| prior obligations closed / open / routed / unavailable | counted from `prior_findings[].disposition`; must equal the `outcomes` fields exactly. Twenty rows are frozen, so `prior_findings_total` is 20 unless the reviewer justifies a split or merge in the report |
| prior observations vs prior effect episodes | distinct `observation_id` count vs distinct `effect_episode_id` count; divergence is the duplication signal this pilot exists to expose |
| new observations vs new effect episodes | same rule over `new_findings` |
| candidate-created findings | `new_findings` with `attribution: "created_by_candidate"`, each carrying `repair_introduced_regression` |
| unplanned surfaces | `changed_surfaces` with `planned: false`; two are declared here (the `deliverable_paths` shape and the exemplar's new limitation claim), so a different number is itself a finding |
| `repair_service_ms` | candidate committer time minus dispatch-event commit committer time = `1785503677 − 1785502560` = **1,117,000 ms**, recorded with the explicit note that it is bounded by commit metadata, not an execution clock |
| `review_service_ms` | reviewer start to `OVERALL_VERDICT` from the event log; `null` plus an `unavailable_metrics` entry if the log cannot establish it |
| `verdict_to_report_ms` | `OVERALL_VERDICT` to report commit, from the event log; same null rule |
| unavailable evidence | enumerated in the report and mirrored in the record |
| operator time | manager-side minutes to produce and validate the record, for the Cargo-versus-KGI comparison |
| cross-arm comparison | the Cargo arm's counterpart figures are frozen in `cfr168-source-keyed-cargo-third-closure-rereview-with-recurrence-20260731.md`; compare obligation counts, planned/unplanned surfaces, duplication ratio, and operator time across the two arms rather than reading either alone |

## Non-Goals

- Do not edit the candidate, the generation-2 or generation-3 review reports,
  either repair packet, the AIA-047 schema, `check_review_recurrence.py`, or
  any canonical management document.
- Do not repair `R1C-C-TRUST`, `N3-MERGE-SEAL`, or `P1-RENAME-SHALLOW`, and do
  not treat their routing as closure.
- Do not reopen the 13 generation-2 executable residuals or the 18 previously
  closed rows absent new exact evidence of regression at `9ca01d9a`.
- Do not propose or adopt a finding quota, a checklist length, a template
  shape, a mandatory detail count, a repair-delta threshold, a severity or
  routing policy, or a universal review method — the pilot resume gate forbids
  all of these as outputs.
- Do not treat the candidate's 16 A-row tests, six new probes, or the frozen
  55/15 tables as the invariant.
- Do not run Cargo, `rustc`, Clippy, `rustfmt`, Docker, install, dogfood,
  demo, or the product CLI suite; do not touch product source, issue state, or
  agent wrappers.
- Do not use Codex, Fugu, a subagent, a scheduler, a network command, or any
  parallel lane.
- Do not rebase, amend, squash, cherry-pick, or perform any destructive Git
  operation; do not integrate, Close, Install, Dogfood, or Retrospect.
- Do not promote AIA-047 to permanent policy, and do not open a holdout of the
  frozen review-intervention pilot.

## Validation

Run held-out controls and negative controls first, then the broad checks. Run
the `unittest` commands from `docs/development/tools`.

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  test_ai_tools.DevelopmentFlowRoundFourTests
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest test_ai_tools
PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile docs/development/tools/*.py
python3 docs/development/tools/check_ai_contracts.py
python3 docs/development/tools/check_ai_contracts.py --json
python3 docs/development/tools/check_ai_contracts.py \
  --validation-instant 1970-01-01T00:00:00Z
python3 docs/development/tools/check_ai_development.py
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_source_docs.py
python3 docs/development/tools/check_ai_acceleration_docs.py
git diff --check
```

Record the complete exit status of each and distinguish a terminal result from
background progress. Every control in the negative-control table must be run
and reverted, with the restored tree re-verified green. The candidate defines
145 test methods; report the number you measure rather than repeating that
figure. `check_project_docs.py` will print a different Markdown count once the
review report exists — state both numbers and the arithmetic that links them.

## Handoff

Write one report at:

```text
docs/development/ai/swarm-results/development-flow-kgi-kpi-r4-rereview-claude-20260731/review.md
```

Use the compact exact independent review shape plus the successor review
recurrence addendum in `docs/development/ai/validation-report-template.md`.
The report must contain the recomputed immutable identity table including the
base-pinning check; one disposition per frozen obligation row; the
predecessor-failure re-derivation table; the candidate-positive verification
table; the exact Git-derived changed-path coverage table; the actual-surface
table with planned/unplanned marked; the three `deliverable_paths` scope-test
results; the held-out negative-control results; causal classes and evidence
states from the schema enum; explicit unavailable evidence; and the three
routed rows recorded as routed rather than closed. End with exactly one line:
`OVERALL_VERDICT: ACCEPT` or `OVERALL_VERDICT: REQUEST_CHANGES`.
`ACCEPT_WITH_NOTES` is not permitted.

Commit only that report path, leave the worktree clean, stop every child
process, and return the report commit, tree, path, byte count, SHA-256, and
the exact validation results.

Acceptance of this report authorizes no integration and no process change. The
manager then writes the final `docs/development/ai/review-recurrence/` record
against `docs/development/ai/schemas/review-recurrence-control.v1.schema.json`
with `generation: 4`, `record_status: "final"`,
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

Both must clear. Neither may be waived by a reviewer or an implementer.

1. **The AIA-047 gate independent review must be repaired and accepted.** That
   review is committed at `df156088b7f416e2a7c3ded8f3623c1b286e9e9a`, path
   `docs/development/ai/swarm-results/cfr168-review-recurrence-gate-independent-review-claude-20260731/review.md`,
   33,307 bytes, SHA-256
   `02bc2030728fd2c471237dd1a5da8eca268c07836c7d5c494bb73b2050ffb05b`, and its
   verdict is **`OVERALL_VERDICT: REQUEST_CHANGES`**. It recommends fixing
   F-1 through F-4 before either pilot run: the unconstrained
   `base_revision` (F-1), `ancestor_of_candidate` admitting a retro-fitted
   contract (F-2), `dispatch_event_attested` performing no timing check at all
   (F-3), and no evidence reference or log digest ever being resolved (F-4).
   Three of those four bear directly on this packet — the base pinning, the
   dispatch attestation, and the hand-resolution of evidence digests are all
   compensating manual controls written above precisely because the tool does
   not yet enforce them. Launch requires the repair to land and the manager to
   accept a fresh verdict on it; a `REQUEST_CHANGES` gate review cannot
   authorize a pilot run.
2. **The preregistered historical replay pilot must be terminally
   adjudicated.** The frozen retroactive review-intervention pilot
   (`docs/development/ai/swarm-batches/cfr168-review-intervention-pilot-resume-decision-gate-20260731.md`;
   pilot task commit `cdaee6ad3b5e497f6f872bfe76783d0e26e8df85`, calibration
   SHA-256 `4d57c323b7c20015c9cd388c4175c973631a67839d8e3d0cc427e58a7cec87d9`,
   holdout state **unopened**, pilot decision **none**) must reach exactly one
   of `RESUME_UNCHANGED`,
   `RESUME_WITH_SEPARATE_SENSITIVITY_REPLAY`, `RETIRE_PRESERVED_PILOT`, or
   `UNAVAILABLE`, recorded in the required dated manager record. Until then
   this review would run while a competing measurement of the same
   review-loop question is frozen mid-flight, and its causal classifications
   could not be compared against the pilot's own. Do not open a holdout to
   satisfy this precondition; a `RETIRE_PRESERVED_PILOT` or `UNAVAILABLE`
   adjudication clears it just as well.

Additionally, at dispatch: live resource and provider admission must grant a
slot under the recovery execution ledger's floor; a fresh reviewer session
must be created in a clean detached worktree at exactly
`9ca01d9a457aae05adc7f7881b66ce066aacaff6`; and this packet must be committed
as the review-control commit whose sole parent is that candidate,
byte-identical to the preparation-lane copy, with the manager recording that
commit, its tree, its canonical patch digest, its changed-path set, and this
file's SHA-256 as the record's control-overlay identity.
