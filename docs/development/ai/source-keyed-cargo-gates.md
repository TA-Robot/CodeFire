# Source-keyed Cargo gates

This is the mandatory project-side entry point for trusted Rust validation.
The shared Cargo target root is an allocation pool only; it is never itself
an executable provenance namespace.

## Why the shared root alone is not evidence

Cargo's target directory is a build namespace, not a multi-worktree
provenance database. Relative local-package paths and mtimes can let one
worktree reuse a newer artifact built from another worktree.

The 2026-07-31 File Doctor gate demonstrated the failure:

- candidate `cd130e544af0476162be153db58a0a840b75b134` contained 16
  `fn live_file_doctor_*` tests;
- `cargo test ... live_file_doctor` returned success in 0.22 seconds without
  compiling, with 0 passed and 794 filtered;
- the executed `debug/deps/codefire_rs-2521b6556af2330d` listed 794 tests and
  no `live_file_doctor` test; and
- its dep-info named the Rust HTTP worktree as its only absolute worktree
  source root.

Exit code zero, an artifact filename, a filter count, and mtime are therefore
not candidate evidence.

## Required entry point

Create the three private roots once. They must be real, owner-controlled
directories and must not overlap the candidate, target root, Git common
directory, or linked-worktree Git administration directory:

```bash
install -d -m 0700 \
  /workspace/.cache/codefire-gate-evidence \
  /workspace/.cache/codefire-source-snapshots \
  /workspace/.cache/cargo-target-shared
```

Use the exact low-memory profile and give the wrapper an external, new log
and result path. The low-level command below is admitted only from an
already trusted, loader-scrubbed shell/service; the absolute interpreter is
part of the recorded-command contract:

```bash
export RUSTUP_HOME=/home/devuser/.cache/codefire-rust-1.97/rustup
export CARGO_HOME=/home/devuser/.cache/codefire-rust-1.97/cargo
export RUSTUP_TOOLCHAIN=1.97.0-x86_64-unknown-linux-gnu
export CARGO_TARGET_DIR=/workspace/.cache/cargo-target-shared
export CARGO_BUILD_JOBS=2
export RUST_TEST_THREADS=2
export CARGO_INCREMENTAL=0

/usr/bin/python3.10 -I -S \
  docs/development/tools/run_source_keyed_cargo_gate.py \
  --repo "$WORKTREE" \
  --timeout 2400 \
  --lock-timeout 21600 \
  --log /workspace/.cache/codefire-gate-evidence/file-doctor.log \
  --result /workspace/.cache/codefire-gate-evidence/file-doctor.json \
  -- cargo test -p codefire-cli --bin codefire-rs --locked live_file_doctor
```

`--lock-timeout 21600` permits a six-hour wait behind a healthy Rust lane; it
is not a six-hour process timeout. Choose the process `--timeout` for the
specific gate. Both limits must be finite.

`run_timed_gate.py` rejects direct or known wrapped Cargo/rustc/clippy-driver
dispatch. It remains the entry point for trusted non-Rust argv gates.
Known shell `-c` and code-interpreter `-c`/`-e` payloads are rejected as
unresolved wrappers even when their text looks harmless; static inspection
cannot prove that generated code will not start Rust. This is conservative
dispatch control, not an `execve` reference monitor: any other admitted
program or script is trusted not to spawn Rust tools.

For the canonical release binary, prefer
`/usr/bin/python3.10 -I -S
docs/development/tools/build_source_bound_rust.py`. That launcher passes a
positive pre-interpreter environment, owns the evidence lock through gate
completion and provenance publication, and revalidates the complete
result/log/export chain. A direct source-gate invocation cannot undo loader
state inherited before Python starts.

Trusted execution is clean-only. A tracked, staged, or untracked change is
`BLOCK`, even with `--ephemeral`, because the source mount must be an exact
HEAD Git-blob snapshot. `--ephemeral` is available only for a clean candidate
whose artifact namespace should be removed after the gate:

```bash
/usr/bin/python3.10 -I -S \
  docs/development/tools/run_source_keyed_cargo_gate.py \
  --repo "$WORKTREE" --ephemeral --timeout 2400 --lock-timeout 21600 \
  --log /workspace/.cache/codefire-gate-evidence/focused.log \
  --result /workspace/.cache/codefire-gate-evidence/focused.json \
  -- cargo test -p codefire-cli --bin codefire-rs --locked focused_filter
```

