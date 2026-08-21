# AI Task Packet

Status: completed

Packet ID: MS-agent-local-contract-seam

Schema version: human-v1

Risk score: 3

Reasoning tier: high

## Objective

Classify whether the smallest plausible M1 local-contract fix actually requires a CFR-169 module/test ownership seam, or whether CFR-169 must remain parked while CFR-170/CFB-133 are handled in existing owners.

This was node 2 of completed milestone `MS-agent-local-contract`. It ran read-only in parallel with the clean-room public-contract probe.

## Inputs and questions

Read CFR-169, CFR-170, CFB-133, `module-boundaries.md`, `source-architecture.md`, `command-source-trace.md`, the completed prior loop packets, and only the source/test files necessary to locate current ownership.

Answer:

1. Which modules own JSON envelopes, `next_actions`, help rendering, migrate postconditions, and command argument completion?
2. Can the likely smallest contract correction stay inside existing owners with focused tests?
3. Would touching CFR-169 now shorten the M1 critical path, or create an architecture/refactor detour?
4. If a seam is unavoidable, what is the smallest exact file boundary and deterministic focused test? If it is avoidable, classify CFR-169 `park` and stop.

Do not design a broad refactor, implement code, expand tests, or promote a parked remote/adoption/release root.

## Scope

Read-only ownership mapping plus this packet's Result. Source reading is limited to files needed to locate the existing contract owners. The only writable tracked file is this packet.

## Non-Goals

- implementing or designing a broad CFR-169 refactor;
- changing issue status or admitting CFR-169 before evidence;
- evaluating remote, adoption, release, or product-economics work;
- running build/test gates for unchanged product code;
- duplicating the clean-room public-contract probe.

## Validation

Change Status to `completed` and record `block-m1`, `park`, or `drop` with evidence. Run:

```bash
python3 docs/development/tools/check_project_docs.py
git diff --check
```

No product/dogfood/state write, build, install, full test suite, or issue-status change is allowed.

## Handoff

- ownership map: JSON envelopes and shared `next_actions` builder in `automation.rs`; migrate postconditions in `migration.rs`; top-level help and shell completion in `completion.rs` + `command_registry.rs`; per-command help strings remain dispatch-adjacent in `main.rs::subcommand_help`. `main.rs` renders envelopes; it does not own envelope shape or action reduction.
- CFR-169 classification: `park`. It is real later architecture work (lifecycle/Store/test extraction in `main.rs` / `remote.rs`) and does not sit on this M1 critical path.
- smallest exact write set if unavoidable: none. A later admitted contract fix stays in the existing owners above; do not open a CFR-169 extraction.
- focused tests already owning that surface: `automation.rs` `#[cfg(test)]`, `tests/docs.rs`, `tests/migration_apply.rs`, plus `tests.rs::cli_error_next_actions_are_contextual`.
- integration/conflict risk: node 1 may reproduce a CFR-170 string/`argv`/placeholder gap and/or the already-coded CFB-133 migrate slice. Those live in `automation.rs` / `migration.rs`, not in the CFR-169 residuals. Write sets stay disjoint.
- next allowed action after fan-in: node 3 orchestrator admission of the smallest *reproduced* contract slice into existing owners. Do not implement here. Do not admit CFR-169 on this evidence.

## Result

Classification: **park**. Exact base: `4185a5551230548da800c311fc60f67656178ec8`. Read-only ownership mapping only; no product, board, issue, or probe duplication.

CFR-169 is not `block-m1` and not `drop`. The architecture residual is real, but the smallest plausible M1 local-contract correction does not require its module/test seam. CFR-170/CFB-133 stay in existing owners.

### 1. Current owners

| Surface | Owner | Evidence |
|---|---|---|
| JSON envelope `codefire.command_result.v1` | `crates/codefire-cli/src/automation.rs` | `COMMAND_RESULT_SCHEMA`, `command_result_envelope`. Canonical trace: "JSON / `automation::command_result_envelope` owns the top-level shape." Public schema `schemas/public/command-result-v1.schema.json` requires `schema/command/ok/exit_code/repo/data/diagnostics/next_actions`; `next_actions` items are untyped objects. |
| Envelope render / error wrap | `crates/codefire-cli/src/main.rs` | Dispatch calls `command_result_envelope`. `print_cli_error_json` → `cli_error_envelope` → same helper. This is composition, not shape ownership. |
| Shared `next_actions` builder | `automation.rs` | `next_action`, `normalize_next_action`, `bounded_next_actions`. Emitted fields: `kind`/`id`, string `command`, `reason`/`description`, `target`/`context`. No `argv`, mode, expected precondition, mutation class, or idempotency/dry-run flags. |
| Local-loop action reducers | `automation.rs` | `status_next_actions_with_prediction`, `scan_next_actions`, `verification_next_actions`. Prior sealed-loop fixture (`milestone-local-sealed-loop-run.md`) shows the current contract: verify success emitted `command: "codefire commit -m <message> --path …"` from `verification_next_actions` (`cli_command(path_command("commit -m <message>", …))`). |
| Migrate postconditions | `crates/codefire-cli/src/migration.rs` | `MigrationReport.{planned,applied,remaining}_actions`; `migration_report_next_actions` branches on mode + current `remaining_actions`. Successful apply/replay with empty remaining returns `[]` even when historical `planned_actions` is non-empty. Trace: "Next actions are derived from mode and the current remaining postcondition, while planned/applied actions remain historical audit fields." Command-source-trace owner is `migration`, not `main`. |
| Top-level help | `crates/codefire-cli/src/completion.rs` | `help_text()`; `main.rs` prints it for no-args / `-h`. |
| Per-command help | `main.rs` | `command_help_for_args`, `subcommand_help` — static usage strings, dispatch-adjacent. Not the CFR-169 lifecycle/object residual. |
| Argument completion + capability list | `completion.rs` + `command_registry.rs` | `completion_script` / `bash_completion_script` / `zsh_completion_script`; `COMMAND_CAPABILITIES`, `COMMAND_NAMES`, `capabilities_data_json`. Trace owners: `completion`, `command_registry`. |
| Architecture map | `source-architecture.md` CLI table | `automation` owns envelopes/diagnostics/bounded `next_actions`; `completion` owns top-level help and shell completion; `command_registry` owns declared public capabilities; `migration` owns inspect/plan/locked apply. |

