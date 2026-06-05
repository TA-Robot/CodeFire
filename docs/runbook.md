# CodeFire Runbook

このドキュメントは、CodeFire MVP実装の実行手順と確認手順を集約する場所です。

## セットアップ

- Python v0.2 reference CLIは依存追加なし。Python 3.10+ の標準ライブラリだけで動く。
- Python CLI本体は `project/codefire`。
- Rust v0.6 rewriteは `project/crates/` 配下のCargo workspaceで、開発時は `cargo build --workspace` または `cargo run -p codefire-cli --bin codefire-rs -- ...` を使う。
- 実行は `project/` 直下から行う。

インストールする場合:

```bash
cd /workspace/project
./install.sh --prefix "$HOME/.local"
./install.sh --prefix "$HOME/.local" --completion bash --completion-dir "$HOME/.local/share/bash-completion/completions"
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

v0.6のinstaller切替が完了するまでは、`install.sh` の既定はPython `codefire` である。Rust binaryを既定の `codefire` にし、Python fallbackを `codefire-py` として残す作業は `CF-217` で追跡する。

## 実行

```bash
cd /workspace/project
./codefire --help
./codefire completion bash
./codefire completion zsh
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
/workspace/project/codefire init
/workspace/project/codefire open main "$tmp/main"
cd "$tmp/main"

# codefire.yaml / codefire.links.yaml / codefire.policy.yaml と
# docs/spec, docs/design, src, tests を作成する

/workspace/project/codefire scan
/workspace/project/codefire extinguish FIRE-001 --resolution changed --evidence initial-import
/workspace/project/codefire verify
/workspace/project/codefire commit -m "Initial consistent commit"
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
    - id: lint-smoke
      command: "python3 -m py_compile src/auth.py"
      cwd: "."
```

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

remote HTTP server の例:

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
`commit_signatures.required` が `true` の場合、upload / request-apply でbranch headになるcommitはHMAC-SHA256署名を持ち、`commit_signatures.keys` のsigner keyで検証できる必要がある。署名対象はcommit payloadから `signature` フィールドを除いた内容と署名metadataで、commit object ID自体への循環参照は作らない。`keys` は既存互換の文字列secretに加えて、`secret` / `signers` / `not_before` / `not_after` / `status` を持つobjectを受け付ける。`status: "revoked"` または `"disabled"` のkeyは拒否される。`require_history: true` の場合、branch headだけでなく到達可能な親commitの署名も検証する。
`request_signatures.required` が `true` の場合、upload / request-merge / request-review / request-apply / gc は `CODEFIRE_REQUEST_SIGNING_KEY` と `--request-key-id` で生成されるHMAC-SHA256 request署名を持つ必要がある。署名対象はactor、operation、target、timestamp、nonce、key_idで、`max_skew_seconds` によりtimestampずれを拒否できる。`request_signatures.keys` は文字列secretまたは `secret` / `signers` / `status` を持つobjectを受け付ける。nonce replay cacheはremote projectの `request_nonce_cache.json` に保存され、`nonce_ttl_seconds` の範囲で同じ署名requestを拒否する。
GCは削除前にremote object hash、missing object reference、sealed commit参照を検証する。異常がある場合は削除せずに失敗する。
`gc.retention_seconds` は到達不能objectでもmtimeが保持期間内なら削除しない。
`gc.retention_generations` は到達不能objectでも直近N世代でremoteへコピーまたは参照されたobjectなら削除しない。remote projectの現在世代は `gc_state.json` に保存され、branch head更新時に進む。
GCのdry-runと実削除はremote project内の `audit/gc.jsonl` にJSONL形式で記録される。各行にはactor、retention秒数、retention世代数、現在世代、reachable/protected/removed件数、removed/protected object一覧が入る。

## テスト

```bash
cd /workspace/project
python3 -m unittest discover -s tests -v
```

repository / remote project の診断:

```bash
/workspace/project/codefire doctor
/workspace/project/codefire doctor "cf://$server/org/app"
```

`doctor` は object recordのhash/id/filename mismatch、branch/MR/commit rootから構造化されたobject参照として辿れるはずの missing object reference、branch/MRが指すsealed commit参照の不正を報告する。sealed commit validationでは、parents/roots/certificateの構造、parent履歴、commit rootが期待するobject typeを指しているかも検証する。

## よくある詰まり

- `not inside a CodeFire repository`: `codefire init` 済みのrepo内、または `.codefire-open` を持つopen directory内で実行する。
- `copied open directory detected`: open directoryをOSコピーした疑いがある。正規の `codefire open` で開き直す。
- slash入りbranch名は使える。内部の `.codefire/branches/` と `.codefire/opened/` ではURLエンコードしたファイル名で保存される。
- remote URLでslash入りbranch名を指す場合は `cf://.../feature%2Fsession` のようにbranch部分をURLエンコードする。
- `commit blocked`: `codefire verify` の出力で open fire、missing link、stale resolution、duplicate Atom ID、failed check を確認する。
- `Blocking checks:`: `codefire verify` がpolicy上commit blockerとして扱っている項目だけを列挙する。件数表示にはnon-blocking diagnosticsも含まれる。
- `verify` の件数だけでは対象が分からない場合は `codefire verify --details` でmissing link、stale resolution、duplicate Atom ID、failed checkの代表例を確認する。
- `Stale resolutions: N`: 過去のfire解消後にsource/target/link/policyが変わっている。再確認済みなら `codefire extinguish FIRE-xxx --resolution changed --evidence ... --refresh` でresolution basisを更新する。
- `cannot clone/merge from branch ... open-burning`: source branchをcommitするかdiscardして、sealed headが確定してから再実行する。
- `sealed commit validation failed`: `codefire doctor` でbranch headやobject storeの不正を確認する。`branch list` / `list` / `open` / `clone` / `merge` / `show` / `diff` / `status` / `scan` / `fire` / `extinguish` / `verify` / `commit` / `request-list` / `request-review` / `request-apply` は壊れた履歴を通常操作や表示へ広げない。
- `open registry current_base_commit is invalid` / `open branch head is invalid`: 既に開いている作業ディレクトリのregistryまたはbranch headが破損している。`codefire doctor` で診断し、必要なら正しいsealed commitから開き直す。
- `merge request validation failed`: MR内の `source_head` / `target_head_at_request` / `applied_head` がsealed commitとして妥当か確認する。
- `extinguish requires --rationale or --evidence`: fireを解消する判断根拠を必ず残す。
- `no-change-required requires --rationale`: 変更不要でfireを消す場合は理由を必ず残す。
