# AIA-046 Source-keyed Cargo Gate MUST Closure — Independent Rereview (Claude)

Review date: 2026-07-31

Reviewer: Claude Opus 5 (`claude-opus-5`)

Batch: `aia-source-keyed-cargo-gate-closure-rereview-claude-20260731`

Subject: candidate `aaa39b61b37df273fe4d57e274ed8e1321e788ad`, additive over checkpoint
`bfbbc2b2bdc22f88679e5e1dc794665d863304a1`

**Verdict: `REQUEST_CHANGES`.**

Five of the six claimed MUST/CRITICAL closures are real, and each is anchored by a negative control
that I re-executed against the checkpoint and confirmed was failing there. The sixth — closing the
inherited isolation-tool descriptors before Cargo — is only partially implemented: the gated command
still inherits the pinned `unshare`, parent-death-helper, policy-supervisor, and every non-selected
toolchain executable descriptor, so the invariant that `source-keyed-cargo-gates.md` step 9 now
states ("retaining only the descriptors the child environment itself names") is false as written, and
the new live control asserts only the four descriptors the patch enumerates, so it cannot observe the
gap. Two further newly written claims are factually wrong: the recorded isolation identity still
declares the pre-fix `unshare` flag list and omits the `--propagation private` flag the fix depends
on, and the new "exact reclaimed bytes charged back against the aggregate budget" claim is wrong in
both halves. None of the three is a privilege escalation, and no finding invalidates the gate's
fail-closed posture; they are claim-versus-implementation defects in exactly the controls this commit
says it closed, which is the standard this document sets for itself.

This verdict was derived independently. The implementer's reported gate results were treated as
claims and re-run. The failed Fugu attempt is treated as infrastructure loss and is recorded
separately from the product findings below; it contributed no verdict, no report, and no repo
residue, so nothing in it was inherited.

## 1. Recomputed subject identity and clean state

| Claim | Recomputed | Match |
|---|---|---|
| Candidate | `git rev-parse HEAD` → `aaa39b61b37df273fe4d57e274ed8e1321e788ad` | yes |
| Candidate tree | `git rev-parse HEAD^{tree}` → `a99d1703be115d7008fad884f977d5506b927db0` | yes |
| Checkpoint is an ancestor | `git merge-base --is-ancestor bfbbc2b HEAD` → true | yes |
| Delta | `git diff --shortstat bfbbc2b aaa39b6` → `9 files changed, 679 insertions(+), 77 deletions(-)` | yes |
| Delta binary-patch SHA-256 | `git diff --binary bfbbc2b aaa39b6 \| sha256sum` → `6712a598c4c2a5ed0a6f0a7ba8e6ff9f52b3e49778cb144df2a32d4f4a43ab24` | recorded |
| Worktree state at review start | `git status --porcelain` empty | yes |

Changed paths, all nine reviewed in full:

```text
docs/development/ai/acceleration-master-plan.md
docs/development/ai/failure-cookbook.md
docs/development/ai/source-keyed-cargo-gates.md
docs/development/history.md
docs/development/tools/build_source_bound_rust.py
docs/development/tools/readonly_source_exec.py
docs/development/tools/run_source_keyed_cargo_gate.py
docs/development/tools/run_timed_gate.py
docs/development/tools/test_ai_tools.py
```

File digests at candidate HEAD:

```text
a85f4251fa5fc413a1f872b7e0321a2ae1969feeefdbcc4a15cf79702e59a64a  docs/development/tools/run_source_keyed_cargo_gate.py
654b47a4578c695e97d62cf99dfdfb7c142868ca752d6ce6d259deea9a99339d  docs/development/tools/readonly_source_exec.py
db397a6ecd379d35a034adf97a1764c309f6337d9a663c1fffe36d7cb3d4ed3f  docs/development/tools/test_ai_tools.py
901af9d3186a36bd19c329c3210ff3d7cc0f3e66c00e4a728a33fbcfbcd19afb  docs/development/ai/source-keyed-cargo-gates.md
```

The implementation was not modified during this review. All adversarial work ran from `/tmp`
(`probe_fd_leak.py`, `probe_userns_regain.py`, `probe_userns_raw.py`, `probe_adversarial.py`,
`probe_hardlink.py`) and against a `git archive bfbbc2b` extraction in `/tmp/ckpt`, which never
touches the repository. `git status --porcelain` is empty apart from this report.

