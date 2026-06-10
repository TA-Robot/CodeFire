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

## Current Open State

Current known open issue count:

- CFB/CFR product/review issues: 0.
- Algorithm CodeFire open fires after the latest dogfood seal: 0.
- Active frozen issue batch: none.
- The historical 146-item global batch below is already closed. The number 146 is the old starting count, not a remaining-open count.
- Closed issue detail files remain in `docs/development/bug-issues/` and `docs/development/code-review-issues/` as audit history. Counting files in those directories is not the open issue count; `Status: fixed` / `Status: fixed in v0.7` means closed.

## Closed Historical Batch

Closed batch ID:

- `BATCH-2026-06-global-open-backlog`

Scope:

- 2026-06-09時点でopenだった全既知issue。
- Initial frozen CFB dogfood issue count: 68。
- Initial frozen CFR code review issue count: 78。
- Historical initial frozen total: 146件を同一burn-down対象としてfreezeした。
- This batch is complete. 新規issue探索は、このglobal batchを0件にした後に再開した。

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
| CFB-045 | fixed | Diff JSON included sections | impact presence is reflected in options and included_sections |
| CFB-025 | fixed | Legacy policy compatibility | parser accepts both `verification:` and `verification.required:` command lists |
| CFB-021 | fixed | Commit summary | commit text/JSON summarize message, changed atoms, extinguished fires, active resolutions, and evidence refs |
| CFB-017 | fixed | Non-Atom file visibility | scan/context/status/commit expose non-Atom changed file counts and bounded details |
| CFR-117 | fixed | Core scan model | ScanResult carries non-Atom file changes as first-class metadata |
| CFR-092 | fixed | Branch unsupported JSON | unsupported branch subcommands return branch-specific JSON diagnostics and actionable next_actions |
| CFR-111 | fixed | HTTP timeout env diagnostics | invalid/nonpositive CODEFIRE_HTTP_TIMEOUT_MS values fail explicitly instead of silently falling back |
| CFR-128 | fixed | HTTP response body limit | client rejects oversized response Content-Length before reading the body |
| CFR-118 | fixed | Object lookup prefix validation | unknown CF-* object prefixes no longer trigger fallback subdir scans |
| CFR-121 | fixed | Required commit roots | sealed commit validation requires resolution_ledger |
| CFR-081 | fixed | CLI JSON failure fallback | uncaught `--json` failures now return command_result error envelopes |
| CFR-082 | fixed | Command capability registry | command names/capabilities are shared by help, completion, and capability JSON |
| CFR-083 | fixed | Debug command JSON envelope | read-only debug commands support `--path` and JSON envelopes |
| CFR-084 | fixed | Remote read JSON | remote branch/MR read commands expose command_result JSON |
| CFR-094 | fixed | Link help/error contract | link help is parser-before-mutation and JSON errors are structured |
| CFR-095 | fixed | Structured CLI diagnostics | `CliError::diagnostic()` maps all variants to typed diagnostics |
| CFR-150 | fixed | Metrics capability registry | metrics support is declared in command registry and exposed by `capabilities --json` |
| CFB-007 | fixed | Performance metrics | status/scan/verify/branch list expose `data.metrics` and `capabilities --json` reports support |
| CFB-008 | fixed | Storage/artifact retention | storage report and artifact_ref keep large artifact payloads external with warnings/summaries |
| CFR-113 | fixed | Upload dry-run envelope | upload dry-run exposes top-level repo and next_actions |
| CFR-093 | fixed | Status repo context diagnostics | status no longer swallows open context mismatch in JSON output |
| CFR-090 | fixed | HTTP remote view temp dirs | HTTP bundle temp object directories are unique per call and cleaned on drop |
| CFR-110 | fixed | HTTP remote diagnostic propagation | HTTP error responses preserve remote diagnostic kind/status/exit code |
| CFR-116 | fixed | Verification command execution contract | core policy and local runner now enforce timeout/output/env/allow_failure |
| CFR-122 | fixed | Commit certificate verification cross-check | sealed commit validation compares certificate result/counts to verification root |
| CFR-123 | fixed | Root payload shape validation | known required roots validate payload shape before commit acceptance |
| CFR-112 | fixed | Remote upload dry-run validation | file remote dry-run validates existing layout and reports validation checks |
| CFR-129 | fixed | HTTP request-merge dry-run validation | HTTP dry-run resolves source/target heads instead of returning empty heads |
| CFB-009 | fixed | CLI module boundary | import workflow was implemented in a dedicated module with thin main dispatch |
| CFB-048 | fixed | Existing project import | `codefire import` adopts existing non-empty directories as open CodeFire worktrees |
| CFR-089 | fixed | Manifest diff memory | normal diff and patch export compare manifest metadata before reading changed blobs |
| CFR-105 | fixed | Storage scan observability | storage report exposes object scan metrics and quick/full JSON parse counts |
| CFR-119 | fixed | Durable write contract | commit transaction marker and docs align object/metadata durability model |
| CFR-130 | fixed | Commit transaction diagnostics | interrupted multi-object commit leaves a doctor-visible pending marker |
| CFR-120 | fixed | Issue root cause map | open issue mapping is enforced by machine-readable root cause map and docs consistency test |

Remaining open count for this batch:

- 0 issues.

## Global Issue Inventory

This active batch is the whole known open backlog.

Current tracked issue inventory as of 2026-06-10:

| Series | Total tracked | Fixed / closed | Open | Notes |
|---|---:|---:|---:|---|
| CFB dogfood issues | 95 | 95 | 0 | `docs/development/bug-backlog.md` is the summary source. Detail files exist for every `CFB-001..CFB-095` issue. |
| CFR code review issues | 160 | 160 | 0 | `docs/development/code-review-issues-2026-06-06.md` and detail files are the summary/detail source. `fixed in v0.7` is counted as fixed/closed. |
| Total | 255 | 255 | 0 | This global batch plus cycle 4 through cycle 36 dogfood passes are the whole known issue backlog. |

Completed global batch progress:

