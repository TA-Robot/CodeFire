# AI Task Packet

Status: operator-ready

Packet ID: MS-release-operational-trust-run

Schema version: human-v1

Risk score: 8

Reasoning tier: high

## Objective

Build one fresh source-bound Rust candidate from the accepted clean release-M1 source, assemble an
internal-only closed artifact set under a new local release root, publish it atomically inside that
root, hand it to a fresh other-family operator, and prove checksum/provenance verification, isolated
install/smoke, and rollback. Do not publish externally and do not claim public release rights.

## Scope

- one fresh source-bound Rust build and immutable local candidate;
- Rust CLI, Python fallback, installer, source/provenance/checksum records, and three public docs;
- one fresh-operator verification/install/smoke/rollback pass in an isolated prefix;
- preservation evidence and this packet only outside build/temp artifacts.

## Fan-In Decision

Node 1 found no product `block-m1`: every artifact is **internal-rehearsal only**, no artifact is
admissible for public distribution, and missing outbound licence/copyright-holder evidence remains a
public-distribution blocker. Node 2 found no product `block-m1`: a fresh clean-source build plus a
local atomic publication pointer and isolated rollback can satisfy the mechanical path.

Admitted synthesis:

- build fresh; do not reuse the B1 runner because its provenance binds `c78d5e0e`, not the accepted
  release source revision;
- no registry-capable command, tag, push, URL fetch, credential, or network publication is admitted;
- do not add `publish = false` because no registry path is admitted; if that boundary changes, stop;
- do not change licence/package metadata or infer public permission;
- exclude wheel/sdist, full source archive, `subprojects/**`, `docs/development/**`, `.codex/**`, and
  `audit-latest.md` from the candidate;
- resolve the classifier conflict by replacing node 2's internal
  `docs/development/strategy/release-and-format-roadmap.md` artifact with the public
  `docs/automation_interface.md`. The three docs are exactly `README.md`, `docs/04_cli_spec.md`, and
  `docs/automation_interface.md`.

No node-4 product fix is admitted.

## Fixed Inputs

- source: the clean persistent node-2 worktree after it merges the current integration branch;
- required Rust input manifest before build:
  `3925fc035690296fa5e132b630dab65a7fbd5ddb1a55f0fb1226e6efd295d97a`, 101 files;
- toolchain: Rust/cargo 1.97.0 from the existing isolated cache; record absolute toolchain identity;
- canonical preservation baseline, dirty-main baseline, historical-copy baseline, and user-install
  baseline from node 2 Result section 4, rehashed immediately before setup;
- rights decision and version tuple from node 1 Result sections 3–6;
- topology, commands, falsifiers, and rollback rules from node 2 Result sections 2–5 except for the
  admitted public-doc substitution above.

The exact source SHA is the node-2 branch HEAD after merging this packet. Record it before build. The
worktree must be clean and its Rust input manifest must equal the fixed manifest; otherwise stop.

## Stage A — Persistent Node-2 Owner

1. Merge the current integration branch into the persistent node-2 branch. Record exact clean HEAD.
2. Rehash every fixed input and preservation subject. Stop on drift.
3. Run the packet's focused pre-build validations without changing source.
4. Build only with:

   ```bash
   python3 docs/development/tools/build_source_bound_rust.py \
     --output target/codefire-ai/rust-build-provenance.json
   ```

5. Create a new mode-700 `mktemp -d /tmp/codefire-m1-release.XXXXXX` on one filesystem. Follow node 2
   Result section 2 for `$STAGING`, `$PUB`, `$CURRENT`, `$PREFIX`, `$PRIOR`, `$EVIDENCE`, and
   `$SMOKE_CWD`.
6. Assemble the exact candidate below. Recompute every file hash from the accepted source:

   ```text
   RELEASE_MANIFEST.json
   RIGHTS_DECISION.txt
   SHA256SUMS
   SOURCE_REVISION
   rust-build-provenance.json
   rust-build-inputs.json
   artifacts/codefire-rs
   artifacts/codefire
   artifacts/install.sh
   docs/README.md
   docs/04_cli_spec.md
   docs/automation_interface.md
   ```

7. `RIGHTS_DECISION.txt` must state exactly: internal rehearsal only; public distribution not
   authorized from repository evidence; outbound licence and copyright holder unresolved; Cargo MIT,
   Python UNLICENSED, and pyproject silence conflict; no public beta/SemVer/signed-release claim.
8. `RELEASE_MANIFEST.json` must retain node 2's identity/version/artifact fields and additionally
   include `public_distribution_authorized:false`, `rights_status:"unresolved"`,
   `copyright_holder:"unidentified"`, `sbom_present:false`, and `external_publication:false`.
