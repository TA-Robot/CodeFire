# AI Task Packet

Status: completed

Packet ID: MS-product-value-proof-segment

Schema version: human-v1

Risk score: 6

Reasoning tier: high

## Objective

Select one honest candidate segment and freeze a bounded claims-comprehension / first-seal value study
before any participant setup or mutation. Define the task, value hypothesis, questions, falsifier,
measurements, privacy boundary, and ordered adopt / narrow / stop table. Do not run the study.

## Scope

- one segment that can genuinely consume CodeFire's current standalone CLI contract;
- one first-seal task and source-blind comprehension rubric;
- frozen timing/friction/value thresholds and terminal consequences;
- this packet only.

## Required Classification

1. Compare the smallest credible segments, including an AI-assisted repository maintainer/operator,
   and select exactly one. State what one clean-room participant can and cannot prove about that segment.
2. Freeze one isolated first-seal task whose output has user value beyond merely making tests pass.
3. Freeze questions covering base/head, certificate/claim scope, local versus portable authority,
   missing proof, actionable next step, and why the sealed result is or is not worth the setup cost.
4. Define observable setup time, first useful result, first seal, command count, docs/help load,
   misunderstanding, wrong action, private correction, off-task mutation, intervention/support cost,
   task value, and maintenance-friction measurements.
5. Define a falsifier and ordered stop / narrow / adopt table before results exist. One run and at most
   one evidence-triggered bounded correction/rerun may be admitted; no indefinite retry.
6. Define privacy and source-blind boundaries. No network, telemetry, hidden source/tests/internal docs,
   first-run answer key, or maintainer clarification.
7. Classify existing CFR-171..173 residuals and the adoption/release parked findings as
   `block-m1`, `park`, or `drop` for this exact study without pre-admitting them.

## Exact Write Set

- `docs/development/ai/task-packets/milestone-product-value-proof-segment.md` only.

## Validation

```bash
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_ai_acceleration_docs.py
git diff --check
```

## Non-Goals

- running or recruiting a participant, contacting external users, or claiming market validation;
- changing product/source/tests/public docs, creating telemetry, or building an adapter;
- broad interviews, ICP sizing, pricing, multi-repository/team pilots, or support-policy work;
- altering thresholds after evidence or starting M2–M5.

## Handoff

Return the exact segment, limitations, task, public inputs, questions/answer clauses, metrics, privacy
boundary, falsifier, ordered decision table, issue classification, `block-m1`, validation, and commit.
Commit only this packet.

## Result

Status **completed**. Verdict **`block-m1`: none** — 0 `block-m1`, 11 `park`, 8 `drop`.

One honest segment, one first-seal task, a seven-item source-blind comprehension rubric with explicit
accept/reject clauses, sixteen observable measurements, a privacy/source-blind boundary, two
falsifiers, and an ordered stop / narrow / adopt table are frozen below **before** any fixture,
runner handoff, or participant exists. No participant was run or recruited. No user was contacted.
No repository, install, canonical store, or product file was mutated. No telemetry was created. No
network was used. No CodeFire command of any kind was issued by this packet. The only file written
is this packet.

Read for this classification: `AGENTS.md`, `docs/development/milestones.md`, this packet,
`docs/development/ai/task-packets/milestone-product-value-proof-first-seal.md` (node 2's frozen
remit, read to fix the authority split in §0 — its Result section does not exist yet),
`docs/development/ai/audit-latest.md`, `docs/development/code-review-issues/cfr-17{1,2,3}.md`,
the closed `milestone-adoption-wedge-{contract,b1-fix}.md` packets, `docs/development/history.md`,
and the public product surface (`README.md`, `docs/01_product_definition.md`,
`docs/04_cli_spec.md`, `docs/05_consistency_model.md`, `docs/09_index_trace_policy.md`,
`docs/automation_interface.md`, `docs/runbook.md`, `schemas/public/**`, plus targeted greps of
`docs/08_object_store.md` and `docs/11_internal_architecture.md` **to establish what the
participant must not receive**). No `crates/**` and no `tests/**` were read.

### 0. Fixed HEAD, authority split, and what this packet does not own

```text
integration HEAD (exact)   d9481644f5dfd47d8cdff507df3c4215fa542fa6  ("docs: close release M1 and shift to product value")
packet worktree branch     claude-agent/m1-product-segment-20260822
worktree status at start   clean
node                       1 of 7, active M1 MS-product-value-proof
owner                      Claude Opus 5 (read-only classifier)
write set honored          docs/development/ai/task-packets/milestone-product-value-proof-segment.md only
```

Nodes 1 and 2 are independent and must not silently overlap. The split, frozen here:

| Owned by node 1 (this packet) | Owned by node 2 (`…-first-seal.md`) |
|---|---|
| segment selection and its proof limits (§1, §2) | runner identity / fresh-build rule and acquisition |
| first-seal task **semantics**, work order, value clause (§3) | fixture bytes, project content, and its sealed base |
| public inputs handed to the participant (§4) | exact argv sequence, help/JSON field walk, mutation boundary |
| comprehension rubric and accept/reject clauses (§5) | controller/key/participant root topology, transcript format |
| measurement definitions (§6) | timing instrumentation mechanics and preservation manifest/digests |
| privacy and source-blind boundary (§7) | no-private-correction enforcement mechanics |
| falsifiers and the ordered decision table (§8, §9) | crash / wrong-action / claim-misreading detection points |

Reconciliation rules for node 3, frozen now so neither classifier can be quietly overridden:

1. If node 2's derived public path cannot support a rubric item, node 3 **narrows the task** (§3) or
   **removes the rubric item and re-scores out of the reduced total**, and records it. Node 3 must
   not edit the thresholds in §9 to fit the path.
2. If node 2's admitted docs/runner packaging differs from §4, the **intersection** wins (the smaller
   handed set), and the resulting limitation is recorded in the board on M1 exit.
3. Nothing in §3–§9 may be edited after the fixture root exists. A post-evidence edit invalidates the
   study and forces `stop`.

### 1. Segment comparison and the one selected

Seven smallest-credible segments were compared against four hard filters: (F1) can genuinely consume
the **current standalone CLI contract** with no remote, no server, no Git bridge, no hosted service
and no agent platform; (F2) the first-seal output has value to that segment **immediately and
locally**, i.e. it does not depend on authority CodeFire does not yet claim; (F3) one bounded
clean-room participant can complete the whole loop alone; (F4) it does not require product source,
tests, or internal docs to be understood.

| # | Segment | F1 | F2 | F3 | F4 | Disposition |
|---|---|---|---|---|---|---|
| S1 | **AI-assisted solo maintainer/operator of a spec-driven repository** | yes | yes | yes | yes | **selected** |
| S2 | Automation/CI tool author consuming the JSON envelope | yes | no | yes | yes | drop D1 |
| S3 | Reviewer/auditor of someone else's sealed commit | yes | no | no | yes | drop D2 |
| S4 | Release owner restricting circulation to consistent branches | no | no | no | yes | drop D3 |
| S5 | Repository maintainer repairing object/layout damage | yes | no | yes | yes | drop D4 |
| S6 | Framework/tool author extending Atom extraction | yes | yes | no | **no** | drop D5 |
| S7 | Regulated/compliance owner needing an audit trail | yes | **no** | yes | yes | drop D6 |

**Selected: S1 — the AI-assisted solo maintainer/operator.** Concretely: one person who owns a small
specification-driven repository (requirement docs, design notes, implementation, tests as files),
whose edits are largely produced by an AI coding agent, who has no team, no CodeFire server, and no
release obligation, and whose actual daily risk is exactly `docs/01_product_definition.md` §1.2
bullet 6 — "AIや自動化ツールが大量変更を行い、人間が成果物間の整合を後追いで確認できなくなる".

Why S1 and not the others:

- It is the segment the product's own claims are written for. `README.md:3` sells "consistency-first
  version-control style CLI for specification-driven development **in the AI era**" and "make it hard
  to record inconsistent work as complete"; `01_product_definition.md` G-002 (traceable change
  impact), G-005 (no hidden checkpoint), G-008 (local-first durability) are all satisfiable by one
  person on one machine. Testing S1 tests the claim the project actually makes.
- It needs **only** the standalone local loop (`init`/`open`/`scan`/`fire`/`extinguish`/`verify`/
  `commit`/`show`/`status`), which is the exact surface M1 is allowed to touch. No M2–M5 dependency.
- The value accrues to the same person who paid the setup cost, inside one session. That makes
  "is the verified result worth the setup" a question one bounded run can actually answer, instead of
  a question about someone else's future trust.
- It is the only candidate whose *comprehension* burden is the product's real risk. S1's failure mode
  is not "cannot run the command", it is "believes the seal proves more than it does", which is what
  M1 exit bullet 3 asks about.

Rejections, with the reason each is not the subject (recorded as `drop` in §10, not filed as work):

- **D1 / S2** — Already the subject of the *closed* `MS-adoption-wedge` (source-blind read-only
  inspection, terminal `adopt`). Its value test is envelope/schema stability, and it needs no seal at
  all, so it cannot test "first useful seal worth its setup cost". Re-running it would be a duplicate.
- **D2 / S3** — A reviewer cannot perform a *first* seal; someone must produce one first. Worse, the
  reviewer's value depends on **portable** authority, and the current certificate is explicitly local
  (`signature.present=false`, subject `live_open` / `local_unisolated`). Selecting S3 would test a
  claim the product does not make. S3 is retained as the *hypothetical second reader* inside rubric
  item Q3/Q7 — which is where it belongs.
- **D3 / S4** — Requires `upload`, merge requests, and server-side validation. Off M1 scope by the
  board's own non-goals, and needs infrastructure this study may not create.
- **D4 / S5** — Requires a pre-damaged store, produces diagnosis rather than a seal, and yields no
  user-visible work product. Cannot satisfy "value beyond making tests pass" because it produces no
  work at all.
