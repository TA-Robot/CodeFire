# REC-03: Repair native/external runtime consistency

Status: implemented; awaiting independent review

Milestone: MS-recovery-baseline

## Objective

Make runtime verification correctly distinguish native Codex roles from external Grok/Claude profiles, preserve strict invalid-configuration rejection, and support the installed CLI adapter.

## Scope

Owner: current-session Codex bootstrap worker, explicitly admitted by the 2026-09-05 recovery policy.
Write set: .codex/; docs/development/ai/role-routing-matrix.json; docs/development/ai/agents/; docs/development/tools/validate_ai_runtime.py; docs/development/tools/check_ai_acceleration_docs.py; docs/development/tools/run_ai_role.py; docs/development/tools/run_milestone_audit.py; relevant runtime tests; this packet.

## Non-Goals

External model calls, API authentication, product source, provider-routing-matrix.json, governance prose, base scripts.

Policy: Grok 4.6 primary external worker; Fable 5.1 scarce escalation only; current-session Codex supports recovery bootstrap; no metered billing. Native Codex config must never point at non-Codex config files. Keep default Codex model gpt-5.6-sol pin for CLI compatibility unless factual local evidence requires a change; this does not change the active host session model.
Invariants: reject malformed/provider-mismatched roles; validate CLI capability without inference; do not delete checks just to turn gate green. Understand current tests before editing.

## Validation

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest docs/development/tools/test_ai_runtime.py`: 22 offline regression tests pass, including the final execute guard.
- `python3 docs/development/tools/validate_ai_runtime.py --json`: passes with installed `codex-cli 0.153.0`, real strict parser probes and bundled catalog; no model request.
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest docs/development/tools/test_ai_tools.py`: all 61 tests pass, including the 21 runtime regressions.
- `python3 docs/development/tools/check_ai_acceleration_docs.py --json`: passes after both recovery packets use the required sections.
- `git diff --check`: passes. Do not alter unrelated policy checks to hide failures.

## Handoff

Native config contains only Codex manager/architect. External roles use Grok profiles with explicitly selected Fable alternates. The validator checks native/external containment, provider/model consistency, malformed TOML/JSON roles, alternate profiles and installed CLI capabilities. The policy checker preserves mandatory validation while following the accepted Grok-first, subscription-only policy. Dedicated regression tests are imported into the existing full-tool test entrypoint.

`run_ai_role.py` rejects external roles before probing capabilities, even in dry-run, and command construction independently rejects them. Native manager/architect command construction remains intact. The audit prompt now follows event-triggered fresh-session review, Grok default, scarce Fable opt-in and the explicit native recovery exception. The obsolete unreferenced Claude Opus 5 profile is replaced with the Fable profile; historical Git retains it.

REC-06 readiness review additionally admitted a fail-closed guard: `run_ai_role.py --execute` is disabled during subscription-only recovery, even with `--allow-codex`, until a verified Codex billing preflight exists. This check runs before repository, capability or CLI probes. Native-role dry-run and the current subscription session remain available. An offline regression covers both manager and architect with every applicable explicit execution override.

Active milestone: MS-recovery-baseline. This packet is on the critical path, parallel to REC-02. Required because current runtime configuration did not agree with provider routing. Parked: broader scheduler/backend work and product changes. Next allowed action: independent REC-05 review, then integrated M1 gates. No commits: integrator owns shared branch commits.

## Independent Review

Grok 4.6 resumed its interrupted read-only review and returned explicit ACCEPT (`end_turn`, five focused tests).
The subsequent native reviewer independently checked the hard `--execute` refusal and all 22 runtime tests.
No block-m1 remains for runtime routing. Three future-hardening suggestions are parked: enforce model pairing
inside native dry-run command construction too; reject a future Fable-primary role edit; validate presently
unconsumed legacy-entrypoint enable flags. Current matrices are Grok-primary and those flags false.
The earlier cancelled Grok run is not counted as approval. Full final integration gate remains required.

## Final integration check

`check_ai_development.py --timeout 60` passed all seven groups on the integrated working tree. An earlier
aggregate run reported runtime failure while the long Rust suite was running; a separate installed-CLI
validation returned no findings. The final aggregate uses the explicit 60-second per-group budget. The
earlier aggregate failure is not claimed to have been a configuration error or silently counted as success.