- Historical starting count for the closed global batch: 146. This is not the current open count.
- Fixed during global burn-down so far: CFB-005, CFB-006, CFB-007, CFB-008, CFB-009, CFB-010, CFB-011, CFB-012, CFB-013, CFB-014, CFB-015, CFB-016, CFB-017, CFB-018, CFB-020, CFB-021, CFB-022, CFB-025, CFB-026, CFB-027, CFB-028, CFB-029, CFB-032, CFB-033, CFB-034, CFB-035, CFB-037, CFB-040, CFB-041, CFB-042, CFB-043, CFB-044, CFB-045, CFB-048, CFB-049, CFB-050, CFB-051, CFB-052, CFB-053, CFB-054, CFB-055, CFB-056, CFB-057, CFB-058, CFB-059, CFB-060, CFB-061, CFB-062, CFB-063, CFB-064, CFB-065, CFB-066, CFB-067, CFB-068, CFB-069, CFB-070, CFB-071, CFB-072, CFB-073, CFB-074, CFB-075, CFB-076, CFB-077, CFB-078, CFB-081, CFB-082, CFB-083, CFB-084, CFR-081, CFR-082, CFR-083, CFR-084, CFR-085, CFR-086, CFR-087, CFR-088, CFR-089, CFR-092, CFR-094, CFR-095, CFR-096, CFR-097, CFR-098, CFR-099, CFR-100, CFR-101, CFR-102, CFR-103, CFR-104, CFR-105, CFR-106, CFR-107, CFR-108, CFR-109, CFR-111, CFR-112, CFR-113, CFR-114, CFR-115, CFR-116, CFR-117, CFR-118, CFR-119, CFR-120, CFR-121, CFR-122, CFR-123, CFR-124, CFR-125, CFR-126, CFR-127, CFR-128, CFR-129, CFR-130, CFR-132, CFR-133, CFR-134, CFR-135, CFR-136, CFR-137, CFR-138, CFR-139, CFR-140, CFR-141, CFR-142, CFR-143, CFR-144, CFR-145, CFR-146, CFR-147, CFR-148, CFR-149, CFR-150, CFR-151, CFR-152, CFR-153, CFR-154, CFR-155, CFR-156, CFR-157, CFR-158, CFR-159, CFR-160.
- Remaining open now: 0.

## Cycle 4 Dogfood Mini-batch

After the global backlog reached zero, cycle 4 discovery resumed through `algorithm-evolution-agent-lab` dogfooding.

| Issue | Status | Root fix group | Current decision |
|---|---|---|---|
| CFB-093 | fixed | Status prediction next_actions | `status --json` uses `scan_prediction` to recommend `commit` when pending changes have zero open fires |

Cycle 4 mini-batch status:

- Discovered: 1.
- Fixed / closed: 1.
- Remaining open: 0.

## Cycle 5 Dogfood Pass

After cycle 4 reached zero open issues, cycle 5 resumed algorithm development with the installed CodeFire binary.

Algorithm work:

- Added `ResearchCyclePlanMarkdown` to render research cycle plans as operator-readable Markdown.
- Added `REQ-AUTO-041`, `DES-AUTO-023`, `CODE-ResearchCyclePlanMarkdown`, and `TEST-research-cycle-plan-markdown-renders-lanes`.
- Algorithm unittest result: 146 tests pass.
- CodeFire scan opened 6 trace-change fires for the new REQ/DES/CODE/TEST atoms.
- All 6 fires were extinguished in the same cycle and sealed as `CF-COMMIT-476c73ed58ac5e64be166e38`.

Cycle 5 issue status:

- Newly discovered CodeFire product issues: 0.
- Newly frozen CFB/CFR batch size: 0.
- Remaining known CFB/CFR open issue count: 0.

## Cycle 6 Dogfood Pass

After cycle 5 reached zero open issues, cycle 6 resumed algorithm development with the installed CodeFire binary.

Algorithm work:

- Added `ResearchCyclePlanLint` to validate next-cycle plans before execution.
- Added `REQ-AUTO-042`, `DES-AUTO-024`, `CODE-ResearchCyclePlanLint`, and `TEST-research-cycle-plan-lint-flags-invalid-plan`.
- Algorithm unittest result: 148 tests pass.
- CodeFire scan opened 6 trace-change fires for the new REQ/DES/CODE/TEST atoms.
- All 6 fires were extinguished in the same cycle and sealed as `CF-COMMIT-e9809fabb758032363e80407`.
- A follow-up non-Atom history update was sealed as `CF-COMMIT-2de6ff5b70a1591e55786035` after CFB-094 was fixed and installed.

Cycle 6 issue status:

- Newly discovered CodeFire product issues: 1 (`CFB-094`).
- Newly frozen CFB/CFR batch size: 1.
- Fixed / closed: 1.
- Remaining known CFB/CFR open issue count: 0.

Cycle 6 burn-down:

| Issue | Status | Root fix group | Current decision |
|---|---|---|---|
| CFB-094 | fixed | Status prediction next_actions | `status --json` uses scan prediction changed/open-fire counts to recommend `commit` for non-Atom-only pending changes |

## Cycle 7 Dogfood Pass

After cycle 6 reached zero open issues, cycle 7 resumed algorithm development with the installed CodeFire binary.

Algorithm work:

- Added `ResearchCyclePlanLintMarkdown` to render lint reports for reviewer-facing planning logs.
- Added `REQ-AUTO-043`, `DES-AUTO-025`, `CODE-ResearchCyclePlanLintMarkdown`, and `TEST-research-cycle-plan-lint-markdown-renders-findings`.
- Algorithm unittest result: 149 tests pass.
- CodeFire scan opened 6 trace-change fires for the new REQ/DES/CODE/TEST atoms.
- All 6 fires were extinguished in the same cycle and sealed as `CF-COMMIT-a1d8e9e852adbb15c0e1e532`.

Cycle 7 issue status:

- Newly discovered CodeFire product issues: 0.
- Newly frozen CFB/CFR batch size: 0.
- Remaining known CFB/CFR open issue count: 0.

## Cycle 8 Dogfood Pass

After cycle 7 reached zero open issues, cycle 8 resumed algorithm development with the installed CodeFire binary.

Algorithm work:

