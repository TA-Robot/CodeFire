# AI Task Packet

Status: ready-for-build

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

