# ADR-003: branchはopenにより実ディレクトリとして展開する

## Status

Accepted

## Context

Gitのcheckoutは、同じ作業ディレクトリを別branchへ切り替える。このモデルはcurrent branch、detached head、worktree、stashなどの複雑さにつながる。

## Decision

Code Fireではcheckoutを廃止し、branchはopenにより実ディレクトリとして展開する。

```bash
codefire open main ./main
codefire open feature-login ./feature-login
```

同時に複数branchをopenできる。ただし、同一branchの多重openは禁止する。

## Consequences

良い点：

- current branchが不要になる。
- 複数branchを同時に見られる。
- checkoutに伴う混乱がなくなる。

悪い点：

- ディスク使用量が増える可能性がある。
- Git利用者の感覚とは異なる。

