# AI Task Packet

Status: completed

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

Status **completed**. Stage A remains recorded below exactly as it was written and is unchanged by
Stage B. Stage B executed exactly once and is recorded under "Stage B result (fresh other-family
operator)".

### Stage A record (recorded operator-ready; unchanged)

Stage A only. Inspected and built at exact clean HEAD
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

## Stage B result (fresh other-family operator)

Status **completed**. Stage B ran **exactly once**, in one pass, against
`RELEASE_ROOT=/tmp/codefire-m1-release.vT1Uzs`. Window 2026-08-22T04:15:20Z – 2026-08-22T04:21:20Z UTC.

`block-m1`: **none reproduced in Stage B.** Every admitted verification, install, smoke, and rollback
falsifier passed. One evidence-format gap is classified `park` (B.12).

**Public-claim refusal, restated by the operator on its own authority.** This rehearsal is
**internal-only**. No artifact in `$CURRENT` is admissible for public distribution. The operator did
not infer distribution permission from repository access, from `Cargo.toml` `license = "MIT"`, from
package metadata, from the retired v0.6–v1.0 labels, or from the fact that the build, install, and
smoke all succeeded. The outbound licence and the copyright holder remain unresolved and are recorded
as public-distribution blockers. Not a public beta, not a SemVer release, not signed, not legally
cleared, not published, not third-party-licence compliant, not supported. CodeFire's own HMAC
commit/request signing is a product feature over repository content and is **not** release-artifact
signing; nothing in Stage B is reported as a signature or attestation.

### B.0 Operator identity, preconditions, and refusals honoured

| Item | Value |
|---|---|
| Operator | Claude Opus 5 — other family to Stage A (Grok 4.6), fresh context, no prior release-agent state |
| Private correction channel | none received and none used |
| Inputs received | source worktree, `$RELEASE_ROOT`, this packet, and the two completed classifier packets only |
| Source worktree | `/workspace/worktrees/m1-release-rehearsal-20260822` |
| Branch HEAD at entry | `76975ea40f7b5379eba4af57c1293bb5db158e12`, `git status --porcelain=v1` empty |
| Bound source used for provenance | detached `01a4de0ffcc775df08473e2088f639e69f268ec5` |
| Branch HEAD at exit | `76975ea…` restored via `git switch grok-agent/m1-release-rehearsal-20260822`, tracked tree clean |
| Rebuild | none — no `cargo`, no `build_source_bound_rust.py`, no `install.sh` compile path |
| CodeFire against a repository | none — no `init`/`open`/`status`/`commit`/`scan` against any store |
| Network / external publication | none — no fetch, push, tag, registry, credential, or upload |
| PATH install | none — `PATH` never exported; `/usr/local` and `$HOME/.local` untouched |
| Second pass | none — every command below ran once |
| Write set | this packet (tracked) + `$RELEASE_ROOT/evidence/stage-b/**` (untracked evidence) |

Ordering note, stated plainly: the Stage A handoff instructs the operator to read this packet first
and only then detach `$SRC`. The operator therefore read the packet before hashing. Every hash in B.1
was nevertheless computed **from disk**, never transcribed from the packet, and only afterwards
compared against the recorded values. No recorded value was used as an input to its own check.

### B.1 Frozen hashes before conclusions (T0 = 2026-08-22T04:15:29.675661308Z)

All 13 regular files under `$CURRENT` were independently hashed and **all 13 matched Stage A exactly**
(`evidence/stage-b/T0-candidate-tree.sha256`). The candidate contains exactly 13 regular files:
the 11 checksummed files plus `SHA256SUMS` and `.complete`. No extra file, no symlink, no
non-regular entry, three directories (`.`, `artifacts`, `docs`).

Release-root shape at T0: `$RELEASE_ROOT` mode 700 dev 79; `$PUB` mode 500; `current` a relative
symlink; `prefix`, `prior`, and `evidence/smoke` all empty; every path on device 79.

