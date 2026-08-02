# AIA-048 Source-keyed Live-Sampler Churn Repair Result

Date: 2026-08-02  
Status: implementation candidate frozen; manager Cargo reproduction and fresh
exact review still required  
Packet: `docs/development/ai/task-packets/aia048-source-keyed-live-sampler-churn-repair-20260802.md`

This report does **not** accept AIA-048, AIA-046, File Ancestry, or CFR-168.
It records the exact implementation subject and the non-Cargo evidence that a
manager must combine with the required live source-keyed Cargo rerun before a
fresh reviewer may decide acceptance.

## 1. Exact implementation subject

| Field | Value |
|---|---|
| implementation commit | `98809413cef7df25d56e985944c3701f48635fb2` |
| tree | `cd0014c85625404767554512de0af5bea8851ea0` |
| parent / packet commit | `5b283ccbc8d0bd7dcf166939a8d5f24dcf997d29` |
| parent tree | `a9f554460361929991573c13fcf2077226198fcc` |
| subject | `fix(aia): tolerate gate-owned live sampler churn` |
| production SHA-256 | `821a90604521904e053e406dc59c6155f362711ff755f316192407c889314dcc` |
| test SHA-256 | `f588b50fd81f62c703c4115fdb2f974bb15d87e76b5715c9b249868012172bdd` |
| contract SHA-256 | `7f2be7289772a61f0da7524eecbbe44abe2eda6782bdf78b905f7e73173dac8e` |

The implementation commit changes exactly the packet's three authorized
paths:

- `docs/development/tools/run_source_keyed_cargo_gate.py`
- `docs/development/tools/test_ai_tools.py`
- `docs/development/ai/source-keyed-cargo-gates.md`

This result is a separate report-only commit adding only this file.

## 2. Mechanism

The repair separates two named measurement policies:

- `STRICT_NAMESPACE_SCAN` remains the admission, registry, recovery,
  post-gate, and retention behavior. Missing entries and identity changes
  remain fail-closed.
- `LIVE_BEST_EFFORT_NAMESPACE_SCAN` is used only by the one-second runtime
  supervisor. It tolerates only descriptor-relative `ENOENT` after a name was
  enumerated or a bound directory disappeared before reopen. The enumerated
  name still consumes the entry budget.

The implementation does not treat replacement as disappearance. It binds and
revalidates:

1. the namespace's absolute component chain from `/`, using no-follow
   directory opens plus `(st_dev, st_ino)` identity;
2. every queued in-namespace directory component;
3. the current directory pathname after each scan, including an empty scan;
4. the complete namespace root chain again after all pending children drain.

Consequently direct root replacement, registry/ancestor replacement,
intermediate ancestor symlinks, nested parent replacement, replacement while
a cached file still stats successfully, and replacement during a late child
scan remain fail-closed. The initial open/stat race preserves the predecessor
`directory-identity-mismatch` code; traversal-time replacement uses
`namespace-entry-identity-mismatch`.

Deadline checks now occur before directory work, for each enumerated entry,
after every directory scan, after tolerated directory disappearance, and
after final root revalidation. There is no sleep-until-quiet or unbounded
retry. The existing two attempts remain only for an unavailable namespace
root.

The runtime guard still evaluates, in order, the per-namespace growth bound,
the 32-GiB persistent aggregate budget, and an independent filesystem free
sample against the 100-GiB floor. Ordinary observed churn does not skip the
free-space check. Strict post-gate measurement still decides retention.

## 3. Strict/live caller matrix

| Caller | Policy | Churn tolerance | Identity/deadline behavior |
|---|---|---|---|
| `namespace_usage(path)` | strict | none | absolute root chain plus every internal component; fail-closed |
| `inspect_registry(...)` | strict | none | same bound root supplied to the shared bounded traversal |
| cleanup/recovery descriptor scans | strict default | none | existing caller-owned root validation remains in force |
| final/post-gate retention measurements | strict | none | missing/replaced paths block retention |
| `live_namespace_sample(path, ...)` | live best effort | proven descriptor-relative disappearance only | replacement, symlink, identity, entry, and deadline violations block |
| `runtime_resource_guard()` | live best effort | consumes the live sample | allocation, aggregate, and free-floor checks unchanged |

A stable symlink entry is measured without being followed. A symlink replacing
a bound directory or any symlink in the bound namespace ancestry is rejected.

## 4. Deterministic regression evidence

The added controls cover all packet rows and the gaps found during independent
pre-freeze review:

- file disappearance after enumeration;
- directory disappearance before reopen;
- direct, nested-ancestor, current-directory, namespace-root, registry-root,
  intermediate-symlink, and late-child replacement;
- strict controls for the corresponding disappearance/replacement windows;
- stable allocation, aggregate-budget, and independent free-floor blockers;
- entry-count and empty-directory deadline exhaustion;
- a behavioral predecessor simulation in which two strict attempts each lose
  a different enumerated file and reach `live-target-usage-unavailable`;