- Added `ResearchCyclePlanningPacketBuilder` to produce the plan, lint report, plan Markdown, and lint Markdown as one synchronized planning packet.
- Added `REQ-AUTO-044`, `DES-AUTO-026`, `CODE-ResearchCyclePlanningPacketBuilder`, and `TEST-research-cycle-planning-packet-builds-plan-lint-and-markdown`.
- Algorithm unittest result: 150 tests pass.
- CodeFire scan opened 6 trace-change fires for the new REQ/DES/CODE/TEST atoms.
- All 6 fires were extinguished in the same cycle and sealed as `CF-COMMIT-7000b03b972c87a78bda0893`.

Cycle 8 issue status:

- Newly discovered CodeFire product issues: 0.
- Newly frozen CFB/CFR batch size: 0.
- Remaining known CFB/CFR open issue count: 0.

## Cycle 9 Dogfood Pass

After cycle 8 reached zero open issues, cycle 9 resumed algorithm development with the installed CodeFire binary.

Algorithm work:

- Added `ResearchCyclePlanningPacketMarkdown` to render a synchronized planning packet as one cycle handoff Markdown document.
- Added `REQ-AUTO-045`, `DES-AUTO-027`, `CODE-ResearchCyclePlanningPacketMarkdown`, and `TEST-research-cycle-planning-packet-markdown-renders-review-document`.
- Algorithm unittest result: 151 tests pass.
- CodeFire scan opened 6 trace-change fires for the new REQ/DES/CODE/TEST atoms.
- All 6 fires were extinguished in the same cycle and sealed as `CF-COMMIT-f16f1f0a0de14e661cbde9d0`.

Cycle 9 issue status:

- Newly discovered CodeFire product issues: 0.
- Newly frozen CFB/CFR batch size: 0.
- Remaining known CFB/CFR open issue count: 0.

## Cycle 10 Dogfood Pass

After cycle 9 reached zero open issues, cycle 10 resumed algorithm development with the installed CodeFire binary.

Algorithm work:

- Added `ResearchCyclePlanningPacketManifestBuilder` to record relative artifact paths, SHA-256 digests, and byte counts for planning packet handoff artifacts.
- Added `REQ-AUTO-046`, `DES-AUTO-028`, `CODE-ResearchCyclePlanningPacketManifestBuilder`, and `TEST-research-cycle-planning-packet-manifest-records-artifact-hashes`.
- Algorithm unittest result: 152 tests pass.
- CodeFire scan opened 6 trace-change fires for the new REQ/DES/CODE/TEST atoms.
- All 6 fires were extinguished in the same cycle and sealed as `CF-COMMIT-738987cf39675353564e60da`.
- A formatting follow-up for `TEST-research-cycle-planning-packet-manifest-records-artifact-hashes` opened 1 trace-change fire, which was extinguished and sealed as `CF-COMMIT-a82560bcd3f29c8b06b7edf4`.

Cycle 10 issue status:

- Newly discovered CodeFire product issues: 1 (`CFB-095`).
- Newly frozen CFB/CFR batch size: 1.
- Fixed / closed: 1.
- Remaining known CFB/CFR open issue count: 0.
- Dogfood note: a parallel mutating `extinguish` attempt hit repo lock contention; sequential retry succeeded, so this cycle treated it as expected lock protection rather than a product issue.

Cycle 10 burn-down:

| Issue | Status | Root fix group | Current decision |
|---|---|---|---|
| CFB-095 | fixed | Status prediction next_actions | `status --json` uses scan prediction changed/open-fire counts to recommend `scan` and `verify` when prediction-only open fires exist |

## Cycle 11 Dogfood Pass

After cycle 10 reached zero open issues, cycle 11 resumed algorithm development with the installed CodeFire binary.

Algorithm work:

- Added `ResearchCyclePlanningPacketManifestMarkdown` to render planning packet manifests as reviewer-readable audit tables.
- Added `REQ-AUTO-047`, `DES-AUTO-029`, `CODE-ResearchCyclePlanningPacketManifestMarkdown`, and `TEST-research-cycle-planning-packet-manifest-markdown-renders-audit-table`.
- Algorithm unittest result: 153 tests pass.
- CodeFire scan opened 6 trace-change fires for the new REQ/DES/CODE/TEST atoms.
- All 6 fires were extinguished in the same cycle and sealed as `CF-COMMIT-85c55142b38b449258674506`.

Cycle 11 issue status:

- Newly discovered CodeFire product issues: 0.
- Newly frozen CFB/CFR batch size: 0.
- Remaining known CFB/CFR open issue count: 0.

## Cycle 12 Dogfood Pass

After cycle 11 reached zero open issues, cycle 12 resumed algorithm development with the installed CodeFire binary.

Algorithm work:

- Added `ResearchCyclePlanningPacketManifestVerifier` to detect missing, hash-drifted, and byte-count-drifted planning packet artifacts.
- Added `REQ-AUTO-048`, `DES-AUTO-030`, `CODE-ResearchCyclePlanningPacketManifestVerifier`, and `TEST-research-cycle-planning-packet-manifest-verifier-detects-drift`.
- Algorithm unittest result: 154 tests pass.
- CodeFire scan opened 6 trace-change fires for the new REQ/DES/CODE/TEST atoms.
- All 6 fires were extinguished in the same cycle and sealed as `CF-COMMIT-3235c02fd7d24ef40b6cf9cd`.

Cycle 12 issue status:

- Newly discovered CodeFire product issues: 0.
- Newly frozen CFB/CFR batch size: 0.
- Remaining known CFB/CFR open issue count: 0.

## Cycle 13 Dogfood Pass

After cycle 12 reached zero open issues, cycle 13 resumed algorithm development with the installed CodeFire binary.

Algorithm work:

- Added `ResearchCyclePlanningPacketManifestVerificationMarkdown` to render planning packet manifest verification results as reviewer-facing audit logs.
- Added `REQ-AUTO-049`, `DES-AUTO-031`, `CODE-ResearchCyclePlanningPacketManifestVerificationMarkdown`, and `TEST-research-cycle-planning-packet-manifest-verification-markdown-renders-findings`.
- Algorithm unittest result: 155 tests pass.
- CodeFire scan opened 6 trace-change fires for the new REQ/DES/CODE/TEST atoms.
- All 6 fires were extinguished in the same cycle and sealed as `CF-COMMIT-27ff328879a1d33f166509f1`.

