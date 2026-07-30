# CFR-168 Rust File Doctor Independent Review

Independently review the completed Rust live file-remote Doctor decision-work adoption. Use Fugu
1.1 Ultra (`fugu-ultra-v1.1`). This is review-only: do not modify product code, tests, issue state,
or management ledgers.

## Exact subject

- baseline/task revision: `ea8f2a641b68d06fe4a266f304dd03daa1f4a7de`
- completed target: `52a025e`
- target branch:
  `fugu-agent/cfr168-rust-file-doctor-recovery-20260729-file-doctor-restart`
- cumulative path: `crates/codefire-cli/src/remote/doctor.rs`
- contract:
  `docs/development/ai/swarm-batches/cfr168-rust-file-doctor-ledger-20260729.md`

Do not infer acceptance from the implementation agent's clean commit or reported 775/775 result.
Re-derive the production call chain and the required invariants from the target.

## Required review

1. The live `doctor cf://...` file route owns exactly one command ledger through target discovery,
   project gate, Store validation, diagnostics, report encoding, terminal selection, and cleanup.
   No compatibility helper may silently create an unobserved second decision owner.
2. Every retained collection, path, raw document, diagnostic projection, output buffer, and Store
   handoff is reserved before allocation/read/growth. Exact/minus-one/plus-one failures are typed
   budget outcomes, never corruption or a successful partial report.
3. The final subject fence detects same-path/inode/size/mtime content replacement and occurs before
   report publication. Every injected budget/deadline/tamper/observer failure leaves zero current
   debt, no open permit, and no semantic repository mutation.
4. Quick/full behavior, text/JSON bytes, issue ordering, missing/corrupt/future-version precedence,
   local/file compatibility, and advertised capability semantics remain unchanged.
5. Audit the test snapshot treatment of persistent project-gate files. It may exclude only
   nondeterministic lock ownership metadata and must still compare every semantic file byte-for-byte.
6. Independently run the focused Doctor tests, strict Clippy, format/diff checks, and a representative
   set of legacy Doctor tests. The implementation agent's broad 775-case run may be corroborating
   evidence, but do not claim reviewer execution unless rerun.

## Output

Write exactly:

`docs/development/ai/swarm-results/cfr168-rust-file-doctor-independent-review-20260730/review.md`

Include the exact hashes, `ACCEPT` or `REQUEST_CHANGES`, severity-ordered findings with concrete
file/line evidence, a verdict for all six items, commands/results, and residual risk. Commit only
the report and leave the reviewer worktree clean.
