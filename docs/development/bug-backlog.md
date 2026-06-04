# CodeFire Bug Backlog

このファイルは、CodeFire自身を実プロジェクト管理に使いながら見つかった不具合、バグに近いUX、修正候補を管理する。

## Status

- `open`: 未修正
- `investigating`: 再現条件や修正方針を確認中
- `fixed`: 修正済み
- `wontfix`: 現時点では仕様として扱う

## Bugs

| ID | Status | Area | Summary | Observed while | Impact | Workaround | Candidate fix |
|---|---|---|---|---|---|---|---|
| CFB-001 | fixed | Extinguish / stale resolution | extinguished済みfireのresolutionがstaleになった後、通常CLIで同じfireを再extinguishできない | `algorithm-evolution-agent-lab` で `StaticPlanner` を別moduleへ移動した後の `codefire verify` | `Open fires: 0` でも `Stale resolutions: N` でcommit不能になり、通常操作だけでは復旧しづらい | `codefire extinguish <fire> --refresh ...` で現在のbasisへ更新する | `--refresh` で既存resolutionを `superseded` にし、新しいresolution basisを保存する |
| CFB-002 | fixed | Python Atom extraction | Python methodの派生Atom IDがownerなしの `CODE:path::def:<method>` になり、異なるclassの同名methodでduplicate Atom IDになる | `CodexPlanner.propose` と `StaticPlanner.propose` を同一fileに置いたとき | 正当なPython設計でもduplicate Atom IDでverify/commitが止まる | 明示 `# cf-atom:` でも回避可能 | Python extractorでclass ownerを追跡し、method派生IDを `method:Class.method` にした |
| CFB-003 | fixed | Diagnostics | `verify` は `Missing required links` の件数を表示するが、`commit_policy.require_trace_completeness: false` の場合でも失敗原因に見えやすい | 仕様先行フェーズでtrace completenessを緩和した状態 | 実際のblockerがstale/duplicateでも、missing linkが主原因に見える | `Blocking checks:` 行を見る | verify出力にpolicy上のblockerだけを列挙する `Blocking checks:` を追加した |
| CFB-004 | fixed | Diagnostics | `verify` の失敗件数から対象Atomやstale resolutionを直接確認できない | duplicate Atom IDやmissing linkの原因を調査するとき | active JSONやscan結果を読まないと次の操作対象を特定しづらい | `.codefire-open/state/active_verify.json` を読む | `codefire verify --details` で失敗diagnosticの代表例を表示する |

## Triage Notes

- CFB-001は `test_stale_resolution_blocks_verify` でrefresh導線を確認する。
- CFB-002は `test_python_methods_include_class_owner_in_derived_atom_id` で同名methodの派生ID衝突回避を確認する。
- CFB-003は既存verify失敗系テストで `Blocking checks:` を確認する。
- CFB-004は `test_verify_details_reports_missing_required_links` とduplicate Atom ID詳細表示で確認する。