`issue-root-cause-map.json` already splits the two roots: CFR-169 `owner_module` is `crates/codefire-cli/src/main.rs` (`architecture-proof-seam`); CFR-170 `owner_module` is `crates/codefire-cli/src/automation.rs` (`typed-automation-outcome-actions`).

CFR-169's own Current Source Audit names a different residual: `remote.rs` object-reference traversal / object-directory registry; production init/commit JSON construction in `main.rs`; remaining `main.rs` state/commit/open workflow extraction; test-helper splitting. Those are Store/lifecycle/proof seams. They are not envelope, `next_actions`, help, migrate postcondition, or completion owners.

### 2. Smallest likely contract correction stays in existing owners

The plausible M1 correction, if node 1 reproduces a gap, is CFR-170's remaining typed executable-action work: `argv`/mode/target/precondition/mutation class on `next_actions`, human command as a projection, reducer from current postcondition. That is `automation::next_action` plus the already-separated command reducers (`automation` for status/scan/verify; `migration` for migrate; feature modules for doctor/storage/fire; `main.rs::cli_error_next_actions` as a caller of the shared builder).

CFB-133's migrate apply/replay slice is already implemented in `migration_report_next_actions` and covered by `tests/migration_apply.rs` (`migration_next_actions_follow_mode_and_current_postcondition`, `failed_apply_recovery_action_is_derived_from_remaining_actions`). Independent review/Retrospect/status transition remain; that is not a module-boundary problem.

No new module, Store registry move, `main.rs` workflow extraction, or test-corpus split is required to change the action object or migrate reducer. `module-boundaries.md` already lists `automation` / `completion` / `command_registry` / `migration` as preferred locations. Adding fields to `next_action` does not add a new responsibility to `main.rs`.

### 3. Touching CFR-169 now is an architecture detour

CFR-169 acceptance forbids mixing unrelated functional change into a large behavior-preserving extraction. Opening its first residual (remote object traversal, commit-root builders, lifecycle pull-out of `main.rs`) would widen the write set onto Store/CLI identity paths that the local-contract exit does not name, serialize behind Ultra-review hotspots, and delay the typed-action change that actually feeds M1.

It does not shorten the critical path `probe → admission of the smallest reproduced blocker → focused fix → replay → audit`. The local sealed-loop commands (`status`/`scan`/`extinguish`/`verify`/`commit`) already have envelope/action owners outside that extraction.

### 4. Seam avoidable → `park` and stop

CFR-169 `park`. Not `drop`: the lifecycle/Store/test residual remains valid later work. Not `block-m1`: M1 can proceed in `automation.rs` / `migration.rs` / `completion.rs` / `command_registry.rs` without this seam.

Smallest exact write set if a contract fix is later admitted (not implemented here): `crates/codefire-cli/src/automation.rs` (`next_action` / `normalize_next_action` and status/scan/verify reducers); existing per-command reducers including `migration.rs::migration_report_next_actions`; `schemas/public/command-result-v1.schema.json` only if `next_actions` items become typed; callers such as `main.rs::cli_error_next_actions` updated in place. Moving `subcommand_help` into `completion.rs` is optional composition cleanup and is not the CFR-169 seam.

Focused tests already on those surfaces:

- `automation.rs` `#[cfg(test)]`: envelope shape, bound/omission, legacy normalization, status/scan/verify action tables (`command_result_envelope_has_stable_shape`, `command_result_envelope_normalizes_legacy_next_actions`, `verification_next_actions_*`).
- `crates/codefire-cli/src/tests/docs.rs`: `help_and_completion_scripts_cover_rust_default_cli` and registry/help/completion consistency.
- `crates/codefire-cli/src/tests/migration_apply.rs`: postcondition-driven migrate actions (apply success/replay empty; check→review; dry-run→apply; failed apply from `remaining_actions`).
- `crates/codefire-cli/src/tests.rs::cli_error_next_actions_are_contextual` for error-path actions.

### Probe conflict / next action

Node 1's clean-room probe was not duplicated. Whatever first public-contract gap it records maps onto the owners in §1, not onto CFR-169's Store/lifecycle residuals. Packet write sets remain disjoint. Fan-in: node 3 may admit the smallest reproduced CFR-170 and/or CFB-133 slice in those owners. It must not admit CFR-169 on this evidence. No product code, board, or issue status was changed.
