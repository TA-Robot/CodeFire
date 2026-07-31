# Stage 5 Manual Trusted-Throughput Control

Status: active manual control until the foundation repository has usable Git
provenance and the fail-closed `dev-swarm.v2` implementation is accepted

## Purpose

This runbook applies the Stage 5 throughput design without pretending the
current scheduler already enforces it. It reduces review queueing, duplicate
validation, restart loss, resource collisions, identity transcription, and
accepted-but-uncomposed time while keeping acceptance and integration under
manager authority.

## Capability boundary

| Control | Current `dev-swarm.v1` | Manual Stage 5 control | Future `dev-swarm.v2` |
|---|---|---|---|
| task DAG and per-task worktree | implemented | use only after exact dependency audit | retained |
| per-batch provider limit | implemented | reconcile across all batches on one board | machine-wide portfolio limit |
| declared write-set conflict within one batch | implemented | inspect cross-batch conflict domains and postflight actual paths | global lease plus actual-path enforcement |
| process status/collect/reconcile | implemented | bind PID/session/worktree/HEAD and trusted checkpoint | atomic run state and adopt |
| total/heavy resource limit | not implemented | manager owns total 4/heavy 2 leases | global semaphore |
| critical-path ready ordering | not implemented | manager orders the portfolio queue | scheduler ordering |
| progress/checkpoint events | not implemented | ledger every signal/checkpoint and queue reason | append-only events |
| candidate admissibility/card | not implemented | manager recomputes exact identity and evidence | typed candidate event/card |
| independent review rollover | not implemented | prepare blocked review, bind and launch after card | one-shot automatic enqueue |
| implementer continuation | second-agent session resume exists; scheduler adoption does not | exact clean same-agent manual continuation only after old manager exits | typed continuation/adopt |
| validation cache/evidence card | not implemented | manager-owned deterministic focused-gate index | content-addressed `dev-gate` |
| shadow integration train | not implemented | manager ports accepted product delta only | manager-owned automated train |
| trusted KGI/KPI event view | incomplete manual record | append exact stage/queue/resource/cost events | generated portfolio view |

`dev-swarm passed` and collection `all_passed=true` mean only that the current
task process/postflight contract passed. They are not product acceptance,
review acceptance, integration authorization, or lifecycle completion.

## Fail-closed manifest rule

Do not add proposed Stage 5 fields to an active `dev-swarm.v1` manifest:

- `scheduling`;
- `resource_class`;
- `execution_profile`;
- `progress_events`;
- `hedge_of`;
- `continuation_of`;
- `supersedes`;
- candidate/review/gate/shadow-train fields; or
- any other field whose enforcement is not implemented and tested.

The current loader can preserve an unknown field without enforcing its
intended safety semantics. A manifest that validates is therefore not proof
that a future control is active.

The foundation implementation must introduce `dev-swarm.v2`, make an old
v1-only binary reject it, and reject unknown fields recursively on both
versions. Until then, express future state only in blocked task packets and
this manager ledger.

## Portfolio control board

Maintain one current table in the recovery execution ledger:

| Field | Required value |
|---|---|
| surface/task | stable packet or batch ID |
| exact stage | blocked, preflight, implement, candidate-ready, validate, review-ready, review, repair, accepted, composition-ready, composition, terminal |
| conflict domain | physical owner/write-set domain, not just filenames |
| provider/model/role | exact provider, model, and independence role |
| manager/task PID | live identity or terminal state |
| worktree/branch | exact paths and branch |
| base/head/tree | exact Git subjects |
| last trusted signal | timestamp plus evidence reference |
| last durable checkpoint | commit/report/log prefix and timestamp |
| queue reason | provider, heavy lease, review, gate, dependency, manager, conflict, or none |
| review-ready time | timestamp or not ready |
| total/heavy lease | owner and release condition |
| next exact command | one executable action, not prose |

Before every launch and after every terminal collection:

```bash
/workspace/scripts/dev-swarm status <manifest>
/workspace/scripts/dev-swarm collect <manifest>
awk '/MemAvailable|SwapTotal|SwapFree/' /proc/meminfo
du -sb /workspace/worktrees
df -B1 /workspace
```

Manual floors remain:

- at most four external sessions;
- at most two heavy lanes;
- `MemAvailable >= 6 GiB`;
- swap used `<= 2 GiB`;
- workspace filesystem available `>= 100 GiB`.

