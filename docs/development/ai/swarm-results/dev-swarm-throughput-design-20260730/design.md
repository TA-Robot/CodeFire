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
      "execution_profile": "rust-shared-low-memory-v1",
      "progress_events": {
        "checkpoint_due_seconds": 2700
      },
      "hedge_of": {
        "batch_id": "older-batch",
        "task_id": "original-task",
        "revision": "<exact-launch-commit>"
      },
      "continuation_of": {
        "batch_id": "older-batch",
        "task_id": "original-implementer",
        "head_revision": "<exact-clean-candidate>",
        "run_id": "<durable-provider-run-id>"
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
- missing `execution_profile` preserves the inherited environment;
- missing progress thresholds disables age events;
- missing lineage preserves the current state and collection shape;
- a hedge must have no task dependency and its exact revision must equal the
  batch base;
- a continuation is opt-in, implementer-only, and must begin at its declared
  terminal clean head;
- every lineage revision must resolve as an exact Git subject.

## Scheduler changes

### Validated execution profiles

Do not add arbitrary manifest environment injection. Resolve an optional
`execution_profile` through a foundation-owned allowlist and record its ID and
non-secret effective values in task state. The initial
`rust-shared-low-memory-v1` profile sets the pinned Rust/Cargo homes, one
shared target directory, bounded Cargo/test jobs, and incremental-build
policy already used by the recovery envelope. Reject an unknown profile or a
resolved cache path outside the configured workspace cache.

The profile makes build reuse and resource limits part of task identity
instead of prompt advice. Terminal cleanup removes an inactive task-local
target, but never blindly cleans the shared target. Shared-cache retention is
size/age based and runs only when no matching Cargo process is active.

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

### Review rollover and shadow integration train

`candidate.ready` must be a typed event, not a condition discovered by manager
polling. It binds the exact base/head/tree, changed-file set, patch digest,
task-packet digest, focused validation IDs, clean-state result, and provider
run identity. The portfolio supervisor may then enqueue a prepared independent
review automatically when:

- the candidate is a descendant of its frozen base;
- every changed path is inside the declared write set;
- no required focused validation is missing or stale;
- the requested reviewer is independent of the candidate run; and
- global provider/resource permits are available.

Automatic enqueueing is not automatic acceptance. A `REQUEST_CHANGES` result
returns the surface to Freeze with immutable findings; `ACCEPT` enters a
manager-controlled integration queue.

### Review-derived repair continuation

After a bound `REQUEST_CHANGES`, the manager may resume the original
implementer session instead of paying repository-orientation cost again.
`continuation_of` is accepted only when:

- the prior task is a terminal implementer task with durable run/session
  identity;
- its worktree and branch still exist, are clean, and equal the declared exact
  candidate head and tree;
- the new repair packet is a committed immutable successor that binds the
  exact review report and findings;
- provider, exact model, role, workspace, worktree, branch, and ancestry
  match;
- the new write set is equal to or narrower than the manager-approved repair
  set; and
- no other process or scheduler state owns the agent lock.

Resume appends a new provider invocation and new commits; it never amends,
rebases, rewrites the prior log, or treats old validation as evidence for the
new head. Put the new findings and changed acceptance evidence first in the
resume prompt so stale session assumptions are explicitly superseded.

Reviewer sessions are never used as implementation continuations, and a
reviewer remains a new independent run. If any continuation identity cannot
be proven, fail closed and create a new isolated implementation lane; do not
silently fall back to an unbound session.

The integration queue owns one shadow head per frozen batch. It applies only
the accepted product delta, never a worker branch tip or its manager/control
documents. After each admitted delta it runs the smallest cross-surface gates
selected by the packet's invariant and changed-symbol map. A broad/full gate
runs only when the ready accepted queue is empty and the composition revision
is stable. Any conflict, changed patch digest, failed focused gate, or
previously unreviewed conflict resolution produces a new exact candidate and
review; it never falls through to the next delta.

### Content-addressed validation evidence

Store a successful deterministic gate under a key covering:

```text
sha256(
  exact tree and declared non-Git input digests
  + validation ID and argv
  + selected environment contract
  + toolchain/binary provenance
  + fixture/corpus version
)
```

The scheduler may reuse the result only for the exact same key and must expose
the original run, log digest, duration, and cache decision in collection
output. Failed, flaky, interrupted, time-sensitive, install, dogfood, and
final release gates are never promoted from cache. A packet may mark a
focused deterministic gate cacheable; absence of that declaration remains
the compatible run-always behavior.

This cache avoids rebuilding and rerunning unchanged proof while preserving
the exact-revision evidence boundary. It is separate from Cargo's compilation
cache.

### Manager-generated postflight evidence

Workers must not spend model time transcribing identities and gate output that
the coordinator can derive. Run declared validations through a small
`dev-gate` wrapper that emits one append-only typed record containing:

- validation ID, exact argv/cwd, execution profile, start/end, and return code;
- exact tree and declared-input digests;
- toolchain/binary provenance and fixture/corpus version;
- stdout/stderr byte counts and SHA-256 digests, plus a bounded terminal
  excerpt;
- parsed test pass/fail/skip counts when the command exposes a supported
  format; and
- resource samples and cache/run decision.

At task exit, `dev-swarm` generates a state-owned handoff card with base, head,
tree, parent, merge base, commit range, patch digest, changed paths,
write-set conformance, clean state, log/event-prefix digests, duration,
provider identity, and every required validation record. The worker supplies
only semantic outcome, residual risk, and next action. The generated card is
evidence input, not an acceptance decision, and is stored outside the product
worktree so it cannot contaminate a candidate delta.

`candidate.ready` is emitted from this generated evidence. A worker-authored
hash, count, or success sentence never overrides a recomputed value.

### Counterexample and context compaction

Freeze resolves recurring failure signatures from
`docs/development/ai/failure-cookbook.md` into stable counterexample IDs. Each
reviewable invariant carries:

- one real production entry point;
- one independent observable oracle;
- one deliberate-regression mutation that the oracle rejects;
- exact boundary/minus-one/plus-one cases when a limit exists; and
- the smallest source/test/context files required to reproduce it.

The generated worker handoff contains only the frozen objective, exact
identity, applicable counterexamples, write set, focused validations, and
their content-addressed files. Stable repository/role instructions form a
cacheable prompt prefix; historical transcripts and unrelated packet prose do
not enter the worker context. The full manager record remains durable outside
the prompt.

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
- unknown execution profiles and cache paths outside the configured root fail;
- the Rust profile binds one shared target and bounded build/test concurrency
  into state, launch environment, and validation-cache identity;
- terminal cleanup reclaims an inactive private target but never an active or
  shared target;
- `light=2, heavy=1` permits one heavy plus one light but not two heavy jobs;
- critical-path priority wins a constrained permit;
- provider limits remain active;
- exact candidate-ready events enqueue an independent review once and bind all
  candidate identities;
- dirty, out-of-write-set, stale-evidence, non-descendant, and non-independent
  candidates do not enqueue;
- accepted product deltas enter the shadow queue while reports/control files
  remain excluded;
- conflict resolution and post-apply focused-gate failure stop the train and
  require a fresh exact review;
- broad validation waits for a stable empty accepted queue;
- validation-cache keys change for tree, argv, environment, toolchain,
  fixture, and declared-input changes;
- failed, flaky, install, dogfood, and final gates cannot be reused;
- postflight handoff cards recompute identity, patch, path, clean-state, log,
  duration, provider, and validation evidence without worker transcription;
- malformed gate output remains raw bounded evidence and never fabricates a
  parsed count or passing state;
- worker-authored identity/count fields cannot override generated values;
- counterexample handoffs contain the production entry point, oracle,
  deliberate regression, exact write set, and no unrelated transcript;
- stale/resumed/checkpoint events fire once per epoch and bind exact SHAs;
- hedge base and lineage validation fail closed;
- lineage survives plan, prompt, state, and collection;
- reconcile -> manual resume -> adopt -> normal postflight succeeds;
- wrong run ID, non-resume, wrong worktree/branch, stale PID, overflow, and
  unselected running tasks are rejected;
- completed-before-adoption runs postflight normally;
- stop never signals adopted PIDs.
- exact clean implementer continuation resumes the durable session/worktree
  and appends commits and a new invocation record;
- dirty, moved, rewritten, wrong-model, wrong-role, widened-write-set,
  unbound-review, and locked continuations fail closed;
- a reviewer run cannot become an implementation continuation and old gate
  evidence cannot satisfy the successor head;

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
2. `dev-swarm: add validated execution profiles and terminal cache policy`
3. `dev-swarm: enforce opt-in resource and critical-path scheduling`
4. `dev-swarm: bind hedge and supersedes lineage`
5. `dev-swarm: emit signal and checkpoint age events`
6. `dev-swarm: adopt manually resumed second-agent runs`
7. `dev-swarm: resume exact review-derived implementer continuations`
8. `dev-swarm: enqueue exact independent reviews from candidate events`
9. `dev-swarm: add content-addressed deterministic gate evidence`
10. `dev-gate: emit typed validation evidence`
11. `dev-swarm: generate state-owned postflight handoff cards`
12. `dev-swarm: run manager-owned shadow integration trains`
13. `dev-agent: generate counterexample-bound compact handoffs`
14. `docs: document swarm scheduling and recovery contracts`

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