Sources read in full before any finding: `AGENTS.md`,
`docs/development/ai/source-keyed-cargo-gates.md` (428 lines, post-candidate),
`docs/development/ai/swarm-batches/cfr168-recovery-execution-ledger-20260730.md` (recovery execution
ledger / audit), the complete candidate delta, and the surrounding implementation regions each hunk
depends on in `run_source_keyed_cargo_gate.py`, `readonly_source_exec.py`,
`build_source_bound_rust.py`, `run_timed_gate.py`, and `test_ai_tools.py`. No implementer final
summary exists as a separate artifact; the commit message of `aaa39b6` is the only implementer
statement in the commit evidence, and it is the text audited below.

## 2. Fugu infrastructure failure, kept separate from the product

The prior Fugu checkpoint-review attempt stopped on repeated tool/model-refresh errors before any
report or commit. I confirmed it left no repo residue: `grep -rn "aia-source-keyed|source-keyed-cargo-gate-closure" docs/`
returns nothing, `docs/development/ai/swarm-results/` contains no directory for this lane, and no
partial verdict, packet, or JSON exists for it. There is therefore no prior verdict to reconcile,
adopt, or contradict, and no evidence was salvaged from that session. Every hypothesis in section 4
was re-derived from the code rather than inherited from that attempt.

That failure is a provider/harness incident, not a product signal. It is not counted as a finding
against `aaa39b6`, and none of the findings below depend on it.

No Cargo, `rustc`, Clippy, or `rustfmt` process was started by this review. That is consistent with
the recovery execution ledger, which holds Rust lanes until the exact toolchain is rehydrated and
records that a review owns no Cargo rerun; it is a deliberate scope boundary, not an unmet gate.

## 3. The six claimed closures, verified against the checkpoint

I reconstructed the checkpoint into `/tmp/ckpt` and ran its own suite there:

```text
checkpoint bfbbc2b: Ran 148 tests — FAILED (failures=4, errors=8)
candidate  aaa39b6: Ran 159 tests — OK (22.677s)
```

Every checkpoint failure is reproducible and each maps to a claimed defect. The four `FAIL`/`ERROR`
root causes I captured:

```text
AttributeError: probe_validation_capabilities has no attribute 'validate_provenance_payload'
TypeError: change_second_capture() got an unexpected keyword argument 'require_clean'
AssertionError: Lists differ: ['-I', '-S'] != ['--pdeathsig', 'SIGKILL']
AssertionError: 3 != 2                       (x2, reservation/unexpected-failure result paths)
AssertionError: OSError not raised            (provenance rollback barrier)
```

