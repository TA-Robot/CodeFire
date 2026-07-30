# CFR-168 Rust HTTP MR Preflight Independent Claude Review

Independently review the completed Rust HTTP merge-request create/review response-preflight
candidate using Claude Opus 5 (`claude-opus-5`). This is review-only: do not modify product code,
tests, issue state, or management ledgers.

## Exact subject

- baseline/task revision: `3c23e381f871724d31332bd179671db7cbb9d941`
- completed target: `71854e20689d2dd79246fa741e0e6da443d04d1a`
- target branch:
  `claude-agent/cfr168-rust-http-mr-preflight-claude-hedge-20260730-mr-create-review-preflight-hedge`
- cumulative path: `crates/codefire-cli/src/http.rs`
- contract:
  `docs/development/ai/swarm-batches/cfr168-rust-http-mr-preflight-20260729.md`

Do not infer acceptance from the implementation agent's 141/141 or 776/776 results.

## Required review

1. Create and review prebuild the exact success and durable-retry status/body/header bytes on the
   request ledger before the first object/publication effect; writer paths never re-encode or select
   fallibly after an unrecorded effect.
2. Exact/minus-one/plus-one `response_bytes`, `response_header_bytes`, retained/temporary ownership,
   encoding failures, deadline phases, and disconnects are pre-effect or recover to one durable
   byte-identical result with zero request debt and no open permit.
3. Audit the reported pre-existing outer retained residual on merge-create success/error. Because
   CFR-168 owns the complete request invariant, prove it is intentionally owned through request
   termination or require repair; baseline provenance alone is not exemption.
4. Audit the second `MergeRequestResponsePreflight` owner beside
   `PreparedHttpMutationTransaction`: one ledger may have multiple typed owners, but no duplicate
   encoding/reservation, split fence, or cleanup order may create an effect gap.
5. Preserve authentication/nonce/signature, escrow, review materialization, error precedence,
   replay, post-commit 503 retry semantics, apply/upload neighbors, and exact wire bytes.
6. Verify production-path non-vacuity, updated existing tests, test-only hook isolation, full
   authority snapshots including durable result/transaction/audit state, strict Clippy, focused
   HTTP tests, and representative cross-runtime cases.

## Output

Write exactly:

`docs/development/ai/swarm-results/cfr168-rust-http-mr-preflight-review-claude-20260730/review.md`

Include exact hashes, `ACCEPT` or `REQUEST_CHANGES`, severity-ordered findings with concrete
file/line evidence, all six verdicts, commands/results, integration conflict map, and residual
risk. Commit only the report and leave the reviewer worktree clean.