Cycle 13 issue status:

- Newly discovered CodeFire product issues: 0.
- Newly frozen CFB/CFR batch size: 0.
- Remaining known CFB/CFR open issue count: 0.

## Cycle 14 Dogfood Pass

After cycle 13 reached zero open issues, cycle 14 resumed algorithm development with the installed CodeFire binary.

Algorithm work:

- Added `ResearchCyclePlanningHandoffBundleBuilder` to assemble the planning packet, artifact contents, manifest, manifest Markdown, verification result, and verification Markdown into one deterministic handoff object.
- Added `REQ-AUTO-050`, `DES-AUTO-032`, `CODE-ResearchCyclePlanningHandoffBundleBuilder`, and `TEST-research-cycle-planning-handoff-bundle-builds-artifacts-and-audits`.
- Algorithm unittest result: 156 tests pass.
- CodeFire scan opened 6 trace-change fires for the new REQ/DES/CODE/TEST atoms.
- All 6 fires were extinguished in the same cycle and sealed as `CF-COMMIT-a8a2133eebf4e2d25a6b5ff4`.

Cycle 14 issue status:

- Newly discovered CodeFire product issues: 0.
- Newly frozen CFB/CFR batch size: 0.
- Remaining known CFB/CFR open issue count: 0.

## Cycle 15 Dogfood Pass

After cycle 14 reached zero open issues, cycle 15 resumed algorithm development with the installed CodeFire binary.

Algorithm work:

- Added `ResearchCyclePlanningHandoffBundleMarkdown` to render planning handoff bundles as artifact and audit index Markdown.
- Added `REQ-AUTO-051`, `DES-AUTO-033`, `CODE-ResearchCyclePlanningHandoffBundleMarkdown`, and `TEST-research-cycle-planning-handoff-bundle-markdown-renders-index`.
- Algorithm unittest result: 157 tests pass.
- CodeFire scan opened 6 trace-change fires for the new REQ/DES/CODE/TEST atoms.
- All 6 fires were extinguished in the same cycle and sealed as `CF-COMMIT-f28a264a7eb8ba11f09c6f69`.

Cycle 15 issue status:

- Newly discovered CodeFire product issues: 0.
- Newly frozen CFB/CFR batch size: 0.
- Remaining known CFB/CFR open issue count: 0.

## Cycle 16 Dogfood Pass

After cycle 15 reached zero open issues, cycle 16 resumed algorithm development with the installed CodeFire binary.

Algorithm work:

- Added `ResearchCyclePlanningHandoffBundleSummary` to render planning handoff bundles as a machine-readable status, artifact, digest, audit, and finding-count index.
- Added `REQ-AUTO-052`, `DES-AUTO-034`, `CODE-ResearchCyclePlanningHandoffBundleSummary`, and `TEST-research-cycle-planning-handoff-bundle-summary-reports-machine-readable-index`.
- Algorithm unittest result: 158 tests pass.
- CodeFire scan opened 6 trace-change fires for the new REQ/DES/CODE/TEST atoms.
- All 6 fires were extinguished in the same cycle and sealed as `CF-COMMIT-7e54b0cd74ec6147417adfae`.

Cycle 16 issue status:

- Newly discovered CodeFire product issues: 0.
- Newly frozen CFB/CFR batch size: 0.
- Remaining known CFB/CFR open issue count: 0.

## Cycle 17 Dogfood Pass

After cycle 16 reached zero open issues, cycle 17 resumed algorithm development with the installed CodeFire binary.

Algorithm work:

- Added `ResearchCyclePlanningHandoffReadinessGate` to evaluate planning handoff bundles as ready/blocked before storage or reviewer handoff.
- Added `REQ-AUTO-053`, `DES-AUTO-035`, `CODE-ResearchCyclePlanningHandoffReadinessGate`, and `TEST-research-cycle-planning-handoff-readiness-gate-blocks-drifted-bundle`.
- Algorithm unittest result: 159 tests pass.
- CodeFire scan opened 6 trace-change fires for the new REQ/DES/CODE/TEST atoms.
- All 6 fires were extinguished in the same cycle and sealed as `CF-COMMIT-e339eafbdb954d1d3aa94199`.

Cycle 17 issue status:

- Newly discovered CodeFire product issues: 0.
- Newly frozen CFB/CFR batch size: 0.
- Remaining known CFB/CFR open issue count: 0.

## Cycle 18 Dogfood Pass

After cycle 17 reached zero open issues, cycle 18 resumed algorithm development with the installed CodeFire binary.

Algorithm work:

- Added `ResearchCyclePlanningHandoffReadinessMarkdown` to render readiness gate decisions as reviewer-facing audit Markdown.
- Added `REQ-AUTO-054`, `DES-AUTO-036`, `CODE-ResearchCyclePlanningHandoffReadinessMarkdown`, and `TEST-research-cycle-planning-handoff-readiness-markdown-renders-status`.
- Algorithm unittest result: 160 tests pass.
- CodeFire scan opened 6 trace-change fires for the new REQ/DES/CODE/TEST atoms.
- All 6 fires were extinguished in the same cycle and sealed as `CF-COMMIT-b6e595769abe0e97f721bcf6`.

Cycle 18 issue status:

- Newly discovered CodeFire product issues: 0.
- Newly frozen CFB/CFR batch size: 0.
- Remaining known CFB/CFR open issue count: 0.

## Cycle 19 Dogfood Pass

After cycle 18 reached zero open issues, cycle 19 resumed algorithm development with the installed CodeFire binary.

Algorithm work:

- Added `ResearchCyclePlanningHandoffReviewPacketBuilder` to compose an existing handoff bundle, machine-readable summary, readiness gate result, and readiness Markdown into one review packet without recomputing planning or writing files.
- Added `REQ-AUTO-055`, `DES-AUTO-037`, `CODE-ResearchCyclePlanningHandoffReviewPacketBuilder`, and `TEST-research-cycle-planning-handoff-review-packet-bundles-summary-and-readiness`.
- Algorithm unittest result: 161 tests pass.
- CodeFire scan opened 6 trace-change fires for the new REQ/DES/CODE/TEST atoms.
- All 6 fires were extinguished in the same cycle and sealed as `CF-COMMIT-b1606907ae8c9c0ef9f8d86d`.

