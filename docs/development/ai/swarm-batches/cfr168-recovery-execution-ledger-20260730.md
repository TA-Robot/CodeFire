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
