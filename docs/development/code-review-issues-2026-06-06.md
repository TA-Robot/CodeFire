# Code Review Issues 2026-06-06

この文書は CodeFire 本体コードレビューの全体要約リストである。詳細は `docs/development/code-review-issues/` 配下の1 issue 1 fileで管理する。

## How To Use

- 全体の優先順位と対象領域はこのindexで見る。
- 実装修正に着手する場合は、各CFRの詳細ファイルを開き、Problem / Code Evidence / Recommended Fix / Acceptance Criteria / Test Planを確認する。
- dogfoodingで観測した不具合・UX issueは `docs/development/bug-backlog.md`、コードレビュー起点の改善候補はこのCFR系列で管理する。
- 修正完了時は詳細ファイルの `Status` を更新し、必要に応じて `history.md` と仕様文書へ反映する。

## Count

- Total: 160
- High: 36
- Medium: 121
- Low: 3
- Fixed / closed: 155
- Remaining open: 5

## Cycle 2 Source Review Expansion

Cycle 2ではdogfooding CFB-066..085を20件まで増やした後、CodeFire本体sourceを再レビューしてCFR-121..160を追加した。主なroot causeは、sealed commit/root validation、JSON envelope/next_actions schema drift、parser/help capability registry不足、storage/context/migrationのlarge repo対応不足である。

### Cycle 2 High Priority

| ID | Type | Area | Title | Detail |
|---|---|---|---|---|
| CFR-121 | correctness-risk | Commit validation | sealed commit validationがresolution_ledger rootを必須化していない | [CFR-121](code-review-issues/cfr-121.md) |
| CFR-122 | correctness-risk | Commit certificate | commit certificateがverification payloadと相互検証されていない | [CFR-122](code-review-issues/cfr-122.md) |
| CFR-123 | correctness-risk | Object validation | root validationがatom_indexやtrace_graphなどのpayload shapeを検証しない | [CFR-123](code-review-issues/cfr-123.md) |
| CFR-124 | correctness-risk | Fire lifecycle | manual fire作成がlock取得前の古い状態を元に実行される | [CFR-124](code-review-issues/cfr-124.md) |
| CFR-125 | correctness-risk | Batch link | link batch dry-runのinvalid結果が成功envelopeに包まれる | [CFR-125](code-review-issues/cfr-125.md) |
| CFR-126 | ux | Metrics | branch listが--metricsを受け付けるが出力へ反映しない | [CFR-126](code-review-issues/cfr-126.md) |
| CFR-127 | correctness-risk | Explain | explain verify-failureがnon-blocking warningをblockerとして数える | [CFR-127](code-review-issues/cfr-127.md) |
| CFR-128 | security | HTTP client | HTTP client response bodyに読み取り上限がない | [CFR-128](code-review-issues/cfr-128.md) |
| CFR-129 | correctness-risk | Remote merge | HTTP request-merge dry-runがhead検証なしで空head planを返す | [CFR-129](code-review-issues/cfr-129.md) |
| CFR-130 | correctness-risk | Commit durability | commit書き込みがmulti-object transactionとして扱われていない | [CFR-130](code-review-issues/cfr-130.md) |

### Cycle 2 Medium Priority

| ID | Area | Title | Detail |
|---|---|---|---|
| CFR-131 | CLI help | top-level help routingが主要commandを網羅していない | [CFR-131](code-review-issues/cfr-131.md) |
| CFR-132 | Migration | migrateのtarget formatがv0.6に固定されている | [CFR-132](code-review-issues/cfr-132.md) |
| CFR-133 | Repository layout | repo layout必須directory一覧がdoctorとmigrationで重複している | [CFR-133](code-review-issues/cfr-133.md) |
| CFR-134 | Storage report | storage quick modeのlargest object typeがunscanned固定になる | [CFR-134](code-review-issues/cfr-134.md) |
| CFR-135 | Storage report | storage largest object集計が全object情報を保持してからtruncateする | [CFR-135](code-review-issues/cfr-135.md) |
| CFR-136 | Context | contextのtruncated flagが省略件数を返さない | [CFR-136](code-review-issues/cfr-136.md) |
| CFR-137 | Context | branch context selectorが空selectionを返して実質的なbranch要約にならない | [CFR-137](code-review-issues/cfr-137.md) |
| CFR-138 | CLI parser | context parserが--path=などのequals形式に対応していない | [CFR-138](code-review-issues/cfr-138.md) |
| CFR-139 | CLI parser | commit/extinguish parserの--path形式が他commandと揃っていない | [CFR-139](code-review-issues/cfr-139.md) |
| CFR-140 | Verification JSON | verification diagnostic filterがpayloadを実際には絞り込まない | [CFR-140](code-review-issues/cfr-140.md) |
| CFR-141 | Next actions | verify成功時next_actionが変更有無を見ずcommitを勧める | [CFR-141](code-review-issues/cfr-141.md) |
| CFR-142 | Next actions | bounded_next_actionsのfallbackが常にstatus固定で文脈を失う | [CFR-142](code-review-issues/cfr-142.md) |
| CFR-143 | Automation JSON | next_actions schemaがmoduleごとに重複して揺れている | [CFR-143](code-review-issues/cfr-143.md) |
| CFR-144 | Explain | explain text rendererがnext_actionのid fieldを前提にしている | [CFR-144](code-review-issues/cfr-144.md) |
| CFR-145 | Fire dry-run | fire operation planが全item詳細を無制限に含む | [CFR-145](code-review-issues/cfr-145.md) |
| CFR-146 | Fire dry-run | fire operation planのnext_actionがpath情報を持たない | [CFR-146](code-review-issues/cfr-146.md) |
| CFR-147 | Evidence | evidence dry-run planが解決後cwdを返さない | [CFR-147](code-review-issues/cfr-147.md) |
| CFR-148 | Doctor | doctorがactive state file欠損を明示診断しない | [CFR-148](code-review-issues/cfr-148.md) |
| CFR-149 | Doctor | doctorがstate/open_firesの整合性を検証しない | [CFR-149](code-review-issues/cfr-149.md) |
| CFR-150 | Metrics | metrics対応がcommand capabilityとして管理されていない | [CFR-150](code-review-issues/cfr-150.md) |
| CFR-151 | Migration | migration next_actionsがautomation共通schemaを使っていない | [CFR-151](code-review-issues/cfr-151.md) |
| CFR-152 | Storage report | storage next_actionsがautomation共通schemaを使っていない | [CFR-152](code-review-issues/cfr-152.md) |
| CFR-153 | Doctor | doctor next_actionsがautomation共通schemaを使っていない | [CFR-153](code-review-issues/cfr-153.md) |
| CFR-154 | JSON envelope | plan result envelopeがoperation plan内next_actionsを昇格しない | [CFR-154](code-review-issues/cfr-154.md) |
| CFR-155 | JSON envelope | plan result envelopeがrepo_root以外のrepo fieldを読まない | [CFR-155](code-review-issues/cfr-155.md) |
| CFR-156 | Migration | migrateのJSON object走査にlimitやquick modeがない | [CFR-156](code-review-issues/cfr-156.md) |
| CFR-157 | Context | context atom traversalがlimitで落としたneighbor情報を説明しない | [CFR-157](code-review-issues/cfr-157.md) |
| CFR-158 | Fire identity | manual fire UIDがbase_commitを含み同じ意味のfireがcommit後に再発行される | [CFR-158](code-review-issues/cfr-158.md) |
| CFR-159 | Commit certificate | certificate resultの用語がverification resultとずれている | [CFR-159](code-review-issues/cfr-159.md) |
| CFR-160 | JSON envelope | data result envelopeのrepo抽出規則がcommandごとに弱い | [CFR-160](code-review-issues/cfr-160.md) |

### Cycle 2 Root Cause Buckets

