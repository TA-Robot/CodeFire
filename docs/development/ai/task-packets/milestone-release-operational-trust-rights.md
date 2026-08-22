# AI Task Packet

Status: completed

Packet ID: MS-release-operational-trust-rights

Schema version: human-v1

Risk score: 6

Reasoning tier: high

## Objective

Classify the exact rights, license, source-identity, and version-axis facts for the smallest bounded
M1 release rehearsal. Produce a decision table that distinguishes what may be locally rehearsed,
what could later be publicly distributed, and what remains blocked. Do not infer permission from
repository access, package metadata, historical version labels, or a successful build.

## Scope

- read-only inventory of current rights, license, package, source-identity, and version-axis evidence;
- a per-artifact internal-rehearsal / later-publication / stop decision table;
- classification and handoff in this packet only.

## Authoritative Inputs

- `AGENTS.md` and `docs/development/milestones.md`;
- `README.md`, `Cargo.toml`, `Cargo.lock`, `pyproject.toml`, `setup.py`, `install.sh`;
- all checked-in license, notice, copyright, contribution, security, governance, maintainer, support,
  release, package, and artifact-identity documents that actually exist;
- `docs/development/strategy/release-and-format-roadmap.md` and `modernization-roadmap.md`;
- current Git source identity and the retained source-bound runner/provenance records, read-only.

Absence is evidence. Record missing files and undecidable ownership explicitly; do not create or
choose license text.

## Questions

1. Which exact Rust binary, Python package/fallback, public schemas/docs, source archive, checksum,
   provenance, and generated metadata are candidates for the bounded rehearsal?
2. For each candidate, what checked-in evidence identifies author/owner, applicable license,
   third-party obligations, generated-source relationship, and source revision?
3. Which candidates are safe only for an isolated internal rehearsal, which are supportable for later
   public distribution, and which must stop pending a human/legal decision?
4. What are the independent version axes today: product/package version, CLI-reported version,
   schema/object format versions, compatibility profiles, and historical internal labels?
5. What exact statement prevents the rehearsal from being described as public beta, SemVer release,
   signed release, or legal clearance?
6. Is any product/source change the smallest unavoidable M1 blocker, or can the rehearsal proceed
   truthfully with an internal-only/stop decision and a manifest of unresolved rights?

## Exact Write Set

- `docs/development/ai/task-packets/milestone-release-operational-trust-rights.md` only.

## Validation

```bash
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_ai_acceleration_docs.py
git diff --check
```

## Non-Goals

- legal advice, license selection, copyright transfer, contributor outreach, or external publication;
- editing package metadata, source, tests, README, LICENSE/NOTICE, SECURITY, governance, or release docs;
- building, signing, installing, uploading, or deleting artifacts;
- completing the whole Stage 4 roadmap or starting M2–M5;
- filing a new issue unless one fact is a genuine `block-m1` that cannot be represented in the packet.

## Handoff

Return exact evidence paths/lines, the per-artifact decision table, version-axis matrix, missing-rights
list, public-claim prohibitions, `block-m1` / `park` / `drop` classification, the smallest admissible
rehearsal scope, validation results, and commit SHA. Commit only this packet.

## Result

Status **completed**. Verdict: **zero `block-m1`**, seven `park`, four `drop`.

The rights lane needs **no product or source change** to satisfy M1. M1 exit bullet 1 requires an
*explicit recorded decision*, and the honest decision is available today from checked-in evidence:

> **Every candidate artifact is admissible for an isolated internal rehearsal only. No artifact is
> admissible for public distribution. The outbound licence is undeterminable from this tree, and that
> missing right is recorded as a public-distribution blocker, not inferred away.**

That decision is recorded here, so the bounded rehearsal may proceed truthfully with an internal-only
verdict plus the unresolved-rights manifest in section 6. No licence was chosen, no permission was
inferred, no legal advice is given, and nothing outside this packet was written.

### 0. Fixed source identity and read-only method

```text
integration HEAD (exact)   d803527599808cb2ac31acf74e3d0b988208f2fc  ("docs: close adoption M1 and shift to release trust")
packet worktree branch     claude-agent/m1-release-rights-20260822   (git status --porcelain: empty before this packet)
git author = committer     t.asakura <asakuratakaya@gmail.com>       336 / 336 commits, one identity only
HEAD signature status      git log -1 --format=%G? -> N              (unsigned)
git tags                   git tag -> empty                          (no release tag anywhere in the repository)
declared remote            origin git@github.com:TA-Robot/CodeFire.git  (= Cargo.toml:13 repository URL)
tracked file count         git ls-files | wc -l -> 920
```

