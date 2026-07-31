# CFR-168 Fugu Quota Review Interruptions

Date: 2026-07-31  
Disposition: provider quota; prior infrastructure-cause attribution corrected  
Provider/model: Sakana Fugu / `fugu-ultra-v1.1`  
Product or review verdict: none

## Summary

Four Fugu review/research lanes stopped at approximately 08:41–08:43 UTC.
Every event log terminates with the same provider message:

```text
You've hit your usage limit. Try again at Aug 3rd, 2026 12:00 AM.
```

The displayed reset has no timezone. This incident does not convert it to UTC
or assume that capacity will return at a particular instant.

Earlier manager notes and two later Claude reports described these attempts as
model-refresh, process-creation, or generic tool failures. Direct inspection of
the four complete JSONL logs does not support that as the terminal cause:

- the only top-level `error` and `turn.failed` cause is the usage-limit
  message;
- searches for `codex_models_manager` and `missing field models` in the Cargo
  log match repository documentation being read by the reviewer, not a
  top-level runtime error;
- searches for `CreateProcess` and `No such file or directory` match
  reasoning about suspected command issues or ordinary command output, not a
  top-level runtime error; and
- each lane successfully completed dozens of command executions before quota
  termination.

The corrected classification is **Fugu provider quota**. It is not a product
finding, authentication failure, API-key-file failure, or proven process-spawn
failure.

## Transport Verification

Read-only inspection, without printing a secret, established:

- `/usr/local/bin/fugu` dispatches to `/workspace/scripts/fugu`;
- `/workspace/scripts/fugu` invokes Codex CLI `0.146.0` as a transport;
- the wrapper sets `model_provider="sakana"`;
- the base URL is `https://api.sakana.ai/v1`;
- the environment-key name is `SAKANA_API_KEY`;
- the default model is `fugu-ultra-v1.1`; and
- `/workspace/project/fugu-api` is present and non-empty.

The quota therefore occurred on the configured Sakana transport. The presence
of an unrelated `OPENAI_API_KEY` environment-variable name does not show that
these Fugu calls used the OpenAI provider; the invocation explicitly overrides
the provider.

## Exact Preserved Lanes

All four worktrees were clean at inspection. No authorized final report was
created by these attempts.

| Lane | Exact head | Session | Completed commands | Event log size | Event log SHA-256 |
|---|---|---|---:|---:|---|
| Review-recursion root-cause audit | `1409290b1df276936d522008fedc240a0fe085be` | `019fb733-dc67-7d93-b8e0-dfd71d5d6646` | 64 | 1,435,531 | `0985e3dd560c88e59c86c11030726466b78e0371fda231e0005b6bff10c9b3ab` |
| Source-keyed Cargo-gate checkpoint rereview | `bfbbc2b2bdc22f88679e5e1dc794665d863304a1` | `019fb71c-628c-7412-819e-2635c72ff8bb` | 198 | 5,837,565 | `263b006dc00f3236feaf204a92700fef7c3d899bf7c9dfa09128016f2ae2b4b2` |
| KGI/KPI round-two rereview | `0944bf3deba9e753132b3f71716b5c009a8fc30f` | `019fb737-1ed8-7be2-825d-dd8bd310c6a2` | 66 | 948,954 | `291c249a1ffe4d6d608245829e68bccd94480719607848d2f95c8d82619eb079` |
| Upload second closure rereview | `02e04c7e6458035d9c7d58de4d30b4a12e7cd594` | `019fb72f-f312-7d82-89f7-c6ca694d95a2` | 44 | 582,936 | `4098bc00d0e2940a156dffbcc8ecf68d15556eadd23641c0e53ed34e45ddbc2d` |

Event logs:

```text
/workspace/worktrees/cfr168-p0-canonical/.fugu-second-agent/10631d79a0155a0632b4d57b27dfb37cf0c130e38291153ef5b37f8893aa708c/agents/cfr168-review-recursion-root-cause-audit-fugu/logs/events.jsonl
/workspace/worktrees/aia-source-keyed-cargo-gate-base/.fugu-second-agent/309658cdb004fc0b743a2ee3c1e3041f34fa1a14523a3abf8ca12716a6a3aff5/agents/aia-source-keyed-cargo-gate-checkpoint-rereview-fugu/logs/events.jsonl
/workspace/worktrees/cfr168-p0-canonical/.claude-second-agent/10631d79a0155a0632b4d57b27dfb37cf0c130e38291153ef5b37f8893aa708c/worktrees/development-flow-kgi-kpi-zero-residual-repair-claude-20260730/.fugu-second-agent/e28f2fc56adbafe65c4458c8cb8bf3c09845823caff796f12fa0835fed78fe6d/agents/development-flow-kgi-kpi-r2-rereview-fugu/logs/events.jsonl
/workspace/worktrees/cfr168-python-http-upload-zero-residual-repair-base/.claude-second-agent/5cecc4984d163f4eedc2809fe8bbe2c69e79e2cadcbd5d123667ee8b01641505/worktrees/cfr168-python-http-upload-zero-residual-repair-20260730-upload-zero-residual-repair/.fugu-second-agent/4183ba45f6e93d86a1debb235622b22ab06736bae82acaf63112d2ad7e857acf/agents/cfr168-python-http-upload-second-review-closure-rereview-fugu/logs/events.jsonl
```

## Supersession

Do not resume these four sessions after quota recovery:

- the root-cause audit was completed independently by Claude and integrated at
  `abfee6e`;
- the Cargo-gate review was completed independently by Claude and integrated
  at `b589864`;
- the KGI/KPI review was completed independently by Claude and integrated at
  `5760c98`; and
- the Upload review was completed independently by Claude and integrated at
  `b3de0d7`.

The interrupted Fugu reasoning is not review evidence and must not be merged
with those reports.

## Backoff And Recovery

1. Keep Fugu closed to new work before the provider-displayed reset.
2. At or after that time, run one minute-scale, report-free health probe on
   exact model `fugu-ultra-v1.1`.
3. Treat another limit response as continued backoff; do not retry-loop.
4. On success, record the probe command, model, return code, event-log digest,
   and secret-free response.
5. Start a fresh session for the review-root-cause reliability classifier; do
   not reuse any superseded session above.
6. Do not automatically fall back to Codex.