| # | Claimed closure | Independent finding |
|---|---|---|
| 1 | Policy modules reused after pinned-path verification instead of re-executed | **Closed, and the defect was product-visible.** `test_result_is_reserved_before_gate_and_survives_publish_error` and `test_unexpected_failure_is_durably_recorded_as_block` are unchanged by this commit and failed at the checkpoint with `3 != 2`. The checkpoint stderr shows why: the gate rejected the caller's log path against the *ambient* evidence root (`gate log must be a direct child of /workspace/.cache/codefire-gate-evidence`) because `mock.patch.dict(DEFAULT_PROFILE, …)` patched a different dict object than the orphaned second `check_rust_execution_profile` instance the preload had created. The reuse path at `run_source_keyed_cargo_gate.py:328` fixes exactly that, and `test_policy_module_bound_to_a_foreign_origin_is_rejected` proves a hijacked name fails closed rather than being silently replaced. See NIT-2 for a precision limit. |
| 2 | Destructive cleanup is deadline-bounded, reclaim measured by the removal walk | **Closed.** `test_incomplete_namespace_over_entry_cap_is_recovered` is byte-identical between checkpoint and candidate (verified by diffing the extracted test bodies) and moves from `ERROR` to pass, so this is a pre-existing negative control, not a control written to fit the fix. `remove_tree_contents_at` (`:4986`) is now deadline-only, with the entry cap retained *only* where a caller passes one, and `namespace_usage_at` (`:3740`) keeps the cap where it belongs. The wedge claim holds for a directory-dominated tree, because `rmdir` happens only on the deferred "leaving" event, so an interrupted scan of a deep chain removes nothing. See MUST-3 for the accounting claim attached to this fix. |
| 3 | `--propagation private` requested explicitly and re-proved from `mountinfo` | **Closed at the enforcement layer.** `--propagation private` is present on all three launchers (`run_source_keyed_cargo_gate.py:2958`, `build_source_bound_rust.py:496`, `run_timed_gate.py:1051`); `util-linux 2.37.2` on this host accepts it. `prove_private_propagation` runs before the first mount (`readonly_source_exec.py:833`), in the post-drop phase (`:867`), and after the mount phase (`:1275`); because a new mount inherits its parent's propagation, one proof before the first mount is sufficient for the security property even though the doc says "before and after every mount". The separator search from index 6 is correct, and my probe confirms a `/-` mount point with `shared:3` is still reported. See MUST-2 for the identity-recording gap and NIT-1 for a parser inconsistency. |
| 4 | Isolation-tool descriptors closed before Cargo, retaining only what the child environment names | **Not closed as stated.** See MUST-1. Four of the enumerated descriptors are closed; the pinned `unshare`, parent-death helper, policy supervisor, and unselected toolchain executables survive the exec. |
| 5 | Cleanliness separated from unchangedness; dirty recapture returns a BLOCK result | **Closed.** Admission still requires clean twice — `main()` pre-checks with `require_clean=True` (`:8256`) and `run_source_keyed_gate` re-asserts `before["dirty"]` with the `dirty-candidate-not-trusted` blocker (`:6945`) — while both recaptures (`:7128`, `:7378`) now use the full identity, so a candidate dirtied while waiting for locks or by the gate itself yields `candidate-changed-before-gate` / `candidate-changed-during-gate` with `candidate_after` evidence. I verified the digest is stable across the change: for a clean tree `dirty_identity` and `clean_dirty_identity` build the same tagged digest from the same empty components, so the source key is not perturbed by capturing the fuller form. |
| 6 | Global execution lock preserved; parallelism prerequisites stated concretely | **Closed.** `cargo_registry_locks` (`:3330`) still takes the global execution lock unconditionally before the registry lock, and the lock order is evidence (`main`, `:8340`) → global → registry → namespace (`:7086`) → provenance, matching the documented order. `cleanup_ephemeral_namespace` is the only function that self-acquires the registry lock and has no in-gate call site, so there is no inversion against the namespace lock. The four prerequisites are now stated as conjunctive requirements rather than as an aspiration. |

The three "stale test" updates are legitimate rather than accommodations. The provenance rollback
test now targets the barrier by observable state (fail the first directory `fsync` taken once the
destination already holds the new bytes) instead of counting `fsync` calls, which is a strictly
stronger predicate; the launch-argv test now pins the whole pinned-FD chain including `pass_fds`
membership for each dispatched executable; the capability-probe test tracks the current
`validate_locked_canonical_provenance` signature, whose absence was the checkpoint `AttributeError`.

## 4. Adjudication of the ten MUST/CRITICAL classes and the checkpoint hypotheses

