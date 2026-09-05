# Subscription-only Provider Routing

Status: active development policy

Effective: 2026-09-05

Owner: Orchestrator

Update trigger: user budget constraints, measured quota, or a verified runtime capability changes.

The user accepted recovery and reports Grok has substantial capacity while Fable exhausts quickly.
**Grok 4.6 is the primary worker. Fable 5.1 is scarce escalation, never routine polling. Metered billing is forbidden.**

## Selection and budget

- Current-session Codex owns admission and integration. External Codex CLI compatibility remains pinned to `gpt-5.6-sol`; this pin does not describe or change the active host session model.
- Grok High starts ordinary implementation and focused review. Extra High is a deliberate escalation when a task needs more reasoning.
- Fable 5.1 is explicit-only for hard architecture, root causes, or important independent review, after subscription inclusion and available allowance are verified. Do not use it for classification, status polling, log summaries, or automatic fallback.
- No purchased credits, extra usage, automatic top-up, metered API key, gateway, or cloud-billed fallback. Fugu is disabled for this recovery.
- Provider outage/quota exhaustion means preserve and pause, or hand off to a verified subscription lane. Authentication success or `--max-budget-usd 0` is not proof of no billing.
- Check authentication source, effective provider configuration, subscription entitlement, included quota, and overage settings before external model execution. Unknown billing state blocks that lane.
- Current-session native Codex subagents may execute REC-02/REC-03 setup and REC-04/REC-05 inspection/review while external billing is unverified. This user-authorized bootstrap exception does not authorize an external Codex API route or self-review.

## Runtime authority

`provider-routing-matrix.json` owns the intended routing policy. `role-routing-matrix.json` separates native
Codex and external CLI roles. `.codex/` contains only supported native Codex configuration; a Grok/Claude
profile must never be loaded as a Codex config.

The old `/workspace/scripts/dev-agent` and `dev-swarm` have duplicated Claude-first defaults and no Grok
scheduler backend. They are **disabled entrypoints for this recovery** until independently updated and
validated against this policy. The JSON file is not falsely claimed to control those scripts already.
No automatic workflow may invoke them simply because they exist.
`run_ai_role.py --execute` is also disabled until an external Codex subscription billing preflight exists;
`--allow-codex` alone does not establish billing. Its native-role dry-run remains available.

The first allowed external Grok route is a bounded official CLI session in an orchestrator-created worktree,
with explicit model, turn bound, declared write set, verified subscription-only billing, and captured result.
CLI presence, model catalog, or OAuth login alone is not a billing gate. The current CLI `/usage` is a
read-only source for included allowance and overage information; record only non-sensitive metadata.

## Concurrency and review

Start with one orchestrator and two independent workers. Grok ceiling 2, Claude 1, no Fugu. Reviews reuse
these slots in fresh sessions. Stop admitting new implementation if two reviews are waiting.

Writers own both disjoint paths and shared contract boundaries. A shared large source file has one writer.
Deterministic verification and integrated-head evidence outrank model approval. Product implementation and
review must be independent; an agent never approves its own code. Other-family review is preferred when
verified quota permits, but a separate same-family session is acceptable.

Monitoring checks progress and quota before new dispatch and at 30-minute intervals while work runs.
LLM audits happen at integration, a high-risk boundary, or detected stall, not automatically every 30 minutes.
The actual source and command results must accompany a checkpoint. Preserve main and its sequencer;
automation prepares an integration branch and does not publish or merge main.