- repeated repaired sampling under deterministic churn;
- candidate/log/snapshot/quarantine/runtime-scratch invariants on both PASS
  and BLOCK;
- canonical durable result publication, mode, link count, stdout equality, and
  publication residue on both terminal decisions.

The review loop found and closed real pre-commit gaps rather than converting
them into later successors: the first draft checked deadlines only inside the
entry loop, stored only leaf identities, lacked a behavioral predecessor
reversal, and did not publish a result in the PASS/BLOCK topology control.
Further deterministic probes exposed file-parent, post-scan, root, ancestor,
symlink, and late-child replacement windows. All were repaired before the
implementation commit was created.

## 5. Non-Cargo validation

Manager validation against the exact frozen bytes:

```text
PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/test_ai_tools.py
Ran 266 tests in 73.923s
OK

PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile \
  docs/development/tools/run_source_keyed_cargo_gate.py \
  docs/development/tools/test_ai_tools.py
PASS

PYTHONDONTWRITEBYTECODE=1 python3 \
  docs/development/tools/check_ai_acceleration_docs.py
AI acceleration docs check passed.

PYTHONDONTWRITEBYTECODE=1 python3 \
  docs/development/tools/check_project_docs.py
Project docs check passed; 689 Markdown files inventoried.

git diff --check
PASS
```

Two independent read-only pre-freeze audits recomputed the same three file
hashes and returned `ACCEPT` with no required finding. Their independent full
runs reported `266/266 OK` in 75.476s and 75.121s; focused AIA-048 runs reported
`12/12 OK` in 4.799s and 4.822s. These audits are supporting evidence, not the
packet's final reviewer commit, because the manager Cargo rerun was not yet
available.

## 6. Manager-only validation still unavailable here

The implementation lane ran no Cargo, Rust, install, Docker, network, or
dogfood command. The exact failed behavior must now be retried through this
candidate wrapper against the clean File Ancestry R1f candidate:

```bash
RUSTUP_HOME=/home/devuser/.cache/codefire-rust-1.97/rustup \
CARGO_HOME=/home/devuser/.cache/codefire-rust-1.97/cargo \
RUSTUP_TOOLCHAIN=1.97.0-x86_64-unknown-linux-gnu \
CARGO_TARGET_DIR=/workspace/.cache/cargo-target-shared \
CARGO_BUILD_JOBS=2 \
RUST_TEST_THREADS=2 \
CARGO_INCREMENTAL=0 \
/usr/bin/python3.10 -I -S -B -X pycache_prefix=/dev/null \
  /workspace/worktrees/cfr168-aia048-live-sampler-freeze-20260802/.claude-second-agent/04a312079c5faeb8375be40fd4c1003b71a34fc0b9ddf35dc60e23668a81a62d/worktrees/aia048_live_sampler_churn_repair_20260802/docs/development/tools/run_source_keyed_cargo_gate.py \
  --repo /workspace/worktrees/cfr168-file-ancestry-repair-20260802/.claude-second-agent/4564e1cd1a5d6b52f206a3d40209373bda5331cd8dd210ec3a87fbdb2715b431/worktrees/cfr168_file_ancestry_r1c_forced_cleanup_repair_20260802/.claude-second-agent/5536e3efc7b8ff8e29764cabc2000374356645fddf94f49dfe98f151d219d88e/worktrees/cfr168_file_ancestry_r1f_lint_repair_20260802 \
  --timeout 3600 \
  --lock-timeout 21600 \
  --log /workspace/.cache/codefire-gate-evidence/cfr168-file-ancestry-r1f-focused-58bb5a2-aia048-9880941-20260802.log \
  --result /workspace/.cache/codefire-gate-evidence/cfr168-file-ancestry-r1f-focused-58bb5a2-aia048-9880941-20260802.json \
  -- cargo test -p codefire-cli --bin codefire-rs \
  remote::file_remote_ancestry_tests --locked
```

Acceptance requires `PASS`, one `codefire-rs` harness with at least 40 selected
tests and zero failures, runtime guard `PASS`, committed target run state,
unchanged clean candidate HEAD/tree, complete result/log digests, and no
residual snapshot or namespace quarantine. Only after that evidence exists may
a fresh independent report-only reviewer decide the exact implementation and
result subject.

## 7. Residual boundary

The live sample remains a non-atomic best-effort reaction boundary. A write or
allocation burst can occur after final sample revalidation or between the
one-second samples. This repair is not a filesystem quota, cgroup memory/PID
limit, atomic snapshot, or proof against a hostile same-UID actor. The strict
post-gate scan remains authoritative for retention, and the global Cargo lock,
jobs/thread caps, aggregate budget, and free-space floor remain the primary
resource controls.