## Source, artifact, and execution identities

The evidence source key binds:

- canonical worktree root, Git common directory, and linked-worktree Git
  directory;
- exact HEAD commit and tree;
- tagged porcelain, worktree/index binary diffs, and untracked-content
  evidence;
- the Rust input manifest covering Cargo manifests, lockfile, `.cargo`,
  toolchain files, and `crates/`;
- the admitted execution profile and exact Rust/isolation tool identities;
  and
- a stable projection of the environment actually passed to the gate:
  literal admitted values, pinned-tool identities for `/proc/self/fd/<n>`
  dispatch, and symbolic policies for the derived target, key, `HOME`, and
  `TMPDIR` paths.

The artifact key deliberately excludes commit identity and Git-administration
state, but binds the canonical worktree root, exact full Git tree, Rust input
manifest, allowlisted child environment, profile, selected executable
identities, and isolation identity. An identical tree replayed by a different
commit can reuse the same local cache namespace, while a tree change or a
different worktree cannot. This is a local trusted-host cache identity, not a
content-complete or reproducible-build identity; the non-hermetic boundary is
spelled out below.

The execution key additionally binds the full evidence source key, argv, and
timeout. The only Cargo execution paths are:

```text
/workspace/.cache/cargo-target-shared/source-keys/<64-lowercase-hex-artifact-key>
```

The wrapper accepts the shared root only as the ambient allocation root and
replaces it before execution. It rejects another key, `--target-dir`,
`--manifest-path`, `--config`, toolchain overrides, an unfrozen lockfile, and
non-validation subcommands. `cargo fmt` requires `-- --check`; Clippy
fix/dirty/staged modes and `cargo doc --open` are rejected. `cargo test
--no-run` is compile-only and cannot satisfy trusted test evidence, so it is
also rejected.

## Exact source and tool isolation

For every admitted gate the wrapper:

1. reads the exact HEAD tree entries, blob OIDs, modes, and sizes;
2. rejects Git symlinks/submodules and symlinked or hard-linked Rust inputs;
3. streams `git archive <exact-commit>` into a unique partial directory using
   exclusive no-follow file creation;
4. verifies every extracted path and digest, writes a canonical marker, fsyncs
   the files/directories, makes the tree read-only, and atomically renames it
   to `ready-*`;
5. pins the candidate, snapshot, toolchain, target, and every executable by
   device/inode/mode/uid/size/link-count/digest;
6. arms the parent-death signal through a pinned helper that re-reads
   `getppid` after `prctl` and refuses to exec an already-orphaned gate,
   then uses pinned `setpriv --pdeathsig SIGKILL` before
   `unshare -Urnmpf --mount-proc --kill-child=SIGKILL --propagation private`,
   giving the gate private user, network, mount, PID, and proc namespaces;
   Cargo is namespace PID 1, manager death kills `unshare`, and
   namespace-init death kills remaining descendants. `setpriv` alone would
   arm the signal only after the fork, so a manager that died in that
   window would leave an unsupervised Cargo namespace;
7. re-derives private mount propagation from `/proc/self/mountinfo` before
   and after every mount, refusing any `shared:` or `master:` tag, then
   creates a size-bounded private tmpfs at the original candidate path,
   copies only the verified snapshot manifest into it, verifies file count,
   logical bytes, modes, and SHA-256, excludes the source-key marker, and
   remounts the private copy read-only. Propagation is requested explicitly
   rather than inherited from a `util-linux` default, because that tmpfs
   covers the candidate worktree's own path and a shared namespace would
   mask the developer's real worktree on the host for the whole gate;
8. masks the host snapshot and evidence roots, mounts the admitted target root
   read-only with only the selected artifact child writable, mounts marker,
   lock, and run-state files read-only, and mounts the toolchain root
   read-only;