### B.2 Current pointer, complete marker, source revision (cwd `$RELEASE_ROOT`)

| Check | Result |
|---|---|
| `test -L current` | PASS |
| `readlink current` = `published/01a4de0ffcc7-90cb2895aadc` | PASS |
| `readlink -f current` | `/tmp/codefire-m1-release.vT1Uzs/published/01a4de0ffcc7-90cb2895aadc` |
| `test -f current/.complete` | PASS |
| `.complete` is empty (`e3b0c442…`) | PASS |
| `cat current/SOURCE_REVISION` | `01a4de0ffcc775df08473e2088f639e69f268ec5` |
| `SOURCE_REVISION` = bound source, 41 bytes, newline-terminated | PASS |

### B.3 Checksums — exact 11, verified and closed (cwd `$CURRENT`)

`sha256sum -c SHA256SUMS` → **11 of 11 `OK`**, exit 0. `SHA256SUMS` has exactly 11 lines, lists
neither itself nor `.complete`, and every path is relative (no absolute path, no `..`).

Set-equality was checked in both directions: the 11 listed paths are exactly the set of regular files
under `$CURRENT` minus `SHA256SUMS` and `.complete` — **no omission and no extra**. This closes the
"listed-but-absent" and "present-but-unchecksummed" gaps that a plain `-c` pass alone would not.

### B.4 Independent rejection of forbidden rights / public-release fields

`RIGHTS_DECISION.txt` is byte-identical to the wording required by Stage A step 7 (digest
`3742080e…`, 242 bytes), verified verbatim.

`RELEASE_MANIFEST.json` (digest `126721cd…`) asserted independently by the operator, not read back
from Stage A's table:

| Requirement | Observed | Verdict |
|---|---|---|
| `public_beta` present and false | false | PASS |
| `semver_release` present and false | false | PASS |
| `signed` present and false | false | PASS |
| `public_distribution_authorized` present and false | false | PASS |
| `sbom_present` present and false | false | PASS |
| `external_publication` present and false | false | PASS |
| `channel` | `internal-rehearsal-only` | PASS |
| `rights_status` | `unresolved` | PASS |
| `copyright_holder` | `unidentified` | PASS |
| `source_revision` | `01a4de0ffcc775df08473e2088f639e69f268ec5` | PASS |
| `source_worktree_clean` | true | PASS |
| `version_axes.product_semver` | `unassigned` | PASS |

Rejection was constructed to fail closed: a **missing** required field is a reject, not only a
`true` one. No field other than `source_worktree_clean` is true anywhere in the manifest. A
substring scan found no `publish`, registry, PyPI, crates.io, URL, token, credential, signature,
attestation, SLSA, in-toto, upload, `audit-latest`, `subprojects`, `docs/development`, wheel, sdist,
or licence key. The version tuple is reported as a tuple (Cargo `0.1.0`, Python `0.2.0`, CLI
`codefire foundation 1`, object `1`, commit root `2`, atom hash `4`, envelope
`codefire.command_result.v1`, capabilities `codefire.capabilities.v1`, support
`codefire.support.v1`, layout `1`), never as a single version number.

### B.5 Closure and the three exact artifact digests

`docs/` contains exactly 3 files (`04_cli_spec.md`, `README.md`, `automation_interface.md`);
`artifacts/` exactly 3 (`codefire`, `codefire-rs`, `install.sh`). Absent from the candidate, each
checked by path glob: `docs/development`, `release-and-format-roadmap.md`, `subprojects`, `.codex`,
`audit-latest`, `*.whl`, `*.tar.gz`, sdist, `LICENSE`, `NOTICE`.

