# CodeFire Runbook

このドキュメントは、CodeFire MVP実装の実行手順と確認手順を集約する場所です。

## セットアップ

- Rust v0.6 CLIは `project/crates/` 配下のCargo workspaceで、`install.sh` の既定CLIである。
- Python v0.2 reference CLIは依存追加なし。Python 3.10+ の標準ライブラリだけで動き、`install.sh` では `codefire-py` fallbackとして配置される。
- Python CLI本体は `project/codefire`。
- 実行は `project/` 直下から行う。

インストールする場合:

```bash
cd /workspace/project
./install.sh --prefix "$HOME/.local"
./install.sh --prefix "$HOME/.local" --completion bash --dry-run
./install.sh --prefix "$HOME/.local" --completion bash --completion-dir "$HOME/.local/share/bash-completion/completions"
./install.sh --prefix "$HOME/.local" --binary ./target/release/codefire-rs
python3 -m pip install .
python3 -m pip install --no-build-isolation --no-deps .
```

Rust v0.6 rewriteを開発確認する場合:

```bash
cd /workspace/project
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace
./target/debug/codefire-rs --help
```

`install.sh` はrelease Rust binaryをbuildして `PATH/bin/codefire` に配置し、Python reference implementationを `PATH/bin/codefire-py` として残す。`--binary` 指定時はprebuilt binaryを使うが、install前後に `--help` / `--version` smokeでCodeFire identityを確認する。`--dry-run` はbuild/mkdir/installを行わず、配置予定のbinary/completion pathを表示する。prefixとcompletion dirは絶対pathを要求し、symlink prefix/bin/completion dirは拒否する。`python3 -m pip install .` はPython reference implementationを `codefire` entrypointとして入れるため、Rust default installとは用途を分ける。

## 実行

```bash
cd /workspace/project
./target/debug/codefire-rs --help
./target/debug/codefire-rs completion bash
./target/debug/codefire-rs completion zsh
./codefire --help
```

一気通貫デモ:

```bash
cd /workspace/project
./demo.sh
```

`demo.sh` は一時ディレクトリ内で local init/open/commit、feature branch、local merge、file-backed remote upload、merge request review/apply、remote doctor まで実行する。

最小デモの流れ:

```bash
tmp=$(mktemp -d)
cd "$tmp"
/workspace/project/target/debug/codefire-rs init
/workspace/project/target/debug/codefire-rs open main "$tmp/main"
cd "$tmp/main"

# codefire.yaml / codefire.links.yaml / codefire.policy.yaml と
# docs/spec, docs/design, src, tests を作成する

/workspace/project/target/debug/codefire-rs scan
/workspace/project/target/debug/codefire-rs extinguish FIRE-001 --resolution changed --evidence initial-import
/workspace/project/target/debug/codefire-rs verify
/workspace/project/target/debug/codefire-rs commit -m "Initial consistent commit"
```

`codefire.policy.yaml` の verification command は複数指定できる:

```yaml
version: 1
commit_policy:
  require_no_required_fires: true
  require_no_stale_resolutions: true
  require_trace_completeness: true
  require_verification_success: true
  reject_duplicate_atom_ids: true
extinguish_policy:
  no-change-required:
    requires_rationale: true
required_links:
  requirement:
    - type: refined_by
      target_kind: design
      min: 1
    - type: verified_by
      target_kind: test
      min: 1
  design:
    - type: implemented_by
      target_kind: code
      min: 1
verification:
  required:
    - id: unit-tests
      command: "python3 -m unittest discover -s tests"
      cwd: "."
      timeout_ms: 300000
      max_output_bytes: 65536
      inherit_env: false
    - id: lint-smoke
      command: "python3 -m py_compile src/auth.py"
      cwd: "."
      env: PYTHONDONTWRITEBYTECODE=1
      allow_failure: false
```

Rust defaultでは上記のうち`require_no_required_fires`、`require_no_stale_resolutions`、
`require_verification_success`、`reject_duplicate_atom_ids`はsafety floorなので`true`固定である。
`false`を指定するとverify/commitはconfiguration errorで停止する。missing required linkをwarningへ
落とす必要がある仕様先行作業では、`require_trace_completeness: false`だけを使用する。

