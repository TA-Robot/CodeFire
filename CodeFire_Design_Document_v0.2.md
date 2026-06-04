# Code Fire 仕様・設計ドキュメント v0.2

この文書は、Code Fireの思想背景、外部仕様、内部実装設計、MVP計画を統合したものです。


---

# 00. 思想背景

## 0.1 問題意識

従来のバージョン管理は、主にファイルの履歴を管理してきた。Gitはその代表であり、非常に強力な分散バージョン管理システムである。しかし、Gitの基本的な関心は、ファイル集合のスナップショット、差分、ブランチ、マージである。

一方、現代のソフトウェア開発で本当に問題になるのは、ファイルが保存されているかどうかだけではない。

問題は、次のような成果物同士の意味的な乖離である。

```text
仕様書と設計書がずれる
設計書とコードがずれる
コードとテストがずれる
仕様書とテストがずれる
API仕様と実装がずれる
ADRと現在の実装判断がずれる
```

AIコーディングが普及すると、この問題はさらに大きくなる。AIは大量のコード変更を高速に実行できるが、その速度は仕様書、設計書、テスト、トレーサビリティの更新速度を超えやすい。結果として、コードだけが先に進み、周辺成果物が置き去りになる。

Code Fireは、この状況に対して次の原則を置く。

> 仕様・設計・コード・テストが整合した状態だけを正式履歴に残す。

## 0.2 Git的自由からの離脱

Gitの強さは自由度にある。

```text
好きな時点でcommitできる
WIP commitを作れる
stashできる
partial commitできる
rebaseできる
force pushできる
pullで取得と統合をまとめて行える
```

しかし、Code Fireの目的から見ると、この自由度は副作用を持つ。

不整合状態をcommitできるなら、履歴は「整合済み状態の列」ではなくなる。WIP commitやcheckpointが使えるなら、不整合状態が管理対象として保存される。stashがあれば、未整合の変更を横に置いて後から戻せる。partial commitがあれば、仕様・設計・コード・テストが分断される。

Code Fireは、これらを機能不足としてではなく、意図的に提供しない。

Code Fireは、開発者に自由な履歴操作を与えるツールではない。開発者とAIに対して、整合する小さな変更単位を作ることを強制するツールである。

## 0.3 AI時代の開発規律

AIコーディング時代には、人間が一つひとつのファイル変更を把握し続けることは難しくなる。その代わりに、AIと人間が共有できる作業単位を定義する必要がある。

Code Fireでは、その作業単位を `fire` として扱う。

fireは不整合そのものではなく、未解消の確認責任である。

例：

```text
CODE-SessionPolicy が変更された
  ↓
DES-AUTH-001 を確認する必要がある
REQ-AUTH-001 を確認する必要がある
TEST-session-expiration を確認する必要がある
```

この確認責任がすべて解消されるまで、commitはできない。

AIには、「リポジトリをいい感じに直せ」と指示するのではなく、次のように指示できる。

```text
未解消fireをすべて解消し、commit可能状態にせよ。
```

これはAI開発において極めて重要である。AIの作業目標が、曖昧な自然言語タスクから、明示的な整合回復タスクになるからである。

## 0.4 履歴の意味を変える

Gitのcommitは、おおむねファイルスナップショットである。

Code Fireのcommitは異なる。

Code Fireのcommitは、以下を含むsealed stateである。

```text
成果物ファイルの内容
Artifact Atom索引
Trace Graph
fire解消記録
検証結果
policy
整合性証明
```

つまり、Code Fireのcommitは保存ではなく、封印である。

> Code Fire commit = consistent system state seal

このため、正式履歴上の任意のcommitは、少なくともCode Fire policy上は整合済みである。

## 0.5 Code Fireの価値

Code Fireが提供する価値は、履歴の自由度ではない。

Code Fireの価値は、次の保証である。

```text
任意のcommitを開いても、未解消fireは存在しない
任意のcommitは、その時点のpolicyに基づく整合性証明を持つ
任意のcommitには、なぜ変更不要と判断したかの証跡が残る
serverやmerge requestには整合済み状態だけが流通する
```

この保証は、AIコーディングの高速化と引き換えに失われがちな意味的整合性を、開発フローの中核に戻す。



---

# 01. プロダクト定義

## 1.1 一文定義

Code Fireは、仕様・設計・コード・テスト・トレーサビリティを単一のシステムグラフとして管理し、未解消の確認責任が存在する状態を正式履歴にcommitすることを禁止する、整合性ファーストの仕様駆動VCSである。

短く言えば、次である。

> Code Fireは、整合済みシステム状態だけをcommitできる、AI時代の仕様駆動バージョン管理システムである。