| Artifact | Independently recomputed SHA-256 | Expected | Verdict |
|---|---|---|---|
| `artifacts/codefire-rs` | `90cb2895aadc45b559a96207cdd5ae3b08485f9a7e5a07014db8cca8b5b90ce9` | same | PASS |
| `artifacts/codefire` | `efd124fd442addf1c7f385a73d0fbbe959e3455d609ab3d5eb09dc82fb54fa10` | same | PASS |
| `artifacts/install.sh` | `0bc5a6cb4a89bc003ce667e0acff79c0619a0c60db1c3b1be5ed0138da59752e` | same | PASS |

### B.6 Provenance at the bound source (detached `01a4de0`)

Preconditions before switching: tracked tree clean; gitignored `target/release/codefire-rs`
(`90cb2895…`) and `target/codefire-ai/rust-build-provenance.json` (`7998bcf6…`) present; both
confirmed ignored by `.gitignore:7` so `git switch` preserves them.

```bash
git switch --detach 01a4de0ffcc775df08473e2088f639e69f268ec5
```

Exit 0. HEAD became `01a4de0ffcc775df08473e2088f639e69f268ec5`; `git status --porcelain=v1` empty;
both gitignored artifacts survived byte-identical.

| Check (cwd `$SRC` @ `01a4de0`) | Result |
|---|---|
| `git rev-parse HEAD` = `$CURRENT/SOURCE_REVISION` | PASS |
| `cmp -s target/release/codefire-rs $CURRENT/artifacts/codefire-rs` | PASS byte-identical |
| `cmp -s target/codefire-ai/rust-build-provenance.json $CURRENT/rust-build-provenance.json` | PASS byte-identical |
| `validate_provenance_payload(..., expected_revision=git_head(root))` | `provenance-ok`, exit 0 |
| `validate_source_inputs_against_head` | `source_vs_head_errors=[]` |

Provenance payload fields asserted independently: `schema=codefire.rust_build_provenance.v1`,
`ok=true`, `base_revision=01a4de0ffcc775df08473e2088f639e69f268ec5`,
`source_manifest_sha256=3925fc03…`, `source_file_count=101`, `binary_sha256=90cb2895…` (recomputed
equal to the candidate binary), `build_command` exactly
`cargo build --release --locked -p codefire-cli --bin codefire-rs` (`--locked` present).
`rust-build-inputs.json`: `schema=codefire.rust_build_inputs.v1`, `manifest_sha256=3925fc03…`,
101 files, all paths repo-relative.

**B1 rejected.** `base_revision` is not `c78d5e0e…`; the forbidden B1 provenance file
(`1731b863…`) was never read, referenced, or retargeted. Binary bytes equalling B1's `90cb2895…` is
expected from identical Rust inputs and toolchain and was **not** accepted as identity — identity
rests on `base_revision` plus `validate_provenance_payload` against the current HEAD.

**Candidate re-derived from tracked bytes.** Beyond the checksum set, each candidate blob was
compared with `cmp` against its tracked source at `01a4de0`: `README.md`→`docs/README.md`,
`docs/04_cli_spec.md`, `docs/automation_interface.md`, `codefire`→`artifacts/codefire`,
`install.sh`→`artifacts/install.sh`. All five byte-identical. No dirty-main source, no wrong binary,
no incomplete pointer, no hidden external publication.

### B.7 Install into the isolated prefix only

Pre-install: `$PREFIX` a real directory (not a symlink), 0 entries, both target binaries absent;
`cp -a "$PREFIX/." "$PRIOR/"` captured the **empty** prior (0 entries, empty-tree digest
`e3b0c442…`); `$SMOKE_CWD` empty; no `codefire` on PATH.

```text
cwd           /tmp/codefire-m1-release.vT1Uzs/published/01a4de0ffcc7-90cb2895aadc/artifacts
argv          ./install.sh --prefix /tmp/codefire-m1-release.vT1Uzs/prefix \
                           --binary /tmp/codefire-m1-release.vT1Uzs/current/artifacts/codefire-rs
start_utc     2026-08-22T04:19:37.344983731Z
end_utc       2026-08-22T04:19:37.383673187Z
exit_code     0
stdout        154 bytes  sha256 20cc36768436b162376ab4e9c0861c284bc85fcc98e24a083ef9a4907fa70c53
stderr        157 bytes  sha256 39ca910534197aa6703784f9adda5284ac54f3d356c775125290fa6954cbaeeb
```

