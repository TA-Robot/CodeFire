# CFR-168 Trusted-Throughput Bottleneck Audit

Date: 2026-07-30

Status: manager operating decision; not product acceptance evidence

## Outcome

The next speed gain must come from increasing trusted-review yield and removing
stage-transition latency, not from launching more independent implementations.
The current system can create candidates faster than it can prove, compose, and
integrate them.

The north-star unit is:

```text
accepted + composition-revalidated product deltas per week
```

Agent count, patch volume, commit count, and raw slot occupancy are capacity
signals only. They are not delivery.

## Current measured baseline

The following snapshot was recomputed from the canonical disposition index,
Git state, scheduler state, and current filesystem at approximately 19:55 UTC:

| Signal | Observation | Interpretation |
|---|---:|---|
| indexed exact review reports | 14 | enough evidence to stop treating rejection as an isolated event |
| `ACCEPT` decisions | 3 / 14 (21.4%) | two accepted lines still have successor or composition gates |
| `REQUEST_CHANGES` decisions | 11 / 14 (78.6%) | review-derived rework is the dominant queue |
| trusted product integrations on canonical recovery head | 0 | candidate throughput has not become delivered throughput |
| product paths changed from recovery product snapshot `e2a218e` to canonical control head | 0 | manager has preserved the no-unreviewed-product rule |
| live external lanes | 4 | available concurrency is already occupied |
| top-level `/workspace/worktrees` directories | 97 | lifecycle overhead is material even after cleanup |
| durable `dev-swarm` state files | 49 | there is enough history for event-driven management |
| `/workspace/worktrees` bytes | 3,037,378,168 | controlled after the earlier Cargo duplication incident |
| shared Cargo target bytes | 781,198,979 | cache reuse is working but lacks a scheduler-owned lease |
| foundation Git metadata | empty `.git` directory | safe scheduler implementation is blocked on repository identity/recovery |

Completed scheduler-managed report lanes vary by scope; observed passed
durations include approximately 14.9, 19.3, 23.1, 24.3, 26.4, 26.4, and 79.6
minutes. The larger unclassified delays occur before or after provider runtime:
candidate readiness, validation, collection, manager disposition, and next-lane
launch are not yet emitted as one event stream. Provider duration alone is
therefore not the lead-time baseline.

The existing active flow record was captured at 09:51 UTC and does not cover
the later review/repair wave. Until typed scheduler events are implemented,
this document treats later arithmetic as manager-observed and keeps unknown
queue segments explicit.

## Root bottlenecks

### 1. The correctness oracle arrives after implementation

Large candidates have reached green focused and broad gates and then failed on
physical ownership, final authority fences, cleanup retryability, publication
ordering, response selection, or non-vacuous evidence. More test execution
does not fix an oracle that observes the wrong thing.

### 2. Review supply is smaller than candidate supply

An exact reviewer must independently bind the immutable tree, run deliberate
regressions, and emit a committed report. Starting another implementation when
the review queue is full increases WIP age and later composition conflicts.

### 3. Shared primitives were frozen after surface implementations

Ledger, deadline, response, authority, snapshot, cleanup, and publication
ownership recur across routes. Surface-local repairs repeatedly rediscover the
same lifecycle defect and then require manual composition.

### 4. Candidate branches mix concerns

Product, tests, task packets, reports, rejected intermediates, and rewritten
WIP history have appeared on the same lineage. The manager then pays a manual
product-delta extraction and exact rereview tax.

### 5. Validation is not content-addressed

The system recompiles and reruns deterministic focused evidence when the exact
tree, command, toolchain, environment contract, and fixtures are unchanged.
Conversely, worker prose can report a gate without one coordinator-owned
record binding those inputs.

### 6. Restart recovery depends on operator memory

Sessions and worktrees survive, but `second-agent` has no atomic invocation
record and `dev-swarm` cannot adopt a manually resumed exact run. Reboots turn
resumption into reconstruction.

### 7. Portfolio scheduling is manual

Each one-task batch owns only its local queue. No supervisor sees the global
critical path, review debt, conflict domain, provider permit, heavy lease,
resource floor, or composition queue.

### 8. Telemetry is not decision-grade

