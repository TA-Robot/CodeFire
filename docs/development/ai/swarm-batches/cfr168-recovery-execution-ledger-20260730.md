# CFR-168 / CFB-136 Recovery Execution Ledger

This is the live execution ledger for the frozen first batch after the 2026-07-29 host restart.
Canonical issue status remains `investigating`; this ledger does not authorize Close.

## Authority and recovery boundary

- Immutable audited product snapshot: `e2a218e015953f082245f36dbc6aee906b837e5a`.
- Canonical integration branch: `integration/cfr168-p0-canonical`.
- The host restarted at 2026-07-29 23:31 UTC. All old manager PIDs became stale while their
  worktrees and Git objects remained intact.
- No old task was classified as a product failure merely because its process disappeared.
- Every non-empty interrupted worktree was first committed on its own agent branch. Recovery tasks
  start from those commits and do not mutate the dirty main worktree.
- External implementation models are fixed to Claude Opus 5 (`claude-opus-5`) and Fugu 1.1 Ultra
  (`fugu-ultra-v1.1`). No external Codex implementation lane is active.
- The initial recovery tasks used an eight-hour envelope. Current
  repair/review successors use 12 hours; the streaming checkpoint rule below
  is a durability requirement, not an early task timeout.

## 2026-07-30 second host-restart recovery

The host restarted again at approximately 2026-07-30 02:58 UTC. The operator reported memory
exhaustion as the cause. The restarted container cannot read the previous host boot's kernel OOM
record, so the cause is operator-observed rather than independently proven by this container.

The restart interrupted every remaining external-agent process. Their `dev-swarm` state files still
say `running`, but all recorded manager/worker PIDs are stale and no Claude, Fugu, Cargo, Rust,
Python-test, `second-agent`, or `dev-swarm` process survived. The states must not be interpreted as
live work.

Crash recovery found and repaired filesystem write loss:

- 19 zero-length loose Git objects were moved, without deletion, to
  `.git/recovery/oom-20260730T0258Z/zero-objects/`.
- The staged ledger blob
  `ce848ba02134dd762a3da84ea61d8f13c9a2d830`, canonical ledger commit
  `afcfa9bac88f616162658a00ec21fbf792991f28`, canonical HTTP-review commit
  `e28c90d4f10a8e7c0a27846750bf682e341e0cd5`, and HTTP repair-base review commit
  `b382b27a73ffabf7668dd8924ae75cd14dba93be` were reconstructed byte-for-byte from
  surviving trees, source commits, indexes, and metadata.
- The HTTP-review working file was zero-length in the canonical and HTTP repair-base worktrees. It
  was restored from committed blob `52127017c52819fc014159633875997a71f108e4`; both working copies
  now hash to that exact blob.
- Two branch reflogs, two linked-worktree HEAD reflogs, and five interrupted Claude event logs had
  one trailing NUL run. The pre-repair bytes were copied under
  `.git/recovery/oom-20260730T0258Z/`; only the trailing NUL runs were removed. Every repaired event
  log parses as JSONL.
- `git fsck --full` now reports no missing or corrupt object. Its remaining output consists only of
  ordinary dangling objects from interrupted/reconstructed work.

Every non-empty CFR-168 agent worktree found by the post-restart audit is now durable:

| Interrupted work | Recovery commit | Post-recovery state |
|---|---|---|
| Rust storage Claude finisher | `b3bc32a6bd4c792edc5a1302c61e16c624a9b648` | 436 insertions / 59 deletions preserved; 779/779 CLI binary tests and source-doc check had passed before the final strict-Clippy command was interrupted |
| Rust file-remote ancestry Fugu precursor | `9437ed86cd4fd7b5ed752d645b078101ef6bedaa` | prior 859-insertion / 173-deletion precursor preserved for comparison; it is not the reviewed Claude candidate |
| Rust HTTP MR preflight Fugu precursor | `dd8ef3e0c71064ba5cc8c71a3d32ea4955faec1c` | prior 272-insertion / 8-deletion precursor preserved for comparison; it is not the reviewed Claude candidate |

The File Doctor, File Ancestry, and Python Upload zero-residual repair workers had only inspected
their inputs and left clean worktrees. The MR-list and MR-mutation reviewers preserved analysis in
their event logs but had not committed formal reports. Those five tasks therefore require resume or
replacement; no product/report commit is inferred from a transcript.

The temporary Rust toolchain previously installed below `/tmp/codefire-rustup` did not survive the
host restart. Rust gates remain historical evidence until the exact toolchain is rehydrated and
rerun on the resumed candidate.

### Memory-safe restart envelope

Until the scheduler enforces resource classes directly, recovery uses this explicit envelope:

- at most four external-agent sessions at once;
- at most two heavy test/build jobs at once, with only one full-suite gate at a time;
- Rust work uses `CARGO_BUILD_JOBS=2` and `RUST_TEST_THREADS=2`;
- full Python, full Rust, release builds, and cross-runtime corpus runs are manager-serialized;
- no new heavy job starts below 6 GiB host `MemAvailable` or while swap use exceeds 2 GiB;
- each implementer creates a durable WIP checkpoint before a broad/full gate;
- no stale `running` state is resumed until its worktree, event log, and exact base/head have been
  re-audited.

## 2026-07-30 third host-restart and disk recovery

The host restarted a third time at approximately 2026-07-30 08:21 UTC after the operator observed
both memory pressure and Windows C-drive exhaustion. The container cannot mount or inspect the host
C drive, so the host's current free-space value is not claimed here. The container-side cause is
measured:

- `/workspace` occupied approximately 98 GiB before cleanup;
- 49 per-worktree Cargo `target` directories occupied 91.8 GiB in aggregate;
- `/workspace/worktrees` occupied 67 GiB and `/workspace/project` occupied 28 GiB because both
  included regenerated Cargo artifacts;
- source, Git metadata, task packets, review reports, event logs, and agent worktrees themselves
  were not the dominant consumers.

Only reproducible Cargo build artifacts were removed with Cargo-aware cleanup; source changes,
commits, worktree registrations, agent state, logs, and review evidence were preserved.
`fstrim /workspace` was then run so the backing filesystem could deallocate unused extents. The
post-cleanup audit records `/workspace` at 2.678 GiB, `/workspace/project` at 403 MiB,
`/workspace/worktrees` at 2.3 GiB, and one empty Cargo `target` directory. The mounted filesystem
reports 844 GiB available. This does not prove that the host VHDX has already returned the same
space to Windows.

The abrupt stop also left 12 zero-length loose Git objects and two zero-length working copies of
the MR-mutation review. The empty objects, ref/reflog snapshots, linked-worktree HEAD logs, and
indexes were moved without deletion under
`.git/recovery/oom-disk-20260730T0816Z/`. Surviving blobs, trees, parent identities, author and
committer metadata, and exact commit messages were sufficient to reconstruct:

- canonical review commit `0da5855124fc9f52465cabb13fafa44e55e10fde`;
- MR-mutation repair-base review commit `a4b21fdec3dafd344396447c1c02a52e767c16d6`;
- canonical tree `bf939fe1ca58449d41dec2ceac9c86069ab21477`;
- repair-base tree `f424c8aa4760da4d49c845e98a6c621dfa74d022`;
- review blob `d466db49a6d832c6509052f04d295d9500dc6003`.

The reconstructed object names exactly match the damaged ref targets. A full Git audit now reports
no missing/corrupt object, and the active object store has no zero-length loose object. The
remaining `fsck` output is dangling interrupted/recovery work only.

### Disk-safe continuation envelope

The memory envelope above remains in force and is extended as follows:

- all resumed Rust lanes use one manager-selected shared Cargo target directory instead of a
  separate target below every worktree;
- `CARGO_INCREMENTAL=0`, `CARGO_BUILD_JOBS=2`, and `RUST_TEST_THREADS=2` are fixed for recovery
  gates;
- only one heavy Cargo gate runs at a time; agent inspection, report writing, and Python-focused
  work may proceed in the remaining lanes;
- `/workspace` and the shared target are measured before and after every heavy gate;
- no heavy gate starts if `/workspace` exceeds 20 GiB or mounted free space falls below 100 GiB;
- the shared target is reproducible cache, not evidence, and may be Cargo-cleaned after its
  authoritative log/result has been committed;
- worktrees, `.git`, task packets, reports, state, and event logs are never selected by automatic
  cache cleanup.

## Active recovery lanes (superseded restart snapshot)

| Surface | Recovery base | Provider | New batch | State at restart audit | Required finish |
|---|---|---|---|---|---|
| Python HTTP Doctor | `d79be3e` | Claude Opus 5 | `cfr168-python-http-doctor-canonical-recovery-20260729` | product result `01a7c862`; focused 19/19 and 11 neighboring cases pass; standalone full-suite claim is not accepted because its background run stopped with the agent | independent review, composition gate, canonical integration |
| Python HTTP upload ancestry/signature | `3153d75` | Claude Opus 5 | `cfr168-python-http-upload-recovery-20260729` | product result `cb13da08`; focused 10/10 and agent full Python 431/431 pass; CFB-138 discovered at default-profile scale | zero-residual repair was interrupted before a diff; resume repair, independent re-review, canonical integration |
| Python HTTP branch session | `8265ea1` | Claude Opus 5 | `cfr168-python-http-branch-recovery-20260729` | product result `fbef572e`; focused 13/13 and agent full Python 433/433 pass; one old CLI assertion encoded the former deadline defect | independent review, corrected neighboring assertion, canonical integration |
| Python HTTP MR create/review/apply | `cab3126` | Claude Opus 5 | `cfr168-python-http-mr-mutation-recovery-20260729` | exact target `21fe751` is formally `REQUEST_CHANGES`; hidden `local_v1` ledgers/unaccounted traversals, reusable replay nonce, legacy preflight residue, and typed-fence defects are bound in report `3567589` | repair packet `cfr168-python-http-mr-mutation-zero-residual-repair-20260730`, re-review |
| Rust storage | `40c095d` | Fugu 1.1 Ultra, Claude Opus 5 finisher | `cfr168-rust-storage-recovery-20260729` | finisher WIP `b3bc32a` has 779/779 CLI evidence, clean format, strict all-target Clippy, 11 focused storage-decision tests, four exact legacy storage tests, source-doc and diff checks in the recovered JSONL; the final combined remote/HTTP rerun was interrupted | write a no-build evidence report from the durable log, then independent review |
| Rust file-remote ancestry | `fe54e60` | Fugu 1.1 Ultra, Claude Opus 5 hedge | `cfr168-rust-file-remote-ancestry-recovery-20260729` | Claude candidate rejected; zero-residual repair was interrupted before a diff | resume operation-scoped owner/fence repair and re-review |
| Rust file-remote Doctor | `ea8f2a6` | Fugu 1.1 Ultra, Claude Opus 5 hedge | `cfr168-rust-file-doctor-recovery-20260729` | Claude candidate accepted with residual findings; zero-residual repair was interrupted before a diff | resume all Should/Could hardening and re-review |
| Rust HTTP MR create/review preflight | `3c23e38` | Fugu 1.1 Ultra, Claude Opus 5 hedge | `cfr168-rust-http-mr-preflight-20260729` | Claude candidate review says `REQUEST_CHANGES` on missing/inexact evidence; Fugu precursor preserved as `dd8ef3e`; exact repair packet is registered at `9e5aec3` | execute repair, re-review |

Two Rust lanes remained in design analysis for about 45 minutes without a product diff. They were
not stopped: independent Claude Opus 5 hedge lanes now run from the same immutable task bases:

- `cfr168-rust-file-remote-ancestry-claude-hedge-20260730`;
- `cfr168-rust-http-mr-preflight-claude-hedge-20260730`.

This is candidate competition, not dual integration. Each result must satisfy the same packet,
focused/broad gates, and independent review; only the accepted product commit is integrated.

## 2026-07-30 10:24 UTC streaming checkpoint

The recovery pipeline now has durable repair candidates on three still-open surfaces. None is
accepted or integrated:

| Surface | Exact repair range | Changed production/test set | Current independent work |
|---|---|---|---|
| Rust storage | `40c095d..b3bc32a` | `crates/codefire-cli/src/main.rs`, `storage.rs`, `tests.rs`, and new `tests/storage_decision_work.rs`; +2310/-498 | Fugu 1.1 Ultra review is checkpointing its confirmed severity-ordered findings as a report-only commit; no Cargo rerun belongs to that review |
| Python HTTP MR list | `20b5080..38da898` | `codefire`, four source-contract documents, and `tests/test_python_http_mr_list_decision_work.py`; +1635/-105 | Fugu 1.1 Ultra is reproducing live-socket authority-fence and filesystem-identity boundaries after the exact focused module passed 33/33 |
| Python HTTP MR mutation | `3afef8f..6dd4527` | `codefire` and `tests/test_python_http_mr_mutation_decision_work.py`; +1516/-295 | Fugu 1.1 Ultra is reproducing request-ledger, replay/publication, cleanup, and unowned-authority counterexamples |

The MR-list and MR-mutation candidates both modify `codefire`; they are not independent
cherry-picks. Their accepted successors must be ported in the serialized Python-spine order below
and re-reviewed on the composed exact revision. The Rust storage range has no file overlap with
those Python ranges, but it still waits for its own report and repair.

The development control plane is also under repair, independently of product source. Canonical
commit `b307b47` binds the KGI/KPI `REQUEST_CHANGES` review, `9dd0da4` freezes the one-piece
zero-residual repair packet, and `8e49bac` completes the packet's Markdown contract. Claude Opus 5
is implementing the semantic validator, 55-mutation negative matrix, honest reference record,
lifecycle reconciliation, and decision-rule repair in an isolated worktree. These commits do not
authorize a product acceptance decision.

### Streaming review rule

Long adversarial review remains valuable and is not stopped merely for elapsed time. It must,
however, create a durable artifact while it continues:

- after 45 minutes without a committed unique artifact, checkpoint all confirmed findings in a
  report-only WIP or final review commit;
- mark hypotheses and unresolved questions separately from reproduced blockers;
- a repair lane may start from confirmed blocking findings while deeper review continues against
  the eventual repaired exact candidate;
- only the final review decision on that exact repaired candidate can authorize integration;
- manager coordination continues with changed-file mapping, dependency ordering, task-packet
  preparation, and lightweight gates instead of polling as an idle lane.

This is a flow control, not a task timeout. It preserves hours-long useful investigation while
preventing an entire dependency chain from waiting for the reviewer's last possible observation.

### Next-slot ready queue (superseded checkpoint snapshot)

The four-external-session recovery ceiling remains in force. All four slots are occupied at this
checkpoint, so a fifth external lane is not launched merely to maximize utilization. When a slot
becomes available, the manager dispatches from this verified clean queue without repeating
orientation:

| Priority | Ready task base | Why it is ready / collision boundary |
|---:|---|---|
| 1 | Python Upload/CFB-138 zero-residual repair `72209c5` (packet `5f79899`) | Critical predecessor for the Python spine; exact rejected candidate `dece688` and review `0b1fd6f` are bound. It writes `codefire` and source docs, so implementation stays isolated until the later MR ports |
| 2 | Rust file-remote ancestry zero-residual repair `aba1439` (packet `86a0c2e`) | Exact rejected candidate `e93ebc2` and review `ac626f6` are bound; unlocks clone/storage-report/GC successors. It avoids storage `main.rs` but overlaps later HTTP work in `work_budget.rs` |
| 3 | Rust File Doctor zero-residual repair `e41f26c` (packet `dbeaac1`) | Exact candidate `52a025e` has independent accepted-nucleus report `84ea698`; zero-residual Should/Could repair is concentrated in `remote/doctor.rs` apart from shared docs |
| 4 | Rust HTTP MR preflight zero-residual repair `9e5aec3` (packet `1a371fc`) | Exact candidate `71854e2` and rejecting report `b1905e9` are bound, but its Rust/Python/conformance/source-doc write set has the largest overlap and is therefore last in this ready group |

The Storage repair enters this queue as soon as its report-only checkpoint and exact repair packet
are committed. Its relative priority is recomputed from the report's confirmed write set; the
uncommitted reviewer transcript is not a task contract.

### Gate ladder and evidence posture

Canonical head `984906a` is still byte-identical to immutable product snapshot `e2a218e` across
`codefire`, `crates/`, `tests/`, `Cargo.toml`, and `Cargo.lock`. The commits since that snapshot are
control-plane evidence and task contracts; no recovery candidate is integrated. Candidate-only or
rejected-revision test results therefore cannot authorize canonical product acceptance.

To preserve the mandatory final evidence without repeatedly spending heavy-gate time on revisions
already expected to change:

1. **Lane candidate:** syntax/static, focused production, mutation/non-vacuity, exact/−1/+1, and
   1x/3x/10x evidence.
2. **Each overlapping port:** the affected composition-focused suites and an exact-candidate
   independent decision.
3. **Stable Python or Rust runtime-group head:** one broad runtime gate after no more same-runtime
   ports are pending.
4. **One final frozen integrated SHA:** full Python, Rust workspace, format, strict Clippy, demo,
   source/project docs, shared Rust/Python conformance, independent High review, clean release,
   isolated Install, installed reversals, canonical Dogfood, issue synchronization, Retrospect, and
   the final zero-residual ledger audit.

An intermediate pre-install milestone is a `source-close candidate`, not a status transition.
`CFR-168`/`CFB-136`/`CFB-138` remain open until the final installed and dogfood evidence is bound to
the immutable accepted product SHA.

### Resource and lightweight-gate sample