Cycle 19 issue status:

- Newly discovered CodeFire product issues: 0.
- Newly frozen CFB/CFR batch size: 0.
- Remaining known CFB/CFR open issue count: 0.

## Cycle 20 Dogfood Pass

After cycle 19 reached zero open issues, cycle 20 resumed algorithm development with the installed CodeFire binary.

Algorithm work:

- Added `ResearchCyclePlanningHandoffReviewPacketMarkdown` to render an already-built review packet into one reviewer-facing Markdown document with readiness, status, artifact digest, blocker, warning, and audit availability sections.
- Added `REQ-AUTO-056`, `DES-AUTO-038`, `CODE-ResearchCyclePlanningHandoffReviewPacketMarkdown`, and `TEST-research-cycle-planning-handoff-review-packet-markdown-renders-review-document`.
- Algorithm unittest result: 162 tests pass.
- CodeFire scan opened 6 trace-change fires for the new REQ/DES/CODE/TEST atoms.
- All 6 fires were extinguished in the same cycle and sealed as `CF-COMMIT-8bfb09c524a9502892cb6fcc`.

Cycle 20 issue status:

- Newly discovered CodeFire product issues: 0.
- Newly frozen CFB/CFR batch size: 0.
- Remaining known CFB/CFR open issue count: 0.

## Cycle 21 Dogfood Pass

After cycle 20 reached zero open issues, cycle 21 resumed algorithm development with the installed CodeFire binary.

Algorithm work:

- Added `ResearchCyclePlanningHandoffReviewPacketArtifactBuilder` to package an already-built review packet, readiness Markdown, manifest Markdown, and verification Markdown into deterministic handoff artifacts for storage adapters.
- Added `REQ-AUTO-057`, `DES-AUTO-039`, `CODE-ResearchCyclePlanningHandoffReviewPacketArtifactBuilder`, and `TEST-research-cycle-planning-handoff-review-packet-artifact-builder-packages-audits`.
- Algorithm unittest result: 163 tests pass.
- CodeFire scan opened 6 trace-change fires for the new REQ/DES/CODE/TEST atoms.
- All 6 fires were extinguished in the same cycle and sealed as `CF-COMMIT-5dd928c08fe5c0eeb673bb04`.

Cycle 21 issue status:

- Newly discovered CodeFire product issues: 0.
- Newly frozen CFB/CFR batch size: 0.
- Remaining known CFB/CFR open issue count: 0.

## Cycle 22 Dogfood Pass

After cycle 21 reached zero open issues, cycle 22 resumed algorithm development with the installed CodeFire binary.

Algorithm work:

- Added `ResearchCyclePlanningHandoffReviewPacketArtifactManifestBuilder` to build a deterministic path, SHA-256, and byte-count manifest for packaged review packet artifacts.
- Added `REQ-AUTO-058`, `DES-AUTO-040`, `CODE-ResearchCyclePlanningHandoffReviewPacketArtifactManifestBuilder`, and `TEST-research-cycle-planning-handoff-review-packet-artifact-manifest-records-artifacts`.
- Algorithm unittest result: 164 tests pass.
- CodeFire scan opened 6 trace-change fires for the new REQ/DES/CODE/TEST atoms.
- All 6 fires were extinguished in the same cycle and sealed as `CF-COMMIT-a301c378a9e91020c26669e1`.

Cycle 22 issue status:

- Newly discovered CodeFire product issues: 0.
- Newly frozen CFB/CFR batch size: 0.
- Remaining known CFB/CFR open issue count: 0.

## Cycle 23 Dogfood Pass

After cycle 22 reached zero open issues, cycle 23 resumed algorithm development with the installed CodeFire binary.

Algorithm work:

- Added `ResearchCyclePlanningHandoffReviewPacketArtifactManifestMarkdown` to render review packet artifact manifests as deterministic reviewer-facing Markdown tables.
- Added `REQ-AUTO-059`, `DES-AUTO-041`, `CODE-ResearchCyclePlanningHandoffReviewPacketArtifactManifestMarkdown`, and `TEST-research-cycle-planning-handoff-review-packet-artifact-manifest-markdown-renders-table`.
- Algorithm unittest result: 165 tests pass.
- CodeFire scan opened 6 trace-change fires for the new REQ/DES/CODE/TEST atoms.
- All 6 fires were extinguished in the same cycle and sealed as `CF-COMMIT-bdf99515ee6fa2576e7b0acc`.

Cycle 23 issue status:

- Newly discovered CodeFire product issues: 0.
- Newly frozen CFB/CFR batch size: 0.
- Remaining known CFB/CFR open issue count: 0.

## Cycle 24 Dogfood Pass

After cycle 23 reached zero open issues, cycle 24 resumed algorithm development with the installed CodeFire binary.

Algorithm work:

- Added `ResearchCyclePlanningHandoffReviewPacketArtifactManifestVerifier` to detect missing artifacts, SHA-256 drift, and byte-count drift for review packet artifact manifests.
- Added `REQ-AUTO-060`, `DES-AUTO-042`, `CODE-ResearchCyclePlanningHandoffReviewPacketArtifactManifestVerifier`, and `TEST-research-cycle-planning-handoff-review-packet-artifact-manifest-verifier-detects-drift`.
- Algorithm unittest result: 166 tests pass.
- CodeFire scan opened 6 trace-change fires for the new REQ/DES/CODE/TEST atoms.
- All 6 fires were extinguished in the same cycle and sealed as `CF-COMMIT-efcc72220a4aec9228629367`.

Cycle 24 issue status:

- Newly discovered CodeFire product issues: 0.
- Newly frozen CFB/CFR batch size: 0.
- Remaining known CFB/CFR open issue count: 0.

## Cycle 25 Dogfood Pass

After cycle 24 reached zero open issues, cycle 25 resumed algorithm development with the installed CodeFire binary.

Algorithm work:

