# AI Task Packet

Status: blocked

Packet ID: MS-local-sealed-loop-runner

Schema version: human-v1

Base revision: the commit that closed `MS-2026-08-control-restore` on 2026-08-21

Risk score: 3

Reasoning tier: high

Parent packet: `milestone-local-sealed-loop.md`. This is node 1 of that graph — one of two parallel ready slices.

Node 1 has now run. Everything from `## Objective` through `## Handoff` is the original **plan** and carries no evidence — do not read those sections as observations. The only observed content in this file is `## Result`.

## Objective

Decide which `codefire` executable runs the M1 local sealed loop, and record it in this file: path, provenance, and capability outcome. If no such executable can be obtained inside M1 scope, record an exact blocker instead.

This node answers *what runs the loop*. It does not answer what the loop carries — that is node 2, running concurrently.

## Context

- Required context pack(s): pack-0-universal-orientation
- Additional files: `docs/development/milestones.md`, `docs/development/known-limitations.md`, `docs/development/README.md`
- Known constraints to check, not to assume: the base environment has been recorded as exposing no `cargo` on `PATH`, and the only recorded source-matching binary is the July artifact `/tmp/codefire-cfr167-final.KWa8TC/bin/codefire` with SHA-256 `1483874b4c1de4a78478595d5d158cf34a12e5ec3b05255dfc8819f511f00272`. A `/tmp` path is not durable across sessions. Verify current reality; do not restate these as findings.

## Plan

1. Determine which `codefire` executables are actually reachable now.
2. For each candidate, record path and provenance — how it was produced and whether its identity is known.
3. Record the capability outcome: can it run `status` / `scan` / `extinguish` / `verify` / `commit` against the dogfood project at all, on a read-only basis where possible.
4. Name the one runner node 3 should use, or record an exact blocker.

A recorded blocker is a legitimate and complete result for this node. Report it as `block-m1` and stop.

## Scope

Allowed files:

- `docs/development/ai/task-packets/milestone-local-sealed-loop-runner.md` — this file, and nothing else

Disallowed files:

- `crates/**`, `codefire`, `tests/**`, `install.sh`, `pyproject.toml`, `setup.py`, `adr/**`
- `subprojects/**` — this node makes no product or dogfood edit
- CodeFire project state; this node must not mutate any repository CodeFire manages
- `docs/development/ai/task-packets/milestone-local-sealed-loop-increment.md` — node 2 owns it
- `docs/development/milestones.md` — the orchestrator owns the board
- the classified dirty tree in `/workspace/project`

## Non-Goals

- Building a release pipeline, pinning a toolchain, or establishing CI identity; those are M5 release-slot concerns.
- Fixing `install.sh` or packaging.
- Compiling the workspace to manufacture a binary if `cargo` is genuinely unavailable — record the blocker instead.
- Selecting the increment, editing product code, or running the loop.
- Starting any parked root.

## Invariants

- no product or dogfood file is modified by this node;
- no CodeFire-managed repository is mutated — probing is read-only, and the separately configured legacy repository is not touched at all;
- path/workspace safety: work stays inside the configured workspace;
- an executable whose provenance cannot be established is recorded as unknown provenance, not silently accepted as the runner.

## Validation

This node changes one planning file, so it runs only:

```bash
python3 docs/development/tools/check_project_docs.py
git diff --check
```

Any `codefire` invocation here is a read-only capability probe, not milestone validation. The M1 exit-gate list stays on the M1 card and is node 3's to run. Do not widen.

## Handoff

- runner path, or the exact blocker;
- provenance: how the binary was produced, and its identity if known;
- capability outcome per loop command probed, read-only;
- whether the runner is durable across sessions;
- risks node 3 inherits;
- next allowed action.

## Result

Outcome: **`block-m1`**. No runner exists inside M1 scope that can execute the M1 exit-gate loop commands. Per the Plan, a recorded blocker is a complete node 1 result; node 1 stops here.

Run by: Claude Opus 5 individual agent, 2026-08-21, on a clean worktree at integration head `b996562`.

### The blocker, exactly

The M1 card and the parent packet gate the loop on this command surface:

```bash
codefire status --path subprojects/algorithm-evolution-agent-lab --json
codefire scan --path subprojects/algorithm-evolution-agent-lab
codefire extinguish --path subprojects/algorithm-evolution-agent-lab
codefire verify --path subprojects/algorithm-evolution-agent-lab
codefire commit --path subprojects/algorithm-evolution-agent-lab
```

That `--path` / `--json` surface belongs to the **Rust** CLI (`crates/codefire-cli`, bin `codefire-rs`), which cannot be built here. The **Python** reference implementation is reachable and runnable but does not implement `--path` or `--json` at all. So the two facts combine into one blocker: the only runnable executable cannot express the gate, and the executable that can express the gate cannot be produced without leaving M1 scope.

### Probe 1 — is any `codefire` on `PATH`?

