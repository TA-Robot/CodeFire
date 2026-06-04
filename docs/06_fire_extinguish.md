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

staleになったresolutionは、同じfireを現在の前提で再確認したうえで `--refresh` により更新できる。

```bash
codefire extinguish FIRE-002 \
  --resolution changed \
  --evidence "stale resolutionを再確認した" \
  --refresh
```
