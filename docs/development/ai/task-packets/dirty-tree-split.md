# AI Task Packet: Dirty Tree Split

Status: ready.

## Objective

Split the current mixed dirty tree into reviewable save/commit packages without discarding any user work.

## Background

The repository currently mixes:

- CodeFire product implementation fixes;
- CodeFire management documentation rebuild;
- CFB/CFR issue closure docs;
- algorithm dogfood implementation;
- algorithm dogfood documentation expansion.

This blocks safe review and slows all future AI work.

## Context Pack

- Required: Pack 0, Pack 5, Pack 6.
- Read:
  - `docs/development/worktree-cleanup-plan.md`
  - `docs/development/project-management.md`
  - `docs/development/README.md`

## Scope

Allowed files:

- `docs/development/worktree-cleanup-plan.md`
- `docs/development/history.md`
- optional temporary classification notes under `docs/development/`

Disallowed actions:

- `git reset --hard`
- `git checkout -- <path>`
- deleting untracked files
- committing without explicit human confirmation

## Non-Goals

- Do not implement product fixes.
- Do not edit algorithm code.
- Do not resolve merge conflicts speculatively.

## Plan

1. Generate a package list from `git status --short`.
2. Assign each file to exactly one package:
   - management docs;
   - CodeFire product fixes;
   - CodeFire issue metadata;
   - algorithm implementation;
   - algorithm docs;
   - unknown/needs human decision.
3. Identify files that are likely to conflict during later commits.
4. Recommend save order.
5. Record validation required per package.

## Validation

```bash
git status --short --branch
git diff --stat
git ls-files --others --exclude-standard
```

Docs consistency:

```bash
rg -n "stale|open issue count|issue status count" README.md docs/development AGENTS.md
```

## Handoff

Return:

- package table;
- unknown files;
- recommended commit order;
- validation gaps;
- human decisions required.
