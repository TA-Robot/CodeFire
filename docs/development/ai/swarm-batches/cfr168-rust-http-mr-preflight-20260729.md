# CFR-168 Rust HTTP MR Create/Review Response Preflight

Close the independent review's Rust MR create/review pre-effect response gap on immutable
current-state snapshot `e2a218e015953f082245f36dbc6aee906b837e5a`.

Use Fugu 1.1 Ultra (`fugu-ultra-v1.1`). Work only in the assigned worktree and write set. Do not
change issue status, edit management ledgers, or widen this packet into upload, apply, Python,
storage, metrics, or unrelated HTTP refactoring.

## Production defect

Rust HTTP upload and MR apply preflight a request-owned response before durable publication.
`handle_http_request_merge` and `handle_http_request_review` currently leave
`RequestContext.prepared` empty, so response encoding can fail only after the mutation commits.
The default response limit may make the gap hard to reach with ordinary inputs; deterministic
reduced-limit/fault tests must establish the contract directly.

## Required behavior

- For MR create and review, build/freeze the exact success response and every durable recovery
  response on the same request `DecisionWorkLedger` before the first mutation.
- Charge response body/header and retained ownership before effect, poll the request deadline, and
  hand the exact prepared owner to the outer HTTP publisher without re-encoding.
- Preserve authorization, signature checks, subject binding, status/error precedence,
  idempotency/recovery behavior, response JSON bytes, audit records, and successful durable state.
- A response limit, deadline, final-fence, serialization, or injected prepared-owner failure must
  produce the typed existing diagnostic and leave MR, branch, generation, nonce, idempotency,
  transaction, recovery, and audit state byte-identical.
- Replay must preflight the immutable recorded result and must not repeat the mutation.
- Release every prepared-response permit/retained byte on success, abort, disconnect, or error;
  leave one terminal request reason and no open permit.
- Keep upload and apply byte-identical and behaviorally unchanged.

## Required production-path tests

Add focused tests in `crates/codefire-cli/src/http.rs`:

- live MR create and review success/replay byte equivalence;
- response body/header/retained exact / minus-one / plus-one;
- deterministic deadline at response preflight, pre-effect, and publication handoff;
- filesystem authority snapshots proving no effect on every typed failure;
- prepared owner identity is the request ledger and no second encoding occurs;
- disconnect/abort cleanup with zero concurrent debt;
- upload/apply neighboring regression tests remain green.

Use reduced profiles or explicit fault injection to make the previously structural failure
reachable. Do not accept a test that only inspects source or asserts `prepared.is_some()`.

## Validation

At minimum run:

```bash
export RUSTUP_HOME=/tmp/codefire-rustup/rustup
export CARGO_HOME=/tmp/codefire-rustup/cargo
export PATH="$CARGO_HOME/bin:$PATH"
cargo fmt --all -- --check
cargo test -p codefire-cli --bin codefire-rs http::tests --locked
cargo clippy -p codefire-cli --bin codefire-rs --locked -- -D warnings
python3 docs/development/tools/check_source_docs.py
git diff --check
```

Run the full `codefire-cli` binary test target after focused tests if time permits. Commit the
implementation and leave the worktree clean.

## Handoff

Report the exact create/review preflight call chains, ledger/owner identity, effect boundary,
exact-limit/no-effect/replay evidence, neighboring route results, and every residual response
preflight surface.
