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
- Recovery task timeout is eight hours.

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

## Active recovery lanes

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

### Next-slot ready queue

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
| Python upload / CFB-138 zero-residual repair | Claude `claude-opus-5` | active from exact base `5f79899`; no product commit yet |
| Python MR-list exact rereview | Fugu `fugu-ultra-v1.1` | active against `38da898`; confirmed symlinked/replaced `branches` absence-conflation defect remains under final report analysis |
| Python MR-mutation exact rereview | Fugu `fugu-ultra-v1.1` | active against `6dd4527`; confirmed unaccounted authority reads remain under final report analysis |
| Rust storage zero-residual repair | Claude `claude-opus-5` | active from exact base `13767c7` through the registered dev-swarm batch |

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

The rereview is frozen on taskbase
`/workspace/worktrees/development-flow-kgi-kpi-zero-residual-rereview-base`
with confirmed-probe launch base `110a38c` and manifest commit `09e5911`, for
Fugu Ultra 1.1 dispatch when a review slot becomes available.

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