- Added `ResearchCyclePlanningHandoffReviewPacketArtifactManifestVerificationMarkdown` to render review packet artifact manifest verification results as deterministic reviewer-facing Markdown findings.
- Added `REQ-AUTO-061`, `DES-AUTO-043`, `CODE-ResearchCyclePlanningHandoffReviewPacketArtifactManifestVerificationMarkdown`, and `TEST-research-cycle-planning-handoff-review-packet-artifact-manifest-verification-markdown-renders-findings`.
- Algorithm unittest result: 167 tests pass.
- CodeFire scan opened 6 trace-change fires for the new REQ/DES/CODE/TEST atoms.
- All 6 fires were extinguished in the same cycle and sealed as `CF-COMMIT-ec21ce3f2d608602068ca5b6`.

Cycle 25 issue status:

- Newly discovered CodeFire product issues: 0.
- Newly frozen CFB/CFR batch size: 0.
- Remaining known CFB/CFR open issue count: 0.

## Cycle 26 Dogfood Pass

After cycle 25 reached zero open issues, cycle 26 resumed algorithm development with the installed CodeFire binary.

Algorithm work:

- Added `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveBuilder` to package final review packet artifacts, manifest, manifest Markdown, verification, and verification Markdown into one deterministic archive object.
- Added `REQ-AUTO-062`, `DES-AUTO-044`, `CODE-ResearchCyclePlanningHandoffReviewPacketArtifactArchiveBuilder`, and `TEST-research-cycle-planning-handoff-review-packet-artifact-archive-builder-builds-final-audits`.
- Algorithm unittest result: 168 tests pass.
- CodeFire scan opened 6 trace-change fires for the new REQ/DES/CODE/TEST atoms.
- All 6 fires were extinguished in the same cycle and sealed as `CF-COMMIT-11cd0738c4cc3e281c943b16`.

Cycle 26 issue status:

- Newly discovered CodeFire product issues: 0.
- Newly frozen CFB/CFR batch size: 0.
- Remaining known CFB/CFR open issue count: 0.

## Cycle 27 Dogfood Pass

After cycle 26 reached zero open issues, cycle 27 resumed algorithm development with the installed CodeFire binary.

Algorithm work:

- Added `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummary` to render final review packet artifact archives as deterministic machine-readable indexes.
- Added `REQ-AUTO-063`, `DES-AUTO-045`, `CODE-ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummary`, and `TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-reports-index`.
- Algorithm unittest result: 169 tests pass.
- CodeFire scan opened 6 trace-change fires for the new REQ/DES/CODE/TEST atoms.
- All 6 fires were extinguished in the same cycle and sealed as `CF-COMMIT-ef003ce1fbfaa856969d83ca`.

Cycle 27 issue status:

- Newly discovered CodeFire product issues: 0.
- Newly frozen CFB/CFR batch size: 0.
- Remaining known CFB/CFR open issue count: 0.

## Cycle 28 Dogfood Pass

After cycle 27 reached zero open issues, cycle 28 resumed algorithm development with the installed CodeFire binary.

Algorithm work:

- Added `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryMarkdown` to render final review packet artifact archive summaries as deterministic reviewer-facing Markdown indexes.
- Added `REQ-AUTO-064`, `DES-AUTO-046`, `CODE-ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryMarkdown`, and `TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-markdown-renders-index`.
- Algorithm unittest result: 170 tests pass.
- CodeFire scan opened 6 trace-change fires for the new REQ/DES/CODE/TEST atoms.
- All 6 fires were extinguished in the same cycle and sealed as `CF-COMMIT-b0059f76d36e80505a0e7522`.

Cycle 28 issue status:

- Newly discovered CodeFire product issues: 0.
- Newly frozen CFB/CFR batch size: 0.
- Remaining known CFB/CFR open issue count: 0.

## Cycle 29 Dogfood Pass

After cycle 28 reached zero open issues, cycle 29 resumed algorithm development with the installed CodeFire binary.

Algorithm work:

- Added `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryMarkdownGate` to validate final review archive summary Markdown against the machine-readable summary before reviewer handoff.
- Added `REQ-AUTO-065`, `DES-AUTO-047`, `CODE-ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryMarkdownGate`, and `TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-markdown-gate-blocks-drift`.
- Algorithm unittest result: 171 tests pass.
- CodeFire scan opened 6 trace-change fires for the new REQ/DES/CODE/TEST atoms.
- All 6 fires were extinguished in the same cycle and sealed as `CF-COMMIT-21c452eaaed817710d58b84f`.

Cycle 29 issue status:

- Newly discovered CodeFire product issues: 0.
- Newly frozen CFB/CFR batch size: 0.
- Remaining known CFB/CFR open issue count: 0.

## Cycle 30 Dogfood Pass

After cycle 29 reached zero open issues, cycle 30 resumed algorithm development with the installed CodeFire binary.

Algorithm work:

- Added `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryMarkdownGateMarkdown` to render archive summary Markdown gate decisions as deterministic reviewer-facing audit Markdown.
- Added `REQ-AUTO-066`, `DES-AUTO-048`, `CODE-ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryMarkdownGateMarkdown`, and `TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-markdown-gate-markdown-renders-blockers`.
- Algorithm unittest result: 172 tests pass.
- CodeFire scan opened 6 trace-change fires for the new REQ/DES/CODE/TEST atoms.
- All 6 fires were extinguished in the same cycle and sealed as `CF-COMMIT-50356d058653af136bbfe2fe`.

Cycle 30 issue status:

- Newly discovered CodeFire product issues: 0.
- Newly frozen CFB/CFR batch size: 0.
- Remaining known CFB/CFR open issue count: 0.

## Cycle 31 Dogfood Pass

After cycle 30 reached zero open issues, cycle 31 resumed algorithm development with the installed CodeFire binary.

Algorithm work:

- Added `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactBuilder` to package archive summary Markdown and archive summary gate Markdown as deterministic storage-ready handoff artifacts.
- Added `REQ-AUTO-067`, `DES-AUTO-049`, `CODE-ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactBuilder`, and `TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-artifact-builder-packages-gated-summary`.
- Algorithm unittest result: 173 tests pass.
- CodeFire scan opened 6 trace-change fires for the new REQ/DES/CODE/TEST atoms.
- All 6 fires were extinguished in the same cycle and sealed as `CF-COMMIT-f337eedec17e36c98916d3aa`.