Method: `git ls-files`, `git log`, `git diff`, `grep`, `sha256sum`, and file reads only. No build, no
install, no publish, no network, no `cargo`, no `codefire` invocation, no mutation of any tree outside
this packet's own file. `cargo` and `rustc` are absent from this environment, so no build could have
run even accidentally.

**Source-binding facts established for node 3** (read-only `git diff` at this HEAD):

```text
git diff --quiet c78d5e0e HEAD -- Cargo.toml Cargo.lock crates .cargo rust-toolchain rust-toolchain.toml   -> exit 0
git diff --quiet c78d5e0e HEAD -- schemas/public                                                            -> exit 0
git diff --stat c78d5e0e HEAD -- pyproject.toml setup.py install.sh codefire README.md schemas adr docs
        -> only docs/development/** changed (15 files); every product-facing path is byte-identical
```

So the whole product-facing surface — Rust inputs, Python fallback, packaging, installer, README, all
`schemas/`, all `adr/`, all 16 public `docs/` files — is **byte-identical between `c78d5e0e` and
`d803527`**. Only internal `docs/development/**` moved. The retained runner below is therefore
source-bound to this HEAD for the Rust input set; whether to reuse it is node 2's decision, not this
packet's.

**Retained source-bound runner and provenance (inspected, not executed):**

```text
binary        /workspace/worktrees/ms-adoption-experiment-20260822/target/release/codefire-rs
              sha256 90cb2895aadc45b559a96207cdd5ae3b08485f9a7e5a07014db8cca8b5b90ce9   14590776 bytes, nlink 3
provenance    /workspace/worktrees/ms-adoption-experiment-20260822/target/codefire-ai/rust-build-provenance.json
              sha256 1731b8636330841f20dcad9265710eacc3830767e8c693b34ad75b5907688dea   680 bytes
              schema codefire.rust_build_provenance.v1, ok=true, base_revision c78d5e0e…,
              build_command "cargo build --release --locked -p codefire-cli --bin codefire-rs",
              cargo 1.97.0, rustc 1.97.0, source_file_count 101,
              source_manifest_sha256 3925fc035690296fa5e132b630dab65a7fbd5ddb1a55f0fb1226e6efd295d97a
```

**Rights-critical property of that record:** `codefire.rust_build_provenance.v1` has **thirteen keys and
not one rights key**. It binds a binary to a source revision and a toolchain. It carries no copyright
holder, no licence expression, no author, no third-party dependency licence set, and no SBOM
(`docs/development/tools/build_source_bound_rust.py`, `BUILD_COMMAND` / record construction). Source
provenance and rights provenance are separate axes here, and only the first one exists.

**`/workspace/project` is not a rehearsal source.** `git -C /workspace/project status --porcelain | wc -l`
-> **248** modified paths at HEAD `f164412`. The canonical main worktree is intentionally dirty and far
behind; it must never be the artifact source, and this packet did not touch it.

**Non-repository observation, explicitly not evidence:** the operating session identity differs in
domain from the commit identity. That is a harness fact, not a checked-in fact, and it is listed only
because individual-versus-organization ownership is exactly one of the unresolved questions in
section 6. It resolves nothing.

### 1. Q1 — exact candidate artifacts for the bounded rehearsal

Candidates enumerated from `install.sh`, the manifests, and `git ls-files` at this HEAD. Hashes are of
the checked-in bytes at `d803527`.