At `2026-07-30T10:24:11Z`, `/workspace` measured `2,836,481,180` bytes, the mounted filesystem
reported `905,999,032,320` bytes available, `MemAvailable` was `15,124,860` KiB, and swap use was
zero. No heavy gate is running. Canonical head `8e49bac` is clean, and the bounded aggregate AI
development gate passes all seven subchecks. Windows C-drive space remains unavailable from the
container and no claim is made about it.

## Recovered candidate checkpoint

The following results are durable candidate commits, not canonical integration or Close evidence:

| Surface | Candidate | Verified candidate evidence | Independent disposition |
|---|---|---|---|
| Python HTTP Doctor + branch composition | `701469f91dd0c4c52c5628fb3e9d9e9d0ab0c714` | combined focused 42/42, Python-pattern 463/463, and unrestricted 691/691 in 2550.503 seconds pass with the configured Cargo toolchain; corrected expectation is included | Fugu report `4659318` says `ACCEPT` for the corrected composition only; product integration still waits for the upload disposition |
| Python HTTP upload default-profile scale | `dece688afea815d1242e7751367052d1ceac4580` | focused 13/13, neighboring owners 59/59, selected legacy 22/22, and Python-pattern 434/434 pass; 1201-commit default-profile reversal and 1x/3x/10x counters are bound | independent report `0b1fd6f50fce2126a07238c422b1ff01b575c047` says `REQUEST_CHANGES`; zero-residual repair `cfr168-python-http-upload-zero-residual-repair-20260730` is active |
| Python HTTP MR create/review/apply | `21fe75175e997cc7c6f8c3ef2a7558144d850f94` | dedicated 18/18 pass; broad 667-case run has 665 passes and two Cargo-PATH-only install failures | Claude report `35675899c66070564fa010894315cbe33bb57657` says `REQUEST_CHANGES`; repair task and manifest are committed at `dd23a63` and `3afef8f` |
| Python HTTP MR list | `37f7427d46d2cf3ad66de3c1885ef4c0527cae70` | dedicated 18/18 in 54.077 seconds, Python-pattern 481/481 in 432.026 seconds, compilation, source/project docs, and diff checks pass | Claude report `39b803bad72d0faceb538f1c5fbc22964699c7c2` says `REQUEST_CHANGES`; repair task and manifest are committed at `1695cc8` and `20b5080` |
| Rust live file Doctor | `52a025ecda3fe2f5ad8b5b2a485562e7c16ab6de` | focused 8/8, all Doctor 119/119, full `codefire-cli` binary 775/775, strict Clippy, format, source docs, and diff checks pass | independent Claude fallback report `84ea698b397e57a193cc5faa0ef63a0cff027e2c` says `ACCEPT`; zero-residual repair was interrupted before a diff |
| Rust file-remote ancestry Claude hedge | `e93ebc2394c8bd0398cfc23e8bf2d23d52e253b9` | focused remote 118/118, clean full-binary rerun 778/778, strict Clippy, format, source docs, and diff checks pass | independent report `ac626f65cdfa57a097bef5d757f02be6bbef409e` says `REQUEST_CHANGES`; zero-residual repair was interrupted before a diff |
| Rust HTTP MR create/review preflight Claude hedge | `71854e20689d2dd79246fa741e0e6da443d04d1a` | focused HTTP 141/141, full binary 776/776, strict Clippy/all-targets, format, source docs, cross-runtime neighbors, and diff checks pass | independent report `b1905e9383e0201b841a6efd94578f46ffa98808` says `REQUEST_CHANGES`: the product core is sound, but required test evidence is missing or not bound to the stated production boundary |

The interrupted Python MR-list worktree was resumed without discarding its diff. Background
validation that died with the worker was not accepted; manager-owned foreground runs produced the
durable results recorded above before the final clean candidate commit.

The resumed MR-list review reproduced eight of ten multi-fault precedence divergences and the
uncaught no-follow `OSError` zero-byte response on MR-list, branch-list, and Doctor. It also bound
repeated same-source escrow traversal, project-ID reopen charging, and excessive deadline-poll
overhead. Its repair packet requires deferred raw capture that preserves the legacy first-error
order, a shared typed `OSError` response boundary for GET and POST, memoized escrow validation per
session/root/source-head, one pinned descriptor read, and bounded polling.

The resumed MR-mutation review measured hidden sealed-validation work at 2.3x to 3.4x the request
ledger's charged bytes, proved the focused ledger test admitted nested non-HTTP ledgers, and bound
the replay-nonce, legacy residue, grow-tamper typing, and GC separation residuals. Its repair packet
requires one request ledger/deadline across authorization, nonce, validation, collection, and copy,
strictly forbids nested ledgers, restores legacy failure ordering without residue, consumes replay
nonces correctly, and leaves GC to its separately owned successor.

The Rust storage work was first preserved at WIP snapshot
`f5bce305da7ebfd08a840a4b281e29b0476874e0` after the Fugu session could not be resumed. The
Claude Opus 5 finisher then produced recovery checkpoint
`b3bc32a6bd4c792edc5a1302c61e16c624a9b648` and passed 779/779 CLI binary tests before the second
host restart interrupted the final strict-Clippy command. Both WIP commits are recovery evidence
only and are not integration candidates.

The four new Fugu independent-review sessions ended before producing reports after repeated Sakana
model-list response incompatibility. Their partial logs are non-authoritative evidence only.
Separate Claude Opus 5 fallback reviews are active for File Doctor, the Claude File Ancestry
candidate, Python MR mutation, and the corrected Upload/CFB-138 candidate. No candidate is accepted
from an interrupted reviewer transcript.

The File Doctor fallback review completed on exact target `52a025e…` and independently reproduced
8/8 focused, 119/119 Doctor, and 775/775 full-binary passes. Its `ACCEPT` verdict does not waive the
review's Should/Could findings under the batch-wide zero-residual rule. A new task base rooted at the
accepted candidate therefore runs
`cfr168-rust-file-doctor-zero-residual-repair-20260730` to bind the production local profile,
truthful failure phases, failure-path authority snapshots, deterministic gate fields, branch/MR
fences, extreme-scale behavior, first-error cleanup, release-build zero-debt enforcement, and the
test observer/oracle boundaries. The broader final CLI envelope/text projection remains assigned to
the command-wide response-projection residual packet after overlapping `main.rs` storage work.

The File Ancestry fallback review rejected exact target `e93ebc2…`: live upload can copy object
records before the ancestry/final-fence decision, while earlier sealed validation and object
collection still traverse the same closure outside the decision-scoped ledger. The review also
found test-only deadline evidence, missing fixture/boundary classes, helper-only scale evidence, and
ledger-loss/primary-error cleanup gaps. The repair task rooted at the rejected candidate must create
one operation-scoped validation/collection/ancestry/publication owner, move or stage all object
effects behind the final fence, bind production cancellation and all exact/1x/3x/10x evidence, and
leave a reusable accounted API for the separately tracked local-merge/clone/storage/GC residuals.

The Upload/CFB-138 fallback review verified that the 1201-commit default-profile reversal is real,
but rejected `dece688…` for three remaining live defects: publication-root validation performs two
fresh `local_v1()` sealed traversals outside the request ledger/deadline, the traversal source
constructs collection/path/descriptor owners without a reservation, and a
`128×`/`129× maximum_record_bytes` term refuses otherwise valid records at about 127 KiB. The
reviewer reproduced an elapsed 1.0 s request against a 100 ms deadline still returning 200 and
publishing. The repair lane must reuse one request-owned sealed snapshot, poll immediately before
effect, reserve every source/session owner, and derive tight retained/temporary bounds from real
lifetimes without raising the public profile.

## Recovered Python composition evidence

- The isolated composition probe is
  `probe/cfr168-python-composition` at `701469f91dd0c4c52c5628fb3e9d9e9d0ab0c714`.
- It composes Doctor (`5e3b0c7`), upload (`0703f71`), and branch session (`7e87665`) without
  replaying either interrupted worktree wholesale.
- The six adjacent `do_GET` conflicts were resolved through one shared read-route phase dispatcher,
  preserving independent Doctor and branch-list deadline flags and resetting both per request.
- `tests/test_codefire_cli.py` was corrected in `701469f`: an already-expired branch-list request
  now expects the typed 503 deadline diagnostic at `branch list planning`, poll count one, and zero
  debt instead of the pre-fix 200/no-poll behavior.
- Combined focused Doctor, upload, and branch production tests pass 42/42.
- Python-pattern discovery
  (`PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tests -p 'test_python_*.py'`)
  on the exact probe passes 463/463 in 296.500 seconds. This is a broad runtime gate, not the
  `AGENTS.md` full-suite command because it excludes `test_codefire_cli.py` and other names.
- The exact-probe full suite (`python3 -m unittest discover -s tests -v`) passes 691/691 in
  2550.503 seconds with `RUSTUP_HOME=/tmp/codefire-rustup/rustup`,
  `CARGO_HOME=/tmp/codefire-rustup/cargo`, and the configured Cargo `PATH`, so the install tests are
  non-vacuous.
- This probe is validation/input for later reviewed integration; it does not by itself Close or
  authorize CFB-138.

## Serialized integration order

Product commits are reviewed and integrated, never entire agent worktrees:

1. Build the Python spine from the reviewed product commits, not branch tips:
   Doctor `5e3b0c7`, upload `0703f71`, the accepted successor of scale delta `dece688`, branch
   `7e87665`, composition correction `701469f`, then the accepted MR-list successor. Do not replay
   duplicate recovery commits `3153d75`/`cb13da0`; their upload code/test patch is already present.
2. Resolve the adjacent `do_GET`/response-dispatch hunks while preserving one exact request ledger
   and deadline, and re-review the composed head because no isolated review proves that resolution.
3. Port the accepted Python MR-mutation successor onto that composed head instead of blindly
   cherry-picking `21fe751`; its response dispatcher overlaps the read-route spine.
4. Integrate non-overlapping Rust `remote.rs`, `remote/doctor.rs`, and `http.rs` changes.
5. Port the cumulative accepted Rust storage range including precursor `40c095d` and the final
   successor of `f5bce30`, limited to `main.rs`, `storage.rs`, `tests.rs`, and
   `tests/storage_decision_work.rs`. Never cherry-pick `f5bce30` alone or merge its 75-commit
   recovery branch.
6. Run focused tests after every product commit and broad gates after each runtime group.

## Predecessor-gated residual packets

These are still part of CFR-168 / CFB-136 but deliberately wait for a reusable predecessor or
overlapping write set:

| Residual | Predecessor |
|---|---|
| Python `GET …/merge-requests` request ledger/session | predecessor satisfied by the reviewed composition input; active batch `cfr168-python-http-mr-list-session-20260730` starts from `7327e2a8` |
| Python `POST …/gc` request ledger/deadline and pre-effect response | Python upload/MR mutation response-owner result |
| Python local `common_ancestor` / `ancestor_distances` parity | stable integrated Python ancestry and cross-runtime tie-break contract |
| Rust local `cf merge` ancestry | Rust storage task releases overlapping `main.rs` write set |
| Rust `cf scan` validation ledger/session | Rust storage task releases overlapping `main.rs`/metrics write set |
| Rust local Doctor open-summary/evidence/active-state ownership | Rust storage/session boundary and local `main.rs` integration slot |
| file clone/storage-report/GC graph walks | accepted reusable file-ancestry session plus storage integration |
| cache `invalidated`/entry-count metrics and command-wide decision-work projection, including final CLI envelope/text bytes | all selected production surfaces integrated |
| asymptotic/order table and cross-runtime corpus | stable final counters and production call graph |
| capability re-advertisement | every surface/schema/conformance gate passes |

Later roots CFR-169 through CFR-173 and CFB-133 remain predecessor-gated. Their probes are evidence,
not authorization to violate the frozen-batch-to-zero rule.

## Same-root blocker admitted during recovery

- `CFB-138` was reproduced against upload target `cb13da0`: a root plus 1200 descendants under the
  default HTTP profile is refused during `semantic_validation` because a `130,977,816` byte
  all-record heap upper is treated as concurrent retained memory against the `33,554,432` byte
  limit, although the reached retained peak is `2,274,829` bytes.
- The refusal is typed, authority remains byte-identical, and cleanup reaches zero, but the test's
  512 MiB override cannot count as CFR-168 scale acceptance.
- The fix is frozen in
  `cfr168-python-http-upload-staging-scale-20260730.md`: stream actual per-record owner lifetime,
  keep the default profile, and bind 1x/3x/10x plus exact-boundary evidence before Close.
- Claude Opus 5 is executing that packet from task base `88df84b7`; the independent upload review
  remains separate and can still reject the predecessor result.

## Mandatory acceptance sequence

The batch cannot Close until all of the following are bound to the exact integrated revision:

1. focused production-path tests and exact/minus-one/plus-one boundaries;
2. 1x/3x/10x stable counter curves and explicit complexity/order evidence;
3. full Python, Rust workspace, format, strict Clippy, demo, source docs, and project docs;
4. Rust/Python shared conformance corpus and independent High review;
5. clean-HEAD release install with provenance;
6. installed reversal tests and canonical Algorithm Dogfood;
7. issue/evidence/history synchronization and Retrospect;
8. a final ledger audit proving no CFR-168 / CFB-136 residual remains.

## Evidence

- `docs/development/ai/swarm-results/current-state-closure-audit-20260729/01-issue-ledger.md`
- `docs/development/ai/swarm-results/current-state-closure-audit-20260729/02-candidate-reconciliation.md`
- `docs/development/ai/swarm-results/current-state-closure-audit-20260729/03-acceptance-evidence.md`
- `docs/development/ai/swarm-results/current-state-closure-audit-20260729/04-cfr168-review.md`
- `docs/development/ai/swarm-results/claude-cfr168-review-20260729/review.md`
- `docs/development/ai/swarm-results/cfr168-python-http-mr-list-review-claude-20260730/review.md`
- `docs/development/ai/swarm-results/cfr168-python-http-mr-mutation-review-claude-fallback-20260730/review.md`
- `docs/development/ai/swarm-results/cfr168-review-disposition-index-20260730/index.md`

## 2026-07-30 10:49 UTC continuous-utilization checkpoint

Canonical head `63962c7` remains byte-identical to immutable product snapshot
`e2a218e015953f082245f36dbc6aee906b837e5a` across `codefire`, `crates/`,
`tests/`, `Cargo.toml`, and `Cargo.lock`. The new commits are report and task
contracts only:

- independent Rust storage report integrated as `4380985`;
- storage repair Markdown integrated as `daba568`;
- storage repair manifest integrated as `63962c7`.

The storage report binds exact candidate `b3bc32a` and returns
`REQUEST_CHANGES` for F1 through F9: ABI-dependent accounting, unreserved JSON
projection, recursive/off-by-one traversal depth, ambient-size/reopened-size
JSON reads, unbounded URL parsing, incomplete path ownership, collapsed
terminal semantics, partial/dual stdout, and a stale command trace. The
zero-residual task freezes all nine findings, every missing non-vacuous
boundary/race/publication test, and the three review questions. Its original
taskbase commits are `13767c7` and `c5f008a`.

The first newly available external slot was dispatched immediately to
`cfr168-rust-storage-zero-residual-repair-20260730` under Claude Opus 5 with a
12-hour envelope. The active external-session set is now:

| Lane | Provider/model | State |
|---|---|---|
| Python upload / CFB-138 zero-residual repair | Claude `claude-opus-5` | active from exact base `5f79899`; durable WIPs `600750c` (bounded canonical encoding) and `c33592a` (pre-allocation session control ownership) are clean checkpoints |
| Python MR-list exact rereview | Fugu `fugu-ultra-v1.1` | active against `38da898`; confirmed symlinked/replaced `branches` absence-conflation defect remains under final report analysis |
| Python MR-mutation exact rereview | Fugu `fugu-ultra-v1.1` | active against `6dd4527`; confirmed unaccounted authority reads remain under final report analysis |
| Rust storage zero-residual repair | Claude `claude-opus-5` | active from exact base `13767c7` through the registered dev-swarm batch; durable WIP `e9e42aa` records the F1-F8 repair before continued `main.rs`/`storage.rs` refinement |

The development-flow KGI/KPI repair completed cleanly at exact candidate
`0ae583e`. Manager reproduction passes 79/79 tool tests, the AI contract check,
all seven aggregate AI-development checks, project-doc validation, and the
exact diff check. This is not yet integrated: the change adds roughly 4,000
lines across semantic validation, schema, policy, record, and tests and
therefore requires a fresh exact-candidate independent review.

Additional manager probes prove the passing official gate is not sufficient:
the exact candidate accepts a one-nanosecond post-capture event, `NaN` and
`Infinity` scalar measurements, a two-node candidate cycle beside one root,
and an already-overdue active record. It also couples active records from
different batches through one global newest timestamp, accepts repository
document references whose final symlink resolves outside the repository,
accepts non-normalized agent-log locator paths, and makes a final-record digest
self-recomputable without binding it to the introducing commit. These are now
required review dispositions; `0ae583e` is not an integration candidate.
The shared-primitives repair architecture and independent regression matrix
are frozen at
`docs/development/ai/swarm-results/development-flow-kgi-kpi-manager-repair-design-20260730/design.md`.

The rereview is frozen on taskbase
`/workspace/worktrees/development-flow-kgi-kpi-zero-residual-rereview-base`
with confirmed-probe launch base `110a38c` and manifest commit `09e5911`, for
Fugu Ultra 1.1 dispatch when a review slot becomes available.

