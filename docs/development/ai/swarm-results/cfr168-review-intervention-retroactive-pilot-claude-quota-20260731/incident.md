# CFR-168 Review-Intervention Pilot Claude Quota Incident

Date: 2026-07-31  
Disposition: provider incident; no pilot decision  
Provider/model: Claude / `claude-opus-5`  
Task packet commit: `cdaee6ad3b5e497f6f872bfe76783d0e26e8df85`  
Session: `46a9248f-c888-44b1-9adc-5a9477c85e25`

## Summary

The preregistered review-intervention pilot stopped immediately after its
calibration analysis was frozen and before either lineage holdout or temporal
holdout was opened/scored.

The terminal provider message was:

```text
You've hit your org's monthly spend limit · run /usage-credits to ask your admin for a higher limit
```

The structured event immediately before termination reported a rejected
`five_hour` rate limit with overage disabled. These two provider surfaces use
different labels. This record preserves both and does not infer which billing
window is authoritative.

This is not:

- a result for I3, I1, I4, or I5;
- evidence for or against any template shape, field count, checklist length,
  or repair-delta boundary;
- permission to read the holdout and tune calibration retroactively;
- a product, review, or integration verdict; or
- a reason to fall back automatically to Codex.

## Exact preserved state

| Item | Value |
|---|---|
| Agent worktree | `/workspace/worktrees/cfr168-p0-canonical/.claude-second-agent/10631d79a0155a0632b4d57b27dfb37cf0c130e38291153ef5b37f8893aa708c/worktrees/cfr168-review-intervention-retroactive-pilot-claude-20260731` |
| Worktree HEAD | `cdaee6ad3b5e497f6f872bfe76783d0e26e8df85` |
| Worktree state | clean; no pilot report path and no task commit |
| Event log | `/workspace/worktrees/cfr168-p0-canonical/.claude-second-agent/10631d79a0155a0632b4d57b27dfb37cf0c130e38291153ef5b37f8893aa708c/agents/cfr168-review-intervention-retroactive-pilot-claude-20260731/logs/events.jsonl` |
| Event-log size | `1,065,522` bytes |
| Event-log SHA-256 | `af6baeac543f716784c4be65cc9c2bf6b991860c30f8b6f89394427ec586e204` |
| Calibration freeze event UUID | `e7ee30a0-adc5-4a7a-8589-52b76a5bbdb4` |
| Calibration freeze file at interruption | `/tmp/pilot/calibration-frozen.md` |
| Calibration freeze size | 384 lines |
| Calibration freeze SHA-256 | `4d57c323b7c20015c9cd388c4175c973631a67839d8e3d0cc427e58a7cec87d9` |
| Freeze timestamp | `2026-07-31T09:43:17Z` |
| Provider error status | `429` |
| Provider call cost reported by wrapper | `7.0558495 USD` |

The complete frozen calibration text is also embedded in the event-log tool
call carrying freeze event UUID
`e7ee30a0-adc5-4a7a-8589-52b76a5bbdb4`. Therefore loss of `/tmp` alone does
not erase the calibration boundary.

## What had and had not happened

Completed before the provider refusal:

1. all preregistered task, audit, report, and holdout-selection digests were
   recomputed;
2. the calibration corpus was reconstructed from Git objects;
3. I3, I1, I4 E1–E9, and I5 calibration measurements were computed;
4. calibration conclusions were written to one file; and
5. that file was content-frozen before holdout access.

Not completed:

1. lineage-holdout scoring;
2. temporal-holdout scoring;
3. common calibration/holdout comparison;
4. threats-to-validity and intervention decision labels;
5. the authorized `pilot.md`;
6. repository validation and report-only commit; and
7. independent checking of a final pilot report.

Any calibration numbers visible in the provider log are incomplete evidence
until the preregistered holdouts are scored. They must not be promoted
piecemeal into policy.

## Resume contract

When Claude capacity returns:

1. resume the same session and worktree;
2. verify the event-log prefix size and SHA-256 above;
3. recover `/tmp/pilot/calibration-frozen.md` directly, or extract its exact
   text from freeze event UUID
   `e7ee30a0-adc5-4a7a-8589-52b76a5bbdb4`;
4. verify SHA-256
   `4d57c323b7c20015c9cd388c4175c973631a67839d8e3d0cc427e58a7cec87d9`;
5. do not alter calibration conclusions;
6. open and score the preregistered lineage holdout, then the temporal
   holdout;
7. finish the exact packet output and validations; and
8. commit only the authorized `pilot.md`.

If the freeze digest cannot be reproduced, restart the entire pilot from its
task base. Do not continue from a reconstructed or edited calibration.

Fugu and Codex are not authorized fallbacks for this incident. Fugu remains
under separate infrastructure backoff, and the low-Codex policy reserves
Codex for control-plane integration and conflicting evidence rather than
provider-quota substitution.