## 1.2 プロダクト目的

Code Fireの目的は、以下の状態を防ぐことである。

```text
コードだけが変わったが、仕様は未確認
仕様は変わったが、テストは未更新
設計判断は変わったが、ADRが古い
API挙動は変わったが、OpenAPI定義が古い
テストは変わったが、要求仕様との関係が追えない
```

Code Fireは、成果物間の関係をTrace Graphとして管理し、変更時に関連成果物へfireを発火させる。fireがすべてextinguishされるまで、commitは拒否される。

## 1.3 非Git方針

Code FireはGitと共存しない。Gitを内包するわけでも、Git repositoryを裏側に使うわけでもない。

以下は提供しない。

```text
checkout
current branch
staging area
partial commit
stash
checkpoint
pull
fetch
rebase
force upload
remote tracking branch
WIP commit
```

この非提供は制約ではなく、プロダクト思想である。

## 1.4 提供する中核体験

ユーザ体験は以下である。

```text
1. sealed branchをcloneする
2. branchをopenし、実ディレクトリとして展開する
3. 仕様・設計・コード・テストを編集する
4. scanにより変更Atomを検出する
5. Trace Graphに基づきfireが立つ
6. fireをextinguishする
7. verifyする
8. consistentならcommitできる
9. 必要ならuploadまたはmerge requestする
10. closeすると実ディレクトリが消える
```

## 1.5 基本保証

Code Fireは以下を保証する。

```text
branch headは常にsealed commitを指す
sealed commitは常にconsistent certificateを持つ
serverにuploadできるのはsealed branchのみ
merge requestにできるのはsealed branchのみ
open-burning状態は正式履歴に入らない
open-burning状態はserverに流通しない
```

## 1.6 Code Fireが保証しないこと

Code Fireは、世界に対する絶対的な正しさを保証しない。

Code Fireが保証するのは、次である。

> 対象commitが、その時点のCode Fire policy上で、未解消の確認責任を持たず、必要な検証に通っていること。

したがって、policyが不十分なら、Code Fire上でconsistentでも実際のビジネス要求とずれる可能性はある。Code Fireは真理判定機ではなく、確認責任の管理システムである。



---

# 02. 中核概念

## 2.1 Repository

Repositoryは、Code Fireが管理するプロジェクト全体の保管庫である。

Repositoryは以下を持つ。

```text
sealed commit群
branch群
object store
policy
open directory管理情報
server接続情報
cache
```

Repositoryにcurrent branchは存在しない。

## 2.2 Branch

Code Fireのbranchは、Git branchではない。

Code Fire branchは、整合済みcommitだけからなる開発線である。

```text
C1 consistent -> C2 consistent -> C3 consistent
```

branch headは常にsealed commitを指す。

## 2.3 Open Directory

branchは、`open` すると実ディレクトリとして展開される。

```bash
codefire open main ./main
codefire open feature-login ./feature-login
```

これにより、複数branchを同時に開ける。

```text
project/
  .codefire/
  main/
  feature-login/
```

Gitのcheckoutのように、1つの作業ディレクトリを切り替えることはしない。

## 2.4 Close

`close` はopenされたbranch directoryを削除する。

```bash
codefire close feature-login
```

cleanでないbranchは通常closeできない。

```bash
codefire close feature-login --discard
```

を使うと、未commit作業を破棄してcloseする。

## 2.5 Clone

Code Fireのcloneは、sealed branch headから新しいbranchを作る操作である。

```bash
codefire clone main feature-login
```

open directoryの未commit状態をコピーする操作ではない。

## 2.6 Commit

Code Fireのcommitは、整合済み状態の封印である。

commitには以下が含まれる。

```text
content manifest
artifact index
atom index
trace graph
fire delta
resolution records
verification result
policy
consistency certificate
```

未解消fireが残っている状態ではcommitできない。

## 2.7 Artifact

Artifactは、Code Fireが管理する成果物である。

例：

```text
仕様書
設計書
ADR
API仕様
DBスキーマ
コード
テスト
運用Runbook
```

## 2.8 Artifact Atom

Artifact Atomは、成果物内の意味的な最小管理単位である。

例：

```text
REQ-AUTH-001
DES-AUTH-001
CODE-SessionPolicy
TEST-session-expiration
API-login
```

Code Fireではファイル単位ではなくAtom単位で変更影響を管理する。

## 2.9 Trace Link

Trace LinkはArtifact Atom同士の関係である。

例：

```text
REQ-AUTH-001 refined_by DES-AUTH-001
DES-AUTH-001 implemented_by CODE-SessionPolicy
REQ-AUTH-001 verified_by TEST-session-expiration
```