| # | Candidate artifact | Exact identity at this HEAD | Kind |
|---|---|---|---|
| A1 | Rust CLI binary `codefire-rs` | built from `crates/**` (99 tracked files) + `Cargo.toml` + `Cargo.lock`; retained build sha256 `90cb2895…` | compiled product |
| A2 | Python fallback script `codefire` | sha256 `efd124fd442addf1c7f385a73d0fbbe959e3455d609ab3d5eb09dc82fb54fa10`, 1341034 bytes, stdlib-only | interpreted product |
| A3 | Installer `install.sh` | sha256 `0bc5a6cb4a89bc003ce667e0acff79c0619a0c60db1c3b1be5ed0138da59752e` | packaging script |
| A4 | Python sdist/wheel `codefire-mvp` 0.2.0 | `pyproject.toml` sha256 `851b8dff…`, `setup.py` sha256 `3fb6e1c6…` | distributable package |
| A5 | Public JSON schema family | `schemas/public/` — **7** files, all 7 registered in `crates/codefire-cli/src/compatibility.rs:125-131` | public interface contract |
| A6 | Conformance corpus | `schemas/conformance/manifest-v1.json` (`codefire_conformance_manifest` v1) + 5 fixtures + `minimal_consumer.py` | public contract fixtures |
| A7 | Other checked schemas/examples | remaining `schemas/` files (26 total incl. A5/A6), `schemas/repository_layout.txt` | public reference |
| A8 | Public product docs | 16 files: `docs/00_*`–`docs/11_*`, `docs/13_*`, `docs/automation_interface.md`, `docs/runbook.md`, `docs/tickets/task-ticket.template.md` | documentation |
| A9 | ADR set | `adr/ADR-001..ADR-020` (20 files) | documentation |
| A10 | `README.md` | sha256 `422174c3a419d846abce1844461e449c4cbb8178a0983aac2afc3911f355110e` | documentation |
| A11 | `Cargo.lock` | sha256 `26b989f38cf05a1a5fde5770afba3c006e589f8cfa3b25456a8487db65c20059`; 79 `[[package]]` entries = 4 local + **75 third-party** | generated dependency identity |
| A12 | Full-tree source archive of `d803527` | all 920 tracked files, i.e. A1–A11 **plus** A13–A16 | source distribution |
| A13 | Dogfood subproject | `subprojects/algorithm-evolution-agent-lab/**`, 184 tracked files, a separate research project per `README.md:20-22` | non-product project |
| A14 | Internal development corpus | `docs/development/**`, 525 tracked files (ledgers, audits, packets, AI routing) | internal management |
| A15 | Agent runtime config | `.codex/**`, 4 files | internal tooling config |
| A16 | Local audit record | `docs/development/ai/audit-latest.md`, **gitignored** (`.gitignore`), mode 0600, untracked | local-only evidence |
| A17 | Checksum / provenance / manifest records | `rust-build-provenance.json` (`codefire.rust_build_provenance.v1`) and any rehearsal checksum set | generated metadata |
| A18 | Generated shell completions | produced at install time by `install.sh:206-211` via `codefire-rs completion bash|zsh` | generated, derived from A1 |
| A19 | CLI-reported identity strings | `codefire foundation 1` (`crates/codefire-cli/src/main.rs:1396`), `capabilities --json` payload | generated metadata |

### 2. Q2 — checked-in rights evidence per candidate

**Every licence-relevant statement that exists in the tree.** This list is exhaustive; it was produced
by grepping the whole repository for licence, notice, copyright, SPDX, and rights markers.

| Evidence | Exact location | Content |
|---|---|---|
| E1 | `Cargo.toml:12` | `license = "MIT"` (workspace-inherited by all four crates via `license.workspace = true`) |
| E2 | `setup.py:11` | `license="UNLICENSED"` |
| E3 | `pyproject.toml:5-15` | **no** `license` field, **no** licence classifier; `authors = [{name = "CodeFire MVP"}]` (`pyproject.toml:11`) |
| E4 | `setup.py:10` | `author="CodeFire MVP"` |
| E5 | `Cargo.toml:13` | `repository = "https://github.com/TA-Robot/CodeFire"` |
| E6 | `docs/development/strategy/release-and-format-roadmap.md:29` | already records the contradiction: "Cargo workspace `MIT`; `setup.py` says `UNLICENSED`; `pyproject.toml` has no license field; no root `LICENSE` file — Rights and distribution terms are unresolved and inconsistent." |
| E7 | `docs/development/exploration/phase-3/02-oss-license-governance-and-maintainers.md:20-28` | observed baseline (dated `2026-07-11`), status `exploration`, with an explicit legal disclaimer at line 9 |
| E8 | `docs/development/exploration/phase-3/07-premortem-kill-and-pivot-criteria.md:258-269` | "Failure 11: License and IP Ambiguity"; leading indicators include "no license text or copyright owner" and "dependency license inventory is missing" |
| E9 | `docs/development/exploration/phase-3/README.md:62` | "Nothing in this directory is legal advice." |

`Cargo.toml:12` is the **only** affirmative licence grant expression in the entire tree, and it is
package metadata with no corresponding licence text. `E1` and `E2` are mutually contradictory: MIT
grants rights, `UNLICENSED` grants none.

**Per-candidate evidence and absence.** "Owner" = a checked-in statement naming a copyright holder.

