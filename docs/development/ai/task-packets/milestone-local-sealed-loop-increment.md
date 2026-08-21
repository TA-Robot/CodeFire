# AI Task Packet

Status: ready

Packet ID: MS-local-sealed-loop-increment

Schema version: human-v1

Base revision: the commit that closed `MS-2026-08-control-restore` on 2026-08-21

Risk score: 2

Reasoning tier: high

Parent packet: `milestone-local-sealed-loop.md`. This is node 2 of that graph — one of two parallel ready slices.

Nothing here has run. No candidate file has been chosen. Do not read any statement below as an observation.

## Objective

Select, **by reading only**, the one small real change that node 3 will take through the local sealed loop, and record it in this file: candidate file, minimal edit, reason, and the focused test that covers it.

This node answers *what the loop carries*. It does not answer what runs the loop — that is node 1, running concurrently.

## Context

- Required context pack(s): pack-0-universal-orientation, pack-6-dogfood-algorithm-project
- Additional files: `docs/development/milestones.md`, `subprojects/algorithm-evolution-agent-lab/AGENTS.md`, `subprojects/algorithm-evolution-agent-lab/README.md`
- Target project: `subprojects/algorithm-evolution-agent-lab`, the canonical dogfood project.

## Plan

1. Read the dogfood project and find a change worth making that touches exactly one file.
2. Record the candidate file path and the minimal edit, at one-line scale, precisely enough that node 3 can apply it without re-deciding.
3. Record why the change is worth making. "Something to commit" is not a reason; the change must stand on its own merit.
4. Record which focused test covers that file, and how to run it inside the subproject.

Selection rules:

- exactly one file;
- smallest real change that has a reason to exist;
- not a replay of any parked root (CFB-133, CFB-136, CFR-168..173);
- a prior record that the project is `open-clean` is not permission to skip the edit — the increment is a new small change either way;
- pick a file whose focused test is genuinely focused. If the only covering test is the full suite, say so and pick differently.

## Scope

Allowed files:

- `docs/development/ai/task-packets/milestone-local-sealed-loop-increment.md` — this file, and nothing else

Disallowed files:

- `subprojects/**` — read freely, write nothing. The edit itself is node 3's work
- `crates/**`, `codefire`, `tests/**`, `adr/**`, `install.sh`
- CodeFire project state
- `docs/development/ai/task-packets/milestone-local-sealed-loop-runner.md` — node 1 owns it
- `docs/development/milestones.md` — the orchestrator owns the board
- the classified dirty tree in `/workspace/project`

Disallowed actions:

- running any `codefire` command, including `status`. This node records **no** CodeFire state. The pre-loop baseline belongs to node 3, which will have the node 1 runner; capturing it here would make this node secretly depend on node 1
- applying the edit;
- selecting more than one candidate as "the" increment. Alternates may be listed as fallbacks, clearly marked.

## Non-Goals

- Editing product or dogfood code.
- Running the loop, the focused test, or any CodeFire command.
- Choosing or probing the runner.
- Expanding scope to a second file "while we are here".
- Starting any parked root.

## Invariants

- no file outside this packet is modified;
- no CodeFire command is invoked and no CodeFire-managed repository is read through the CLI or mutated;
- the selected change touches exactly one file and does not alter sealed-commit, object-identity, or branch-head semantics;
- the selected change does not require a product refactor, a new module boundary, or a new dependency to be worth making;
- the recorded edit is specific enough that node 3 applies it without re-deciding scope.

## Validation

This node changes one planning file, so it runs only:

```bash
python3 docs/development/tools/check_project_docs.py
git diff --check
```

The M1 exit-gate list stays on the M1 card and is node 3's to run. Do not run it here and do not widen.

## Handoff

- candidate file, exactly one;
- the minimal edit, specific enough to apply without re-deciding;
- why the change is worth making;
- the focused test and how to run it inside the subproject;
- fallback candidates, clearly marked as alternates;
- why this is not a replay of a parked root;
- risks node 3 inherits;
- next allowed action.

## Result

Not started. Empty by design — this section is filled by the node 2 agent.