9. uses pinned `setpriv` to set `no_new_privs` and clear effective, permitted,
   inheritable, ambient, and bounding capabilities, proves protected remounts
   fail, admits only mapped-root/overflow supplementary GIDs (the user
   namespace denies `setgroups`), and closes the complement of the retained
   set over the whole inherited `--pass-fds` set before Cargo — not an
   enumerated list of descriptors, because the manager pins more than the
   helper is told about by name (the policy supervisor, `unshare`, the
   parent-death helper, and every unselected toolchain driver), and each of
   those is a program a build script could exec through `/proc/self/fd`.
   Of the descriptors declared in `--pass-fds`, what survives is stdio, the
   primary tool, and the ones the child environment itself names. That
   qualifier is load-bearing: the close set is the complement over the
   *declared* set, so a descriptor the kernel inherits but `--pass-fds`
   omits would survive. The gate cannot reach that state today — the
   `--pass-fds` string and the kernel-level `pass_fds=` argument are built
   from the same `pinned["pass_fds"]` tuple, and the timed runner forwards
   it unmodified for PID-namespace-isolated callers — so it is a bound on
   the wording, not a live gap. The in-child control is deliberately
   stronger than the mechanism: it enumerates the whole of
   `/proc/self/fd`, so a future divergence fails the test even where the
   implementation would leak; and
10. executes pinned Cargo, cargo-clippy, or cargo-fmt by inherited file
    descriptor and gives Cargo pinned `/proc/self/fd/<n>` paths for
    rustc/rustdoc/clippy-driver/rustfmt, then revalidates tool, directory,
    snapshot-content, marker, and candidate identities.

The child environment is built from a positive allowlist of build/profile
inputs and locale settings. Ambient loader/Python startup hooks and tool
overrides are rejected. Cloud credential variables and general ambient
variables (for example AWS credentials, Google credential-file paths,
`KUBECONFIG`, and `NETRC`) are not passed to Cargo or `build.rs`.
Ambient `HOME` and `TMPDIR` are discarded. Per-run owner-only replacements
are created below the selected artifact namespace, pinned during execution,
and removed before a successful namespace commit.

All Git control calls use a positive environment, disable replacement
objects/grafts, fsmonitor, hooks, global/system config, attributes, excludes,
untracked cache, and automatic maintenance, and have finite wall deadlines
plus bounded output. The Git-control policy is itself part of the isolation
identity.

The snapshot root is separately bounded to 200,000 files, 2 GiB logical
bytes per snapshot, and 8 GiB actual aggregate allocation. Pre- and
post-materialization checks also preserve the 100 GiB filesystem free floor.
Every terminal path removes its exact snapshot by pinned quarantine rename
and no-follow dirfd traversal; the marker is removed last. Before a new
materialization, crash-left `.tmp-*` and `ready-*` snapshots are recovered
under the global lock.

## Test and evidence contract

The complete lock order is evidence-root lock, global execution lock,
registry lock, artifact namespace lock, then release-provenance publication
lock. The evidence lock remains outermost through final result publication
and, for the release wrapper, provenance fsync. The global lock is a
temporary OOM control for the current roughly 20.97 GB host; it serializes all
Cargo gates.

The global lock stays until bounded parallelism is *proven* safe, not until
it looks safe. Removing serialization requires all of: cgroup `memory.max`
and `pids.max` per lane, a filesystem project quota per artifact namespace,
a measured per-lane peak-RSS reservation admitted against live memory and
swap rather than a fixed lane count, and evidence that concurrent lanes on
distinct artifact keys never contend for the same Cargo registry or
toolchain lock. Until every one of those exists, serialization is the
control. Do not widen it by assumption, and do not infer headroom from a
gate that happened to fit.

The gate's policy modules are pinned the same way its executables are: each
is read no-follow from its exact canonical path, size- and digest-checked,
and its descriptor held open for the process lifetime. A module already
bound in the interpreter is reused after verifying the pinned file and the
module's origin path, rather than re-executed. Loading a second copy would
leave two divergent instances of the same policy state, so the profile an
embedder configured and the profile the gate enforced could silently
differ; a name bound to any other origin fails closed.