9. Compute `SHA256SUMS` over every other regular candidate file using relative paths. Write `.complete`
   last, make staging read-only, rename it once into `$PUB`, then atomically replace `$CURRENT` with a
   relative symlink. An incomplete tree is never current.
10. Run no installer or product smoke. Record Stage A evidence and hashes, update this packet to
    `operator-ready`, validate, and commit only this packet. Preserve the release/source roots.

## Stage B — Fresh Other-Family Operator

The operator receives only the exact source worktree path, `$RELEASE_ROOT`, this packet, and the two
completed classifier packets. It receives no private correction channel.

1. Freeze candidate/evidence hashes before reading conclusions.
2. Verify `$CURRENT`, `.complete`, `SOURCE_REVISION`, `SHA256SUMS`, manifest rights fields, the three
   exact artifact digests, and provenance with node 2 Result section 3 commands.
3. Reject any B1 provenance, dirty-main source, wrong binary, incomplete pointer, hidden external
   publication, or public-release claim.
4. Snapshot the empty `$PREFIX` into `$PRIOR`; install only with both `--prefix` and `--binary` from
   `$CURRENT/artifacts`; do not export `PATH`.
5. Smoke from the empty `$SMOKE_CWD` by absolute path: Rust `--version`, `--help`,
   `capabilities --json`, and Python `--help`. Compare exact expected identities from node 2.
6. Roll back `$PREFIX` to the empty prior and prove both installed binaries are absent.
7. Rehash every preservation subject. Leave `$PUB`, `$CURRENT`, and all evidence intact.
8. Record exact commands, cwd, timestamps, stdout/stderr hashes, install/rollback hashes, result, and
   public-claim refusal in this packet; validate and commit only this packet. Do not rerun.

## Validation

Stage A:

```bash
cargo fmt --check
bash -n install.sh
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  docs.development.tools.test_ai_tools.RustBuildProvenanceTests
python3 docs/development/tools/check_source_docs.py
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_ai_acceleration_docs.py
git diff --check
```

Stage B runs the exact identity/checksum/provenance/install/smoke/rollback commands plus the three
management checks. No workspace-wide test, public upload, or canonical CodeFire command is required.

## Non-Goals

- public release, registry/GitHub upload, tag, signing, credential, network, public beta, or SemVer;
- choosing/fixing a licence, creating a wheel/sdist/SBOM, or editing package/install/product docs;
- changing `install.sh`, provenance schema, source, tests, public schemas, or product behavior;
- installing to `/usr/local`, `$HOME/.local`, PATH, or any non-isolated prefix;
- invoking CodeFire against canonical Algorithm, dirty main, or any repository;
- Stage 4 policy completion, M2 product research, M3–M5 ecosystem work, or open issue residuals.

## Handoff

Return exact source and toolchain identity, release/source roots, artifact manifest and hashes,
rights/public-claim fields, atomic-publication proof, fresh-operator identity, every verification and
smoke result, rollback proof, preservation hashes, `block-m1`, parked limitations, validation, and
commits. Preserve all evidence through the final independent audit.

## Result

Status **operator-ready**. Stage A only. Inspected and built at exact clean HEAD
`01a4de0ffcc775df08473e2088f639e69f268ec5`. No install, no smoke, no CodeFire invocation, no
external publication, no source/product/public-doc edit. Write set: this packet only.

`block-m1`: none reproduced in Stage A.

### Stage A identity

| Item | Value |
|---|---|
| Source worktree | `/workspace/worktrees/m1-release-rehearsal-20260822` |
| Branch | `grok-agent/m1-release-rehearsal-20260822` |
| Bound source SHA | `01a4de0ffcc775df08473e2088f639e69f268ec5` (integration fan-in already merged; porcelain empty before build) |
| Rust input manifest | `3925fc035690296fa5e132b630dab65a7fbd5ddb1a55f0fb1226e6efd295d97a`, 101 files, `source_vs_head_errors=[]` |
| `$RELEASE_ROOT` | `/tmp/codefire-m1-release.vT1Uzs` mode 700, device 79 |
| `$PUB` | `/tmp/codefire-m1-release.vT1Uzs/published/01a4de0ffcc7-90cb2895aadc` mode 500 |
| `$CURRENT` | relative symlink `published/01a4de0ffcc7-90cb2895aadc` |
| `$PREFIX` / `$PRIOR` / `$SMOKE_CWD` | empty directories |
| Artifact id | `01a4de0ffcc7-90cb2895aadc` |
| Stage A UTC | 2026-08-22T04:10:47Z |

