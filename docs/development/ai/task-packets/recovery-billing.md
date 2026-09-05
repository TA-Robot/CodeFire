# REC-04: Verify subscription-only Grok execution

Status: completed; independent review passed

## Objective

Prove current Grok account has no paid overflow source without invoking a model. Fail closed on unknown state.

## Scope

`grok_subscription_preflight.py`, its offline tests, and the integration test import. No credentials, raw billing replies, account changes, paid calls or alternate API routes are permitted.

## Evidence

Installed Grok Build 1.0.13 responded to ACP initialize, `_x.ai/billing`, `_x.ai/auto-topup-rule` without session/prompt. At 2026-09-05 10:46 UTC, allowance used 23%, purchased credits zero, PAYG cap/used zero, auto-topup disabled. Standard effective config had no auth/model/endpoint overrides; provider env overrides absent. The model route is built-in grok-4.6 on the existing OAuth subscription.

## Validation

```bash
python3 docs/development/tools/grok_subscription_preflight.py
python3 -m unittest docs/development/tools/test_grok_subscription_preflight.py
```

Run the first command immediately before and after each bounded external job. A failure blocks launch, not merely a warning. Preserve the status JSON, never the raw replies. Check effective model/auth configuration again if CLI/config/environment changes. No cached admission and no silent fallback.

Independent native review found and reverified a fix for a process-group cleanup defect: a descendant ignoring TERM could survive the probe leader. Cleanup now kills the remaining group even after leader exit; a real descendant regression independently passes. Offline billing tests: 11 passed; combined tool tests: 72 passed. No inference is sent by the probe.

## Non-Goals

A general scheduler, account mutations, subscription purchases, Fable calls, API endpoints with keys, or atomic protection against billing settings being changed externally. The provider offers no verified per-request subscription-only switch; the confirmed zero/off account settings govern quota exhaustion.

## Handoff

Account is verified for bounded official CLI use with before/after checks. Routine jobs use Grok High; bootstrap recovery correctness uses xhigh. Disable subagents and web tools, name exact write set and turn bound, preserve checkpoints and use independent review. Legacy dev-agent/dev-swarm remain disabled for this recovery.