Do not start another heavy lane below a floor. Do not stop a healthy
long-running lane solely because elapsed time is large.

## Candidate-ready card

Before review, the manager creates:

```text
docs/development/ai/swarm-results/<surface>-candidate-ready-<date>/candidate.md
```

The card contains:

- task packet and manifest digest;
- exact base, parent, head, tree, and merge-base;
- canonical binary patch SHA-256;
- sorted changed paths and declared-write-set result;
- clean tracked/untracked status;
- focused validation IDs, commands, cache keys, terminal results, log paths,
  and log-prefix digests;
- provider/model/role/session/run/worktree/branch identity;
- durable checkpoint history and any rewrite/process deviation; and
- manager `READY_FOR_REVIEW` or `BLOCK` decision.

Recompute identity rather than copying worker prose:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 \
  docs/development/tools/capture_candidate_identity.py \
  --repo "$CANDIDATE_WORKTREE" \
  --base "$BASE" \
  --head "$HEAD" \
  --manifest "$CONTROL_WORKTREE/$MANIFEST" \
  --task-id "$TASK_ID" \
  --format markdown
```

The command resolves the exact commits, merge base, parents, tree, additive
commit list, merge commits, canonical binary patch digest, sorted
name-status/numstat/blob identities, checkout HEAD, clean tracked/untracked
state, manifest/ticket digests, and declared write-set containment. It emits
`READY_FOR_REVIEW` only for a clean, linear, non-empty exact range; otherwise
it emits `BLOCK` and exits 2 while preserving the evidence needed to diagnose
the block. The manifest may live in a manager control worktree above the task
base; its ticket is always resolved from the candidate repository.

Dirty, non-descendant, out-of-write-set, rewritten-with-unreconciled-evidence,
missing-focused-gate, or moving candidates remain `BLOCK`.

## Review rollover

Prepare the review ticket, typed packet, report path, counterexample matrix,
and blocked manifest during implementation. Keep exact candidate fields
`UNBOUND`.

After the candidate card passes:

1. bind base/head/tree/parent/patch/paths/log digests;
2. create the review task commit immediately above the candidate with the
   bound ticket, card, and the manager-vetted
   `capture_candidate_identity.py` blob when the candidate baseline does not
   already contain it;
3. record that helper's SHA-256 in the card so the reviewer can recompute the
   same canonical identity without leaving the isolated task worktree;
4. bind reviewer provider/model/session independence;
5. validate and plan the exact manifest;
6. record `review_ready`;
7. launch immediately when the review/light lease is free; and
8. record `review_start`.

```bash
/workspace/scripts/dev-swarm validate <review-manifest>
/workspace/scripts/dev-swarm plan <review-manifest>
/workspace/scripts/dev-swarm start <review-manifest>
```

Review preparation target is 100%. Candidate-ready to review-start target is
p50 10 minutes and p95 30 minutes. Never change the bound candidate after
review starts. A missing helper is not an identity mismatch—the reviewer can
reconstruct the command—but it is avoidable manager/reviewer time.

After the scheduler and reviewer are terminal, verify a report-only handoff
with the same tool:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 \
  docs/development/tools/capture_candidate_identity.py \
  --repo "$REVIEW_WORKTREE" \
  --base "$REVIEW_TASK_COMMIT" \
  --head "$REVIEW_HEAD" \
  --manifest "$CONTROL_WORKTREE/$REVIEW_MANIFEST" \
  --task-id "$REVIEW_TASK_ID" \
  --expected-role reviewer \
  --require-exact-write-set
```

This additionally blocks a wrong role or any declared report path missing
from the committed range. Each changed blob carries committed byte length and
SHA-256, so the immutable report digest comes from Git rather than a mutable
working file. The manager must still read and route the literal verdict;
identity automation never converts scheduler success into acceptance.

## Exact implementer continuation

After `REQUEST_CHANGES`, prefer the original Claude implementer session only
when all of these are true:

- prior manager/task is terminal and holds no lock;
- exact agent/session/worktree/branch/provider/model/role match;
- worktree is clean at the reviewed head/tree;
- new immutable repair packet binds the exact report and all findings;
- approved write set is equal or narrower; and
- continuation appends commits without amend/rebase/history rewrite.

Inspect before resume:

```bash
claude-second-agent --agent "$AGENT" paths
claude-second-agent --agent "$AGENT" session
git -C "$WORKTREE" status --short
git -C "$WORKTREE" rev-parse HEAD
```