### Toolchain (inherited isolated 1.97.0)

| Item | Value |
|---|---|
| `CARGO_HOME` | `/home/devuser/.cache/codefire-rust-1.97/cargo` |
| `RUSTUP_HOME` | `/home/devuser/.cache/codefire-rust-1.97/rustup` |
| `RUSTUP_TOOLCHAIN` | `1.97.0-x86_64-unknown-linux-gnu` (active because env override) |
| rustup proxies | `/home/devuser/.cache/codefire-rust-1.97/cargo/bin/{cargo,rustc}` → `rustup` |
| cargo binary | `/home/devuser/.cache/codefire-rust-1.97/rustup/toolchains/1.97.0-x86_64-unknown-linux-gnu/bin/cargo` SHA-256 `eff12bab37b9d9e01324db4583eaf55b2cd82ac3008a7e59876e4cd2e9a028f5` |
| rustc binary | `/home/devuser/.cache/codefire-rust-1.97/rustup/toolchains/1.97.0-x86_64-unknown-linux-gnu/bin/rustc` SHA-256 `df13f58759c0662831983e3a6501c63c1fc12ea60ec4e1d1ac35e5fe43c500c0` |
| `cargo --version` | `cargo 1.97.0 (c980f4866 2026-06-30)` |
| `rustc --version` | `rustc 1.97.0 (2d8144b78 2026-07-07)` |
| rustc commit-hash | `2d8144b7880597b6e6d3dfd63a9a9efae3f533d3` |
| host | `x86_64-unknown-linux-gnu` |
| LLVM | 22.1.6 |

Build command (only):

```bash
python3 docs/development/tools/build_source_bound_rust.py \
  --output target/codefire-ai/rust-build-provenance.json
```

Duration 146823 ms. Did not use `install.sh` to compile. Did not reuse B1 provenance.

### Provenance (on-disk file; extra stdout keys not stored)

`target/codefire-ai/rust-build-provenance.json` and `$CURRENT/rust-build-provenance.json` are
byte-identical. File SHA-256 `7998bcf60a5de839f88a40d54f7fabc724da83c09bd34d7403eb97b8cc8d9798`.

| Field | Value |
|---|---|
| schema | `codefire.rust_build_provenance.v1` |
| ok | true |
| base_revision | `01a4de0ffcc775df08473e2088f639e69f268ec5` |
| source_manifest_sha256 | `3925fc035690296fa5e132b630dab65a7fbd5ddb1a55f0fb1226e6efd295d97a` |
| source_file_count | 101 |
| binary_path | `target/release/codefire-rs` |
| binary_bytes | 14590776 |
| binary_sha256 | `90cb2895aadc45b559a96207cdd5ae3b08485f9a7e5a07014db8cca8b5b90ce9` |
| build_command | `cargo build --release --locked -p codefire-cli --bin codefire-rs` |
| build_log_sha256 | `3b11cfc62324f7ad4e011b077b5d16b3f7cc2c17706b96f15796a62aee0aa887` |
| cargo_version | `cargo 1.97.0 (c980f4866 2026-06-30)` |
| rustc_version | `rustc 1.97.0 (2d8144b78 2026-07-07)` |

`validate_provenance_payload` against HEAD `01a4de0` returned `[]`. Binary bytes match B1
(`90cb2895…`) because rust inputs and toolchain match; **B1 provenance is still forbidden**
(`base_revision` `c78d5e0e`, provenance file SHA-256 `1731b863…`).

### Closed candidate and relative checksums

`$CURRENT` contains exactly:

```text
.complete                         empty, SHA-256 e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
RELEASE_MANIFEST.json             126721cd3b3d775f6587a17d36fdebe25bd62eaafb3e88d50af5edaa7c6d91f4
RIGHTS_DECISION.txt               3742080e65967f055cbdfdadbf8794e23d695f5bcefd66b29681b7286f6242b9
SHA256SUMS                        b434d3188cd2c96f06e582c4e2cef4916797d41516d7799571490accea68c3c3
SOURCE_REVISION                   1e028055e356ceaf1bb0a7b74e57f4b386865656fa437fdb40ab78805950b228
artifacts/codefire                efd124fd442addf1c7f385a73d0fbbe959e3455d609ab3d5eb09dc82fb54fa10
artifacts/codefire-rs             90cb2895aadc45b559a96207cdd5ae3b08485f9a7e5a07014db8cca8b5b90ce9
artifacts/install.sh              0bc5a6cb4a89bc003ce667e0acff79c0619a0c60db1c3b1be5ed0138da59752e
docs/04_cli_spec.md               b71428806556534f1d50fd0538e7680eded78a08db673d4b24c876146264b28e
docs/README.md                    422174c3a419d846abce1844461e449c4cbb8178a0983aac2afc3911f355110e
docs/automation_interface.md      9c87a4c98e4d2b9c24dd35d4fe6a76feb79a6e6e95938c4ef713497eb034efe6
rust-build-inputs.json            7b6ddc5283a7a269a1cedf760b3972a62ef64d7dbbb7d0ce1770b5022ff7f2da
rust-build-provenance.json        7998bcf60a5de839f88a40d54f7fabc724da83c09bd34d7403eb97b8cc8d9798
```

