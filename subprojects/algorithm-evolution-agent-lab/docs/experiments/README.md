# Experiment Log

実験結果をここに記録します。成功した実験だけでなく、失敗・timeout・budget overrun・inconclusive resultも研究資産として残します。

## File Naming

Use one Markdown file per experiment or experiment batch.

Recommended name:

```text
YYYY-MM-DD-<short-topic>.md
```

Examples:

```text
2026-06-14-frontier-budget-ablation.md
2026-06-14-baseline-reproduction-smoke.md
```

## Required Sections

Each experiment note should include:

- goal
- hypothesis
- candidate or algorithm id
- baseline
- dataset / benchmark
- metric and direction
- command
- artifact paths
- configuration summary
- result
- failure mode
- evidence refs
- review objections
- next action

## Template

```markdown
# YYYY-MM-DD Short Topic

## Goal

## Hypothesis

## Candidate

## Baseline

## Benchmark

## Metric

## Command

## Artifacts

## Configuration

## Result

## Analysis

## Failure Mode

## Evidence

## Review Objections

## Next Action
```

## Decision Labels

Use one of these labels in the `Next Action` section:

- `promote`: enough evidence to evaluate more seriously
- `rerun`: execution or environment issue requires another run
- `ablate`: result is promising but mechanism is unclear
- `revise`: candidate needs design changes
- `defer`: valid work, but not current capacity or budget
- `archive-negative`: result should prevent repeated dead-end work
- `block-claim`: evidence is insufficient for external claim

## Reference Examples

The following files are documentation examples, not claimed benchmark results. Use them as formatting references when creating new experiment notes:

- [2026-06-14-example-success-record.md](2026-06-14-example-success-record.md): a positive result record with follow-up evidence requirements.
- [2026-06-14-example-negative-record.md](2026-06-14-example-negative-record.md): a negative result record that preserves why a candidate should not be retried unchanged.
- [experiment_record_patterns.md](experiment_record_patterns.md): reusable record patterns for baseline reproduction, ablation, replication, negative results, claim review, handoff, and documentation audits.

When adding a real experiment note, preserve the same separation between observed result, interpretation, review objection, and next action. Do not turn a promising result into a claim until the evidence and claim review guide says the release gate is satisfied.