| # | Owner identified? | Licence evidence | Third-party obligations | Generated-source relationship | Source revision binding |
|---|---|---|---|---|---|
| A1 | **no** — no copyright holder anywhere | E1 `MIT` metadata only; no LICENSE text | **75 third-party crates statically linked**; zero in-tree licence inventory, no `deny.toml`, no `about.toml`, no NOTICE | compiled from `crates/**` + `Cargo.lock` | **yes** — provenance `base_revision c78d5e0e`, byte-identical to HEAD |
| A2 | **no**; `"CodeFire MVP"` is not a legal person | **contradictory**: E2 `UNLICENSED` vs E1 `MIT`; no header licence in the file | none (Python stdlib only, per its own docstring) | hand-written source | **yes** — tracked bytes, sha256 `efd124fd…` |
| A3 | **no** | none at all | none | hand-written source | **yes** — tracked bytes |
| A4 | **no** | **worst case**: E2 says `UNLICENSED`, E3 declares nothing; the wheel would carry "no rights granted" | bundles A2 only | setuptools-generated from E2/E3 | derivable, not recorded |
| A5 | **no** | none; E1 does not reach `schemas/` | none | hand-written | **yes** — `git diff --quiet c78d5e0e HEAD -- schemas/public` exits 0 |
| A6 | **no** | none | none; `minimal_consumer.py` is stdlib-only | fixture digests are self-recorded (`sha256`/`canonical_sha256` per vector) | **yes** — tracked bytes |
| A7 | **no** | none | none | hand-written | **yes** |
| A8 | **no** | none; no docs reuse/licence policy exists | none | hand-written | **yes** |
| A9 | **no** | none | none | hand-written | **yes** |
| A10 | **no**; `README.md` has no licence or copyright section | none | none | hand-written | **yes** — sha256 `422174c3…` |
| A11 | n/a | n/a | **this file *is* the third-party obligation index**: 75 external crate identities and their checksums, with no corresponding licence record | cargo-generated | **yes** — tracked bytes |
| A12 | **no** | none for the tree as a whole | inherits A11 | `git archive` of tracked files | **yes** by construction |
| A13 | **no**; separate project, own `AGENTS.md`, own `.gitignore`, no licence file | none | unknown — not inventoried | hand-written research project | **yes** |
| A14 | **no** | none | none | hand-written | **yes** |
| A15 | **no** | none | none | hand-written | **yes** |
| A16 | **no** | none | none | agent-generated | untracked; gitignored |
| A17 | n/a | n/a | none | tool-generated; schema has **no rights fields** | **yes** — that is its entire purpose |
| A18 | **no** | inherits A1's unresolved status as binary output | inherits A1 | generated by A1 at install time | via A1 |
| A19 | n/a | n/a | none | generated by A1 | via A1 |

**Authorship texture that no checked-in document addresses.** All 336 commits carry one identity, but
`docs/development/ai/README.md:3` states the repository is developed by an "external AI-assisted
development system", and `docs/development/ai/provider-routing-policy.md:11-17` names Codex
`gpt-5.6-sol`, `claude-opus-5`, and `grok-4.6` as the actual producers of implementation, review, and
audit work. No contributor, authorship, DCO, CLA, or AI-attribution policy file exists. This is a
recorded gap, not an assessment of its consequences.

### 3. Q3 — per-artifact decision table

Three columns, exactly as the packet requires. **Internal rehearsal** = may exist inside an isolated
local release root and isolated install prefix, never leaving this machine. **Later publication** =
whether the artifact could be *supported* for public distribution once a rights decision exists.
**Stop** = must not proceed on this axis pending a human/legal decision.

