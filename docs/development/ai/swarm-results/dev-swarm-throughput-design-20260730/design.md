# Dev-Swarm Throughput and Recovery Design

Date: 2026-07-30
Status: design ready; infrastructure implementation not started
Target repository: `/workspace` devcontainer foundation, not CodeFire product

## Outcome

The current coordinator can keep long Claude/Fugu work alive, but it does not
yet model machine-wide resource classes, critical-path priority, durable
progress checkpoints, or a manually resumed second-agent process. This makes
coordinator idle time, heavy-gate collisions, and restart recovery dependent
on operator memory.

The smallest compatible repair is an additive `dev-swarm.v1` extension:

- retain old manifest order and behavior when new fields are absent;
- schedule ready tasks by critical-path length when explicitly enabled;
- enforce one total external-lane semaphore and a nested heavy-work
  semaphore;
- emit durable stale-signal and checkpoint-due events without timing out or
  killing useful work;
- adopt an exact manually resumed second-agent invocation after host/manager
  interruption;
- preserve explicit manager review, integration, and acceptance decisions.

No CodeFire product file needs to change.

## Operating constraints

1. Long-running agents are expected. Elapsed time alone is not failure.
2. A progress warning is advisory; it must never imply cancellation,
   acceptance, integration, or timeout.
3. The recovery envelope remains four external sessions total and at most two
   heavy jobs until measured resource evidence supports a change.
4. Heavy work consumes one total-session permit and one heavy permit.
5. Existing provider limits remain conjunctive with resource limits.
6. Hedge/replacement lineage records provenance only. It does not select a
   winner or bypass exact-revision review.
7. Active candidates with overlapping write sets remain isolated in separate
   worktrees/batches.
8. The manager never auto-merges or changes issue state from task completion.

## Additive manifest surface

```json
{
  "schema": "dev-swarm.v1",
  "scheduling": {
    "ready_order": "critical_path",
    "resource_limits": {
      "light": 4,
      "heavy": 2
    },
    "progress_events": {
      "signal_stale_seconds": 900,
      "checkpoint_due_seconds": 2700
    }
  },
  "tasks": [
    {
      "resource_class": "heavy",
      "progress_events": {
        "checkpoint_due_seconds": 2700
      },
      "hedge_of": {
        "batch_id": "older-batch",
        "task_id": "original-task",
        "revision": "<exact-launch-commit>"
      },
      "supersedes": [
        {
          "batch_id": "older-batch",
          "task_id": "rejected-task",
          "revision": "<exact-candidate-commit>"
        }
      ]
    }
  ]
}
```

Compatibility rules:

- missing `scheduling` preserves current topological launch order and
  provider-only limits;
- missing `resource_class` means `light`;
- missing progress thresholds disables age events;
- missing lineage preserves the current state and collection shape;
- a hedge must have no task dependency and its exact revision must equal the
  batch base;
- every lineage revision must resolve as an exact Git subject.

## Scheduler changes

### Priority and capacity

Compute each task's reverse-DAG critical-path length:

```text
length(task) =
  1                                      when it has no child
  1 + max(length(child))                 otherwise
```

With `ready_order=critical_path`, order ready tasks by descending length,
existing topological index, then task ID. If the first task cannot acquire its
provider or resource permits, continue scanning so an independently runnable
lower-priority task can use the remaining capacity.

Count both scheduler-managed and adopted live tasks. Provider, total-session,
and heavy limits must all pass before launch.

### Durable progress events

Write append-only events to:

```text
<workspace>/.dev-swarm/<batch>/events.jsonl
```

Use schema `dev-swarm.event.v1`. Observe:

- provider signal: `(mtime_ns, size)` changes in the provider `events.jsonl`;
- checkpoint: task worktree `HEAD` changes from launch or the prior observed
  checkpoint.

Emit once per unchanged epoch:

- `progress.signal_stale`;
- `progress.signal_resumed`;
- `progress.checkpoint_due`;
- `progress.checkpoint_observed`.

Every event binds task, provider, run ID when known, measured age, threshold,
and exact checkpoint revision. Do not emit on every scheduler poll.

Keep existing `status` output stable. Add `status --verbose` for signal age,
checkpoint age, resource class, run mode, and lineage. Add optional progress
and adoption objects to collection output without changing its schema ID.

### Manually resumed run adoption

Add:

```bash
dev-swarm adopt MANIFEST \
  --run task-a=<run-id> \
  --run task-b=<run-id>
```

Selections are validated atomically. Adoption requires:

- the old manager is dead;
- no unselected task remains `running`;
- each selected task is `needs_attention`;
- the exact second-agent run ID is current and its status is `running` or
  `completed`;
- the invocation is a resume and began after `needs_attention`;
- provider, agent, workspace, worktree, branch, and launch ancestry match;
- a live PID also matches its kernel start identity;
- combined adopted work fits all provider/resource semaphores.

The manager polls the durable invocation record; it cannot `waitpid()` a
non-child. A disappeared PID without a completed record returns the task to
`needs_attention`. `stop` must stop only the manager and never signal an
adopted user-owned process group.

## Second-agent durable invocation state

Publish one atomically replaced record per agent:

```text
<agent-state-dir>/run.json
```

Required fields:

- schema `second-agent.run.v1`;
- run ID, backend, agent name, workspace root, effective worktree;
- events and transcript paths;
- PID, process group, and `/proc/<pid>/stat` start ticks;
- start/finish times, status, return code;
- resumed session ID.

Write `running` after the existing agent lock and path resolution but before
provider launch. On every normal or trapped exit, atomically publish
`completed` with the exact wrapper return code.

Bump second-agent state version from 1 to 2 with an additive version-file
migration only. Preserve sessions, logs, transcripts, locks, and worktrees.
Unknown future versions remain warning-only.

## Required tests

Scheduler coverage:

- old manifests preserve current behavior;
- invalid resource classes/limits fail;
- `light=2, heavy=1` permits one heavy plus one light but not two heavy jobs;
- critical-path priority wins a constrained permit;
- provider limits remain active;
- stale/resumed/checkpoint events fire once per epoch and bind exact SHAs;
- hedge base and lineage validation fail closed;
- lineage survives plan, prompt, state, and collection;
- reconcile -> manual resume -> adopt -> normal postflight succeeds;
- wrong run ID, non-resume, wrong worktree/branch, stale PID, overflow, and
  unselected running tasks are rejected;
- completed-before-adoption runs postflight normally;
- stop never signals adopted PIDs.

Shared second-agent coverage:

- active and completed `run.json`;
- exact return code and resumed session identity;
- PID start identity;
- read-only management behavior;
- state v1 -> v2 migration;
- Claude source-rewrite cleanup regression;
- isolated Fugu and Codex metadata contracts.

Use a test-only scheduler poll override instead of long sleeps. Production
polling remains unchanged.

## Safe implementation partitions

1. `second-agent: publish atomic invocation state`
2. `dev-swarm: enforce opt-in resource and critical-path scheduling`
3. `dev-swarm: bind hedge and supersedes lineage`
4. `dev-swarm: emit signal and checkpoint age events`
5. `dev-swarm: adopt manually resumed second-agent runs`
6. `docs: document swarm scheduling and recovery contracts`

Each functional commit includes its black-box tests. Documentation remains a
separate commit.

## Documentation targets

- `/workspace/docs/low-codex-multi-agent.md`
- `/workspace/docs/architecture.md`
- `/workspace/AGENTS_TEMPLATE.md`
- `/workspace/README.md`

## Migration and removal

No existing manifest rewrite is required. Historical manually resumed runs
cannot be inferred; only invocations created after `run.json` support exists
are adoptable.

Rollback order:

1. finish or reconcile adopted runs;
2. remove adoption;
3. stop emitting new scheduling/progress/lineage fields;
4. remove event and semaphore support;
5. remove `run.json` publication last;
6. if fully reverting state, remove only `run.json` after retaining evidence
   and rewrite `VERSION` to 1.

Never remove sessions, logs, worktrees, or transcripts as part of rollback.

## Current implementation boundary

The `/workspace` devcontainer foundation currently exposes an empty/unusable
`.git` directory, so this design was not applied directly to
`/workspace/scripts`. Mutating unversioned infrastructure would make rollback
and review weaker. The next safe action is to restore or identify the
foundation repository's usable Git metadata, then execute the six small
commits above with the `AGENTS.md` wrapper and swarm test matrix.

Until then, the active recovery uses the same rules manually: four external
sessions, at most two heavy lanes, 45-minute durable checkpoints, exact
candidate/report binding, and a coordinator-owned ready queue.