| Class / hypothesis | Verdict | Basis |
|---|---|---|
| Same-UID host mutation via staging paths or `/proc` child root | **Refuted as an escalation; already documented as a boundary.** Staging is descriptor-relative: `copy_snapshot_tree` (`readonly_source_exec.py:250`) walks from the pinned snapshot dirfd with per-component `O_NOFOLLOW`, and the copy is verified against file count, logical bytes, modes, and per-file SHA-256 inside the namespace. `-p --mount-proc` gives the child a fresh procfs for its own PID namespace, so no host `/proc/<pid>/root` path exists inside. The host Git administration directory is not masked, but a resulting HEAD/dirty change is caught by the post-gate recapture, and the doc already states no-follow/inode checks are drift detection, not authentication against a same-UID actor. |
| Transient rename/replace reaching the copied snapshot | **Refuted.** The snapshot is read through the pinned dirfd, then content-verified against the manifest digest inside the namespace; a swap has to survive SHA-256, not just a `stat` comparison. |
| Mount propagation / recursive flags | **Refuted after the fix, with a recording gap (MUST-2).** Propagation is requested and re-proved. Masks use `tmpfs` over the mount point, which hides submounts outright, so the non-recursive `--bind` + `remount,bind,ro` pattern shadows rather than exposes any submount; the failure direction of a non-recursive bind here is a missing subtree, not a writable one. |
| Post-drop recreation of user namespaces / capabilities | **Mechanism CONFIRMED, consequence REFUTED.** Live probe: after `setpriv --no-new-privs --inh-caps=-all --ambient-caps=-all --bounding-set=-all`, `CapEff`/`CapBnd` are `0000000000000000` with `NoNewPrivs: 1`; a direct `unshare(CLONE_NEWUSER\|CLONE_NEWNS)` succeeds and restores `CapEff`/`CapBnd` to `000001ffffffffff`. From that nested namespace, `umount2` of the mask fails `EINVAL` (both plain and `MNT_DETACH`), the read-only bind cannot be remounted (`EPERM`, and writes then fail `EROFS`), and the host mask is untouched. The protection is the kernel's locking of mounts copied into a child user namespace, not the capability drop. See SHOULD-1. |
| Forged Cargo test-summary text or zero-test spoof | **Refuted within the documented boundary.** Direct analyzer probes: forged summary before any harness → `test-summary-without-running-harness`; forged summary followed by a genuine zero-test harness → additionally `zero-selected-target-harness`; ignored-only harness → `executed_tests=0`; harness outside the artifact namespace and a `..` escape → `cargo-test-harness-outside-target`; running/summary count mismatch → `test-summary-running-count-mismatch`; a second harness under a focused filter → `unexpected-target-harness-count`. All `BLOCK`; the genuine single-harness control is the only `PASS`. `cargo_test_contract` rejects a filter without `--lib`/`--bin`/`--test`. The residual — a test body forging a summary and exiting before the real one — is already stated as out of scope for trusted source. |
| Cleanup TOCTOU / missing `openat2`/`renameat2`/inode/mount fences | **Bounded, not a defect.** `reopen_directory_at` (`:3661`) reopens per component with `dir_fd` + `O_NOFOLLOW` and rejects `.`/`..`/embedded separators, and each event carries the expected `(st_dev, st_ino)` so a cross-device or swapped component blocks. Renames use `src_dir_fd`/`dst_dir_fd` with an explicit prior absence check rather than `RENAME_NOREPLACE`; Python 3.10 exposes neither `openat2` nor `renameat2`, and the residual window is the same-UID actor the boundary section already excludes. |
| Hardlink / no-`xdev` allocated-byte accounting | **CONFIRMED as an accuracy defect in the new claim, safe in direction.** See MUST-3. |
| Partial writes / overflow / process-group termination / same-FD durable reparse | **Refuted.** `run_timed_gate.py` caps the log at 256 MiB (`:30`), starts the child in a new session (`:1066`) and terminates by `killpg` (`:917`, `:925`), re-hashes the complete log through the same descriptor and compares against the streamed digest (`:1189`–`:1201`), then sets mode `0400` and fsyncs. Analyzer counts are bounded before `int()` conversion by both digit length and value (`:6380`). The live resource-guard and bounded-output tests pass. |
| Executable / policy-module / environment binding | **Refuted.** Every executable is dispatched by inherited descriptor; `project_runtime_environment` re-projects the child environment and blocks on `runtime-environment-contract-mismatch` if the observed contract differs from the admitted one; `preloaded_policy_identity` (`:2049`) re-verifies device, inode, mode, uid, nlink, size, both timestamps, and the digest through the held descriptor. Precision limit in NIT-2. |
| Key separation; dirty recapture / result ordering; reservations and lock order | **Refuted.** `build_source_identity` (`:3048`) binds the full candidate state including Git-administration paths and HEAD; `build_artifact_identity` (`:3067`) binds `candidate_root` + exact tree + Rust manifest and excludes commit and Git-administration state, as documented. `readonly_isolation_identity` — which contains the content digests of the supervisor, helper, and every policy module — is in both keys, so any change to this commit's own files rotates the keys. Recapture ordering and lock order are as in section 3, rows 5 and 6. |