| # | Artifact | Internal rehearsal | Later publication | Verdict |
|---|---|---|---|---|
| A1 | Rust binary `codefire-rs` | **admit** | **stop** — no owner, no licence text, 75 uninventoried third-party crates | rehearse internally; publication blocked |
| A2 | Python fallback `codefire` | **admit** | **stop** — E1/E2 directly contradict | rehearse internally; publication blocked |
| A3 | `install.sh` | **admit** | **stop** — no licence reaches it | rehearse internally; publication blocked |
| A4 | Wheel/sdist `codefire-mvp` 0.2.0 | **exclude** | **stop** | **drop from M1** — see D1 |
| A5 | `schemas/public/` (7) | **admit** | **stop today**; strongest later candidate (single author, no third-party content, already a versioned contract) | rehearse internally; publication blocked |
| A6 | Conformance corpus | **admit** | **stop today**; second-strongest later candidate | rehearse internally; publication blocked |
| A7 | Other `schemas/` | **admit** | **stop** | rehearse internally; publication blocked |
| A8 | 16 public docs | **admit** (recommend a bounded subset) | **stop** — no docs reuse policy exists | rehearse internally; publication blocked |
| A9 | `adr/` (20) | **admit** | **stop** | rehearse internally; publication blocked |
| A10 | `README.md` | **admit** | **stop** — and it is the file a consumer would look in for a licence | rehearse internally; publication blocked |
| A11 | `Cargo.lock` | **admit** as build input and obligation index | **stop** — publishing it without a licence inventory publishes the obligation without the record | rehearse internally; publication blocked |
| A12 | Full-tree source archive | **stop** | **stop** | **stop** — see the scope hazard below |
| A13 | `subprojects/**` | **stop** | **stop** | **stop** — a distinct project; not CodeFire product rights |
| A14 | `docs/development/**` | **exclude from the artifact set** | **stop** | **drop from M1** — see D2 |
| A15 | `.codex/**` | **exclude** | **stop** | **drop from M1** — see D3 |
| A16 | `audit-latest.md` | **stop** — gitignored, 0600, local-only | **stop** | **stop** — must not enter any artifact set or manifest |
| A17 | Provenance/checksum records | **admit** | **stop** as a rights statement — it makes none | admit as source evidence only |
| A18 | Generated completions | **admit** | **stop** — inherits A1 | rehearse internally; publication blocked |
| A19 | CLI identity strings | **admit** | n/a | admit; see the version-axis prohibitions |

**The A12 scope hazard is the sharpest finding on this axis.** A naive `git archive HEAD` produces a
"CodeFire source archive" in which **709 of 920 files are not CodeFire product**: 184 files of a
separate research project (A13), 525 files of internal development management including AI provider
routing, agent configs, issue ledgers, and audit records (A14), and 4 agent runtime configs (A15).
Any archive-shaped artifact must therefore be an explicitly scoped subset, and the full tree is a
**stop** on both the rights axis and the scope axis. This is independent of licence choice and does
not go away when a licence is selected.

### 4. Q4 — independent version axes observed at `d803527`

`release-and-format-roadmap.md:33-45` defines seven axes to keep separate. Observed values today:

| Axis | Observed value at this HEAD | Exact evidence | Notes |
|---|---|---|---|
| Product SemVer | **does not exist** | `git tag` empty; no product version declared anywhere | there is no product version to increment or claim |
| Cargo package version | `0.1.0` in all four crates | `crates/{codefire-cli,codefire-core,codefire-store,codefire-util}/Cargo.toml:3` | unrelated to the retired v0.6–v1.0 labels |
| Python package version | `0.2.0` | `pyproject.toml:8`, `setup.py:6` | the two agree; only their licence fields disagree |
| Rust CLI reported identity | `codefire foundation 1` | `crates/codefire-cli/src/main.rs:1396` printing `codefire_core::VERSION` (`crates/codefire-core/src/lib.rs:30`, `u32 = 1`) | a `u32`, not SemVer; `--version` and `version` are the same branch |
| Python fallback internal version | `VERSION = 1` | `codefire:63` | independent constant; the two implementations agree by coincidence of value, not by a shared contract |
| Object/record version | `1` | `codefire-core` `VERSION`, persisted `"version": 1` records | domain serialization axis |
| Commit root schema | writes `2`; absence readable as legacy `1` | `crates/codefire-store/src/lib.rs:9865`, `crates/codefire-cli/src/main.rs:5080,7349` (`"root_schema_version": 2`) | `known-limitations.md:210` records that the Python fallback is not in this writer claim |
| Atom hash schema | writes `4`; `1`–`4` readable | `crates/codefire-core/src/lib.rs:31` (`ATOM_HASH_SCHEMA_VERSION: u32 = 4`) | identity/extractor axis |
| Automation envelope | `codefire.command_result.v1` on every declared JSON surface | 21 occurrences across `crates/`; `schemas/public/command-result-v1.schema.json` | the Python fallback has no `--json` envelope surface |
| Support / capability profile | `codefire.support.v1`; remote branch & MR version `1`; semantic profile `sealed-snapshot-v1` | `crates/codefire-cli/src/compatibility.rs`, `known-limitations.md:210` | not product SemVer |
| Registered public schema IDs | **7** | `crates/codefire-cli/src/compatibility.rs:125-131`; 7 files in `schemas/public/` | `release-and-format-roadmap.md:28` still says "five checked public schema IDs" — stale, see P4 |
| Git release identity | **no tags**, 336 commits, HEAD unsigned (`%G? = N`) | `git tag`, `git rev-list --count HEAD`, `git log -1 --format=%G?` | no repository-tag evidence for any release line |
| Toolchain pin | **absent** | no `rust-toolchain`, no `rust-toolchain.toml`, no `.cargo/` | reproducibility rests on the provenance record's `cargo 1.97.0` / `rustc 1.97.0`, not on an in-tree pin |
| Historical labels v0.6–v1.0 | retired | `release-and-format-roadmap.md:1-11,101-113` | `AGENTS.md:61` forbids using them as release-completion evidence |