Be precise about what reuse verifies. The recorded digest is of the pinned
file's current bytes, not of the code the interpreter actually executed at
its earlier import. If the file changed between that import and the
preload, the recorded `policy-*` digest describes bytes the gate is not
enforcing — which requires a same-UID edit inside the process lifetime, the
actor the boundary section already excludes. In production the reuse path
is normally inert: the release launcher runs the gate as a subprocess under
`-I -S`, so none of these names are bound when the preload runs. A launcher
executed as `__main__` still yields a second instance under its own policy
module name, which this mechanism cannot address.

Logs and result JSON must be distinct direct children of the fixed dedicated
evidence root and outside the candidate, target, and Git administration
paths. Nested paths and pre-existing destinations are rejected; path
validation and reservation happen only while the evidence lock is held. The
timed runner opens one exclusive no-follow log descriptor, streams at most
256 MiB through it, hashes and analyzes those same bytes, re-hashes the
complete log through that descriptor, changes it to mode `0400`, and fsyncs
the file and parent. Result publication is exclusive; a post-link validation
or parent-fsync failure removes only the exact inode it published, never a
replacement inode.

Before any gate or export, the result path is durably reserved as mode `0400`
canonical `BLOCK`. Final publication replaces that reservation only after
all acceptance predicates. A crash before replacement therefore cannot
leave an absent or stale PASS beside a newly exported artifact.

For `cargo test`, the analyzer correlates each Cargo `Running ... (<target
binary>)` line with its `running N tests` line and exact libtest summary. For
trusted test source, every selected harness must report at least one
non-ignored test. A focused filter requires exactly one explicit `--lib`,
`--bin`, or `--test` harness; an unexpected extra harness, outside-target
binary, mismatched count, failed summary, ignored-only harness, or genuine
zero-test harness is `BLOCK`. Prepending a fake passing summary cannot hide a
later zero-test harness. Parser errors are deduplicated, numeric counts are
bounded, at most 1,024 summaries are admitted, and only 128 bounded harness
records plus a digest and aggregate counters are retained. The final result
JSON has a hard 1 MiB publication limit. On Python 3.10, where stdlib
`tomllib` is unavailable, escaped or multiline TOML strings are conservatively
`BLOCK` because the fallback harness audit cannot safely interpret them.

This correlation is transcript validation, not a cryptographic structured
test protocol. An adversarial test body can write a forged libtest-looking
summary and terminate its process before the real harness summary. Therefore
the non-vacuity claim is valid only for reviewed/trusted candidate source; a
future hostile-source lane needs a supervisor-owned structured side channel.

The first log line binds source key, artifact key, execution key, commit,
tree, and dirty digest. The result binds complete log size/SHA-256 and
`SHA256(source_key || NUL || log_bytes)`. Once the result path passes
containment, admission and unexpected failures are durably written as
canonical `BLOCK` JSON.

Cleanliness and unchangedness are separate predicates. Admission requires a
clean candidate and reports `dirty-candidate-not-trusted`. After the locks
are taken, and again after the gate, the candidate's *complete* dirty
identity is recaptured and compared with the admitted one, reporting
`candidate-changed-before-gate` or `candidate-changed-during-gate`. The
recapture therefore has to tolerate a dirty tree: a gate that dirtied the
candidate must produce a `BLOCK` result carrying both the admitted and the
observed state, not a raised admission error that discards the evidence of
what the candidate became.

Each persistent target has a durable `PREPARED` → `RUNNING` → `COMMITTED`
state file. Only `COMMITTED` namespaces are reusable. Under the global and
registry locks, creation first publishes an fsynced `.prepare-*` directory by
atomic rename. Startup removes interrupted prepare/quarantine entries and
every unmarked, corrupt, invalidated, missing-state, `PREPARED`, or `RUNNING`
namespace across all keys before admission accounting. A crash cannot turn a
partially built target into reusable evidence or leave incomplete keys
consuming the budget forever. A normal terminal `BLOCK` removes its exact
incomplete namespace before releasing the locks; startup recovery covers
abrupt host/process loss.

Release publication uses `--export-artifact` and `--export-path`. The wrapper
copies and hashes the requested single-link executable under the same locks,
but does not replace the canonical export until the gate, candidate recapture,
snapshot cleanup, log analysis, retention record, run-state commit, and final
disk predicates have all passed. Replacement is same-directory and
inode-checked; crash-left export temporary/previous entries are recovered on
the next export. `build_source_bound_rust.py` uses this path and accepts the
binary only when stdout, result JSON, export digest, and no-follow binary
identity agree. Export destinations are limited to Git-untracked,
Git-ignored `target/**` paths and are checked before and immediately before
publication.