| Root Cause | Issues |
|---|---|
| Commit/root validation contract不足 | [CFR-121](code-review-issues/cfr-121.md), [CFR-122](code-review-issues/cfr-122.md), [CFR-123](code-review-issues/cfr-123.md), [CFR-130](code-review-issues/cfr-130.md), [CFR-159](code-review-issues/cfr-159.md) |
| Operation locking/identity/dry-run検証不足 | [CFR-124](code-review-issues/cfr-124.md), [CFR-125](code-review-issues/cfr-125.md), [CFR-129](code-review-issues/cfr-129.md), [CFR-145](code-review-issues/cfr-145.md), [CFR-146](code-review-issues/cfr-146.md), [CFR-158](code-review-issues/cfr-158.md) |
| Command capability registry不足 | [CFR-126](code-review-issues/cfr-126.md), [CFR-131](code-review-issues/cfr-131.md), [CFR-132](code-review-issues/cfr-132.md), [CFR-138](code-review-issues/cfr-138.md), [CFR-139](code-review-issues/cfr-139.md), [CFR-150](code-review-issues/cfr-150.md) |
| Automation JSON/next_actions schema drift | [CFR-140](code-review-issues/cfr-140.md), [CFR-141](code-review-issues/cfr-141.md), [CFR-142](code-review-issues/cfr-142.md), [CFR-143](code-review-issues/cfr-143.md), [CFR-144](code-review-issues/cfr-144.md), [CFR-151](code-review-issues/cfr-151.md), [CFR-152](code-review-issues/cfr-152.md), [CFR-153](code-review-issues/cfr-153.md), [CFR-154](code-review-issues/cfr-154.md), [CFR-155](code-review-issues/cfr-155.md), [CFR-160](code-review-issues/cfr-160.md) |
| Large repo/observability対応不足 | [CFR-128](code-review-issues/cfr-128.md), [CFR-134](code-review-issues/cfr-134.md), [CFR-135](code-review-issues/cfr-135.md), [CFR-136](code-review-issues/cfr-136.md), [CFR-137](code-review-issues/cfr-137.md), [CFR-147](code-review-issues/cfr-147.md), [CFR-148](code-review-issues/cfr-148.md), [CFR-149](code-review-issues/cfr-149.md), [CFR-156](code-review-issues/cfr-156.md), [CFR-157](code-review-issues/cfr-157.md) |

## Current Open Expansion

Cycle 1 Phase Eで、dogfooding CFB-046..065を踏まえたsource reviewからCFR-081..120を追加した。これらは次version計画でroot causeごとに束ねて解消する。

### Open Priority Buckets

#### High

| ID | Type | Area | Title | Detail |
|---|---|---|---|---|
| CFR-081 | maintainability | CLI JSON output | JSON失敗経路が共通envelopeを通らない | [CFR-081](code-review-issues/cfr-081.md) |
| CFR-085 | bug | View CLI | `show --json` がhelp上の契約と実装で一致していない | [CFR-085](code-review-issues/cfr-085.md) |
| CFR-088 | performance | Patch export | patch exportがfile contentを無制限にbase64化し得る | [CFR-088](code-review-issues/cfr-088.md) |
| CFR-089 | performance | View/Diff | manifest diffが全file blobをメモリへ展開する | [CFR-089](code-review-issues/cfr-089.md) |
| CFR-095 | maintainability | Error model | CliErrorにstructured diagnosticへの変換責務がない | [CFR-095](code-review-issues/cfr-095.md) |
| CFR-100 | correctness-risk | Batch extinguish | batch extinguishが検証と適用を単一transactionで守れていない | [CFR-100](code-review-issues/cfr-100.md) |
| CFR-103 | correctness-risk | Storage remote | storage reportが存在しないremote rootを空remoteとして扱い得る | [CFR-103](code-review-issues/cfr-103.md) |
| CFR-109 | correctness-risk | Evidence capture | evidence commandの非ゼロ終了が成功証跡のように見える | [CFR-109](code-review-issues/cfr-109.md) |
| CFR-114 | correctness-risk | Context | contextがactive scan freshnessを検証しない | [CFR-114](code-review-issues/cfr-114.md) |

#### Medium / Low

| ID | Severity | Type | Area | Title | Detail |
|---|---|---|---|---|---|
| CFR-082 | medium | ux | CLI help | command help metadataがdispatchと分離して不完全になっている | [CFR-082](code-review-issues/cfr-082.md) |
| CFR-083 | medium | maintainability | Automation JSON | debug系commandがautomation envelopeを迂回している | [CFR-083](code-review-issues/cfr-083.md) |
| CFR-084 | medium | ux | Remote CLI | remote read commandがJSON出力に対応していない | [CFR-084](code-review-issues/cfr-084.md) |
| CFR-086 | medium | maintainability | Diff CLI | `diff --json` がcommand_result envelopeではなくraw JSONを返す | [CFR-086](code-review-issues/cfr-086.md) |
| CFR-087 | medium | ux | Review/Patch CLI | review-packとpatch exportにautomation向けJSON envelopeがない | [CFR-087](code-review-issues/cfr-087.md) |
| CFR-090 | medium | correctness-risk | HTTP remote view | HTTP remote commitish解決の一時directory名が競合し得る | [CFR-090](code-review-issues/cfr-090.md) |
| CFR-092 | medium | ux | Branch CLI | branch subcommandのunsupported pathがstructured errorにならない | [CFR-092](code-review-issues/cfr-092.md) |
| CFR-093 | medium | correctness-risk | Status JSON | status JSONがrepo root取得失敗を握りつぶす | [CFR-093](code-review-issues/cfr-093.md) |
| CFR-094 | medium | ux | Link CLI | link commandのhelp routingとJSON error contractが弱い | [CFR-094](code-review-issues/cfr-094.md) |
| CFR-096 | medium | ux | Automation next_actions | status next_actionsがchanged countやfreshnessを見ずにverifyを勧める | [CFR-096](code-review-issues/cfr-096.md) |
| CFR-097 | medium | ux | Scan JSON | scan JSONの`changed_atoms`がID配列だけで詳細不足 | [CFR-097](code-review-issues/cfr-097.md) |
| CFR-098 | medium | ux | Scan next_actions | clean scanのnext_actionsがverify誘導に寄りすぎる | [CFR-098](code-review-issues/cfr-098.md) |
| CFR-099 | medium | compatibility | Batch parser | batch JSON parserがtop-level arrayをJSONとして扱わない | [CFR-099](code-review-issues/cfr-099.md) |
| CFR-101 | medium | performance | Batch operation plan | batch extinguish dry-run planがitem詳細を無制限に含む | [CFR-101](code-review-issues/cfr-101.md) |
| CFR-102 | medium | ux | Batch result JSON | batch extinguish成功JSONが適用結果の直接summaryを持たない | [CFR-102](code-review-issues/cfr-102.md) |
| CFR-104 | medium | correctness-risk | Filesystem scan helper | `collect_files` がmissing rootとempty directoryを区別しない | [CFR-104](code-review-issues/cfr-104.md) |
| CFR-105 | medium | performance | Storage report | storage full scanがobject JSONを毎回全件parseする | [CFR-105](code-review-issues/cfr-105.md) |
| CFR-106 | low | ux | Storage report | storage quick/full modeの保証範囲がJSONに明示されない | [CFR-106](code-review-issues/cfr-106.md) |
| CFR-107 | medium | ux | Evidence capture | evidence JSONが解決後のcommand cwdを返さない | [CFR-107](code-review-issues/cfr-107.md) |
| CFR-108 | medium | ux | Evidence dry-run | evidence dry-runが空のevidence_idを返して実体と紛らわしい | [CFR-108](code-review-issues/cfr-108.md) |
| CFR-110 | medium | maintainability | HTTP client | HTTP error responseがCliError文字列へ潰される | [CFR-110](code-review-issues/cfr-110.md) |
| CFR-111 | low | ux | HTTP client config | `CODEFIRE_HTTP_TIMEOUT_MS` の不正値が診断されない | [CFR-111](code-review-issues/cfr-111.md) |
| CFR-113 | medium | ux | Remote upload JSON | upload dry-run envelopeのrepo/next_actionsがtop-levelで弱い | [CFR-113](code-review-issues/cfr-113.md) |
| CFR-115 | medium | ux | Context/Explain | context/explainのunknown target errorがJSON contractへ乗らない | [CFR-115](code-review-issues/cfr-115.md) |
| CFR-117 | medium | maintainability | Core scan model | ScanResultがchanged atom詳細とnon-Atom変更を同じ粒度で表現しない | [CFR-117](code-review-issues/cfr-117.md) |
| CFR-118 | medium | performance | Object lookup | unknown object ID lookupがsubdir全走査へ落ちる | [CFR-118](code-review-issues/cfr-118.md) |
| CFR-119 | medium | correctness-risk | Durability helpers | object store writeのfsync/rename方針がactive metadata writeと別々に進化しやすい | [CFR-119](code-review-issues/cfr-119.md) |
| CFR-120 | medium | maintainability | Development process | source reviewで見つかったissue群をroot cause計画へ自動接続できない | [CFR-120](code-review-issues/cfr-120.md) |