トレーサビリティ表はTrace Linkから生成される派生ビューであり、手編集する一次情報ではない。

## 2.10 Fire

fireは未解消の確認責任である。

```text
CODE-SessionPolicy changed
  -> DES-AUTH-001 の確認が必要
  -> REQ-AUTH-001 の確認が必要
  -> TEST-session-expiration の確認が必要
```

fireはcommit blockerである。

## 2.11 Extinguish

extinguishはfireを解消する操作である。

解消方法：

```text
changed
verified
no-change-required
invalid-fire
```

`no-change-required` でも理由が必要である。



---

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



---

# 04. CLI仕様

## 4.1 最小コマンドセット

```bash
codefire init

codefire branch list

codefire open <branch> <path>
codefire close <branch>
codefire close <branch> --discard

codefire clone <source-branch-or-url> <new-branch>

codefire status
codefire scan
codefire fire <atom-id> --reason <text>
codefire extinguish <fire-id> --resolution <type> [--rationale <text>] [--evidence <ref>]
codefire verify
codefire commit -m <message>

codefire merge <source-branch> --into <target-branch>

codefire upload <branch> <server-url>
codefire request-merge <source-url> <target-url>

codefire list <server-url>
codefire show <branch-or-url>
codefire diff <branch-or-url> <branch-or-url>
codefire request-list <server-url>

codefire discard <branch>
codefire doctor
```

## 4.2 作らないコマンド

```text
checkout
pull
fetch
rebase
stash
checkpoint
add
stage
reset --soft
force-upload
```

## 4.3 `open`

```bash
codefire open main ./main
```

仕様：

```text
branch headのsealed commitを指定pathに展開する
同一branchの多重openは禁止する
target pathは存在しないか空でなければならない
.codefire-openを生成する
open registryを更新する
```

## 4.4 `close`

```bash
codefire close feature-login
```

仕様：

```text
open-cleanの場合のみ実ディレクトリを削除する
open-burningまたはopen-consistentの場合は拒否する
```

破棄して閉じる場合：

```bash
codefire close feature-login --discard
```

## 4.5 `clone`

```bash
codefire clone main feature-login
```

仕様：

```text
source branchのsealed headをtarget branchのheadとして設定する
sourceがopen-burningの場合は拒否する
target branchはclosed状態で作成される
```

remoteから取得する場合：

```bash
codefire clone cf://server/org/app/main main-latest
```

remote tracking branchは作らない。

## 4.6 `scan`

```bash
codefire scan
```

仕様：

```text
open directoryをindexする
Atom単位で変更を検出する
Trace Graphに基づきfireを生成する
obsolete fireを整理する
branch stateを更新する
```

## 4.7 `fire`

```bash
codefire fire REQ-AUTH-001 --reason "仕様と実装が一致していない可能性がある"
```

仕様：

```text
manual fireを立てる
manual fireはrevertで自動消滅しない
extinguishが必要
```

## 4.8 `extinguish`

```bash
codefire extinguish FIRE-001 \
  --resolution no-change-required \
  --rationale "外部仕様は変化していない"
```

仕様：

```text
fireを解消する
basisとしてsource/target/link/policy hashを保存する
必要なrationale/evidenceがない場合は拒否する
```

## 4.9 `verify`

```bash
codefire verify
```

仕様：

```text
scanを実行する
policy checkを実行する
required verification commandを実行する
verification objectをactive stateに保存する
```

## 4.10 `commit`

```bash
codefire commit -m "Implement login handler"
```

仕様：

```text
scanとverifyを再実行する
committable条件を満たす場合だけsealed commitを作る
branch headを更新する
active stateをresetする
open stateをopen-cleanにする
```

## 4.11 `merge`

```bash
codefire merge feature-login --into main
```

仕様：

```text
source sealed headをtarget open directoryへ統合する
targetはopen-cleanでなければならない
merge実行時点ではcommitを作らない
merge fireを生成し、targetをopen-burningにする
```

## 4.12 `upload`

```bash
codefire upload feature-login cf://server/alice/app/feature-login
```

仕様：

```text
sealed branchだけuploadできる
open-burning branchはupload不可
server branchが進んでいたら拒否する
force uploadは存在しない
```

## 4.13 `request-merge`

```bash
codefire request-merge \
  cf://server/alice/app/feature-login \
  cf://server/org/app/main
```

仕様：

```text
source sealed commitをtargetに統合してほしいという申請を作る
source/targetともsealed commitを指す
不整合状態はmerge request不可
```



---

# 05. 整合性モデル

## 5.1 整合の定義

Code Fireにおける整合は、絶対的な正しさではない。

Code Fireにおける整合とは、定義されたpolicyのもとで、未解消の確認責任がなく、必須リンクが存在し、検証に成功している状態である。