Release provenance is not accepted by its JSON alone. The consumer
first reads the bounded canonical provenance only to identify its evidence
root, acquires the already-existing evidence lock without creating one, then
re-reads the same canonical bytes/digest under that lock. While retaining the
lock it no-follow reads the mode-`0400` source-gate result and bounded log,
verifies every recorded size/digest, and rechecks PASS, argv, candidate
commit, source/artifact keys, runtime guard, committed namespace, export
evidence, current binary identity, and a final canonical-provenance read.
Missing/changed evidence, a missing lock, or a producer generation change
invalidates `rust-source-bound-binary`.

## Disk retention and cleanup

The policy uses measured allocated bytes under the registry lock; retention
JSON is never trusted as the allocation source.

1. Persistent namespaces have a 32 GiB aggregate budget
   (34,359,738,368 bytes), not a count cap. Its basis is the measured
   3,204,700,000-byte maximum times eight reference candidates times 1.25
   headroom. Eight is sizing evidence, not a namespace limit.
2. A new persistent artifact key reserves 4,005,875,000 bytes at admission.
   A new persistent or ephemeral namespace must also leave at least 100 GiB
   free after that reservation.
3. Existing reuse is not a bypass. Admission blocks if actual aggregate
   allocation already exceeds 32 GiB or current free space is below 100 GiB.
   A new persistent key also blocks when aggregate plus reserve exceeds the
   budget.
4. While the process is live, a one-second supervisor samples the selected
   namespace, aggregate allocation, and filesystem free floor. Crossing the
   per-run growth bound, 32 GiB aggregate budget, or 100 GiB floor terminates
   the process group and records `BLOCK`.
5. After the gate, the wrapper measures the namespace and aggregate again,
   then repeats the measurement after retention and run-state control writes.
   Any aggregate or free-floor violation is `BLOCK`. A newly created
   persistent namespace is quarantined and removed; an existing namespace is
   retained with explicit blocking evidence. Ephemeral namespaces are
   removed but the policy violation still blocks the gate.
6. Cleanup atomically renames only the exact marked namespace to a quarantine
   name under the registry lock, validates its pinned identity, recursively
   removes entries without following links, removes the marker last, fsyncs
   the registry, and proves both original and quarantine names are absent.
   Reclaimed allocated and logical bytes are measured from that same
   descriptor-relative removal walk, so every removed entry is measured
   without a second traversal of a namespace that can hold gigabytes.
   Nothing charges that figure back against the aggregate budget: the
   aggregate is always re-measured under the registry lock, and the
   reclaimed figure is evidence only.
   Allocated bytes use one convention throughout — `st_blocks * 512` summed
   per directory entry, with no inode de-duplication — so a hard-linked
   inode is counted once per link. Cargo's uplift gives every produced
   artifact a second link, so the reclaimed figure is an upper bound on the
   allocation a removal actually returns, roughly double for the uplifted
   files. This is deliberate: the removal walk and the aggregate scan share
   the convention, so budget comparisons stay internally consistent and err
   toward blocking earlier, and the measured sizing basis below was derived
   under it. The result reports the convention, the number of removed
   entries that carried extra links, and the bytes counted more than once,
   so a consumer reads the bound instead of inferring exactness. Switching
   to inode de-duplication would loosen the budget and requires
   re-measuring the sizing basis first.
7. Removal is bounded by a finite wall deadline only, never by an entry
   count. An entry cap is the correct bound for a measurement scan, which
   makes no progress when repeated, but applying it to destructive cleanup
   would let a crash-left namespace larger than the cap fail every attempt
   identically and stay addressable forever, consuming the budget and
   blocking all later admissions. Every unlink is progress, so a deadline
   stop leaves a strictly smaller quarantined tree that the next run's
   quarantine recovery resumes.
8. A blocked admission never authorizes implicit pruning. Never clean the
   shared root and never use an unguarded recursive delete.

