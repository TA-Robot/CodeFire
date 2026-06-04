# ADR-005: active stateには作業中ファイル内容を保存しない

## Status

Accepted

## Context

scan時に作業中ファイルをobject storeへ保存すると、実質的なcheckpointになる。これはcheckpointなし方針に反する。

## Decision

active stateにはfire、resolution、scan hash、verification結果などのmetadataのみを保存する。作業中ファイル内容、patch、diff、snapshotは保存しない。

## Consequences

良い点：

- checkpointなし方針を内部実装でも守れる。
- 不整合状態を復元可能な形でrepositoryに保存しない。

悪い点：

- open directoryを失うと作業内容は復元できない。
- ユーザはこまめに整合commitを作る必要がある。

