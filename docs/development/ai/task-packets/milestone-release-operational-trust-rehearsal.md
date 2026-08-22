# AI Task Packet

Status: completed

Packet ID: MS-release-operational-trust-rehearsal

Schema version: human-v1

Risk score: 7

Reasoning tier: high

## Objective

Derive the smallest isolated, locally published release-candidate topology that a fresh operator can
verify, install into an isolated prefix, smoke, and roll back using recorded evidence only. Classify
the current build/provenance/install surfaces and stop at the first missing contract. Do not build,
install, publish, or mutate during this packet.

## Scope

- read-only classification of build, provenance, packaging, install, smoke, and rollback surfaces;
- one proposed local-only release root, isolated prefix, evidence boundary, and operator handoff;
- classification and handoff in this packet only.

## Authoritative Inputs

- `AGENTS.md` and `docs/development/milestones.md`;
- `install.sh`, package manifests, source-bound Rust build/provenance tooling, checksum/conformance
  schemas, and their focused tests;
- public version/help/capabilities surfaces and release/format strategy documents;
- retained B1 runner and provenance as historical evidence, not automatically the release candidate;
- the current intentional dirty main worktree and the integration branch, both read-only.

## Required Classification

1. Select the exact source revision and clean-worktree rule for a rehearsal candidate. State whether
   the existing source-bound runner can be reused or a fresh accepted-source build is required.
2. Define one `$RELEASE_ROOT`, immutable artifact staging area, manifest/checksum/provenance record,
   local publication pointer, isolated install prefix, prior-state/rollback target, and evidence root.
3. Name the exact operator commands for identity verification, checksum validation, source binding,
   install, `--version`/`--help`/`capabilities --json` smoke, and rollback. No command may touch the
   user's current install or canonical Algorithm repository.
4. Define before/after hashes for integration/main, canonical CodeFire store, current user install,
   artifact staging, install prefix, and rollback target.
5. Define crash/partial-publication and wrong-artifact falsifiers. Publication must be local and
   atomic or fail without presenting a mixed candidate.
6. Separate required M1 evidence from parked Stage 4 signing, SBOM, external registry, credential,
   support/security policy, backup, and two-person response work.
7. Classify the first missing contract as `block-m1`, `park`, or `drop`, and propose the smallest
   exact write set only if a reproduced blocker exists.

## Exact Write Set

- `docs/development/ai/task-packets/milestone-release-operational-trust-rehearsal.md` only.

## Validation

```bash
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_ai_acceleration_docs.py
git diff --check
```

## Non-Goals

- running a build or installer, changing the user's PATH/prefix, or invoking CodeFire against canonical state;
- public GitHub/PyPI/crates.io upload, network publication, credentials, signing, or release naming;
- editing source, tests, product/public docs, package metadata, license files, or another task packet;
- treating a retained binary as source-bound without revalidating its recorded revision and manifest;
- broad Stage 4 completion, M2 product research, or M3–M5 ecosystem work.

## Handoff

Return the exact local topology, source/reuse decision, public commands and cwd, preservation manifest,
operator handoff, rollback/falsifier matrix, first missing contract, `block-m1` / `park` / `drop`, any
smallest proposed fix write set, validation results, and commit SHA. Commit only this packet.

## Result

Status **completed**. Classification at exact inspected HEAD
`d803527599808cb2ac31acf74e3d0b988208f2fc` (`docs: close adoption M1 and shift to release trust`),
worktree `/workspace/worktrees/m1-release-rehearsal-20260822` on
`grok-agent/m1-release-rehearsal-20260822`, which is the same commit as
`refs/heads/integration/ms-2026-08-control-restore`. This worktree was clean
(`git status --porcelain=v1` empty) before this packet was written. No build,
install, publication, CodeFire invocation, license edit, or extra-file mutation
ran. Write set: this packet only.

**No product `block-m1` was reproduced.** The isolated release/verify/rollback
path can be admitted as operator procedure. There is no smallest product/source/test
fix write set. Node 3 must not start node 5 until node 1 rights fan-in also
admits an internal-only rehearsal of the same artifact set, and until node 5a
has `cargo`+`rustc`.

Run by: Grok 4.6 node 2, 2026-08-22.

### 1. Source revision, clean-worktree rule, reuse decision

