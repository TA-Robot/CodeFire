# Milestone audit

Final independent **other-family** audit of node 6 for active M1 `MS-release-operational-trust`
at exact integration HEAD `62c53f7`. This is the release/verify/rollback final audit. It
authorizes or blocks M1 exit.

- time (ISO): `2026-08-22T04:35:33Z`
- audited HEAD (exact): `62c53f7a6920a976240311e92f8d185bd618b36b` (`62c53f7`, "docs: admit release trust final audit")
- preserved roots audited (read-only):
  - `$RELEASE_ROOT` `/tmp/codefire-m1-release.vT1Uzs`
  - `$CURRENT` → `published/01a4de0ffcc7-90cb2895aadc`
  - source worktree `/workspace/worktrees/m1-release-rehearsal-20260822` @ `9ce05818bc964bce5f871a6c5d52fd5d4ed6cb37`
  - bound source commit `01a4de0ffcc775df08473e2088f639e69f268ec5` (inspected via `git show` / `git ls-tree`; this audit did **not** switch that worktree)
  - historical B1 runner `/workspace/worktrees/ms-adoption-experiment-20260822` (identity-reject control only)
  - dirty main `/workspace/project` @ `f1644122134d479333cf422fdb1d48b8fa92dd8e`
  - canonical store `/workspace/project/subprojects/.codefire`
  - historical Algorithm copy `/workspace/algorithm-evolution-agent-lab`
- active milestone: M1 `MS-release-operational-trust` — Release and operational trust
- scope of this audit: **node 6 only**. Independently recompute terminal evidence. Do not trust packet conclusions. Adjudicate the operator park finding on node-2 legacy directory digests.
- stalled: **no**
- scope_creep: **no**
- next_milestone_risk: low. Nothing in this evidence admits a public release, a licence choice, a registry upload, SemVer, signing, PATH/canonical mutation, or any M2–M5 work.
- auditor: Grok 4.6, fresh session, separate worktree
  `worktrees/m1-release-final-audit-20260822` on branch
  `grok-agent/m1-release-final-audit-20260822` at `62c53f7`. Stage A builder was Grok 4.6;
  Stage B operator was Claude Opus 5. This session is not the builder, not the operator,
  did not use any prior agent conversation, and did not inspect uncommitted executor state.
  Other-family versus the Claude operator is satisfied. Codex was not used.
- write set honored: **this file only.** No product, source, schema, packet, board, issue, or
  evidence file was edited. No worktree was switched. No rebuild, reinstall, or smoke ran.
  This audit issued **no CodeFire command of any kind**.

This audit did not accept packet prose as proof. Every hash, pointer, manifest field, checksum
set, provenance field, smoke identity, rollback emptiness, and preservation value below was
re-derived independently from the preserved roots and from `git` at the audited HEAD.

**Terminal verdict: ACCEPT. `block-m1`: none.**

## 1. Requirement matrix — exact M1 exit bullets

| # | Exact M1 exit bullet | Verdict | How it was verified independently |
|---|---|---|---|
| R1 | rehearsed artifacts have an explicit rights/license and source-identity decision; a missing right remains a public-distribution blocker rather than being inferred | **pass** | `RIGHTS_DECISION.txt` is the required one-sentence internal-only decision (digest `3742080e…`, 242 B). Manifest fail-closed fields are all present and false/unresolved (§2, §4). No permission was inferred from repository access, `Cargo.toml` MIT metadata, package metadata, retired v0.6–v1.0 labels, or a green build/install/smoke. |
| R2 | one exact source revision, version-axis statement, artifact manifest, checksum/provenance set, local publication target, verification procedure, isolated install target, and rollback target were fixed before mutating the isolated release area | **pass** | Bound source is `01a4de0ffcc775df08473e2088f639e69f268ec5`. Closed candidate is the 13-file tree under `$CURRENT`. Rust input manifest `3925fc03…` / 101 files recomputed from git objects at `01a4de0`. `$CURRENT` is an atomic relative symlink to a mode-500 published tree with `.complete` last. |
| R3 | a separate operator follows only the recorded handoff, verifies identity and checksums, exercises the bounded artifact, and restores the declared prior state with before/after hashes | **pass** | Stage B operator was Claude Opus 5 (other family to Stage A Grok). Evidence under `$RELEASE_ROOT/evidence/stage-b/` (manifest `3f876e47…`) records one install, four absolute-path smokes, and one rollback. This audit rehashed candidate, smoke, install, preservation, and emptiness independently (§3–§8). |
| R4 | result records publishable / internal-only / stop, unresolved signing/SBOM/support/security limits, and explicitly refuses a public beta label while source or release identity is unresolved | **pass** | Channel `internal-rehearsal-only`; `public_distribution_authorized=false`; `rights_status=unresolved`; `copyright_holder=unidentified`; `public_beta=false`; `semver_release=false`; `signed=false`; `sbom_present=false`; `external_publication=false`; product SemVer `unassigned`. Version axes reported as a tuple, never a single number. |
| R5 | an independent other-family audit verifies the rights record, source binding, artifact closure, preservation boundary, operator independence, rollback result, and absence of hidden external publication | **pass (this audit)** | This is node 6. Absence checks in §9. Park adjudication in §10. |