## 5.2 commit可能条件

作業状態を \( W \) とする。

変数の定義：

- \( W \)：現在のopen directory上の作業状態
- \( F_{open}(W) \)：状態 \( W \) に存在する未解消required fireの集合
- \( C_{fail}(W) \)：状態 \( W \) で失敗している検証の集合
- \( L_{missing}(W) \)：状態 \( W \) で欠落している必須Trace Linkの集合
- \( R_{stale}(W) \)：状態 \( W \) で古くなったresolutionの集合
- \( D_{dup}(W) \)：状態 \( W \) で重複しているAtom IDの集合

Code Fireでcommit可能である条件は以下である。

\[
committable(W) =
|F_{open}(W)| = 0
\land |C_{fail}(W)| = 0
\land |L_{missing}(W)| = 0
\land |R_{stale}(W)| = 0
\land |D_{dup}(W)| = 0
\]

## 5.3 open state

branchは以下の状態を持つ。

```text
closed
open-clean
open-burning
open-consistent
```

### closed

branchは存在するが、実ディレクトリは存在しない。

### open-clean

実ディレクトリが存在し、branch headと一致している。

### open-burning

実ディレクトリに変更または未解消fireがあり、commit不可である。

### open-consistent

変更はあるが、未解消fireがなく、検証に成功しており、commit可能である。

## 5.4 状態遷移

```text
closed
  │ open
  ↓
open-clean
  │ edit / scan / fire / merge
  ↓
open-burning
  │ extinguish / verify
  ↓
open-consistent
  │ commit
  ↓
open-clean
  │ close
  ↓
closed
```

## 5.5 stale resolution

resolutionは、fireを解消した時点のsource atom、target atom、trace link、policyに対する判断である。

以下のいずれかが変わった場合、resolutionはstaleになる。

```text
source atom hash
target atom hash
trace link hash
policy hash
```

stale resolutionが1つでもある場合、commit不可である。

## 5.6 commit certificate

commit時には、consistent certificateを生成する。

certificateは以下を含む。

```text
入力root
open fire数
failed check数
missing link数
stale resolution数
duplicate atom id数
verification結果
policy hash
```

certificateは、そのcommitがそのpolicyのもとで整合条件を満たしたことを示す。



---

# 06. Fire / Extinguish設計

## 6.1 fireの意味

fireは、ある変更または発見された不整合により、関連成果物へ確認責任が発生したことを表す。

fireは不整合そのものではない。確認責任である。

## 6.2 fireの発生契機

```text
仕様書の変更
設計書の変更
コードの変更
テストの変更
Trace Linkの変更
手動fire
AIによるfire提案
mergeによる衝突または意味的衝突候補
policy違反
```

## 6.3 fire severity

MVPでは `required` のみを扱う。

将来、以下を追加できる。

```text
required
warning
info
```

ただし、commit blockerになるのはrequired fireである。

## 6.4 fire生成

変更Atomを \( s \) とする。

変数の定義：

- \( s \)：変更されたArtifact Atom
- \( v \)：確認対象候補Atom
- \( G=(V,E) \)：Trace Graph
- \( V \)：Atom集合
- \( E \)：Trace Link集合
- \( P \)：fire propagation policy
- \( I(s) \)：変更Atom \( s \) により確認対象になるAtom集合

単純化した影響集合は以下である。

\[
I(s) = \{ v \in V \mid reachable_P(s, v, G) \}
\]

MVPでは、変更Atomにincoming/outgoingで直接接続されるAtomへrequired fireを立てる。

## 6.5 fire重複排除

同じscanで同じfireが増殖してはいけない。

fire keyは以下から作る。

```text
source_atom_id
target_atom_id
reason
trace_path_hash
base_commit
```

同じfire keyのopen fireが存在する場合は新規作成しない。

## 6.6 obsolete fire

auto fireは、変更がrevertされた場合にobsoleteになる。

```text
current atom hash == base atom hash
  -> 変更なし
  -> その変更由来のauto fireはobsolete
```

manual fireは自動obsoleteにしない。

## 6.7 extinguish種別

```text
changed
  対象を変更して解消した

verified
  検証により問題ないと確認した

no-change-required
  確認した結果、変更不要と判断した

invalid-fire
  fire自体が不適切だった
```

## 6.8 no-change-required

`no-change-required` は非常に重要である。

AI時代には、「変更しなかった理由」も履歴に残す必要がある。

例：

```bash
codefire extinguish FIRE-002 \
  --resolution no-change-required \
  --rationale "外部仕様は変化せず、内部処理の整理のみであるため"
```

## 6.9 resolution basis