Both `--prefix` and `--binary` were passed; neither was ever omitted, so the unlocked
`cargo build --release` path and the default `/usr/local` prefix were both structurally unreachable.
`--completion` was not passed and no completion file was installed. `PATH` was not exported — the
installer's own stderr line ("no active codefire found on PATH") is independent evidence that no
PATH-visible install exists.

Post-install `$PREFIX`: `bin/` (0755), `bin/codefire` (0755, 14590776 B), `bin/codefire-py`
(0755, 1341034 B).

| Installed file | SHA-256 | Equals candidate |
|---|---|---|
| `$PREFIX/bin/codefire` | `90cb2895aadc45b559a96207cdd5ae3b08485f9a7e5a07014db8cca8b5b90ce9` | `cmp` byte-identical to `artifacts/codefire-rs` |
| `$PREFIX/bin/codefire-py` | `efd124fd442addf1c7f385a73d0fbbe959e3455d609ab3d5eb09dc82fb54fa10` | `cmp` byte-identical to `artifacts/codefire` |

Both binaries installed, so the "crash between the two `install` calls" falsifier did not fire.
`$CURRENT` still resolved to `published/01a4de0ffcc7-90cb2895aadc` and re-verified 11/11 `OK`
immediately after install. `codefire` still absent from PATH, `/usr/local/bin/codefire` absent,
`$HOME/.local/bin` absent.

### B.8 Smoke — absolute paths only, from the empty `$SMOKE_CWD`

cwd `/tmp/codefire-m1-release.vT1Uzs/evidence/smoke` for all four; 0 entries before and 0 after, so
the smoke cwd was never written to. Every invocation used an absolute path; no PATH lookup, no
relative path, no repository argument, no CodeFire subcommand against any store.

| # | argv | start_utc | end_utc | exit | stdout bytes / sha256 | stderr bytes / sha256 |
|---|---|---|---|---|---|---|
| 1 | `$PREFIX/bin/codefire --version` | `04:20:05.890556961Z` | `04:20:05.893187245Z` | 0 | 22 / `4a2fd935f1976a3e651b0dbac4a16907685c8cb8fc40b49d4cf5ae04664623bf` | 0 / `e3b0c442…` |
| 2 | `$PREFIX/bin/codefire --help` | `04:20:05.903040380Z` | `04:20:05.906869964Z` | 0 | 825 / `79e58bed9a6d4de3fb591ce1c9b0332300e87e2431c45c2681f547852c062960` | 0 / `e3b0c442…` |
| 3 | `$PREFIX/bin/codefire capabilities --json` | `04:20:05.918073268Z` | `04:20:05.921253411Z` | 0 | 21970 / `f3f18424d8aa5bcd4a22a8ced1a51b967892f25901dc23bdc42ec2c5712d9d65` | 0 / `e3b0c442…` |
| 4 | `$PREFIX/bin/codefire-py --help` | `04:20:05.932001716Z` | `04:20:06.242364284Z` | 0 | 554 / `329f392e1cd0d10e1759e750df3de344c8238943f6fcc1935d5b750c4be59ab6` | 0 / `e3b0c442…` |

All four exit 0 with **empty stderr** (`e3b0c442…` is the empty-string digest).

Identity comparison against the expected values from node 2 Result §3:

| Command | Expected | Observed | Verdict |
|---|---|---|---|
| Rust `--version` | exactly `codefire foundation 1` | exactly `codefire foundation 1` | PASS |
| Rust `--help` | contains `usage: codefire <command> [args]` | present | PASS |
| Rust `capabilities --json` | envelope `codefire.command_result.v1`; `data.schema` `codefire.capabilities.v1`; `data.type` `codefire_command_capabilities`; `support.profile` `codefire.support.v1` | all four match | PASS |
| Python `--help` | contains `usage: codefire` | `usage: codefire [-h] {…}` | PASS |