The evidence root has a separate finite owner-only flock held from
reservation through final result publication (and, for the release wrapper,
through provenance fsync). Compliant direct and release gates reserve 258 MiB
per attempt, enforce an 8 GiB / 10,000-entry aggregate plus a 1 GiB free
floor, and remeasure before PASS publication. That postflight explicitly
reserves another 2 MiB and two directory entries for bounded result
publication. Release evidence uses two deterministic slots per canonical
repo/output scope: the current canonical provenance slot is retained and only
the inactive slot is reclaimed under the lock. Corrupt prior provenance
blocks without deleting either slot.

Those two slots support the current canonical local capability, not archival
history. A copied review provenance may expire after later canonical builds
rotate its external result/log slot, and a review copy outside its bound
canonical provenance path is rejected. There is currently no supported
durable archive/move operation because the result, log, paths, slot, and
digests are mutually bound. An authenticated multi-consumer pin registry
remains follow-up work. Legacy nonce-named evidence is never guessed or
automatically deleted.

Keep persistent namespaces only for active candidates, accepted composition
heads, or referenced local acceptance evidence. Record source key, artifact
key, marker digest, and external evidence paths. A separate age-aware,
reference-aware, lock-compatible dry-run prune command remains the next
control; until it exists, budget/floor pressure blocks admission.

## Explicit non-hermetic and resource boundary

This gate prevents stale cross-worktree reuse and binds local acceptance
evidence. It is not a hermetic builder, reproducible-build attestation, or
dependency supply-chain attestation.

- The mount namespace is not a private-root/pivot-root filesystem sandbox.
  A trusted project `build.rs` can still read or write host paths outside the
  masked source, snapshot, evidence, and target roots when ordinary Unix
  permissions permit. Target-local `HOME`/`TMPDIR` narrows common writes but
  is not an arbitrary-path write sandbox.
- **The capability drop is not a durable capability boundary.** Step 9
  clears the effective, permitted, inheritable, ambient, and bounding sets
  and sets `no_new_privs`, and both are proved. It does not follow that a
  build script cannot hold `CAP_SYS_ADMIN` again: an unprivileged
  `unshare(CLONE_NEWUSER|CLONE_NEWNS)` from inside the gate succeeds and
  restores a full capability set *within that nested namespace*, with
  `no_new_privs` still set. Measured:

  ```text
  after setpriv:                 CapEff 0000000000000000  CapBnd 0000000000000000  NoNewPrivs 1
  after unshare(NEWUSER|NEWNS):  CapEff 000001ffffffffff  CapBnd 000001ffffffffff  NoNewPrivs 1
  umount mask / MNT_DETACH:      EINVAL
  remount read-only bind rw:     EPERM, subsequent write EROFS
  ```

  What holds the masks and the read-only source is the kernel's locking of
  mounts copied into a child user namespace, not the capability drop. The
  drop remains worth doing — it removes capabilities over the *host*
  namespace — but any future claim that rests on it alone would be unsound.
- Paths and allowlisted environment strings are bound, but arbitrary files
  they reference, the complete toolchain/native linker tree, Cargo registry
  contents, system libraries, and network dependency bytes are not all
  content-bound.
- The live supervisor is a best-effort reaction boundary. It is not a
  filesystem project quota, cgroup `memory.max`, or cgroup `pids.max`; a
  sufficiently fast write/allocation burst can occur between samples, and
  source tmpfs/compiler memory can still pressure the host.
- The current global Cargo lock plus jobs/test-thread caps remains the primary
  OOM control. Do not claim that this gate alone prevents host ENOSPC, OOM, or
  PID exhaustion.
- The manager, evidence root, registry, and cleanup paths assume no hostile
  process with the same host UID. A same-UID actor can race directory-entry
  cleanup or replace a complete unsigned result/log/provenance chain.
  No-follow/inode checks turn ordinary drift into `BLOCK`; they are not
  authentication against that actor. A stronger lane needs a separate
  service UID plus append-only/signed evidence.

Use the result only as local trusted-source validation. A future hermetic
release lane must use a private root with an explicit filesystem allowlist,
content-bind every admitted dependency/tool/configuration input (or bind an
immutable image digest), and enforce cgroup memory/PID plus filesystem quota
limits before it can emit reproducible release provenance.