## 2. Explicit internal-only / no-public-rights decision

`RIGHTS_DECISION.txt` bytes (242) and SHA-256
`3742080e65967f055cbdfdadbf8794e23d695f5bcefd66b29681b7286f6242b9` were recomputed from disk.
Verbatim text:

```text
internal rehearsal only; public distribution not authorized from repository evidence; outbound licence and copyright holder unresolved; Cargo MIT, Python UNLICENSED, and pyproject silence conflict; no public beta/SemVer/signed-release claim.
```

That is the rights packet's honest decision, not a licence grant. Missing outbound licence text
and missing copyright holder remain public-distribution blockers. Repository access is not
permission.

`RELEASE_MANIFEST.json` SHA-256
`126721cd3b3d775f6587a17d36fdebe25bd62eaafb3e88d50af5edaa7c6d91f4` (1535 B). Fail-closed:
a **missing** required field is a reject, not only a `true` one. Independently asserted:

| Field | Observed | Fail-closed verdict |
|---|---|---|
| `public_beta` | `false` (bool, present) | PASS |
| `semver_release` | `false` | PASS |
| `signed` | `false` | PASS |
| `public_distribution_authorized` | `false` | PASS |
| `sbom_present` | `false` | PASS |
| `external_publication` | `false` | PASS |
| `channel` | `internal-rehearsal-only` | PASS |
| `rights_status` | `unresolved` | PASS |
| `copyright_holder` | `unidentified` | PASS |
| `source_revision` | `01a4de0ffcc775df08473e2088f639e69f268ec5` | PASS |
| `source_worktree_clean` | `true` | PASS |
| `version_axes.product_semver` | `unassigned` | PASS |
| `rust_source_manifest_sha256` | `3925fc035690296fa5e132b630dab65a7fbd5ddb1a55f0fb1226e6efd295d97a` | PASS |
| `rust_source_file_count` | 101 | PASS |

The only JSON `true` anywhere in the manifest is `source_worktree_clean`. Version tuple:
Cargo `0.1.0`, Python `0.2.0`, CLI `codefire foundation 1`, object `1`, commit-root writer `2`,
atom-hash writer `4`, envelope `codefire.command_result.v1`, capabilities `codefire.capabilities.v1`,
support `codefire.support.v1`, layout `1`, product SemVer `unassigned`. Not a SemVer release.

## 3. Closed candidate, current pointer, complete marker, 11-path checksums both ways