## 5. Findings

### MUST-1 — the descriptor-closure fix does not achieve the invariant it states, and its control cannot detect that

`docs/development/tools/readonly_source_exec.py:939`–`:961`; contract text at
`docs/development/ai/source-keyed-cargo-gates.md:176`–`:181`.

The post-drop phase closes a hard-coded list of fourteen descriptors. Everything else in
`--pass-fds` survives into `os.execve(primary_fd, …)`. The helper is never told the numbers of the
pinned `unshare` and parent-death-helper descriptors, and `pinned_tool_files`
(`run_source_keyed_cargo_gate.py:2670`) additionally pins the policy supervisor and *every* toolchain
executable — `cargo`, `rustc`, `rustdoc`, `clippy-driver`, `rustfmt`, `cargo-clippy`, `cargo-fmt` —
of which only `primary_fd` plus the two-to-four named by `CARGO`/`RUSTC`/`RUSTDOC`
(`:7200`–`:7224`) are legitimately retained.

Reproduced live (`/tmp/probe_fd_leak.py`, which drives the real helper through the real
`unshare … --propagation private` launch and reports the inherited set instead of asserting a
four-element subset):

```json
{"still_open": {
  "extra-toolchain-cargo-fmt": "/usr/bin/true",
  "pdeath-helper": ".../docs/development/tools/pdeath_exec.py",
  "policy-supervisor": ".../docs/development/tools/run_source_keyed_cargo_gate.py",
  "primary": "/usr/bin/python3.10",
  "unshare": "/usr/bin/unshare"}}
```

Only `primary` belongs there. The doc's "retaining only the descriptors the child environment itself
names" and the helper's own comment "the isolation-tool descriptors are all closed first" are both
false: `unshare` and the parent-death helper are members of `isolation["tools"]`
(`run_source_keyed_cargo_gate.py:2527`) and are never closed.

Failure scenario: a `cargo test` gate execs Cargo with an inherited, still-open descriptor to
`/usr/bin/unshare`, to the gate supervisor's own source, and to three unselected toolchain drivers.
Any of them is executable by a `build.rs` as `/proc/self/fd/<n>`.

I state the practical severity honestly: it is low. Inside the namespace `/usr` is not masked, so
`/usr/bin/unshare` is already reachable by path — I executed it from inside a post-drop namespace in
the capability probe — and the toolchain root is mounted read-only but readable. No *capability* is
gained that the documented non-hermetic boundary does not already concede. What is defective is the
claim and the control: the fix asserts an exact invariant it does not implement, and
`test_readonly_helper_mounts_snapshot_and_drops_namespace_caps`
(`test_ai_tools.py:2335`–`:2342`) asserts `len(closed) == 4` over the four descriptors the patch
enumerates, so the assertion passes while the invariant is violated for at least four more. A
control that can only confirm the change already made is not a negative control for the property.

Smallest correct fix: `--pass-fds` is already parsed into `inherited` (`:772`). Close every
descriptor in `inherited` that is not in `retained`, and change the test to assert that the exec'd
command's entire inherited set equals `retained`, so the assertion is bounded by the property rather
than by the enumeration.

### MUST-2 — the recorded isolation identity still declares the pre-fix `unshare` flag list

`docs/development/tools/run_source_keyed_cargo_gate.py:2584`.

```python
"unshare_flags": [
    "-Urnmpf",
    "--mount-proc",
    "--kill-child=SIGKILL",
],
```

Confirmed live: `isolation_identity()["unshare_flags"]` returns exactly that list, while
`isolated_command` launches `-Urnmpf --mount-proc --kill-child=SIGKILL --propagation private`.