verification commandのdefaultは `timeout_ms: 300000`、各stream `max_output_bytes: 65536`、`inherit_env: false`、`env: ""`、`allow_failure: false` である。defaultはcaller environmentをclearし、固定system PATHとLANG/LC_ALL/TZ allowlistへ`env` の `KEY=VALUE` comma-separated listを追加する。`inherit_env: true`は明示的なweaker modeで、実行時Unicode environment snapshotとoverrideを固定してdigestへ束縛する。cwdはopen directory配下だけを許し、outputは実行中からstreaming drainする。`allow_failure: true` は非ゼロ終了だけをnonblockingにできるが、timeoutとworktree subject変更は常に失敗として扱う。`assessment subject no longer matches the worktree` はverify/commitと並行してworktreeが変わったことを示すため、変更processを止めて同じcommandを再実行する。pending commit markerが残った場合も手動削除せず、同じ引数のlive commitでprepublication recoveryを通す。
verification policyはcommit時にpolicy rootへsealされる。command literalや明示`env`へtoken/password/key等を直書きするとpreflightで拒否されるため、secretが必要なcheckは値を含まないhelper scriptと明示`inherit_env: true`を用い、secret outputを出さないこと。

`scan --json` / `verify --json`の`effective_policy`は各ruleの`source`と`effective_value`を示す。
`change_coverage`は変更fileをnon-Atom、deleted Atom、out-of-Atom、Atom変更+未分類spanへ分けるが、
`span_precision=file_only_no_span_claim`は行/byte範囲を証明していないことを意味する。通常JSONのentry sampleは
50件までで、全件が必要な監査だけ`--full`を使う。summaryとdigestは省略件数に関係なく完全なcontractを表す。

commit certificateの `result` はverification rootから導出される。現行語彙は `passed` / `failed` で、既存commit互換のためcertificate側の `consistent` は `verification.result=passed` のlegacy aliasとしてだけ受理する。

`codefire.yaml` / `codefire.links.yaml` / `codefire.policy.yaml` は標準ライブラリのみの限定YAML subsetとして読む。subset内で必須項目が欠けている場合は、`verify` / `scan` 時に `invalid codefire.*.yaml` エラーとして失敗する。

remote file-backed server の例:

```bash
server=$(mktemp -d)
cd "$tmp"

/workspace/project/codefire upload main "cf://$server/org/app/main"
/workspace/project/codefire list "cf://$server/org/app"

/workspace/project/codefire clone main feature
/workspace/project/codefire upload feature "cf://$server/org/app/feature"
/workspace/project/codefire request-merge \
  "cf://$server/org/app/feature" \
  "cf://$server/org/app/main"
/workspace/project/codefire request-list "cf://$server/org/app"
/workspace/project/codefire doctor "cf://$server/org/app"

mr_id=$(/workspace/project/codefire request-list "cf://$server/org/app" | awk 'NR==1 {print $1}')
/workspace/project/codefire request-review "cf://$server/org/app" "$mr_id" \
  --reviewer alice \
  --decision approve \
  --comment "sealed source is ready"
/workspace/project/codefire request-apply "cf://$server/org/app" "$mr_id"
/workspace/project/codefire gc "cf://$server/org/app"
```

remote HTTP server の例。`gc` はPython fallbackで実行する:

```bash
storage=$(mktemp -d)
/workspace/project/codefire serve "$storage" --host 127.0.0.1 --port 8080

# 別shellで実行する
cd "$tmp"
/workspace/project/codefire upload main "cf+http://127.0.0.1:8080/org/app/main"
/workspace/project/codefire list "cf+http://127.0.0.1:8080/org/app"
/workspace/project/codefire clone "cf+http://127.0.0.1:8080/org/app/main" main-from-http

/workspace/project/codefire clone main feature-http
/workspace/project/codefire upload feature-http "cf+http://127.0.0.1:8080/org/app/feature-http"
/workspace/project/codefire show "cf+http://127.0.0.1:8080/org/app/feature-http"
/workspace/project/codefire diff \
  "cf+http://127.0.0.1:8080/org/app/main" \
  "cf+http://127.0.0.1:8080/org/app/feature-http"
/workspace/project/codefire request-merge \
  "cf+http://127.0.0.1:8080/org/app/feature-http" \
  "cf+http://127.0.0.1:8080/org/app/main"
/workspace/project/codefire request-list "cf+http://127.0.0.1:8080/org/app"
/workspace/project/codefire doctor "cf+http://127.0.0.1:8080/org/app"
/workspace/project/codefire gc "cf+http://127.0.0.1:8080/org/app" --dry-run
```

