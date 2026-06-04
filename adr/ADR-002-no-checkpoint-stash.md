# ADR-002: checkpoint / stashを提供しない

## Status

Accepted

## Context

checkpointやstashは、不整合状態を保存する機能になり得る。これを提供すると、Code Fireの「整合済み状態だけを正式履歴・共有対象にする」という思想が弱くなる。

## Decision

Code Fireはcheckpoint、stash、WIP commitを提供しない。

## Consequences

良い点：

- 不整合状態がCode Fire管理下に保存されない。
- 開発者に整合可能な小さい変更単位を促せる。
- branch履歴とserverが常にcleanになる。

悪い点：

- 長い作業中に保存したい場合、ユーザはOSコピーなど外部手段に頼る必要がある。
- Git文化の利用者には厳しく感じられる。

## Note

OSコピーされたopen directoryではCode Fire操作を拒否する。これはCode Fire管理外のバックアップであり、checkpointではない。