No `docs/development/**`, no `release-and-format-roadmap.md`, no wheel/sdist, no `subprojects/**`,
no `.codex/**`, no `audit-latest.md`. `SHA256SUMS` lists the 11 regular files above excluding itself
and `.complete`. `sha256sum -c SHA256SUMS` passed from `$CURRENT`.

`RIGHTS_DECISION.txt` is exactly:

```text
internal rehearsal only; public distribution not authorized from repository evidence; outbound licence and copyright holder unresolved; Cargo MIT, Python UNLICENSED, and pyproject silence conflict; no public beta/SemVer/signed-release claim.
```

`RELEASE_MANIFEST.json` rights/public-claim fields:

| Field | Value |
|---|---|
| channel | `internal-rehearsal-only` |
| public_beta | false |
| semver_release | false |
| signed | false |
| public_distribution_authorized | false |
| rights_status | `unresolved` |
| copyright_holder | `unidentified` |
| sbom_present | false |
| external_publication | false |
| source_revision | `01a4de0ffcc775df08473e2088f639e69f268ec5` |
| source_worktree_clean | true |

### Atomic publication proof

1. Assembled under `$RELEASE_ROOT/staging.work` while `$CURRENT` was absent.
2. Wrote `SHA256SUMS` over the other 11 files (relative paths, two-space GNU format).
3. Wrote empty `.complete` last.
4. `chmod -R a-w` on staging files. Linux `rename(2)` requires write on the directory itself to
   update `..`, so owner-write was restored on the staging directory only, then `mv` to `$PUB`,
   then `chmod a-w` on `$PUB` (now mode 500). Incomplete root
   `/tmp/codefire-m1-release.j6JHUw` never had `$CURRENT` and was deleted fail-closed.
5. `ln -sfn published/01a4de0ffcc7-90cb2895aadc current.tmp && mv -Tf current.tmp current`.
   `readlink current` is exactly `published/01a4de0ffcc7-90cb2895aadc`. Same filesystem (dev 79).
6. `$PREFIX` and `$PRIOR` left empty. No installer ran.

### Preservation T0 (before mutation) and rehash after publication

Both matched node 2 Result section 4. No drift.

| Subject | Hash / identity |
|---|---|
| Dirty main HEAD | `f1644122134d479333cf422fdb1d48b8fa92dd8e` |
| Dirty main porcelain | 248; SHA-256 `1385b6249f6455510bc449aed054d5ce425494a29edb9a4c21d65165cb6b1bcf` |
| Canonical store | `/workspace/project/subprojects/.codefire` tree `12d84568ee63fc69b54536346da7a2072290edcc952726330be97ff03fdca4d1` (2905 files / 114660974 bytes); `repo.json` `bf6d978b…`; `branches/main.json` `bfd52e35…`; head `CF-COMMIT-2792b54ad730e6d644668c38` |
| Historical copy | `/workspace/algorithm-evolution-agent-lab` tree `b41e7e884e3d18a72e81d29c2d93088fcb3ffdafc537fd6e6687ec1155187495` (1470 / 12232765) |
| User install | no `codefire` on PATH; `/home/devuser/.local/bin` absent; no `/usr/local/bin/codefire` |

### Stage A validations (pre-build, cwd `$SRC`)

```text
cargo fmt --check                                              pass
bash -n install.sh                                             pass
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  docs.development.tools.test_ai_tools.RustBuildProvenanceTests  3 tests OK
python3 docs/development/tools/check_source_docs.py            pass (44 modules, 34 commands, 12 object kinds)
python3 docs/development/tools/check_project_docs.py           pass
python3 docs/development/tools/check_ai_acceleration_docs.py   pass
git diff --check                                               pass
```

Worktree remained clean of tracked files through build (only gitignored `target/` changed).

### Exact fresh-operator handoff (Stage B; do not run from this node)