### Fixed After v0.7

| ID | Fixed in | Summary | Evidence |
|---|---|---|---|
| CFR-091 | v0.9 Phase A | `init --help` をparser前にread-only helpとして処理 | `command_help_routes_before_mutating_parsers` |
| CFR-131 | v1.0 Phase 1a | `context` / `explain` / `migrate` / debug read-only commandsのhelp routingを追加 | `command_help_routes_before_mutating_parsers` |
| CFR-081 | global burn-down | top-level JSON failure fallbackをcommon command_result envelopeへ統一 | `cargo run -p codefire-cli --bin codefire-rs -- definitely-not-a-command --json` |
| CFR-082 | global burn-down | command registryを追加し、help/completion/capability JSONのcommand listを共有 | `command_capabilities_are_shared_by_help_completion_and_metrics_parser` |
| CFR-083 | global burn-down | debug read-only commandの`--json` envelopeと`--path` contractを確認 | `command_help_routes_before_mutating_parsers` |
| CFR-084 | global burn-down | remote `list --json` / `request-list --json` をcommand_result envelope化済みとしてclose | `file_remote_upload_clone_show_diff_and_merge_request_flow` |
| CFR-094 | global burn-down | `link --help` routingとJSON error contractをfixed済みとしてclose | `command_help_routes_before_mutating_parsers` |
| CFR-095 | global burn-down | `CliError::diagnostic()` で全variantをstructured diagnosticへ写像 | `cli_error_diagnostics_cover_representative_error_classes` |
| CFR-150 | global burn-down | command capability registryに`supports_metrics`を追加し`capabilities --json`で公開 | `command_capabilities_are_shared_by_help_completion_and_metrics_parser` |
| CFR-113 | global burn-down | upload dry-run envelopeがtop-level repo/next_actionsを返す既存実装をclose auditで確認 | `file_remote_upload_clone_show_diff_and_merge_request_flow` |
| CFR-093 | global burn-down | status JSONのrepo context取得失敗を握りつぶさずstructured diagnosticにした | `status_context_mismatch_returns_structured_diagnostic` |
| CFR-090 | global burn-down | HTTP remote view bundle temp directoryを呼び出しごとの一意pathとDrop cleanupへ変更 | `view::tests::http_bundle_temp_dirs_are_unique_and_cleaned_on_drop` |
| CFR-110 | global burn-down | HTTP error responseを`RemoteDiagnostic`として保持しremote diagnostic kind/statusをJSONへ伝播 | `http::tests::http_remote_error_preserves_structured_diagnostic` |
| CFR-116 | global burn-down | VerificationCommandのtimeout/output/env/allow_failure契約をcore型とlocal runnerへ追加 | `verification_policy_parses_command_execution_contract`; `verification_command_contract_controls_failure_timeout_and_output` |
| CFR-122 | global burn-down | sealed commit validationでcertificateとverification root result/countを相互検証 | `validate_sealed_commit_rejects_certificate_verification_mismatch` |
| CFR-123 | global burn-down | required root payload shape validatorsとcoverage testを追加 | `validate_sealed_commit_rejects_malformed_root_payloads`; `current_required_commit_roots_have_payload_validators` |
| CFR-112 | global burn-down | file remote upload dry-runで既存remote layoutをread-only検証しplanへvalidationsを追加 | `file_remote_upload_dry_run_rejects_existing_invalid_layout`; `file_remote_upload_clone_show_diff_and_merge_request_flow` |
| CFR-129 | global burn-down | HTTP request-merge dry-runでremote branch headを解決し空head planを廃止 | `http_request_merge_dry_run_validates_branch_heads` |

### Open Area View

| Area | Issues |
|---|---|
| Automation JSON / errors | none |
| CLI help and parser consistency | none |
| View, diff, patch | [CFR-089](code-review-issues/cfr-089.md) |
| Remote | none |
| Next actions and scan model | none |
| Batch | none |
| Storage and filesystem scan | [CFR-105](code-review-issues/cfr-105.md) |
| Evidence | none |
| Context and explain | none |
| HTTP config | none |
| Verification policy | none |
| Object store and durability | [CFR-119](code-review-issues/cfr-119.md), [CFR-130](code-review-issues/cfr-130.md) |
| Development process | [CFR-120](code-review-issues/cfr-120.md) |

## v0.7 Progress