```bash
command -v codefire   # exit 1, no output
which -a codefire     # exit 1, no output
command -v cargo      # exit 1, no output
command -v rustc      # exit 1, no output
```

No `codefire`, no `cargo`, no `rustc` on `PATH`. The previously recorded absence of `cargo` is re-confirmed as current, not restated from the prior record.

### Probe 2 — the recorded July artifact

```bash
ls -la /tmp/codefire-cfr167-final.KWa8TC/bin/codefire
# ls: cannot access '/tmp/codefire-cfr167-final.KWa8TC/bin/codefire': No such file or directory
ls -d /tmp/*codefire*
# /tmp/codefire-milestone-audit-brief.md   (a Markdown brief, not an executable)
```

The July artifact with SHA-256 `1483874b4c1de4a78478595d5d158cf34a12e5ec3b05255dfc8819f511f00272` **no longer exists**. `/tmp` did not survive, as the packet warned it might not. It is unavailable as a runner and its identity cannot be re-verified here.

### Probe 3 — any compiled binary or toolchain anywhere reachable

```bash
find /workspace -type f -path '*target*' -name 'codefire'   # no results
ls -la /home/devuser/.cargo /home/devuser/.rustup           # both: No such file or directory
find /home /usr/local /opt /root -name 'codefire*' -maxdepth 6   # no results
python3 -m pip show codefire   # WARNING: Package(s) not found: codefire
ls -la /tmp                    # no executable codefire artifact
```

No build output, no Rust toolchain, no installed package, no vendored prebuilt binary. There is no compiled `codefire` on this machine.

### Probe 4 — the Python reference implementation (the only runnable candidate)

Path: `./codefire` at the repository root (also present identically at `/workspace/project/codefire`).

Provenance and identity — **known**:

```bash
git ls-files codefire                 # codefire   (tracked)
sha256sum codefire
# efd124fd442addf1c7f385a73d0fbbe959e3455d609ab3d5eb09dc82fb54fa10
git show HEAD:codefire | sha256sum
# efd124fd442addf1c7f385a73d0fbbe959e3455d609ab3d5eb09dc82fb54fa10
git log --oneline -3 -- codefire
# eb4fd0c snapshot: preserve classified pre-M1 worktree
# 4ad5cc6 Add detailed verify diagnostics
# d25fc97 Fix dogfooding issues in fire recovery and Python atoms
```

Identity is fully knowable: the working file, `/workspace/project/codefire`, and `HEAD:codefire` are byte-identical at SHA-256 `efd124fd442addf1c7f385a73d0fbbe959e3455d609ab3d5eb09dc82fb54fa10`. Provenance is a stdlib-only Python script whose current content was committed by `eb4fd0c` ("snapshot: preserve classified pre-M1 worktree", 29537 insertions / 1639 deletions to this file) — i.e. it is the preserved pre-M1 working state carried into this milestone, **not** an independently released or reviewed artifact. `python3` is `Python 3.10.12` and the script declares stdlib-only dependencies, so it needs no install step.

Durability: **durable**. It is version-controlled at the integration head, so it survives sessions — unlike the vanished `/tmp` artifact.

Read-only capability outcome, help surface only:

```bash
python3 codefire --help      # exit 0
python3 codefire --version   # exit 2 — no --version flag exists
```

`--help` lists the subcommands: `init, branch, open, close, discard, clone, upload, list, status, scan, fire, extinguish, verify, commit, merge, request-merge, request-list, request-review, request-apply, gc, show, diff, doctor, serve, completion, token-hash`. There is no global `--path` option before the subcommand. Per-subcommand help (`exit 0` for each):

| Gate command | Python subcommand help | Can it express the gate command? |
|---|---|---|
| `status --path <dir> --json` | `usage: codefire status [-h]` | **No** — accepts neither `--path` nor `--json` |
| `scan --path <dir>` | `usage: codefire scan [-h]` | **No** — no `--path` |
| `extinguish --path <dir>` | `usage: codefire extinguish [-h] --resolution {changed,verified,no-change-required,invalid-fire} [--rationale R] [--evidence E] [--refresh] fire_id` | **No** — no `--path`; also requires a positional `fire_id` and `--resolution` |
| `verify --path <dir>` | `usage: codefire verify [-h] [--details]` | **No** — no `--path` |
| `commit --path <dir>` | `usage: codefire commit [-h] -m MESSAGE [--signer S] [--key-id K]` | **No** — no `--path`; requires `-m` |
| `open` | `usage: codefire open [-h] branch path` | Takes a positional `path`, so `open` alone is expressible |

Confirmed against the source, not inferred from help alone:

```bash
grep -n -- '"--json"\|--json' codefire   # no matches anywhere in the file
grep -n -- '"--path"' codefire           # no matches anywhere in the file
```

