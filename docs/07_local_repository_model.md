# 07. ローカルRepository / Branch / Open Directory設計

## 7.1 repository構造

```text
project/
  .codefire/
    repo.yaml
    objects/
    branches/
    opened/
    active/
    cache/
    locks/
    remotes/

  main/
  feature-login/
```

`.codefire/` が正式repositoryである。

`main/` や `feature-login/` はopenされた実ディレクトリであり、closeすると削除される。

## 7.2 current branchなし

repositoryにcurrent branchは存在しない。

```text
NG: このrepositoryは今mainをcheckoutしている
OK: mainは./mainにopenされている
OK: feature-loginは./feature-loginにopenされている
```

## 7.3 open registry

openされたbranchは `.codefire/opened/` に登録される。

例：

```yaml
version: 1
branch:
  branch_id: br_01HZXA2AE2RPWZ7M2E
  name: feature-login
open:
  open_instance_id: open_01HZXA3P99HHH3ZC8Q
  path: /workspace/project/feature-login
  opened_from_commit: CF-COMMIT-21ab77aa00
  current_base_commit: CF-COMMIT-21ab77aa00
  active_state_path: /workspace/project/.codefire/active/open_01HZXA3P99HHH3ZC8Q
state:
  last_known: open-clean
```

## 7.4 `.codefire-open`

open directoryには識別ファイルを置く。

```yaml
version: 1
repository:
  path: /workspace/project/.codefire
  repository_id: repo_01HZX9Q5XJ3M8K7G2T
branch:
  name: feature-login
  branch_id: br_01HZXA2AE2RPWZ7M2E
  opened_from_commit: CF-COMMIT-abc123
open:
  open_instance_id: open_01HZXA3P99HHH3ZC8Q
  opened_path: /workspace/project/feature-login
```

Code Fireコマンドは、これをもとにopen directoryの正当性を検証する。

## 7.5 同一branchの多重open禁止

同一branchを複数pathにopenすることは禁止する。

理由：

```text
1 branchに複数のburning stateが存在すると、どちらが正式な変更元か不明になる
多重openはcheckpointやstashに近い逃げ道になる
fire/resolutionの由来が曖昧になる
```

## 7.6 OSコピーされたopen directory

ユーザはOSレベルでdirectoryをコピーできる。

```bash
cp -r feature-login feature-login-backup
```

しかし、コピー先ではCode Fire操作を拒否する。

判定：

```text
.codefire-openは存在する
opened_pathが現在pathと違う
またはopen_instance_idがrepository側registryと一致しない
```

これはCode Fire管理外のバックアップであり、checkpointではない。

## 7.7 active state

open branchごとにactive stateを持つ。

```text
.codefire/active/open_xxx/
  state.yaml
  events.jsonl
  fires.yaml
  resolutions.yaml
  verification.yaml
  scan.sqlite
```

active stateには作業中ファイル内容を保存しない。

保存してよいもの：

```text
fire metadata
resolution metadata
scan hash
verification result
open identity
```

保存してはいけないもの：

```text
作業中ファイルのblob
復元可能snapshot
patch
diff
checkpoint
stash
```