- Batch 1 fixed: [CFR-005](code-review-issues/cfr-005.md), [CFR-006](code-review-issues/cfr-006.md), [CFR-010](code-review-issues/cfr-010.md), [CFR-031](code-review-issues/cfr-031.md), [CFR-032](code-review-issues/cfr-032.md), [CFR-047](code-review-issues/cfr-047.md), [CFR-058](code-review-issues/cfr-058.md), [CFR-067](code-review-issues/cfr-067.md), [CFR-077](code-review-issues/cfr-077.md)
- Batch 2/8 fixed: [CFR-001](code-review-issues/cfr-001.md), [CFR-002](code-review-issues/cfr-002.md), [CFR-003](code-review-issues/cfr-003.md), [CFR-004](code-review-issues/cfr-004.md), [CFR-007](code-review-issues/cfr-007.md), [CFR-008](code-review-issues/cfr-008.md), [CFR-009](code-review-issues/cfr-009.md), [CFR-036](code-review-issues/cfr-036.md), [CFR-037](code-review-issues/cfr-037.md), [CFR-038](code-review-issues/cfr-038.md), [CFR-039](code-review-issues/cfr-039.md), [CFR-040](code-review-issues/cfr-040.md), [CFR-057](code-review-issues/cfr-057.md), [CFR-059](code-review-issues/cfr-059.md), [CFR-060](code-review-issues/cfr-060.md), [CFR-061](code-review-issues/cfr-061.md), [CFR-062](code-review-issues/cfr-062.md), [CFR-063](code-review-issues/cfr-063.md), [CFR-064](code-review-issues/cfr-064.md), [CFR-065](code-review-issues/cfr-065.md), [CFR-066](code-review-issues/cfr-066.md), [CFR-071](code-review-issues/cfr-071.md), [CFR-072](code-review-issues/cfr-072.md), [CFR-073](code-review-issues/cfr-073.md), [CFR-074](code-review-issues/cfr-074.md), [CFR-075](code-review-issues/cfr-075.md), [CFR-076](code-review-issues/cfr-076.md), [CFR-080](code-review-issues/cfr-080.md)
- Batch 3 fixed: [CFR-011](code-review-issues/cfr-011.md), [CFR-012](code-review-issues/cfr-012.md), [CFR-013](code-review-issues/cfr-013.md), [CFR-014](code-review-issues/cfr-014.md), [CFR-015](code-review-issues/cfr-015.md), [CFR-016](code-review-issues/cfr-016.md), [CFR-017](code-review-issues/cfr-017.md), [CFR-018](code-review-issues/cfr-018.md)
- Batch 4 fixed: [CFR-019](code-review-issues/cfr-019.md), [CFR-020](code-review-issues/cfr-020.md), [CFR-021](code-review-issues/cfr-021.md), [CFR-022](code-review-issues/cfr-022.md), [CFR-023](code-review-issues/cfr-023.md), [CFR-024](code-review-issues/cfr-024.md)
- Batch 5 fixed: [CFR-025](code-review-issues/cfr-025.md), [CFR-026](code-review-issues/cfr-026.md), [CFR-027](code-review-issues/cfr-027.md), [CFR-028](code-review-issues/cfr-028.md), [CFR-029](code-review-issues/cfr-029.md)
- Batch 6 fixed: [CFR-030](code-review-issues/cfr-030.md), [CFR-033](code-review-issues/cfr-033.md), [CFR-034](code-review-issues/cfr-034.md), [CFR-035](code-review-issues/cfr-035.md), [CFR-068](code-review-issues/cfr-068.md), [CFR-069](code-review-issues/cfr-069.md), [CFR-070](code-review-issues/cfr-070.md), [CFR-078](code-review-issues/cfr-078.md), [CFR-079](code-review-issues/cfr-079.md)
- Batch 7 partial fixed: [CFR-041](code-review-issues/cfr-041.md), [CFR-042](code-review-issues/cfr-042.md), [CFR-043](code-review-issues/cfr-043.md), [CFR-044](code-review-issues/cfr-044.md), [CFR-045](code-review-issues/cfr-045.md), [CFR-046](code-review-issues/cfr-046.md), [CFR-048](code-review-issues/cfr-048.md), [CFR-049](code-review-issues/cfr-049.md), [CFR-050](code-review-issues/cfr-050.md), [CFR-051](code-review-issues/cfr-051.md), [CFR-052](code-review-issues/cfr-052.md), [CFR-053](code-review-issues/cfr-053.md), [CFR-054](code-review-issues/cfr-054.md), [CFR-055](code-review-issues/cfr-055.md), [CFR-056](code-review-issues/cfr-056.md).
- Global burn-down fixed: [CFR-085](code-review-issues/cfr-085.md), [CFR-086](code-review-issues/cfr-086.md), [CFR-087](code-review-issues/cfr-087.md), [CFR-088](code-review-issues/cfr-088.md), [CFR-096](code-review-issues/cfr-096.md), [CFR-097](code-review-issues/cfr-097.md), [CFR-098](code-review-issues/cfr-098.md), [CFR-099](code-review-issues/cfr-099.md), [CFR-100](code-review-issues/cfr-100.md), [CFR-101](code-review-issues/cfr-101.md), [CFR-102](code-review-issues/cfr-102.md), [CFR-103](code-review-issues/cfr-103.md), [CFR-104](code-review-issues/cfr-104.md), [CFR-106](code-review-issues/cfr-106.md), [CFR-107](code-review-issues/cfr-107.md), [CFR-108](code-review-issues/cfr-108.md), [CFR-109](code-review-issues/cfr-109.md), [CFR-112](code-review-issues/cfr-112.md), [CFR-115](code-review-issues/cfr-115.md), [CFR-116](code-review-issues/cfr-116.md), [CFR-122](code-review-issues/cfr-122.md), [CFR-123](code-review-issues/cfr-123.md), [CFR-124](code-review-issues/cfr-124.md), [CFR-125](code-review-issues/cfr-125.md), [CFR-126](code-review-issues/cfr-126.md), [CFR-127](code-review-issues/cfr-127.md), [CFR-129](code-review-issues/cfr-129.md), [CFR-132](code-review-issues/cfr-132.md), [CFR-133](code-review-issues/cfr-133.md), [CFR-134](code-review-issues/cfr-134.md), [CFR-135](code-review-issues/cfr-135.md), [CFR-136](code-review-issues/cfr-136.md), [CFR-138](code-review-issues/cfr-138.md), [CFR-139](code-review-issues/cfr-139.md), [CFR-140](code-review-issues/cfr-140.md), [CFR-141](code-review-issues/cfr-141.md), [CFR-142](code-review-issues/cfr-142.md), [CFR-143](code-review-issues/cfr-143.md), [CFR-144](code-review-issues/cfr-144.md), [CFR-145](code-review-issues/cfr-145.md), [CFR-146](code-review-issues/cfr-146.md), [CFR-147](code-review-issues/cfr-147.md), [CFR-148](code-review-issues/cfr-148.md), [CFR-149](code-review-issues/cfr-149.md), [CFR-151](code-review-issues/cfr-151.md), [CFR-152](code-review-issues/cfr-152.md), [CFR-153](code-review-issues/cfr-153.md), [CFR-154](code-review-issues/cfr-154.md), [CFR-155](code-review-issues/cfr-155.md), [CFR-156](code-review-issues/cfr-156.md), [CFR-157](code-review-issues/cfr-157.md), [CFR-158](code-review-issues/cfr-158.md), [CFR-159](code-review-issues/cfr-159.md), [CFR-160](code-review-issues/cfr-160.md).
- Plan: [v0.7-issue-improvement-plan.md](v0.7-issue-improvement-plan.md)

## Priority Buckets

### High

| ID | Type | Area | Title | Detail |
|---|---|---|---|---|
| CFR-001 | maintainability | CLI dispatch | `main.rs` の command dispatch が巨大化している | [CFR-001](code-review-issues/cfr-001.md) |
| CFR-002 | maintainability | CLI domain model | CLI内部型が `main.rs` に集中している | [CFR-002](code-review-issues/cfr-002.md) |
| CFR-003 | correctness-risk | State model | `status` と `scan` の状態算出が別実装になっている | [CFR-003](code-review-issues/cfr-003.md) |
| CFR-004 | correctness-risk | State model | `verify` 成功後の状態更新責務が分散している | [CFR-004](code-review-issues/cfr-004.md) |
| CFR-007 | maintainability | JSON output | `--json` 出力が command envelope に統一されていない | [CFR-007](code-review-issues/cfr-007.md) |
| CFR-010 | correctness-risk | Atomic write | `write_json_atomic` の一時ファイル名が固定で衝突し得る | [CFR-010](code-review-issues/cfr-010.md) |
| CFR-019 | correctness-risk | Atom extraction | 明示 `cf-atom` のcontent hashがマーカー行だけを見ている | [CFR-019](code-review-issues/cfr-019.md) |
| CFR-025 | performance | Diff | rename detectionが削除×追加の全組み合わせLCSになっている | [CFR-025](code-review-issues/cfr-025.md) |
| CFR-026 | performance | Diff | 類似度計算が巨大ファイルのLCSを無制限に使う | [CFR-026](code-review-issues/cfr-026.md) |
| CFR-031 | security | HTTP server | HTTP request body size上限がない | [CFR-031](code-review-issues/cfr-031.md) |
| CFR-032 | correctness-risk | HTTP server | 短いHTTP bodyを不完全リクエストとして拒否していない | [CFR-032](code-review-issues/cfr-032.md) |
| CFR-035 | correctness-risk | Remote generation | remote generation更新が独立lockされていない | [CFR-035](code-review-issues/cfr-035.md) |
| CFR-041 | correctness-risk | Evidence capture | `evidence add --from-command` にtimeoutがない | [CFR-041](code-review-issues/cfr-041.md) |
| CFR-047 | correctness-risk | Link batch | link batch appendの一時ファイル名が固定 | [CFR-047](code-review-issues/cfr-047.md) |
| CFR-063 | maintainability | Install | install.shがrelease binaryの検証をしない | [CFR-063](code-review-issues/cfr-063.md) |

### Medium / Low