The capabilities payload was also scanned for public-release claims: no `public_beta`, `semver`,
`signed_release`, `pypi`, `crates.io`, or licence assertion. The wrong-artifact falsifier is
therefore closed on both axes — the installed binary's digest equals provenance `binary_sha256`
**and** its three identity surfaces match, so a help/version-shaped impostor could not have passed.

### B.9 Rollback to the empty prior

```bash
rm -rf "$PREFIX"; mkdir -p "$PREFIX"; cp -a "$PRIOR/." "$PREFIX/"
```

start `2026-08-22T04:20:34.672370578Z`, end `2026-08-22T04:20:34.682874606Z`. Prefix went from 3
entries to 0.

| Proof | Result |
|---|---|
| `test ! -e $PREFIX/bin/codefire` | PASS — absent |
| `test ! -e $PREFIX/bin/codefire-py` | PASS — absent |
| `$PREFIX/bin` itself | absent entirely |
| `$PREFIX` entry count | 0 |
| `$PREFIX` tree digest vs `$PRIOR` | both `e3b0c442…` — identical to the empty prior |
| `codefire` on PATH | absent |
| `/usr/local/bin/codefire`, `$HOME/.local/bin` | both absent |

`$PUB` and `$CURRENT` survived rollback: `readlink current` unchanged, 11/11 `OK`, `.complete`
present.

### B.10 Preservation rehash — zero drift

Every preservation subject was hashed at T0 (`04:15:29Z`, before any mutation) and rehashed at T1
(`04:20:50Z`, after rollback). The substantive diff is **empty**
(`evidence/stage-b/T0-T1-diff.txt`, "RESULT: zero drift").

| Subject | T0 = T1 | Value |
|---|---|---|
| Dirty main `/workspace/project` | unchanged | HEAD `f1644122134d479333cf422fdb1d48b8fa92dd8e`; 248 porcelain; porcelain SHA-256 `1385b6249f6455510bc449aed054d5ce425494a29edb9a4c21d65165cb6b1bcf` — matches node 2 T0 exactly; never written to |
| Canonical store `/workspace/project/subprojects/.codefire` | unchanged | 2905 files / 114660974 bytes; `repo.json` `bf6d978b0eece0f185c6995630339326f56513a0c8645af3e7e984ad69a7f054`; `branches/main.json` `bfd52e3525f6a6faf0db4f3d2d1d9d7fa3fa0d400f6561c3c5b8227e79e989f8`; `repository_id` `repo_8a9e9ea836af`; head `CF-COMMIT-2792b54ad730e6d644668c38` `open-clean` |
| Historical copy `/workspace/algorithm-evolution-agent-lab` | unchanged | 1470 files / 12232765 bytes; `repo_f9f434abf020`; head `CF-COMMIT-5d3f0f33e5ce16ab34d72ac1` `open-clean` |
| User install | unchanged | no `codefire` on PATH; `$HOME/.local/bin` absent; `/usr/local/bin/codefire` absent (`/usr/local/bin` exists) |
| `$PUB` / `$CURRENT` | intact | candidate T0 vs T1 digest lists identical across all 13 files |
| `$PREFIX` / `$PRIOR` / `$SMOKE_CWD` | empty | 0 / 0 / 0 entries |

Operator-defined, reproducible digest used for the two trees:
`PRESERVE_DIGEST(dir) = sha256(` concatenation of `"<sha256(content)>  <relpath>\n"` over all regular
non-symlink files, relpaths sorted `LC_ALL=C` `)`. Observed at both T0 and T1:
`/workspace/project/subprojects/.codefire` →
`a941db6fe3e9ce5f80c1c51ddd5ca26f424cc9c8bd84cd264a4e2d2d9449a4fa`;
`/workspace/algorithm-evolution-agent-lab` →
`4259b2d8862df08c2868ecbc188653a155c8039ea2267237bff78a7328770886`.

