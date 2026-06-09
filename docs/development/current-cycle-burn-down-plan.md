# Current Cycle Burn-Down Plan

この文書は、直近のcodefire改善サイクルで発見したissue batchを、次のissue探索へ進む前に全部修正するための計画である。

## Policy

今後のcodefire改善サイクルは、次の順序を固定する。

1. Discovery: algorithm dogfoodとCodeFire source reviewでissue batchを作る。
2. Freeze: batch範囲を凍結し、修正対象issue IDを確定する。
3. Burn-down: batch内issueを全部fixedまたは明示wontfixにする。
4. Install: 修正版CodeFireをinstallし、実コマンドとして確認する。
5. Dogfood confirmation: batch内issueの再現コマンドが通ることをalgorithm projectで確認する。
6. Close audit: backlog、detail issue、history、version planが一致していることを確認する。
7. Next discovery: batchが0件になってから次のissue出しへ進む。

Burn-down中に別の問題を見つけた場合の扱い:

- batch内issueの修正を妨げるblockerは、同じbatchへ追加し、そのbatchで消す。
- blockerでない周辺改善は、CFB/CFR採番せず `next discovery candidate` として短くメモする。
- 次の正式issue batchは、現在batchが全部閉じた後に作る。

この運用により、「issueを出す -> 全部消す -> issueを出す」を崩さない。

## Active Batch

Active batch ID:

- `BATCH-2026-06-cycle3-dogfood`

Scope:

- algorithm cycle 3 dogfoodingと、その直後の2周目改善サイクル中に見つけた `CFB-086..CFB-092`。
- 古いopen CFB/CFRはこのbatchの対象外。ただし、同じroot causeで同時に潰せるものは修正時にclose候補として扱ってよい。

Completion rule:

- `CFB-086..CFB-092` の全件が `fixed` または明示 `wontfix` になっている。
- `wontfix` は原則使わない。仕様として残す場合だけ、理由と代替運用を書く。
- `cargo fmt --check`、`cargo test --workspace`、`cargo clippy --workspace --all-targets -- -D warnings` が通る。
- 修正版をinstallし、algorithm projectで該当再現コマンドを再実行する。
- `docs/development/bug-backlog.md` と `docs/development/bug-issues/cfb-*.md` が一致している。

## Issue Status

| Issue | Status | Root fix group | Current decision |
|---|---|---|---|
| CFB-086 | fixed | Evidence/extinguish JSON | fixed済み。close auditで再確認する |
| CFB-087 | open | Evidence/extinguish JSON failure envelope | このbatchで修正する |
| CFB-088 | fixed | Evidence dry-run JSON | fixed済み。close auditで再確認する |
| CFB-089 | open | Debug/read-only command path/json contract | このbatchで修正する |
| CFB-090 | open | Bounded context graph metadata | このbatchで修正する |
| CFB-091 | fixed | Explicit atom span stability | fixed済み。close auditで再確認する |
| CFB-092 | open | Extractor/hash schema migration | このbatchで修正する |

Remaining open count for this batch:

- 4 issues: CFB-087, CFB-089, CFB-090, CFB-092.

## Root Fix Groups

### Group 1: Evidence/extinguish JSON failure envelope

Issues:

- CFB-087.
- Audit already-fixed CFB-086 and CFB-088 at the same time.

Problem:

- `extinguish --evidence-ref <missing> --dry-run --json` can fail before the JSON command_result envelope is rendered.
- This breaks automation recovery because missing evidence cannot be handled as structured diagnostics.

Implementation target:

- `crates/codefire-cli/src/main.rs`
- `crates/codefire-cli/src/extinguish_ux.rs`
- `crates/codefire-cli/src/batch.rs`
- `crates/codefire-cli/src/automation.rs`
- `crates/codefire-cli/src/tests.rs`

Plan:

1. Make evidence-ref validation return a typed CLI error that keeps missing evidence ID, repo path, command, and remediation target.
2. Ensure JSON mode wraps that error in `codefire.command_result.v1` with `ok:false`, nonzero `exit_code`, and diagnostic `kind:"missing_evidence_ref"`.
3. Keep text mode readable and non-JSON.
4. Add tests for single extinguish, all-matching extinguish, and batch extinguish if they share validation.
5. Re-run CFB-086 and CFB-088 regression tests to ensure existing fixes remain stable.

Acceptance:

- Missing evidence-ref dry-run returns JSON envelope, not plain text.
- Diagnostic includes the missing evidence ID.
- `next_actions` recommends creating evidence or removing the bad evidence-ref.
- Existing successful evidence-ref extinguish still returns `has_evidence:true`.

### Group 2: Debug/read-only command path/json contract

Issues:

- CFB-089.
- Possible related close candidates: CFB-041, CFB-042, CFB-058, CFB-083 if the shared command parser work naturally covers them.

Problem:

- `atom-index --path . --json` and `missing-links --path . --json` treat `--path` as a positional filesystem path.
- These commands look like normal CodeFire commands in help, but do not use normal path/json parsing.

Implementation target:

- `crates/codefire-cli/src/main.rs`
- `crates/codefire-cli/src/automation.rs`
- `crates/codefire-cli/src/tests.rs`