`$RELEASE_ROOT` mode 700, device 79. `$PUB`
`/tmp/codefire-m1-release.vT1Uzs/published/01a4de0ffcc7-90cb2895aadc` mode 500.
`current` is a symlink; `readlink current` is exactly `published/01a4de0ffcc7-90cb2895aadc`;
`readlink -f current` is the `$PUB` path. `.complete` exists, size 0, mode 444, SHA-256
`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
`SOURCE_REVISION` is 41 bytes, newline-terminated, value
`01a4de0ffcc775df08473e2088f639e69f268ec5`, SHA-256
`1e028055e356ceaf1bb0a7b74e57f4b386865656fa437fdb40ab78805950b228`.
No `staging.work`. Incomplete root `/tmp/codefire-m1-release.j6JHUw` is absent.
`$CURRENT` contains **exactly 13 regular files**, two directories (`artifacts`, `docs`),
zero extra symlinks.

Independently recomputed SHA-256 of every regular file:

| Path | SHA-256 |
|---|---|
| `.complete` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `RELEASE_MANIFEST.json` | `126721cd3b3d775f6587a17d36fdebe25bd62eaafb3e88d50af5edaa7c6d91f4` |
| `RIGHTS_DECISION.txt` | `3742080e65967f055cbdfdadbf8794e23d695f5bcefd66b29681b7286f6242b9` |
| `SHA256SUMS` | `b434d3188cd2c96f06e582c4e2cef4916797d41516d7799571490accea68c3c3` |
| `SOURCE_REVISION` | `1e028055e356ceaf1bb0a7b74e57f4b386865656fa437fdb40ab78805950b228` |
| `artifacts/codefire` | `efd124fd442addf1c7f385a73d0fbbe959e3455d609ab3d5eb09dc82fb54fa10` |
| `artifacts/codefire-rs` | `90cb2895aadc45b559a96207cdd5ae3b08485f9a7e5a07014db8cca8b5b90ce9` |
| `artifacts/install.sh` | `0bc5a6cb4a89bc003ce667e0acff79c0619a0c60db1c3b1be5ed0138da59752e` |
| `docs/04_cli_spec.md` | `b71428806556534f1d50fd0538e7680eded78a08db673d4b24c876146264b28e` |
| `docs/README.md` | `422174c3a419d846abce1844461e449c4cbb8178a0983aac2afc3911f355110e` |
| `docs/automation_interface.md` | `9c87a4c98e4d2b9c24dd35d4fe6a76feb79a6e6e95938c4ef713497eb034efe6` |
| `rust-build-inputs.json` | `7b6ddc5283a7a269a1cedf760b3972a62ef64d7dbbb7d0ce1770b5022ff7f2da` |
| `rust-build-provenance.json` | `7998bcf60a5de839f88a40d54f7fabc724da83c09bd34d7403eb97b8cc8d9798` |

`SHA256SUMS` has exactly 11 GNU two-space relative lines, lists neither itself nor `.complete`,
contains no absolute path and no `..`. `sha256sum -c SHA256SUMS` → 11/11 `OK`, exit 0.
Set-equality **both directions**: listed paths = (regular files under `$CURRENT`) minus
`{SHA256SUMS, .complete}`. No omission, no extra, no duplicate. Every listed digest matches
an independent rehash.

T0 and T1 candidate-tree listings are **byte-identical** (file digest
`a6cb35c43f0ba2d56e12fb750275dbe7043986578f7b775d151a5302711232c4`). The 13 `(digest, path)`
pairs equal the audit-now set. Listing *order* in T0 is path-sorted (`LC_ALL=C`); a
hash-then-path sort of the same lines is a different concatenation and is **not** a content
mismatch.

Forbidden candidate paths independently absent: `docs/development`, `release-and-format-roadmap`,
`subprojects`, `.codex`, `audit-latest`, wheel/sdist, `LICENSE`, `NOTICE`, `pyproject.toml`,
`setup.py`. Docs set is exactly `{04_cli_spec.md, README.md, automation_interface.md}`.
Artifacts set is exactly `{codefire, codefire-rs, install.sh}`.

Five candidate blobs compared with `git show 01a4de0:<path>` from this audit worktree
(no switch): `README.md`, `docs/04_cli_spec.md`, `docs/automation_interface.md`, `codefire`,
`install.sh` are byte-identical to the candidate.

`git diff --name-only 01a4de0 62c53f7` is only
`docs/development/ai/task-packets/milestone-release-operational-trust-run.md` and
`docs/development/milestones.md`. Product/Rust/Python/installer/public-doc bytes are identical
to `01a4de0`.

## 4. Provenance binding to `01a4de0` and Rust input manifest `3925fc03` — B1 bytes are not identity

Rust input manifest independently recomputed from **git objects at `01a4de0`**
(`git ls-tree` + `git show`, `sha256_json` over the 101-file list):

```text
path_count                 101
manifest_sha256            3925fc035690296fa5e132b630dab65a7fbd5ddb1a55f0fb1226e6efd295d97a
candidate inputs JSON      same digest, schema codefire.rust_build_inputs.v1, 101 repo-relative paths
file-set equality          exact
validate_source_inputs_against_head(rehearsal_worktree, 01a4de0)  []
```

Provenance payload on `$CURRENT` is byte-identical to
`$SRC/target/codefire-ai/rust-build-provenance.json` (SHA-256
`7998bcf60a5de839f88a40d54f7fabc724da83c09bd34d7403eb97b8cc8d9798`). Fields independently
asserted:

| Field | Value | Verdict |
|---|---|---|
| `schema` | `codefire.rust_build_provenance.v1` | PASS |
| `ok` | `true` (bool) | PASS |
| `base_revision` | `01a4de0ffcc775df08473e2088f639e69f268ec5` | PASS |
| `source_manifest_sha256` | `3925fc03…` | PASS |
| `source_file_count` | 101 | PASS |
| `binary_path` | `target/release/codefire-rs` | PASS |
| `binary_bytes` | 14590776 | PASS |
| `binary_sha256` | `90cb2895aadc45b559a96207cdd5ae3b08485f9a7e5a07014db8cca8b5b90ce9` | PASS (= candidate binary, = rehearsal `target/release/codefire-rs`) |
| `build_command` | `cargo build --release --locked -p codefire-cli --bin codefire-rs` | PASS (`--locked` present) |
| `cargo_version` / `rustc_version` | `cargo 1.97.0 (c980f4866 2026-06-30)` / `rustc 1.97.0 (2d8144b78 2026-07-07)` | recorded |

**B1 rejected as identity.** B1 binary SHA-256 is also `90cb2895…` (byte-identical, 14590776 B).
B1 provenance file SHA-256 is `1731b8636330841f20dcad9265710eacc3830767e8c693b34ad75b5907688dea`,
`base_revision=c78d5e0e5e151a1489744c21340716a439597e71`. Candidate provenance is a **different
file** (`7998bcf6…`) bound to `01a4de0`. Identical B1 binary bytes were **not** accepted as
release identity. Identity is `base_revision` + manifest `3925fc03…` +
`validate_source_inputs_against_head` against `01a4de0`.

`validate_provenance_payload` invoked from this audit against the rehearsal worktree with
`expected_revision=01a4de0` returned the single error `review head_revision is not the current
checked-out HEAD` because rehearsal HEAD is `9ce0581` and this audit was forbidden to switch
that worktree. Every other clause of that function passed (schema, ok, base_revision, source
manifest, file count, binary path/size/digest, build command, source-vs-head). Operator
evidence `verify-07-provenance-validate.txt` records `provenance-ok` at a transient detach to
`01a4de0` which this audit did not repeat. That HEAD-checkout clause is environment state, not
an artifact mismatch.

## 5. Fresh-operator independence and one-pass / no-private-correction

| Check | Independent observation |
|---|---|
| Operator family | Stage B packet and evidence identify Claude Opus 5; Stage A / node 2 were Grok 4.6; this auditor is Grok 4.6, fresh session, other worktree |
| Inputs | source worktree + `$RELEASE_ROOT` + the three packets; no other write set |
| Private correction channel | none in the handoff; none visible in evidence |
| Rebuild / CodeFire-against-repo / PATH export | none recorded; `install.sh` used `--prefix` and `--binary`; installer stderr `no active codefire found on PATH` |
| One pass | monotonic UTC 04:15:29 → 04:21:20; **one** install (04:19:37Z); **four** smokes once (04:20:05Z–04:20:06Z); **one** rollback (04:20:34Z) |
| `verify-05` then `verify-05b` | same-pass scanner correction, not a second install/smoke and not a Stage A whisper |

`verify-05-provenance-payload.txt` rejected on a crude substring scan (`registry` / `token` in
Rust **filenames** `command_registry.rs`, `opened_registry.rs`, `sealed_token_boundary.rs`; and
inputs key named `manifest_sha256` not `source_manifest_sha256`). `verify-05b` re-checked the
actual JSON keys and path-aware scan and accepted. Independently: inputs `manifest_sha256` is
`3925fc03…`; those filenames are source inputs, not publication. This is recorded operator
self-correction inside one pass. It is **not** a private-correction channel and **not** a
second Stage B run.

## 6. Installed artifact hashes (from evidence; prefix now empty by design)

This audit did not reinstall. Post-rollback `$PREFIX` has 0 entries; `bin/codefire` and
`bin/codefire-py` are absent **now**. Install-time records (`install-03-post.txt`) give:

| Installed path | Recorded SHA-256 | Equals still-present candidate |
|---|---|---|
| `$PREFIX/bin/codefire` (14590776 B, mode 755) | `90cb2895aadc45b559a96207cdd5ae3b08485f9a7e5a07014db8cca8b5b90ce9` | `artifacts/codefire-rs` |
| `$PREFIX/bin/codefire-py` (1341034 B, mode 755) | `efd124fd442addf1c7f385a73d0fbbe959e3455d609ab3d5eb09dc82fb54fa10` | `artifacts/codefire` |

Install argv (cwd `$CURRENT/artifacts`):
`./install.sh --prefix /tmp/codefire-m1-release.vT1Uzs/prefix --binary /tmp/codefire-m1-release.vT1Uzs/current/artifacts/codefire-rs`.
Both `--prefix` and `--binary` present, so unlocked `cargo build --release` and default
`/usr/local` were structurally unreachable. Exit 0.
stdout 154 B SHA-256 `20cc36768436b162376ab4e9c0861c284bc85fcc98e24a083ef9a4907fa70c53`
(independently rehashed from `install-02-stdout.txt`).
stderr 157 B SHA-256 `39ca910534197aa6703784f9adda5284ac54f3d356c775125290fa6954cbaeeb`
(independently rehashed; contains PATH-warning, not a PATH mutation).
`--completion` not passed. After install, `$CURRENT` still 11/11 OK. User install still absent.

## 7. All four smoke outputs (from retained stdout; not re-executed)

cwd `$RELEASE_ROOT/evidence/smoke` was empty before and is empty now (0 entries). All four
invocations used absolute `$PREFIX/bin/…` paths. This audit **did not** invoke those binaries.
Independently rehashed retained stdout/stderr:

| # | argv | exit | stdout B / SHA-256 | stderr |
|---|---|---:|---|---|
| 1 | `$PREFIX/bin/codefire --version` | 0 | 22 / `4a2fd935f1976a3e651b0dbac4a16907685c8cb8fc40b49d4cf5ae04664623bf` | 0 B `e3b0c442…` |
| 2 | `$PREFIX/bin/codefire --help` | 0 | 825 / `79e58bed9a6d4de3fb591ce1c9b0332300e87e2431c45c2681f547852c062960` | 0 B `e3b0c442…` |
| 3 | `$PREFIX/bin/codefire capabilities --json` | 0 | 21970 / `f3f18424d8aa5bcd4a22a8ced1a51b967892f25901dc23bdc42ec2c5712d9d65` | 0 B `e3b0c442…` |
| 4 | `$PREFIX/bin/codefire-py --help` | 0 | 554 / `329f392e1cd0d10e1759e750df3de344c8238943f6fcc1935d5b750c4be59ab6` | 0 B `e3b0c442…` |

Identity, parsed from those files:

| Command | Expected | Observed | Verdict |
|---|---|---|---|
| Rust `--version` | exactly `codefire foundation 1` | `codefire foundation 1\n` | PASS |
| Rust `--help` | contains `usage: codefire <command> [args]` | present | PASS |
| Rust `capabilities --json` | envelope `codefire.command_result.v1`; `data.schema=codefire.capabilities.v1`; `data.type=codefire_command_capabilities`; `data.support.profile=codefire.support.v1` | all four match | PASS |
| Python `--help` | contains `usage: codefire` | `usage: codefire [-h] {…}` | PASS |

Capabilities payload has no `public_beta`, `semver`, `signed_release`, `pypi`, `crates.io`, or
licence assertion. Token `upload` occurs as the product `upload` command in help/capabilities,
not as an external publication action. Token `Public` in the payload is `public_object_id`.

## 8. Rollback to empty prior; candidate / source / main / canonical / user-install preservation

Rollback recorded command: `rm -rf "$PREFIX"; mkdir -p "$PREFIX"; cp -a "$PRIOR/." "$PREFIX/"`.
Independently **now**:

| Subject | Now | Verdict |
|---|---|---|
| `$PREFIX` entries | 0 | PASS |
| `$PREFIX/bin/codefire` | absent | PASS |
| `$PREFIX/bin/codefire-py` | absent | PASS |
| `$PRIOR` entries | 0 | PASS |
| `$SMOKE_CWD` entries | 0 | PASS |
| `$CURRENT` / `$PUB` | intact; 13 files; 11/11 checksums; `.complete` present | PASS |
| empty-tree digest | `e3b0c442…` (prefix and prior) | PASS |

`$PREFIX` directory mtime equals `$PRIOR` / `$SMOKE_CWD` mtime (Stage A empty-dir timestamp).
That is consistent with a timestamp-preserving restore of the empty prior, not leftover
installed binaries.

**PRESERVE_DIGEST** (operator-stated, independently recomputed **now**):
`sha256(concat of "<sha256(content)>  <relpath>\n"` over regular non-symlink files, relpaths
sorted `LC_ALL=C`)`.

| Tree | T0 | T1 | Audit-now | files / bytes |
|---|---|---|---|---|
| `/workspace/project/subprojects/.codefire` | `a941db6fe3e9ce5f80c1c51ddd5ca26f424cc9c8bd84cd264a4e2d2d9449a4fa` | same | **same** | 2905 / 114660974 |
| `/workspace/algorithm-evolution-agent-lab` | `4259b2d8862df08c2868ecbc188653a155c8039ea2267237bff78a7328770886` | same | **same** | 1470 / 12232765 |

Unambiguous fields, independently recomputed now, equal node-2 / Stage A / Stage B T0:

| Subject | Value |
|---|---|
| Dirty main HEAD | `f1644122134d479333cf422fdb1d48b8fa92dd8e` |
| Dirty main porcelain | 248 lines; SHA-256 `1385b6249f6455510bc449aed054d5ce425494a29edb9a4c21d65165cb6b1bcf` |
| Canonical `repo.json` | `bf6d978b0eece0f185c6995630339326f56513a0c8645af3e7e984ad69a7f054`; `repository_id=repo_8a9e9ea836af` |
| Canonical `branches/main.json` | `bfd52e3525f6a6faf0db4f3d2d1d9d7fa3fa0d400f6561c3c5b8227e79e989f8`; head `CF-COMMIT-2792b54ad730e6d644668c38`; `state=open-clean` |
| Historical copy `.codefire` | `repository_id=repo_f9f434abf020`; head `CF-COMMIT-5d3f0f33e5ce16ab34d72ac1`; `state=open-clean` |
| User install | no `codefire` on PATH; `/home/devuser/.local/bin` absent; `/usr/local/bin/codefire` absent (`/usr/local/bin` exists) |
| Source worktree | branch `grok-agent/m1-release-rehearsal-20260822` @ `9ce0581`; porcelain empty; gitignored `target/release/codefire-rs` `90cb2895…` and provenance `7998bcf6…` still present |
| Candidate T0 vs T1 vs now | 13-file digest set identical |

`T0-T1-diff.txt` reads `RESULT: zero drift`. Audit-now matches T0/T1. This audit wrote nothing
to those roots.

Stage B evidence: 31 files under `evidence/stage-b/` (30 listed + `EVIDENCE-MANIFEST.sha256`).
Manifest digest `3f876e47f5117fae8ccee9cd5167790f031edba148dc0fe05ca595ba5836a949`. All 30
listed hashes independently verified. `$EVIDENCE/smoke` has no files.

## 9. Absence of network, registry, tag, credential, external publication, public-beta/SemVer/signing claim, PATH or canonical-repository mutation

| Prohibited thing | Verdict | Independent basis |
|---|---|---|
| GitHub/PyPI/crates.io/registry upload | **absent** | no such command in Stage B evidence; no wheel/sdist in candidate; P1 `publish=false` not required because no registry path was admitted |
| `git tag` / release tag | **absent** | `git tag` empty at audited HEAD |
| Credential / token / URL fetch | **absent** | no credential files in `$RELEASE_ROOT`; evidence-network needle scan hits only the **negative** assertions `pypi` / `crates.io` in `smoke-02-assertions.txt` |
| External publication | **absent** | `external_publication=false`; local `$CURRENT` only; no push evidence |
| Public beta / SemVer / signed-release claim | **absent** | manifest booleans false; rights sentence forbids them; capabilities payload has none of those claims |
| PATH mutation / user install | **absent** | PATH never gained `codefire`; `/usr/local/bin/codefire` absent; `~/.local/bin` absent |
| Canonical Algorithm / dirty-main mutation | **absent** | PRESERVE_DIGEST T0=T1=now; porcelain hash unchanged; this audit issued no CodeFire command |
| Full-tree archive / `docs/development` / `.codex` / `audit-latest` in candidate | **absent** | §3 |
| Treating B1 bytes as `01a4de0` identity | **rejected** | §4 |
| Second Stage B pass / rebuild / re-smoke | **absent** | §5; this audit also did not rebuild, reinstall, smoke, switch a worktree, or invoke CodeFire |

## 10. Operator park finding — node-2 legacy directory digests omitted their algorithm

**Finding (reproduced independently):** node 2 §4 recorded two directory digests without an
algorithm:

- canonical store `12d84568ee63fc69b54536346da7a2072290edcc952726330be97ff03fdca4d1` over 2905 files / 114660974 bytes
- historical copy `b41e7e884e3d18a72e81d29c2d93088fcb3ffdafc537fd6e6687ec1155187495` over 1470 files / 12232765 bytes

This auditor tried concat-of-raw-digests, GNU `sha256sum` relative / absolute / `./`-prefixed
streams, content-only concatenation, and hashes-only newline streams over the canonical store,
the historical lab root, and historical `.codefire`. **None reproduced either digest.** The
operator's seven-formulation failure is confirmed.

Node 2 also labeled the historical path as `/workspace/algorithm-evolution-agent-lab/.codefire`
while recording 1470 / 12232765, which independently matches the **lab root**, not `.codefire`
(1185 files / 11564679 bytes, PRESERVE_DIGEST `ad0ccd54…`). That is a classifier recording
defect (path vs count), not live drift.

**Adjudication: `park`, not `block-m1`.**

M1 requires before/after hashes that prove the operator restored the declared prior and did not
mutate non-admitted repositories. Those hashes exist, have a **stated algorithm**
(`PRESERVE_DIGEST`), and this auditor independently recomputed them at T0 = T1 = audit-now,
together with every unambiguous field (dirty-main HEAD and porcelain SHA-256, `repo.json`,
`branches/main.json`, both `repository_id`s, both head ids and `open-clean` states, both file
counts, both byte totals, user-install absence). Non-mutation is established by that evidence.

The unreproducible node-2 values are a documentation gap in the classifier T0 record. They are
not a failed preservation proof and not a product defect. Promoting them into required M1 work
would convert a `park` finding into a slice the exit already satisfied by the operator's
reproducible pair. A later packet that records a directory digest must also record its
algorithm, or use `PRESERVE_DIGEST` as defined in Stage B B.10.

## 11. Findings

### `block-m1` (0)

None. Rights are explicitly internal-only. The closed candidate, 11-path checksums both ways,
current pointer, complete marker, fail-closed manifest fields, provenance binding to `01a4de0`
and manifest `3925fc03…` (without treating B1 bytes as identity), fresh-operator independence,
installed hashes, four smokes, rollback to empty prior, preservation, absence checks, and
one-pass scope all independently recompute. The parked digest-algorithm gap is not a blocker.

### `park` (carried; not now-work)

1. **B.12 / this §10** — node-2 legacy directory digests omit their algorithm and remain
   unreproducible; path/count labelling of the historical copy is sloppy. Preservation is
   nonetheless proven. Future records must state the algorithm.
2. **Rights P1–P7** — no `publish=false`; MIT vs UNLICENSED vs pyproject silence; 75 crates
   without licence inventory; stale schema-count prose in strategy docs; missing
   SECURITY/SUPPORT/GOVERNANCE; no AI-authorship policy; no in-tree toolchain pin. P1 does
   **not** escalate: no admitted step could reach a registry.
3. **Topology parks** — signing/SBOM/registries/policies; installer provenance binding;
   SemVer / `--version` source SHA; pip as default install.
4. **Operational, not kernel, containment** of `$RELEASE_ROOT` (shared uid). Evidenced
   absence of external publication; a stronger isolation claim is unavailable.
5. **`validate_provenance_payload` HEAD-checkout clause** was not re-executed here because
   switching the rehearsal worktree was forbidden. Artifact binding is independently proven
   via git objects at `01a4de0` plus on-disk binary/provenance identity.

### `drop`

Public beta / SemVer / signed-release labels; registry upload; licence selection; wheel/sdist;
full-tree archive; treating B1 as `01a4de0`; dirty-main as source; PATH/`/usr/local` install;
M2–M5 work; a second Stage B pass; rebuilding for this audit.

## 12. Authorization for node 7

**Authorized:**

1. Treat this node-6 audit as terminal **`ACCEPT`** with **`block-m1`: none**.
2. Exit M1 `MS-release-operational-trust` on this ACCEPT, then apply the board shift rule:
   record exit evidence in `history.md`; rename slots (old M2 becomes M1, and so on); name a
   new M5 from `strategy/modernization-roadmap.md` without turning that roadmap into a task
   board; refuse any in-flight parked work that tried to start early.
3. Record the rehearsed artifact as **internal-only**: candidate id
   `01a4de0ffcc7-90cb2895aadc`, `$RELEASE_ROOT=/tmp/codefire-m1-release.vT1Uzs`, source
   `01a4de0ffcc775df08473e2088f639e69f268ec5`, rust input manifest `3925fc03…`, 11/11
   checksums both ways, 4/4 smoke, rollback to empty prior, PRESERVE_DIGEST T0=T1=audit-now.
4. Carry the eight public-claim prohibitions unchanged. Missing licence/copyright-holder
   remain public-distribution blockers.

**Refused:**

1. Any public GitHub/PyPI/crates.io release, tag, credential, network publication, public
   beta, SemVer, or signed-release claim.
2. Choosing or fixing a licence, adding `publish=false` (P1 not escalated), SBOM, signing,
   or Stage 4 policy work.
3. Rebuilding, reinstalling, re-smoking, or starting a second rehearsal.
4. Starting M2 `MS-product-value-proof` or any parked issue (CFR-168..173, CFB-133/136, H1/H2,
   P1–P7, B.12).
5. Mutating canonical Algorithm, dirty main `/workspace/project`, user PATH install, or
   deleting the preserved release/source/evidence roots as part of "closing" M1.
6. Any product, source, packet, or issue edit as part of closing M1 beyond the
   board/history/current-state docs that node 7 owns.

Exact node-7 handoff: orchestrator may close M1 and shift slots on this ACCEPT. Write set is
board/history/current-state docs only. Preserve `/tmp/codefire-m1-release.vT1Uzs` and
`/workspace/worktrees/m1-release-rehearsal-20260822` through that record. Do not infer
publication rights from this audit.

## 13. Validation run by this audit

```bash
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_ai_acceleration_docs.py
git diff --check
```

Results recorded with the commit. No build, no product test, no CodeFire command of any kind,
no reinstall, no smoke, no worktree switch, no network publication, and no later-slot gate.
Write set is this file only.

## 14. Verdict

**ACCEPT.**

The isolated internal-only rehearsal at `$RELEASE_ROOT=/tmp/codefire-m1-release.vT1Uzs` is
closed, locally published, independently verified, installed into an isolated prefix, smoked,
and rolled back to the empty prior. Source binding is `01a4de0` plus rust input manifest
`3925fc035690296fa5e132b630dab65a7fbd5ddb1a55f0fb1226e6efd295d97a`. Identical B1 binary bytes
are not identity. Rights are internal-only; nothing is publishable. The node-2 digest-algorithm
gap is **`park`**, not `block-m1`.

`block-m1`: **none**.

Node 7 may exit M1 and shift slots. It must not start parked work, publish, or inherit a
public-release claim.