| Item | Decision |
|---|---|
| Exact source revision | `d803527599808cb2ac31acf74e3d0b988208f2fc` |
| Integration ref | `integration/ms-2026-08-control-restore` @ that SHA |
| Classifier worktree | `/workspace/worktrees/m1-release-rehearsal-20260822`, porcelain empty |
| Dirty main (not a candidate) | `/workspace/project` @ `f1644122134d479333cf422fdb1d48b8fa92dd8e`, 248 porcelain (116 `M` + 132 `??`), porcelain SHA-256 `1385b6249f6455510bc449aed054d5ce425494a29edb9a4c21d65165cb6b1bcf`. Uncommitted rust vs that HEAD is dirty (35 cargo/crate files). Integration is 60 commits ahead of main and 0 behind. |
| Clean-worktree rule | Admitted checkout SHA equals the recorded source revision; `git status --porcelain=v1` empty; `validate_source_inputs_against_head(root, HEAD)` returns `[]` for `Cargo.toml`, `Cargo.lock`, `.cargo`, `rust-toolchain`, `rust-toolchain.toml`, `crates`. Symlinks in that set fail closed. |
| Observed rust inputs at d803527 | `codefire.rust_build_inputs.v1`, 101 files, `source_manifest_sha256` `3925fc035690296fa5e132b630dab65a7fbd5ddb1a55f0fb1226e6efd295d97a`. Matches B1. `source_vs_head_errors=[]`. This worktree has no `target/release/codefire-rs` and no provenance file. |
| Reuse of retained B1 runner | **Forbidden as the published candidate.** Historical only. |
| Fresh accepted-source build | **Required.** |

Retained B1 runner (historical, not the release candidate):

```text
path         /workspace/worktrees/ms-adoption-experiment-20260822/target/release/codefire-rs
             hardlink also at .../target/release/codefire
sha256       90cb2895aadc45b559a96207cdd5ae3b08485f9a7e5a07014db8cca8b5b90ce9
bytes        14590776
provenance   .../target/codefire-ai/rust-build-provenance.json
prov sha256  1731b8636330841f20dcad9265710eacc3830767e8c693b34ad75b5907688dea
schema / ok  codefire.rust_build_provenance.v1 / true
base_rev     c78d5e0e5e151a1489744c21340716a439597e71
manifest     3925fc035690296fa5e132b630dab65a7fbd5ddb1a55f0fb1226e6efd295d97a (101 files)
command      cargo build --release --locked -p codefire-cli --bin codefire-rs
toolchain    cargo 1.97.0 (c980f4866 2026-06-30) / rustc 1.97.0 (2d8144b78 2026-07-07)
B1 tree HEAD 93cd289f7a5ed452fbbb958b405a914e91bb6f8a (clean; not d803527)
```

`git diff --quiet c78d5e0e d803527 -- Cargo.toml Cargo.lock crates .cargo rust-toolchain rust-toolchain.toml` is 0, and the same for `codefire` / `install.sh`. Byte-identity of rust inputs does **not** rebind provenance: `validate_provenance_payload` requires `base_revision == expected_revision == current HEAD`. Against d803527 the B1 document fails that check. Packet non-goal forbids treating the retained binary as source-bound without revalidating its **recorded** revision. Do not invent a rebind-without-rebuild tool.

Classifier environment (`probe_validation_capabilities.py --json`, no product execution): `cargo=false`, `rustc` absent, `codefire` absent, `rust-binary-present=false`, `rust-source-bound-binary=false`. Node 5a cannot build here. Toolchain acquisition is **park** (Stage 4 CI identity), not a source patch. Node 5a must run where `cargo` and `rustc` exist; the only historically successful source-bound toolchain on this lineage is cargo/rustc 1.97.0.

A later docs-only commit of this packet does not change the 101 rust inputs. Node 5 may use the node-2 commit as the procedure-bearing checkout; provenance `base_revision` will be **that** checkout SHA. Rust input digest must remain `3925fc03…`. Dirty main is **drop** as a candidate.

### 2. Local-only topology

Create one disposable root on a single filesystem (required for atomic `mv`). Do not use `/workspace/project`, `/usr/local`, `$HOME/.local`, `/workspace/algorithm-evolution-agent-lab`, or `$SRC/target` as the published candidate.

```bash
RELEASE_ROOT="$(mktemp -d /tmp/codefire-m1-release.XXXXXX)"
chmod 700 "$RELEASE_ROOT"
```