remote HTTPS server の例:

Python v0.2では `/workspace/project/codefire`、Rust v0.6では `/workspace/project/target/debug/codefire-rs` を使う。

```bash
storage=$(mktemp -d)
openssl req -x509 -newkey rsa:2048 -nodes \
  -keyout "$storage/server.key" \
  -out "$storage/server.crt" \
  -days 30 \
  -subj "/CN=127.0.0.1" \
  -addext "subjectAltName=IP:127.0.0.1"

/workspace/project/codefire serve "$storage" \
  --host 127.0.0.1 \
  --port 8443 \
  --tls-cert "$storage/server.crt" \
  --tls-key "$storage/server.key"

# 自署名証明書のローカル検証時のみ使う
CODEFIRE_TLS_INSECURE=1 \
  /workspace/project/codefire list "cf+https://127.0.0.1:8443/org/app"

# Rust v0.6 rewriteで確認する場合
CODEFIRE_TLS_INSECURE=1 \
  /workspace/project/target/debug/codefire-rs list "cf+https://127.0.0.1:8443/org/app"
```

Rust HTTPS serve can require client certificates with `--tls-client-ca <ca-pem>` alongside `--tls-cert` and `--tls-key`. The server private key PEM may be PKCS#8, RSA, or SEC1 EC. Unsupported key files fail with a format-specific error before the listener accepts TLS connections.

server-side verification policy の例:

```bash
mkdir -p "$server/.codefire-server/projects/org/app"
cat > "$server/.codefire-server/projects/org/app/server_policy.json" <<'JSON'
{
  "verification": {
    "required": [
      {
        "id": "commit-env-present",
        "cwd": ".",
        "timeout_seconds": 30,
        "env": {
          "EXAMPLE_ALLOWED": "1"
        },
        "command": "test -n \"$CODEFIRE_REMOTE_COMMIT\" && test \"$EXAMPLE_ALLOWED\" = 1"
      }
    ]
  }
}
JSON
```

server-side verificationの `cwd` はremote project root配下に制限される。親プロセスの環境変数はそのまま渡さず、最小の `PATH` / `HOME`、`CODEFIRE_REMOTE_PROJECT` / `CODEFIRE_REMOTE_BRANCH` / `CODEFIRE_REMOTE_COMMIT`、およびcheckごとの `env` だけを渡す。`timeout_seconds` を超えたcheckは失敗として扱う。

permission policy の例:

```json
{
  "permissions": {
    "upload": ["alice"],
    "request_merge": ["alice"],
    "review": ["reviewer"],
    "apply": ["admin"],
    "gc": ["admin"]
  }
}
```

branch protection policy の例:

```json
{
  "branch_protection": {
    "main": {
      "upload": ["alice"],
      "apply": ["admin"]
    },
    "release/*": {
      "upload": ["release-manager"],
      "apply": ["release-manager"]
    }
  }
}
```

`branch_protection` は `upload` と `request-apply` によるbranch head更新をbranch pattern単位で制御する。patternは `fnmatch` 形式で、`main` や `release/*` を指定できる。`permissions` は操作全体の許可、`branch_protection` は対象branchごとの追加制約として扱う。

実行例:

```bash
/workspace/project/codefire upload main "cf://$server/org/app/main" --actor alice
/workspace/project/codefire gc "cf://$server/org/app" --actor admin
```

commit signature、token authentication、GC retention を含む例:

```bash
/workspace/project/codefire token-hash alice-token
/workspace/project/codefire token-hash admin-token --salt admin-salt
CODEFIRE_SIGNING_KEY=alice-signing-key \
  /workspace/project/codefire commit -m "Signed change" --signer alice --key-id alice-2026-06
CODEFIRE_REQUEST_SIGNING_KEY=alice-request-key \
  /workspace/project/codefire upload main "cf://$server/org/app/main" \
  --actor alice \
  --request-key-id alice-request
```

```json
{
  "permissions": {
    "upload": ["alice"],
    "request_merge": ["alice"],
    "review": ["reviewer"],
    "apply": ["admin"],
    "gc": ["admin"]
  },
  "auth": {
    "required": true,
    "tokens": {
      "alice": "sha256:9c220f200955d76c0a38d308225e0ef10c5f971acaf2f8d1d8f732affa5bd1dc",
      "reviewer": "review-token",
      "admin": {
        "algorithm": "sha256",
        "salt": "admin-salt",
        "hash": "83eaef7876395a355b94a551176c438446f0d211663cff09b7a88fb9d77b1fc1"
      }
    }
  },
  "commit_signatures": {
    "required": true,
    "require_history": true,
    "keys": {
      "alice-2026-01": {
        "secret": "old-alice-signing-key",
        "signers": ["alice"],
        "not_after": "2026-07-01T00:00:00Z"
      },
      "alice-2026-06": {
        "secret": "alice-signing-key",
        "signers": ["alice"],
        "not_before": "2026-06-01T00:00:00Z"
      }
    }
  },
  "request_signatures": {
    "required": true,
    "max_skew_seconds": 300,
    "nonce_ttl_seconds": 300,
    "max_nonce_cache_entries": 10000,
    "keys": {
      "alice-request": {
        "secret": "alice-request-key",
        "signers": ["alice"]
      },
      "admin-request": {
        "secret": "admin-request-key",
        "signers": ["admin"]
      }
    }
  },
  "gc": {
    "retention_seconds": 86400,
    "retention_generations": 2
  }
}
```

```bash
/workspace/project/codefire upload main "cf://$server/org/app/main" \
  --actor alice \
  --token alice-token

CODEFIRE_TOKEN=admin-token \
  /workspace/project/codefire gc "cf://$server/org/app" --actor admin
```

`auth.required` が `true` の場合、対象actorのtokenが `server_policy.json` に存在し、CLIの `--token` または `CODEFIRE_TOKEN` と一致する必要がある。`auth.tokens` は既存互換の平文文字列、`sha256:<hex>` 文字列、または `{"algorithm":"sha256","salt":"...","hash":"..."}` objectを受け付ける。
`commit_signatures.required` が `true` の場合、upload / request-apply でbranch headになるcommitはHMAC-SHA256署名を持ち、`commit_signatures.keys` のsigner keyで検証できる必要がある。署名対象はcommit payloadから `signature` フィールドを除いた内容と署名metadataで、commit object ID自体への循環参照は作らない。`keys` は既存互換の文字列secretに加えて、`secret` / `signers` / `not_before` / `not_after` / `status` を持つobjectを受け付ける。`signed_at`、`not_before`、`not_after` はUTC秒の `YYYY-MM-DDTHH:MM:SSZ` としてparseされ、表記揺れは拒否される。`status: "revoked"` または `"disabled"` のkeyは拒否される。`require_history: true` の場合、branch headだけでなく到達可能な親commitの署名も検証する。
`request_signatures.required` が `true` の場合、upload / request-merge / request-review / request-apply / gc は `CODEFIRE_REQUEST_SIGNING_KEY` と `--request-key-id` で生成されるversion 2 HMAC-SHA256 request署名を持つ必要がある。署名subjectはactor、operation、target、timestamp、nonce、key_id、idempotency keyと、`request_signature`だけを除いた完全canonical payload digestを持つため、head/object/lock fieldの変更もpre-writeで拒否する。`max_skew_seconds` はtimestampずれを制限する。`request_signatures.keys` は文字列secretまたは `secret` / `signers` / `status` / `not_before` / `not_after` を持つobjectを受け付け、windowはsigned timestampへ適用する。nonce replay cacheはremote projectの `request_nonce_cache.json` に保存され、actor/key単位で`nonce_ttl_seconds` 中の同一nonceをpayloadにかかわらず拒否する。malformed cache/entryは保存してfail closedし、`max_nonce_cache_entries` 到達時もTTL内entryを削除せず、新しいrequestをfail closedする。`storage report --remote ... --json` はnonce cacheのentry_count/max_entries/ttl_secondsを返す。
`server_policy.json`、`request_signatures`、`required`、key validity windowの型/UTC秒形式が不正な場合はunsigned許可へfallbackせず、最初のremote layout/domain write前にconfiguration/authentication failureとなる。
GCは削除前にremote object hash、missing object reference、sealed commit参照を検証する。異常がある場合は削除せずに失敗する。
`gc.retention_seconds` は到達不能objectでもmtimeが保持期間内なら削除しない。
`gc.retention_generations` は到達不能objectでも直近N世代でremoteへコピーまたは参照されたobjectなら削除しない。remote projectの現在世代は `gc_state.json` に保存され、branch head更新時に進む。
GCのdry-runと実削除はremote project内のexact `gc/GC-*.json` をaudit authorityとして記録する。各recordにはactor、retention秒数、retention世代数、現在世代、reachable/protected/removed件数、removed/protected object一覧が入る。`audit/gc.jsonl` はexact recordから決定的に再構築される閲覧用derived viewであり、exactly-once判定には使わない。GC中断時は同じactor/modeのretryだけが固定 `pending_transaction.json` とplan-owned quarantine/delete progressを検証して再開でき、foreign/future transactionは保存して拒否する。