resolutionにはbasisを保存する。

```text
source atom hash at resolution
target atom hash at resolution
trace link hash at resolution
policy hash at resolution
```

これにより、後から前提が変わった場合にstale判定できる。



---

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



---

# 08. Object Store設計

## 8.1 基本方針

Code FireはGitを使わず、独自のcontent-addressed object storeを持つ。

objectはimmutableである。一度作成したobjectは更新しない。

## 8.2 Object ID

全objectは以下でIDを計算する。

\[
object\_id = sha256(type\_tag \parallel 0x00 \parallel canonical\_payload)
\]

変数の定義：

- \( object\_id \)：objectのID
- \( sha256 \)：SHA-256 hash関数
- \( type\_tag \)：`blob`, `commit`, `trace_graph` などのobject種別
- \( \parallel \)：バイト列の連結
- \( canonical\_payload \)：順序や空白差分を排除した正規化payload

## 8.3 主要object

```text
BlobObject
ContentManifestObject
ArtifactObject
AtomIndexObject
TraceGraphObject
FireObject
ResolutionObject
VerificationObject
CommitObject
BranchObject
```

## 8.4 ContentManifestObject

commit時点の全ファイル一覧を表す。

```json
{
  "type": "content_manifest",
  "version": 1,
  "entries": [
    {
      "path": "docs/spec/auth.md",
      "kind": "file",
      "mode": "100644",
      "blob": "CF-BLOB-222bbb"
    }
  ]
}
```

`.codefire-open` はmanifest対象外である。

## 8.5 AtomIndexObject

commit時点のArtifact Atom索引を表す。

```json
{
  "type": "atom_index",
  "version": 1,
  "atoms": [
    {
      "atom_id": "REQ-AUTH-001",
      "kind": "requirement",
      "artifact_path": "docs/spec/auth.md",
      "selector": {
        "type": "markdown_heading",
        "value": "REQ-AUTH-001"
      },
      "content_hash": "sha256:aaa111"
    }
  ]
}
```

## 8.6 CommitObject

sealed commitを表す。

```json
{
  "type": "commit",
  "version": 1,
  "commit_id": "CF-COMMIT-8f3a91d0c2",
  "parents": ["CF-COMMIT-21ab77aa00"],
  "message": "Change session expiration policy",
  "roots": {
    "content_manifest": "CF-MANIFEST-aaa111",
    "atom_index": "CF-ATOMINDEX-ccc333",
    "trace_graph": "CF-TRACE-ddd444",
    "verification": "CF-VERIFY-fff666",
    "policy": "CF-POLICY-ggg777"
  },
  "certificate": {
    "result": "consistent",
    "open_required_fires": 0,
    "failed_checks": 0,
    "missing_required_links": 0,
    "stale_resolutions": 0,
    "duplicate_atom_ids": 0
  }
}
```

## 8.7 object保存タイミング

未commitファイル内容はobject storeへ保存しない。

```text
scan時:
  Atom hashやfire metadataはactive stateに保存してよい
  作業中ファイル内容はblob化しない

commit時:
  初めて作業中ファイルをBlobObjectとして保存する
  manifest, atom index, trace graph, verification, commitを保存する
```

これはcheckpointなしの思想を内部実装でも守るための重要ルールである。



---

# 09. Index / Trace / Policy設計

## 9.1 versioned project files

open directory内には以下を置く。

```text
codefire.yaml
codefire.links.yaml
codefire.policy.yaml
```

これらは通常の成果物としてcommit対象である。

## 9.2 codefire.yaml

プロジェクト構成を定義する。

```yaml
version: 1
artifacts:
  requirements:
    - path: docs/spec/**/*.md
      kind: requirement_document
  designs:
    - path: docs/design/**/*.md
      kind: design_document
  code:
    - path: src/**/*.py
      kind: python_code
  tests:
    - path: tests/**/*.py
      kind: pytest_test
```

## 9.3 Atom ID規約

推奨prefix：

```text
REQ-   requirement
DES-   design
ADR-   architecture decision record
API-   API operation
DB-    database schema element
CODE-  code symbol
TEST-  test case
OPS-   operation/runbook
```

例：

```text
REQ-AUTH-001
DES-AUTH-001
CODE-SessionPolicy
TEST-session-expiration
```

## 9.4 Markdown Atom抽出

Markdownでは、見出し先頭のIDをAtom IDとする。

```markdown
## REQ-AUTH-001: セッション有効期限

ユーザのセッションは最終操作から30分で失効する。
```

同階層以上の次の見出しまでをAtom本文とし、content hashを計算する。

## 9.5 Python Atom抽出

MVPでは、明示IDコメントを推奨する。

