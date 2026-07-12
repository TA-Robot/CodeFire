# Agent Execution Playbook

This playbook describes how an AI agent should operate inside the algorithm evolution lab.

It focuses on execution discipline: how to choose work, modify the project, verify results, and leave a readable trail for the next agent or human reviewer.

## Operating Model

The agent should treat the repository as a research-management system, not just a Python package.

The normal operating loop is:

1. Read current state.
2. Choose one small objective.
3. Work in a feature branch.
4. Update specs, design, implementation, tests, and docs as appropriate.
5. Run tests and CodeFire verification.
6. Commit through CodeFire.
7. Merge into main.
8. Close the feature branch.
9. Record history.

The loop is intentionally conservative. It keeps research decisions auditable and prevents unreviewed work from accumulating in an open directory.

## Pre-Flight Checklist

Before editing:

- Confirm `codefire status --json` reports `open-clean` on main.
- Read `AGENTS.md`.
- Read the relevant operations guide.
- Identify the expected changed files.
- Decide whether the change is docs-only, code-only, or trace-linked code/design/spec work.
- If changing code, identify the test that will prove behavior.
- If changing docs, identify where the navigation link belongs.

## Choosing Work

Prefer work that moves a documented workflow forward.

Good tasks:

- add a small missing operation guide
- make a workflow repeatable
- add a deterministic renderer
- add a safety gate with positive and negative tests
- add an experiment record template
- connect a requirement to design, code, and tests

Avoid tasks that:

- make broad untested architecture changes
- add generic background material
- introduce a new dependency without a deletion plan
- make unsupported SOTA claims
- mix unrelated research and infrastructure changes

## Branch Discipline

Use one feature branch per coherent change.

Branch naming:

```text
feature/<short-topic>
```

Open directory naming:

```text
/workspace/project/subprojects/algorithm-evolution-agent-lab-feature-<short-topic>
```

Do not open a feature branch inside the main open directory.

## Edit Discipline

Keep changes easy to review.

For code changes:

- Add or update requirement atoms.
- Add or update design atoms.
- Add explicit `# cf-atom:` markers for implementation and tests.
- Add trace links.
- Keep class and function names short.

For documentation changes:

- Update the nearest index or atlas.
- Add history and todo entries when the change is durable project work.
- Keep examples concrete.
- Separate instructions from rationale.
- Use tables for repeated decision criteria.

## Verification Ladder

Use the smallest verification that can catch local mistakes, then the full checks before commit.

Minimum checks:

```bash
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=src python3 -m unittest discover -s tests -v
codefire verify --details --json --path <open-dir>
```

For code changes, run CodeFire scan before commit:

```bash
codefire scan --path <open-dir> --json
```

For docs-only changes, CodeFire status should show `changed_atom_count: 0` and `open_fire_count: 0`. A CodeFire commit is still required to seal the docs change.

## Handling CodeFire Fires

When scan opens fires:

1. Inspect the changed atoms.
2. Confirm trace links are intentional.
3. Generate a batch template.
4. Dry-run the batch.
5. Apply the batch.
6. Verify.

Use rationale text that states what relationship was reviewed. Do not use vague rationales such as "ok" or "fixed".

## Recording Work

Every durable change should update at least one of:

- `docs/development/history.md`
- `docs/development/todo-checklist.md`
- `docs/development/implementation-roadmap.md`
- `docs/development/source-code-atlas.md`
- `docs/operations/README.md`

The history entry should include:

- date
- category
- summary
- changed files or atoms
- verification evidence

## Failure Handling

If tests fail:

- read the first failing test carefully
- fix the behavior or the test expectation
- rerun the focused test if possible
- rerun the full suite before commit

If CodeFire verify fails:

- inspect blocking diagnostics
- resolve open fires or duplicate atoms
- avoid bypassing verify unless the failure is explicitly documented as non-blocking

If branch operations fail:

- do not close or discard main
- do not run parallel CodeFire commands against the same repo
- do not move worktrees manually unless recovering from a known broken state

## Completion Definition

A task is complete only when:

- main is open-clean
- feature branch is closed
- feature open directory is removed
- unittest suite passes
- CodeFire verify passes
- history records the change
- any new guide is discoverable from an index

