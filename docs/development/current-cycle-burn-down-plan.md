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

- `BATCH-2026-06-global-open-backlog`

Scope:

- 2026-06-09時点でopenだった全既知issue。
- Initial frozen CFB dogfood issue count: 68。
- Initial frozen CFR code review issue count: 78。
- Initial frozen total: 146件を同一burn-down対象としてfreezeする。
- 新規issue探索は、このglobal batchを0件にするまで行わない。作業中に見つけた周辺問題は、修正blockerでない限り採番せず、次回discovery候補として短くメモする。

Completion rule:

- freeze済み146件の全件が `fixed` または明示 `wontfix` になっている。
- `wontfix` は原則使わない。仕様として残す場合だけ、理由と代替運用を書く。
- `cargo fmt --check`、`cargo test --workspace`、`cargo clippy --workspace --all-targets -- -D warnings` が通る。
- 修正版をinstallし、algorithm projectで該当再現コマンドを再実行する。
- `docs/development/bug-backlog.md`、`docs/development/bug-issues/cfb-*.md`、`docs/development/code-review-issues-2026-06-06.md`、`docs/development/code-review-issues/cfr-*.md` が一致している。

## Issue Status

| Issue | Status | Root fix group | Current decision |
|---|---|---|---|
| CFB-006 | fixed | Verification diagnostic filtering | blocking-only viewでnon-blocking missing linksを分離した |
| CFB-012 | fixed | State reducer | verify passed後にfire 0でburning表示を残さない |
| CFB-013 | fixed | Scan/verify text summaries | clean scanはempty stateを明示する |
| CFB-014 | fixed | State reducer | open fire正ならstatusはopen-burningへ補正する |
| CFB-015 | fixed | State reducer | clean verifyはopen-cleanを維持する |
| CFB-016 | fixed | State reducer | open fire正ならstatusはopen-cleanを返さない |
| CFB-022 | fixed | Scan/verify text summaries | passed verifyもblocking summary/countersを表示する |
| CFB-028 | fixed | Verification diagnostic filtering | blocking-only JSON dataをfiltered viewにした |
| CFB-029 | fixed | Next action path context | verify next_actionsに元pathを保持した |
| CFB-040 | fixed | Show JSON envelope | `show --json` をtyped command_resultへ統一した |
| CFB-041 | fixed | List JSON surface | local/remote `list --json` をcommand_result envelopeへ統一した |
| CFB-042 | fixed | Branch JSON surface | `branch show --json` とunsupported branch JSON failureを実装した |
| CFB-044 | fixed | Diff JSON envelope | `diff --json` をcommand_result envelopeへ統一した |
| CFB-052 | fixed | Verify next_actions state awareness | clean passed verifyはcommitではなくstatusを返す |
| CFB-057 | fixed | Show JSON envelope | exit 0でhuman textを返すshow JSON契約違反を解消した |
| CFB-058 | fixed | Branch JSON surface | branch detailをmachine-readable envelopeで取得できるようにした |
| CFB-060 | fixed | Context/explain JSON failure | unknown Atom/fire targetをJSON failure envelopeへ包んだ |
| CFB-062 | fixed | Diff JSON failure | diff target resolution failureをstructured JSONへ包んだ |
| CFB-064 | fixed | Remote read JSON surface | `list <remote> --json` と `request-list <remote> --json` を実装した |
| CFB-065 | fixed | Operation plan envelope | upload dry-run planのrepo/next_actionsをtop-levelへ持ち上げた |
| CFB-068 | fixed | Next action path context | scan next_actionsに元pathを保持した |
| CFB-069 | fixed | Verification diagnostic filtering | blocking-only next_actionsをblocking diagnosticsに限定した |
| CFB-070 | fixed | Operation plan envelope | local dry-run planのrepo/next_actionsをtop-levelへ持ち上げた |
| CFB-071 | fixed | CLI path contract | storage/doctor/migrateが`--path`/`--path=`を受けるようにした |
| CFB-073 | fixed | Context bounded output metadata | context summary countsとcontext_expand next_actionを追加した |
| CFB-074 | fixed | Next action schema | explain/storage/migration/doctorを共通next_action schemaへ寄せた |
| CFB-075 | fixed | Bounded next action recovery | omitted_by_kindとfirst omitted targetを追加した |
| CFB-076 | fixed | Batch dry-run validation | invalid link batch dry-runはtop-level ok:false/exit 2にした |
| CFB-081 | fixed | Explain verification summary | passed warningをblocker扱いしないsummaryへ修正した |
| CFB-083 | fixed | Branch list metrics | `branch list --metrics` がmetrics blockとhelpを返すようにした |
| CFB-084 | fixed | Migrate JSON failure | migrate parser failureをJSON envelopeへ包んだ |
| CFB-086 | fixed | Evidence/extinguish JSON | fixed済み。close auditで再確認する |
| CFB-087 | fixed | Evidence/extinguish JSON failure envelope | fixed済み。missing evidence-ref JSON envelopeを確認した |
| CFB-088 | fixed | Evidence dry-run JSON | fixed済み。close auditで再確認する |
| CFB-089 | fixed | Debug/read-only command path/json contract | fixed済み。atom-index/missing-links path/jsonを確認した |
| CFB-090 | fixed | Bounded context graph metadata | fixed済み。trace endpoint omission metadataを確認した |
| CFB-091 | fixed | Explicit atom span stability | fixed済み。close auditで再確認する |
| CFB-092 | fixed | Extractor/hash schema migration | fixed済み。schema migration detectionとrebaseline commitを確認した |
| CFR-127 | fixed | Explain verification summary | blocker数をdiagnosticsのblocking fieldから計算する |
| CFR-136 | fixed | Context bounded output metadata | omitted countsとtext follow-up commandを追加した |
| CFR-140 | fixed | Verification diagnostic filtering | filtered/unfiltered diagnostic summaryをJSON dataへ追加した |
| CFR-141 | fixed | Verify next_actions state awareness | verify next_actionsをscan-awareにした |
| CFR-142 | fixed | Next action fallback context | bounded omissionとerror fallbackを文脈別next_actionへした |
| CFR-143 | fixed | Next action schema | command_result envelopeでlegacy actionを正規化し、共通serializerを使うようにした |
| CFR-144 | fixed | Explain action rendering | explain text rendererをkind優先/id fallbackにした |
| CFR-151 | fixed | Migration next action schema | migration next_actionsを共通schemaへ移行した |
| CFR-152 | fixed | Storage next action schema | storage next_actionsを共通schemaへ移行した |
| CFR-153 | fixed | Doctor next action schema | doctor next_actionsを共通schemaへ移行した |
| CFR-157 | fixed | Context bounded atom traversal | omitted neighbor countとcontext_expand next_actionを追加した |
| CFR-115 | fixed | Context/explain JSON failure | unknown target errorをtyped JSON diagnosticへ包んだ |
| CFR-138 | fixed | CLI parser consistency | context主要optionのequals形式を受け付けた |
| CFR-139 | fixed | CLI parser consistency | commit/extinguishの`--path=`形式を受け付けた |
| CFR-154 | fixed | Operation plan envelope | plan内next_actionsをtop-levelへ昇格した |
| CFR-155 | fixed | Operation plan envelope | plan内repo/repo_root/open_dirからtop-level repoを解決した |
| CFR-096 | fixed | Automation next_actions | open-clean statusはverify loopを出さない |
| CFR-098 | fixed | Automation next_actions | clean scanはverifyではなくstatus確認へ誘導する |
| CFR-125 | fixed | Batch dry-run validation | invalid link batch dry-run envelopeを失敗扱いにした |
| CFR-126 | fixed | Branch list metrics | accepted `--metrics` を黙殺せずJSON/textへ出力する |
| CFR-085 | fixed | Show JSON envelope | help上の `show --json` 契約を実装と一致させた |
| CFR-086 | fixed | Diff JSON envelope | raw `codefire_diff` を標準command_result envelopeへ包んだ |
| CFB-037 | fixed | Evidence command failure | failed command capture is blocking unless `--allow-failed-command` is explicit |
| CFB-049 | fixed | Evidence command cwd | command capture cwd resolution is documented and returned in JSON |
| CFB-072 | fixed | Evidence dry-run cwd | dry-run plan includes resolved cwd, source, and repo-relative cwd |
| CFR-107 | fixed | Evidence cwd metadata | applied evidence result and payload include resolved command cwd metadata |
| CFR-108 | fixed | Evidence dry-run ID | dry-run uses `evidence_id: null` with `created: false` |
| CFR-109 | fixed | Evidence failed command policy | non-zero/timeout commands fail by default and intentional failed logs are marked |
| CFR-147 | fixed | Evidence dry-run cwd | operation plan resolves cwd before reporting dry-run output |
| CFB-061 | fixed | Storage remote diagnostics | missing remote remains in `remotes[]` with invalid layout warning |
| CFB-082 | fixed | Storage quick largest type | quick mode infers largest object type from ID prefix/subdir |
| CFR-103 | fixed | Storage remote layout | required remote layout validation distinguishes missing from empty |
| CFR-104 | fixed | Storage file collection | optional and required storage paths are separated |
| CFR-106 | fixed | Storage mode coverage | JSON reports quick/full coverage and skipped checks |
| CFR-134 | fixed | Storage quick largest type | largest objects include inferred type and type confidence |
| CFR-135 | fixed | Storage largest top-N | largest object aggregation keeps bounded top-N and reports limit |
| CFB-063 | fixed | Review/patch JSON surface | review-pack and patch export return metadata command_result envelopes |
| CFR-087 | fixed | Review/patch JSON envelope | generated payload metadata is structured without embedding payload body |
| CFR-088 | fixed | Patch export bounded payload | large write content is omitted by default with hash/size metadata |
| CFB-053 | fixed | Scan JSON boundedness | default scan JSON returns counts, samples, and omitted metadata |
| CFB-055 | fixed | Scan changed atom shape | changed_atom_ids and changed_atoms_sample clarify ID vs object shape |
| CFR-097 | fixed | Scan changed atom detail | changed atom sample includes kind, path, selector, and content hash |
| CFB-032 | fixed | Doctor/migrate layout health | auto-creatable layout gaps are non-blocking doctor warnings |
| CFB-033 | fixed | Doctor repair next_actions | repairable layout issues route to `migrate dry-run --json` |
| CFB-034 | fixed | Doctor/migrate layout health | doctor and migrate share layout severity vocabulary |
| CFB-035 | fixed | Doctor/migrate layout health | migration planned actions come from the same layout registry as doctor |
| CFR-132 | fixed | Migration target registry | `current` is default and supported targets are returned in JSON/errors |
| CFR-133 | fixed | Repository layout registry | doctor and migration consume a shared layout definition |
| CFR-148 | fixed | Doctor active state diagnostics | missing active `state.json` is explicitly diagnosed |
| CFR-149 | fixed | Doctor active state invariants | clean/consistent state with open fires is diagnosed |
| CFR-156 | fixed | Migration quick/full scan mode | quick mode skips object integrity and reports skipped checks |
| CFB-077 | fixed | Manual fire identity | manual fire IDs use scan-compatible digest format |
| CFB-078 | fixed | Manual fire state recovery | no-source-change manual fire cleanup returns to open-clean |
| CFR-124 | fixed | Manual fire lock lifecycle | apply reloads active state after acquiring repo lock |
| CFR-145 | fixed | Fire operation plan boundedness | dry-run plan returns bounded sample metadata by default |
| CFR-146 | fixed | Fire operation next_actions | fire plan next_actions include open_dir path context |
| CFR-158 | fixed | Manual fire semantic identity | base commit removed from manual fire semantic key |
| CFB-043 | fixed | Commit JSON blocker envelope | blocked dry-run returns `ok:false` commit result JSON |
| CFB-051 | fixed | Commit dry-run boundedness | changed atoms are counted and sampled by default |
| CFB-067 | fixed | Commit result JSON contract | commit JSON uses typed result data and non-null repo |
| CFR-159 | fixed | Commit certificate vocabulary | certificate result uses verification vocabulary with legacy compatibility |
| CFR-160 | fixed | JSON envelope repo source | commit envelope repo comes from execution context |
| CFB-066 | fixed | Install PATH precedence | install reports and verifies the active codefire path |
| CFB-005 | fixed | Extinguish UX | interactive/all-matching/batch-template reduce repeated rationale and evidence input |
| CFB-011 | fixed | Extinguish batch template | current open fires can generate an `extinguish --batch` JSON wrapper |
| CFB-018 | fixed | Fire volume | batch-template and all-matching collapse repeated evidence entry across many fires |
| CFB-020 | fixed | Extinguish output | success output includes source/target context and remaining open fire count |
| CFB-027 | fixed | Scan/verify next_actions | multiple-fire next_actions include `extinguish --batch-template --json` |
| CFB-010 | fixed | Status scan prediction | `status --json` reports read-only scan prediction counts |
| CFB-026 | fixed | Context changed summary | context data exposes scan-compatible changed/open fire summary fields |
| CFB-054 | fixed | Context freshness | stale active scans fall back to read-only preview context |
| CFR-114 | fixed | Context active scan freshness | context compares active scan fingerprint with current preview scan |
| CFR-137 | fixed | Branch context summary | branch selector includes changed atoms and open-fire endpoints |