```python
# cf-atom: CODE-SessionPolicy
class SessionPolicy:
    def expires_after_minutes(self) -> int:
        return 30
```

明示IDがない場合は派生IDを使う。

```text
CODE:src/auth.py::class:SessionPolicy
```

## 9.6 pytest Atom抽出

```python
# cf-atom: TEST-session-expiration
def test_session_expiration():
    assert SessionPolicy().expires_after_minutes() == 30
```

## 9.7 Trace Graph

Trace Linkは `codefire.links.yaml` に定義する。

```yaml
version: 1
links:
  - from: REQ-AUTH-001
    to: DES-AUTH-001
    type: refined_by
  - from: DES-AUTH-001
    to: CODE-SessionPolicy
    type: implemented_by
  - from: REQ-AUTH-001
    to: TEST-session-expiration
    type: verified_by
```

Trace Graphを以下の有向グラフとして扱う。

\[
G = (V, E)
\]

変数の定義：

- \( G \)：Trace Graph
- \( V \)：Artifact Atom集合
- \( E \)：Trace Link集合

## 9.8 required link policy

policyでは、Atom種別ごとに必須リンクを定義する。

```yaml
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
```

必須linkが欠落している場合、commit不可である。

## 9.9 propagation policy

MVPでは単純に、変更Atomに直接接続されたAtomへrequired fireを立てる。

将来的には、変更種別ごとのpolicyを導入する。

```yaml
propagation:
  code_behavior_change:
    implemented_by:
      inverse: required
    verified_by:
      related: required
  requirement_change:
    refined_by:
      forward: required
    verified_by:
      forward: required
```



---

# 10. Merge / Remote / Server設計

## 10.1 mergeの意味

Code Fireのmergeは、source branchのsealed headをtarget branchのopen directoryへ統合し、必要なfireを立てる操作である。

merge実行時点では正式履歴にcommitを作らない。

```text
merge
  ↓
target open directoryがopen-burningになる
  ↓
fireを解消する
  ↓
verifyする
  ↓
commitする
```

## 10.2 merge前提

```text
source branch:
  sealed headを持つ
  open-burningではない

target branch:
  openされている
  open-cleanである
```

未完了変更があるtargetにはmergeできない。

## 10.3 merge inputs

```text
B = common ancestor commit
S = source head commit
T = target head commit
W = target open directory
```

変数の定義：

- \( B \)：sourceとtargetの共通祖先commit
- \( S \)：source branchのhead commit
- \( T \)：target branchのhead commit
- \( W \)：target branchのopen directory

## 10.4 merge処理

```text
1. B, S, TのContentManifestを読む
2. B, S, TのAtomIndexを読む
3. file-level 3-way mergeを行う
4. Atom-level変更を計算する
5. TraceGraph変更をmergeする
6. conflictがあればconflict markerを出す
7. semantic conflict候補を検出する
8. merge fireを作る
9. targetをopen-burningにする
```

## 10.5 server方針

serverは不整合状態を保存しない。

保存するもの：

```text
sealed commit
sealed branch
merge request
review record
policy
permission
```

保存しないもの：

```text
burning workspace
checkpoint
stash
unverified WIP state
```

## 10.6 remote clone

```bash
codefire clone cf://server/org/app/main main
```

remote branchをlocal branchとして取得する。remote tracking branchは作らない。

最新版が必要な場合は、再度cloneする。

```bash
codefire clone cf://server/org/app/main main-latest
```

その後、必要ならmergeする。

## 10.7 upload

```bash
codefire upload feature-login cf://server/alice/app/feature-login
```

uploadできるのはsealed branchのみである。

server branchの現在headを \( R \)、uploadしようとするcommitを \( U \) とする。

upload可能条件：

\[
R \in ancestors(U)
\]

変数の定義：

- \( R \)：server上の現在branch head
- \( U \)：clientがuploadしようとしているcommit
- \( ancestors(U) \)：commit \( U \) の祖先commit集合

つまり、fast-forward可能な場合だけuploadできる。

## 10.8 merge request

```bash
codefire request-merge \
  cf://server/alice/app/feature-login \
  cf://server/org/app/main
```

merge requestは、source sealed commitをtarget branchへ統合してほしいという申請である。

MR作成時target commitを \( T_0 \)、現在target headを \( T_1 \) とする。

stale条件：

\[
T_0 \ne T_1
\]

targetが進んだら、MRはstaleである。

serverは分岐したcommitを勝手にsemantic mergeしない。source側で最新targetをcloneし、local mergeし、整合commitを作り直す。



---

# 11. 内部アーキテクチャ設計

## 11.1 レイヤ構成