`readonly_isolation_identity` is embedded verbatim in the source key payload, the artifact key
payload, and the published result JSON, so every result and every release provenance produced by
this candidate records an isolation identity that omits the one flag this commit added and the doc
now leans on ("Propagation is requested explicitly rather than inherited from a `util-linux`
default"). A reviewer or consumer auditing the recorded identity to answer "was propagation pinned?"
gets the pre-fix answer.

This does not weaken enforcement: the helper re-derives propagation from `mountinfo` and fails
closed, and because the isolation identity contains the content digests of the supervisor and the
helper, the keys do rotate whenever this argv changes. So it is an evidence-accuracy defect, not a
binding hole — but the commit's own rationale is that a default is not evidence, and the recorded
evidence of the launch is now wrong. Fix: add the flag to the declared list (and, since the two are
required to agree, assert the equality between the declared list and the launched argv in the
existing launch-argv test).

### MUST-3 — the new "exact reclaimed bytes charged back against the aggregate budget" claim is wrong in both halves

`docs/development/ai/source-keyed-cargo-gates.md:354`–`:357`; docstring at
`docs/development/tools/run_source_keyed_cargo_gate.py:5003`–`:5005`; return payload at `:5922`.

(a) Nothing charges it back. `reclaimed_allocated_bytes` has no consumer: `grep` across the tools
finds it only at its production site and in the new test. Aggregate allocation is always
re-measured by `inspect_registry`/`namespace_usage_at` after cleanup (`:7683`, `:7800`), which is the
stronger design — but the doc describes an accounting flow that does not exist.

(b) It is not exact. Both the removal walk (`:5079`) and the pre-measure scan (`:3802`) add
`st_blocks * 512` per directory entry with no inode de-duplication, and Cargo's normal uplift creates
a second hard link for every produced executable. Measured (`/tmp/probe_hardlink.py`, one 4 MiB
payload with `st_nlink == 2`):

```text
du -s --block-size=1 (dedups by inode):   4194304
namespace_usage_at allocated_bytes:       8388608
remove_tree_contents_at allocated_bytes:  8388608
overstatement vs du:                      4194304 bytes
```

The two figures agree with each other, so the 32 GiB budget stays internally consistent and errs
toward blocking earlier — the safe direction, and the reason this is a claim defect rather than a
retention bug. But `resource_accounting_complete: True` is now returned unconditionally alongside a
figure that can be double the reclaimed allocation, and the same convention presumably underlies the
"measured 3,204,700,000-byte maximum" sizing basis. Fix: either de-duplicate by
`(st_dev, st_ino)` when `st_nlink > 1` in both walks, or state the convention explicitly ("measured
under the same per-entry convention as the aggregate scan, which counts a hard-linked inode once per
link") and drop "charged back against the aggregate budget".

### SHOULD-1 — the boundary section should say the capability drop is not a durable capability boundary

`docs/development/ai/source-keyed-cargo-gates.md:176`–`:181` and `:401`–`:422`.

Step 9 says the gate clears "effective, permitted, inheritable, ambient, and bounding capabilities"
and "proves protected remounts fail". Both are true and both were verified. What a reader may
reasonably but wrongly infer is that a build script therefore cannot hold `CAP_SYS_ADMIN` again.
Measured, in the post-drop namespace:

```text
before:               CapEff: 0000000000000000  CapBnd: 0000000000000000  NoNewPrivs: 1
after unshare(NEWUSER|NEWNS): CapEff: 000001ffffffffff  CapBnd: 000001ffffffffff  NoNewPrivs: 1
umount mask / MNT_DETACH:     EINVAL
remount read-only bind rw:    EPERM, subsequent write EROFS
```

The bounding-set drop is not what holds the masks and the read-only source; the kernel's locking of
mounts copied into a child user namespace is. That distinction belongs in the explicit-boundary
section, alongside the existing statement that the mount namespace is not a filesystem sandbox,
because a future change that relies on the capability drop for a stronger claim would be unsound.

### SHOULD-2 — the permanent record misstates both test counts

`docs/development/history.md:16` records "checkpoint時点でtest 148中10件failだったものを157件全pass". The
commit message says the same "10 failures out of 148" and then "159 tests". Measured: the checkpoint
is `Ran 148 tests — FAILED (failures=4, errors=8)`, i.e. **12** failing, and the candidate is
`Ran 159 tests — OK`. Two of the three numbers in the permanent history row are wrong. Correct them
to 12 and 159 (the commit message's 159 is already right).

### NIT-1 — the propagation parser silently accepts a line it declares unparseable

`docs/development/tools/readonly_source_exec.py:41`–`:71`. The docstring and
`test_unparseable_mountinfo_line_is_not_read_as_private` establish the rule that a line without a
separator must raise rather than read as "not shared". But the `len(fields) < 7: continue` guard
short-circuits before that rule applies, so a shorter malformed line is silently treated as private:

```text
mount_propagation_tags("24 20 0:1 / / shared:1\n") -> []
prove_private_propagation(...)                     -> ACCEPTED
```

A real `/proc/self/mountinfo` line always has at least ten fields, so this is unreachable in
practice; it is worth one line for consistency with the rule the function states, and a matching
counterexample.

### NIT-2 — policy-module reuse verifies the file, not the loaded code

`docs/development/tools/run_source_keyed_cargo_gate.py:328`–`:365`. Reuse admits a live module whose
`__file__` equals the pinned path; the digest recorded in the isolation identity is of the file's
*current* bytes, not of the code the interpreter actually executed. If the file changed between an
earlier import and the preload, the recorded `policy-*` digest describes bytes the gate is not
enforcing. That requires a same-UID edit inside the process lifetime, which is the excluded actor,
and in production the reuse path is normally inert: `build_source_bound_rust.py` launches the gate as
a subprocess rather than importing it, so `-I -S` starts with none of these names bound. Separately,
a launcher run as `__main__` still yields a second instance under its policy module name, which the
fix cannot address. The doc sentence "A module already bound in the interpreter is reused after that
verification" reads as if the module were verified; "after verifying the pinned file and the module's
origin path" would be exact.

## 6. What I ran

```text
python3 -m unittest test_ai_tools                    # candidate: Ran 159 tests, OK, 22.677s
python3 -m unittest test_ai_tools   (in /tmp/ckpt)   # checkpoint: Ran 148 tests, FAILED (4 failures, 8 errors)
python3 -m py_compile run_source_keyed_cargo_gate.py readonly_source_exec.py \
        build_source_bound_rust.py run_timed_gate.py test_ai_tools.py     # clean
python3 docs/development/tools/check_ai_acceleration_docs.py             # passed
python3 docs/development/tools/check_ai_development.py                   # passed (7 subchecks)
/tmp/probe_fd_leak.py        # live unshare+mount+drop: inherited descriptor set at exec (MUST-1)
/tmp/probe_userns_regain.py  # live: setpriv drop, then util-linux nested userns attempts
/tmp/probe_userns_raw.py     # live: raw unshare(CLONE_NEWUSER|CLONE_NEWNS) capability regain
/tmp/probe_adversarial.py    # analyzer spoofs, mountinfo edge cases, recorded unshare_flags
/tmp/probe_hardlink.py       # hardlink allocated-byte accounting vs du (MUST-3)
unshare --version / setpriv --version                                    # util-linux 2.37.2
```

No Cargo, `rustc`, Clippy, or `rustfmt` invocation. Host at review time: 14 CPUs, ~19 GiB RAM with
~10 GiB available, 16 GiB swap — consistent with the ledger's one-heavy-gate posture, and a further
reason the global execution lock retained by closure 6 is the right control today.

## 7. Verdict

`REQUEST_CHANGES`.

Five of six claimed MUST/CRITICAL closures are verified real, each against a control that demonstrably
failed at the checkpoint, and none of the ten reviewed MUST/CRITICAL classes yields a privilege
escalation or an evidence-forgery path beyond the boundaries the document already states. The gate's
fail-closed posture is intact and the global lock is correctly retained.

The change is not acceptable as written because closure 4 does not implement the invariant it
publishes and its new control is scoped so that it cannot fail while that invariant is violated
(MUST-1), the isolation identity that every result and provenance record embeds still declares the
pre-fix launch flags and omits the propagation flag this commit adds (MUST-2), and the new retention
accounting text asserts an exactness and a budget flow that neither exists nor holds for the normal
Cargo hard-link case (MUST-3). All three are small, local fixes in the same files; MUST-1 additionally
requires the test to be rewritten against the property rather than the enumeration. SHOULD-1,
SHOULD-2, NIT-1 and NIT-2 should be taken in the same pass.

The failed Fugu checkpoint-review attempt is recorded as an infrastructure incident with no repo
residue and no salvaged evidence. It is not a finding against this candidate, and no part of this
verdict derives from it.