## テスト

```bash
cd /workspace/project
python3 -m unittest discover -s tests -v
```

repository / remote project の診断:

```bash
/workspace/project/codefire doctor
/workspace/project/codefire doctor "cf://$server/org/app"
/workspace/project/codefire doctor "cf://$server/org/app" --quick --json
```

`doctor` は object recordのhash/id/filename mismatch、branch/MR/commit rootから構造化されたobject参照として辿れるはずの missing object reference、branch/MRが指すsealed commit参照の不正を報告する。sealed commit validationでは、parents/roots/certificateの構造、parent履歴、commit/root version、root schema、期待object type、reachable reference、Atom/Trace endpointとschema-2のFire/Resolution/policy/verification/certificate相互整合も検証する。full object scanは64 MiB上限のpinned no-follow root/subdirectory snapshotであり、symlink rootやidentity差し替えは成功扱いしない。

remote URLはlocal pathへ変換せず、file-backedまたはHTTP(S)の`codefire.remote_doctor.v1` reportを返す。
missing projectもtyped reportになる。remote listを分割する場合は
`codefire list "$url" --limit 100 --json`または`request-list`の`data.page.next_cursor`を、同じlimitで
次の呼び出しへ渡す。途中でbranch/MR collectionが変わってstale cursorになった場合は、cursorを捨てて
先頭ページからやり直し、古いページと新しいページを結合しない。

## よくある詰まり

