# AI Development Evaluation Corpus

This directory contains small, deterministic cases that protect the development operating system from silent policy drift.

`routing-cases.v1.json` covers all numeric tiers and every mandatory-Ultra reason. It does not measure model quality. It verifies that the repository's risk policy, role routing, task packets, and runtime adapter agree before live model evaluations are attempted.

`historical-cases.v1.json` contains eight real CFB/CFR diagnosis, trust-boundary, durability, contract, and completion-audit cases. Each case separates `candidate_context` from an evaluator-only `oracle`. Prepare an input with:

```bash
python3 docs/development/tools/prepare_historical_eval.py --list
python3 docs/development/tools/prepare_historical_eval.py --case cfb-037-failed-command-evidence
```

The prepared JSON contains only the prompt, observations, source reading list, response contract, risk tier, and digests. Do not give the raw corpus file to the candidate model: it contains the reference answer. The digest permits a result record to bind to that answer without revealing it during inference.

Live evaluation should add separate result records containing the model snapshot, runtime adapter, task packet digest, output/patch digest, reviewer decision, validation outcome, latency, and token/cost observations. Do not overwrite an old result when a model or prompt changes.

`live-routing-smoke-2026-07-11.v1.json` records a minimal account-level smoke through Codex CLI 0.144.1. Medium, High, and Ultra each completed a strict-config, read-only GPT-5.6 Sol invocation. This proves live access and tier acceptance on the observation date; it does not compare quality, exercise useful Ultra delegation, or replace the future scored historical cohort.
