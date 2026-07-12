# CodeFire Branch Workflow

This project uses CodeFire as a branch-per-feature dogfood target.

## Normal Flow

1. Confirm main is clean.

```bash
codefire status --json
```

2. Clone a feature branch.

```bash
codefire clone main feature/<short-topic> --json
```

3. Open the feature branch into a sibling directory under the CodeFire repo root.

```bash
codefire open feature/<short-topic> /workspace/project/subprojects/algorithm-evolution-agent-lab-feature-<short-topic> --json
```

4. Make scoped changes in the feature open directory.

5. Run tests.

```bash
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=src python3 -m unittest discover -s tests -v
```

6. Scan, resolve fires, verify, and commit the feature.

```bash
codefire scan --path <feature-open-dir> --json
codefire extinguish --batch-template --output /tmp/<topic>-fires.json --path <feature-open-dir> --json
codefire extinguish --batch /tmp/<topic>-fires.json --path <feature-open-dir> --dry-run --json
codefire extinguish --batch /tmp/<topic>-fires.json --path <feature-open-dir> --json
codefire verify --details --path <feature-open-dir> --json
codefire commit -m "<message>" --path <feature-open-dir> --json
```

7. Merge into main and repeat verification on main.

```bash
codefire merge feature/<short-topic> --into main --dry-run --json
codefire merge feature/<short-topic> --into main --json
```

8. Resolve merge fires on main, run tests, commit main, and close the feature.

```bash
codefire close feature/<short-topic> --dry-run --json
codefire close feature/<short-topic> --json
```

## Path Rules

- Use sibling open directories under `/workspace/project/subprojects`.
- Do not open a feature inside the main open directory.
- Do not open a feature outside the CodeFire repo root unless CodeFire explicitly supports that policy.
- Do not apply `close` or `discard` to `main`.

## Known CodeFire Issues To Avoid

- `CFB-105`: `close main` / `discard main` should not be applied.
- `CFB-108`: repo-root-external open targets are currently accepted by dry-run.
- `CFB-110`: nested open targets under an existing open directory are currently accepted by dry-run.
- `CFB-111`: clone next actions still use a `<path>` placeholder.

Until these are fixed, choose the feature open path manually and keep branch commands sequential for the same repo.

## Clean Completion Criteria

- feature branch is closed
- feature open directory is removed
- main status is `open-clean`
- open fires are 0
- unit tests pass
- the resulting CodeFire commit ids are recorded in development history