| ID | Severity | Type | Area | Title | Detail |
|---|---|---|---|---|---|
| CFR-005 | medium | ux | Scan output | clean scanの空状態表示が曖昧 | [CFR-005](code-review-issues/cfr-005.md) |
| CFR-006 | medium | ux | Verify output | verify成功時に要約が出ない | [CFR-006](code-review-issues/cfr-006.md) |
| CFR-008 | medium | maintainability | CLI parser | 引数parserが手書きで重複している | [CFR-008](code-review-issues/cfr-008.md) |
| CFR-009 | medium | compatibility | CLI naming | help/next actionが `codefire-rs` に固定されている | [CFR-009](code-review-issues/cfr-009.md) |
| CFR-011 | medium | correctness-risk | Atomic write | atomic write後にfsyncがない | [CFR-011](code-review-issues/cfr-011.md) |
| CFR-012 | medium | correctness-risk | Object store | object書き込み完了前クラッシュの検出粒度が弱い | [CFR-012](code-review-issues/cfr-012.md) |
| CFR-013 | medium | performance | Object lookup | object検索が全subdir走査になっている | [CFR-013](code-review-issues/cfr-013.md) |
| CFR-014 | medium | correctness-risk | Commit validation | sealed commit検証がroot参照の到達性を限定的にしか見ていない | [CFR-014](code-review-issues/cfr-014.md) |
| CFR-015 | medium | maintainability | Object schema | object type mappingが複数箇所に散っている | [CFR-015](code-review-issues/cfr-015.md) |
| CFR-016 | medium | compatibility | Object ID | 12桁digest IDは大規模化時の衝突余裕が小さい | [CFR-016](code-review-issues/cfr-016.md) |
| CFR-017 | medium | compatibility | Fire model | fire UIDにSHA-1由来の短縮値が残っている | [CFR-017](code-review-issues/cfr-017.md) |
| CFR-018 | medium | correctness-risk | Fire model | display fire番号がledger長依存で安定性が弱い | [CFR-018](code-review-issues/cfr-018.md) |
| CFR-020 | medium | correctness-risk | Atom extraction | Atom ID validationが経路ごとに違う | [CFR-020](code-review-issues/cfr-020.md) |
| CFR-021 | medium | correctness-risk | Markdown parser | Markdown Atom抽出がコードフェンスを考慮しない | [CFR-021](code-review-issues/cfr-021.md) |
| CFR-022 | medium | correctness-risk | YAML parser | links/policy parserが限定的な手書きYAMLになっている | [CFR-022](code-review-issues/cfr-022.md) |
| CFR-023 | medium | correctness-risk | Link model | trace link IDが生Atom ID連結で長大化・衝突曖昧化し得る | [CFR-023](code-review-issues/cfr-023.md) |
| CFR-024 | medium | performance | Required links | required link計算がAtomごと全link走査になっている | [CFR-024](code-review-issues/cfr-024.md) |
| CFR-027 | medium | performance | Diff | diff出力にhunk化・context制限がない | [CFR-027](code-review-issues/cfr-027.md) |
| CFR-028 | medium | correctness-risk | Merge | conflict markerがbinary/text混在を十分に区別しない | [CFR-028](code-review-issues/cfr-028.md) |
| CFR-029 | medium | correctness-risk | Merge | common ancestor計算が履歴全探索寄り | [CFR-029](code-review-issues/cfr-029.md) |
| CFR-030 | medium | security | HTTP client | HTTP clientに接続・読み取りtimeoutがない | [CFR-030](code-review-issues/cfr-030.md) |
| CFR-033 | medium | security | HTTP server | HTTP serverが単一thread逐次処理 | [CFR-033](code-review-issues/cfr-033.md) |
| CFR-034 | medium | correctness-risk | HTTP remote | upload時のtmp directory名が衝突し得る | [CFR-034](code-review-issues/cfr-034.md) |
| CFR-036 | medium | security | Signatures | request nonce cacheの上限・GC方針が見えない | [CFR-036](code-review-issues/cfr-036.md) |
| CFR-037 | medium | security | Signatures | commit signatureの時刻検証が文字列比較に依存している | [CFR-037](code-review-issues/cfr-037.md) |
| CFR-038 | medium | maintainability | Dual implementation | Python旧本体とRust本体が二重実装として残っている | [CFR-038](code-review-issues/cfr-038.md) |
| CFR-039 | medium | test-gap | Tests | integration testが巨大単一ファイルに集中している | [CFR-039](code-review-issues/cfr-039.md) |
| CFR-040 | medium | maintainability | Config/time/hash | 低レベルutilityが各crate/moduleに重複している | [CFR-040](code-review-issues/cfr-040.md) |
| CFR-042 | medium | security | Evidence capture | 証跡commandがshell経由で実行される | [CFR-042](code-review-issues/cfr-042.md) |
| CFR-043 | medium | correctness-risk | Evidence capture | command cwdのdefaultがopen/repo rootではなく現在dirになっている | [CFR-043](code-review-issues/cfr-043.md) |
| CFR-044 | medium | security | Evidence artifacts | artifact_refに絶対pathを保存している | [CFR-044](code-review-issues/cfr-044.md) |
| CFR-045 | medium | performance | Evidence artifacts | artifact hash計算にサイズ上限やstreaming policyがない | [CFR-045](code-review-issues/cfr-045.md) |
| CFR-046 | medium | maintainability | Batch parser | batch YAML parserが複数moduleに重複している | [CFR-046](code-review-issues/cfr-046.md) |
| CFR-048 | medium | correctness-risk | Link batch | link batch appendが既存YAML構造を保てない可能性がある | [CFR-048](code-review-issues/cfr-048.md) |
| CFR-049 | medium | ux | Link batch | link batchは失敗時に全件diagnosticを返さない | [CFR-049](code-review-issues/cfr-049.md) |
| CFR-050 | medium | correctness-risk | Doctor | doctorのrepo layout必須dir一覧が古い | [CFR-050](code-review-issues/cfr-050.md) |
| CFR-051 | medium | performance | Doctor | doctorがobject JSONを全件parseする | [CFR-051](code-review-issues/cfr-051.md) |
| CFR-052 | medium | correctness-risk | Doctor | doctorがactive stateのJSON shapeを検証していない | [CFR-052](code-review-issues/cfr-052.md) |
| CFR-053 | medium | correctness-risk | Doctor | opened registryのopen path整合性をdoctorが検証していない | [CFR-053](code-review-issues/cfr-053.md) |
| CFR-054 | medium | maintainability | Doctor | doctor issue severityがerror/warningの2段階に閉じている | [CFR-054](code-review-issues/cfr-054.md) |
| CFR-055 | medium | performance | Storage report | storage reportが全fileを再帰走査する | [CFR-055](code-review-issues/cfr-055.md) |
| CFR-056 | medium | correctness-risk | Storage report | storage reportのobject type判定がrecord wrapperを考慮していない可能性がある | [CFR-056](code-review-issues/cfr-056.md) |
| CFR-057 | medium | ux | Storage report | storage next_actionが `codefire-rs` 固定 | [CFR-057](code-review-issues/cfr-057.md) |
| CFR-058 | medium | security | TLS | `CODEFIRE_TLS_INSECURE` が存在だけで証明書検証を無効化する | [CFR-058](code-review-issues/cfr-058.md) |
| CFR-059 | medium | security | TLS | TLS serverがclient authをサポートしない | [CFR-059](code-review-issues/cfr-059.md) |
| CFR-060 | low | compatibility | TLS | private key parserがPKCS8/RSAのみ対応 | [CFR-060](code-review-issues/cfr-060.md) |
| CFR-061 | medium | maintainability | Metrics | metricsがtotalのみでphase timingを持たない | [CFR-061](code-review-issues/cfr-061.md) |
| CFR-062 | medium | performance | Metrics | metricsにcache disabled固定値だけが出る | [CFR-062](code-review-issues/cfr-062.md) |
| CFR-064 | medium | compatibility | Install | install.shにdry-runがない | [CFR-064](code-review-issues/cfr-064.md) |
| CFR-065 | medium | correctness-risk | Install | completion生成失敗時にbinary installだけ完了した状態になる | [CFR-065](code-review-issues/cfr-065.md) |
| CFR-066 | medium | security | Install | install先prefixの安全性チェックが弱い | [CFR-066](code-review-issues/cfr-066.md) |
| CFR-067 | medium | correctness-risk | Remote idempotency | remote idempotency keyが空文字をOptionとして受ける | [CFR-067](code-review-issues/cfr-067.md) |
| CFR-068 | medium | security | Remote idempotency | idempotency recordにpayload全文を保存している | [CFR-068](code-review-issues/cfr-068.md) |
| CFR-069 | medium | performance | Remote idempotency | idempotency recordのretention/GCが見えない | [CFR-069](code-review-issues/cfr-069.md) |
| CFR-070 | medium | correctness-risk | Remote idempotency | idempotency resultにplanを保存して再利用している | [CFR-070](code-review-issues/cfr-070.md) |
| CFR-071 | medium | maintainability | Automation schema | command_result schema versionが文字列直書き | [CFR-071](code-review-issues/cfr-071.md) |
| CFR-072 | medium | ux | Automation next_actions | next_actionsが多すぎる場合の上限がない | [CFR-072](code-review-issues/cfr-072.md) |
| CFR-073 | medium | correctness-risk | Automation diagnostics | missing required linksが常にerror severity | [CFR-073](code-review-issues/cfr-073.md) |
| CFR-074 | medium | maintainability | Context | context snapshotがscan resultを再構築している | [CFR-074](code-review-issues/cfr-074.md) |
| CFR-075 | medium | correctness-risk | Context | contextが固定時刻でscanを構築している | [CFR-075](code-review-issues/cfr-075.md) |
| CFR-076 | medium | performance | Context | context depth traversalに出力上限がない | [CFR-076](code-review-issues/cfr-076.md) |
| CFR-077 | medium | correctness-risk | Remote URL parser | cf+http URL parserが余分なpath segmentを許容する | [CFR-077](code-review-issues/cfr-077.md) |
| CFR-078 | medium | correctness-risk | HTTP parser | HTTP URL parserがIPv6 authorityを扱いにくい | [CFR-078](code-review-issues/cfr-078.md) |
| CFR-079 | medium | security | HTTP parser | HTTP request method/pathの正規化が弱い | [CFR-079](code-review-issues/cfr-079.md) |
| CFR-080 | medium | maintainability | Release docs | Python fallback専用commandの移行状態がコードから検証されない | [CFR-080](code-review-issues/cfr-080.md) |