The eight already reproduced counterexamples also have a blocked successor
packet at
`docs/development/ai/task-packets/development-flow-kgi-kpi-manager-counterexample-repair-20260730.md`
and typed packet JSON, integrated as `f118262`. It binds exact-nanosecond
instants, finite JSON numbers, iterative candidate/record lineage,
operational due dates, per-batch freshness, symlink-safe repository
resolution, normalized locators, and Git-history-backed final provenance.
Its status remains `blocked`; no implementation manifest is created until the
exact Fugu rereview report and any additional findings are bound.

The next-slot policy is result-driven:

1. a completed active reviewer first yields a durable report and, on
   `REQUEST_CHANGES`, an exact repair packet before reuse of that surface;
2. a completed implementer yields an exact candidate and independent-review
   packet before any integration;
3. otherwise dispatch the KGI/KPI exact rereview, Rust ancestry repair, Rust
   File Doctor repair, then Rust HTTP MR evidence repair in dependency and
   write-collision order.

The coordinator is not a polling-only lane. While all four external slots are
occupied it owns task freezing, exact-range/write-collision mapping,
lightweight evidence reproduction, canonical report/task integration,
ready-queue maintenance, and final gate preparation.

The swarm-control improvement design is durable at
`docs/development/ai/swarm-results/dev-swarm-throughput-design-20260730/design.md`.
It specifies additive critical-path scheduling, nested light/heavy
semaphores, durable signal/checkpoint events, hedge/supersedes lineage, and
safe adoption of manually resumed second-agent runs. Direct implementation in
`/workspace/scripts` waits for usable version-control metadata in the
devcontainer foundation rather than creating unreviewable infrastructure
changes.

At `2026-07-30T10:48:57Z`, `/workspace` measured `2,887,872,262` bytes,
`MemAvailable` was `15,433,168` KiB, swap use was zero, and the mounted
filesystem reported `905,927,389,184` bytes available. This remains within
the four-session/two-heavy recovery envelope.

## 2026-07-30 11:24 UTC review-to-repair and restart checkpoint

The Python HTTP MR-write independent review completed on exact candidate
`6dd4527f23135388b53fc5c6d637c4105cc5d659`. Its report commit is
`b86a782b6ac72cabe684106f969681c5cd56380e`, and the exact report SHA-256 is
`214e42fc8f36108f45e17b13e4343bfc2b56cc89b72be5c1258f5b497c9ec6ef`.
The verdict is `REQUEST_CHANGES`. The report-only commit was integrated into
canonical as `84175bd`; no reviewed product byte was integrated.

The seven required closure areas are now frozen in
`cfr168-python-http-mr-write-review-closure-repair-20260730.md` and its
validated `dev-swarm.v1` manifest:

1. whole-request ownership of authority reads and allocations;
2. reuse of the sealed request snapshot during apply, with zero physical
   reread gap at 1x/3x/10x;
3. independent exact/minus-one/plus-one and deliberate-regression evidence;
4. prepublication sealing before any authoritative object copy;
5. retryable cleanup that cannot forget debt or replace a durable response;
6. replay nonce persistence only with reducer confirmation;
7. synchronized canonical source data-model truth.

The task packet commits are `fcb2324` and `8cd2511`; their report/task-only
canonical equivalents are `59f786c` and `1248ed2`. The taskbase remains an
additive successor of the reviewed candidate. It is ready for Claude Opus 5
when a product-implementation slot becomes available.

The Upload/CFB-138 lane advanced from `c33592a` to clean durable WIP
`0b288985aa37b23567e8e28afb6505cc54b0ce2a`, which records one
request-owned sealed traversal and a final effect fence. The lane remains
active and must still finish its focused evidence, source-document alignment,
full authorized Python gate, and exact-candidate handoff before review.

The Rust storage lane remains active at durable WIP
`e9e42aae9eb3c5a0993ca07688ade5d4d6e0eed3`. Its current uncommitted
continuation is confined to `main.rs`, `storage.rs`, and
`tests/storage_decision_work.rs`; the diff check remains manager-owned until
the next checkpoint. No partial storage range is eligible for integration.

The MR-list Fugu review remains active against exact candidate
`38da8988ac8bc1fcdfcb82c62e7aa0d28791500c`. It has not produced an
authoritative report commit, so no partial transcript is used for acceptance
or repair scope.

The first Rust file-ancestry launch inherited scheduler state from the earlier
host interruption. Reconciliation proved that its preserved worktree was
clean at exact launch revision
`86a0c2e42c15bb44df8a2ff299140fedbcff4aae` with zero task commits. Both the
interrupted state and the subsequent same-name launch-conflict state were
retained under `.dev-swarm/archive/`; neither was deleted or treated as
evidence.

The replacement manifest
`cfr168-rust-file-ancestry-zero-residual-repair-r1-20260730.json` uses a new
batch identity while preserving the exact task base and write set. Its taskbase
registration is `3611fb4`, integrated report-only into canonical as
`53b220d`. The replacement lane is now running under verified
`claude-opus-5` with a 12-hour envelope.

The same cross-state audit found the earlier File Doctor zero-residual task
stopped by the host interruption before its first implementation commit. Its
preserved worktree is clean at exact task revision
`dbeaac1526acc8c2e102ee2b137e8f2a708890b0`; the dead batch was reconciled
without deleting its state. A collision-free replacement is prevalidated as
`cfr168-rust-file-doctor-zero-residual-repair-r1-20260730.json`, registered on
its taskbase as `5231403` and in canonical as `18ddd57`. It remains queued
until one of the two heavy permits is released.

The queue audit also recovered the unclosed Rust HTTP MR create/review
preflight decision on exact candidate
`71854e20689d2dd79246fa741e0e6da443d04d1a`. Independent report
`b1905e9383e0201b841a6efd94578f46ffa98808` has SHA-256
`f30c22aa34559829254b6ca305449abc874b653c8b3dbc767248fdbbc5fe0a5a`
and returns `REQUEST_CHANGES`: the main create-order regression check remains
green when preflight is deliberately moved after observable object escrow.
The new closure packet binds that evidence gap plus success-residual truth,
response-owner fail-closed lifecycle, missing response/header boundaries,
stale dead-code claims, post-commit cleanup response stability, transactional
fallback selection, directory-complete snapshots, and unique deadline-phase
identity. Its taskbase commits are `7e4e685` and `3876e31`; canonical
report/task-only equivalents are `bdacf70` and `c6dc49a`. The task conflicts
with active ancestry work only on `work_budget.rs`, so it remains queued until
that write set and a heavy permit are released.

The external-session set is again full without exceeding the two-heavy limit:

| Lane | Provider/model | Resource class | Durable state |
|---|---|---|---|
| Python upload / CFB-138 repair | Claude `claude-opus-5` | light | active at clean WIP `0b28898` |
| Python MR-list exact review | Fugu `fugu-ultra-v1.1` | light | active on `38da898` |
| Rust storage repair | Claude `claude-opus-5` | heavy | active after WIP `e9e42aa` |
| Rust file-ancestry repair restart | Claude `claude-opus-5` | heavy | active from exact base `86a0c2e` |

Coordinator work during the full four-lane interval produced one durable
review integration, one seven-finding repair contract and manifest, one
reconciled/restarted implementation lane, and the corresponding canonical
ledger updates. This interval therefore contains no polling-only coordinator
wait.

At `2026-07-30T11:23:50Z`, `/workspace` measured `3,708,179,380` bytes,
`MemAvailable` was `15,068,104` KiB, swap use was zero, and the mounted
filesystem reported `905,087,758,336` bytes available. This remains above the
6 GiB memory and 100 GiB free-space launch floors.

## 2026-07-30 18:27 UTC four-result rollover checkpoint

All four external lanes from the prior checkpoint reached durable terminal
results. No product result was integrated automatically:

| Completed lane | Exact result | Scope | Manager disposition |
|---|---|---|---|
| Python Upload/CFB-138 repair | `76baff84cb1192e11b26dcc669fb800f7b13fa30` | 11 commits from `5f79899`; six files, `+2227/-120` | clean candidate; exact independent rereview required |
| Python MR-list rereview | report `80784151e0a42b2f839cdcf01336252319e5830a` on candidate `38da898` | one 497-line report | `REQUEST_CHANGES`; report integrated as `78b127c` |
| Rust storage repair | `cdcb9bdefe7d25a118a9dc14171864d75f5ad304` | two commits from `13767c7`; eight files, `+2972/-291` | scheduler passed and clean; exact independent rereview required |
| Rust file-ancestry repair | `02b0d63651703aaac88c249bdc437cd173dfa397` | one commit from `86a0c2e`; six files, `+2900/-815` | scheduler passed and clean; exact independent rereview required |

`dev-swarm collect` reports `all_passed=true` for storage and ancestry with
manifest SHA-256 values
`bb1fa233191f9f04dcf298c5fd950429df500480e6d4448d9470988cea3b1c30`
and
`cc04bb6d5d6d99f1e427f36a8677a3bc2df03a0f96e8335628e380cbf115e82a`.
Those scheduler results prove task completion and clean handoff, not product
acceptance.

The MR-list rereview report SHA-256 is
`6840035553871f5251e32d67bb5b2f9d6c6ee061b7ab3106e94de373ec97b20d`.
It preserves the candidate's precedence, leaf-symlink, escrow-memo,
single-session, helper-scoped descriptor, and affine-counter improvements,
but rejects seven remaining areas:

1. unavailable/symlinked/denied parent authority is treated as leaf absence;
2. filesystem `TimeoutError` is confused with client transport failure on GET
   and POST;
3. cleanup erases local ownership before a release succeeds;
4. mode/link-count changes after the last metadata observation escape;
5. authority can change after every fence but before response publication;
6. the descriptor oracle overclaims a whole-route one-read property;
7. poll evidence removes whole helpers instead of proving individual sites.

All seven plus the shared GET/POST composition handoff are frozen in
`cfr168-python-http-mr-list-review-closure-repair-20260730.md`. Taskbase
commits `7f37394` and `a38b5f2` were integrated report/task-only as `8b3fd1d`
and `fb65837`. The new implementation lane is active under verified
Claude Opus 5 with a 12-hour envelope.

Two queued heavy repairs were started immediately after the prior storage and
ancestry lanes released their permits:

| Active lane | Provider/model | Resource class | Exact task base |
|---|---|---|---|
| Rust File Doctor zero-residual restart | Claude `claude-opus-5` | heavy | `dbeaac1526acc8c2e102ee2b137e8f2a708890b0` |
| Rust HTTP MR preflight review closure | Claude `claude-opus-5` | heavy | `7e4e685efbfc453a6d2209f37c99edb1ab46243d` |
| Python MR-list review closure | Claude `claude-opus-5` | light | `7f37394c76e3626f810d0cd133be70feea89b5d2` |

The fourth external permit remains intentionally reserved for the Upload exact
Fugu rereview rather than dispatching a lower-priority review. Upload,
storage, and ancestry rereview packets are being prepared in three isolated
manager worktrees; only one Fugu review will run at a time.

At `2026-07-30T18:26:41Z`, `/workspace` measured `7,333,166,047` bytes,
`MemAvailable` was `13,397,540` KiB, swap use was zero, and the mounted
filesystem reported `901,302,951,936` bytes available. The larger workspace
measurement reflects preserved result/review worktrees and remains above all
launch floors.

## 2026-07-30 18:30 UTC exact-review dispatch checkpoint

The three manager-isolated exact-candidate review packets are now complete and
clean. They were registered in canonical without importing any candidate
product bytes:

| Candidate | Review task commits | Canonical packet commits | Queue state |
|---|---|---|---|
| Python Upload `76baff84cb1192e11b26dcc669fb800f7b13fa30` | `de63a14`, `2c0de3e` | `4537cf9`, `8a8267d` | Fugu Ultra review active |
| Rust storage `cdcb9bdefe7d25a118a9dc14171864d75f5ad304` | `844e4a3`, `dfbad64` | `0981524`, `f7bf266` | prepared; next single-Fugu review |
| Rust file ancestry `02b0d63651703aaac88c249bdc437cd173dfa397` | `8882ead`, `37b0a02` | `ecefdf2`, `ab119b0` | prepared; follows storage |

All three manifests bind a 12-hour report-only reviewer envelope and exact
candidate identity. The active Upload task is routed to Fugu
`fugu-ultra-v1.1`; its scheduler manager and reviewer process are both live.
Storage and ancestry remain deliberately unlaunched so the one-Fugu limit is
not exceeded.

The external-session set is full and remains within the two-heavy limit:

| Active lane | Provider/model | Resource class | Current purpose |
|---|---|---|---|
| Rust File Doctor repair | Claude `claude-opus-5` | heavy | close accepted-review Should/Could residuals |
| Rust HTTP MR preflight closure | Claude `claude-opus-5` | heavy | close ordering and lifecycle findings |
| Python MR-list closure | Claude `claude-opus-5` | light | close seven rejected-review findings and composition handoff |
| Python Upload exact rereview | Fugu `fugu-ultra-v1.1` | light | independent decision on `76baff8` |

At `2026-07-30T18:30:49Z`, `MemAvailable` was `15,089,623,040` bytes, swap
use was zero, and the mounted filesystem reported `901,535,006,720` bytes
available. The canonical worktree was clean before this ledger update and all
three documentation/contract gates plus diff checks passed after registering
the six review-packet commits.

## 2026-07-30 manager composition and ownership audit

Three read-only manager audits ran while the four external lanes continued.
All 13 committed independent reviews are present in the disposition index;
there is no unindexed report. The audits fixed the following composition
boundaries before any candidate integration:

### Python HTTP spine

The safe order is:

1. accepted Doctor+Branch composition;
2. exact accepted Upload successor;
3. exact accepted MR-list successor;
4. exact accepted MR-write successor;
5. final full-spine review.

Accepted Doctor+Branch tree `701469f` contains the older Upload slice
`0703f71`. It is therefore a composition scaffold, not a whole-tree
integration candidate while Upload remains undecided. If Upload is rejected,
no descendant tree that carries those bytes may be integrated wholesale.
MR-write diverges at `cab3126`; only its eventual accepted product delta may
be ported last. Every manager conflict-resolution head receives a fresh exact
review; isolated predecessor acceptance does not authorize the composed
dispatcher, session, response-owner, or final-fence bytes.

### Rust runtime composition

The preferred accepted-candidate order is:

1. file ancestry `e93ebc2` through `02b0d63`;
2. Rust HTTP MR `71854e2` through its eventual closure head;
3. storage `40c095d` through `cdcb9bd`;
4. File Doctor `52a025e` through its eventual residual-repair head;
5. one manager-owned synchronization commit for the four shared source docs.

The only current direct code overlap is ancestry and Rust HTTP on
`work_budget.rs`. Their observed hunks are line-disjoint, but both alter
ledger/deadline semantics and therefore require manual composition plus a
fresh semantic review. Storage owns `main.rs`, `secure_worktree.rs`,
`storage.rs`, and focused storage tests; Doctor owns `remote/doctor.rs`.
All four lanes overlap on the same source-document set, so agent-authored doc
tips must not be merged independently.

### Remaining predecessor ownership

The current repair/review packets do not yet close these successor surfaces:

1. Python `POST …/gc`;
2. Python local `common_ancestor` and `ancestor_distances`;
3. Rust local `cf merge`;
4. Rust `cf scan`;
5. Rust local Doctor open-summary/evidence/active-state;
6. file clone, storage-report, and GC graph walks;
7. cache invalidation/entry metrics and command-wide projection;
8. the canonical asymptotic table and cross-runtime corpus;
9. final capability re-advertisement.

File transport branch/generation/plan authority, HTTP-client Upload, and
`remote::publication::validate_object_roots` remain explicit ancestry
handoffs. Typed composition packets must bind the eventual accepted SHAs
before these successors launch.

### Canonical self-containment repair

The active File Doctor manifest's referenced Markdown contract was restored
as canonical commit `abe52d2`. The KGI/KPI exact-rereview Markdown, typed
packet, and manifest were restored by `bef89c5` through `c59b47c`; the old
`ready` repair packet was retired as a rejected-candidate task by `8b5f21f`.
Its manager-counterexample successor remains correctly blocked until the
single-Fugu CFR-168 product-review queue releases and the exact KGI/KPI report
is bound.

The audit's next-stage designs are now typed, blocked execution contracts:

| Contract | Canonical commit | Readiness blocker |
|---|---|---|
| Python HTTP composition | `53cf720` | exact accepted Upload, MR-list, and MR-write SHAs plus immutable review identities |
| Rust runtime composition | `08dced5` | exact accepted ancestry, Rust HTTP, storage, and File Doctor SHAs |
| Post-composition Close→Install→Dogfood→Retrospect | `619f439` | accepted Python/Rust compositions, every successor surface, exact installed runner and real-repository commands |

These packets contain no worker manifest and cannot launch while their inputs
are unbound. This keeps planning ahead of implementation without converting a
candidate, worker success, or isolated review into an integration decision.

## 2026-07-30 19:06 UTC terminal-cache maintenance

Resource re-audit found a `3,204,719,352`-byte per-worktree Cargo target under
the completed file-ancestry repair lane. No live process referenced that
target; exact candidate `02b0d63651703aaac88c249bdc437cd173dfa397`,
its branch, worktree, scheduler state, logs, task packet, and review packet
were already durable. Cargo-aware cleanup removed only reproducible build
artifacts and left the worktree clean.

After cleanup, `/workspace/worktrees` measured `5,265,503,056` bytes and the
mounted filesystem reported `902,231,105,536` bytes available. Active File
Doctor build artifacts were not touched. This is the intended terminal-cache
retention rule: preserve source/evidence identity, reclaim only inactive
reproducible targets.

## 2026-07-30 19:10 UTC trusted-throughput acceleration decision