```text
CLI Layer
  ユーザコマンドを受ける

Command Service Layer
  open, close, clone, scan, fire, extinguish, verify, commit, mergeを実行する

Repository Store
  sealed object, branch, policy, commit graphを保存する

Open Directory Manager
  branchを実ディレクトリへ展開し、close/discardする

Indexer
  ファイルからArtifact / Atomを抽出する

Trace Graph Engine
  Atom間の関係グラフを管理する

Fire Engine
  変更に基づいてfireを生成・伝播・重複排除する

Policy Engine
  commit可能性、link欠落、fire未解消、stale resolutionを判定する

Verifier
  テスト、静的解析、生成物鮮度、trace completenessを検証する

Commit Sealer
  consistent状態をsealed commitとしてobject storeへ保存する

Merge Engine
  sealed branch同士を統合し、target open directoryをburningにする

Remote Client / Server
  clone, upload, merge requestを扱う
```

## 11.2 Rust module構成案

```text
crates/
  codefire-cli/
    src/main.rs

  cf-core/
    src/types.rs
    src/errors.rs

  cf-store/
    src/object_store.rs
    src/commit_store.rs
    src/branch_store.rs
    src/hash.rs
    src/canonical.rs

  cf-workspace/
    src/open.rs
    src/close.rs
    src/materialize.rs
    src/discard.rs
    src/identity.rs

  cf-index/
    src/indexer.rs
    src/markdown.rs
    src/python.rs
    src/pytest.rs
    src/atom_hash.rs

  cf-trace/
    src/graph.rs
    src/links.rs
    src/policy.rs

  cf-fire/
    src/fire_engine.rs
    src/propagation.rs
    src/resolution.rs
    src/stale.rs

  cf-policy/
    src/commit_policy.rs
    src/link_policy.rs
    src/extinguish_policy.rs

  cf-verify/
    src/verifier.rs
    src/external_command.rs
    src/internal_checks.rs

  cf-commit/
    src/sealer.rs
    src/certificate.rs

  cf-merge/
    src/merge_engine.rs
    src/file_merge.rs
    src/semantic_fire.rs

  cf-remote/
    src/client.rs
    src/protocol.rs
    src/upload.rs
    src/clone.rs
    src/merge_request.rs
```

## 11.3 scan処理

```text
1. open context確認
2. codefire.yaml読み込み
3. indexer実行
4. current AtomIndex生成
5. base AtomIndexと比較
6. changed_atoms算出
7. TraceGraph読み込み
8. FireEngine実行
9. active fires更新
10. obsolete fire整理
11. status更新
```

## 11.4 commit処理

```text
1. repo lock
2. branch lock
3. open identity確認
4. copied directoryでないことを確認
5. scan
6. verify
7. committable判定
8. blob保存
9. manifest保存
10. atom index保存
11. trace graph保存
12. fire delta保存
13. verification保存
14. commit object作成
15. branch head更新
16. active state reset
17. open registry更新
```

## 11.5 lock設計

```text
repo.lock
  repository全体の構造変更

branch-<name>.lock
  branch head/open state変更

object-store.lock
  object書き込み

active-<open_instance_id>.lock
  active state変更
```

branch head更新はatomic renameで行う。

## 11.6 recovery

クラッシュ対応として `codefire doctor` を用意する。

```text
orphan tmp削除
object hash再検証
branch head参照検証
opened registryと.codefire-openの整合確認
active state破損確認
cache再構築
```

cacheは正式データではないため、壊れたら再構築する。

## 11.7 重要な実装判断

未commitファイル内容をrepositoryに保存しない。

scan時に作業中ファイルをblob化すると、実質checkpointになる。したがって、BlobObject保存はcommit sealing時だけ行う。

active stateにはfire/resolution/verification metadataを保存してよいが、それだけでは作業中ファイルを復元できないようにする。



---

# 12. MVP実装計画

## 12.1 MVPの目的

MVPの目的は、次の中核価値を証明することである。

> コードや仕様を変更すると関連成果物へfireが立ち、fireが残っている状態ではcommitできず、すべてextinguishしてverifyに通った場合だけsealed commitを作れる。

## 12.2 MVP対象

```text
Local repository
Local branch
open / close / clone
Markdown仕様書
Markdown設計書
Pythonコード
pytestテスト
Trace Link YAML
Fire ledger
Extinguish
Verify
Commit
Merge basic
```

## 12.3 MVP対象外

```text
Remote server
Merge request
AI連携
多言語対応
OpenAPI
DB schema
高度なsemantic merge
権限管理
Review workflow
warning fire
GUI
```

## 12.4 Phase 1: Object store

作るもの：

```text
object_id計算
blob store
manifest object
commit object
branch object
init
```

成功条件：