The system does not yet separate provider-active, provider-wait,
validation-queue, gate-active, manager-queue, collect, and integration time.
It can identify slow delivery but cannot automatically assign the delay.

### 9. Hot files collapse otherwise independent conflict domains

The current product snapshot has approximately 33,455 lines in the Python
`codefire` reference, 18,639 in Rust `http.rs`, 12,647 in Rust `main.rs`, and
4,101 in Rust `remote.rs`. Route-local agents repeatedly overlap the same
dispatcher, ledger, response, test, and source-document hunks even when their
business surfaces differ. This increases prompt context, merge conflicts,
manual product-delta extraction, and the blast radius of a reviewer mutation.

File size is not the objective by itself. The defect is that ownership and
write-set boundaries do not match the invariant/conflict domains.

## Development topology

| Plane | Default owner | Capacity | Responsibility |
|---|---|---:|---|
| Control | Codex `gpt-5.6-sol` | 1 coordinator | Freeze, conflict-domain decisions, exact composition, final lifecycle authority |
| Build | Claude Opus 5 | at most 2 heavy lanes | bounded implementation from an accepted preflight contract |
| Preflight | Fugu 1.1 Ultra | 1 light review permit when available | production owner/effect/failure/oracle design before implementation |
| Review | independent Fugu 1.1 Ultra, Claude fallback | 1–2 permits | immutable exact-tree falsification; never edits the candidate |
| Validation | scheduler-owned | one heavy lease per broad train | focused evidence per delta; broad evidence only on a stable composition head |

Codex should not spend quota on routine implementation, routine review,
transcript polling, hash transcription, or repeated test execution. It acts at
stage transitions and global authority decisions. If Fugu has one effective
permit, product review outranks preflight and the review-waiting WIP limit is
one.

## Mandatory flow

1. **Discover:** define the exact product outcome and conflict domain.
2. **Freeze:** Fugu preflight binds the real production entry point, physical
   resource owner/lifetime, effect timeline, cleanup state machine, final
   authority fence, exact boundaries, and deliberate-regression mutations.
3. **READY/BLOCK:** implementation starts only when each material claim has an
   independent observable oracle.
4. **Implement:** Claude works from the exact product base and produces small
   product/test/source-doc commits. It checkpoints before long validation.
5. **Candidate freeze:** the manager recomputes base, head, tree, parent,
   patch digest, changed paths, write-set conformance, clean state, and focused
   evidence. The candidate is immutable.
6. **Review:** an independent session reviews that exact tree. A finding
   produces one closure packet; it does not mutate the subject.
7. **Continuation:** when identity is exact, resume the original implementer
   session/worktree for the closure packet; never resume the reviewer.
8. **Shadow integration:** an accepted product delta enters the single
   manager-owned runtime train immediately. Control/report commits do not.
9. **Composition validation:** run the smallest cross-surface gate after each
   delta and one broad gate only when the accepted queue is empty and the head
   is stable.
10. **Close → Install → Dogfood → Retrospect:** completion remains impossible
    until every required successor and composed review is terminal.

## Hard WIP and quality controls

Superseded for launch admission on 2026-07-31: the two numeric occupancy
limits below were incident-recovery guards, not measured capacity. Current
control uses `check_swarm_admission.py`, candidate reservations, adaptive
provider windows, PSI, and conflict ownership with no fixed total-session
count. The remaining quality/WIP controls still apply.

- External sessions: at most four; heavy work: at most two.
- One writer per conflict domain.
- Review-waiting candidates: no more than available independent review
  permits; normally two, one when Fugu is the only independent reviewer.
- One shadow integration train per runtime/frozen batch.
- A full/broad suite runs at most once per stable composition revision.
- A second consecutive `REQUEST_CHANGES` returns the design to shared
  primitive/oracle Freeze before a third local patch.
- Every 45 minutes after mutation, produce a durable checkpoint; a useful
  multi-hour task may continue indefinitely while checkpoints and unique
  signals remain fresh.
- `ACCEPT` means no required material finding in scope. A useful nucleus with
  remaining mandatory work is recorded as non-terminal, not promoted by
  wording.
- No branch-tip merge when product and control/report commits are mixed. Port
  the reviewed product delta only.