The current bottleneck is not lack of occupied lanes. All four external
permits remained occupied for roughly 39--46 minutes, but none had yet
produced a newly accepted, integration-ready result. The disposition index
contains 13 independent decisions: 2 `ACCEPT` and 11 `REQUEST_CHANGES`.
Increasing implementation concurrency before changing that correction rate
would create more review debt and composition work rather than trusted
throughput.

The operating objective is therefore changed from keeping every permit busy
to minimizing packet-ready-to-reviewed-and-integration-ready lead time. Raw
agent count, patch size, commits, and transcript activity remain
anti-metrics. The following controls apply immediately to the remaining
CFR-168 recovery:

1. **Counterexample before implementation.** Freeze must bind a production
   reproducer, an independent oracle, and a deliberate-regression reversal
   for each material finding. A prose-only invariant is not implementation
   ready.
2. **Primitive before surface.** Repeated authority, ledger, deadline,
   response, cleanup, publication, and snapshot findings are repaired through
   one shared primitive and then composed into consumers. Surface-local
   variants require an explicit incompatibility reason.
3. **Review debt before new surface WIP.** When any current lane releases a
   permit, the prepared Storage and ancestry exact reviews and fresh reviews
   of completed repairs take precedence over another implementation surface.
   A rejected review first yields a bounded successor packet.
4. **Product-only handoff.** A candidate handoff contains the exact product
   delta and focused evidence. Manager task/report/control documents remain
   separate so integration does not have to untangle worker-authored control
   files.
5. **Continuous shadow composition.** Only independently accepted deltas
   enter a manager-owned composition head. Run the smallest affected
   cross-surface gate after each accepted delta; reserve broad/full gates for
   stable composition boundaries.
6. **Review findings become reusable gates.** Every confirmed counterexample
   is converted into a regression oracle and tagged by invariant family so
   later packets can preflight the same failure class instead of rediscovering
   it after implementation.
7. **Event-driven rollover.** Candidate-ready, review-decision,
   checkpoint-due, provider-retry, resource-floor, and terminal-cleanup events
   are the scheduler inputs. Manual polling is observation only and must not
   be the mechanism that releases the next task.
8. **Terminal resource lifecycle.** Preserve commits, exact reports, logs,
   and lineage; clean only inactive reproducible build targets and compress
   terminal transcripts after their immutable evidence prefix is bound.

The first bounded experiment is the next five comparable reviewable product
packets. It does not rewrite the authoritative KGI baseline. It tests these
leading outcomes:

| Signal | Experimental target | Failure action |
|---|---|---|
| material reviewer correction | at most 2 of 5 packets | strengthen Freeze counterexamples and primitive contract before adding WIP |
| review-ready queue time | below 15% of trusted-integration lead time | reserve the next released permit for review and auto-dispatch |
| manager integration time | below 20% of trusted-integration lead time | shrink product deltas and move overlap into a composition packet |
| broad/full validation | no more than one run per stable composition revision | move lane proof to focused deterministic gates |
| first durable checkpoint | before long or broad validation after mutation | require a clean WIP checkpoint before continuing |
| lost source/evidence work | zero | stop new launches and repair recovery controls |

Model/provider routing will be changed from a fixed quota to observed outcome
by task class: exact acceptance rate, trusted lead time, rework cost, and
provider continuity. Claude remains the primary bounded implementer, Fugu the
primary adversarial reviewer, and Codex the freeze/composition/final-authority
coordinator only while those measured roles remain superior. No silent model
downgrade or automatic acceptance is introduced.

Foundation automation remains the durable
`dev-swarm-throughput-design-20260730` work item. Its implementation order is
now: typed event stream and atomic run state; automatic exact-review rollover;
global resource/write-set locks; shadow integration train; generated status
views; terminal cache/log retention. It remains blocked from direct edits
until `/workspace` has usable version-control metadata.

## 2026-07-30 19:21 UTC first review-first rollover

The File Doctor zero-residual repair completed with scheduler
`all_passed=true` on exact candidate
`1a04490806b59f8c39b5c1cf03f42d06aa750a98`. Its exact task base is
`dbeaac1526acc8c2e102ee2b137e8f2a708890b0`, candidate tree is
`eafb2cbf20980944d16086a972100c321a8b56be`, manifest SHA-256 is
`db98dc6dd28218522bafe9ce80fac7f7aeadae527ace2cdd1ea977c9b5e45e55`,
and the one-commit changed set is `remote/doctor.rs` plus four shared source
documents. The worker worktree is clean. This proves a candidate handoff, not
acceptance.

The released external permit was not used for another implementation
surface. A report-only Claude Opus 5 exact rereview was frozen at task commit
`cb561908fe97e10faaa927946ccd051805be1bd4` and registered in canonical as
`92d80c7`. It binds the candidate tree, binary patch digest, prior report and
repair-contract digests, all four Should and four Could residuals, production
reachability, and deliberate-regression evidence. The review is active under
batch
`cfr168-rust-file-doctor-zero-residual-rereview-claude-20260730`.

This is the first live application of the trusted-throughput decision:
candidate-ready to reviewer start occurred in the same manager interval, no
new surface WIP was opened, and no candidate product byte was copied to
canonical before an exact decision.

After the implementation manager and every child process terminated, the
completed File Doctor worktree had a private
`2,327,617,292`-byte Cargo target and no process with a cwd beneath that
worktree. `cargo clean` with the pinned toolchain and explicit manifest
removed 2,256 reproducible files; source, branch, scheduler state, logs, and
the exact candidate remained clean and intact. `/workspace/worktrees` then
measured `2,977,396,538` bytes and the mounted filesystem reported
`904,444,588,032` bytes available. Active Rust targets were not touched.

## 2026-07-30 19:48 UTC File Doctor accepted rereview and note closure

The report-only File Doctor exact rereview completed with scheduler
`all_passed=true` at report commit
`1c555ff3f426d7fa6874db92ceb71515a8d908e6`. Its exact subject is candidate
`1a04490806b59f8c39b5c1cf03f42d06aa750a98`; the report SHA-256 is
`f7c52b05e312cc131fdfc2e2c0d9d486f5ee372b5ec93a85fdb44a95fb3ce1b9`,
and the reviewer manifest SHA-256 is
`b4672384b3552f519fd7ae74bb53007b589430bab99c25431e161192a3223247`.
Only the immutable report was integrated into canonical as `c22a78a`.

The verdict is `ACCEPT`: all four inherited Should and four inherited Could
items pass production reachability, 20 focused cases, 787 binary cases,
strict Clippy, release-build enforcement, and eight deliberate regressions.
The report also records three new notes:

1. a fail-closed production panic if failure cleanup cannot remove an open
   permit or ownership invariant violation;
2. digest/path projection allocations occur immediately before their fence
   reservation; and
3. metadata/read-unavailable records are not fenced against becoming readable
   before publication.

The accepted verdict is retained as exact evidence, but the batch-wide
zero-residual goal does not infer those notes away. A successor packet binds
N1 as an explicit typed-failure-versus-abort decision and requires product
repairs plus deliberate regressions for N2 and N3. Its task and manifest
commits are `1e5a6f9` and `a1e3b52`, registered in canonical as `3a0bc67` and
`0145cc4`. Claude Opus 5 is active under
`cfr168-rust-file-doctor-review-note-closure-repair-20260730` with the shared
Cargo target, two build jobs, two test threads, and incremental compilation
disabled. No File Doctor product candidate has entered canonical.

After the reviewer terminated, its private default target measured
`1,590,727,568` bytes. No process had a cwd beneath the reviewer worktree;
Cargo-aware cleanup removed 1,614 reproducible files while preserving the
report commit, source candidate, logs, and clean state. `/workspace/worktrees`
then measured `3,007,461,624` bytes and the mounted filesystem reported
`904,419,512,320` bytes available. Active Rust targets were not touched.

## 2026-07-30 19:54 UTC Rust HTTP repair handoff and exact rereview

The Rust HTTP MR preflight closure repair completed with scheduler
`all_passed=true` on exact candidate
`73bc5327ff6d750ce95f63333055ee39b693cb8c`. Its parent/task revision is
`7e4e685efbfc453a6d2209f37c99edb1ab46243d`, tree is
`430f2e2ccf9dca485575e7f9db0b639390d7e761`, manifest SHA-256 is
`478a17fc4d9a1a3f72ecd5404f1389c713063f985100bfb00eb6d4f99abedcc0`,
and binary patch SHA-256 is
`216c12f8d9e0c6939bf6889856a999e8f45b56237eca9ff63cdef34105ef268d`.
The exact changed set is `http.rs`, `work_budget.rs`, and the four canonical
source documents. The worker reported the focused 788-case CLI target,
89-case core target, strict Clippy, format, and source-document gates green.
Those are worker evidence, not an acceptance decision.

The lane created intermediate WIP commits and then rewrote them into the
single delivered commit despite the repair packet's additive/no-rewrite
instruction. The final parent, tree, patch, and clean state are exact and
reviewable, but the process deviation is not inferred away. The fresh review
must reconcile whether any intended product bytes or evidence disappeared and
must report the deviation explicitly.

The released implementation permit was rolled directly into a report-only
Claude Opus 5 exact rereview. Its packet and manifest were committed as
`5671977` and `49a977a`, registered in canonical as `ced46da` and `d9061c6`,
and started under batch
`cfr168-rust-http-mr-preflight-zero-residual-rereview-claude-20260730`.
The review binds F1--F5, R1--R4, production call-graph reachability, deliberate
regressions, exact response bytes, simultaneous ownership peaks, and WIP
provenance. It may write only its immutable report. No Rust HTTP product byte
has entered canonical.

At launch, the resource floor was healthy: `MemAvailable` was approximately
13.3 GiB, swap use was zero, and the workspace filesystem had approximately
843 GiB available. Four external lanes are now occupied: Upload exact review,
Python MR-list repair, File Doctor note repair, and Rust HTTP exact review.
The manager continues control-plane, composition, and evidence work rather
than polling idly; the next released Fugu permit is reserved for Storage exact
review.

## 2026-07-30 19:55 UTC trusted-throughput Stage 5 activation

A fresh manager audit counted 14 indexed exact review reports: 3 `ACCEPT` and
11 `REQUEST_CHANGES`. Accepted review wording has not yet produced one
composition-revalidated product integration on the canonical recovery head.
Four external permits were already active, so adding writers would increase
review debt rather than shorten the critical path.

The measured baseline, topology, WIP limits, next-five-packet targets, and
automation order are now durable in
`docs/development/ai/swarm-results/cfr168-trusted-throughput-audit-20260730/audit.md`.
The acceleration master plan has an active Stage 5. Immediate controls are:
Fugu `READY/BLOCK` counterexamples before implementation, one writer per
conflict domain, review-waiting WIP bounded by reviewer permits, exact
candidate freeze, original-implementer continuation after a rejecting report,
product-only shadow composition, and at most one broad gate per stable
composition revision.

The compatible foundation implementation is already partitioned in the
`dev-swarm-throughput-design-20260730` design. A read-only recovery audit
confirmed that `/workspace/.git` is an empty directory, no foundation remote
identity is present, and GitHub CLI is unauthenticated. The manager will not
initialize an unrelated history or mutate unversioned scheduler code. Atomic
run state, resource leases, event telemetry, auto-review rollover, validation
cache, and shadow-train automation remain ready to implement as soon as the
actual foundation Git provenance is restored.

The first Stage 5 shift-left packet is committed as `ba8dd25`. It blocks the
prepared MR-write implementation until exact accepted Upload and MR-list
identities are bound and an independent Fugu 1.1 Ultra report returns
`READY`. The report must reconcile one request context, sealed snapshot,
authority fence, prepared response, retryable cleanup, publication
transaction, and error-origin model through physical owner/lifetime and
effect timelines. Every MR-write F1--F7 claim requires a production oracle and
deliberate regression before Claude implementation begins. The packet has no
worker manifest while blocked, so it adds no live WIP or provider cost.

## 2026-07-30 20:20 UTC residual review-unit decomposition

The manager corrected stale root-routing rows without changing immutable
review reports. Python Upload `76baff8` and Rust HTTP `73bc532` are active
exact rereviews, MR-list and File Doctor NOTE-1--NOTE-3 are active repairs,
and Storage `cdcb9bd` plus Rust file ancestry `02b0d63` are completed
candidates waiting for their exact reviews. MR-write is explicitly blocked
on accepted Upload/MR-list identities and a committed shared-primitives Fugu
`READY`; it is no longer described as merely queued behind composition.

A read-only parallel audit confirmed that S1, S2, S3, and S5 had no dedicated
current Markdown/typed-JSON successor. The post-composition packet named
their outcomes but forbade product implementation, while the older generic
decision-work packet predated the findings. Leaving the gap until composition
would add another planning and queue delay at the end of the critical path.

The residual is now frozen as three dependency-ordered, blocked packets:

1. `cfr168-final-decision-work-metrics-projection-freeze-20260730` owns S1,
   S2, and functional S5 under one counter/schema/CLI conflict domain;
2. `cfr168-final-complexity-conformance-freeze-20260730` owns S3 and the
   shared Rust/Python corpus after exact metrics acceptance; and
3. `cfr168-final-capability-readvertisement-freeze-20260730` owns the
   declaration-only S5 tail as the separately accepted last product mutation.

The split avoids both four independent schema writers and one oversized
candidate. Each packet starts blocked with no manifest, exact predecessor
bindings, a Fugu 1.1 Ultra `READY/BLOCK` preflight, one Claude Opus 5 writer,
an independent exact Fugu review, deliberate regressions, durable stage
events, and a manager-owned shadow integration. Complexity/corpus changes
cannot repair product behavior, and capability changes cannot add behavior or
reuse pre-mutation lifecycle evidence. This prepares the work without adding
live WIP, provider cost, or a third heavy lane.

## 2026-07-30 20:23 UTC manual Stage 5 enforcement boundary

A read-only scheduler audit separated implemented behavior from the Stage 5
design. The current scheduler provides per-manifest DAG/worktrees,
per-batch provider limits, declared within-batch write-set checks,
status/collect/stop/reconcile, and a process-level postflight. It does not
provide global total/heavy leases, critical-path ordering, durable candidate
or checkpoint events, run adoption, exact review rollover, typed gate
evidence, validation cache, shadow integration, or event-derived KGI/KPI.
`passed` therefore remains process/postflight evidence and never means
candidate or review acceptance.

More importantly, the v1 loader can retain proposed fields without enforcing
their safety semantics. The throughput design now reserves all Stage 5 fields
for fail-closed `dev-swarm.v2`; v1 keeps historical behavior but must reject
unknown and Stage 5 fields when the foundation repair is implemented. An old
v1-only binary will reject v2 instead of silently accepting inert resource,
progress, continuation, gate, or integration controls.

Until usable foundation Git provenance is restored, the active manual
contract is `docs/development/ai/stage5-manual-control-runbook.md`. It freezes
one portfolio board across all batches, total four/heavy two leases, manager-
recomputed candidate cards, blocked review manifests prepared during
implementation, exact clean implementer continuation, deterministic
focused-gate evidence keys, accepted-product-only shadow composition, stage
events, resource floors, and trusted-throughput metrics. No `/workspace`
script was changed and no live lane was added.

The CodeFire-side AI checker now rejects known Stage 5 keys at any nesting
depth in `dev-swarm.v1` batch manifests. Two negative/compatibility tests and
the full 44-test AI-tool suite pass. This guard cannot replace the future
foundation parser's recursive unknown-field rejection, but it makes the
active invariant of zero inert Stage 5 fields mechanically enforceable now.

## 2026-07-30 20:27 UTC Rust HTTP exact review rollover

The Rust HTTP exact rereview terminated cleanly with report commit
`28eb5a94e2f452740df7e8c5b64a8a8f3b9646b0`, report tree
`41369845a91a005b50c6d499b1c757c5990f1559`, and report SHA-256
`8bb1a418ce6db1aa932eca2d19990212a58a5f44ad42eb3ccd439e68b4f1417d`.
Only the immutable report changed; canonical integrated it as `4b11ef9`.

The decision is `REQUEST_CHANGES`. The review independently closes F1--F5
and R1--R4 product behavior and found no shipped-behavior defect. Its blockers
are a vacuous 1x/3x/10x test that adds unreachable objects so no counter moves,
and a durable evidence claim of 57 review-success bytes when the exact
production fixture measures 60. Under the batch-wide zero-residual goal, the
two non-blocking comment/suppression findings and local residuals R-1--R-5 are
also included. Cross-runtime 149-byte parity R-6 is routed to the final
complexity/conformance corpus; snapshot mode/ownership R-7 remains a
file-authority successor.

The manager froze additive repair task `e3b005a` and registered it in
canonical as `f92dab4`. Instead of opening another large rewrite, it resumed
the exact original Claude Opus 5 implementer session
`8d2b5b6e-54d7-4151-84bb-0820ea5d1efb` on its clean worktree at reviewed
candidate `73bc532`, after the prior manager/task had terminated. The launch
uses the pinned shared Cargo target, two build jobs, two test threads,
incremental compilation disabled, and a 12-hour progress lease. The provider
command includes `-r 8d2b5b6e-…`, proving actual session continuation.

Four external sessions are occupied again: Upload exact Fugu review, Python
MR-list repair, File Doctor note repair, and this Rust HTTP repair. Heavy
occupancy is two, so Storage remains queued behind the single Fugu review
permit and the next heavy release. No third heavy lane was started.

## 2026-07-30 20:34 UTC prepared exact-review rollover

Three fresh Claude Opus 5 report-only review tickets are now frozen before
their implementation candidates finish:

- `cfr168-python-http-mr-list-review-closure-rereview-claude-20260730`;
- `cfr168-rust-file-doctor-review-note-closure-rereview-claude-20260730`; and
- `cfr168-rust-http-mr-preflight-evidence-closure-rereview-claude-20260730`.

Each ticket already fixes the independent reviewer role, exact writable
report path, inherited findings, counterexamples, mutation matrix, broad-gate
ownership, and strict `ACCEPT`/`REQUEST_CHANGES` boundary. Candidate SHA,
parent, tree, cumulative patch digest, changed paths, result digest, and clean
state remain literally `UNBOUND`. The manager must bind every value
atomically from a clean completed candidate and generate the candidate-ready
card before creating a runnable manifest. A reviewer must then recompute the
identity rather than trust the card.

No manifest was created, no external permit was occupied, and no provider
cost was incurred by this preparation. This moves review specification work
off the candidate-ready critical path without creating speculative product
WIP or allowing a branch tip to become the review subject.

## 2026-07-30 20:43 UTC candidate-identity automation

The repeated manager handoff step is now automated by
`docs/development/tools/capture_candidate_identity.py`. One read-only command
recomputes the task base, merge base, parent/head/tree, additive and merge
commit lists, canonical binary patch SHA-256, sorted
name-status/numstat/base-and-head blob identities, checkout HEAD, clean
tracked/untracked state, manifest/ticket digests, and declared write-set
containment. A dirty, moving, empty, merged, wrong-base, or out-of-scope
candidate emits `BLOCK` and exits 2; worker prose cannot override it.

The helper was exercised against the live clean MR-list worktree while its
manager remained active. It independently resolved six additive commits,
tree `17cb396ee30a887be6698b822b60f52ac2adb67c`, patch SHA-256
`6a9ca3d1046ba5ec213645c0710f900425191269360a70a3d5c4a50090528543`,
six changed paths, exact manifest/ticket digests, and no write-set escape.
This is a provisional identity observation only: the candidate is not frozen
until the worker and scheduler are terminal, so a later HEAD must be
recomputed rather than inheriting these values.

Three focused fail-closed tests cover a clean linear candidate, dirty state,
and a committed path outside the manifest write set. The complete AI-tool
suite now has 47 passing tests. The Stage 5 runbook invokes the helper
directly, and the acceleration plan records AIA-038 as implemented.

## 2026-07-30 20:45 UTC MR-list candidate-ready rollover

The Python HTTP MR-list closure repair terminated with scheduler
`all_passed=true` on clean candidate
`4930d25178be11e064060487f8c254802b09868d`. The exact task base and merge
base are `7f37394c76e3626f810d0cd133be70feea89b5d2`, its parent is
`2045955951d595d05602aa31f3cdae2596f6054a`, tree is
`17cb396ee30a887be6698b822b60f52ac2adb67c`, and cumulative canonical binary
patch SHA-256 is
`6a9ca3d1046ba5ec213645c0710f900425191269360a70a3d5c4a50090528543`.
The range contains six additive commits, no merge, no rewrite, six changed
paths, and no declared-write-set escape.

The candidate-ready card is committed at
`swarm-results/cfr168-python-http-mr-list-review-closure-repair-candidate-ready-20260730/candidate.md`.
It binds exact name-status, numstat, base/head blobs, implementation
manifest/ticket digests, scheduler result, provider/model/role/session, and
three terminal log digests. Worker evidence includes ten consecutive 44-case
focused passes, 115 affected legacy CLI passes, the stable 507-case Python
gate, source/project doc checks, exact reversals, and a clean final worktree.
These remain worker evidence, not acceptance.

The prebuilt review ticket was atomically bound with no `UNBOUND` candidate
field. Review task commit `8143f437342cf7bf11322d82f6ca63ab888382e8`
sits immediately above the immutable candidate; manifest commit
`0c13d1339178118890685dc94bf9d79c4ccec8e1` sits above the ticket. The
report-only manifest SHA-256 is
`2a306dba5102da788b3e2ec4cefc258e72a638a1f098bb790f2b1a301f58d45b`;
`dev-swarm validate` and `plan` pass. Its only write is the fresh Claude Opus
5 review report. This is the first live use of AIA-038 from terminal
candidate to bound exact-review task.

At `2026-07-30T20:49:24Z` the released light permit started that review under
manager PID `1055467`, fresh Claude session
`dd97a22f-f9c0-4f89-a159-6b6c1e2ca8d6`, and a new isolated worktree. The
implementer session was `3dd468cb-d9a9-42be-b266-befde4fba6fd`, so session,
agent, branch, role, and worktree independence are all explicit. Candidate
card capture to verified reviewer start was under five minutes. Total
external occupancy returned to four while heavy occupancy remained two.

The manager also found a resource-contract omission in the two remaining
prepared Rust review tickets. The active File Doctor worker selected
`/tmp/codefire-rustup`, which currently duplicates about 727 MiB of toolchain
state; it still uses the shared Cargo target and remains healthy, so it was
not interrupted. The future File Doctor and Rust HTTP exact-review tickets
now pin the shared Rustup/Cargo homes, shared target, two build jobs, two test
threads, disabled incremental compilation, and serialized Cargo. They
explicitly forbid a worktree-local target or another toolchain home. The AI
checker now rejects any active CFR-168 Rust packet with Cargo validation that
omits one of those exact profile markers; its focused negative/positive test
and the complete 48-test AI-tool suite pass.

The active MR-list reviewer independently reproduced all candidate blobs and
the patch digest, but first spent time discovering that the new identity
helper was not present in the older candidate-derived taskbase. This does not
change the exact subject or review validity. It exposes a rollover packaging
cost: future review task commits now carry the manager-vetted helper blob when
the candidate predates it, and cards bind its SHA-256. The current canonical
helper digest is
`ef548dd9a0c84ff617e2553110b0f3e25d68c0c2eb1857275e684ee561cfbd02`.

The two prepared Rust reviews now also separate non-cacheable independent
falsification from deterministic broad gates. Identity, live routes, focused
debug/release cases, and every deliberate mutation must be rerun. Full
CLI/core, strict Clippy, and format may be reused only through a
manager-generated key binding the exact tree, argv, shared environment,
toolchain, Cargo lock/config, fixtures, terminal pass, and complete log
digest. This removes duplicate stable compilation when evidence is exact
without turning worker prose or a mutable log filename into acceptance.

AIA-038 now covers the other side of rollover as well. In report mode it
requires the selected manifest task to be a reviewer, requires every declared
report-only output to exist in the committed range, and records committed
blob bytes and SHA-256. Wrong role, missing report, extra path, dirty state,
moving HEAD, merge, or wrong base remains `BLOCK`. Verdict interpretation is
deliberately not automated, so scheduler success can never become `ACCEPT`
without reading the exact immutable report.

The validation report template now has a compact exact-review form. It binds
identity, required finding disposition, owner/effect/fence, mutation,
content-addressed validation, integration overlap, unavailable evidence, and
one terminal `OVERALL_VERDICT` line. Prepared File Doctor and Rust HTTP
reviews require this form. This targets report synthesis and manager parsing
time rather than reducing any counterexample, mutation, or gate requirement.

## 2026-07-31 01:38 UTC wall-clock discontinuity and monotonic gate evidence

The host/container wall clock advanced by roughly 4 hours 50 minutes during
active validation while process-monotonic time remained continuous. Concrete
evidence exposed the KPI hazard:

- the Upload reviewer selected 13 legacy tests; unittest reported 13/13 in
  202.701 seconds, while a `date +%s%3N` wrapper reported 16,319.902 seconds;
- the MR-list reviewer broad gate reported 507/507 in 407.631 seconds, while
  shell `time` reported 275 minutes 28.791 seconds; and
- `timeout`-controlled agent processes remained healthy because their
  enforcement uses monotonic elapsed time.

No gate was rerun merely to make the wall duration look plausible. Gate
result/count and internal monotonic duration remain evidence; wall-derived
lead-time intervals crossing this event are unavailable until reconciled.

`docs/development/tools/run_timed_gate.py` now runs one argv without a shell,
uses a new process group and monotonic timeout/duration, binds complete log
bytes/SHA-256, writes optional JSON atomically, and reports wall-minus-
monotonic drift explicitly. A two-second drift threshold prevents wall time,
mtime, or provider timestamps from silently contaminating gate and flow KPI
durations.

## 2026-07-31 01:55 UTC fixed occupancy cap superseded

The four-external/two-heavy values recorded earlier in this ledger were
incident-recovery guards after host memory and disk exhaustion. They were not
derived from a saturation experiment and are no longer launch limits. At the
supersession audit the host exposed:

- 14 effective CPUs with no cgroup CPU quota;
- no cgroup memory ceiling, `13,540,003,840` bytes effective available
  memory, and zero swap use;
- CPU some PSI avg10 `0.36`, memory full PSI avg10 `0.00`, and I/O full PSI
  avg10 `0.00`;
- `903,601,012,736` bytes free on the workspace filesystem; and
- 23,168 cgroup PIDs of headroom.

Four external lanes were live: three Claude Opus 5 and one Fugu 1.1 Ultra.
The three healthy Claude lanes already disproved treating the routing
matrix's initial Claude window of two as a physical limit. Historical 529
retry evidence still requires provider backpressure control.

`docs/development/tools/check_swarm_admission.py` now evaluates each candidate
from its reservation, any recently admitted outstanding reservations,
projected effective memory, swap, disk, CPU/memory/I/O PSI, PID headroom,
provider window/backoff, and conflict ownership. It has no total-session
input. Initial Fugu/Claude values are provider congestion windows that grow
one lane at a time on clean evidence and contract after backpressure; Codex
remains one manager lane for cost control.

The exact Storage rereview candidate
`cdcb9bdefe7d25a118a9dc14171864d75f5ad304` passed `rust-heavy` admission
with a 4 GiB reservation: projected available memory was
`9,245,036,544` bytes, Fugu occupancy was 1/4, and every pressure/floor check
passed. The first scheduler launch created no commit or candidate change and
exited in five seconds because its manager environment did not inherit
`SAKANA_API_KEY`; this is provider-readiness plumbing, not capacity
backpressure.

The manager then reused the exact clean agent/worktree and inherited the
already-active key without printing it. Storage review session
`019fb5e4-1fc2-7572-a3f6-06231f25ade7` is live under Fugu
`fugu-ultra-v1.1` (provider PID 1151557). This is the fifth simultaneous
external lane and the first concrete execution after removing the arbitrary
total-four and single-Fugu queues. A sixth burst launch remains subject to
the outstanding 4 GiB reservation until Storage reaches measured steady
state; the same apparent free memory cannot be admitted twice.

## 2026-07-31 02:02 UTC current bottleneck audit

The active bottleneck is trusted correction flow, not raw agent count or
provider permits.

| Rank | Bottleneck | Direct evidence | Control |
|---|---|---|---|
| 1 | Review-to-repair recurrence on the Python critical path | Upload and MR-list focused/broad gates are green, but independent production probes found new request-ledger/deadline, final-fence, error-origin, prepared-owner, and success-with-debt defects. Upload acceptance blocks MR-write and Python composition; MR-list acceptance blocks the same train. | Freeze the immutable reports, generate one counterexample-first closure packet per shared root, and resume the exact original implementer session/worktree. Do not create another local owner fork. |
| 2 | Shared lifecycle primitives are still duplicated across routes | Upload and MR-list independently reproduced the same classes of request-ledger, response-owner, cleanup, and post-fence failures despite route-local tests. | Bind the shared-primitives preflight before MR-write, then extract stable request/session/authority/response/cleanup owners after accepted composition. |
| 3 | Verdict-to-report/rollover latency | Both active reviewers settled `REQUEST_CHANGES` before their required evidence matrices and immutable report commits were complete. Until the commit exists, the manager cannot bind a safe repair continuation. | Use the compact exact-review template, one terminal verdict line, manager-computed identity, and prebuilt blocked repair/review tickets; measure decision-settled to report-commit time separately from review time. |
| 4 | Rust validation conflict and launch ramp | File Doctor and Rust HTTP are simultaneously running Cargo gates, while Storage has a newly admitted 4 GiB reservation. The next light launch projects only `5,715,308,544` bytes available, below the 6 GiB floor, until Storage load reaches steady state. | Keep Cargo at two jobs/two test threads with one shared target; release the outstanding reservation only from measured load or termination, then re-evaluate File ancestry. |
| 5 | External scheduler readiness/recovery seams | Storage's first scheduler launch exited in five seconds because `SAKANA_API_KEY` was absent from the manager environment although an active Fugu lane had it. `dev-swarm.v1` retained a terminal failed state and could not adopt the clean manual exact-session restart. | Foundation repair must provide secret-preserving provider readiness inheritance, typed preflight failure, clean retry, and exact manual-run adoption. Until foundation Git provenance is restored, record the manual PID/session and never expose the key. |

The 02:02 resource snapshot shows 14 CPUs, load1/CPU `0.282`, CPU some PSI
avg10 `0.07`, memory and I/O full PSI avg10 `0.00`, zero swap, and about
841 GiB workspace free. Fugu occupancy is 2/4 with no provider backoff.
Therefore CPU, disk, swap, and provider-window capacity are not current
bottlenecks. The temporary memory block is intentional burst protection, not
a return to a fixed session cap.

## 2026-07-31 02:10 UTC MR-list exact rejection and closure rollover

The fresh Claude Opus 5 reviewer terminated normally with report-only commit
`0aa0d3d0bfbaafcc9d67bccdc90207810a60fe4b`. Manager recomputation proved:

- exact report task base `8143f437342cf7bf11322d82f6ca63ab888382e8`;
- one additive commit and no merge;
- exact write set containing only
  `cfr168-python-http-mr-list-review-closure-rereview-claude-20260730/review.md`;
- clean checkout and report tree
  `4514e38ebc122421612136c009e596dd8df633c2`;
- report blob SHA-256
  `dc4f66ba9649e4333980996c6f83d7f28b0f5f4bd165683adac55bf6d9704b2e`;
- report patch SHA-256
  `2848022a51e55fc449dafacacb3349ca4177387eb9b1176d6616d60a0403d7ce`;
  and
- scheduler collection `all_passed=true`.

Canonical integrated only the immutable report as `909978c`. The decision is
`REQUEST_CHANGES`: all seven inherited findings are closed, but F1--F3 prove
post-fence fallible teardown, a filesystem error discarded behind an older
prepared `200`, and an inert publishability guard that permits success with
retained debt. The zero-residual successor also owns the review's write-only
cleanup evidence and silent defensive over-release observations.

The exact original implementer branch remained clean at reviewed candidate
`4930d25178be11e064060487f8c254802b09868d`, with Claude session
`3dd468cb-d9a9-42be-b266-befde4fba6fd`. The manager appended the immutable
report as `6ee3646` and counterexample-first closure packet
`06e519e`, integrated packet-only in canonical as `6ab6383`. No new
implementation session or repository-orientation cost is needed.

Immediate continuation was correctly held by adaptive admission: with active
Rust gates and Storage's not-yet-reflected 4 GiB reservation, even a measured
2 GiB MR-list reservation projected only `4,081,827,840` bytes available,
below the 6 GiB floor. Provider window, swap, disk, PSI, PID, dependency, and
write-conflict checks all passed. Resume is the first action after a heavy
gate releases or Storage reaches measured steady state; the exact session,
worktree, report, and packet are already bound.

## 2026-07-31 02:17 UTC bottleneck follow-up and MR-list continuation

The disposition index now contains 16 exact independent reports: 3 `ACCEPT`
and 13 `REQUEST_CHANGES`, an exact-candidate acceptance yield of 18.75%.
This is not a model score and several reports deliberately review successive
zero-residual candidates, but it confirms that the critical-path constraint
is correction yield rather than unoccupied provider capacity.

Two execution-control defects became concrete:

- the active Upload reviewer had run for about 3 hours 15 minutes and had
  emitted repeated decisive/“report ready” signals for more than 20 minutes,
  while its exact report path and worktree remained absent/clean. The findings
  are useful, but downstream repair cannot be bound until an immutable report
  commit exists. This is the live AIA-043/AIA-045 terminalization bottleneck;
- Rust HTTP and File Doctor launched private release targets despite the
  shared low-memory profile. At the snapshot, the shared target was about
  1.6 GiB, `/tmp/reg-target` about 560 MiB, and the File Doctor target about
  150 MiB, with three compile streams and one Rust compiler above 1.2 GiB RSS.
  No healthy near-terminal gate was interrupted, but prompt-only profile
  guidance is now proven insufficient and is tracked as AIA-044.

Storage had completed its Cargo gates and reached a reasoning/reporting steady
state, so its bootstrap-only 4 GiB outstanding reservation no longer described
unreflected load. A fresh count-free admission for the measured 2 GiB Python
MR-list continuation returned `ALLOW`: projected effective available memory
`7,950,061,568` bytes, swap zero, memory/I/O full PSI zero, about 841 GiB disk
free, no conflict owner, and Claude occupancy 2 of the current adaptive window
3. The manager therefore resumed exact Claude Opus 5 session
`3dd468cb-d9a9-42be-b266-befde4fba6fd` in its original clean worktree from
packet `06e519e`; provider PID 1223570 confirmed the same session identity.
This restores five concurrent external lanes without a fixed global count and
without opening another implementation fork.

## 2026-07-31 02:42 UTC Upload rejection rollover and fourth Claude admission

The Upload Fugu reviewer committed its immutable report as
`35983261e1abd32045e1ff6cbd7dd7dfff5e806d` on exact task base
`de63a148c199ade7a05d3d7b68b60ef6af0d576a`. Manager recomputation fixed:

- exact reviewed candidate
  `76baff84cb1192e11b26dcc669fb800f7b13fa30`;
- report tree `c95ec4ef04cbcc1da8bbd8a6762807b8555783ca`;
- report blob `1fd3de9b48e04d29b56f36d79584a0a03c22bcff`;
- report SHA-256
  `7632552eb0ca60addca0d83fe3ba53ead642a7cbee82937b1b7b37754f6cb2cf`;
- report patch SHA-256
  `30529b8c321329323bfb30799a129905de679206742e8cde1f6778116dc4aa0`;
- one additive report-only commit, clean checkout, and exact write-set
  containment.

The decision is `REQUEST_CHANGES`. BLOCKER-1 proves completed-quarantine
recovery silently replaces the HTTP request ledger/deadline. HIGH-2 proves
caller-owned sealed validation cannot acquire its ordered projection.
HIGH-3 proves the publication-root proof can be consumed twice before revoke.
MEDIUM-4 proves the projection/coverage/layout-poll reversal evidence is
absent or vacuous. The disposition index now contains 17 exact reports:
3 `ACCEPT` and 14 `REQUEST_CHANGES` (17.65% exact-candidate acceptance yield).

The reviewer continued after its report commit. AIA-045 classified the
committed-report/live-worker state as an intervention, the manager preserved
the exact session and transcript, allowed active child output to flush, and
stopped the scheduler without creating a competing reviewer. This converted
the earlier verdict-to-report loop into a durable repair input.

The original implementer lineage now carries the report as `fe7cf943` and the
counterexample-first repair packet as
`b19c4346c0437ed28bdfe2fb6af1dd466abe9f8b`; canonical registered the packet
as `e3df2a3`. A fresh count-free `python-heavy` admission with the measured
2 GiB reservation returned `ALLOW` at `2026-07-31T02:41:41Z`: projected
available memory was `10,185,891,840` bytes, swap use and memory/I/O full PSI
were zero, workspace free space was `903,168,794,624` bytes, Claude occupancy
was 3 in an adaptive window of 4, and no provider backoff or write conflict
was present.

The manager therefore resumed exact Claude Opus 5 session
`7baa1275-0ee2-476a-af46-1907386ebceb` in the original clean worktree at
provider PID `1318933`. This is a fourth concurrent Claude lane admitted from
measured pressure rather than a fixed global count. It closes the review to
repair idle gap without paying for another implementation orientation.

## 2026-07-31 03:06 UTC Storage rejection rollover and source-target quarantine

The Storage Fugu reviewer had already emitted repeated report-ready and
verdict signals without a durable report. AIA-045 classified the exact
session as `INTERVENE`: 179 agent-text events, 149 events after the first
ready signal, 135 after the first verdict signal, seven ready signals, five
verdict signals, a live worker, and no report path. The manager stopped only
the looping process group after confirming that no useful Cargo command was
active, then resumed the same Fugu Ultra 1.1 session
`019fb5e4-1fc2-7572-a3f6-06231f25ade7` in report-only mode.

That exact session committed only
`cfr168-rust-storage-zero-residual-rereview-fugu-20260730/review.md` as
`71222e1cec1b0dc10003845428abfb611ba8bf71`. Manager recomputation fixed:

- reviewed candidate `cdcb9bdefe7d25a118a9dc14171864d75f5ad304`;
- report task base `844e4a319e458e6be2507ab8b166640a74d98c85`;
- report tree `d43b5a1da31aba03943cdfe6c38c17fd5438511b`;
- report blob `cd0ffc8458d8240ac7a0215a365cf32b5b91e7da`;
- report size `54,661` bytes and SHA-256
  `6438df445b2d05510895fd23f73a4e71ba49238da15b00e86abf1a477b5ab960`;
- one additive report-only commit, no merge, clean checkout, and exact
  write-set containment; and
- one machine-stable `OVERALL_VERDICT: REQUEST_CHANGES`.

The report contains six High, seven Medium, and one zero-residual
release-blocking finding. All F1 through F9 remain open. The material roots
are phantom EOF graph nodes and missing local edges, release-build
visitor-versus-Store error erasure, publication failure recorded as
`Complete`, a post-flush finish error that can append a second JSON document,
omitted traversal/authority/pre-ledger owners, nonportable path/formula
contracts, structurally undercounted JSON fields, blanket URL/path
projections, non-exhaustive text-dependent terminal classification, missing
final parent/path authority, and incomplete writer/channel reversals.

Canonical integrated only the immutable report as `8ac6741`. The original
Claude lineage carries the report as `9b5f4ea` and the counterexample-first
R1--R14 closure packet as
`926b179f2dcad8d4fabbeebaf8547f34a50b0c1b`. Exact original implementer
session `c28925df-d4bc-4994-b905-73aaaf2425e5` remains recoverable. Resume is
held for the source-bound target guard and Cargo conflict preflight, not for a
provider slot or a new implementation orientation.

The disposition index now contains 18 exact independent reports: 3 `ACCEPT`
and 15 `REQUEST_CHANGES`, an exact-candidate acceptance yield of 16.67%.
Again this is a flow-yield measure, not a model score.

During File Doctor manager revalidation, the configured shared debug target
returned exit zero in 0.267 seconds with **zero selected tests** and 794
filtered tests. The binary's test list belonged to a different Rust HTTP
worktree even though the command ran from exact File Doctor candidate
`cd130e544af0476162be153db58a0a840b75b134`. A release invocation rebuilt the
candidate and passed 16 `live_file_doctor` tests in 550.765 monotonic seconds,
but that filter did not cover the required NOTE-1/NOTE-2 cases. The manager
preserved both content-addressed logs, stopped the held parent before gate 03,
and kept the candidate `BLOCK`.

This is stronger than the earlier AIA-044 profile defect: one global
`CARGO_TARGET_DIR` can silently reuse a fresh artifact from a different
worktree when Cargo's dependency metadata does not bind the active source
root strongly enough for this workflow. Exit code and target path alone are
not evidence. The new source-target control must:

- bind candidate tree plus tracked/untracked Cargo/crate/config/toolchain
  inputs to a source key;
- serialize inspection/cleanup/build ownership under one lock;
- reject or safely rebuild a target whose provenance names another worktree;
- reject zero selected tests when a nonzero focused set is required; and
- attach the source key, argv, toolchain, nonzero count, duration, and complete
  log digest to every reusable result.

File Doctor, Rust HTTP, File ancestry, and Storage reviewer/repair launches
remain held until this control is implemented and the affected exact gates
are rerun. This quarantine prevents fast false acceptance; it does not restore
a fixed Rust-lane count.

## 2026-07-31 03:28 UTC blocked-Rust backfill and MR-list reviewer rollover

The source-target quarantine did not turn the coordinator into a waiting
lane. Two non-Cargo critical-path reviews were admitted into the released
provider capacity.

The prepared development-flow KGI/KPI Fugu rereview passed manifest
validation and count-free `light` admission:

- projected available memory `12,889,415,680` bytes after its 1 GiB
  reservation;
- swap use, memory full PSI, and I/O full PSI all zero;
- `903,697,518,592` workspace bytes free;
- Fugu occupancy zero in the current adaptive window four; and
- no provider backoff or write/build conflict.

`dev-swarm` started exact report-only Fugu Ultra 1.1 session
`019fb62e-deb1-7193-9efb-c39b024aab6e` against candidate `0ae583e`.
The review writes only
`development-flow-kgi-kpi-zero-residual-rereview-fugu-20260730/review.md`;
the eight manager counterexamples and all original Must/Should/Could rows are
bound inputs. This uses a Fugu slot while Rust product validation is blocked
and consumes no additional Codex implementation lane.

The MR-list exact original implementer session terminated naturally after
producing clean candidate
`b16404676639336f705feffd9af4857d4a1fcc69`. Its final evidence includes:

- 512/512 full Python tests;
- three independent 49/49 focused runs;
- the affected 16/16 CLI set;
- a 228-test full CLI run whose only two failures require absent Cargo/Rust;
  both failures reproduce unchanged on the exact review base; and
- exact candidate tree, log/result digests, and empty porcelain.

Canonical `1096ef7` and `8ffd0f0` bind the candidate card, fresh report-only
Claude Opus 5 ticket, and manifest. Count-free `python-heavy` admission
conservatively retained the KGI/KPI 1 GiB outstanding reservation and still
returned `ALLOW`: projected available memory `10,733,461,504` bytes, swap and
full PSI zero, about 842 GiB free, Claude occupancy two in adaptive window
four, and no conflict. Fresh reviewer session
`220f8ff1-1c65-4424-be46-4e1ccf279c8c` is active. The implementer session is
not reused for review.

The Storage closure packet also passed a separate manager audit before
resume. That audit caught one nonexistent candidate-parent SHA, an
under-authorized final authority path, a missing real-process test path, and
overbroad directory-shaped Store/test grants. No provider was started.
Canonical `c79224d` preserves the original packet as an audit record while
correcting the parent and reducing authorization to eleven exact paths.
Canonical `3bea053` binds the full report/control/session identities, required
fixed-limit subject-plus-one axes, explicit debug/release process gates, and
the rule that a required shared-parser change leaves R9 open. Final Storage
resume remains blocked until the source-keyed guard is committed, ported,
machine-bound, and freshly admitted.

## 2026-07-31 03:47 UTC Upload terminal candidate and credential-safe rereview

The exact resumed Claude Opus 5 implementation session
`7baa1275-0ee2-476a-af46-1907386ebceb` terminated naturally with clean
candidate `467b0c27715e424ae18ffb5664da053b55915e2d`, tree
`3b129b9e41ae45defe4d6e7c43da04b016f07404`. Manager recomputation fixed:

- continuation base and merge base
  `b19c4346c0437ed28bdfe2fb6af1dd466abe9f8b`;
- three additive commits, no merges, and empty porcelain;
- exact binary patch SHA-256
  `a71ea9276361cd426c0eecbf06c900034b5129849df281c8b8f881ffa0f63dc1`;
- implementer result blob `7100966bf76b1ada44d797e8787eb8377a18f6fd`
  and SHA-256
  `a36149570fafcb7e38cc838738796b8e2b8c49095bab2325e1b136a3e31cb361`;
- focused Upload 29/29, neighboring owner 59/59, broad Python 450/450,
  selected legacy 13/13, source-doc, project-doc, and patch-hygiene passes.

The worker claims closure of the completed-quarantine ledger/deadline escape,
caller-owned projection failure, reusable root proof, and nine missing
reversals. These are implementer claims only. Canonical `6eb370d` and
`153c37c` bind the manager candidate card, fresh report-only Fugu Ultra 1.1
ticket, and original launch manifest.

The first scheduler attempt stopped before provider dispatch with return code
3 because the manager worktree did not contain a credential and its launch
environment did not name the existing external
`/workspace/project/fugu-api`. It created no Fugu session, no report, no task
commit, and left the generated task worktree clean at its launch revision.
Canonical `c3a6cb3` preserves the exact state/log digests, cause, and retry
contract. No secret was copied into a worktree or log.

Canonical `8597451` registers a new batch/task identity on top of that
incident. A fresh count-free `light` admission at
`2026-07-31T03:47:13.564766Z` returned `ALLOW`:

- projected available memory `10,334,683,136` bytes after a 2 GiB
  reservation;
- swap use and memory/I/O full PSI zero;
- workspace available bytes `902,823,243,776`;
- Fugu occupancy one in adaptive window four; and
- no provider backoff or write/build conflict.

The retry exports only
`FUGU_API_KEY_FILE=/workspace/project/fugu-api` to the scheduler process.
Fresh Fugu Ultra 1.1 session
`019fb648-a2cb-7ae2-ab60-d0514671c933` is active under manager/task/provider
PIDs `1496834` / `1497118` / `1497189`. Its sole authorized repository write
is
`cfr168-python-http-upload-review-closure-rereview-fugu-20260731/review.md`.
The initial provider-ready failure therefore became durable AIA-042 evidence
without weakening credential isolation or substituting a different model.

## 2026-07-31 03:49 UTC source-keyed Cargo guard pre-integration audit

The first source-keyed Cargo guard was held before commit and before any Rust
candidate consumed it. An independent adversarial audit found ten mandatory
closure classes:

1. source mutate-build-restore TOCTOU and persistent namespace poisoning;
2. post-gate aggregate growth above the 32 GiB budget and stale,
   record-trusting allocation totals;
3. path-based ephemeral cleanup races and incomplete-removal false success;
4. log/result paths able to target linked-worktree Git metadata;
5. toolchain binding by name without exact executable/version/digest or
   compiler-wrapper control;
6. arbitrary/fake test-summary text able to satisfy the nonzero-test rule;
7. incomplete durable `BLOCK` journaling and missing file/directory fsync;
8. admitted source-mutating `fmt`, `clippy --fix`, and `doc --open` forms;
9. indirect Rust dispatch and live shared-root profile-audit bypasses; and
10. non-finite timeout values able to defeat bounded waits.

The speed-related audit also rejected permanent global one-lane execution as
an unexplained end state, identified a future lock-order cycle, and showed
that full-HEAD namespace keys would rebuild several GiB for docs-only
changes.

The repair direction is now fixed: exact tool binaries are content/version
bound; Cargo executes only inside an available `unshare -Urnm` namespace
whose exact candidate root is bind-remounted read-only; evidence uses a
dedicated root and bounded pinned file descriptors; registry usage is
measured from the filesystem under one lock order; cleanup atomically
quarantines the exact inode before descriptor-relative deletion; actual
post-gate budget and source drift block/invalidate reuse; artifact identity is
separated from full candidate evidence identity; and different source keys
move to resource-token concurrency after the conservative bootstrap.

File Doctor, Rust HTTP, File ancestry, and Storage remain held until the
repaired guard is committed, its Python adversarial suite and independent
rereview pass, and File Doctor reproduces nonzero exact-candidate debug and
release evidence. This is a correctness quarantine, not idle provider or
fixed-agent-count waiting.

## 2026-07-31 07:31 UTC open-execution audit and three terminal reviews

Canonical `f20798e` integrates a frozen execution audit captured at
`664f907`. Its machine-readable inventory records three in-scope issues,
29 unique open execution items, 18 then-committed review dispositions, and
six then-absent report outputs. All declared evidence paths existed at that
snapshot, all six pending paths were absent, and the AI-contract and
project-document gates passed. The artifact is intentionally historical:
the three reports below now exist and supersede its pending-output facts
without rewriting the frozen audit.

The three long-running independent lanes terminated normally and their sole
report writes are canonical:

| Subject | Report integration / SHA-256 | Decision | Current disposition |
|---|---|---|---|
| Python Upload `467b0c27715e424ae18ffb5664da053b55915e2d` | `8bdc076` / `57d6792106f09c17a1d154aedb5a64d342e5d19c23db7e9963c46061c868c132` | `REQUEST_CHANGES` | completed-quarantine ownership and publication-proof reuse are closed; retryable release ownership and pre-allocation reversal evidence remain open |
| Python MR-list `b16404676639336f705feffd9af4857d4a1fcc69` | `ee6517a` / `5c952a9a5c162bfbb93522bb644fcba44ce32ddf3628501e6f94308899fe760e` | `ACCEPT` | preserve as an exact composition input; do not integrate alone; route all four informational observations under the manager's zero-residual objective |
| Development-flow KGI/KPI `0ae583ef8b4cfd26e10ebc1f13564d32a7f5e7e1` | `d04ec99` / `268ec0b768a334ad11dbdf57c81245d7b315d7699e34c511e1f782253c178b30` | `REQUEST_CHANGES` | 15 of 35 bound rows are closed and 20 remain partially closed across seven residual owner groups |

Upload returns to original Claude Opus 5 session `7baa1275-…` for a minimal
additive HIGH-2/MEDIUM-4 repair. The release operation must preserve claim
and owner state across pre-effect failure, reconcile post-effect failure,
and reach zero debt exactly once on retry. The ordered projection must expose
a production pre-allocation boundary so removing either its claim or the
source temporary dimension fails before `list(self.documents)` allocates.

MR-list acceptance does not relax successor work. Its report identifies
redundant retained state, unbounded per-fault text, inherited POST response
displacement that is weaker than GET, and one imprecise source-contract
clause. Those four rows are routed into the shared-primitives and POST
successor before any Upload/MR-write composition adopts the protocol.

The KGI/KPI repair must close every partially closed row, including exact
non-finite/nanosecond-safe arithmetic, independently recomputed KGI/cohort
targets, byte-verified contained evidence, immutable final-record binding,
acyclic connected candidate and supersession graphs, complete terminal
lifecycle evidence, per-batch freshness, evidence-derived anti-gaming state,
and bounded schema collections. Existing green official gates are not
decision evidence until the manager and adjacent production counterexamples
fail closed without crashes.

After integrating these reports, the canonical aggregate is 21 independent
reports: four `ACCEPT` and seventeen `REQUEST_CHANGES`. Nineteen are product
reports and two are development-control reports. No issue, Install, Dogfood,
Close, capability, or final KGI decision is authorized by this update.

## 2026-07-31 09:00 UTC Upload second closure rereview and bounded third repair

Canonical `b3de0d7` integrates the independent Claude Opus 5 report for exact
Upload candidate `02e04c7e6458035d9c7d58de4d30b4a12e7cd594`. The report blob
has SHA-256
`3df1db92d6adcef9d318e23182fb9c62519f6eacb091266ced9616626f64cf1f`
and returns `REQUEST_CHANGES`.

The result narrows rather than expands the lineage:

- BLOCKER-1 and HIGH-3 remain independently closed;
- MEDIUM-4 is now independently closed because both required mutation shapes
  fail at the production pre-allocation boundary; and
- HIGH-2 remains open because both live `_AccountedUploadCommitSource` routes
  clear `_session` / `_memory` before fallible cleanup. One real `close(2)`
  failure therefore strands retained debt, leaves no terminal reason, and
  makes a later `close()` a no-op. Constructor unwind can also replace the
  primary exception.

The preceding Fugu Ultra 1.1 attempt produced no report or commit after
provider model-refresh and process-creation failures. It made no product
decision; the failure is retained as provider-infrastructure evidence.

Original implementer session `7baa1275-0ee2-476a-af46-1907386ebceb` is
resumed on only the remaining HIGH-2 root. Its packet requires recovery
handles to remain reachable until cleanup succeeds, primary-error precedence
during constructor unwind, an idempotent reachable recovery path, and real
`close(2)` failure/retry regressions on the two live ancestry and
signature-history routes. Candidate `02e04c7` remains excluded from
integration, and the additive successor will require a fresh independent
decision.

## 2026-07-31 09:05 UTC KGI/KPI round-2 rejection and exact residual replay

Canonical `5760c98` integrates the independent Claude Opus 5 report for exact
development-flow candidate
`0944bf3deba9e753132b3f71716b5c009a8fc30f`. The report-only commit has
SHA-256
`54a4244a8e563b48b2f086a51433ea858996c08b0104c0309f2c8445ce6706b2`
and returns `REQUEST_CHANGES`.

The reviewer independently reproduced 108/108 tests, the frozen 55 semantic
mutations and 15 structural controls, all 17 declared production probes, all
seven AI-development gates, and the positive two-batch, real-Git sealed, and
complete-cohort cases. Candidate `0944bf3` preserves all 15 prior closed rows
and closes F4, F-S10, and F-S12. Seventeen of the twenty previously partial
rows remain partial; thirteen have executable falsifications.

The decision-critical residuals are not a request for more checklist bullets:

- the record can select its own improvement threshold, arm samples, costs,
  throughput, completeness, and zero guardrails rather than deriving them
  from typed lane/evidence facts;
- a finite 401-digit numeric value raises instead of producing a stable
  finding, while a JSON underflow literal can become an accepted exact zero;
- edited records can be resealed against a later or unreachable commit;
- `superseded_by` is not symmetrically checked and a batch may have zero
  authoritative heads;
- validation can be backdated without disclosure, artifact identity can be
  split by citation padding, and the published diagnostic bound does not
  cover structural findings; and
- lifecycle completion can be declared on an active record using generic
  events unrelated to the named stages.

The earlier Fugu Ultra 1.1 attempt produced no report or verdict after
provider/tool failures. Its absence is infrastructure evidence and did not
authorize any product or process conclusion.

Original implementer session `34cdf037-2326-480e-8353-621713838feb` is
resumed on the report's exact owner table and acceptance items 9–14. It must
preserve every independently closed row, reproduce each executable
counterexample before editing, and return a clean additive candidate for
another independent review. Candidate `0944bf3` cannot authorize a
development-process change.

## 2026-07-31 09:08 UTC source-keyed Cargo gate rereview rejection

Canonical `b589864` integrates the independent Claude Opus 5 report for exact
control candidate `aaa39b61b37df273fe4d57e274ed8e1321e788ad`. Its sole report
has SHA-256
`212d391c8fdb23c9178e6f8ddbb68cfc7087a50950049f137d19d342a5e2e259`
and returns `REQUEST_CHANGES`.

The reviewer reconstructed checkpoint `bfbbc2b` independently. The checkpoint
ran 148 tests with four failures and eight errors; candidate `aaa39b6` ran
159/159. Five of the six claimed defect closures are real, and adversarial
probes refuted the hypothesized source-snapshot, result-forgery, cleanup,
key-separation, and lock-order escapes. The global Cargo lock remains the
correct control.

Three local MUST findings prevent using the gate:

- the exec child keeps every `--pass-fds` descriptor except a hard-coded
  fourteen, while the test asserts only the four descriptors the patch
  enumerates; it therefore shares the implementation's blind spot instead of
  checking the complete inherited set;
- the recorded `unshare_flags` identity omits `--propagation private` even
  though the real launch uses it; and
- retention is conservatively re-measured and double-counts normal Cargo
  hardlinks, but the documentation claims exact reclaimed bytes are charged
  back through a flow that does not exist.

Four bounded residuals require the same pass: explain that nested user
namespaces can reacquire capabilities and that kernel mount locking is the
actual protection; correct the checkpoint/candidate test counts; reject short
malformed mountinfo lines; and narrow policy-module reuse wording to the file
and origin checks actually performed.

Original Claude Opus 5 session `f7692f47-4af1-4f84-a186-d57ddd4b083e`
is resumed on those seven rows. It must keep the five verified closures, all
ten fail-closed safety classes, and the global lock. No Rust candidate may
consume this gate until the additive successor receives a fresh independent
acceptance.

## 2026-07-31 09:19 UTC evidence-first review-recursion audit

Canonical `abfee6e` integrates the report-only root-cause audit captured
against historical task base `c258cbf`. The 1,281-line audit has SHA-256
`2392450e957c8437b323ded492fc7597dc77f5147b74f3056c97c0efd9329780`
and reconstructs all 22 candidate commits and 21 then-indexed reports from Git
objects.

The evidence does not support treating simple review omission as the sole or
dominant bottleneck. Across 51 successor findings:

- repair-introduced regressions are primary for 16/51;
- initial review scope misses are primary for 12/51;
- bound findings not closed are primary for 8/51;
- oracle defects appear as primary or contributor in 24/51, but none of the
  17 findings with a reproduced production effect is oracle-primary; and
- 14 of those 17 reproduced-effect findings are repair-introduced or initial
  scope misses.

The audit therefore identifies an interaction loop: bounded recall on very
large shared surfaces, median successor deltas above two thousand insertions
that add new state machines/owners/classifiers, and a zero-residual criterion
that sometimes promotes claim-only or unsatisfiable rows into mandatory
product repairs. Reviews remain high precision: only four withdrawal-like
instances were found among 143 findings. Provider choice did not show a signal
in this corpus.

The audit explicitly refuses to choose a review-item count or template depth.
It proposes seven interventions and requires retroactive measurement of
later-finding capture, noise, duplication, evidence burden, false positives,
and unavailable rows before adoption.

## 2026-07-31 09:22 UTC preregistered review-intervention pilot

Canonical `cdaee6a` freezes the report-only comparison packet
`cfr168-review-intervention-retroactive-pilot-20260731.md`, SHA-256
`3d2f8b16c13349738a587b149d635fd65691a2472ca2a6b8c0d43191a338dd39`.
It evaluates the first four interventions in causal order:

1. repair-delta boundary curve, without preselecting a round number;
2. predecessor-behavior re-derivation;
3. oracle admissibility E1–E9, scored per property rather than as one
   checklist total; and
4. evidence-state versus verdict/routing-state separation.

To reduce corpus overfitting, the packet pre-registers Python MR-write and
Rust File Doctor as score-only lineage holdouts using a deterministic
SHA-256 selection seeded by canonical task base `abfee6e`. The audit had
already seen those lineages, so this is not claimed as fully blind.

Three reports absent from the audit snapshot form a stronger temporal
holdout and cannot tune any threshold or rule: Upload `02e04c7` at
`b3de0d7`, KGI/KPI `0944bf3` at `5760c98`, and source-keyed Cargo gate
`aaa39b6` at `b589864`.

Fresh Claude Opus 5 session
`cfr168-review-intervention-retroactive-pilot-claude-20260731` is active on
the exact packet. It may write only its pilot report. No intervention,
template shape, checklist length, process threshold, product integration, or
issue closure is authorized before the report is independently checked and a
manager decision is recorded.

## 2026-07-31 09:37 UTC shared-primitives dependency inversion removed

Canonical `8929a98` changes the existing Python HTTP shared-primitives
preflight from `blocked` to `ready` without authorizing any product byte. The
change implements the independently supported conclusion from both throughput
and review-recursion audits: a report-only negative-contract preflight does
not require an accepted Upload implementation.

The packet now binds:

- accepted Doctor/Branch `701469f` and accepted MR-list `b164046`;
- three chronological Upload rejecting reports (`f835f64`, `8bdc076`,
  `b3de0d7`) with exact SHA-256 identities;
- MR-write rejecting report `84175bd`; and
- Storage rejecting report `8ac6741` only for cross-runtime invariant-family
  counterexamples.

Rejecting reports are specifications, not product authority. Every primitive
clause must cite its source report digest and evidence state. The Upload
retryable-cleanup implementation shape remains `shape-pending`; the invariant
and negative case are fixed, but no currently reviewed Upload byte may freeze
the shape.

The provider is routed to Claude Opus 5 because the recent Fugu lanes failed
before producing reports and the provider remains under backoff. The
preflight may write only
`cfr168-python-http-shared-primitives-preflight-claude-20260731/preflight.md`.
MR-write remains blocked until that report commits literal `READY`; final
Python composition still requires a separately accepted Upload identity.

## 2026-07-31 09:43 UTC pilot calibration freeze and Claude quota stop

The preregistered review-intervention pilot reached its contamination boundary:
it recomputed calibration data and froze a 384-line calibration file before
opening either lineage or temporal holdout. The frozen file has SHA-256
`4d57c323b7c20015c9cd388c4175c973631a67839d8e3d0cc427e58a7cec87d9`
at `2026-07-31T09:43:17Z`.

The next Claude call returned provider status 429 and the exact user-facing
message that the organization had reached its monthly spend limit. The
structured event labels the rejected window `five_hour`; both facts are
preserved without choosing between the provider labels.

No holdout was opened, no `pilot.md` was written, no commit was produced, and
no intervention decision exists. Canonical incident evidence is recorded at
`cfr168-review-intervention-retroactive-pilot-claude-quota-20260731/incident.md`.
Its resume contract binds the clean worktree, session, event-log SHA-256
`af6baeac543f716784c4be65cc9c2bf6b991860c30f8b6f89394427ec586e204`,
freeze event UUID, and freeze digest. Calibration numbers visible in the log
must not be adopted before the preregistered holdouts are scored.

Claude is now under provider backoff for new launches. At this point Fugu was
also held under a separately reported infrastructure backoff; the later direct
event-log correction at 09:57 UTC identifies its actual terminal cause as a
provider usage limit. There is no automatic Codex fallback. Already-running
Claude reviews are allowed to reach their own terminal provider state so their
existing work and logs remain attributable.

## 2026-07-31 09:46 UTC three active reviews stopped at the same Claude quota boundary

The Upload `446cf4a`, source-keyed Cargo gate `4e5c0e7`, and KGI/KPI
`753c8db` independent reviews each later reached the same provider 429. All
three worktrees remain clean at their exact candidate heads. None contains its
authorized report path or a review commit.

Canonical incident
`cfr168-claude-quota-review-interruptions-20260731/incident.md` binds every
session, worktree, event-log byte length/SHA-256, rejected-event UUID, exact
candidate, and absent report. The structured provider reset epoch is
`2026-07-31T12:30:00Z`, but the user-facing message says monthly spend limit;
the reset is not assumed to restore monthly capacity.

All three decisions remain absent. No candidate gains integration authority.
Resume must use the same exact session/worktree after verified capacity,
without launching a competing reviewer. The now-ready shared-primitives
preflight is also launch-held by provider backoff.

## 2026-07-31 09:53 UTC review-root-cause reliability gate

The earlier review-recursion audit is now treated as a substantive but
provisional single-analyst hypothesis, not as a process-change decision.
Packet `cfr168-review-root-cause-reliability-gate-20260731.md` adds a blind
reclassification and disagreement-adjudication gate before any template,
mandatory finding count, repair-delta threshold, or checklist may be adopted.
Its SHA-256 is
`cdc07044dde70b33549c7708ad3a7c3635585b3895d649ba89d49bf0a59131da`.

The correction is methodological:

- the 51 successor findings are a census of **observed later findings**, not a
  denominator for absolute review recall;
- absence of later withdrawal is not sufficient evidence for review
  precision;
- one narrative primary cause is replaced for validation purposes by six
  independently evidenced axes: code origin, prior-review detectability,
  repair-contract handling, effect evidence, closure-oracle quality, and
  review-routing state; and
- every decision-critical disagreement and reproduced production effect must
  be independently adjudicated, followed by an unchanged temporal check.

The interrupted intervention pilot remains frozen under its original
preregistration. It may finish, but its output cannot authorize policy. If the
reliability gate changes a decision-relevant label, only a separate sensitivity
replay may evaluate the difference; the frozen calibration must not be
rewritten.

Preferred low-Codex execution is a blind Fugu Ultra 1.1 classifier followed by
Claude Opus 5 adjudication of disagreements. Both providers remain launch-held
under their recorded backoffs, and neither has an automatic Codex fallback.

The gate now also binds an analysis-only row form. It requires separate axis
values and evidence locators, counterevidence, exact reproducer or
`UNAVAILABLE`, oracle reversal, censoring, and decision relevance. Detailed
evidence blocks are mandatory for decision-critical, disagreed, reproduced,
or denominator-sensitive rows. This is not a permanent review template and
does not reward additional rows; it prevents a long unsupported bullet list
from masquerading as causal analysis.

## 2026-07-31 09:57 UTC Fugu terminal-cause correction

Canonical incident
`cfr168-fugu-quota-review-interruptions-20260731/incident.md` replaces the
manager's earlier model-refresh/process-creation attribution with direct
event-log evidence from all four affected Fugu lanes. Its SHA-256 is
`6a3494c10fbdb369cc5436cfe2477d23339e252c9a4184310b3a981c146d64e2`.

Every complete JSONL log has the same top-level terminal cause:

```text
You've hit your usage limit. Try again at Aug 3rd, 2026 12:00 AM.
```

The displayed time has no timezone and is not converted into a guaranteed
reset instant. The four lanes completed 64, 198, 66, and 44 command executions
before termination. Apparent searches for model-manager or process-creation
errors were matches inside repository text or agent reasoning, not top-level
runtime errors. The wrapper is verified to bind Codex CLI transport to Sakana
API, `SAKANA_API_KEY`, and `fugu-ultra-v1.1`; the secret was not printed.

All four worktrees are clean and all four sessions are superseded by the
already integrated Claude reports, so none will be resumed. Fugu remains
closed to new work until the displayed reset has passed and one bounded health
probe succeeds. The next real Fugu work is a fresh blind root-cause
classification session. No automatic Codex fallback is authorized.

## 2026-07-31 10:07 UTC control-plane composition collision frozen

The exact source-keyed Cargo-gate candidate `4e5c0e7` and KGI/KPI candidate
`753c8db` are both clean and independently awaiting the Claude reviews stopped
by quota. A manager preflight found that their accepted implementations cannot
later be integrated as two blind branch merges:

- the Cargo source range changes 19 paths;
- the KGI/KPI source range changes 17 paths; and
- five paths overlap: the AI README, acceleration plan, history,
  `ai_contracts.py`, and `test_ai_tools.py`.

The initial version of blocked packet
`cfr168-control-plane-composition-20260731.md` froze a
31-path semantic-port contract before either review returns. It requires both
exact candidates to receive fresh `ACCEPT`, starts from then-current canonical
instead of either stale branch history, preserves every test name from both
ranges, runs both counterexample families and deliberate regressions, and
requires a fresh exact composition review. It authorizes no product or Rust
mutation and has no Codex worker fallback. That superseded initial packet's
SHA-256 was
`6c356cdccf9bd8e8559aca85cec1ecb7570f95e41e860af81a36b4b99ddb8cb9`.

The initial packet coupled Cargo-gate integration to KGI/KPI acceptance. That
would unnecessarily hold every Rust candidate behind the independent
development-flow review. Before either packet became executable, the manager
split the dependency:

1. `cfr168-source-keyed-cargo-canonical-port-20260731.md` ports and freshly
   reviews accepted Cargo candidate `4e5c0e7` first; its accepted integration
   releases source-keyed Rust validation immediately.
2. `cfr168-control-plane-composition-20260731.md` then ports accepted KGI/KPI
   candidate `753c8db` onto the Cargo-enabled canonical head and proves that
   the five overlapping files preserve both contracts.

Neither packet is admitted by worker-green evidence, and both remain blocked
on their exact missing independent decisions. The current Cargo canonical-port
packet SHA-256 is
`2116274a375d135433fe5bbacc7b40cd2395ccffbed0e4000c4c98ee15ca5f7b`;
the current KGI/KPI composition packet SHA-256 is
`ac9b2c01a9d3b0ee44957d04912b59f0ba8a229deb7698321e946cc15f587e77`.

## 2026-07-31 10:19 UTC post-reboot candidate revalidation preserved

Manager evidence
`cfr168-post-reboot-candidate-revalidation-20260731/evidence.md` binds exact
HEAD/tree/clean-state observations and monotonic gate results for Cargo
candidate `4e5c0e7`, Upload candidate `446cf4a`, and KGI/KPI candidate
`753c8db`. Cargo passes 168 AI-tool tests and all seven AI-development
components; KGI/KPI passes 129 AI-tool tests and all seven components; Upload
passes 33 focused and 454 broad Python tests, compile, both documentation
checks, and `git diff --check`.

This is health evidence only. It supplies no independent verdict, closes no
finding, and changes no integration state. All three exact Claude reviews
remain pending. Separate provider admission captures show adequate memory,
swap, disk, PSI, PID, provider-window, and conflict-domain state; their only
blocker is provider backoff. The evidence SHA-256 is
`0be5f2a17ac3ccb43c54612595b14be3664531dc8189410dedaccccef2defba7`.