- **D5 / S6** — Extending Atom extraction requires reading `crates/**`. Structurally incompatible with
  the source-blind boundary in §7.
- **D6 / S7** — The audit-trail value an auditor wants is third-party-verifiable attestation:
  signing, isolated re-execution, retention policy. `show.signature.present=false` and
  `trust_kind=local_unisolated` mean the product does not yet claim it. Admitting S7 would build a
  study whose success condition the product is *designed* not to meet yet. Park as the next segment
  **only if** S1 adopts.

### 2. What one clean-room AI-assisted participant proves — and what it does not

This is the honest boundary of the whole milestone. It is stated before the study so no later slot can
inherit a stronger claim.

**Proves (if the run completes):**

1. **Path existence and cost.** That a competent operator with zero prior CodeFire exposure can — or
   cannot — reach a first seal on a real change from the handed artifact plus public docs alone, with
   a recorded command count, doc load, and elapsed time. Existence and non-existence are both real
   findings.
2. **Claim comprehensibility.** Whether the public surface (docs pack + `--json` fields + `--help`)
   communicates *what the seal asserts* well enough that a first-time operator does not overstate it.
   This is a reproducible, falsifiable property of documentation and JSON — not of a human's feelings.
3. **That the refusal works from outside.** Whether the central claim ("hard to record inconsistent
   work as complete", `01_product_definition.md` §1.10) actually fires on a genuinely inconsistent
   intermediate state experienced by someone who did not write the checker.
4. **Absence of a private channel.** That the loop is completable with no maintainer clarification —
   or that it is not.
5. **A strong negative if it fails.** A competent, patient, fast reader failing is powerful evidence
   against the current surface. Negative results from this design generalize better than positive
   ones, and §9 is deliberately built so a fast success with a wrong claim reading scores `stop`.

**Does not prove — none of these may be claimed on this evidence:**

1. **Not human adoption.** An AI participant reads dense Japanese specification prose and raw JSON
   faster than a human, does not get frustrated, does not quit, has no Git muscle memory, no
   switching cost, and no economic stake. **`setup_time_s`, `ttfur_s`, and `ttfs_s` are machine
   times and must never be reported as human onboarding times.**
2. **Not willingness to pay, retention, or ongoing use.** One seal is not a habit.
3. **Not maintenance cost.** §6 records a *labelled estimate* only. This study changes no code and
   spans one session.
4. **Not team, multi-branch, multi-repository, merge, remote, review, or release workflows.**
5. **Not third-party value.** Portable authority is absent by construction; the seal's worth to
   anyone other than its producer is untested and, on this contract, untestable.
6. **Not toolchain-free adoption.** Inherited limitation (`MS-adoption-wedge` park P2): the runner is
   handed to the participant, not acquired by them.
7. **Not release-payload sufficiency.** §4 hands the repository's public docs. It does not test
   whether the internal-only release candidate's three shipped docs are enough.
8. **Not scale or language coverage.** One small fixture, whatever Atom extractors happen to cover.
9. **Not a market, ICP, segment size, or pricing claim.** n=1. A single run cannot separate "the
   surface is good" from "this participant was capable". One bounded rerun is permitted by §9 and
   still supports no rate, frequency, or population claim.
10. **Not that S1 exists in commercially meaningful numbers.** Segment *plausibility* is argued from
    the product's own requirement doc in §1; it is not measured here.

### 3. The frozen first-seal task T

**Shape.** One disposable project inside one isolated CodeFire repository, containing the four
artifact kinds the trace model is built for — requirement document, design document, implementation,
test — plus the three versioned project files (`codefire.yaml`, `codefire.links.yaml`,
`codefire.policy.yaml`, `docs/09_index_trace_policy.md` §9.1) with required links
`requirement → design → code → test` and **one required verification command** in
`codefire.policy.yaml`. Byte content, base sealing, and layout are node 2's.

**Work order handed to the participant (semantics frozen here).** A single **requirement change**:
one already-documented rule in the requirement document is tightened (e.g. a stated input limit or
validation rule changes value). To reach a seal the participant must:

1. edit the requirement document to the new rule;
2. propagate it to the design document, the implementation, and the test;
3. run the local loop, read the fires their edit produced, and extinguish each with a rationale or
   evidence (`docs/01_product_definition.md` FR-007, §1.6 Resolution: `no-change-required` still
   requires a reason or evidence);
4. reach `verify` with no blocker and seal with `commit`.

**Mandatory intermediate inconsistency.** The work order is authored so that a naive "make the test
pass" pass leaves at least one downstream artifact stale — so the participant must experience **at
least one genuine refusal** (a `verify`/`commit` blocker in the documented taxonomy: 10 open fires,
11 missing required links, 12 stale resolutions, 13 duplicate Atom IDs, 14 failed check, 21 missing
evidence ref) and clear it correctly. A run that never triggers a refusal cannot count as value
evidence (§8 falsifier F2, §9 row 4).

**Bounded off-task temptations.** The fixture must contain at least two visible, adjacent, explicitly
out-of-scope items — e.g. an unrelated stale TODO in another document, and a second branch present in
the repository. The work order names them as out of scope. Touching them is measured as
`off_task_mutation_count` (§6) and, for anything outside the fixture root or any forbidden
subcommand, as `wrong_action_count`.

**Value clause — why T's output is worth more than a green test run.** Frozen, and graded in §6
`task_value_score`:

```text
A green test run records: the tests passed at some moment.
T's sealed commit additionally records, retrievable later by public read-only commands:
  - which requirement changed              (Atom index / fire delta on the sealed commit)
  - which downstream artifacts were confirmed as a consequence, and on what basis
                                           (resolutions with rationale/evidence)
  - that a checked committable predicate held at seal time over a named subject
                                           (certificate result + claim profile/phase/subject)
  - and that the repository REFUSED to seal while a downstream artifact was still stale
                                           (the refusal the participant had to clear)
```

That last line is the product's actual differentiator and the reason T is not "run the tests". A
`cargo test` / `pytest` green result records none of the four.

### 4. Public inputs handed to the participant

The participant receives exactly five things and nothing else:

1. the admitted runner (execute + read only) — identity and acquisition are node 2's;
2. the fixture root — the **only** admitted mutation target;
3. the written work order (§3), containing the requirement change and the named out-of-scope items;
4. a frozen, copied **docs pack**;
5. the answer-sheet template (§5), with the instruction that every answer must cite the exact command
   plus JSON field path or doc line it came from.

Docs pack — the repository's public documentation surface, copied read-only:

```text
README.md
docs/00_philosophy.md      docs/01_product_definition.md   docs/02_core_concepts.md
docs/03_workflow_spec.md   docs/04_cli_spec.md             docs/05_consistency_model.md
docs/06_fire_extinguish.md docs/07_local_repository_model.md
docs/08_object_store.md    docs/09_index_trace_policy.md   docs/13_risks_and_non_goals.md
docs/automation_interface.md
docs/runbook.md
schemas/public/*.schema.json          (7 files, incl. branch-list-v1 and show-result-v1)
schemas/conformance/manifest-v1.json
adr/**                                (public decision records cited by the docs above)
```

Plus the runner's own `--help` output and `capabilities --json`. `docs/10_merge_remote_server.md` is
included or omitted at node 2's discretion; it is off the selected path either way.

**Deliberately excluded** (each exclusion is what makes a rubric answer meaningful):

```text
crates/**  tests/**  demo.sh                    product source and tests
docs/11_internal_architecture.md                internal architecture — the ONLY place in the tree that
                                                glosses trust_kind=local_unisolated (line 765)
docs/12_*                                       retired MVP plan
docs/development/**                             board, packets, issues, audits, history, this packet
subprojects/**                                  canonical dogfood store
any prior study/release evidence root           /tmp/m1-* , /tmp/codefire-m1-release.*
the answer key                                  sealed before handoff (§7)
any maintainer clarification channel            none exists during the run
```

Answerability was checked before freezing, so the rubric is not a trick: the public pack **does**
carry the vocabulary needed for the hardest item. `docs/automation_interface.md` prints the claim
subject shape (`kind=live_open`, `trust_kind=local_unisolated`, `immutable=false`) and states that
missing claim fields "are not unknown-as-success and they are not synthesized proof";
`docs/04_cli_spec.md:638` fixes `show.certificate` as the result **string** and `claim` as a
projection that must not synthesize authority; and `docs/04_cli_spec.md:675` (§4.17 `review-pack`)
supplies the explicit stronger contrast — subject `sealed_commit_pair`, `trust_kind=sealed_validated`,
immutable. A diligent participant can therefore infer local-versus-portable from the public pack.
That is precisely why §10 does **not** pre-admit a CFR-171 residual: whether the inference is
actually reachable is the measurement, not an assumption.

### 5. Claim-comprehension rubric — seven items, frozen accept/reject clauses

Answered from the participant's own run output and the docs pack only. Submitted **once**; no
revision, no feedback. Every answer must cite command + exact JSON field path (or doc line). An
answer that is correct but cites an invented field scores 0 and is also counted in
`undocumented_field_reliance`.

| # | Item | Covers |
|---|---|---|
| Q1 | base and head | base/head |
| Q2 | what the certificate/claim asserts, and one thing it does not | certificate/claim scope |
| Q3 | is the seal local or portable authority | local versus portable |
| Q4 | two authorities the output does not carry | missing proof |
| Q5 | one `next_actions` entry used, and one non-zero exit code hit | actionable next step |
| Q6 | what blocked the first refused attempt, and whether policy could switch it off | refusal comprehension |
| Q7 | is this sealed result worth its setup cost — with their own numbers | value versus cost |

**Q1 — base/head.** Accept: names the new sealed commit ID as head and its immediate parent as base
from `show <branch> --json` `data.show.commit` / `data.show.parents[0]`; names the branch; states that
`branch list --json` `branches[].head` is the same head; and correctly states that
`branches[].state` is the **persisted branch-record** state, not live open-directory authority.
Reject: base/head inverted; the `init` commit named as head; `branches[].state` cited as proof the
directory is currently open; any invented field.

**Q2 — claim scope.** Accept: `show.certificate` is the certificate **result string** (`passed`;
`consistent` is its legacy alias) and not the certificate object; `claim.result` matches it;
`claim.profile=local_verification_v1` and `claim.phase=pre_commit` mean the assessment ran under a
local verification profile **before** the seal; the subject names what was assessed. **Plus at least
one correct non-assertion**, e.g.: it does not assert the code is correct or bug-free; it does not
assert any post-seal re-verification; a passing configured check is not evidence the test is adequate;
`passed` means the documented committable predicate held over the listed blocker classes only.
Reject: reading `certificate` as an object; claiming the certificate proves correctness or absence of
bugs; `phase` read as post-commit; treating absent claim fields as satisfied.

**Q3 — local versus portable (carrier item).** Accept: the seal is **embedded/local**, not portable,
and the answer rests on fields actually present — `show.signature.present=false` (no commit
signature) together with `claim.subject.kind=live_open`, `trust_kind=local_unisolated`,
`immutable=false` (the assessed subject was a mutable local worktree, not an isolated sealed pair) —
and concludes that a third party holding only the commit ID cannot verify the claim cryptographically
nor reproduce the assessed subject; they must trust the producing machine. Citing the
`sealed_commit_pair` / `sealed_validated` contrast strengthens but is not required.
Reject **in both directions**: (a) asserting the seal is signed, notarized, portable, attestable, or
third-party verifiable; (b) over-correcting to "the missing signature makes the commit invalid / the
sealed history untrustworthy"; (c) claiming the store validator's internal checks confer portable
authority.

**Q4 — missing proof.** Accept: names **two** absent authorities supported by fields actually present.
Valid: no commit signature; the assessed subject is a mutable live worktree, so there is no isolated
or sandboxed re-execution claim; the required verification command ran under a local shell profile, so
a pass is not a reproducible-build claim; nothing in the seal proves the working tree *still* matches
after the seal; the base (`init`) commit's claim carries only `result`, so the parent asserts strictly
less than the head. Reject: any invented absent field; any item the output or docs contradict; naming
only one.

**Q5 — actionable next step.** Accept: quotes one `next_actions` entry actually returned during their
run (`kind`, `command`, `target`), says whether they followed it and whether it changed the outcome;
and names one non-zero exit code they hit with its documented meaning from the taxonomy. Reject: a
fabricated next action; a wrong exit-code meaning.

**Q6 — refusal comprehension.** Accept: names what blocked the first refused `verify`/`commit`
(exit code + blocker class), explains it is a blocker rather than a warning, and states correctly
that it is a **safety floor**: open required fires, stale resolutions, missing evidence refs,
duplicate Atom IDs, failed required checks and unresolved merge conflicts cannot be disabled by
policy — the only documented relaxation is `require_trace_completeness=false`, which turns missing
required links into a recorded warning; setting the compatibility fields to `false` in working config
is a configuration error (`docs/05_consistency_model.md` §5.2). Reject: claiming any safety-floor
blocker is switchable; describing the refusal as arbitrary; misattributing the blocker class.

**Q7 — worth the cost.** Accept: cites **their own** recorded numbers (`setup_time_s`, `ttfs_s`,
`commands_issued`, config lines authored), names one concrete question the sealed artifact answers
later that a green test run does not (§3 value clause), **and** names one concrete case where the
setup cost would not be worth it (e.g. a repository with no cross-artifact obligations, a throwaway
prototype, or a language the Atom extractors do not cover). Reject: generic marketing language; value
claims not traceable to their own evidence; refusing to name a non-worth case.

**Grading.** Seven items, 0/1, total 7. The node-5a controller grades against these clauses; the
node-6 auditor re-grades independently from the same artifacts. **Any disagreement on any item is
resolved against `adopt`** (the lower score stands). Deterministic values for Q1 are derived by the
controller from the same run's outputs; no value is pre-baked here, so nothing can drift between
contract and run. The graded answer key's `sha256` is recorded in the run evidence before the sheet
is opened.

### 6. Observable measurements

All from local run artifacts the harness writes itself (per-invocation `argv` / stdout / stderr /
`meta` with start, end, duration, exit, cwd — format is node 2's), the participant's transcript, and
the answer sheet. No product instrumentation, no telemetry, no network, no interview. `t0` is the
handover timestamp.

| # | Metric | Definition |
|---|---|---|
| M1 | `setup_time_s` | `t0` → first runner invocation exiting 0 against the fixture repository |
| M2 | `ttfur_s` (first useful result) | `t0` → first command whose output tells the participant something actionable about **their own** change (first `status`/`scan` listing their changed Atoms or fires, exit 0) |
| M3 | `ttfs_s` (first seal) | `t0` → `commit` with `data.applied=true`, exit 0 |
| M4 | `ttac_s` | `t0` → answer sheet submitted |
| M5 | `commands_issued`, `codefire_commands_issued`, `distinct_subcommands_used` | counted from recorded argv |
| M6 | `help_invocations` | `--help` / `capabilities` invocations |
| M7 | `docs_files_read` | self-reported list, cross-checked against citations |
| M8 | `refusals_encountered` | non-zero CodeFire exits, split: `intended_refusal` (10–15, 21, 22 on genuinely inconsistent state), `usage_error` (2), `unexpected` (1, 20, 30, 33, 40, 50, 60, signal, or any crash) |
| M9 | `retry_loops` | consecutive repeats of the same subcommand with no intervening file edit or doc read |
| M10 | `misunderstanding_count` | rubric items scored 0 (§5) |
| M11 | `unanswerable_count` | items the participant marks not answerable from the pack, **with the field/doc they looked for** |
| M12 | `undocumented_field_reliance` | distinct JSON field paths cited in correct answers that appear in neither the docs pack nor `schemas/public/**` |
| M13 | `wrong_action_count` | any mutation outside the fixture root; any forbidden subcommand (`upload`, `request-*`, `serve`, `gc`, `migrate apply`, `clone` of a non-fixture repo, `--discard`); any CodeFire command targeting a non-fixture path |
| M14 | `off_task_mutation_count` | fixture files changed outside the work order's declared scope (the §3 temptations) |
| M15 | `private_correction_events`, `interventions` | any answer/hint/confirmation beyond the frozen handoff (**must be 0**); plus controller actions needed to keep the run alive, each with a reason and whether it disclosed product knowledge |
| M16 | `task_value_score` (0–3) | rubric-scored, **not** participant enthusiasm: +1 the sealed commit lets an independent reader answer "which requirement changed and what was confirmed on what basis" using public read-only commands only; +1 the product refused at least one genuinely inconsistent state during the run and the participant cleared it correctly; +1 Q7 names a specific later question the seal answers, traceable to recorded evidence |

`maintenance_friction_estimate` is an **estimate, not a measurement**, and must be labelled so
wherever it appears: config lines the participant had to author across the three project files,
resolutions required per changed Atom, and the count of manual steps that would repeat on every future
change. This study spans one session and changes no code, so no measured maintenance cost exists.

Recording rules: **missing data is never scored as success** — an unrecorded metric forces the
`narrow` row, never `adopt`. Every directory digest recorded anywhere in this study must state its
algorithm (adopting the release audit's parked lesson, `audit-latest.md` §10); the default is
`PRESERVE_DIGEST` as defined by the release Stage B. Machine timings must carry the §2 limitation-1
label wherever they are reported.

### 7. Privacy and source-blind boundary

```text
network            none. No egress, no remote, no cf:// or cf+http:// target, no package fetch during the run.
telemetry          none. No instrumentation is added to CodeFire. Nothing is transmitted anywhere.
handed to C        runner (execute+read), fixture root (only mutation target), work order, docs pack, answer sheet.
withheld from C    everything in the §4 exclusion list, including the answer key, both classification
                   packets, the board, the issues, all prior audits, and every prior evidence root.
answer key         graded key sealed and sha256-recorded BEFORE the sheet is opened. No first-run answer key.
clarification      none. The controller does not confirm, deny, hint, or correct during the run. Any such
                   event is recorded as private_correction_events and forces stop (§9 row 1).
fixture data       synthetic project content authored for the study only. No third-party code, no secrets,
                   no personal data, no user content. Nothing is collected about any person.
retention          local disk only, retained as node-5b evidence, not published, not uploaded.
enforcement        post-hoc argv/transcript scan for reads or writes outside the handed roots, plus
                   before/after digests of every non-target root (node 2's manifest).
```

Honest limit, stated rather than hidden: containment is **operational, not kernel-enforced** (shared
uid), exactly as in the closed adoption wedge. The node-6 auditor must therefore re-check the
transcript instead of trusting the instruction.

### 8. Falsifiers

**F1 — comprehension falsifier (primary).** The M1 product-value hypothesis is falsified if a
competent source-blind operator, given the published runner and the public docs pack, either
(a) cannot reach a first seal on task T without private correction, or (b) reaches it while
misstating what the seal proves — specifically if **Q3 is wrong**, or if Q2/Q4 are wrong such that
the participant overstates "verified". *Reaching a seal quickly while believing it is a portable,
third-party-verifiable attestation is a failure, not a success.*

**F2 — value falsifier.** If the run never triggers a genuine refusal on a task authored to contain
one, the product's central claim was not exercised, and the run cannot count as value evidence
regardless of how fast the seal was reached.

### 9. Ordered stop / narrow / adopt table

Frozen before any fixture, runner handoff, or participant exists. Evaluated **once**, in this order;
the **first matching row wins**.

| Order | Decision | Condition (any clause) |
|---|---|---|
| 1 | **stop (terminal)** | preservation drift on any non-target root; OR `wrong_action_count >= 1`; OR `private_correction_events >= 1`; OR C read outside the handed roots; OR any intervention disclosed product knowledge; OR any §3–§9 clause was edited after the fixture existed |
| 2 | **stop (terminal)** | no seal reached and the cause is **not** a reproducible product refusal — i.e. crash/unexpected exit (M8 `unexpected`), unrecoverable repository state, or the path is undiscoverable from the docs pack. Record as terminal falsification of the current surface for S1 |
| 3 | **stop (terminal)** | no seal reached because an intended refusal could not be cleared from the public surface (recorded distinctly from row 2); OR rubric score `<= 3/7`; OR `ttfs_s > 7200` |
| 4 | **stop (terminal)** | **Q3 incorrect** — the seal was claimed portable/signed/third-party-verifiable, or the missing signature was read as invalidating the seal (F1) |
| 5 | **narrow** | seal reached and Q3 correct, but Q2 or Q4 incorrect/unanswerable; OR `task_value_score <= 1`; OR the product refused nothing during the run (F2); OR `3600 < ttfs_s <= 7200`; OR `refusals_encountered.unexpected >= 1`; OR `undocumented_field_reliance >= 2`; OR `off_task_mutation_count >= 1`; OR any §6 metric is missing |
| 6 | **adopt** | seal applied (`data.applied=true`, exit 0) **and** rubric `>= 6/7` with **Q2, Q3, Q4 all correct** **and** `task_value_score >= 2` including one genuine refusal cleared correctly **and** `ttfur_s <= 900` **and** `ttfs_s <= 3600` **and** `wrong_action_count == 0` **and** `off_task_mutation_count == 0` **and** `refusals_encountered.unexpected == 0` **and** `private_correction_events == 0` **and** `undocumented_field_reliance <= 1` **and** every §6 metric recorded |
| 7 | **narrow** | anything else |

Precommitted meaning of each decision:

```text
adopt   Record that the standalone first-seal path is comprehensible and locally valuable for S1
        AS MEASURED BY ONE CLEAN-ROOM AI PARTICIPANT, with every §2 limitation attached. Admit ONLY
        the single smallest blocker the run itself reproduced, if any. Do not promote a new command,
        telemetry, Git bridge, hosted service, agent platform, signing, or any human/market claim.
        Do not widen to full CFR-171 or CFR-173.
narrow  Admit ONLY the single smallest reproduced blocker, then choose exactly one of:
        (i) narrow the product's public claim wording to what the evidence supports, or
        (ii) run at most ONE bounded correction plus rerun with a fresh participant on this
             identical frozen contract, then decide adopt/stop on this same table.
        No second segment, no new rubric item, no widened task, no threshold edit.
stop    Record a terminal stop for the S1 first-seal value hypothesis against CFR-172 (mental model)
        and CFR-173 (metrics/decision). Promote nothing to core. Do not retry. Falsification is the
        result and is recorded as such in history.
```

**Retry budget: exactly one run, plus at most one evidence-triggered bounded correction and rerun.
No third attempt.** Thresholds are frozen at this commit; any post-evidence threshold change
invalidates the study and forces `stop`.

### 10. Issue classification for this exact study — `block-m1`: none

No root is pre-admitted. A root may enter only if the frozen study reproduces it as the smallest
blocker of this exact first-seal/value path.

| Issue | Disposition for this study | Reasoning |
|---|---|---|
| CFR-171 | **`park`** | The bounded slice (`codefire.branch_list.v1`, `codefire.show_result.v1`, per-command `data_schema`, additive persisted-certificate claim projection) already landed and is documented at `04_cli_spec.md:638` and in `automation_interface.md`. The residual — one cross-surface descriptor, byte-equal projections on every surface, store-validator fail-closed on descriptor mismatch, and the required-fix bullet 3 *capability-plus-human-language* distinction of embedded/local versus signed portable versus legacy/unproven — is **not** admitted here. **Named reproduction candidate, not admitted:** if Q3 is answered wrong *and* the transcript shows the participant searched for and could not find a public statement of authority semantics, node 3 may admit the smallest slice = state, in the **public** docs pack, what `signature.present`, `claim.subject.kind`, `trust_kind`, and `immutable` mean for authority, plus one explicit sentence that a local seal is not a portable third-party attestation. Not the descriptor, not a store change, not a new command. §4 records that the public pack already carries the `sealed_commit_pair` / `sealed_validated` contrast, so this candidate may well never reproduce. |
| CFR-172 | **`park`** as implementation; this study is its **write-path counterpart** | The closed wedge covered read-only inspection; T covers the mutating first-seal loop and the open-workspace mental model via Q1/Q6. No adapter, no new read-only profile, no UX sugar, no new command is authorized by this packet. |
| CFR-173 | **`park`**; bounded slice only | Only §6's metrics, §5's rubric, §8's falsifiers, and §9's thresholds are used. No cohort replay, no baseline/current re-execution, no Fire-actionability taxonomy implementation, no metric calculator, no telemetry, no retention infrastructure beyond the local evidence root. |
| CFR-168 | `park` | No remote and no decision-work budgeting on this path; not reproduced. |
| CFB-136 | `park` | No validation-cache subject reuse on this path; not reproduced. |
| CFB-133 | `park` | No `migrate` on this path; not reproduced. |
| CFR-169 | `park` | Architecture seam; off the critical path; requires source reading, which §7 forbids. |
| CFR-170 | `park` | Local-loop slice already completed; the all-command residual is not adoption permission. |

Adoption-audit parked findings, re-classified for this exact study:

1. **`open-clean` versus `opened=0` — `park`.** Now documented: `branches[].state` is persisted
   branch-record state and "not live open-directory authority"
   (`docs/automation_interface.md`). On this path the participant owns a genuinely open directory, so
   the field is truthful. It is retained as a deliberate **distractor** inside Q1's reject clause, not
   as work.
2. **Historical `claim.subject.kind=live_open` — `park` as work; load-bearing as the measured
   subject.** A first seal produced by `commit` from a live open directory truthfully projects
   `kind=live_open`, `trust_kind=local_unisolated`, `immutable=false`. That is not a defect to fix
   before the study — it is exactly what Q3 and Q4 grade. Any *change* to it is unadmitted.
3. **Remote `request-list.applied_head` omission — `park`.** No remote on this path; not reproduced.

Release-audit parked findings, re-classified:

4. **Legacy directory-digest method gap — `park` as work, adopted as a recording rule.** Every digest
   this study records must state its algorithm; default `PRESERVE_DIGEST` (§6).
5. **Rights P1–P7, signing, SBOM, registries, installer provenance, SemVer — `park`.** This study
   publishes nothing, uploads nothing, and installs only into an isolated prefix if node 2 installs at
   all. The runner remains internal-only.
6. **Operational-not-kernel containment — `park`,** restated as a §7 limitation rather than fixed.
7. **No toolchain-free adoption (wedge P2) — `park`,** restated as §2 limitation 6.
8. **Release-payload docs sufficiency — `park`,** newly recorded: §4 hands the repository's public
   docs, not the three-doc release candidate payload, so this study does not test that payload (§2
   limitation 7).

`drop` — considered and deliberately not filed as work: **D1–D6** the six rejected segments (§1);
**D7** human participant recruitment, interview protocol, ICP sizing, pricing, or any market claim;
**D8** building signing / portable attestation / isolated re-execution so that Q3 could be answered
"portable" — that is a product-direction change this study exists to inform, not a prerequisite for
it. Also dropped as non-work: telemetry collection, a Git bridge, a hosted service, agent-platform
integration, multi-repository or team pilots, and treating a green `cargo test --workspace` as
product value.

**`block-m1` from node 1: none (0).** One candidate is named above under an explicit reproduction
rule. This packet does not authorize implementing it, and node 3 owns admission.

### 11. Validation run by this packet

```bash
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_ai_acceleration_docs.py
git diff --check
```

All three pass; results are recorded with the commit. No product test, no `cargo` command, no
installer, no CodeFire invocation, and no mutating command of any kind was run. Nothing outside this
packet was written; `git status` shows exactly one modified file.

### 12. Next legal action and node-3 handoff

Node 1 is complete. **Do not create a fixture, hand a runner to anyone, run a participant, or
implement the named CFR-171 candidate.** Node 3 (orchestrator fan-in) is the next legal step and
requires node 2 (`milestone-product-value-proof-first-seal.md`, Grok 4.6), which is independent of
this packet and may still be running. At fan-in, node 3 must:

1. **Fan in against the §0 authority split.** Apply the §0 reconciliation rules: if node 2's derived
   public path cannot support a rubric item, narrow the task or drop the item and re-score out of the
   reduced total — **do not** edit §9. If node 2's docs/runner packaging differs from §4, take the
   intersection and record the limitation.
2. **Admit §3–§9 byte-for-byte as the node-5a/5b run packet, or reject them.** The task semantics,
   docs pack, rubric, accept/reject clauses, metrics, privacy boundary, falsifiers, thresholds, and
   retry budget are frozen here and must not be edited once the fixture root exists.
3. **Adjudicate the named CFR-171 candidate on reproduction only.** Node 1 asserts `block-m1`: none.
   Admitting the candidate before the run would be pre-admission and is refused by the board's own
   issue-input rule.
4. **Name the owners.** Node 5a controller = persistent study owner. Node 5b participant = a **fresh**
   session that has never read this packet, node 2's packet, the board, the issues, or any prior
   evidence root; the node-6 auditor must be **other-family relative to the node-5b participant** and
   must not be the node-1 or node-2 session. Node 1 (Claude Opus 5) and node 2 (Grok 4.6) are both
   ineligible as the participant.
5. **Verify the mandatory refusal is authored in.** If node 2's fixture cannot produce at least one
   genuine `verify`/`commit` blocker on task T, the run cannot satisfy F2 and must not start.
6. **Record the standing limitations in the board on M1 exit**, so no later slot inherits them: n=1;
   an AI participant is not evidence of human adoption and its timings are machine timings; no
   toolchain-free adoption; no third-party/portable value; release-payload docs sufficiency untested;
   no market, pricing, or segment-size claim.