| Variable | Path | Role |
|---|---|---|
| `$RELEASE_ROOT` | `/tmp/codefire-m1-release.<nonce>` | Exclusive local rehearsal root |
| `$SRC` | clean git worktree of the admitted SHA | Build inputs only; not the publication pointer |
| `$STAGING` | `$RELEASE_ROOT/staging.work` | Mutable assembly; never presented as current |
| `$PUB` | `$RELEASE_ROOT/published/<artifact_id>` | Immutable complete candidate after rename |
| `$CURRENT` | `$RELEASE_ROOT/current` | Atomic publication pointer (symlink) |
| `$PREFIX` | `$RELEASE_ROOT/prefix` | Isolated install prefix (`$PREFIX/bin/codefire` and `codefire-py`) |
| `$PRIOR` | `$RELEASE_ROOT/prior` | Rollback snapshot of `$PREFIX` taken before install |
| `$EVIDENCE` | `$RELEASE_ROOT/evidence` | Operator logs, T0/T1 hashes, smoke cwd |
| `$SMOKE_CWD` | `$EVIDENCE/smoke` | Empty directory; not Algorithm, not dirty main, not `$SRC` |

`<artifact_id>` = `d80352759980-<binary_sha256_12>` after the fresh build (use the admitted checkout's 12-hex if node 5 builds at the packet commit).

Published tree (smallest closed set):

```text
$PUB/
  RELEASE_MANIFEST.json          local-only identity; not a product schema
  SHA256SUMS                     sha256 of every regular file below
  SOURCE_REVISION                full admitted git SHA, newline-terminated
  rust-build-provenance.json     codefire.rust_build_provenance.v1 copied from the build
  rust-build-inputs.json         codefire.rust_build_inputs.v1 recomputed at admitted HEAD
  artifacts/codefire-rs          freshly built Rust default CLI
  artifacts/codefire             Python fallback blob at admitted HEAD
  artifacts/install.sh           installer blob at admitted HEAD
  docs/README.md
  docs/04_cli_spec.md
  docs/release-and-format-roadmap.md
```

HEAD d803527 blob SHA-256 values to copy (re-hash at node 5a checkout):

| Path | SHA-256 | Bytes |
|---|---|---|
| `codefire` (Python fallback) | `efd124fd442addf1c7f385a73d0fbbe959e3455d609ab3d5eb09dc82fb54fa10` | 1341034 |
| `install.sh` | `0bc5a6cb4a89bc003ce667e0acff79c0619a0c60db1c3b1be5ed0138da59752e` | — |
| `README.md` | `422174c3a419d846abce1844461e449c4cbb8178a0983aac2afc3911f355110e` | — |
| `docs/04_cli_spec.md` | `b71428806556534f1d50fd0538e7680eded78a08db673d4b24c876146264b28e` | — |
| `docs/development/strategy/release-and-format-roadmap.md` | `d9bf926babd72d299d24f8a38b06c5db1d19c6ba8f31c29ff7cba1afb4eb50e3` | — |

Do not stage `pyproject.toml` / `setup.py` into `$PREFIX`. Those install Python as `codefire` and would shadow Rust. They are **park** as a distribution path.

`RELEASE_MANIFEST.json` (operator record; create only after staging is complete):

```json
{
  "schema": "codefire.local_release_manifest.v1",
  "channel": "internal-rehearsal-only",
  "public_beta": false,
  "semver_release": false,
  "signed": false,
  "source_revision": "<admitted full SHA>",
  "source_worktree_clean": true,
  "rust_source_manifest_sha256": "3925fc035690296fa5e132b630dab65a7fbd5ddb1a55f0fb1226e6efd295d97a",
  "rust_source_file_count": 101,
  "version_axes": {
    "product_semver": "unassigned",
    "rust_crate": "0.1.0",
    "python_package": "0.2.0",
    "cli_reported": "codefire foundation 1",
    "repository_layout": 1,
    "core_object": 1,
    "atom_hash_schema_write": 4,
    "commit_root_schema_write": 2,
    "automation_envelope": "codefire.command_result.v1",
    "capabilities": "codefire.capabilities.v1",
    "support_profile": "codefire.support.v1"
  },
  "artifacts": [
    {"path": "artifacts/codefire-rs", "role": "rust-default-cli"},
    {"path": "artifacts/codefire", "role": "python-fallback"},
    {"path": "artifacts/install.sh", "role": "installer"},
    {"path": "rust-build-provenance.json", "role": "rust-provenance", "schema": "codefire.rust_build_provenance.v1"},
    {"path": "rust-build-inputs.json", "role": "rust-inputs", "schema": "codefire.rust_build_inputs.v1"}
  ]
}
```

Rights/license fields are owned by node 1. This topology labels the set `internal-rehearsal-only` and `public_beta=false` regardless.

Atomic local publication:

1. Assemble under `$STAGING` (must not exist as `$CURRENT`).
2. Write `SHA256SUMS` from that tree.
3. Write `RELEASE_MANIFEST.json`.
4. Write empty `$STAGING/.complete` last.
5. `chmod -R a-w "$STAGING"` after the marker.
6. `mkdir -p "$RELEASE_ROOT/published"` and `mv "$STAGING" "$PUB"` (destination must not exist).
7. `ln -sfn "published/<artifact_id>" "$RELEASE_ROOT/current.tmp" && mv -Tf "$RELEASE_ROOT/current.tmp" "$RELEASE_ROOT/current"`.

A mixed or incomplete tree must never be named `$CURRENT`. `$SRC/target/` is scratch, not publication.

`install.sh` is **not** the publisher. It mutates `$PREFIX` in place (`install` of rust then python) and its cargo path is `cargo build --release -p codefire-cli --bin codefire-rs` (**no `--locked`**, no provenance). Node 5a must build only via `docs/development/tools/build_source_bound_rust.py`.

### 3. Fresh-operator commands (absolute paths; no PATH mutation; no Algorithm; no dirty main)

All commands below are for nodes 5a/5b. This node did not run them.

**5a — build (cwd `$SRC`, admitted clean SHA):**

```bash
python3 docs/development/tools/build_source_bound_rust.py --output target/codefire-ai/rust-build-provenance.json
```

Copy `target/release/codefire-rs`, that provenance file, recomputed inputs JSON, `codefire`, `install.sh`, and the three docs files into `$STAGING` as laid out above, then publish atomically.

**5b — identity (cwd `$RELEASE_ROOT`):**

```bash
readlink -f current
test -f current/.complete
cat current/SOURCE_REVISION
# must equal admitted SHA
```

**5b — checksums:**

```bash
# cwd $CURRENT
sha256sum -c SHA256SUMS

# cwd $SRC — validate_provenance_payload reads $SRC/target/release/codefire-rs
test "$(git rev-parse HEAD)" = "$(cat "$CURRENT/SOURCE_REVISION")"
cmp -s target/release/codefire-rs "$CURRENT/artifacts/codefire-rs"
cmp -s target/codefire-ai/rust-build-provenance.json "$CURRENT/rust-build-provenance.json"
python3 -c "import json,sys; from pathlib import Path; sys.path.insert(0,'docs/development/tools'); from build_source_bound_rust import validate_provenance_payload, git_head; root=Path('.').resolve(); payload=json.loads((root/'target/codefire-ai/rust-build-provenance.json').read_text()); err=validate_provenance_payload(root, payload, expected_revision=git_head(root)); print('\n'.join(err) if err else 'provenance-ok'); raise SystemExit(1 if err else 0)"
```

Required payload facts: `schema=codefire.rust_build_provenance.v1`, `ok=true`, `base_revision` equals admitted SHA, `source_manifest_sha256` equals `3925fc035690296fa5e132b630dab65a7fbd5ddb1a55f0fb1226e6efd295d97a`, `binary_sha256` equals SHA-256 of `$CURRENT/artifacts/codefire-rs`, `build_command` is exactly `cargo build --release --locked -p codefire-cli --bin codefire-rs`. Do not retarget validation at the B1 path.

**5b — source binding (cwd `$SRC`):**

```bash
git rev-parse HEAD
git status --porcelain=v1
python3 -c "import sys; from pathlib import Path; sys.path.insert(0,'docs/development/tools'); from build_source_bound_rust import validate_source_inputs_against_head, git_head; h=git_head(Path('.').resolve()); err=validate_source_inputs_against_head(Path('.').resolve(), h); raise SystemExit(1 if err else 0)"
```

**5b — install (cwd `$CURRENT/artifacts` so `install.sh` finds `./codefire`):**

```bash
mkdir -p "$PREFIX" "$PRIOR" "$SMOKE_CWD"
# prior snapshot: prefix starts empty
test ! -e "$PREFIX/bin/codefire"
test ! -e "$PREFIX/bin/codefire-py"
./install.sh --prefix "$PREFIX" --binary "$CURRENT/artifacts/codefire-rs"
```

Never omit `--binary`. Never omit `--prefix`. Default prefix `/usr/local` is forbidden. Do not pass `--completion` (not required for M1 smoke). Do not `export PATH`.

**5b — smoke (cwd `$SMOKE_CWD`; invoke by absolute path only):**

```bash
"$PREFIX/bin/codefire" --version
"$PREFIX/bin/codefire" --help
"$PREFIX/bin/codefire" capabilities --json
"$PREFIX/bin/codefire-py" --help
```

Expected from source (not executed here):

| Command | Expected |
|---|---|
| Rust `--version` | exactly `codefire foundation 1` (`codefire_core::VERSION` is `1`; `main.rs` prints `codefire foundation {VERSION}`) |
| Rust `--help` | contains `usage: codefire <command> [args]` (`completion.rs::help_text`) |
| Rust `capabilities --json` | `data.schema == codefire.capabilities.v1`, `data.type == codefire_command_capabilities`, `support.profile == codefire.support.v1`, envelope `codefire.command_result.v1` |
| Python `--help` | tests pin substring `usage: codefire`; Python has **no** `--version` and **no** `capabilities --json` |

Python `VERSION = 1` is the object-model axis, not a CLI `--version`. Do not invoke any command against `/workspace/project/subprojects/algorithm-evolution-agent-lab`, `/workspace/project/subprojects/.codefire`, `/workspace/algorithm-evolution-agent-lab`, or `/workspace/project`.

**5b — rollback (cwd `$RELEASE_ROOT`):**

```bash
rm -rf "$PREFIX"
mkdir -p "$PREFIX"
# restore prior (empty at first install): prefix must lack both binaries
test ! -e "$PREFIX/bin/codefire"
test ! -e "$PREFIX/bin/codefire-py"
```

If node 5b ever installs over a non-empty `$PREFIX`, snapshot first with `cp -a "$PREFIX/." "$PRIOR/"` and restore with `rm -rf "$PREFIX"; mkdir -p "$PREFIX"; cp -a "$PRIOR/." "$PREFIX/"`. M1's first install prior is empty.

### 4. Preservation manifest and before/after hashes

T0 captured 2026-08-22 at inspected HEAD `d803527599808cb2ac31acf74e3d0b988208f2fc`. Node 5 must re-hash immediately before mutating `$RELEASE_ROOT` and immediately after rollback. After this packet commit, integration HEAD will move; rust input digest must stay `3925fc03…`.

| Subject | T0 identity | After-rollback requirement |
|---|---|---|
| Integration / this worktree | HEAD `d803527599808cb2ac31acf74e3d0b988208f2fc`, porcelain empty | Product/source files unchanged except this packet commit; rust input set unchanged |
| Dirty main `/workspace/project` | HEAD `f1644122134d479333cf422fdb1d48b8fa92dd8e`; 248 porcelain; porcelain SHA-256 `1385b6249f6455510bc449aed054d5ce425494a29edb9a4c21d65165cb6b1bcf` | Byte-identical HEAD and porcelain hash; no command may write here |
| Canonical CodeFire store | `/workspace/project/subprojects/.codefire`, `repository_id` `repo_8a9e9ea836af`; `repo.json` SHA-256 `bf6d978b0eece0f185c6995630339326f56513a0c8645af3e7e984ad69a7f054`; `branches/main.json` SHA-256 `bfd52e3525f6a6faf0db4f3d2d1d9d7fa3fa0d400f6561c3c5b8227e79e989f8`; head `CF-COMMIT-2792b54ad730e6d644668c38`, `open-clean`; tree SHA-256 `12d84568ee63fc69b54536346da7a2072290edcc952726330be97ff03fdca4d1` over 2905 files / 114660974 bytes; opened path `/workspace/project/subprojects/algorithm-evolution-agent-lab` | All four hashes and head id unchanged |
| Historical Algorithm copy | `/workspace/algorithm-evolution-agent-lab/.codefire`, `repo_f9f434abf020`, head `CF-COMMIT-5d3f0f33e5ce16ab34d72ac1`; tree SHA-256 `b41e7e884e3d18a72e81d29c2d93088fcb3ffdafc537fd6e6687ec1155187495` (1470 files / 12232765 bytes) | Untouched |
| User install | no `codefire` on PATH; `/home/devuser/.local/bin` absent; `/usr/local/bin` exists with no `codefire` | Still absent; do not install here |
| Artifact staging / `$CURRENT` | absent | After rollback, published tree may remain (immutable); `$CURRENT` may remain; `$PREFIX` restored to prior (empty) |
| Isolated prefix | absent | Empty of `bin/codefire` and `bin/codefire-py` |
| Rollback target `$PRIOR` | absent | Matches T0 prefix (empty) |

### 5. Crash / partial-publication and wrong-artifact falsifiers

| Falsifier | Fail closed when | Mixed candidate rule |
|---|---|---|
| Crash during staging | `$STAGING` exists without `.complete`, or `$CURRENT` missing | Do not treat `$STAGING` or `$SRC/target` as current |
| Crash during `mv` to `$PUB` | `$PUB` missing or `$CURRENT` missing | No pointer, no install |
| Crash replacing `$CURRENT` | `readlink current` is not `published/<artifact_id>` | Operator stops; no smoke |
| Crash during `install.sh` after rust `install` and before python `install` | `$PREFIX/bin/codefire` exists and `$PREFIX/bin/codefire-py` does not | Prefix is not the candidate; rollback to `$PRIOR`; do not smoke |
| `install.sh` without `--binary` | would run unlocked `cargo build --release` inside `$SRC` | Forbidden even if cargo exists |
| `install.sh` without `--prefix` | would target `/usr/local` | Forbidden |
| Wrong binary, help/version-shaped | `install.sh` accepts any executable whose `--help` contains `usage: codefire <command> [args]` and `--version` matches `codefire foundation *` (see `tests/test_codefire_cli.py` fake binary) | SHA-256 ≠ provenance `binary_sha256` |
| B1 binary presented as d803527 candidate | provenance `base_revision` is `c78d5e0e…` | Reject even though rust input digest matches |
| Dirty-main binary | rust inputs differ from HEAD; provenance builder rejects | Drop |
| Capabilities/help/version mismatch | `--version` ≠ `codefire foundation 1`; help lacks the `[args]` usage line; capabilities JSON schema id ≠ `codefire.capabilities.v1` | Wrong artifact |
| External publication | any git push, GitHub release, PyPI, crates.io, URL fetch, credential, or network upload | Out of M1; fail the rehearsal |

Publication is local and atomic if and only if `$CURRENT` appears in one `mv` of a complete tree. `install.sh` itself is not atomic across the two binaries; isolation plus prior-empty rollback is the M1 compensation.

### 6. Required M1 evidence vs parked Stage 4

**Required M1 evidence (this slice):**

- one clean source SHA and rust input digest;
- one fresh `codefire.rust_build_provenance.v1` bound to that SHA;
- SHA-256 of rust binary, Python fallback, and `install.sh` from the same SHA;
- local `$CURRENT` pointer, isolated `$PREFIX`, empty `$PRIOR`, smoke of `--version` / `--help` / `capabilities --json` plus Python `--help`;
- before/after hashes of integration checkout, dirty main, canonical store, user install, prefix;
- explicit `internal-rehearsal-only` / not public beta / not SemVer.

**Park (Stage 4 / later; do not do in M1):**

- production signing, SLSA, SBOM, SPDX;
- GitHub/PyPI/crates.io/registry upload, tags (repository has **zero** tags), credentials;
- `SECURITY`, `CONTRIBUTING`, `GOVERNANCE`, `MAINTAINERS`, support/security policy, backup drills, two-person response;
- durable pinned toolchain/CI identity (cargo absent here);
- binding `install.sh` to provenance / adding `--locked` to its cargo path;
- putting source SHA into CLI `--version` or inventing product SemVer;
- `python3 -m pip install .` / setuptools as the default install;
- completing the full conformance matrix beyond checked `schemas/conformance/manifest-v1.json`;
- rights/license text changes (node 1 owns that lane).

**Drop:**

- dirty main `/workspace/project` as source or install prefix;
- treating B1 provenance `base_revision=c78d5e0e` as d803527 identity;
- public beta / v1.0 / signed-release labels;
- mutating canonical Algorithm or the user's PATH install;
- M2–M5 work.

Observed rights facts (inventory only; node 1 decides): no root `LICENSE`/`NOTICE`/`SECURITY`/`CONTRIBUTING`/`GOVERNANCE`/`MAINTAINERS`; Cargo workspace `license = "MIT"`; `setup.py` `UNLICENSED`; `pyproject.toml` has no license field. Do not infer publication permission from that.

### 7. First missing contract, verdict, write set

**First missing contract:** there is no checked-in local-only atomic publication pointer, multi-artifact checksum set, isolated prefix, or rollback snapshot. Existing surfaces cover a **rust-only** source-bound build (`build_source_bound_rust.py` + `codefire.rust_build_provenance.v1`, tests in `docs/development/tools/test_ai_tools.py::RustBuildProvenanceTests` / `CapabilityProvenanceTests`) and an in-place installer (`install.sh`, tests `test_install_script_installs_codefire`, `test_install_script_dry_run_bad_binary_and_prefix_safety`, `test_install_script_completion_failure_does_not_install_binary`). The installer does not publish a candidate, does not hash Python/docs, does not snapshot prior prefix, and will install a help/version-shaped impostor.

That gap is **not** a product `block-m1`. It is closed by the operator topology in this packet.

| Class | Item |
|---|---|
| **block-m1** | **none reproduced.** Isolated release/verify/rollback is possible if node 5 follows this packet with a fresh source-bound build and node 1 admits internal-only rights. |
| **park** | signing/SBOM/registries/policies; toolchain pin; installer provenance binding; SemVer/`--version` source SHA; pip install path; full Stage 4. Cargo absence is an environment admission constraint for node 5a, not a source write set. |
| **drop** | dirty-main candidate; B1-as-d803527; public beta; external upload. |
| Smallest fix write set | **none.** |

Focused tests named for later nodes (not run here): `RustBuildProvenanceTests`, `test_valid_release_provenance_marks_only_release_binary_bound`, the three `install.sh` unittests, `bash -n install.sh`. Do not run `test_install_script_installs_codefire` against `/usr/local` or `$HOME/.local`; that test builds via `install.sh` without `--binary`.

### Node-3 handoff

1. Fan-in with node 1. Rehearse only artifacts node 1 marks internal-rehearsal-safe. Missing rights stay a public-distribution blocker and must not be inferred from this topology.
2. Admit node 5a only on a **clean** checkout whose rust input digest remains `3925fc035690296fa5e132b630dab65a7fbd5ddb1a55f0fb1226e6efd295d97a` (101 files) and that has `cargo`+`rustc`. Preferred checkout: the commit of this packet (procedure in-tree) or d803527 (rust/python/install bytes identical). Dirty main is ineligible.
3. Node 5a: `python3 docs/development/tools/build_source_bound_rust.py` from that checkout; copy the closed artifact set into a new `$RELEASE_ROOT`; publish with the atomic `mv`/`ln` sequence; do not use `install.sh` to compile; do not reuse B1 provenance.
4. Node 5b: fresh other-family operator, `$RELEASE_ROOT` + this packet only; checksum and provenance before install; absolute-path smoke in `$SMOKE_CWD`; rollback `$PREFIX` to empty prior; re-hash preservation subjects.
5. Do not start nodes 4, 5a, 5b, 6, or 7 from this packet alone. Do not serialize behind an invented product fix.

Inspected surfaces: `install.sh`, `pyproject.toml`, `setup.py`, `Cargo.toml` / crate `0.1.0` manifests, `docs/development/tools/build_source_bound_rust.py`, `probe_validation_capabilities.py`, `docs/development/tools/test_ai_tools.py`, `tests/test_codefire_cli.py` installer tests, `crates/codefire-cli/src/{main.rs,completion.rs,command_registry.rs}`, `crates/codefire-core/src/lib.rs` `VERSION`, `codefire` Python `VERSION = 1` / `ArgumentParser(prog="codefire")`, `schemas/conformance/manifest-v1.json`, `schemas/public/capabilities-v1.schema.json`, `docs/development/strategy/release-and-format-roadmap.md`, `docs/development/strategy/modernization-roadmap.md` Stage 4, `docs/04_cli_spec.md`, `docs/development/known-limitations.md` installer notes, retained B1 provenance, dirty main, canonical store.
