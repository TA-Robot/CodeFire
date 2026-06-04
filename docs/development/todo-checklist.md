# CodeFire Development TODO Checklist

このチェックリストは、`IMPLEMENTATION_TODO.md` と `docs/12_mvp_implementation_plan.md` を、開発管理しやすい作業単位へ展開したものです。

## 運用ルール

- `Status` は `todo` / `doing` / `blocked` / `done` のいずれかで更新する。
- 完了時は `Evidence` に commit hash、テスト結果、または確認ログを残す。
- 仕様判断が変わった場合は `docs/development/history.md` と ADR のどちらかに記録する。
- 当初MVPのlocal flowは完了済み。現在はfile-backed remoteまでをv0.2実装範囲として管理する。
- AI agent運用、marketplace、GUI、semantic merge、hosted server isolationは対象外にする。CodeFireは外部ツールの実行管理ではなく、整合性管理と履歴封印に集中する。

## Definition of Done

- `codefire init` から `codefire commit` まで、ローカルMVPのデモシナリオが通る。
- Code/Python/pytest/Markdown spec/design/ADR/ops の Atom を抽出できる。
- Trace Link 欠落、duplicate Atom ID、open fire、stale resolution が commit blocker になる。
- `extinguish --resolution no-change-required` は rationale なしでは拒否される。
- sealed commit には manifest、atom index、trace graph、fire delta、verification、policy、certificate が含まれる。
- `merge` 後に target branch が open-burning になり、fire 解消後に merge commit を作れる。

## Checklist