## Area View

| Area | Issues |
|---|---|
| Atom extraction | [CFR-019](code-review-issues/cfr-019.md), [CFR-020](code-review-issues/cfr-020.md) |
| Atomic write | [CFR-010](code-review-issues/cfr-010.md), [CFR-011](code-review-issues/cfr-011.md) |
| Automation diagnostics | [CFR-073](code-review-issues/cfr-073.md) |
| Automation next_actions | [CFR-072](code-review-issues/cfr-072.md) |
| Automation schema | [CFR-071](code-review-issues/cfr-071.md) |
| Batch parser | [CFR-046](code-review-issues/cfr-046.md) |
| CLI dispatch | [CFR-001](code-review-issues/cfr-001.md) |
| CLI domain model | [CFR-002](code-review-issues/cfr-002.md) |
| CLI naming | [CFR-009](code-review-issues/cfr-009.md) |
| CLI parser | [CFR-008](code-review-issues/cfr-008.md) |
| Commit validation | [CFR-014](code-review-issues/cfr-014.md) |
| Config/time/hash | [CFR-040](code-review-issues/cfr-040.md) |
| Context | [CFR-074](code-review-issues/cfr-074.md), [CFR-075](code-review-issues/cfr-075.md), [CFR-076](code-review-issues/cfr-076.md) |
| Diff | [CFR-025](code-review-issues/cfr-025.md), [CFR-026](code-review-issues/cfr-026.md), [CFR-027](code-review-issues/cfr-027.md) |
| Doctor | [CFR-050](code-review-issues/cfr-050.md), [CFR-051](code-review-issues/cfr-051.md), [CFR-052](code-review-issues/cfr-052.md), [CFR-053](code-review-issues/cfr-053.md), [CFR-054](code-review-issues/cfr-054.md) |
| Dual implementation | [CFR-038](code-review-issues/cfr-038.md) |
| Evidence artifacts | [CFR-044](code-review-issues/cfr-044.md), [CFR-045](code-review-issues/cfr-045.md) |
| Evidence capture | [CFR-041](code-review-issues/cfr-041.md), [CFR-042](code-review-issues/cfr-042.md), [CFR-043](code-review-issues/cfr-043.md) |
| Fire model | [CFR-017](code-review-issues/cfr-017.md), [CFR-018](code-review-issues/cfr-018.md) |
| HTTP client | [CFR-030](code-review-issues/cfr-030.md) |
| HTTP parser | [CFR-078](code-review-issues/cfr-078.md), [CFR-079](code-review-issues/cfr-079.md) |
| HTTP remote | [CFR-034](code-review-issues/cfr-034.md) |
| HTTP server | [CFR-031](code-review-issues/cfr-031.md), [CFR-032](code-review-issues/cfr-032.md), [CFR-033](code-review-issues/cfr-033.md) |
| Install | [CFR-063](code-review-issues/cfr-063.md), [CFR-064](code-review-issues/cfr-064.md), [CFR-065](code-review-issues/cfr-065.md), [CFR-066](code-review-issues/cfr-066.md) |
| JSON output | [CFR-007](code-review-issues/cfr-007.md) |
| Link batch | [CFR-047](code-review-issues/cfr-047.md), [CFR-048](code-review-issues/cfr-048.md), [CFR-049](code-review-issues/cfr-049.md) |
| Link model | [CFR-023](code-review-issues/cfr-023.md) |
| Markdown parser | [CFR-021](code-review-issues/cfr-021.md) |
| Merge | [CFR-028](code-review-issues/cfr-028.md), [CFR-029](code-review-issues/cfr-029.md) |
| Metrics | [CFR-061](code-review-issues/cfr-061.md), [CFR-062](code-review-issues/cfr-062.md) |
| Object ID | [CFR-016](code-review-issues/cfr-016.md) |
| Object lookup | [CFR-013](code-review-issues/cfr-013.md) |
| Object schema | [CFR-015](code-review-issues/cfr-015.md) |
| Object store | [CFR-012](code-review-issues/cfr-012.md) |
| Release docs | [CFR-080](code-review-issues/cfr-080.md) |
| Remote URL parser | [CFR-077](code-review-issues/cfr-077.md) |
| Remote generation | [CFR-035](code-review-issues/cfr-035.md) |
| Remote idempotency | [CFR-067](code-review-issues/cfr-067.md), [CFR-068](code-review-issues/cfr-068.md), [CFR-069](code-review-issues/cfr-069.md), [CFR-070](code-review-issues/cfr-070.md) |
| Required links | [CFR-024](code-review-issues/cfr-024.md) |
| Scan output | [CFR-005](code-review-issues/cfr-005.md) |
| Signatures | [CFR-036](code-review-issues/cfr-036.md), [CFR-037](code-review-issues/cfr-037.md) |
| State model | [CFR-003](code-review-issues/cfr-003.md), [CFR-004](code-review-issues/cfr-004.md) |
| Storage report | [CFR-055](code-review-issues/cfr-055.md), [CFR-056](code-review-issues/cfr-056.md), [CFR-057](code-review-issues/cfr-057.md) |
| TLS | [CFR-058](code-review-issues/cfr-058.md), [CFR-059](code-review-issues/cfr-059.md), [CFR-060](code-review-issues/cfr-060.md) |
| Tests | [CFR-039](code-review-issues/cfr-039.md) |
| Verify output | [CFR-006](code-review-issues/cfr-006.md) |
| YAML parser | [CFR-022](code-review-issues/cfr-022.md) |

## Full List