Plan:

1. Add a small shared parser for read-only debug commands: `--path`, `--path=<path>`, positional path, `--json`, and `--metrics` handling.
2. Route `atom-index` through the same repo/open discovery path used by status/scan where possible.
3. Route `missing-links` through verification data and return a command_result envelope in JSON mode.
4. Make unsupported flags fail as JSON envelopes when `--json` is present.
5. Keep text output compatible for current human use.

Acceptance:

- `codefire atom-index --path . --json` succeeds from algorithm open dir and returns command_result envelope.
- `codefire missing-links --path . --json` succeeds from algorithm open dir and returns command_result envelope.
- Both commands include `repo` and stable `data.type`.
- Invalid flags with `--json` return `ok:false` envelope, not plain text.

### Group 3: Bounded context graph metadata

Issues:

- CFB-090.
- Possible related close candidates: CFB-073 and CFB-075 if count/omission metadata is generalized.

Problem:

- `context --changed --limit N --json` can return `trace_links` whose endpoints are omitted from the bounded atom list.
- Consumers cannot tell whether the link endpoint is missing, dangling, or simply omitted by limit.

Implementation target:

- `crates/codefire-cli/src/context.rs`
- `crates/codefire-cli/src/automation.rs`
- `crates/codefire-cli/src/tests.rs`

Plan:

1. Track returned atom IDs after bounded atom selection.
2. For each trace link, annotate endpoint presence: `returned`, `omitted`, or `missing`.
3. Add summary counts: total atoms, returned atoms, omitted atoms, total trace links, returned trace links, links with omitted endpoints.
4. Add a `next_actions` entry that preserves `--path` and suggests a higher limit or unbounded context command.
5. Keep existing fields stable; add metadata instead of removing current `trace_links`.

Acceptance:

- Bounded context JSON distinguishes omitted endpoints from missing endpoints.
- Summary counts are machine-readable.
- Existing `context --changed --json` consumers continue to work.
- Tests cover a trace link to an omitted atom.

### Group 4: Extractor/hash schema migration

Issues:

- CFB-092.

Problem:

- Changing atom extraction/hash behavior can create massive `atom_changed` fires even when source files did not change.
- CodeFire cannot distinguish user source changes from tool/schema migration changes.

Implementation target:

- `crates/codefire-core/src/lib.rs`
- `crates/codefire-cli/src/main.rs`
- `crates/codefire-cli/src/automation.rs`
- `crates/codefire-cli/src/tests.rs`
- Store/object metadata under `.codefire/objects`.

Plan:

1. Add explicit extractor/hash schema version metadata to atom indexes and sealed commits.
2. During scan, compare the current extractor/hash schema version with the base commit schema version.
3. If versions differ and file content fingerprints are unchanged, classify the diff as `tool_schema_changed` instead of normal `atom_changed`.
4. Add a rebaseline workflow:
   - dry-run reports affected atoms and version transition.
   - apply records a migration/rebaseline commit without requiring per-edge evidence for unchanged source.
5. Keep normal fire behavior when source content changed.
6. Add tests for source-unchanged extractor upgrade and source-changed extractor upgrade.

Acceptance:

- Source-unchanged extractor/hash schema upgrades do not require 100+ manual extinguish operations.
- Scan reports the migration explicitly.
- Rebaseline commit records old/new schema version.
- Source changes under the new schema still create normal fires.

## Execution Order

1. Group 1 first: it improves failure envelopes needed by later automation.
2. Group 2 second: it normalizes debug/read-only command access.
3. Group 3 third: it improves bounded context used during dogfooding.
4. Group 4 last: it is the largest state/model change and needs the cleanest test surface.
5. Close audit: update all issue detail files, backlog rows, history, and this plan.
6. Install and dogfood: install to `/home/devuser/.local/bin/codefire`, then run algorithm project confirmation commands.

## Confirmation Commands

CodeFire:

```bash
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
./install.sh --prefix /home/devuser/.local
which codefire
codefire --version
```

Algorithm dogfood:

```bash
codefire status --path subprojects/algorithm-evolution-agent-lab --json
codefire scan --path subprojects/algorithm-evolution-agent-lab --json --metrics
codefire extinguish --path subprojects/algorithm-evolution-agent-lab FIRE-does-not-exist --evidence-ref CF-EVIDENCE-does-not-exist --dry-run --json
codefire atom-index --path subprojects/algorithm-evolution-agent-lab --json
codefire missing-links --path subprojects/algorithm-evolution-agent-lab --json
codefire context --changed --path subprojects/algorithm-evolution-agent-lab --limit 5 --json
```

The exact fire ID for the CFB-087 missing evidence test can be replaced with a real open fire from a controlled fixture or CLI unit test. The important requirement is that missing evidence-ref failures are structured JSON failures.

## Done Definition

The batch is done only when all are true:

- CFB-086..CFB-092 have no `open` status.
- Fixed issue detail files include `Fix` and `Verification` sections.
- `bug-backlog.md` summary rows match detail files.
- `current-cycle-burn-down-plan.md` marks remaining open count as 0.
- Full Rust checks pass.
- Installed CodeFire passes algorithm dogfood confirmation.
- Git is clean and pushed.