| ID | Phase | Task | Depends on | Status | Evidence |
|---|---|---|---|---|---|
| CF-000 | Project | 実装用リポジトリ構成を決める | - | done | `project/` 直下に標準ライブラリPython CLIとして実装 |
| CF-001 | Project | Rust workspace または採用言語のプロジェクトを作成する | CF-000 | done | `project/codefire` |
| CF-002 | Project | CLI entrypoint とエラー表示方針を作る | CF-001 | done | `project/codefire --help`; `CFError` は stderr + exit 2 |
| CF-003 | Project | fixtures と golden test の置き場を作る | CF-001 | done | `project/tests/test_codefire_cli.py` |
| CF-010 | Object store | canonical JSON serialization を実装する | CF-001 | done | `canonical_json`; `python3 -m unittest discover -s project/tests -v` |
| CF-011 | Object store | object hash calculation を実装する | CF-010 | done | `object_digest`, `object_id`; doctor hash check |
| CF-012 | Object store | immutable object store を実装する | CF-011 | done | `store_object` writes content-addressed JSON once |
| CF-013 | Object store | manifest object を保存できるようにする | CF-012 | done | commit E2E stores `content_manifest` |
| CF-014 | Object store | commit object を保存できるようにする | CF-013 | done | init/commit E2E stores `CF-COMMIT-*` |
| CF-015 | Object store | branch object と branch head atomic update を実装する | CF-014 | done | branch JSON is updated through temp+replace |
| CF-016 | Object store | object graph traversalを型別参照抽出にする | CF-014 | done | `test_object_graph_ignores_non_reference_cf_strings` |
| CF-020 | Repository | `codefire init` を実装する | CF-015 | done | `test_init_creates_main_and_object_store` |
| CF-021 | Repository | 空の main branch を作成する | CF-020 | done | `test_init_creates_main_and_object_store` |
| CF-021A | Repository | `branch list` 前にbranch headのsealed commit妥当性を検証する | CF-021 | done | `test_branch_list_rejects_invalid_local_branch_head` |
| CF-022 | Repository | repo lock / branch lock の最小実装を作る | CF-020 | done | `file_lock`; `test_repo_lock_blocks_structural_changes` |
| CF-030 | Open directory | `codefire open <branch> <path>` を実装する | CF-021 | done | E2E tests open `main` and `feature-session` |
| CF-030A | Open directory | `open` 前にbranch headのsealed commit妥当性を検証する | CF-030 | done | `test_open_and_clone_reject_invalid_local_branch_head` |
| CF-031 | Open directory | `.codefire-open` を生成する | CF-030 | done | open context tests pass through all E2E commands |
| CF-032 | Open directory | open registry を更新する | CF-031 | done | `.codefire/opened/*.json` used by scan/commit |
| CF-033 | Open directory | 同一branch多重openを拒否する | CF-032 | done | `cmd_open` rejects existing registry |
| CF-034 | Open directory | copied directory detection を実装する | CF-031 | done | `open_context` validates path and open_instance_id |
| CF-034A | Open directory | open directory command実行前にbase/headのsealed commit妥当性を検証する | CF-034 | done | `test_open_directory_commands_reject_invalid_current_base_commit`, `test_open_directory_commands_reject_invalid_branch_head` |
| CF-035 | Open directory | `close` と `close --discard` を実装する | CF-032 | done | `test_close_and_close_discard` |
| CF-036 | Branch | local `clone` を実装する | CF-021 | done | `test_clone_open_merge_and_merge_commit` |
| CF-036A | Branch | local `clone` 前にsource headのsealed commit妥当性を検証する | CF-036 | done | `test_open_and_clone_reject_invalid_local_branch_head` |
| CF-037 | Branch | open-burning source branchからのclone/mergeを拒否する | CF-036 | done | `test_clone_and_merge_reject_open_burning_source` |
| CF-038 | Branch | slash入りbranch名を衝突なく保存する | CF-036 | done | `test_branch_names_with_slashes_do_not_collide` |
| CF-039 | Branch | slash入りbranch名のlock file名も衝突なく扱う | CF-038 | done | `test_branch_lock_names_with_slashes_do_not_collide` |
| CF-040 | Config | `codefire.yaml` parser を実装する | CF-001 | done | MVP subset parser in `parse_config` |
| CF-040A | Config | 限定YAML subsetの必須項目不足を明示エラーにする | CF-040, CF-050, CF-054 | done | `test_invalid_codefire_yaml_reports_configuration_error`, `test_invalid_links_yaml_reports_configuration_error`, `test_invalid_policy_yaml_reports_configuration_error`, `test_codefire_yaml_ignores_future_top_level_sections` |
| CF-041 | Indexer | Markdown requirement/design Atom extractor を実装する | CF-040 | done | E2E extracts `REQ-*` and `DES-*` |
| CF-041A | Indexer | Markdown ADR/ops Atom extractor を実装する | CF-041 | done | `test_adr_and_ops_markdown_atoms_are_indexed` |
| CF-041B | Indexer | OpenAPI JSON operation Atom extractor を実装する | CF-040 | done | `test_openapi_json_operations_are_indexed` |
| CF-041D | Indexer | OpenAPI YAML operation Atom extractor を実装する | CF-041B | done | `test_openapi_yaml_operations_are_indexed` |
| CF-041C | Indexer | SQL DB schema table Atom extractor を実装する | CF-040 | done | `test_db_schema_tables_are_indexed` |
| CF-041E | Indexer | SQL DB schema column Atom extractor を実装する | CF-041C | done | `test_db_schema_tables_are_indexed` |
| CF-041F | Indexer | SQL DB schema index Atom extractor を実装する | CF-041E | done | `test_db_schema_tables_are_indexed` |
| CF-041G | Indexer | SQL DB schema view Atom extractor を実装する | CF-041F | done | `test_db_schema_tables_are_indexed` |
| CF-041H | Indexer | SQL DB schema trigger Atom extractor を実装する | CF-041G | done | `test_db_schema_tables_are_indexed` |
| CF-041I | Indexer | SQL DB schema function Atom extractor を実装する | CF-041H | done | `test_db_schema_tables_are_indexed` |
| CF-041J | Indexer | SQL DB schema procedure Atom extractor を実装する | CF-041I | done | `test_db_schema_tables_are_indexed` |
| CF-041K | Indexer | SQL DB schema sequence Atom extractor を実装する | CF-041J | done | `test_db_schema_tables_are_indexed` |
| CF-041L | Indexer | SQL DB schema materialized view Atom extractor を実装する | CF-041K | done | `test_db_schema_tables_are_indexed` |
| CF-041M | Indexer | SQL DB schema type Atom extractor を実装する | CF-041L | done | `test_db_schema_tables_are_indexed` |
| CF-042 | Indexer | Python code Atom extractor を実装する | CF-040 | done | E2E extracts `CODE-SessionPolicy` |
| CF-042A | Indexer | JavaScript/TypeScript code Atom extractor を実装する | CF-042 | done | `test_typescript_code_atoms_are_indexed` |
| CF-042B | Indexer | Go code Atom extractor を実装する | CF-042A | done | `test_go_code_atoms_are_indexed` |
| CF-042C | Indexer | Java code Atom extractor を実装する | CF-042B | done | `test_java_code_atoms_are_indexed` |
| CF-042D | Indexer | C# code Atom extractor を実装する | CF-042C | done | `test_csharp_code_atoms_are_indexed` |
| CF-042E | Indexer | Rust code Atom extractor を実装する | CF-042D | done | `test_rust_code_atoms_are_indexed` |
| CF-042F | Indexer | Kotlin code Atom extractor を実装する | CF-042E | done | `test_kotlin_code_atoms_are_indexed` |
| CF-042G | Indexer | PHP code Atom extractor を実装する | CF-042F | done | `test_php_code_atoms_are_indexed` |
| CF-042H | Indexer | Ruby code Atom extractor を実装する | CF-042G | done | `test_ruby_code_atoms_are_indexed` |
| CF-042I | Indexer | Swift code Atom extractor を実装する | CF-042H | done | `test_swift_code_atoms_are_indexed` |
| CF-042J | Indexer | C/C++ code Atom extractor を実装する | CF-042I | done | `test_cpp_code_atoms_are_indexed` |
| CF-043 | Indexer | pytest Atom extractor を実装する | CF-040 | done | E2E extracts `TEST-session-expiration` |
| CF-044 | Indexer | AtomIndex generation を実装する | CF-041, CF-042, CF-043 | done | scan/verify E2E |
| CF-045 | Indexer | duplicate Atom ID detection を実装する | CF-044 | done | `test_duplicate_atom_id_blocks_verify` |
| CF-050 | Trace | `codefire.links.yaml` parser を実装する | CF-040 | done | E2E link graph |
| CF-051 | Trace | TraceGraph を構築する | CF-050, CF-044 | done | `current_trace_graph` |
| CF-052 | Trace | link参照検証を実装する | CF-051 | done | verify blocks missing links |
| CF-053 | Trace | required link policy を実装する | CF-052 | done | default requirement/design policy |
| CF-054 | Trace | `codefire.policy.yaml` のcustom required_linksを実装する | CF-053 | done | `test_policy_required_links_can_be_customized` |
| CF-060 | Fire | `scan` command を実装する | CF-044, CF-051 | done | E2E creates fires |
| CF-061 | Fire | changed atom detection を実装する | CF-060 | done | `changed_atoms` |
| CF-062 | Fire | fire propagation を実装する | CF-061, CF-053 | done | direct adjacent Atom propagation |
| CF-063 | Fire | active fire ledger を実装する | CF-062 | done | `.codefire/active/*/fires.json` |
| CF-064 | Fire | obsolete auto fire を実装する | CF-063 | done | `test_obsolete_auto_fire_after_revert` |
| CF-065 | Fire | manual `fire` command を実装する | CF-063 | done | `test_manual_fire_survives_scan_until_extinguished` |
| CF-066 | Fire | `status` 表示を実装する | CF-063 | done | merge E2E checks `State: open-burning` |
| CF-070 | Extinguish | `extinguish` command を実装する | CF-063 | done | E2E extinguishes scan fires |
| CF-071 | Extinguish | resolution basis を保存する | CF-070 | done | `resolutions.json` basis |
| CF-072 | Extinguish | no-change-required rationale 必須チェックを実装する | CF-070 | done | `test_no_change_required_requires_rationale` |
| CF-073 | Extinguish | stale resolution check を実装する | CF-071 | done | `test_stale_resolution_blocks_verify` |
| CF-074 | Extinguish | 全resolutionでrationaleまたはevidenceを必須にする | CF-070 | done | `test_extinguish_requires_rationale_or_evidence` |
| CF-075 | Extinguish | `extinguish_policy.no-change-required.requires_rationale` を実装する | CF-072 | done | `test_extinguish_policy_can_allow_no_change_required_evidence_only` |
| CF-080 | Verify | verification command runner を実装する | CF-040 | done | E2E runs `python3 -m unittest discover -s tests` |
| CF-081 | Verify | internal policy checks を実装する | CF-045, CF-053, CF-073 | done | verify checks fires, links, stale, duplicates |
| CF-082 | Verify | verification object を active state に保存する | CF-080, CF-081 | done | `.codefire/active/*/verification.json` |
| CF-083 | Verify | 複数verification commandを実行する | CF-080 | done | `test_verify_runs_multiple_policy_commands` |
| CF-084 | Verify | `commit_policy` booleanをverify/commit/server validationに反映する | CF-081 | done | `test_commit_policy_can_allow_failed_verification_commands` |
| CF-090 | Commit | commit sealer を実装する | CF-012, CF-082 | done | E2E creates sealed commits |
| CF-091 | Commit | consistency certificate を生成する | CF-090 | done | commit payload certificate |
| CF-092 | Commit | commit後にbranch headを更新する | CF-091, CF-015 | done | E2E reads updated branch head |
| CF-093 | Commit | commit後にactive stateをresetし open-clean にする | CF-092 | done | commit E2E output and state reset |
| CF-100 | Merge | common ancestor search を実装する | CF-092 | done | `common_ancestor`; merge E2E |
| CF-100A | Merge | `merge` 前にsource/target headのsealed commit妥当性を検証する | CF-100 | done | `test_merge_rejects_invalid_local_branch_head` |
| CF-101 | Merge | file-level 3-way merge を実装する | CF-100 | done | `test_merge_conflict_blocks_verify` |
| CF-102 | Merge | merge fire generation を実装する | CF-101, CF-063 | done | merge scan emits `merge_changed` fires |
| CF-103 | Merge | merge commit parents を保存する | CF-102, CF-090 | done | `test_clone_open_merge_and_merge_commit` asserts 2 parents |
| CF-110 | Demo | MVPデモfixtureを作成する | CF-093 | done | `write_demo_files` test fixture |
| CF-111 | Demo | `scan -> fire -> extinguish -> verify -> commit` のE2Eテストを作る | CF-110 | done | `test_demo_scan_fire_extinguish_verify_commit` |
| CF-112 | Demo | `clone -> open -> merge -> merge commit` のE2Eテストを作る | CF-103, CF-110 | done | `test_clone_open_merge_and_merge_commit` |
| CF-120 | Release | `codefire doctor` の最小診断を実装する | CF-034, CF-063 | done | `test_init_creates_main_and_object_store` |
| CF-120A | Release | `doctor` でmissing object referenceを診断する | CF-120 | done | `test_doctor_detects_missing_object_references` |
| CF-120B | Release | `doctor` でobject record hash/id/filename mismatchを診断する | CF-120 | done | `test_doctor_detects_object_record_integrity_errors` |
| CF-120C | Release | `doctor` でbranch/MRのsealed commit参照不正を診断する | CF-120 | done | `test_doctor_detects_invalid_local_sealed_commit_references`, `test_remote_doctor_detects_invalid_sealed_commit_references` |
| CF-121 | Release | READMEのMVP手順を実装結果に合わせて更新する | CF-111, CF-112 | done | `README.md`, `docs/runbook.md` |
| CF-122 | Release | v0.3未満の既知制約を整理する | CF-121 | done | `docs/development/known-limitations.md` |
| CF-130 | Remote | file-backed remote upload/list/cloneを実装する | CF-092 | done | `test_upload_list_remote_clone_and_request_merge` |
| CF-130A | Remote | remote `list` 前にbranch headのsealed commit妥当性を検証する | CF-130 | done | `test_remote_list_rejects_invalid_branch_head` |
| CF-131 | Remote | remote show/diffを実装する | CF-130 | done | `test_upload_list_remote_clone_and_request_merge` |
| CF-131A | Remote | local/remote `show`/`diff` 前にcommitishのsealed commit妥当性を検証する | CF-131 | done | `test_show_and_diff_reject_invalid_local_branch_head`, `test_show_and_diff_reject_invalid_remote_branch_head` |
| CF-132 | Remote | merge request create/list/review/applyを実装する | CF-130 | done | `test_upload_list_remote_clone_and_request_merge` |
| CF-132A | Remote | request-list/review/apply前にMR内sealed commit参照を検証する | CF-132 | done | `test_request_list_and_review_reject_invalid_merge_request_refs` |
| CF-133 | Remote | upload時のsealed commit validationを実装する | CF-130 | done | `test_upload_rejects_inconsistent_sealed_commit` |
| CF-133A | Remote | sealed commit root object typeを検証する | CF-133 | done | `test_upload_rejects_sealed_commit_with_wrong_root_type` |
| CF-133B | Remote | sealed commit parents/roots/certificate構造を検証する | CF-133 | done | `test_upload_rejects_sealed_commit_with_malformed_payload_fields` |
| CF-133C | Remote | sealed commit parent履歴を再帰検証する | CF-133B | done | `test_upload_rejects_sealed_commit_with_invalid_parent_history` |
| CF-134 | Remote | server-side verification commandを実装する | CF-130 | done | `test_upload_runs_server_side_verification_policy` |
| CF-135 | Remote | remote permission policyを実装する | CF-130 | done | `test_remote_permissions_gate_mutating_operations` |
| CF-135A | Remote | remote branch protection policyを実装する | CF-135 | done | `test_remote_branch_protection_gates_upload_and_apply` |
| CF-136 | Remote | remote token authenticationを実装する | CF-135 | done | `test_remote_token_auth_required_for_mutating_operations` |
| CF-137 | Remote | remote object GCとretention policyを実装する | CF-130 | done | `test_remote_gc_removes_unreachable_objects`, `test_remote_gc_respects_retention_policy` |
| CF-137A | Remote | GC前にremote object graphとsealed commit参照を検証する | CF-137 | done | `test_remote_gc_blocks_on_unhealthy_object_graph`, `test_remote_gc_blocks_on_invalid_sealed_refs` |
| CF-137B | Remote | remote GCのdry-run/削除監査ログを記録する | CF-137 | done | `test_remote_gc_writes_audit_log` |
| CF-137C | Remote | remote GCの世代保持を実装する | CF-137 | done | `test_remote_gc_respects_generation_retention_policy` |
| CF-138 | Remote | slash入りremote branch名をURLエンコードURLで扱いMR applyまで通す | CF-130 | done | `test_remote_branch_names_with_slashes_use_encoded_urls` |
| CF-139 | Remote | slash入りremote branch名のlock file名も衝突なく扱う | CF-138 | done | `test_remote_branch_lock_names_with_slashes_do_not_collide` |
| CF-140 | Release | 一気通貫デモスクリプトを追加する | CF-137 | done | `demo.sh` |
| CF-141 | Release | bash/zsh shell completion生成とinstall配置を実装する | CF-140 | done | `test_completion_command_and_install_completion` |
| CF-142 | Release | pyproject/setuptools package installを実装する | CF-141 | done | `test_pyproject_installs_codefire_script` |
| CF-143 | Remote | HTTP remote upload/list/clone protocolを実装する | CF-130 | done | `test_http_remote_upload_list_and_clone` |
| CF-144 | Remote | HTTP merge request create/list/review/apply protocolを実装する | CF-143 | done | `test_http_remote_upload_list_and_clone` |
| CF-145 | Remote | HTTP remote show/diffを実装する | CF-144 | done | `test_http_remote_upload_list_and_clone` |
| CF-146 | Remote | HTTP remote doctor/gcを実装する | CF-145 | done | `test_http_remote_upload_list_and_clone` |
| CF-147 | Remote | HTTPS transportと `cf+https://` URLを実装する | CF-146 | done | `test_https_remote_upload_list_clone_and_show` |
| CF-148 | Remote | server-side verificationのcwd/env/timeout制約を実装する | CF-134 | done | `test_server_side_verification_isolates_cwd_env_and_timeout` |
| CF-149 | Remote | remote tokenを平文以外にhash保存できるようにする | CF-136 | done | `test_remote_token_auth_accepts_hashed_tokens` |
| CF-150 | Remote | commit HMAC署名とremote署名必須policyを実装する | CF-133, CF-135 | done | `test_remote_requires_and_verifies_commit_signature`, `test_http_remote_requires_commit_signature_on_upload` |
| CF-151 | Remote | commit署名key rotation policyを実装する | CF-150 | done | `test_remote_commit_signature_key_rotation_policy` |
| CF-152 | Remote | mutating remote operationのHMAC request署名を実装する | CF-135, CF-149 | done | `test_remote_request_signatures_gate_mutating_operations`, `test_http_remote_request_signature_required_on_upload` |
| CF-153 | Remote | HMAC request署名のnonce replay cacheを実装する | CF-152 | done | `test_http_remote_request_signature_rejects_replayed_nonce` |
| CF-160 | Extinguish | stale resolutionを通常CLIで再解消できる復旧導線を実装する | CF-073 | done | CFB-001; `test_stale_resolution_blocks_verify` |
| CF-161 | Indexer | Python method派生IDにclass ownerを含める | CF-042 | done | CFB-002; `test_python_methods_include_class_owner_in_derived_atom_id` |
| CF-162 | Verify | verify出力をblocking/non-blocking diagnosticsに分ける | CF-081 | done | CFB-003; verify failure tests |
| CF-163 | Verify | verify失敗時の対象詳細をCLIで確認できるようにする | CF-162 | done | CFB-004; `test_verify_details_reports_missing_required_links` |
| CF-164 | UX | fire解消のinteractive / batch UXを設計する | CF-163 | todo | CFB-005 |
| CF-165 | Verify | `verify --details` でblocking-only表示またはnon-blocking折りたたみを実装する | CF-163 | todo | CFB-006 |
| CF-166 | Observability | `status` / `scan` / `verify` のphase別時間とAtom/object数を表示するmetrics導線を追加する | CF-163 | todo | CFB-007 |
| CF-167 | Storage | object store size report、large artifact warning、external artifact reference方針を実装/文書化する | CF-166 | todo | CFB-008 |
| CF-200 | Rust | v0.6 Rust rewrite workspaceとcrate境界を作成する | CF-167 | done | `Cargo.toml`; `crates/codefire-*`; `cargo test --workspace` |
| CF-201 | Rust Store | canonical JSON / object IDのgolden test付きRust実装を作る | CF-200 | done | `codefire-store`; Python golden payload parity; `cargo test --workspace` |
| CF-202 | Rust Store | immutable object storeとsealed commit validationをRustで実装する | CF-201 | done | `validate_sealed_commit`; 10 Rust tests pass |
| CF-202A | Rust Store | immutable object record write/read/validateをRustで実装する | CF-201 | done | `codefire-store::store_object`; tamper detection tests |
| CF-203 | Rust Repo | init/open/status/branch registry互換をRustで実装する | CF-202 | done | CF-203A..D; Rust/Python compatibility smoke |
| CF-203A | Rust Repo | Python-created open directoryのread-only `status` 互換をRustで実装する | CF-202 | done | `codefire-rs status` fixture; real repo smoke; `cargo test --workspace`; `cargo clippy --workspace --all-targets -- -D warnings` |
| CF-203B | Rust Repo | Python-created branch registryのread-only `branch list` 互換をRustで実装する | CF-203A | done | branch fixture with marker repo discovery; real repo smoke; `cargo test --workspace`; `cargo clippy --workspace --all-targets -- -D warnings` |
| CF-203C | Rust Repo | Python-compatible `init` をRustで実装し、初期sealed commitとmain branchを生成する | CF-203B | done | Rust init unit test; Python `doctor`/`branch list` smoke; `cargo test --workspace`; `cargo clippy --workspace --all-targets -- -D warnings` |
| CF-203D | Rust Repo | Python-compatible `open` をRustで実装し、marker/registry/active stateとmanifest materializationを生成する | CF-203C | done | Rust open unit test; Rust init/open + Python `status`/`doctor`/`branch list` smoke; `cargo test --workspace`; `cargo clippy --workspace --all-targets -- -D warnings` |
| CF-204 | Rust Index | Markdownと明示 `cf-atom` extractorをRustで実装する | CF-203 | done | `codefire-core::build_atom_index`; `codefire-rs atom-index`; real repo smoke; `cargo test --workspace`; `cargo clippy --workspace --all-targets -- -D warnings` |
| CF-205 | Rust Trace | links parser、trace graph、required link policyをRustで実装する | CF-204 | done | `codefire-core::current_trace_graph`; `current_required_link_missing`; `codefire-rs trace-graph`/`missing-links`; real repo smoke; `cargo test --workspace`; `cargo clippy --workspace --all-targets -- -D warnings` |
| CF-206 | Rust Fire | scan、changed atom detection、fire ledgerをRustで実装する | CF-205 | done | `codefire-core::build_scan_result`; `codefire-rs scan`; active `scan.json`/`fires.json` smoke; `cargo test --workspace`; `cargo clippy --workspace --all-targets -- -D warnings` |
| CF-207 | Rust UX | interactive / batch extinguishをRust CLIで実装する | CF-206, CF-164 | todo | CFB-005 |
| CF-207A | Rust Extinguish | Rust CLIで単一fire解消、resolution basis、resolution ledger更新を実装する | CF-206 | done | `codefire-core::Resolution`; `codefire-rs extinguish`; temp repo scan/extinguish/verify smoke; `cargo test --workspace`; `cargo clippy --workspace --all-targets -- -D warnings` |
| CF-208 | Rust Verify | verification engineとstructured diagnosticsをRustで実装する | CF-206, CF-165 | done | CF-208A..B; Rust verify pass/fail/stale smoke; `cargo test --workspace`; `cargo clippy --workspace --all-targets -- -D warnings` |
| CF-208A | Rust Verify | open fires、missing links、duplicate atoms、verification commandをRust `verify` で判定しactive verificationへ保存する | CF-206 | done | `codefire-core::build_verification`; `codefire-rs verify`; pass/fail temp repo smoke; `cargo test --workspace`; `cargo clippy --workspace --all-targets -- -D warnings` |
| CF-208B | Rust Verify | resolution basisからstale resolutionをRust `verify` で検出する | CF-207A, CF-208A | done | `codefire-core::stale_resolutions`; stale temp repo smoke; `cargo test --workspace`; `cargo clippy --workspace --all-targets -- -D warnings` |
| CF-209 | Rust Commit | commit sealingとbranch head updateをRustで実装する | CF-208 | done | `codefire-rs commit`; Rust init/open/scan/extinguish/verify/commit smoke; Python `doctor`/`branch list` compatibility; `cargo test --workspace`; `cargo clippy --workspace --all-targets -- -D warnings` |
| CF-210 | Rust Merge | clone、show/diff、merge、merge firesをRustで実装する | CF-209 | done | `codefire-rs clone/show/diff/merge`; merge pending state drives `merge_changed` scan fires; `cargo test --workspace`; `cargo clippy --workspace --all-targets -- -D warnings`; Rust local clone/show/diff/merge smoke |
| CF-211 | Rust Remote | file-backed remoteとMR flowをRustで実装する | CF-210 | done | `codefire-rs upload/list/clone/show/diff/request-merge/request-list/request-review/request-apply`; Rust file-backed remote/MR smoke; `cargo test --workspace`; `cargo clippy --workspace --all-targets -- -D warnings` |
| CF-212 | Rust Server | HTTP/HTTPS server parityをRustで実装する | CF-211 | todo | CF-212A HTTP done; HTTPS/TLS transport remains |
| CF-212A | Rust Server | HTTP remote upload/list/clone/show/diff/MR routesをRustで実装する | CF-211 | done | `codefire-rs serve`; `cf+http://` upload/list/clone/show/diff/request-merge/request-list/request-review/request-apply smoke; HTTP transport split into `crates/codefire-cli/src/http.rs`; `cargo test -p codefire-cli --bin codefire-rs`; `cargo clippy -p codefire-cli --all-targets -- -D warnings` |
| CF-213 | Rust Security | commit/request signaturesとkey rotation parityをRustで実装する | CF-212 | todo | v0.5 remote parity |
| CF-214 | Rust Observability | perf/metrics outputをRustで実装する | CF-208, CF-166 | todo | CFB-007 |
| CF-215 | Rust Storage | storage reportとexternal artifact refsをRustで実装する | CF-202, CF-167 | todo | CFB-008 |
| CF-216 | Migration | `migrate check/dry-run` を実装する | CF-209 | todo | v0.6 migration |
| CF-217 | Release | installerをRust binary default + Python fallbackへ切り替える | CF-216 | todo | v0.6 release |
| CF-218 | Automation Interface | state/diagnostic command向けstable JSON envelope/schemaを実装する | CF-200 | todo | v0.6 automation interface |
| CF-219 | Automation Interface | CLI全体のstable exit code taxonomyを実装する | CF-218 | todo | v0.6 automation interface |
| CF-220 | Context | `codefire context` でatom/fire/changed/branch context packを返す | CF-206, CF-218 | todo | v0.6 automation interface |
| CF-221 | Diagnostics | status/scan/verify/doctor/storage/migrateにmachine-readable `next_actions` を追加する | CF-208, CF-220 | todo | v0.6 automation interface |
| CF-222 | Operation Plans | mutating commandに `--dry-run` / operation plan出力を追加する | CF-209, CF-218 | todo | v0.6 automation interface |
| CF-223 | Batch | link/fire/extinguish/evidence/artifactのbatch操作とdry-run validationを実装する | CF-207, CF-222 | todo | v0.6 automation interface |
| CF-224 | Safety | local/remote mutating operationにidempotency keyを実装する | CF-222, CF-211 | todo | v0.6 automation interface |
| CF-225 | Safety | `--wait-lock` / `--lock-timeout` / JSON lock diagnosticsを実装する | CF-203, CF-219 | todo | v0.6 automation interface |
| CF-226 | Evidence | command outputとartifact hashを取り込むevidence capture APIを実装する | CF-215, CF-218 | todo | v0.6 automation interface |
| CF-227 | Explain | fire/atom/verify failure/storage warning向けread-only `codefire explain` を実装する | CF-220, CF-221 | todo | v0.6 automation interface |
| CF-228 | Diff | pluggable Myers / patience / histogram-style text diffを実装する | CF-210 | todo | v0.6 diff intelligence |
| CF-229 | Diff | rename/copy detectionとbinary diff summaryを実装する | CF-228 | todo | v0.6 diff intelligence |
| CF-230 | Diff | Atom diffとTraceGraph diffを実装する | CF-205, CF-228 | todo | v0.6 diff intelligence |
| CF-231 | Impact | policy/fire impact diffとmachine-readable next actionsを実装する | CF-221, CF-230 | todo | v0.6 diff intelligence |
| CF-232 | Merge | merge dry-run predictionとsemantic conflict candidate reportingを実装する | CF-210, CF-231 | todo | v0.6 diff intelligence |
| CF-233 | Review | file diff、Atom diff、Trace diff、verification、next actionsを含むreview-pack exportを実装する | CF-231, CF-232 | todo | v0.6 diff intelligence |
| CF-234 | Patch | review / remote workflow向けpatch export/importを実装する | CF-228, CF-233 | todo | v0.6 diff intelligence |
| CF-235 | Testing | diff/merge golden fixturesとGit comparison smoke testsを追加する | CF-228, CF-232 | todo | v0.6 diff intelligence |

## Immediate Next Actions

1. v0.6 Rust rewriteのcrate境界、golden compatibility tests、automation interface schema方針を先に作る。
2. JSON output、exit code taxonomy、context、next_actionsをv0.3/v0.4から段階導入する。
3. diff algorithm / rename detection / Atom-Trace impact diffのgolden fixture方針を決める。
4. CFB-005/006のUX改善をRust CLI計画へ組み込み、dogfooding中の操作コストと診断ノイズを下げる。
5. YAML parser とobject ID仕様の互換性方針をv0.6 Rust計画で確定する。
