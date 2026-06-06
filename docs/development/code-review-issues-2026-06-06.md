# Code Review Issues 2026-06-06

この文書は CodeFire 本体コードレビューの全体要約リストである。詳細は `docs/development/code-review-issues/` 配下の1 issue 1 fileで管理する。

## How To Use

- 全体の優先順位と対象領域はこのindexで見る。
- 実装修正に着手する場合は、各CFRの詳細ファイルを開き、Problem / Code Evidence / Recommended Fix / Acceptance Criteria / Test Planを確認する。
- dogfoodingで観測した不具合・UX issueは `docs/development/bug-backlog.md`、コードレビュー起点の改善候補はこのCFR系列で管理する。
- 修正完了時は詳細ファイルの `Status` を更新し、必要に応じて `history.md` と仕様文書へ反映する。

## Count

- Total: 80
- High: 15
- Medium: 64
- Low: 1
- Fixed in v0.7: 41
- Remaining open: 39

## v0.7 Progress

- Batch 1 fixed: [CFR-005](code-review-issues/cfr-005.md), [CFR-006](code-review-issues/cfr-006.md), [CFR-010](code-review-issues/cfr-010.md), [CFR-031](code-review-issues/cfr-031.md), [CFR-032](code-review-issues/cfr-032.md), [CFR-047](code-review-issues/cfr-047.md), [CFR-058](code-review-issues/cfr-058.md), [CFR-067](code-review-issues/cfr-067.md), [CFR-077](code-review-issues/cfr-077.md)
- Batch 2/8 partial fixed: [CFR-003](code-review-issues/cfr-003.md), [CFR-009](code-review-issues/cfr-009.md), [CFR-057](code-review-issues/cfr-057.md), [CFR-071](code-review-issues/cfr-071.md), [CFR-072](code-review-issues/cfr-072.md), [CFR-075](code-review-issues/cfr-075.md), [CFR-076](code-review-issues/cfr-076.md)
- Batch 3 fixed: [CFR-011](code-review-issues/cfr-011.md), [CFR-012](code-review-issues/cfr-012.md), [CFR-013](code-review-issues/cfr-013.md), [CFR-014](code-review-issues/cfr-014.md), [CFR-015](code-review-issues/cfr-015.md), [CFR-016](code-review-issues/cfr-016.md), [CFR-017](code-review-issues/cfr-017.md), [CFR-018](code-review-issues/cfr-018.md)
- Batch 5 fixed: [CFR-025](code-review-issues/cfr-025.md), [CFR-026](code-review-issues/cfr-026.md), [CFR-027](code-review-issues/cfr-027.md), [CFR-028](code-review-issues/cfr-028.md), [CFR-029](code-review-issues/cfr-029.md)
- Batch 6 fixed: [CFR-030](code-review-issues/cfr-030.md), [CFR-033](code-review-issues/cfr-033.md), [CFR-034](code-review-issues/cfr-034.md), [CFR-035](code-review-issues/cfr-035.md), [CFR-068](code-review-issues/cfr-068.md), [CFR-069](code-review-issues/cfr-069.md), [CFR-070](code-review-issues/cfr-070.md), [CFR-078](code-review-issues/cfr-078.md), [CFR-079](code-review-issues/cfr-079.md)
- Batch 7 partial fixed: [CFR-041](code-review-issues/cfr-041.md), [CFR-042](code-review-issues/cfr-042.md), [CFR-043](code-review-issues/cfr-043.md).
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
