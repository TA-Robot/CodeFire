# 03. 開発フロー仕様

## 3.1 標準フロー

Code Fireの標準フローは以下である。

```text
remote/localのsealed branchをclone
  ↓
branchをopenして実ディレクトリ化
  ↓
編集
  ↓
scan
  ↓
fire発火
  ↓
extinguish
  ↓
verify
  ↓
commit
  ↓
uploadまたはmerge request
  ↓
close
```

## 3.2 例：feature branchを作る

```bash
codefire clone main feature-login
codefire open feature-login ./feature-login
```

## 3.3 例：変更してfireを立てる

```bash
cd feature-login
# src/auth.py などを編集
codefire scan
```

出力例：

```text
Branch: feature-login
State: open-burning
Commit: blocked

Changed atoms:
  CODE-SessionPolicy

Open fires:
  FIRE-001  CODE-SessionPolicy -> DES-AUTH-001
  FIRE-002  CODE-SessionPolicy -> REQ-AUTH-001
  FIRE-003  CODE-SessionPolicy -> TEST-session-expiration
```

## 3.4 fire解消

設計書を変更した場合：

```bash
codefire extinguish FIRE-001 \
  --resolution changed \
  --evidence docs/design/auth.md#DES-AUTH-001
```

仕様変更不要の場合：

```bash
codefire extinguish FIRE-002 \
  --resolution no-change-required \
  --rationale "外部仕様は変化しておらず、内部実装の整理のみであるため"
```

テストを更新した場合：

```bash
codefire extinguish FIRE-003 \
  --resolution changed \
  --evidence tests/test_auth.py#TEST-session-expiration
```

## 3.5 verify

```bash
codefire verify
```

出力例：

```text
Verification passed.

Open fires: 0
Missing required links: 0
Stale resolutions: 0
Duplicate atom ids: 0

State: open-consistent
Commit: allowed
```

## 3.6 commit

```bash
codefire commit -m "Change session expiration policy"
```

出力例：

```text
Sealed commit created.

Commit: CF-COMMIT-8f3a91d0c2
Branch: feature-login
State: open-clean
```

## 3.7 close

```bash
codefire close feature-login
```

実ディレクトリ `./feature-login` は削除される。branch履歴は残る。

## 3.8 長い作業の扱い

checkpointは存在しない。長期間commitできない作業は、変更単位が大きすぎるというシグナルである。

Code Fireでは、大きな変更を以下のように分割することを推奨する。

```text
1. 互換性を保った内部構造変更を整合commitする
2. 仕様に見える小変更を整合commitする
3. テストと設計を更新して整合commitする
4. 古い経路を削除して整合commitする
```

AIにも「次の整合可能な最小commitを作る」ことを指示する。