### B.11 Evidence

All Stage B evidence is under `$RELEASE_ROOT/evidence/stage-b/` (mode 700), 30 records plus
`EVIDENCE-MANIFEST.sha256`. Manifest digest
`3f876e47f5117fae8ccee9cd5167790f031edba148dc0fe05ca595ba5836a949`. `$EVIDENCE/smoke` was
deliberately left empty; no evidence file was written into the smoke cwd. Nothing under `$PUB`,
`$CURRENT`, or Stage A's evidence was modified or deleted.

### B.12 Finding classified `park` — preservation tree-digest algorithm is unrecorded

Node 2 §4 and Stage A record two directory digests (`12d84568…` for the canonical store,
`b41e7e88…` for the historical copy) without recording the algorithm that produced them. Seven
plausible formulations were tried (`sha256sum` line-stream over `%P`-sorted and `./`-prefixed paths;
hash-then-tab-path; content-only concatenation; and four Python path/content interleavings) and none
reproduced either value, so those two fields are **not independently reproducible** as written.

This is **not** `block-m1` and did not stop Stage B, because every preservation field whose
algorithm *is* unambiguous matched the recorded baseline exactly — dirty-main HEAD and porcelain
SHA-256, `repo.json`, `branches/main.json`, both `repository_id`s, both head commit ids and states,
both file counts (2905, 1470), and both byte totals (114660974, 12232765). Non-mutation is
established by that evidence plus the empty T0→T1 diff under the documented digest in B.10.

`park`: a future packet that records a directory digest should also record its algorithm, or use
`PRESERVE_DIGEST` as defined in B.10. Recommendation for the final audit: adopt one stated
formulation so preservation claims are reproducible by a fresh operator rather than only by their
author.

### B.13 Handoff to the final independent audit

1. **Verdict.** Stage B completed in one pass. Verification, isolated install, smoke, and rollback
   all passed. `block-m1`: none. One `park` (B.12). No product, source, test, public-doc, or
   package-metadata change was made or is proposed.
2. **Preserved for audit.** `$RELEASE_ROOT=/tmp/codefire-m1-release.vT1Uzs` is intact: `$PUB` mode
   500 with all 13 files byte-identical, `current` → `published/01a4de0ffcc7-90cb2895aadc`,
   `evidence/` including `stage-b/` (manifest `3f876e47…`), `prefix`/`prior`/`evidence/smoke` empty.
   Do not delete before the audit reads it.
3. **Source state.** Worktree back on `grok-agent/m1-release-rehearsal-20260822`; the detach to
   `01a4de0` was transient and left no tracked change. Gitignored
   `target/release/codefire-rs` (`90cb2895…`) and `target/codefire-ai/rust-build-provenance.json`
   (`7998bcf6…`) are still present — the audit needs them to re-run
   `validate_provenance_payload`, which requires a detached clean `01a4de0`.
4. **P1 remains `park`, not escalated.** No admitted Stage B step could reach a package registry, so
   no `publish = false` fix became necessary. If any later step gains registry reach, P1 escalates.
5. **Rights position to carry forward unchanged.** Internal-rehearsal-only; no public distribution;
   outbound licence and copyright holder unresolved as recorded public-distribution blockers; all
   eight prohibitions from the rights packet §5 restated at the head of this Stage B result.
6. **Not done, by instruction.** No second Stage B pass, no rebuild, no external publication, no
   PATH install, no CodeFire command against any repository, no network access, and no Stage 4
   signing/SBOM/registry/policy work.

### B.14 Stage B validation

Run at the worktree root on `grok-agent/m1-release-rehearsal-20260822` @ `76975ea`, with only this
packet modified:

```bash
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_ai_acceleration_docs.py
git diff --check
```

Results are recorded in the commit that carries this Stage B section.