**Axis-collision facts a release note must state.** There are at least three mutually independent
numbers that a reader will call "the version": Cargo `0.1.0`, Python `0.2.0`, and CLI-reported `1`.
None is a product SemVer, and no checked-in mapping reconciles them. The rehearsal must therefore
report the whole tuple plus the source revision, never a single number.

### 5. Q5 — exact statements that prevent a public claim

These are the prohibitions the rehearsal record must carry verbatim. Each is grounded in the evidence
above and in `milestones.md:70-71` and `modernization-roadmap.md:110-114`.

1. **Not a public beta.** No public beta, preview, alpha, RC, or "early access" label may be applied.
   Grounds: `Cargo.toml:12` MIT and `setup.py:11` UNLICENSED are contradictory, no LICENSE text
   exists, and release identity is unresolved. `modernization-roadmap.md:114` states the Stage 4 exit
   condition directly: "no public beta label while cargo/source gates or release identity remain
   unresolved."
2. **Not a SemVer release.** No `vX.Y.Z` may be claimed. There is no product SemVer axis
   (section 4), no tag, and three unreconciled version numbers. `0.1.0`, `0.2.0`, and `1` are
   package and internal identities, not a release version.
3. **Not signed.** No signature, attestation, or SLSA/in-toto provenance claim. The only provenance
   record is `codefire.rust_build_provenance.v1`, which is an unsigned source-binding hash record;
   HEAD itself is unsigned (`%G? = N`). CodeFire's own commit/request HMAC signing
   (`known-limitations.md:179,184`) is a *product feature over repository content* and is **not**
   release-artifact signing — conflating the two would be the easiest false claim available here.
4. **Not legally cleared.** Nothing in this packet, the audit, or the exploration corpus is legal
   advice or a rights clearance. `exploration/phase-3/README.md:62` and
   `02-oss-license-governance-and-maintainers.md:9` already say so; `milestones.md:71` forbids
   "inventing legal permission … or treating an audit as legal advice."
5. **Not published.** No GitHub release, PyPI, or crates.io upload; no credential use; no network
   publication. The rehearsal is local-only, and the audit must verify the absence of hidden external
   publication (`milestones.md:48-49`).
6. **Not third-party compliant.** No claim of dependency-licence compliance, notice completeness, or
   SBOM coverage. 75 third-party crates are pinned in `Cargo.lock` with zero in-tree licence records.
7. **Not supported.** No support window, security-response commitment, or compatibility promise. No
   SECURITY, SUPPORT, or maintainer policy file exists, and
   `release-and-format-roadmap.md:31` states its own identity table "is an observed worktree baseline,
   not a compatibility promise."
8. **Repository access is not permission.** Neither this worktree, the `origin` remote, the retained
   runner, a green build, nor `Cargo.toml:12` constitutes a grant of distribution rights to anyone.

### 6. Q6 — smallest admissible rehearsal scope, and the unresolved-rights manifest

**No product or source change is an unavoidable M1 blocker.** The rehearsal can proceed truthfully
today with an internal-only decision, because M1 exit bullet 1 asks for a recorded rights decision and
an explicit refusal to infer, not for a resolved licence. A resolved licence is Stage 4 work
(`modernization-roadmap.md:102`), it requires the copyright holder, and choosing one is a stated
non-goal of both this packet and `milestones.md:71`.

**Smallest admissible rehearsal scope (rights-side).** Node 2 owns the mechanical topology; this is
the rights envelope it must stay inside.