Then invoke the exact existing agent through the approved `dev-agent` role.
If any identity is uncertain, start a fresh isolated implementation lane.
Reviewer sessions are never implementation continuations.

## Manual gate evidence index

For each stable batch, maintain:

```text
docs/development/ai/swarm-results/<batch>-gate-evidence-index-<date>/index.md
```

The key includes exact tree, declared non-Git input digests, validation ID and
argv, selected environment contract, toolchain/binary provenance, and
fixture/corpus version.

Run long gates without a shell and measure them from a monotonic clock:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 \
  docs/development/tools/run_timed_gate.py \
  --cwd "$WORKTREE" \
  --timeout 3600 \
  --log "$LOG" \
  --result "$RESULT_JSON" \
  -- python3 -m unittest discover -s tests -p 'test_python_*.py'
```

The result binds argv, cwd, monotonic duration, timeout outcome, complete log
bytes/SHA-256, wall timestamps, and wall-minus-monotonic drift. Any drift
beyond two seconds is an explicit clock discontinuity; never replace the
monotonic duration with `date`, shell `time`, filesystem mtime, or provider
wall timestamps.

Reuse is allowed only when the task packet explicitly permits a deterministic
focused gate and every key byte matches. Never reuse:

- failed, flaky, interrupted, or truncated results;
- Install, Dogfood, final release, or terminal capability gates;
- results from a different tree, toolchain, fixture, corpus, argv, or
  environment contract; or
- worker-claimed results whose identity was not manager-recomputed.

Run focused gates per accepted delta. Run a broad gate at most once on the
stable composition revision after the accepted queue is empty.

Record avoided gate seconds as well as executed gate seconds.

## Shadow composition

The manager owns one shadow head per frozen runtime/batch.

1. Admit only an exact independently accepted product delta.
2. Exclude worker ticket, report, task packet, history, and branch-tip
   control commits.
3. Apply one delta.
4. Run the smallest cross-surface focused gates.
5. Stop on conflict, changed digest, focused failure, or unreviewed manager
   resolution.
6. Freeze and independently review any conflict-resolution candidate.
7. Run the broad gate only when the accepted queue is empty and head stable.

Track accepted-to-composition p95 at two hours and accepted-but-uncomposed age
continuously.

## Event and KPI record

At minimum record:

- packet, preflight, implementation, candidate, validation, review,
  integration, Close, Install, Dogfood, and Retrospect ready/start/end;
- first trusted signal and durable checkpoint;
- provider wait, gate wait, review wait, manager wait, dependency wait, and
  conflict time;
- review round, finding recurrence, rework LOC/time/cost;
- total/heavy occupancy, memory/swap/disk minima, restart/recovery/lost work;
- validation cache hit/miss and avoided seconds; and
- exact evidence completeness.

Manual targets:

- unsupported Stage 5 field in active v1 manifests: 0;
- scheduler pass misreported as ACCEPT: 0;
- exact candidate evidence completeness: 100%;
- review packet prepared before candidate: 100%;
- candidate-ready to review-start p95: 30 minutes;
- accepted to composed focused-gate p95: 2 hours;
- first-pass exact ACCEPT: at least 60% for the next five comparable packets;
- mean review rounds: at most 1.5;
- third review: below 10%;
- same-key broad-gate repeats: 0;
- stable composition broad gates: at most one;
- durable checkpoint p95: 30 minutes, hard manual alert at 45 minutes;
- provider terminal report artifact rate: at least 95%; and
- heavy collision, wrong-head continuation, identity transcription
  correction, unsafe adoption, and false cache hit: 0.

Agent count, commit count, LOC, transcript size, and raw slot occupancy are
not throughput success metrics.

## Foundation implementation gate

Do not edit `/workspace/scripts` until the actual foundation Git metadata,
remote, intended base, and authentication/provenance are restored and
reviewable:

```bash
git -C /workspace rev-parse --show-toplevel
git -C /workspace remote -v
git -C /workspace fsck --full
git -C /workspace status --short
```

After restoration, implement in the dependency order frozen by the throughput
design: fail-closed v2 manifest first, atomic invocation state, global
resource/critical-path control, events, adopt/continuation, candidate/review
rollover, typed gate/evidence cache, shadow integration, portfolio KGI view,
and terminal cleanup.