## 2026-07-31 10:24 UTC development bottleneck causal gate frozen

`cfr168-development-bottleneck-causal-gate-20260731.md` prevents the manager
from promoting one retrospective audit directly into process policy. It
separates observed delay, causal attribution, later replication, and measured
intervention effect; uses trusted integration rather than activity or finding
volume as the outcome; and preserves review/repair causes as independent
hypotheses under the existing blind reliability gate.

The gate also removes the superseded Fugu model-list attribution from future
provider decisions, defines falsifiable cards for the source-keyed Cargo gate,
dependency splitting, terminalization, manager queue, gate reuse, provider
readiness, resource retention, and shared-primitives preflight, and requires
raw queue/service/rework/burden/guardrail evidence before adoption. No fixed
finding count or template depth is selected. The packet SHA-256 is
`61e337f2944bd5d11a2eb868f104520cd3b9cef11079eb9575c3cf54cb2a237f`.

The gate does not add a classification dependency to the already prepared
report-only shared-primitives preflight. That preflight may collect evidence
without mutating product or accepting rejected behavior; a `READY` result is
not itself proof of a shared-root cause or speedup.

The current review queue now admits the three exact Claude resumes
concurrently when the verified adaptive provider window allows. Under a
constrained window, Cargo `4e5c0e7` precedes Upload `446cf4a` and KGI/KPI
`753c8db` because its accepted canonical port releases four held Rust gates.
This is a downstream-release tie-break, not a fixed one-lane cap or a review
quality judgment. The updated disposition-index SHA-256 is
`5f5a1f5e3b20385bcb163380f4235b7dbbe4561825d5451da4d3110cacd77991`.

## 2026-07-31 10:30 UTC Cargo canonical-port inputs mechanically frozen

Read-only preflight
`cfr168-source-keyed-cargo-canonical-port-preflight-20260731/evidence.md`
binds exact source/candidate/canonical trees, the binary source-range digest,
all 19 path states, the two current canonical blob collisions, and normalized
test-name-set digests. Its SHA-256 is
`90449137903e2b15f0fd780cee7a3e9809cb54e7d1b7da18cce885282579ba34`.

At comparison head `9c1eefd`, 17 source-range paths still matched the
historical base state and only the File Doctor and Rust HTTP review packets
had diverged. The candidate adds 104 unique test names, removes one permissive
shared-root test name, and replaces its behavior with separate unkeyed-block
and source-keyed-pass cases. These finite counts describe the exact Git
range; they are not quality quotas. The future accepted port must recompute
all sets from then-current canonical and prove set preservation rather than a
plausible total count.

This preflight does not review or accept `4e5c0e7` and does not admit port
execution. It exists so the eventual implementer and reviewer compare exact
machine-derived sets rather than generating a large prose checklist from
memory.

## 2026-07-31 10:34 UTC KGI/KPI composition inputs mechanically frozen

Read-only preflight
`cfr168-kgi-kpi-control-plane-composition-preflight-20260731/evidence.md`
binds the exact KGI/KPI source range, 17 path states, six current canonical
collisions, and normalized source/Cargo test-name sets. Its SHA-256 is
`07e7fb0eb43550c3f496a9c9427e65cb956edf695349f575fe5494b36ed0f910`.

The KGI/KPI candidate adds 87 unique test names relative to its base, none of
which collides with the Cargo candidate's added set. The source-candidate
union contains 255 names at this snapshot, but the future composition must
prove set inclusion and nonzero execution rather than an exact total. The
preflight predates the Cargo canonical port, so every blob, collision, and
test set remains mandatory to recompute after both independent decisions.

This evidence neither accepts `753c8db` nor starts composition. It reduces
future ambiguity without turning a large manually generated checklist into a
quality metric.

## 2026-07-31 10:35 UTC intervention-pilot freeze reverified after reboot

The frozen calibration file still exists at
`/tmp/pilot/calibration-frozen.md`, is 27,813 bytes, and exactly matches
SHA-256
`4d57c323b7c20015c9cd388c4175c973631a67839d8e3d0cc427e58a7cec87d9`.
The complete 1,065,522-byte provider event log also still exactly matches
`af6baeac543f716784c4be65cc9c2bf6b991860c30f8b6f89394427ec586e204`.

The pilot worktree remains clean at HEAD `cdaee6a`, tree
`c8125d0008b63eafff34d6bd7057da34dcc92229`. No holdout was opened and no
pilot result or decision was created. Resume can therefore continue from the
original frozen calibration after verified Claude capacity; no reconstruction
or calibration retuning is needed.

## 2026-07-31 10:38 UTC Rust source-keyed recovery wave bound

Blocked addendum `cfr168-rust-source-keyed-recovery-wave-binding-20260731.md`
binds exact File Ancestry `02b0d63`, File Doctor `cd130e5`, Rust HTTP
`92fc810`, and Storage original-session inputs to the future accepted
canonical Cargo port. Its SHA-256 is
`607f54c9ce4b51b9d2904d7c1a41da720f58fc5b2b929718e748f1bf084227aa`.

The addendum supersedes only old direct/shared-target launch mechanics.
Existing candidate identities, closure matrices, exact test selections,
mutations, scopes, and decision rules remain mandatory. Every Rust command
must use the accepted absolute source-keyed wrapper, exact clean subject,
unique bounded result/log, nonzero selection, complete digest, and unchanged
candidate proof. Older manifests are launch-ineligible until regenerated
against the accepted port.

Timeouts are per command, finite, and based on comparable measured duration;
no blanket 30/40-minute limit remains. Heavy gates may receive six hours, and
the separate six-hour lock wait permits healthy serialization. Deliberate
regressions use one clean scratch worktree with temporary mutation/revert
commits and ephemeral source-keyed artifacts, not dirty candidates or one
target/worktree per mutation.

This binding starts no Rust process or provider. Static/report work may run up
to live admission, but the currently designed global Rust execution lock
continues to serialize Cargo until separately reviewed bounded concurrency
exists. The current disposition index now points all four Rust lines to this
binding; its SHA-256 is
`890c5e0b128baa8b0e369afcf8e95079780dfb3315ea4a760b6fb2e7e45da855`.

The index root map now also names pending Upload `446cf4a`, pending Cargo
`4e5c0e7`, the canonical-port review requirement, and the removed Upload
precondition on the report-only shared-primitives preflight. It does not
promote their manager gates to closure.

## 2026-07-31 10:45 UTC blind Fugu causal classifier prepared

The fresh report-only Fugu Ultra 1.1 blind-classification lane is now defined
without launching a provider. Its execution ticket SHA-256 is
`1ab52b36e7f64ff576dd8eff9dd704f11781c6f1f1e0faf0a48ca7b1f96f30c7`;
its human-readable task view SHA-256 is
`64470d7deeb42a23703a9eb316ad81015f6878ae54c3daa8835ded8849f42c5d`;
and its typed packet SHA-256 is
`076361fb80e1610163251a2842369d4c4346aa80293dfa86a940ebe293c08c95`.

The packet binds the 51 observed successor findings as a finite historical
census, not an output quota. It requires one evidence-rich analysis row per
independently testable effect, six orthogonal causal axes, a digest-bound blind
freeze before provisional labels are opened, row-level disagreement review,
and an unchanged temporal holdout. It explicitly cannot adopt a template,
checklist, item count, routing policy, candidate disposition, or product
change.

This lane remains provider-held. Do not launch it until the displayed Fugu
reset window has passed and one bounded health probe succeeds. Use one fresh
session; do not resume a superseded Fugu lane, start a competing classifier,
or substitute a Codex worker. A scheduler manifest will be bound to the exact
packet-definition commit separately so its launch base cannot float.

## 2026-07-31 10:51 UTC adjudication and pilot admission separated

Commit `1072934` binds the one-lane Fugu scheduler manifest to packet commit
`f916c7f`. The manifest SHA-256 is
`ba6928733a02b170fdeb06159bda1758767ad7fad77d82168884e968213ef20c`;
`dev-swarm validate` and `dev-swarm plan` resolve exactly one Ultra reviewer,
Fugu concurrency one, Claude/Codex zero, and a 12-hour runaway limit. It has
not been started.

The downstream Claude adjudication is now prepared but intentionally
`blocked`. Its execution ticket SHA-256 is
`4e77bc9c6fc4f8e6bda6f0beb9ed1fd994b13ecd2223423cbcda4c534abf4b6d`;
human-readable task-view SHA-256 is
`983112d4a68a34724d13cba6d6ef16d81ba45da09def10cd0618f0b654f609e5`;
and typed-packet SHA-256 is
`2e4ada3b750a9e71c8035fb3d9bd7f09b62f3421c61631d68bd30bb245a7f9ea`.
No adjudication manifest may exist until the classifier report commit/tree,
report/freeze digests, exact row/axis disagreement set, reproduced-effect set,
and routed decision-critical unavailable set are committed into the packet.
The adjudicator reopens only that finite queue; it receives no target finding
count and cannot adopt policy.

Manager gate
`cfr168-review-intervention-pilot-resume-decision-gate-20260731.md`, SHA-256
`635b3ab821b556c4163f43688524cd2524bf35b644ee7f3d1a9f78feba7f8565`,
supersedes capacity-only admission for the preserved Claude pilot. Provider
recovery alone no longer authorizes resume. Blind classification and
adjudication must first show whether I1, I3, I4, or I5 maps to an
adjudicated, decision-relevant failure mode. The manager then chooses
`RESUME_UNCHANGED`, `RESUME_WITH_SEPARATE_SENSITIVITY_REPLAY`,
`RETIRE_PRESERVED_PILOT`, or `UNAVAILABLE` without changing the calibration
freeze or opening holdouts prematurely.

Under a constrained Claude window, the three pending product/control-plane
reviews retain downstream-release priority over this process pilot. This
prevents sunk analysis cost or an available session from displacing work that
releases Cargo, Upload, or KGI/KPI integration.

## 2026-07-31 11:16 UTC exact resumes and Python composition inputs reverified

The three quota-stopped Claude review subjects remain clean and unchanged:

| Subject | HEAD | Tree | Session | Event-log bytes / SHA-256 |
|---|---|---|---|---|
| source-keyed Cargo gate | `4e5c0e7f058cd406cb7a183ecda8b3a07f3ee912` | `6417400cb7b5a3507f85f38d66c0573992ad4007` | `d089c8ae-8364-49af-bcd1-b9bf4b741ea1` | `947871` / `8a371375000054393df3b00253ab7e2e56aa5a4ad21822d01b36cd333abcad1c` |
| Python Upload | `446cf4a97b63e46d011a03083ef59354b9d68178` | `3ac8fa8bc5aa38a1f137fd523db8285980d69692` | `239dd18d-bdd7-4828-b503-336e02ba752a` | `836088` / `5fbad76e50a70e3992ef52dcf6dc8d1cf1c3b6800fad2743f14ba179a88efb1d` |
| KGI/KPI | `753c8db103579d021795d3fe6a97c5d45d4a111a` | `213e16a033aeb2e8eb3599be4ffe725b1d7ad344` | `60548555-ff29-4fd9-9360-2820a663ee54` | `840799` / `22816356a2026d88f9eb8695aa8d4913929d5993bc5bdfe3ba5110960402632f` |

All three saved-session lookups resolve to those exact agent/worktree pairs
and none has an authorized report. At `11:16 UTC`, `MemAvailable` was
`11,880,532 KiB`, swap use `2,060 KiB`, `/workspace` availability
`902,255,611,904` bytes, and memory/I/O full PSI avg10 were both zero. The
candidate and resource state is ready; Claude provider backoff remains the
only admission blocker before the structured `12:30 UTC` reset surface is
verified. One bounded, no-tools, no-session-persistence Opus 5 health probe is
scheduled for `12:31 UTC`; it does not automatically resume a worker and has
no retry loop.

Mechanical evidence
`cfr168-python-http-upload-mr-list-composition-preflight-20260731/evidence.md`,
SHA-256
`5059b135925eed5587021b44fa4518f054486ce5b95db56fa1a6fd3b2f6e1e7f`,
ports only allowed product/test/source-doc ranges into a transient scratch
tree:

1. pending Upload reference range `cb13da0..446cf4a` applies cleanly to
   accepted Doctor+Branch scaffold `701469f`, preserves all 33 Upload tests,
   and passes 65 focused plus 486 Python-pattern tests;
2. accepted MR-list range `701469f..b164046` then applies cleanly, preserves
   all 49 MR-list tests, and the combined tree passes 114 focused plus 535
   Python-pattern tests and both source/project documentation checks; and
3. the scratch worktree was deleted after its exact trees, patches, results,
   logs, durations, and SHA-256 values were bound.

This is compatibility and future-port evidence only. Upload remains
unaccepted, the four MR-list observations remain routed, the shared-primitives
preflight is absent, MR-write is absent, and the speculative tree cannot be
integrated. If Upload is rejected, no byte from this preflight is reusable as
product authority.

The active issue inventory also recomputes consistently across detail files,
summary ledgers, and `issue-root-cause-map.json`: three CFB
(`CFB-133/136/138`) plus six CFR (`CFR-168..173`), nine total. No issue status
changed. The current disposition-index SHA-256 is
`e9f5b3891a33802507d8ba67305d7725f1c9f6fde57890a38d6404bb478ce4f8`.

## 2026-07-31 11:32 UTC review-cause analysis decontaminated before launch

No provider had launched the prepared causal classifier, so no worker result
or provider spend was discarded. Manager inspection found that the prior
design still exposed a methodological risk: one session was asked to freeze
calibration, then open the provisional audit, compare it, and inspect temporal
reports. Its packet also supplied the provisional 51-row census. A content
freeze reduced post-hoc editing but did not remove model-context contamination
or count anchoring.

Commits `6e4c8e5b838248899b3c43cb3f170409d3abf777` and
`c88f0facf71d169dcee706613d54bef05e9c5aae` supersede those launch mechanics
without rewriting the earlier preparation record:

1. the Fugu blind-calibration lane now starts at exact pre-audit snapshot
   `c258cbfc061b14905554c9d319b350d2da7f771b`, tree
   `75135efad20c5908884231b152cde72d0c9be965`;
2. that snapshot contains exactly 21 `review.md` files but contains neither
   the provisional root-cause audit nor the three temporal reports;
3. the blind lane is not told the provisional 51-row result; row count is an
   outcome and every row must bind one independently testable effect;
4. a different, currently blocked Fugu session will reconcile effect identity
   before comparing counts or six-axis values, freeze that comparison, and
   only then open the temporal reports; and
5. a fresh Claude session remains blocked until it can adjudicate only the
   comparator's decision-relevant queue.

The revised reliability gate SHA-256 is
`8fd0c824f0f87a25b4c6c3eb400033672eb99d73c2b86b16858fc65534b39665`.
Blind execution ticket, task view, typed packet, and one-lane scheduler
manifest SHA-256 values are respectively
`f39259cddcae985014a70d66d618702ff9a7ebe758786ddd54e9204c173eaa23`,
`31bec81b4c7148b99aa1e0a42b352c9e7f5f4ee7485ea7533b24514fb791b7e4`,
`98554b90400f263ff53d883f7cb8002b64368ddcc7a68713f31ba955e6b4bcaf`,
and
`315fac4193de43f2a10c3fb7578238f2ae70d70631e6757750e38685f3d01488`.
`dev-swarm validate` and `dev-swarm plan` resolve one Fugu Ultra reviewer,
Claude/Codex zero, a 12-hour runaway limit, and the exact `c258cbf…` base.

The blocked comparison execution ticket, task view, and typed packet SHA-256
values are respectively
`276cde1c03d11c82876a0017fb6f7f965c9a222f2a10b2cc8426fbe16183d82c`,
`a63331d50b90e712e30bcfea2e03c81c7b9c5aadf713fbded80cfe7faf991d22`,
and
`044db1037803311634e0f0e38d2a1c623c9fc1212205e3074226a387fc50dae7`.
No comparison manifest may be created until the blind report commit/tree,
bytes, SHA-256, row count, row-identity digest, reproduced-effect set, and
decision-critical-unavailable set are bound.

The blocked Claude adjudication execution ticket, task view, and typed packet
SHA-256 values are now
`364bd76727d359d8042bcc3beb22b191984fa282494c5ffb9cdf87ea12774aeb`,
`32e308b8da34338c27e6a0270e78f8b4d9e9058814fba1666976416588120a6f`,
and
`2c101663e224392e0254e5708827fbba365ec96f4a4125ddd32c95749db810ff`.
The pilot-resume and general bottleneck gates now require calibration,
fresh-session comparison, unchanged temporal application, and adjudication;
their SHA-256 values are
`884cb2221db840b2b585949c07c762c95696790a6b7613e11222eee3c5bcb9d7`
and
`16b0a7a1d16a2213f1e20dd591682a65bedd0fc5c72d753a6f5ddb8b5e4abf0d`.

Before accepting the blind report, the manager must preserve the complete
Fugu event-log digest and inspect tool calls for access to a newer ref/commit,
another worktree, the provisional-audit path, any temporal-report path, or
pilot artifacts. A clean report diff alone does not prove a clean blind input.

The first methodology commit added new untracked Markdown files after the
initial worktree `git diff --check`; staged `git diff --cached --check`
therefore exposed six line-ending whitespace warnings only at commit time.
Commit `c88f0fa…` removes them. Future mixed tracked/untracked documentation
changes must stage intended files before the final diff check so the final
gate covers the actual commit set.
