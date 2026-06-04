# ADR-007: MVP設定ファイルは限定YAML subsetとして扱う

## Status

Accepted

## Context

CodeFire MVPは依存追加なしで動くことを優先している。

Python標準ライブラリには一般YAML parserがない。外部依存を追加すると、この基盤リポジトリの依存追加ルールにより、導入理由、影響、代替案、削除手順の管理が必要になる。

## Decision

MVPでは `codefire.yaml`, `codefire.links.yaml`, `codefire.policy.yaml` を限定YAML subsetとして扱う。

許容する形:

- `codefire.yaml`: `artifacts.<group>:- path: ...` を読む
- `codefire.links.yaml`: `links: - from/to/type` を読む
- `codefire.policy.yaml`: `verification.required[].command` と `cwd` を読む

このsubset外のYAML表現はMVPでは未定義とする。

## Consequences

- 依存追加なしで動く。
- 設定ファイルの表現力は制限される。
- v0.3以降で一般YAML parserへ移行する場合は、このADRを更新または置き換える。