| ID | Severity | Type | Area | Title | Source | Detail |
|---|---|---|---|---|---|---|
| CFR-001 | high | maintainability | CLI dispatch | `main.rs` の command dispatch が巨大化している | `crates/codefire-cli/src/main.rs:109` | [CFR-001](code-review-issues/cfr-001.md) |
| CFR-002 | high | maintainability | CLI domain model | CLI内部型が `main.rs` に集中している | `crates/codefire-cli/src/main.rs:970` | [CFR-002](code-review-issues/cfr-002.md) |
| CFR-003 | high | correctness-risk | State model | `status` と `scan` の状態算出が別実装になっている | `crates/codefire-cli/src/main.rs:1207`, `crates/codefire-cli/src/main.rs:4008` | [CFR-003](code-review-issues/cfr-003.md) |
| CFR-004 | high | correctness-risk | State model | `verify` 成功後の状態更新責務が分散している | `crates/codefire-cli/src/main.rs:202`, `crates/codefire-cli/src/verification.rs:58` | [CFR-004](code-review-issues/cfr-004.md) |
| CFR-005 | medium | ux | Scan output | clean scanの空状態表示が曖昧 | `crates/codefire-cli/src/main.rs:1213` | [CFR-005](code-review-issues/cfr-005.md) |
| CFR-006 | medium | ux | Verify output | verify成功時に要約が出ない | `crates/codefire-cli/src/verification.rs:63` | [CFR-006](code-review-issues/cfr-006.md) |
| CFR-007 | high | maintainability | JSON output | `--json` 出力が command envelope に統一されていない | `crates/codefire-cli/src/main.rs:180`, `crates/codefire-cli/src/main.rs:292` | [CFR-007](code-review-issues/cfr-007.md) |
| CFR-008 | medium | maintainability | CLI parser | 引数parserが手書きで重複している | `crates/codefire-cli/src/main.rs:1273`, `crates/codefire-cli/src/main.rs:1354` | [CFR-008](code-review-issues/cfr-008.md) |
| CFR-009 | medium | compatibility | CLI naming | help/next actionが `codefire-rs` に固定されている | `crates/codefire-cli/src/automation.rs:40`, `crates/codefire-cli/src/main.rs:1321` | [CFR-009](code-review-issues/cfr-009.md) |
| CFR-010 | high | correctness-risk | Atomic write | `write_json_atomic` の一時ファイル名が固定で衝突し得る | `crates/codefire-cli/src/main.rs:3987` | [CFR-010](code-review-issues/cfr-010.md) |
| CFR-011 | medium | correctness-risk | Atomic write | atomic write後にfsyncがない | `crates/codefire-cli/src/main.rs:4000` | [CFR-011](code-review-issues/cfr-011.md) |
| CFR-012 | medium | correctness-risk | Object store | object書き込み完了前クラッシュの検出粒度が弱い | `crates/codefire-store/src/lib.rs:222` | [CFR-012](code-review-issues/cfr-012.md) |
| CFR-013 | medium | performance | Object lookup | object検索が全subdir走査になっている | `crates/codefire-store/src/lib.rs:188` | [CFR-013](code-review-issues/cfr-013.md) |
| CFR-014 | medium | correctness-risk | Commit validation | sealed commit検証がroot参照の到達性を限定的にしか見ていない | `crates/codefire-store/src/lib.rs:299` | [CFR-014](code-review-issues/cfr-014.md) |
| CFR-015 | medium | maintainability | Object schema | object type mappingが複数箇所に散っている | `crates/codefire-store/src/lib.rs:93`, `crates/codefire-cli/src/main.rs:3930` | [CFR-015](code-review-issues/cfr-015.md) |
| CFR-016 | medium | compatibility | Object ID | 12桁digest IDは大規模化時の衝突余裕が小さい | `crates/codefire-store/src/lib.rs:88` | [CFR-016](code-review-issues/cfr-016.md) |
| CFR-017 | medium | compatibility | Fire model | fire UIDにSHA-1由来の短縮値が残っている | `crates/codefire-core/src/lib.rs:853`, `crates/codefire-core/src/lib.rs:1248` | [CFR-017](code-review-issues/cfr-017.md) |
| CFR-018 | medium | correctness-risk | Fire model | display fire番号がledger長依存で安定性が弱い | `crates/codefire-core/src/lib.rs:833` | [CFR-018](code-review-issues/cfr-018.md) |
| CFR-019 | high | correctness-risk | Atom extraction | 明示 `cf-atom` のcontent hashがマーカー行だけを見ている | `crates/codefire-core/src/lib.rs:1122` | [CFR-019](code-review-issues/cfr-019.md) |
| CFR-020 | medium | correctness-risk | Atom extraction | Atom ID validationが経路ごとに違う | `crates/codefire-core/src/lib.rs:1180`, `crates/codefire-core/src/lib.rs:1222` | [CFR-020](code-review-issues/cfr-020.md) |
| CFR-021 | medium | correctness-risk | Markdown parser | Markdown Atom抽出がコードフェンスを考慮しない | `crates/codefire-core/src/lib.rs:1084` | [CFR-021](code-review-issues/cfr-021.md) |
| CFR-022 | medium | correctness-risk | YAML parser | links/policy parserが限定的な手書きYAMLになっている | `crates/codefire-core/src/lib.rs:315`, `crates/codefire-core/src/lib.rs:358` | [CFR-022](code-review-issues/cfr-022.md) |
| CFR-023 | medium | correctness-risk | Link model | trace link IDが生Atom ID連結で長大化・衝突曖昧化し得る | `crates/codefire-core/src/lib.rs:927` | [CFR-023](code-review-issues/cfr-023.md) |
| CFR-024 | medium | performance | Required links | required link計算がAtomごと全link走査になっている | `crates/codefire-core/src/lib.rs:745` | [CFR-024](code-review-issues/cfr-024.md) |
| CFR-025 | high | performance | Diff | rename detectionが削除×追加の全組み合わせLCSになっている | `crates/codefire-cli/src/view/file_diff.rs:128` | [CFR-025](code-review-issues/cfr-025.md) |
| CFR-026 | high | performance | Diff | 類似度計算が巨大ファイルのLCSを無制限に使う | `crates/codefire-cli/src/view/file_diff.rs:218`, `crates/codefire-cli/src/view/file_diff.rs:402` | [CFR-026](code-review-issues/cfr-026.md) |
| CFR-027 | medium | performance | Diff | diff出力にhunk化・context制限がない | `crates/codefire-cli/src/view/file_diff.rs:246` | [CFR-027](code-review-issues/cfr-027.md) |
| CFR-028 | medium | correctness-risk | Merge | conflict markerがbinary/text混在を十分に区別しない | `crates/codefire-cli/src/main.rs:4113` | [CFR-028](code-review-issues/cfr-028.md) |
| CFR-029 | medium | correctness-risk | Merge | common ancestor計算が履歴全探索寄り | `crates/codefire-cli/src/main.rs:4043` | [CFR-029](code-review-issues/cfr-029.md) |
| CFR-030 | medium | security | HTTP client | HTTP clientに接続・読み取りtimeoutがない | `crates/codefire-cli/src/http.rs:111` | [CFR-030](code-review-issues/cfr-030.md) |
| CFR-031 | high | security | HTTP server | HTTP request body size上限がない | `crates/codefire-cli/src/http.rs:336` | [CFR-031](code-review-issues/cfr-031.md) |
| CFR-032 | high | correctness-risk | HTTP server | 短いHTTP bodyを不完全リクエストとして拒否していない | `crates/codefire-cli/src/http.rs:379` | [CFR-032](code-review-issues/cfr-032.md) |
| CFR-033 | medium | security | HTTP server | HTTP serverが単一thread逐次処理 | `crates/codefire-cli/src/http.rs:286` | [CFR-033](code-review-issues/cfr-033.md) |
| CFR-034 | medium | correctness-risk | HTTP remote | upload時のtmp directory名が衝突し得る | `crates/codefire-cli/src/http.rs:581` | [CFR-034](code-review-issues/cfr-034.md) |
| CFR-035 | high | correctness-risk | Remote generation | remote generation更新が独立lockされていない | `crates/codefire-cli/src/remote.rs:1354` | [CFR-035](code-review-issues/cfr-035.md) |
| CFR-036 | medium | security | Signatures | request nonce cacheの上限・GC方針が見えない | `crates/codefire-cli/src/signatures.rs:271` | [CFR-036](code-review-issues/cfr-036.md) |
| CFR-037 | medium | security | Signatures | commit signatureの時刻検証が文字列比較に依存している | `crates/codefire-cli/src/signatures.rs:392` | [CFR-037](code-review-issues/cfr-037.md) |
| CFR-038 | medium | maintainability | Dual implementation | Python旧本体とRust本体が二重実装として残っている | `codefire:1`, `crates/codefire-cli/src/main.rs:1` | [CFR-038](code-review-issues/cfr-038.md) |
| CFR-039 | medium | test-gap | Tests | integration testが巨大単一ファイルに集中している | `crates/codefire-cli/src/tests.rs:1`, `tests/test_codefire_cli.py:1` | [CFR-039](code-review-issues/cfr-039.md) |
| CFR-040 | medium | maintainability | Config/time/hash | 低レベルutilityが各crate/moduleに重複している | `crates/codefire-cli/src/main.rs:4656`, `crates/codefire-cli/src/remote.rs:1428`, `crates/codefire-store/src/lib.rs:390` | [CFR-040](code-review-issues/cfr-040.md) |
| CFR-041 | high | correctness-risk | Evidence capture | `evidence add --from-command` にtimeoutがない | `crates/codefire-cli/src/evidence.rs:327` | [CFR-041](code-review-issues/cfr-041.md) |
| CFR-042 | medium | security | Evidence capture | 証跡commandがshell経由で実行される | `crates/codefire-cli/src/evidence.rs:327` | [CFR-042](code-review-issues/cfr-042.md) |
| CFR-043 | medium | correctness-risk | Evidence capture | command cwdのdefaultがopen/repo rootではなく現在dirになっている | `crates/codefire-cli/src/evidence.rs:322` | [CFR-043](code-review-issues/cfr-043.md) |
| CFR-044 | medium | security | Evidence artifacts | artifact_refに絶対pathを保存している | `crates/codefire-cli/src/evidence.rs:267` | [CFR-044](code-review-issues/cfr-044.md) |
| CFR-045 | medium | performance | Evidence artifacts | artifact hash計算にサイズ上限やstreaming policyがない | `crates/codefire-cli/src/evidence.rs:266` | [CFR-045](code-review-issues/cfr-045.md) |
| CFR-046 | medium | maintainability | Batch parser | batch YAML parserが複数moduleに重複している | `crates/codefire-cli/src/link_batch.rs:335` | [CFR-046](code-review-issues/cfr-046.md) |
| CFR-047 | high | correctness-risk | Link batch | link batch appendの一時ファイル名が固定 | `crates/codefire-cli/src/link_batch.rs:284` | [CFR-047](code-review-issues/cfr-047.md) |
| CFR-048 | medium | correctness-risk | Link batch | link batch appendが既存YAML構造を保てない可能性がある | `crates/codefire-cli/src/link_batch.rs:260` | [CFR-048](code-review-issues/cfr-048.md) |
| CFR-049 | medium | ux | Link batch | link batchは失敗時に全件diagnosticを返さない | `crates/codefire-cli/src/link_batch.rs:211` | [CFR-049](code-review-issues/cfr-049.md) |
| CFR-050 | medium | correctness-risk | Doctor | doctorのrepo layout必須dir一覧が古い | `crates/codefire-cli/src/doctor/checks.rs:7` | [CFR-050](code-review-issues/cfr-050.md) |
| CFR-051 | medium | performance | Doctor | doctorがobject JSONを全件parseする | `crates/codefire-cli/src/doctor/checks.rs:63` | [CFR-051](code-review-issues/cfr-051.md) |
| CFR-052 | medium | correctness-risk | Doctor | doctorがactive stateのJSON shapeを検証していない | `crates/codefire-cli/src/doctor/checks.rs:226` | [CFR-052](code-review-issues/cfr-052.md) |
| CFR-053 | medium | correctness-risk | Doctor | opened registryのopen path整合性をdoctorが検証していない | `crates/codefire-cli/src/doctor/checks.rs:166` | [CFR-053](code-review-issues/cfr-053.md) |
| CFR-054 | medium | maintainability | Doctor | doctor issue severityがerror/warningの2段階に閉じている | `crates/codefire-cli/src/doctor/report.rs:1` | [CFR-054](code-review-issues/cfr-054.md) |
| CFR-055 | medium | performance | Storage report | storage reportが全fileを再帰走査する | `crates/codefire-cli/src/storage.rs:308` | [CFR-055](code-review-issues/cfr-055.md) |
| CFR-056 | medium | correctness-risk | Storage report | storage reportのobject type判定がrecord wrapperを考慮していない可能性がある | `crates/codefire-cli/src/storage.rs:330` | [CFR-056](code-review-issues/cfr-056.md) |
| CFR-057 | medium | ux | Storage report | storage next_actionが `codefire-rs` 固定 | `crates/codefire-cli/src/storage.rs:205` | [CFR-057](code-review-issues/cfr-057.md) |
| CFR-058 | medium | security | TLS | `CODEFIRE_TLS_INSECURE` が存在だけで証明書検証を無効化する | `crates/codefire-cli/src/http_tls.rs:20` | [CFR-058](code-review-issues/cfr-058.md) |
| CFR-059 | medium | security | TLS | TLS serverがclient authをサポートしない | `crates/codefire-cli/src/http_tls.rs:42` | [CFR-059](code-review-issues/cfr-059.md) |
| CFR-060 | low | compatibility | TLS | private key parserがPKCS8/RSAのみ対応 | `crates/codefire-cli/src/http_tls.rs:85` | [CFR-060](code-review-issues/cfr-060.md) |
| CFR-061 | medium | maintainability | Metrics | metricsがtotalのみでphase timingを持たない | `crates/codefire-cli/src/metrics.rs:71` | [CFR-061](code-review-issues/cfr-061.md) |
| CFR-062 | medium | performance | Metrics | metricsにcache disabled固定値だけが出る | `crates/codefire-cli/src/metrics.rs:81` | [CFR-062](code-review-issues/cfr-062.md) |
| CFR-063 | high | maintainability | Install | install.shがrelease binaryの検証をしない | `install.sh:104` | [CFR-063](code-review-issues/cfr-063.md) |
| CFR-064 | medium | compatibility | Install | install.shにdry-runがない | `install.sh:9` | [CFR-064](code-review-issues/cfr-064.md) |
| CFR-065 | medium | correctness-risk | Install | completion生成失敗時にbinary installだけ完了した状態になる | `install.sh:104`, `install.sh:119` | [CFR-065](code-review-issues/cfr-065.md) |
| CFR-066 | medium | security | Install | install先prefixの安全性チェックが弱い | `install.sh:79` | [CFR-066](code-review-issues/cfr-066.md) |
| CFR-067 | medium | correctness-risk | Remote idempotency | remote idempotency keyが空文字をOptionとして受ける | `crates/codefire-cli/src/remote/idempotency.rs:12` | [CFR-067](code-review-issues/cfr-067.md) |
| CFR-068 | medium | security | Remote idempotency | idempotency recordにpayload全文を保存している | `crates/codefire-cli/src/remote/idempotency.rs:120` | [CFR-068](code-review-issues/cfr-068.md) |
| CFR-069 | medium | performance | Remote idempotency | idempotency recordのretention/GCが見えない | `crates/codefire-cli/src/remote/idempotency.rs:269` | [CFR-069](code-review-issues/cfr-069.md) |
| CFR-070 | medium | correctness-risk | Remote idempotency | idempotency resultにplanを保存して再利用している | `crates/codefire-cli/src/remote/idempotency.rs:141` | [CFR-070](code-review-issues/cfr-070.md) |
| CFR-071 | medium | maintainability | Automation schema | command_result schema versionが文字列直書き | `crates/codefire-cli/src/automation.rs:14` | [CFR-071](code-review-issues/cfr-071.md) |
| CFR-072 | medium | ux | Automation next_actions | next_actionsが多すぎる場合の上限がない | `crates/codefire-cli/src/automation.rs:127` | [CFR-072](code-review-issues/cfr-072.md) |
| CFR-073 | medium | correctness-risk | Automation diagnostics | missing required linksが常にerror severity | `crates/codefire-cli/src/automation.rs:195` | [CFR-073](code-review-issues/cfr-073.md) |
| CFR-074 | medium | maintainability | Context | context snapshotがscan resultを再構築している | `crates/codefire-cli/src/context.rs:147` | [CFR-074](code-review-issues/cfr-074.md) |
| CFR-075 | medium | correctness-risk | Context | contextが固定時刻でscanを構築している | `crates/codefire-cli/src/context.rs:175` | [CFR-075](code-review-issues/cfr-075.md) |
| CFR-076 | medium | performance | Context | context depth traversalに出力上限がない | `crates/codefire-cli/src/context.rs:254` | [CFR-076](code-review-issues/cfr-076.md) |
| CFR-077 | medium | correctness-risk | Remote URL parser | cf+http URL parserが余分なpath segmentを許容する | `crates/codefire-cli/src/http.rs:64` | [CFR-077](code-review-issues/cfr-077.md) |
| CFR-078 | medium | correctness-risk | HTTP parser | HTTP URL parserがIPv6 authorityを扱いにくい | `crates/codefire-cli/src/http.rs:248` | [CFR-078](code-review-issues/cfr-078.md) |
| CFR-079 | medium | security | HTTP parser | HTTP request method/pathの正規化が弱い | `crates/codefire-cli/src/http.rs:361` | [CFR-079](code-review-issues/cfr-079.md) |
| CFR-080 | medium | maintainability | Release docs | Python fallback専用commandの移行状態がコードから検証されない | `docs/development/known-limitations.md:65` | [CFR-080](code-review-issues/cfr-080.md) |