```text
ADMITTED  A1   codefire-rs, bound to source revision d803527 (Rust inputs byte-identical to c78d5e0e)
ADMITTED  A2   codefire  (Python fallback script, as install.sh installs it: codefire-py)
ADMITTED  A3   install.sh
ADMITTED  A17  the source-binding provenance record + the rehearsal checksum manifest
ADMITTED  A18  completions, only if install.sh generates them in the isolated prefix
ADMITTED  A10  README.md + a bounded docs subset from A5/A6/A8/A9, read-only, for operator use
EXCLUDED  A4   no wheel/sdist is built            (drop D1)
EXCLUDED  A12  no full-tree source archive        (stop)
EXCLUDED  A13  no subprojects content             (stop)
EXCLUDED  A14  no docs/development content        (drop D2)
EXCLUDED  A15  no .codex content                  (drop D3)
EXCLUDED  A16  audit-latest.md never enters a manifest   (stop)
```

Rights rationale for the shape: A1+A2+A3 is exactly what `install.sh` actually installs
(`install.sh:214-215`: `codefire` and `codefire-py`). It is the smallest set that exercises the real
install path, and it adds **no** artifact whose only new content is an unresolved rights claim. A4 is
excluded precisely because building a wheel manufactures a *distributable-shaped* object whose
metadata asserts `UNLICENSED` — it would add the contradiction to the artifact set while proving
nothing the installed script does not already prove.

**Missing-rights manifest — checked-in files that do not exist.** Absence is evidence; each line was
verified by repository-wide search, not assumed.

| Missing | Verified by | Consequence recorded |
|---|---|---|
| `LICENSE` / `LICENSE.md` / `COPYING` | no match repo-wide | no outbound licence text; `Cargo.toml:12` stands alone and unbacked |
| Copyright holder statement | no `copyright`/`(c) 20`/`SPDX-License-Identifier` in any product file | owner of A1–A11 is undetermined; individual vs organization unresolved |
| `NOTICE` / `THIRD-PARTY-NOTICES` | no match repo-wide | 75 third-party crate obligations unrecorded |
| `deny.toml` / `about.toml` / any SBOM | absent | no dependency licence or supply-chain inventory tooling |
| `CONTRIBUTING` / DCO / CLA | absent | contribution rights path undefined |
| AI-authorship / attribution policy | absent, while `ai/README.md:3` and `provider-routing-policy.md:11-17` record AI-produced work | provenance of authorship unaddressed |
| `SECURITY` / `SUPPORT` | absent | no vulnerability intake or support window |
| `GOVERNANCE` / `MAINTAINERS` / `CODE_OF_CONDUCT` | absent | single-identity bus factor; no change authority record |
| `CHANGELOG` | absent | no release history to bind a version claim to |
| `pyproject.toml` licence field | `pyproject.toml:5-15` | modern Python packaging carries no licence declaration at all |
| Registry ownership record (PyPI `codefire-mvp`, crates.io `codefire-*`) | no checked-in record | name ownership unknown; **not checked** — checking is a network action and out of scope |
| `publish = false` in any manifest | `grep publish` over all manifests: no match | nothing in metadata *prevents* `cargo publish` / upload; see P1 |
| Trademark / export / privacy record | absent | unaddressed, listed for the human decision only |
| Release tag / signing identity | `git tag` empty; HEAD `%G? = N` | no release line, no signing identity |

**Classification.**