## Next-cohort targets

Use the next five comparable reviewable product packets as an experiment. Do
not rewrite the historical baseline to make these targets pass.

| Metric | Current signal | Next five target |
|---|---:|---:|
| first-pass exact acceptance | 21.4% raw indexed acceptance; terminal trusted yield lower | at least 60%, then 75% in the following cohort |
| review rounds per accepted delta | not yet event-derived | mean at most 1.5; third review under 10% |
| candidate-ready to review start | incompletely recorded | p50 at most 10 min; p95 at most 30 min |
| accept to shadow-composition focused gate | not yet event-derived | p50 at most 30 min; p95 at most 2 h |
| final blocker classes caught by preflight | not yet measured | at least 70% |
| review-derived rework | currently dominant | at most 20% of candidate changed lines and active time |
| first durable checkpoint after mutation | existing examples near 16–22 min | p95 at most 30 min |
| exact provider run producing authoritative artifact | provider failures have produced empty runs | at least 95% |
| broad validation on rejected/intermediate heads | repeated | reduce at least 50%; maximum one per stable head |
| lost source/evidence after restart | zero source loss, reconstruction overhead present | zero loss and at most 45 min uncheckpointed work |

Quality non-inferiority is mandatory: zero critical invariant escapes, zero
false completion, exact identity binding, and complete required evidence.

## Post-closure structural acceleration

Do not perform a broad move-only refactor while CFR-168 candidates are under
exact review; changing the product tree would invalidate evidence and
increase current lead time. After the accepted Python and Rust composition
heads are frozen, make decomposition the first following maintenance train:

1. Extract the stable request context, sealed snapshot, authority fence,
   prepared response, retryable cleanup, publication transaction, and
   error-origin primitives before adding another route.
2. Split Rust `http.rs` by shared lifecycle plus route modules; keep only
   parsing/dispatch at the root.
3. Continue the existing Rust `remote.rs` decomposition around project,
   locks, operations, publication, and Doctor ownership.
4. Keep Python as a reference implementation, but move stable HTTP lifecycle
   primitives and route fixtures out of the 33k-line executable without
   creating a second semantics.
5. Decompose focused tests by owner/domain/fault layer and publish a stable
   validation ID per suite, preserving test names, count, golden bytes, and
   behavior before/after each move.

Each extraction is a behavior-preserving product-only commit with exact
before/after public-wire and test inventory evidence. Measure write-set
overlap, conflict-resolution seconds, context bytes, and affected focused
suite size before claiming acceleration. A smaller file that merely moves a
shared global owner is not success.

## Automation order

The foundation design at
`../dev-swarm-throughput-design-20260730/design.md` is implementation-ready.
The highest-leverage order is:

1. atomic `second-agent.run.v1` invocation state;
2. validated Rust/shared execution profiles plus global resource leases;
3. append-only candidate, gate, review, checkpoint, and resource events;
4. exact review auto-rollover and review-derived implementation continuation;
5. coordinator-generated evidence cards and content-addressed focused gates;
6. global critical-path/review-debt portfolio scheduling;
7. manager-owned shadow integration trains;
8. generated KGI/KPI views and terminal cache/log lifecycle.

The direct implementation boundary remains blocked because `/workspace/.git`
is empty, no foundation remote identity is present, and GitHub CLI is not
authenticated. Initializing a new unrelated history or editing unversioned
infrastructure would weaken rollback and review. Restore the actual foundation
repository metadata or provide its remote and intended base before applying
the prepared scheduler partitions.

## Immediate CFR-168 order

1. Drain the active Upload exact review before starting another Fugu task.
2. Reuse that permit for Storage exact review, then File ancestry.
3. Rereview the active Rust HTTP and File Doctor repairs on their immutable
   exact heads.
4. Freeze the shared Python session/authority/publication primitives before
   the MR-write successor.
5. Move each independently accepted product delta immediately into the
   appropriate Python or Rust shadow train.
6. Run cross-runtime composition review, then Close, Install, real dogfood,
   Retrospect, and KGI/KPI cohort evaluation.

This order keeps every new unit of work on the current trusted-integration
critical path and prevents implementation inventory from outgrowing review
and composition capacity.