- `not inside a CodeFire repository`: `codefire init` 済みのrepo内、または `.codefire-open` を持つopen directory内で実行する。
- `copied open directory detected`: open directoryをOSコピーした疑いがある。正規の `codefire open` で開き直す。
- slash入りbranch名は使える。内部の `.codefire/branches/` と `.codefire/opened/` ではURLエンコードしたファイル名で保存される。
- remote URLでslash入りbranch名を指す場合は `cf://.../feature%2Fsession` のようにbranch部分をURLエンコードする。
- `commit blocked`: `codefire verify` の出力で unresolved merge conflict、open fire、missing link、stale resolution、duplicate Atom ID、failed check を確認する。
- `unsupported safety policy relaxation`: messageのrule ID、source、adoptionを確認する。safety-floor ruleを`false`へ変更して回避せず、原因となるFire、Resolution、verification、duplicate Atomを解消する。現行でrelax可能なのはdocumentedなtrace-completeness policyだけである。
- `Blocking checks:`: `codefire verify` がpolicy上commit blockerとして扱っている項目だけを列挙する。件数表示にはnon-blocking diagnosticsも含まれる。
- `verify` の件数だけでは対象が分からない場合は `codefire verify --details` でmerge conflict、missing link、stale resolution、duplicate Atom ID、failed checkの代表例を確認する。
- `Unresolved merge conflicts: N` / exit 15: text markerを解消し、binary conflictでは選択・再構築後に`.codefire-conflicts/<path>/target|source`を除去する。`codefire merge --continue --path <open-dir> --dry-run --json`で再検査し、semantic candidateを確認済みなら`--accept-semantic-conflicts`を付けてlive continueする。`state.json`のconflict arraysを手編集しない。continue後にverifyし、通常commitで二親履歴をsealする。
- `Stale resolutions: N`: 過去のfire解消後にsource/target/link/policyが変わっている。再確認済みなら `codefire extinguish FIRE-xxx --resolution changed --evidence ... --refresh` でresolution basisを更新する。
- `cannot clone/merge from branch ... open-burning`: source branchをcommitするかdiscardして、sealed headが確定してから再実行する。
- `sealed commit validation failed`: `codefire doctor` でbranch headやobject storeの不正を確認する。`branch list` / `list` / `open` / `clone` / `merge` / `show` / `diff` / `status` / `scan` / `fire` / `extinguish` / `verify` / `commit` / `request-list` / `request-review` / `request-apply` は壊れた履歴を通常操作や表示へ広げない。
- `unsupported commit.root_schema_version` / `unsupported commit.version` / `unsupported <root>.version`: 未知future versionを現行schemaへ書き換えない。schema欠落だけがlegacy root schema 1の互換経路であり、schema 1/2以外やcommit/root version 1以外はmigration対応が用意されるまで隔離する。schema 2の`missing_evidence_refs`欠落は空の場合だけ互換で、非空blockerやcertificate count不一致は修復せずcorruptionとして扱う。
- `unsupported remote branch/merge request version`: recordを手作業でversion 1へ書き換えない。`doctor <remote> --json`で対象を隔離し、対応reader/migrationが用意されるまでmutationを止める。
- `stale page cursor`: collectionが最初のpage取得後に変わった。cursorを捨ててpage 1から再取得する。異なるURL、command、limitへcursorを流用しない。
- repairable layout warningは、まず `codefire migrate dry-run --path <repo> --target-format current --full --json` でplanを確認し、その後 `codefire migrate apply --path <repo> --target-format current --full --json` を実行する。applyはrepository lock下でplanを再計算し、shared registryのauto-create directoryだけを作る。`--quick` はapplyでは使用できない。`--target-format v0.6` もcompatibility profile選択でありformat downgrade/writeではない。
- applyが`migration_plan_changed`、`migration_subject_changed`、`migration_action_precondition_changed`を返した場合、directoryを手動で追従作成せず、競合処理の終了とpath identityを確認してdry-runからやり直す。blockerがある場合はlayout writeを行わず、diagnosticのdoctor/migrate check導線で破損を先に解決する。
- object/repository JSONのoversize・symlink・non-regular error: object recordとgeneric repository JSONは64 MiB上限で、readerはno-follow/non-blocking regular-fileだけを受理する。FIFO/device/socketを読ませたりsymlinkをregular fileへ追わせたりせず、正しいauthorityからregular fileを復元する。object root/subdirectory自体がsymlinkまたはidentity mismatchなら、そのstoreを移動・削除せず隔離して`doctor`を再実行する。
- `pending_state_transaction`: live scan/verifyがderived state更新中に中断した。`doctor --json` はmarker pathとmessage内のoperation/phase/intended stateを報告する。該当repoで他のmutatorが動いていないことを確認し、同じopen directoryへ `codefire scan --wait-lock --json` または `codefire verify --wait-lock --json` を再実行する。retryはmarkerを最初のrecovery write前に検証して全stateを再計算する。markerを先に手作業で削除しない。`invalid_active_state_shape`の場合はunknown/malformed markerなので自動retry対象とみなさない。
- `pending_commit_transaction`: commitがpublication前後で中断した。`codefire doctor --json` はmarker pathとmessage内のphase/stored role-entry件数/planned role-entry件数までを報告する。role/path/object_type/object_id/stateとcandidate aliasはdoctor issueの独立fieldではないため、marker fileを変更せずコピーしてそのfieldを確認する。`planned`でもstore-to-marker gapによりobjectが存在し得るので、entry IDのobjectを勝手に削除しない。branch/opened headとmarkerを手動変更せずに原因を解消する。legacy string entryはstoredとして扱われる。
- `pending_close_transaction`: close/discardがworktree、active state、opened registry、branchの途中で中断した。marker/quarantine/completed archiveを削除・renameせず、doctorのbranch/phase/evidenceを確認して、元と完全に同じ`close`または`discard`（`--keep-dir`を含む）を`--wait-lock`付き・非dry-runで再実行する。dry-runはauthorityを検証して再実行方法を報告するだけで回復しない。
- `conflicting_repository_transactions`: pending openとpending closeが同時に存在する。どちらを先と推測して再開したり片方を手動削除したりしない。両marker、completed archive、worktree/active/opened/branch evidenceを保存して、最後に成功したvisibility pointを調査する。divergent evidenceは通常commandによる自動修復対象ではない。
- `lock_contention`: `.codefire/locks/repo.lock`や`.codefire/locks/repo.lock.owner.json`の存在だけではheld判定しない。`--wait-lock --lock-timeout ...`でkernel lockの解放を待つ。`repo.lock`はcontent-immutableかつexactly-one-linkのlock inode、owner sidecarは最後に取得したprocessのbest-effort情報で、unlock後も両方残る。sidecarがmalformed/symlink/16 KiB超過ならowner情報が表示されないのが正しいfail-closed動作であり、手動でlock fileをunlink/truncateしない。hard-link/identity provenance errorは別fileへのaliasやpathname swapを疑い、対象を変更せず隔離して調査する。
- `open_target_resolution_failed`: diagnosticの`reason`、`requested_path`、`resolved_path`、`workspace_boundary`を確認する。`..`やsymlinkを含めず、repositoryや別open directoryと重ならないrepository siblingを選び、提示された`open ... --dry-run --json`で先にpreviewする。race系reasonでは既存targetの所有者を確認し、CodeFireが作ったと決めつけて削除しない。
- `trace_graph_endpoint_integrity`: `issues[].kind` の`missing_atom` / `ambiguous_atom_id`、endpoint、Atom IDを修正する。issuesはbounded sampleなので、`total_issues` / `issues_omitted`が正なら修正後にscan/link batchを再実行して残件を確認する。
- import retry: `.codefire-import.lock`、`.codefire-import-transaction.json`、`.codefire/locks/repo.lock`は復旧のauthority/証拠であり、lock pathの存在だけはcontentionを意味しない。手動削除せず同じtarget/branch/idempotency requestを`--wait-lock`付きで再実行する。import repository lockも通常lockと同じcontent不変/exactly-one-link/owner-sidecar契約である。rootの`.codefire-import.lock.bootstrap-*.part`は未公開scratchでありauthorityにはならず、unlinkedのものは保存・scan除外される。fully-written `.tmp`だけがbootstrap recovery candidateで、`.codefire-import.write-<transaction-id>-*.tmp`はID一致時だけ回収される。foreign/malformed published `.tmp`は保存されfail closedする。1 MiB control JSON、64 MiB repository byte-copy、49,545,216-byte user fileの上限超過やsymlink/FIFO等は自動回収対象ではなく、安全なregular fileへ直してから同一requestを再実行する。
- `open registry current_base_commit is invalid` / `open branch head is invalid`: 既に開いている作業ディレクトリのregistryまたはbranch headが破損している。`codefire doctor` で診断し、必要なら正しいsealed commitから開き直す。
- `merge request validation failed`: MR内の `source_head` / `target_head_at_request` / `applied_head` がsealed commitとして妥当か確認する。
- `extinguish requires --rationale or --evidence`: fireを解消する判断根拠を必ず残す。
- `no-change-required requires --rationale`: 変更不要でfireを消す場合は理由を必ず残す。