Cycle 31 issue status:

- Newly discovered CodeFire product issues: 0.
- Newly frozen CFB/CFR batch size: 0.
- Remaining known CFB/CFR open issue count: 0.

## Cycle 32 Dogfood Pass

After cycle 31 reached zero open issues, cycle 32 resumed algorithm development with the installed CodeFire binary.

Algorithm work:

- Added `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactManifestBuilder` to build a manifest for archive summary and gate Markdown artifacts.
- Added `REQ-AUTO-068`, `DES-AUTO-050`, `CODE-ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactManifestBuilder`, and `TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-artifact-manifest-records-summary-artifacts`.
- Algorithm unittest result: 174 tests pass.
- CodeFire scan opened 6 trace-change fires for the new REQ/DES/CODE/TEST atoms.
- All 6 fires were extinguished in the same cycle and sealed as `CF-COMMIT-9042b8b63434edc218bd993b`.

Cycle 32 issue status:

- Newly discovered CodeFire product issues: 0.
- Newly frozen CFB/CFR batch size: 0.
- Remaining known CFB/CFR open issue count: 0.

## Cycle 33 Dogfood Pass

After cycle 32 reached zero open issues, cycle 33 resumed algorithm development with the installed CodeFire binary.

Algorithm work:

- Added `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactManifestMarkdown` to render archive summary artifact manifests as deterministic reviewer-facing Markdown tables.
- Added `REQ-AUTO-069`, `DES-AUTO-051`, `CODE-ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactManifestMarkdown`, and `TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-artifact-manifest-markdown-renders-table`.
- Algorithm unittest result: 175 tests pass.
- CodeFire scan opened 6 trace-change fires for the new REQ/DES/CODE/TEST atoms.
- All 6 fires were extinguished in the same cycle and sealed as `CF-COMMIT-7f483c5777bf8cb6da929246`.

Cycle 33 issue status:

- Newly discovered CodeFire product issues: 0.
- Newly frozen CFB/CFR batch size: 0.
- Remaining known CFB/CFR open issue count: 0.

## Cycle 34 Dogfood Pass

After cycle 33 reached zero open issues, cycle 34 resumed algorithm development with the installed CodeFire binary.

Algorithm work:

- Added `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactManifestVerifier` to verify archive summary artifact manifests against artifact contents.
- Added `REQ-AUTO-070`, `DES-AUTO-052`, `CODE-ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactManifestVerifier`, and `TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-artifact-manifest-verifier-detects-drift`.
- Algorithm unittest result: 176 tests pass.
- CodeFire scan opened 6 trace-change fires for the new REQ/DES/CODE/TEST atoms.
- All 6 fires were extinguished in the same cycle and sealed as `CF-COMMIT-d8910ff757e08b1119b03d12`.

Cycle 34 issue status:

- Newly discovered CodeFire product issues: 0.
- Newly frozen CFB/CFR batch size: 0.
- Remaining known CFB/CFR open issue count: 0.

## Cycle 35 Dogfood Pass

After cycle 34 reached zero open issues, cycle 35 resumed algorithm development with the installed CodeFire binary.

Algorithm work:

- Added `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactManifestVerificationMarkdown` to render archive summary artifact manifest verification findings into deterministic reviewer-facing Markdown.
- Added `REQ-AUTO-071`, `DES-AUTO-053`, `CODE-ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactManifestVerificationMarkdown`, and `TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-artifact-manifest-verification-markdown-renders-findings`.
- Algorithm unittest result: 177 tests pass.
- CodeFire scan opened 6 trace-change fires for the new REQ/DES/CODE/TEST atoms.
- All 6 fires were extinguished in the same cycle and sealed as `CF-COMMIT-b4d96c977703356c86890ec9`.

Cycle 35 issue status:

- Newly discovered CodeFire product issues: 0.
- Newly frozen CFB/CFR batch size: 0.
- Remaining known CFB/CFR open issue count: 0.

## Cycle 36 Dogfood Pass

After cycle 35 reached zero open issues, cycle 36 resumed algorithm development with the installed CodeFire binary.

Algorithm work:

- Added `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveBuilder` to package archive summary artifacts, their manifest, manifest Markdown, verification result, and verification Markdown as one archive object.
- Added `REQ-AUTO-072`, `DES-AUTO-054`, `CODE-ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveBuilder`, and `TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-artifact-archive-builder-builds-audits`.
- Algorithm unittest result: 178 tests pass.
- CodeFire scan opened 6 trace-change fires for the new REQ/DES/CODE/TEST atoms.
- All 6 fires were extinguished in the same cycle and sealed as `CF-COMMIT-03b681f9ef9c5844c3a1c639`.

Cycle 36 issue status:

- Newly discovered CodeFire product issues: 0.
- Newly frozen CFB/CFR batch size: 0.
- Remaining known CFB/CFR open issue count: 0.

## Cycle 37 Dogfood Pass

After cycle 36 reached zero open issues, cycle 37 resumed algorithm development with the installed CodeFire binary.

Algorithm work:

- Added `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummary` to summarize final archive summary artifact archives into a deterministic machine-readable index.
- Added `REQ-AUTO-073`, `DES-AUTO-055`, `CODE-ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummary`, and `TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-artifact-archive-summary-reports-index`.
- Algorithm unittest result: 179 tests pass.
- CodeFire scan opened 6 trace-change fires for the new REQ/DES/CODE/TEST atoms.
- All 6 fires were extinguished in the same cycle and sealed as `CF-COMMIT-8e531b39a917b74f6e89fec2`.
- Clarified that the historical 146 count is the closed global batch starting count, not current open work.

Cycle 37 issue status:

- Newly discovered CodeFire product issues: 0.
- Newly frozen CFB/CFR batch size: 0.
- Remaining known CFB/CFR open issue count: 0.

## Cycle 38 Dogfood Pass

After cycle 37 reached zero open issues, cycle 38 resumed algorithm development with the installed CodeFire binary.

Algorithm work:

- Added `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryMarkdown` to render final archive summary artifact archive summaries into deterministic Markdown.
- Added `REQ-AUTO-074`, `DES-AUTO-056`, `CODE-ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryMarkdown`, and `TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-artifact-archive-summary-markdown-renders-index`.
- Algorithm unittest result: 180 tests pass.
- CodeFire scan opened 6 trace-change fires for the new REQ/DES/CODE/TEST atoms.
- All 6 fires were extinguished in the same cycle and sealed as `CF-COMMIT-450cc175834950cb4741f574`.

Cycle 38 issue status:

- Newly discovered CodeFire product issues: 0.
- Newly frozen CFB/CFR batch size: 0.
- Remaining known CFB/CFR open issue count: 0.

## Cycle 39 Dogfood Pass

After cycle 38 reached zero open issues, cycle 39 resumed algorithm development with the installed CodeFire binary.

Algorithm work:

- Added `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryMarkdownGate` to evaluate final archive summary artifact archive summary Markdown for reviewer handoff readiness.
- Added `REQ-AUTO-075`, `DES-AUTO-057`, `CODE-ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryMarkdownGate`, and `TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-artifact-archive-summary-markdown-gate-blocks-drift`.
- Algorithm unittest result: 181 tests pass.
- CodeFire scan opened 6 trace-change fires for the new REQ/DES/CODE/TEST atoms.
- All 6 fires were extinguished in the same cycle and sealed as `CF-COMMIT-9385332c9198de84896635b1`.

Cycle 39 issue status:

- Newly discovered CodeFire product issues: 0.
- Newly frozen CFB/CFR batch size: 0.
- Remaining known CFB/CFR open issue count: 0.

## Cycle 40 Dogfood Pass

After cycle 39 reached zero open issues, cycle 40 resumed algorithm development with the installed CodeFire binary.

Algorithm work:

- Added `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryMarkdownGateMarkdown` to render archive summary artifact archive summary Markdown gate results into deterministic audit Markdown.
- Added `REQ-AUTO-076`, `DES-AUTO-058`, `CODE-ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryMarkdownGateMarkdown`, and `TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-artifact-archive-summary-markdown-gate-markdown-renders-blockers`.
- Algorithm unittest result: 182 tests pass.
- CodeFire scan opened 6 trace-change fires for the new REQ/DES/CODE/TEST atoms.
- All 6 fires were extinguished in the same cycle and sealed as `CF-COMMIT-20252fcb8779d550e7740a57`.

Cycle 40 issue status:

- Newly discovered CodeFire product issues: 0.
- Newly frozen CFB/CFR batch size: 0.
- Remaining known CFB/CFR open issue count: 0.

## Cycle 41 Dogfood Pass

After cycle 40 reached zero open issues, cycle 41 resumed algorithm development with the installed CodeFire binary.

Algorithm work:

- Added `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactBuilder` to package archive summary artifact archive summary Markdown and gate Markdown as deterministic handoff artifacts.
- Added `REQ-AUTO-077`, `DES-AUTO-059`, `CODE-ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactBuilder`, and `TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-artifact-archive-summary-artifact-builder-packages-gated-summary`.
- Algorithm unittest result: 183 tests pass.
- CodeFire scan opened 6 trace-change fires for the new REQ/DES/CODE/TEST atoms.
- All 6 fires were extinguished in the same cycle and sealed as `CF-COMMIT-9ac620dc323ed5ea63d5c58c`.

Cycle 41 issue status:

- Newly discovered CodeFire product issues: 0.
- Newly frozen CFB/CFR batch size: 0.
- Remaining known CFB/CFR open issue count: 0.

## Cycle 42 Dogfood Pass

After cycle 41 reached zero open issues, cycle 42 resumed algorithm development with the installed CodeFire binary.

Algorithm work:

- Added `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestBuilder` to generate deterministic manifests for archive summary artifact archive summary artifact sets.
- Added `REQ-AUTO-078`, `DES-AUTO-060`, `CODE-ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestBuilder`, and `TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-artifact-archive-summary-artifact-manifest-records-summary-artifacts`.
- Algorithm unittest result: 184 tests pass.
- CodeFire scan opened 6 trace-change fires for the new REQ/DES/CODE/TEST atoms.
- All 6 fires were extinguished in the same cycle and sealed as `CF-COMMIT-9fb5e3367603990b5a2efed4`.

Cycle 42 issue status:

- Newly discovered CodeFire product issues: 0.
- Newly frozen CFB/CFR batch size: 0.
- Remaining known CFB/CFR open issue count: 0.

## Cycle 43 Dogfood Pass

After cycle 42 reached zero open issues, cycle 43 resumed algorithm development with the installed CodeFire binary.

Algorithm work:

- Added `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdown` to render archive summary artifact archive summary artifact manifests into deterministic reviewer Markdown.
- Added `REQ-AUTO-079`, `DES-AUTO-061`, `CODE-ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdown`, and `TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-artifact-archive-summary-artifact-manifest-markdown-renders-table`.
- Algorithm unittest result: 185 tests pass.
- CodeFire scan opened 6 trace-change fires for the new REQ/DES/CODE/TEST atoms.
- All 6 fires were extinguished in the same cycle and sealed as `CF-COMMIT-adfe508b71f97f0fda8f2203`.

Cycle 43 issue status:

- Newly discovered CodeFire product issues: 0.
- Newly frozen CFB/CFR batch size: 0.
- Remaining known CFB/CFR open issue count: 0.

## Cycle 44 Dogfood Pass

After cycle 43 reached zero open issues, cycle 44 resumed algorithm development with the installed CodeFire binary.

Algorithm work:

- Added `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdownVerifier` to detect required-line and artifact-row drift in archive summary artifact archive summary artifact manifest Markdown.
- Added `REQ-AUTO-080`, `DES-AUTO-062`, `CODE-ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdownVerifier`, and `TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-artifact-archive-summary-artifact-manifest-markdown-verifier-detects-drift`.
- Algorithm unittest result: 186 tests pass.
- CodeFire scan opened 6 trace-change fires for the new REQ/DES/CODE/TEST atoms.
- All 6 fires were extinguished in the same cycle and sealed as `CF-COMMIT-75c6cd1b8a7c40c48f669cff`.

Cycle 44 issue status:

- Newly discovered CodeFire product issues: 0.
- Newly frozen CFB/CFR batch size: 0.
- Remaining known CFB/CFR open issue count: 0.

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
