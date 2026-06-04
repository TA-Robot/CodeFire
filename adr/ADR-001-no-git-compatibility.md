# ADR-001: Git互換を目指さない

## Status

Accepted

## Context

Code Fireは、仕様・設計・コード・テストの整合済み状態だけを正式履歴に残すことを目的とする。Gitの概念を踏襲すると、checkout、stash、partial commit、rebase、pull/fetchなどの概念が流入し、開発フローが複雑化する。

## Decision

Code FireはGit互換を目指さない。Git repositoryを内部実装としても使わない。

## Consequences

良い点：

- 履歴上のcommitをすべて整合済みにできる。
- Gitの複雑な概念を持ち込まない。
- プロダクト思想が明確になる。

悪い点：

- Git資産との直接互換がない。
- 既存開発者には学習コストがある。
- GitHub的なエコシステムをそのまま使えない。

