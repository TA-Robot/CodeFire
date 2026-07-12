# 2026-06-14 Example Negative Record

This is a template-quality example of a negative experiment record. It is not a real experiment result.

## Goal

Show how to record a failed or unhelpful experiment so future planners can avoid repeating it.

## Hypothesis

Increasing exploration weight without checking negative-result overlap will produce more novel candidate proposals.

## Candidate

- candidate id: `example-candidate-negative`
- lineage: synthetic example
- mechanism: increase exploration score while ignoring archive overlap

## Baseline

- baseline id: `example-baseline-negative-aware`
- behavior: penalize candidates that overlap known negative results

## Benchmark

- benchmark id: `example-frontier-memory-fixture`
- dataset: in-memory fixture
- version: example only

## Metric

- metric: useful novel proposals
- direction: higher is better

## Command

```bash
PYTHONPATH=src python3 -m unittest tests.test_novelty -v
```

## Artifacts

Example artifact paths:

- `artifacts/example-negative/stdout.txt`
- `artifacts/example-negative/stderr.txt`
- `artifacts/example-negative/metrics.json`
- `artifacts/example-negative/config.json`

These files are illustrative. A real negative record must point to actual captured artifacts.

## Configuration

```json
{
  "exploration_weight": 1.5,
  "negative_overlap_penalty": 0.0,
  "archive_lookup": false
}
```

## Result

The policy proposed candidates that overlapped known dead ends. The apparent novelty came from ignoring archive memory rather than finding useful new directions.

## Analysis

The failure mode is policy design, not benchmark execution. The planner needs negative-result overlap checks before increasing exploration pressure.

## Failure Mode

`invalid-hypothesis`: the hypothesis assumed exploration pressure alone would produce useful novelty.

## Evidence

Example evidence:

- test output showing overlap is detected when archive lookup is enabled
- configuration showing archive lookup disabled
- negative archive entry naming the repeated dead-end mechanism

## Review Objections

- Do not rerun this policy without archive lookup.
- Do not treat proposal diversity as useful novelty unless overlap is checked.
- Do not use this as evidence against exploration generally.

## Next Action

`archive-negative`: preserve this avoid condition and require archive overlap checks for future exploration policy changes.