The string `--json` does not occur anywhere in the Python implementation. This is decisive, not cosmetic: the M1 exit requires that "the post-loop state is reported by CodeFire itself, not asserted by an agent", and node 3's mandated first step is a verbatim pre-loop `status --json` baseline. The Python runner cannot emit a JSON envelope at all, so it cannot produce the evidence M1 exits on.

### Probe 5 — the gate-matching implementation is the Rust CLI, and it is unbuildable in scope

```bash
grep -n 'name\|\[\[bin\]\]\|\[package\]' crates/codefire-cli/Cargo.toml
# [package] / name = "codefire-cli" / [[bin]] / name = "codefire-rs"
grep -rn -- '"--path"\|--json' crates/codefire-cli/src | head
# crates/codefire-cli/src/verification.rs:41:  "--json" => json_output = true,
# crates/codefire-cli/src/verification.rs:43:  "--path" => {
# crates/codefire-cli/src/fire.rs:86:          "--path" => {
# crates/codefire-cli/src/fire.rs:104:         "--json" => json_output = true,
# crates/codefire-cli/src/tests/doctor_capability.rs:75:
#     format!("codefire scan --path {quoted_open_path} --json")
```

The Rust CLI is the implementation that parses `--path` and `--json`, matching the gate. It also carries an explicit `legacy_python_status_context` compatibility path (`crates/codefire-cli/src/main.rs:10949`), which is further evidence that the Rust binary — not the Python script — is the intended loop runner and that Python state is treated as legacy input to it. Producing that binary requires `cargo`, which is absent, and the packet's Non-Goals forbid compiling the workspace to manufacture a binary. So this candidate is out of scope by construction, not merely inconvenient.

### What was deliberately not probed

No CodeFire command was run against `subprojects/algorithm-evolution-agent-lab` or any other CodeFire-managed repository. Every probe above is `--help`, `ls`, `find`, `grep`, `git`, `sha256sum`, or `pip show`. In particular `status`, `scan`, `verify`, `extinguish`, and `commit` were **not** executed against the dogfood project, because the parent packet reserves the `status --json` baseline for node 3 and forbids nodes 1 and 2 from running `status` to check board state. No `extinguish` and no `commit` was run anywhere. The separately configured legacy repository was not touched. Nothing outside this packet file was written.

Consequently this node reports **no** execution-level capability outcome against the project — only the command-surface outcome above. Claiming otherwise would be unearned evidence.

### Handoff

- **Runner path:** none. There is no usable runner. The blocker is recorded instead.
- **Closest candidate:** `./codefire` (Python reference implementation), SHA-256 `efd124fd442addf1c7f385a73d0fbbe959e3455d609ab3d5eb09dc82fb54fa10`, tracked at `b996562`, durable, runnable under `python3` 3.10.12 with no install — but it implements neither `--path` nor `--json`, so it cannot execute the M1 exit-gate commands or produce the required JSON state evidence.
- **Gate-matching candidate:** `crates/codefire-cli` (bin `codefire-rs`). Not obtainable: no `cargo`, no `rustc`, no toolchain, no prior build output, and compiling is an explicit Non-Goal.
- **Provenance summary:** the one durable candidate has fully known identity and provenance; the previously recorded July `/tmp` binary is gone and its identity is unverifiable here. No executable was accepted with unknown provenance.
- **Durability:** the Python candidate is durable across sessions; the July artifact was not, and is now confirmed lost.
- **Risks node 3 inherits:** node 3 cannot start. Its first mandated step — the verbatim pre-loop `status --json` baseline — has no executable that can produce it. Nodes 3, 4, 5, and 6 stay blocked, so M1 cannot exit on its stated terms with the current environment.
- **Second risk if a runner is later supplied:** even with a Rust binary, the dogfood project's on-disk state was not produced or inspected by this node. The Rust CLI's `legacy_python_status_context` path implies Python-era and Rust-era state are not the same shape, so state-format compatibility is an open question node 3 must confirm from its own baseline output rather than assume.
- **Parked instead of done:** the whole of node 1's "select a runner" outcome. No parked review root was started, and none is admitted — the Non-Goals bar building a pipeline, pinning a toolchain, and fixing the installer, and this blocker is not evidence for admitting CFB-133 or any other parked root.
- **Next allowed action:** orchestrator decision, not agent work. This is a `block-m1` report. Resolving it means either provisioning a Rust toolchain / a durable identified binary, or amending the M1 exit gate so the loop's evidence does not require `--json` — both of which change the milestone's scope and belong to the orchestrator. Adding `--path` / `--json` to the Python implementation would be a product-code change, outside node 1's write set and outside M1 (that surface is the M2 `MS-agent-local-contract` slot). Node 2 is unaffected: it selects by reading only and can still run and complete in parallel.

### Validation actually run

```bash
python3 docs/development/tools/check_project_docs.py   # exit 0, "OK: ..." (see commit report)
git diff --check                                       # exit 0, no output
```

Not widened. No CodeFire command was used as validation; every `codefire` invocation in this node was a read-only help probe.
