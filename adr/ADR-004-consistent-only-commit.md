# ADR-004: commitは整合済み状態のみ許可する

## Status

Accepted

## Context

Code Fireの本質は、正式履歴に整合済み状態だけを残すことである。不整合状態をcommitできると、Gitとの差分が薄くなる。

## Decision

未解消required fire、失敗検証、欠落必須link、stale resolution、重複Atom IDが存在する状態ではcommit不可とする。

## Consequences

良い点：

- 任意のcommitを開いても整合済みである。
- AI開発における仕様・設計・コード・テスト乖離を抑制できる。
- commitが意味単位として強くなる。

悪い点：

- commitが重くなる。
- 大きな変更は小さく分割する必要がある。

