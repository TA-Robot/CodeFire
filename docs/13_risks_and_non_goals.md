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
public-key commit signature
reviewer approval object
advanced OpenAPI/DB schema support
semantic diff
large repository performance
policy version migration
external KMS integration
```
