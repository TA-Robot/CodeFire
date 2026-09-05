# REC-08: Preserve replaced Store staging candidates (CFB-139)

Status: implementation under independent review; two ownership/accounting corrections pending

## Objective

Fix the newly reproduced deletion of a replacement file during prepared-stage recovery. Explicitly admitted
M1 blocker after CLI/core passed and Store aborted. Base3ca04d7 includes reviewed CFB138 fix; do not change it.

## Scope

Only crates/codefire-store/src/lib.rs recovery candidate code around1666–1830 and its focused tests around12649;
read helpers and ledger/identity types as needed. This packet may be updated with diagnosis/results. No other
writer owns Store. Root owns issues/docs/governance. If a separate private helper module materially reduces
complexity, propose it first; prefer a minimal coherent patch in the existing area for this blocker.

## Invariants

No foreign/replaced-file deletion; no-follow and one-link checks preserved; limits charged before effects;
resource ownership bounded and released on errors/cancel. Do not disable debug Drop ownership assertions to
hide the panic. Do not rely on inode/timestamp uniqueness or flaky filesystem allocation for the regression.
Pinning an inode does not by itself freeze file bytes: assess in-place tampering at the before_unlink boundary.
Do not redesign all Store paths, public formats, core ledger, or claim an atomic unlink-identity primitive.

## Validation

Source docs/development/tools/rust_env.sh for cargo. Reproduce exact fully qualified test from cfb-139.md.
Logs and inode-reuse evidence are in target/recovery-rust/integrated-store-focused.log and integrated-inode-reuse.json.
Add meaningful boundary regression, run focused prepared-stage recovery tests and cargo fmt --check. Root
will independently review and run Store plus full workspace gates after patch is stable. Do not run full
workspace yourself or spend turns polling long builds repeatedly; checkpoint command/session evidence.

## Non-Goals

CFB138 bootstrap code, CLI UX/import, provider or toolchain changes, external API/network/model calls,
commits, main worktree or canonical Algorithm mutation, other issue roots, and nested agents.

## Handoff

Explain exact identity/ownership repair and resource bound, list tests actually run (zero matched is not pass),
changed files and residual assumptions. Stop after a reviewable patch; give concise checkpoint before turn
bound. Budget32 turns, last four reserved for handoff. If complex, checkpoint rather than weaken invariants.

## Independent Review Checkpoint

Native reviewer examined Store SHA256 `71c10a70e2a34cf8000f31adb8dcc8eea3e94bde2b9c906845114e2ae6ca2eb9`. Pin lifetime and raw-byte digest direction are valid; block-m1 findings: acquire candidate ownership before File/digest creation, and record scratch ownership only after a successful atomic ledger acquisition. Rejected acquisition must preserve the original typed budget error and caller-owned TemporaryBytes. A suggested SnapshotRevalidation reclassification was withdrawn after checking the preparation call site; existing Initial/AggregateBytes accounting is appropriate to stale cleanup.

Initial Grok32-turn run ended at its limit with a patch, not completion. The test-only resume was interrupted to incorporate the two concrete findings; neither run is counted as acceptance. A focused dependent fix follows in the same session; no Fable usage or paid fallback.