The operator receives **only**:

1. Source worktree `/workspace/worktrees/m1-release-rehearsal-20260822`
2. `$RELEASE_ROOT=/tmp/codefire-m1-release.vT1Uzs`
3. This packet
4. `docs/development/ai/task-packets/milestone-release-operational-trust-rights.md`
5. `docs/development/ai/task-packets/milestone-release-operational-trust-rehearsal.md`

No private correction channel. No PATH export. No `/usr/local`. No `$HOME/.local`. No Algorithm.
No dirty main. No rebuild. No B1 provenance path.

**HEAD vs provenance:** Stage A will commit this packet, so the worktree HEAD after Stage A is **not**
`01a4de0`. Provenance `base_revision` and `$CURRENT/SOURCE_REVISION` remain `01a4de0`. Node 2
`validate_provenance_payload(..., expected_revision=git_head($SRC))` therefore requires a **clean
checkout of `01a4de0`** with gitignored `target/release/codefire-rs` and
`target/codefire-ai/rust-build-provenance.json` still present. Read this packet first, then detach
`$SRC` to `01a4de0` before those commands. Do not treat the later docs commit as the bound source.

Operator steps (node 2 Result §3, docs substitution already applied in `$CURRENT`):

```bash
RELEASE_ROOT=/tmp/codefire-m1-release.vT1Uzs
SRC=/workspace/worktrees/m1-release-rehearsal-20260822
PREFIX="$RELEASE_ROOT/prefix"
PRIOR="$RELEASE_ROOT/prior"
SMOKE_CWD="$RELEASE_ROOT/evidence/smoke"
CURRENT="$RELEASE_ROOT/current"

# 1. freeze hashes before reading conclusions
# 2. identity (cwd $RELEASE_ROOT)
readlink -f current
test -L current
test "$(readlink current)" = "published/01a4de0ffcc7-90cb2895aadc"
test -f current/.complete
cat current/SOURCE_REVISION
# must be 01a4de0ffcc775df08473e2088f639e69f268ec5

# 3. checksums (cwd $CURRENT)
sha256sum -c SHA256SUMS

# 4. provenance (cwd $SRC at 01a4de0)
test "$(git rev-parse HEAD)" = "$(cat "$CURRENT/SOURCE_REVISION")"
cmp -s target/release/codefire-rs "$CURRENT/artifacts/codefire-rs"
cmp -s target/codefire-ai/rust-build-provenance.json "$CURRENT/rust-build-provenance.json"
python3 -c "import json,sys; from pathlib import Path; sys.path.insert(0,'docs/development/tools'); from build_source_bound_rust import validate_provenance_payload, git_head; root=Path('.').resolve(); payload=json.loads((root/'target/codefire-ai/rust-build-provenance.json').read_text()); err=validate_provenance_payload(root, payload, expected_revision=git_head(root)); print('\n'.join(err) if err else 'provenance-ok'); raise SystemExit(1 if err else 0)"

# reject B1: provenance base_revision must not be c78d5e0e
# reject public_beta/semver/signed/external_publication true
# reject missing public_distribution_authorized=false

# 5. snapshot empty prefix into prior, then install
mkdir -p "$PREFIX" "$PRIOR" "$SMOKE_CWD"
test ! -e "$PREFIX/bin/codefire"
test ! -e "$PREFIX/bin/codefire-py"
# cwd $CURRENT/artifacts
./install.sh --prefix "$PREFIX" --binary "$CURRENT/artifacts/codefire-rs"

# 6. smoke cwd $SMOKE_CWD, absolute paths only
"$PREFIX/bin/codefire" --version
"$PREFIX/bin/codefire" --help
"$PREFIX/bin/codefire" capabilities --json
"$PREFIX/bin/codefire-py" --help
# expected: codefire foundation 1; help contains usage: codefire <command> [args];
# capabilities schema codefire.capabilities.v1; python help contains usage: codefire

# 7. rollback prefix to empty prior
rm -rf "$PREFIX"
mkdir -p "$PREFIX"
test ! -e "$PREFIX/bin/codefire"
test ! -e "$PREFIX/bin/codefire-py"

# 8. rehash preservation subjects; leave $PUB, $CURRENT, evidence intact
```

Reject if `$CURRENT` is missing, `.complete` missing, SHA256SUMS fail, provenance `base_revision`
≠ `01a4de0`, binary SHA-256 ≠ `90cb2895…`, docs set ≠ README/04_cli_spec/automation_interface,
`docs/development` present, or any public-release claim is true.

Stage B is **not** started by this node.