Remaining open count for this batch:

- 35 issues.

## Global Issue Inventory

This active batch is the whole known open backlog.

Current tracked issue inventory as of 2026-06-10:

| Series | Total tracked | Fixed / closed | Open | Notes |
|---|---:|---:|---:|---|
| CFB dogfood issues | 92 | 84 | 8 | `docs/development/bug-backlog.md` is the summary source. Detail files exist for CFB-005/010/011/018/020, CFB-023 and later, plus later generated issues; CFB-001..004/006..009/012..017/019/021/022/025 are summary-only legacy entries. |
| CFR code review issues | 160 | 133 | 27 | `docs/development/code-review-issues-2026-06-06.md` and detail files are the summary/detail source. `fixed in v0.7` is counted as fixed/closed. |
| Total | 252 | 217 | 35 | This global batch is the whole known issue backlog. |

Current active batch progress:

- Initial frozen open issue count: 146.
- Fixed during global burn-down so far: CFB-005, CFB-006, CFB-010, CFB-011, CFB-012, CFB-013, CFB-014, CFB-015, CFB-016, CFB-018, CFB-020, CFB-022, CFB-026, CFB-027, CFB-028, CFB-029, CFB-032, CFB-033, CFB-034, CFB-035, CFB-037, CFB-040, CFB-041, CFB-042, CFB-043, CFB-044, CFB-049, CFB-050, CFB-051, CFB-052, CFB-053, CFB-054, CFB-055, CFB-056, CFB-057, CFB-058, CFB-059, CFB-060, CFB-061, CFB-062, CFB-063, CFB-064, CFB-065, CFB-066, CFB-067, CFB-068, CFB-069, CFB-070, CFB-071, CFB-072, CFB-073, CFB-074, CFB-075, CFB-076, CFB-077, CFB-078, CFB-081, CFB-082, CFB-083, CFB-084, CFR-085, CFR-086, CFR-087, CFR-088, CFR-096, CFR-097, CFR-098, CFR-099, CFR-100, CFR-101, CFR-102, CFR-103, CFR-104, CFR-106, CFR-107, CFR-108, CFR-109, CFR-114, CFR-115, CFR-124, CFR-125, CFR-126, CFR-127, CFR-132, CFR-133, CFR-134, CFR-135, CFR-136, CFR-137, CFR-138, CFR-139, CFR-140, CFR-141, CFR-142, CFR-143, CFR-144, CFR-145, CFR-146, CFR-147, CFR-148, CFR-149, CFR-151, CFR-152, CFR-153, CFR-154, CFR-155, CFR-156, CFR-157, CFR-158, CFR-159, CFR-160.
- Remaining open now: 35.

Burn-down interpretation:

- The current cycle is no longer scoped to only CFB-086..CFB-092.
- All open CFB/CFR are part of this single global burn-down batch.
- Root-cause grouping is allowed only to reduce duplicate implementation work; it must not exclude issues from the batch.

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

- All tracked CFB/CFR issues in the active global batch have no `open` status.
- Fixed issue detail files include `Fix` and `Verification` sections.
- `bug-backlog.md` summary rows match detail files.
- `current-cycle-burn-down-plan.md` marks remaining open count as 0.
- Full Rust checks pass.
- Installed CodeFire passes algorithm dogfood confirmation.
- Git is clean and pushed.
