# AI Task Packet

Status: ready (fixture setup)

Packet ID: MS-adoption-wedge-run

Schema version: human-v1

Risk score: 6

Reasoning tier: high

## Objective

Run one source-blind, inspect-only adoption experiment against an isolated closed copy of already
sealed CodeFire history. Measure whether a fresh consumer can identify repository, branches, head,
base, claim/proof kind, missing authority, and a next action using only the frozen public docs and
read-only JSON commands. Produce the precommitted adopt / narrow / stop decision without product code.

## Scope

- one new controller root and one separately handed consumer root under `/tmp`;
- copied sealed `repo.json`, `objects/`, and `branches/`, with all runtime registries empty;
- the exact public docs/schema pack and C1–C6 command/rubric contract frozen in
  `milestone-adoption-wedge-contract.md` Result sections 1–6;
- local experiment evidence and this packet only.

Canonical `/workspace/project`, prior remote evidence roots, product source/tests/docs, and every
non-target repository are read-only preservation subjects.

## Fan-in adjudication

- Node 1's undocumented `show` / `branch list` payload finding is a **conditional CFR-171 candidate**,
  not a pre-run product write. The M1 exit explicitly accepts a retained terminal failure.
- Node 2 independently found the existing profile/subject/result/digest fields sufficient to attempt
  inspection and recommended no product write.
- Therefore run the frozen experiment first. Apply the section-6 table exactly: `adopt` or `narrow`
  may admit only the documented/schema-bound B1 slice and at most one identical rerun; `stop` records
  falsification and admits no fix.
- Review-pack/import/live verify/scan/open/Git are not added to the first experiment. CFR-168/CFB-136
  and later milestones remain parked.

## Exact inputs

- integration base at admission: record exact HEAD before setup;
- runner: `/workspace/worktrees/ms-remote-happy-path-budget-20260821/target/release/codefire`, SHA-256
  `70301c874d7220a71be988562df0e5188ede6a85c65c63dcbd3dcdee09f4706c`;
- canonical copy source: `/workspace/project/subprojects/.codefire`, read-only;
- consumer docs pack: exactly the files listed in contract Result section 1;
- commands, required/optional status, cwd, field questions, preservation manifest, metrics, seven-item
  rubric, falsifier, privacy rules, and ordered decision table: contract Result sections 2–6,
  unchanged;
- fresh consumer: Grok 4.6 session with no previous CodeFire context, no subagents, no web, no
  maintainer clarification, and no access authorization outside the handed consumer root.

## Procedure

### Stage A — persistent fixture owner

1. Rehash runner/provenance and every canonical preservation value. Stop on mismatch.
2. Create separate `$CTRL` and `$ROOT` with `mktemp -d`; do not reuse prior roots.
3. Materialize `$ROOT/repo/.codefire` by copying only `repo.json`, `objects/`, and `branches/`; create
   empty `opened/`, `active/`, `cache/`, `idempotency/`, `locks/`, `remotes/`.
4. Copy the frozen public docs pack into `$ROOT/docs-pack`. Create a task sheet containing only C1–C6,
   the seven questions, required citation shape, and the public thresholds; do not include an answer.
5. Create a controller-owned harness that exposes only C1–C6, records exact underlying argv/cwd/start/
   end/duration/exit/stdout/stderr, rechecks canonical and fixture preservation after every command,
   and refuses every other command label. Store the answer key outside `$ROOT`; put only its SHA-256
   in handed metadata.
6. Derive the answer key by running the same C1–C6 JSON commands controller-side before handoff.
   Optional C6 may run at most once. Stop on any nonzero required command, preservation drift, or
   answer-key ambiguity.
7. Record setup handoff, set this packet `consumer-ready`, validate, and commit only this packet.

### Stage B — fresh source-blind consumer

1. Work only inside `$ROOT`; read the task sheet and docs pack, not this packet or any source tree.
2. Use only the provided harness labels C1–C6; do not invoke CodeFire directly.
3. Submit `$ROOT/evidence/consumer-answer.json` with Q1–Q7, exact command/JSON-field citations,
   `first_useful_command`, and a short claim-scope explanation. Do not read the controller answer key.
4. Return without product/repository edits or private clarification.

### Stage C — persistent fixture owner

1. Verify the consumer transcript/read set, forbidden-token scan, answer citations, preservation, and
   metrics. Score once against the pre-hashed key.
2. Apply the contract section-6 ordered decision table without changing thresholds after seeing data.
3. Run a distinct read-only confirmation of fixture/canonical hashes and the required C2–C5 terminal
   envelopes; no consumer answer changes.
4. Record result/decision, set this packet `completed` or `blocked`, validate, and commit only it.

## Validation

Only the experiment, preservation checks, and:

```bash
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_ai_acceleration_docs.py
git diff --check
```

No build, product test, import, Git bridge, live workspace command, dogfood mutation, or later-slot gate.

## Non-Goals

- implementing B1/CFR-171 before the first score;
- using review-pack, import, open, scan, verify, context, explain, commit, remote commands, or Git as
  part of the consumer path;
- reading source/tests/internal/development docs or asking the maintainer what fields mean;
- telemetry, network, external publication, broad pilot, release, product-value, or ecosystem work;
- deleting evidence before independent audit.

## Handoff

Return exact roots, runner/input hashes, harness identity, answer-key hash, consumer session identity,
all command evidence and durations, Q1–Q7 score/citations, field reliance, preservation result,
adopt/narrow/stop decision, issue admission/parking, stall/scope-creep risk, and next legal action.

## Result

Not run.
