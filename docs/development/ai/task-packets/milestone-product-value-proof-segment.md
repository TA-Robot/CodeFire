# AI Task Packet

Status: ready

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

