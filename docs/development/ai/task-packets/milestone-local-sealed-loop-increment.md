# AI Task Packet

Status: completed

Packet ID: MS-local-sealed-loop-increment

Schema version: human-v1

Base revision: the commit that closed `MS-2026-08-control-restore` on 2026-08-21

Risk score: 2

Reasoning tier: high

Parent packet: `milestone-local-sealed-loop.md`. This is node 2 of that graph — one of two parallel ready slices.

Node 2 selection is recorded in Result. No CodeFire command was run, no dogfood test was run, and the edit was not applied.

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

- candidate file: `subprojects/algorithm-evolution-agent-lab/src/evoagent/external_validity.py` (exactly one)
- minimal edit: replace the `improved_domains` assignment as specified in Result; do not re-decide
- why: unmapped benchmarks currently mint a synthetic `"unknown"` domain and can make `transferable=True` without a second real domain (REQ-SOTA-007)
- focused test: from `subprojects/algorithm-evolution-agent-lab`, `PYTHONPATH=src python3 -m unittest tests.test_external_validity -v`
- fallbacks: listed in Result, marked as alternates; do not apply them
- not a parked root: dogfood evaluator bug, not CFB-133 / CFB-136 / CFR-168..173
- risks and next action: see Result

## Result

Selection completed by reading only. No `codefire` invocation, no product or dogfood test run, no CodeFire-managed state inspected or mutated, and the candidate edit was not applied.

### Candidate (exactly one)

`subprojects/algorithm-evolution-agent-lab/src/evoagent/external_validity.py`

This is `CODE-ExternalValidityEvaluator`. Node 3 edits this file only.

### Exact minimal edit

In `ExternalValidityEvaluator.evaluate`, replace the current `improved_domains` assignment (today line 36):

```python
        improved_domains = tuple(sorted({domain_by_benchmark.get(result.plan.benchmark, "unknown") for result in improvements}))
```

with:

```python
        improved_domains = tuple(sorted({domain_by_benchmark[result.plan.benchmark] for result in improvements if result.plan.benchmark in domain_by_benchmark}))
```

That is the whole increment. Do not add a test, do not rename symbols, do not touch `mean_delta` / ingestion / fixtures, and do not change `transferable=len(improved_domains) >= min_domains`.

### Why this independently merits the change

REQ-SOTA-007 (`docs/spec/sota_challenge_protocol.md`) requires evaluating whether gains transfer beyond one narrow benchmark before treating a candidate as broadly important. `ExternalValidityEvaluator` is that gate: `transferable` is `len(improved_domains) >= min_domains` (default 2).

The current line does `domain_by_benchmark.get(benchmark, "unknown")`. An improvement on one mapped domain plus an improvement on any unmapped benchmark therefore yields `improved_domains == ("tabular", "unknown")` (or similar) and `transferable=True`. `"unknown"` is not a domain; it is missing catalog data. That is a false transfer signal on the claim-discipline path this lab exists to keep conservative.

Skipping unmapped benchmarks keeps transferability tied to named domains only. Two unmapped improvements collapse to a single `"unknown"` today and already fail `min_domains=2`; the false-positive case is mixed mapped + unmapped, and the one-line change closes it. This is not "something to commit": it is a real over-claim in the SOTA governance evaluator.

Call sites: only this module, `tests/test_external_validity.py`, and re-exports. The tests assert mapped names (`("sequence", "tabular")`) and single-domain rejection; they do not depend on the `"unknown"` sentinel.

### Focused test

Inside `subprojects/algorithm-evolution-agent-lab`:

```bash
PYTHONPATH=src python3 -m unittest tests.test_external_validity -v
```

This is the module's own file (`TEST-external-validity-requires-transfer-across-domains`): two unit tests, no runner subprocess, not the 281-test suite. Node 3 must not add a third test; M1 forbids test expansion the loop does not require. The existing tests remain the focused gate and are expected to stay green because they only cover mapped domains.

### Fallback candidates (alternates only; do not apply)

1. Alternate A — `subprojects/algorithm-evolution-agent-lab/src/evoagent/ingestion.py`: change `confidence=float(payload.get("confidence", 0.5))` to `confidence=float(payload.get("confidence", 0.0))`. Missing confidence currently becomes 0.5, which is the cutoff used by `ResearchProgressMeter` (`confidence >= 0.5`) and `RunAnalyst` ("local improvement candidate"). Focused test: `PYTHONPATH=src python3 -m unittest tests.test_ingestion.ResultIngestorTests.test_result_ingestor_parses_metrics_artifact -v`. Not chosen because it is a silent-default policy fix; the selected edit is an actual false-positive on a named SOTA gate.
2. Alternate B — `subprojects/algorithm-evolution-agent-lab/src/evoagent/fixtures.py`: in `accuracy()`, change `zip(predictions, labels)` to `zip(predictions, labels, strict=True)` so a length mismatch cannot silently mis-score the only local benchmark (`requires-python = ">=3.10"`). Focused test: `PYTHONPATH=src python3 -m unittest tests.test_fixtures.LocalBenchmarkFixtureTests.test_local_benchmark_fixture_produces_artifacts -v`. Not chosen because current `evaluate()` always passes equal-length lists; it is a latent footgun, not a present false claim.

### Why this is not a replay of a parked root

Parked roots CFB-133, CFB-136, CFR-168, CFR-169, CFR-170, CFR-171, CFR-172, and CFR-173 are CodeFire product issues (migrate next-action JSON, validation-cache reuse, graph/HTTP budget, module/test ownership, typed executable outcomes, claim/proof descriptor, adoption, falsification). This increment is a one-line bugfix inside the canonical dogfood project's `ExternalValidityEvaluator`. It does not touch `crates/**`, `codefire`, installer, JSON envelopes, or any CFB/CFR write-up.

### Risks node 3 inherits

- The focused tests do not currently include an unmapped-benchmark case, so they will not fail if the line is left unchanged and will not newly fail after the fix. Node 3 still applies the recorded edit; it does not invent a second file or a new test to "prove" the unmapped path.
- If a later caller wanted the string `"unknown"` in `improved_domains`, that sentinel disappears. Present tests and README do not.
- Do not capture pre-loop `status --json` here; that remains node 3 work using the node 1 runner.
- Stop on the first loop command that does not behave as documented. Do not work around a loop failure by widening validation or by starting a parked root.

### Next allowed action

Fan-in with node 1. After both child packets are completed, the orchestrator writes the node 3 packet. Node 3 captures pre-loop `status --json` with the recorded runner, then `open → edit → scan → extinguish → verify → commit` on this one line, then runs the focused unittest above. Nodes 4–6 stay blocked until that loop evidence exists.