```text
repositoryをinitできる
空のmain branchを作れる
manifestを保存できる
commit objectを保存できる
branch headを更新できる
```

## 12.5 Phase 2: open / close / clone

作るもの：

```text
open
close
close --discard
clone
open registry
.codefire-open
copied directory検出
```

成功条件：

```text
mainをopenできる
mainからfeatureをcloneできる
featureをopenできる
同一branch多重openを拒否できる
closeで実ディレクトリを消せる
```

## 12.6 Phase 3: Indexer

作るもの：

```text
codefire.yaml parser
Markdown Atom抽出
Python Atom抽出
pytest Atom抽出
AtomIndex生成
duplicate atom id検出
```

成功条件：

```text
REQ/DES/CODE/TESTを抽出できる
Atom hashを計算できる
変更Atomを検出できる
```

## 12.7 Phase 4: Trace Graph

作るもの：

```text
codefire.links.yaml parser
TraceGraphObject
link参照検証
required link policy
```

成功条件：

```text
REQ -> DES -> CODE
REQ -> TEST
の関係を読める
link欠落でcommit拒否できる
```

## 12.8 Phase 5: Fire Engine

作るもの：

```text
scan
changed atom detection
fire propagation
active fires
manual fire
obsolete fire
status表示
```

成功条件：

```text
コード変更で設計・仕様・テストにfireが立つ
fireが残るとcommit不可
revertするとauto fireがobsoleteになる
manual fireは残る
```

## 12.9 Phase 6: Extinguish / stale

作るもの：

```text
extinguish
resolution basis
no-change-required rationale必須
stale resolution判定
```

成功条件：

```text
fireをextinguishできる
理由なしno-change-requiredは拒否
extinguish後に対象Atomを変えるとstaleになる
staleがあるとcommit不可
```

## 12.10 Phase 7: Verify / Commit

作るもの：

```text
verify
external command runner
commit sealer
certificate
branch head更新
active state reset
```

成功条件：

```text
open fireなし
staleなし
link欠落なし
pytest成功
の場合だけcommitできる
commit後branch headが進む
open stateがopen-cleanになる
```

## 12.11 Phase 8: Merge

作るもの：

```text
common ancestor探索
file-level 3-way merge
atom-level diff
merge fire
merge commit parents
```

成功条件：

```text
featureをmainにmergeできる
merge後mainがopen-burningになる
fire解消後にmerge commitできる
merge commit parentsが2つになる
```

## 12.12 MVPデモシナリオ

```text
1. codefire init
2. mainをopen
3. サンプル仕様・設計・コード・テストを書く
4. linkを定義する
5. verifyしてcommit
6. mainからfeatureをclone
7. featureをopen
8. コードを30分から15分へ変更
9. scanでREQ/DES/TESTにfireが立つ
10. commitが拒否される
11. 仕様・設計・テストを更新する
12. fireをextinguishする
13. verifyする
14. commitできる
```



---

# 13. リスクと非目標

## 13.1 fire fatigue

fireが多すぎると、ユーザはfireを無視するようになる。

対策：

```text
Atom単位で管理する
ファイル単位fireを避ける
変更種別を導入する
policyを調整可能にする
同種fireをまとめる
AIにfire解消案を出させる
```

## 13.2 commitが重くなりすぎる

Code Fireではcommit条件が厳しいため、大きな変更ではcommitまで時間がかかる。

対策：

```text
整合可能な小さい変更単位に分割する
AIに「次の整合commit」を作らせる
仕様・設計・コードを段階的に移行する
```

## 13.3 OSコピーによる疑似checkpoint

ユーザはopen directoryをOSレベルでコピーできる。

対策：

```text
Code Fireとしては管理しない
コピー先ではCode Fire操作を拒否する
open_instance_idで正規open directoryを識別する
```

## 13.4 Git文化との摩擦

Git利用者は以下ができないことに違和感を持つ可能性がある。

```text
checkoutできない
stashできない
WIP commitできない
partial commitできない
pullできない
fetchできない
rebaseできない
force pushできない
```

しかし、これらを入れるとCode Fireの思想が崩れる。Code FireはGit互換の便利ツールではなく、開発規律を変えるツールである。

## 13.5 非目標

v0.2時点の非目標：

```text
Git互換性
Git repositoryとの共存
不整合状態の保存
checkpoint
stash
WIP commit
partial commit
staging area
pull/fetch/rebase
force upload
GUI
全言語対応
完全自動semantic consistency判定
AIによる最終承認
```

## 13.6 将来課題

```text
AI連携
server-side verification certificate
commit signature
reviewer approval object
OpenAPI/DB schema support
semantic diff
large repository performance
policy version migration
```