*`block-m1`: none.* No fact here blocks the recorded rights decision or the isolated
release/verify/rollback path. Per `milestones.md:51-53` no issue root is admitted from this lane, and
no new issue is filed (`milestones.md:71` / this packet's non-goals): every fact above is representable
in this packet, which is the packet's own stated bar for not filing.

| ID | Class | Fact | Why not `block-m1` |
|---|---|---|---|
| P1 | `park` | No `publish = false` in any Cargo/Python manifest; nothing in metadata prevents a registry upload | The rehearsal is local-only and runs no registry-capable command, so a procedural prohibition (node 2's exact command list) is sufficient. **Escalates to `block-m1` if and only if node 3 admits any step that can reach a registry** — then the smallest fix is `publish = false` in the four crate manifests, which is package-metadata editing and forbidden here |
| P2 | `park` | E1 `MIT` vs E2 `UNLICENSED` vs E3 silence | Resolving it requires the copyright holder to choose a licence — Stage 4 work (`modernization-roadmap.md:102`), a stated non-goal, and not needed for an internal-only rehearsal |
| P3 | `park` | 75 third-party crates with no licence inventory, notice file, or SBOM | Stage 4 supply-chain work (`modernization-roadmap.md:107`); an isolated internal install distributes to no one |
| P4 | `park` | `release-and-format-roadmap.md:28` says "five checked public schema IDs"; there are now **7** (`compatibility.rs:125-131`). Line 15's baseline is dated `2026-07-12`, and `exploration/phase-3/02:24` still says 266 commits against 336 today | Documentation drift in files this packet may not edit. The licence rows (E6, E7) remain accurate — only the counts are stale |
| P5 | `park` | No SECURITY / SUPPORT / GOVERNANCE / MAINTAINERS / CONTRIBUTING / CHANGELOG | Explicitly Stage 4 stage work, explicitly excluded from the M1 slice by `modernization-roadmap.md:116` |
| P6 | `park` | No AI-authorship or contributor-attribution policy despite an AI-assisted development system | Rights-relevant, needs a human decision, blocks nothing local |
| P7 | `park` | No in-tree toolchain pin (`rust-toolchain*`, `.cargo/` absent); reproducibility rests on the provenance record | Node 2's build-reuse decision; the provenance record already binds `cargo 1.97.0` / `rustc 1.97.0` |

| ID | Class | Dropped from M1 | Reason |
|---|---|---|---|
| D1 | `drop` | Building a wheel/sdist for `codefire-mvp` 0.2.0 | Adds a distributable-shaped artifact asserting `UNLICENSED` and proves nothing beyond the installed A2 script |
| D2 | `drop` | Any `docs/development/**` content in the artifact set | 525 internal management files; not product, no rehearsal value, expands the rights surface |
| D3 | `drop` | Any `.codex/**` content in the artifact set | Internal agent config; not product |
| D4 | `drop` | Registry name-availability or ownership checks for `codefire-mvp` / `codefire-*` | Requires network access; out of scope and a publication-adjacent action |

### 7. Validation

Run at this worktree root with only this packet modified:

```text
python3 docs/development/tools/check_project_docs.py
        -> exit 0   "Project docs check passed; 555 Markdown files inventoried."
python3 docs/development/tools/check_ai_acceleration_docs.py
        -> exit 0   "AI acceleration docs check passed."
git diff --check
        -> exit 0, no output (no whitespace errors)
git status --porcelain
        -> " M docs/development/ai/task-packets/milestone-release-operational-trust-rights.md"   (only line)
```

### 8. Node-3 handoff

1. **Rights decision to record, verbatim:** all candidate artifacts are **internal-rehearsal only**;
   **no artifact is publishable**; the missing outbound licence and missing copyright holder are
   recorded as public-distribution blockers and are **not** inferred from repository access, from
   `Cargo.toml:12`, from package metadata, from the retired v0.6–v1.0 labels, or from a successful
   build.
2. **No `block-m1` from this lane.** No issue root is admitted; no new issue is filed. Node 3 may
   admit the bounded rehearsal on the rights axis without any node-4 source fix.
3. **Source binding is already established:** rehearse against `d803527`. Rust build inputs and
   `schemas/public` are byte-identical to `c78d5e0e`, and every product-facing path is byte-identical
   between the two commits — only `docs/development/**` moved. The retained runner
   (sha256 `90cb2895…`) and provenance (sha256 `1731b863…`) are consistent with this HEAD; reuse
   versus rebuild is node 2's call.
4. **`/workspace/project` must not be the artifact source** — 248 dirty paths at HEAD `f164412`.
5. **Rights envelope for node 5a/5b:** admit A1, A2, A3, A17, A18, and a bounded read-only doc set.
   Exclude A4, A12, A13, A14, A15, A16. Reject any manifest that names `audit-latest.md` or ships a
   full-tree archive.
6. **Carry all eight prohibitions in section 5 into the rehearsal record and the node-6 audit.** The
   subtlest one to police is #3: CodeFire's HMAC commit/request signing is a product feature over
   repository content and must never be reported as release-artifact signing.
7. **Escalation trigger to watch:** P1. If any admitted rehearsal step can reach a package registry,
   P1 becomes `block-m1` and the smallest fix is `publish = false` in the four crate manifests —
   which this packet was forbidden to write.
8. **Version reporting rule:** the rehearsal must report the tuple (source revision `d803527`; Cargo
   `0.1.0`; Python `0.2.0`; CLI `codefire foundation 1`; object/record `1`; commit root schema `2`;
   Atom hash schema `4`; envelope `codefire.command_result.v1`; support profile `codefire.support.v1`;
   semantic profile `sealed-snapshot-v1`; 7 registered public schema IDs; no tag; unsigned), never a
   single version number.
9. **Fan-in note for node 2's lane:** this packet did not read, prejudge, or duplicate the
   release/verify/rollback topology. Section 6's envelope constrains which artifacts that topology may
   contain; it does not choose the release root, prefix, or operator commands.
