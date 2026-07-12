# 2026-06-14 Example Success Record

This is a template-quality example of a successful experiment record. It is not a real SOTA result and must not be cited as evidence for an external claim.

## Goal

Show how to record a small successful experiment in a way that future agents can parse and review.

## Hypothesis

A bounded planner can choose one high-information experiment from a short candidate list when each candidate has a cost estimate and expected information value.

## Candidate

- candidate id: `example-candidate-success`
- lineage: synthetic example
- mechanism: rank candidates by information per cost, then choose the highest feasible candidate

## Baseline

- baseline id: `example-baseline-static-order`
- behavior: choose the first candidate that fits the budget

## Benchmark

- benchmark id: `example-local-selection-fixture`
- dataset: in-memory fixture
- version: example only

## Metric

- metric: selected candidate information value
- direction: higher is better

## Command

```bash
PYTHONPATH=src python3 -m unittest tests.test_batch_planning -v
```

## Artifacts

Example artifact paths:

- `artifacts/example-success/stdout.txt`
- `artifacts/example-success/stderr.txt`
- `artifacts/example-success/metrics.json`
- `artifacts/example-success/config.json`

These files are illustrative. A real experiment record must point to actual captured artifacts.

## Configuration

```json
{
  "budget": 10,
  "candidate_count": 3,
  "selection_policy": "information_per_cost"
}
```

## Result

The bounded planner selected the candidate with the highest feasible information-per-cost score.

## Analysis

The result supports the local fixture behavior only. It does not show external benchmark improvement, algorithmic novelty, or SOTA performance.

## Failure Mode

None for this example. A real record should still state that no failure occurred and why.

## Evidence

Example evidence:

- unit test output
- metrics artifact
- configuration capture

## Review Objections

- This is not an external claim.
- This does not compare against a real ML benchmark.
- This should not be promoted beyond fixture-level validation.

## Next Action

`promote`: use the pattern for real fixture-level planning experiments, but require real artifacts before using it as evidence.

