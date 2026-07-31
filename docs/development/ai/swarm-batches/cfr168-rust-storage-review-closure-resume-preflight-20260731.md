# CFR-168 Rust Storage Review-Closure Resume Preflight

Status: blocked until source-keyed Rust control integration, final manifest
binding, and count-free admission

This preflight binds the corrected control base and the remaining exact
requirements found by the independent manager audit. It is not an executable
manifest and does not authorize a new implementation session.

## Exact corrected control base

| Item | Exact value |
|---|---|
| Corrected addendum commit | `f69e8de43be8d7540f06a0c8378d176ae1c59bfa` |
| Corrected addendum parent | `926b179f2dcad8d4fabbeebaf8547f34a50b0c1b` |
| Corrected addendum tree | `31e5736013d848b04bd352042f5096429a196177` |
| Corrected addendum SHA-256 | `9a56e1ce681a05acfd8b1e8a4f13670587ec6bda8da3b73a00a8da0c71d8b113` |
| Candidate-rooted report copy | `9b5f4eaad21759c5cefed2d17602d49b3daa08c2` |
| Canonical report copy | `8ac6741d218e1869fef91d96d29ac0d18d9d674d` |
| Independent report commit | `71222e1cec1b0dc10003845428abfb611ba8bf71` |
| Report blob | `cd0ffc8458d8240ac7a0215a365cf32b5b91e7da` |
| Report SHA-256 | `6438df445b2d05510895fd23f73a4e71ba49238da15b00e86abf1a477b5ab960` |
| Original implementer session | `c28925df-d4bc-4994-b905-73aaaf2425e5` |
| Session-id file mode / bytes | `0600` / `36` |
| Session-id file SHA-256 | `6ad290e16f56323234a67189f8ecc61d0654fb17ddf990ab28147958f92d949d` |
| Terminal events SHA-256 / bytes | `a3c682d225965ff3c01a1561697c9addd9145e024e0786d495a2bc8501134564` / `8893540` |
| Terminal transcript SHA-256 / bytes | `d25fb61a8f0b280cda668262a1739eb3072b9c79e6cfd52560fc61ee72303071` / `29734` |

The session terminated once with `success` / `completed` / `end_turn`, 187
turns, scheduler return code zero, and no live manager. Local worktree/session
recovery is therefore available; server-side resume remains unproved until
the one authorized resume call succeeds.

The final executable manifest/control must use a later clean revision that
contains this preflight and the reviewed source-keyed Rust controls. It must
bind that exact later revision, not `f69e8de`, as `base_revision`.

## Closed write set

The authoritative eleven-path set is exactly the list in
`cfr168-rust-storage-review-closure-repair-control-addendum-20260731.md`.
A final machine control must reproduce all eleven paths individually. A
directory entry, glob, repository-root grant, extra result path, or omitted
path is `BLOCK`.

## Required fixed-limit subject axes

In addition to every owner/formula exact-minus-one test and the R1 graph
node/edge cases, the repaired candidate and fresh reviewer must add one
subject unit while holding the configured limit fixed for:

1. `AggregateBytes`;
2. `ObjectRecords`;
3. response bytes and the corresponding response counter;
4. `MaximumFrontier`; and
5. selected-URL `GraphEdges`.

For every axis, prove the exact subject at `L`, rejection at `L - 1`, and the
one-unit-larger subject rejected at unchanged `L`. Expected values must be
hand-derived or independently encoded; replaying a candidate snapshot is not
evidence.

The same subject-growth rule applies to graph nodes/edges and every new
portable collection/authority formula whose public claim includes an input
unit.

## Required process publication gates

The new real-process test must be exactly:

`crates/codefire-cli/tests/storage_report_publication.rs`

Run it through the source-keyed Cargo wrapper in both profiles:

```text
cargo test -p codefire-cli --test storage_report_publication --locked -- --nocapture
cargo test -p codefire-cli --test storage_report_publication --release --locked -- --nocapture
```

Both gates require a nonzero executed test count, exact stdout/stderr byte
assertions, all R4/R13 writer sequences, one visible outcome, source key,
monotonic duration, complete log/result digests, and unchanged post-gate
candidate identity. A unit/helper-only substitute is not closure.

## R9 shared-parser rule

R9/F5 requires full executable closure on the exact candidate. The authorized
write set does not include a shared remote parser implementation. Therefore:

- if the storage-owned adapter can enumerate and acquire every actual parser
  owner before allocation without editing the shared parser, close R9 with
  independent exact/minus-one/subject-plus-one evidence; or
- if a shared parser change is required, leave R9 `OPEN`, record the exact
  symbol/path/formula/test dependency, and return a blocked handoff.

A shared-parser handoff is not R9 closure. Do not widen this lane to
`remote.rs`, call the whole candidate closed, or request acceptance while R9
remains routed.

## Final pre-launch checks

Immediately before resume:

1. verify the final bound revision/tree and clean porcelain including
   untracked files;
2. verify this preflight, both prior controls, the immutable report, session
   file mode/hash, terminal events/transcript hashes, and exact worktree
   registration;
3. verify the source-keyed wrapper/tool/test/doc commit and file hashes;
4. verify no live Cargo/rustc/Clippy or other Storage implementation process;
5. run the exact Rust execution-profile audit;
6. run count-free `rust-heavy` Claude admission;
7. generate the resume envelope through a protected file/stdin; and
8. start only the original session through the existing agent/worktree
   mapping.

The outer wrapper invocation must not contain a secret, prompt body, or
session UUID. The envelope must name the final bound revision, immutable
review, original packet, correction addendum, this preflight, exact write set,
source-keyed Cargo entry point, and no-amend/no-rebase/no-squash/no-reset/
no-merge rule.

No launch occurs from this preflight alone.
