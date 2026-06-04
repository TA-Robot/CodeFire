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

