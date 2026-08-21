# AI Task Packet

Status: ready

Packet ID: MS-remote-happy-path-run

Schema version: human-v1

Risk score: 6

Reasoning tier: high

## Objective

Execute the already-implemented isolated file-backed collaboration path with zero product changes: clone two sealed ancestor heads inside a copied repository, upload them, create and approve a merge request, apply it, and prove the terminal remote head/state from CodeFire JSON.

This is node 4 of active M1. Node 3 admitted the run after both classifiers parked CFR-168 and CFB-136.

## Exact inputs

- runner/public name: `/workspace/worktrees/ms-agent-local-contract-seam-20260821/target/release/codefire`, SHA-256 `207fae943c09c2f739611e37050cbc83f34d3b53318fcefaf9d7304ba03d708b`;
- canonical copy source: `/workspace/project/subprojects/.codefire` (read-only);
- local target alias: `base`, cloned from `feature/docs-doubling-plan` at `CF-COMMIT-d068d2c3995f6da0a0959b98`;
- local source alias: `topic`, cloned from `feature/docs-full-doubling` at `CF-COMMIT-126555fdb84822a0c09c9e6a`;
- isolated root: create with `mktemp -d`; fixture `$ROOT/repo`; remote `$ROOT/remote`; URLs `cf://$ROOT/remote/org/app/{base,topic}`;
- no `server_policy.json`, actor, signing, HTTP, or canonical branch `main` operation.

The two heads are strict ancestor/descendant with a two-commit apply delta. Use no new local commit.

## Scope

- one new isolated temporary root and its file-backed remote;
- ignored evidence output under this agent worktree if helpful;
- this packet's Result.

Canonical `/workspace/project` and CodeFire state are read-only preservation subjects. Every product/source/doc path except this packet is forbidden.

## Procedure

1. Rehash the runner and capture every canonical preservation value from the probe packet §5. Stop on any mismatch.
2. Create `$ROOT/repo` and `$ROOT/remote`; `cp -a` only canonical `subprojects/.codefire` to `$ROOT/repo/.codefire`. This is the sole non-CodeFire fixture materialization step.
3. Run `doctor "$FIX" --quick --json` and `branch list --path "$FIX" --json`. Stop on any open-target/capability problem. Never name branch `main`, never run `close`/`discard`, and never pass a canonical path to a mutator.
4. With cwd `$FIX`, run public JSON commands in this exact order:
   - dry-run then live `clone feature/docs-doubling-plan base`;
   - live `clone feature/docs-full-doubling topic`;
   - dry-run then live/idempotent-key `upload base` to remote base;
   - live/idempotent-key `upload topic` to remote topic;
   - dry-run then live/idempotent-key `request-merge topic -> base`;
   - `request-list ... --json`, recording the observed MR-ID and status field names; stop if JSON contains no usable MR ID;
   - dry-run then live/idempotent-key `request-review ... --decision approve --reviewer m1-reviewer`;
   - dry-run then live/idempotent-key `request-apply`;
   - read-only `list`, `show .../base`, `request-list`, and `doctor`;
   - replay the exact same `request-apply` idempotency key and require `replayed: true` with no state change.
5. After **every** command, recheck all canonical preservation values from probe §5. Stop immediately on drift.
6. Record every command argv, exit code, envelope, plan/postcondition, MR ID/status, source/target/applied heads, object/branch/MR counts, idempotent replay, durations, and isolated-root path.
7. Set this packet `completed` or `blocked`, run validation below, and commit only this packet.

The live sequence is `upload -> request-merge -> request-review(approve) -> request-apply`; the earlier three-command board shorthand was corrected at fan-in.

## Validation

Only the command-driven isolated run, preservation checks, and:

```bash
python3 docs/development/tools/check_project_docs.py
git diff --check
```

No build, product test, installer, HTTP, scale test, Retrospect, or canonical dogfood mutation.

## Non-Goals

- implementing or completing CFR-168/CFB-136;
- documenting discovered JSON field names outside this evidence packet;
- using human output/`awk` for MR identity;
- starting adoption or later slots;
- deleting the evidence fixture before independent confirmation.

## Handoff

- isolated root and runner identity;
- full JSON-envelope sequence and durations;
- observed MR-ID/status/applied-head fields;
- final remote source/target/MR/doctor state and replay proof;
- canonical preservation outcome;
- any `block-m1` and first failing command;
- next legal action: node 5 read-only evidence confirmation, preferably same session if no independence boundary requires otherwise.

## Result

Not run.
