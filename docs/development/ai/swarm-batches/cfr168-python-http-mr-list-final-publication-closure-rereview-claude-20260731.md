# CFR-168 Python HTTP MR-List Final-Publication Closure Rereview

Status: ready for exact independent review

Provider/model: fresh Claude Opus 5 (`claude-opus-5`)

Role: independent reviewer

Mode: report-only

All candidate fields are bound to the immutable manager-recomputed subject
below. The runnable manifest is created only above this ticket, the candidate
card, and the identity helper.

## Objective

Independently review the exact three-commit successor that repairs the prior
Claude `REQUEST_CHANGES`. Reproduce F1--F3, both recorded residuals, the
handler-owned final proof, displacement/disposal precedence, and the
publication state machine through the production GET dispatcher. Re-falsify
all seven earlier closures and return one immutable report.

The reviewer uses a new agent, session, and worktree. It inherits no
implementer conversation, WIP, conclusion, or mutable `/tmp` result.

## Exact subject

| Item | Exact value |
|---|---|
| Rejected predecessor candidate | `4930d25178be11e064060487f8c254802b09868d` |
| Prior independent reviewer commit | `0aa0d3d0bfbaafcc9d67bccdc90207810a60fe4b` |
| Prior report in candidate lineage | commit `6ee36460b7305f69998eefbe11f2c760dea9966f`; blob `48cb3cb98e5e1dde4a0b5cecd1bfdfaf1af61a45` |
| Prior report SHA-256 / verdict | `dc4f66ba9649e4333980996c6f83d7f28b0f5f4bd165683adac55bf6d9704b2e` / `REQUEST_CHANGES` |
| Continuation base / merge base | `06e519e4a434c07324744018af80ecfa8366f809` / `06e519e4a434c07324744018af80ecfa8366f809` |
| Candidate SHA / parent / tree | `b16404676639336f705feffd9af4857d4a1fcc69` / `091dc2a562ffcb94557b974ac1828c80d09e7ccc` / `36185e320338b67e1610d82fcc81cba310c36026` |
| Additive commits | `e59b7ec85548dda6f6dbaed5d9b22b4bd88a1a60`, `091dc2a562ffcb94557b974ac1828c80d09e7ccc`, `b16404676639336f705feffd9af4857d4a1fcc69` |
| Additive count / merges | `3` / `0` |
| Binary full-index patch SHA-256 | `8391b582bd219fc0d7a3385cdb3e139839a442aeac47b80951270213bbe095c2` |
| Exact changed paths | six paths and every blob/numstat/content digest in the candidate card |
| Terminal state | implementer completed; all provider/gate processes terminal; candidate worktree clean |

The complete manager card is:

`docs/development/ai/swarm-results/cfr168-python-http-mr-list-final-publication-closure-repair-candidate-ready-20260731/candidate.md`

Recompute identity, ancestry, tree, ordered commits, patch digest, paths,
blobs, and clean state. The task base contains
`docs/development/tools/capture_candidate_identity.py`, SHA-256
`c4606a4b48af8f844a9b42b32e681cbc028df8b884fe75a66dcb6fafc16557b9`.
Because the review checkout is above the product candidate, run the helper
against a fresh clean detached scratch checkout of the exact candidate. Do not
point it at the implementer's worktree and do not follow a branch tip. Any
identity mismatch is immediate `REQUEST_CHANGES`.

## Scope

Read:

- the exact candidate range and live Python HTTP dispatcher;
- the immutable prior report;
- the continuation packet;
- the candidate card and identity helper;
- the focused MR-list tests and affected legacy CLI tests;
- the four changed source-contract documents; and
- the earlier reports/tests necessary to re-falsify inherited closures.

The only authorized repository write is:

`docs/development/ai/swarm-results/cfr168-python-http-mr-list-final-publication-closure-rereview-claude-20260731/review.md`

Product, tests, source docs, packets, cards, helpers, existing reports, issues,
ledgers, disposition indexes, lifecycle state, Rust, devcontainer, packaging,
Install, and Dogfood are read-only or out of scope. Temporary scratch
checkouts, logs, and deliberate mutations may be created outside the review
worktree and must be removed when no longer needed.

## Required independent falsification

### 1. Final authority fence, teardown, and publication proof

Trace the production order from response encoding through the first response
byte. Observe the real `fcntl.flock(LOCK_UN)` and all relevant `os.close`
calls, not only named Python methods.

The candidate intentionally distinguishes two boundaries:

- the project fence is the final **authority-dependent** observation; and
- the handler-owned publishability proof is the final **fallible** operation
  before owner transfer and publication.

Five teardown calls after the authority fence are expected and are not by
themselves a defect. Prove that all five complete before the handler proof,
that zero fallible/authority-dependent/callback-capable/input-proportional
work occurs after that proof, and that the only remaining transitions are
`take` and `publish`. Move the proof back inside each pinned context in a
scratch mutation and require the production ordering oracle to fail for the
original five-call reason.

### 2. Post-route filesystem failure and prepared-response displacement

Drive genuine filesystem-origin `OSError` values from unlock and from each
relevant descriptor-close boundary after a 200 has been prepared. Through the
live GET dispatcher, prove that:

- the older success is disposed before another response is selected;
- no projection or stale 200 reaches the wire;
- a safe typed/logged filesystem outcome is selected with exact origin;
- transport-origin timeout/reset/disconnect still abandons the connection;
- `MemoryError`, request deadline, and primary-error precedence remain intact;
  and
- GET and POST share compatible origin/displacement semantics without a
  second route-local ownership protocol.

Falsify the budget, `CFError`, and `OSError` arms independently. Do not accept
a result merely because the candidate's one unlock regression is green.

### 3. Publishability proof and zero debt

Leave retained ownership, temporary ownership, and a permit alive at the
proof point in separate production-reachable scratch probes. No success byte
may publish. Verify exact prepared-owner disposal, terminal reason
`cleanup_incomplete`, residual amount, zeroing of every releasable owner,
permit state, typed diagnostic visibility, and connection behavior.

Silence the proof only and require the rejected 200-with-debt outcome to
return. A test-only flag or helper-only counter is not proof.

### 4. Production response-slot state machine

Audit every transition among `empty`, `prepared`, `closing`, `proved`,
`published`, and `disposed`. Prove:

- only `prove_and_take` can hand bytes to the production publication point;
- `take` refuses unproved bytes;
- no state can publish twice or publish after disposal;
- proof failure cannot be caught by an outer arm that revives an old 200;
- a closing encoder publishes nothing; and
- destructors are not required for correctness.

### 5. Disposal failure precedence

Fail prepared-owner disposal while budget, `CFError`, and filesystem errors
are replacing a response. The primary diagnostic must remain primary, the
disposal failure must be recorded observably as secondary, ownership must not
be zeroed before release succeeds, the connection must close, and neither the
old success nor a competing replacement may publish.

Exercise first-attempt recovery and permanent failure separately. A swallowed
release or a second response is blocking.

### 6. Recovered cleanup diagnostics

Fail every relevant cleanup release once and verify the retry. A recovered
fault must reach either the production handler diagnostic or a note on the
already-selected primary error. Verify bounded `fault_count`, attempt count,
owner identity, truncation after `CLEANUP_FAULT_REPORT_LIMIT`, and silence for
a clean request. Append-only state or an unread property does not count.

### 7. Exact over-release and latent double release

Request a release larger than the exact owner scalar and prove a typed
fail-closed accounting error, unchanged ledger ownership, observable fault,
and a successful exact retry. Recreate the per-session release failure that
previously allowed the aggregate `_sessions_retained` release to double
release on retry. Every live session key and residual aggregate must be
released exactly once.

### 8. Preserve all inherited closures

Re-falsify the seven dispositions the prior reviewer marked closed:

1. parent-directory authority versus leaf absence;
2. filesystem versus transport error origin across GET/POST;
3. retryable cleanup without local ownership erasure;
4. terminal mode/link/size/identity/content metadata;
5. authority mutation around encoding/fences/publication;
6. the route-wide descriptor protocol and scoped byte oracle; and
7. each production poll site's bounded-work necessity and exact
   1/8/24/80/200 curves.

Also preserve real multi-fault filename precedence, leaf-symlink typed
responses, operation-local escrow memoization, one sealed session, exact
handler ledger/deadline, reserve-before-allocation, status/byte parity, route
isolation, affine counters, stable-link compatibility, and the cardinality
and transport-harness flake repairs.

Every new six-test closure must be non-green on exact `4930d251…` for the
named reason. State honestly which probes fail versus error on `38da8988…`
because symbols were absent. A legacy pass or missing symbol does not prove
the successor correct.

### 9. Same-root residual and composition audit

Search all callers of the changed shared primitives and all GET/POST
publication paths. Look for:

- a proof bypass or second owner-transfer path;
- a swallowed post-route error;
- incomplete disposal on an outer exception arm;
- a diagnostic that is written but never consumed;
- ownership erased before a successful release;
- unbounded diagnostic work;
- response allocation before reservation;
- a route that depends on destructor cleanup; and
- divergence that Upload or MR-write composition would inherit.

No new same-root residual may remain for `ACCEPT`.

## Required validation

Run and record exact command, count, monotonic duration, exit status,
toolchain, unique log path, and log SHA-256 for:

- identity/ancestry/tree/patch/path/blob/clean-state recomputation;
- `py_compile` on `codefire`, the focused module, and
  `tests/test_codefire_cli.py`;
- `tests.test_python_http_mr_list_decision_work -v` with all 49 tests;
- these exact six new regressions:
  - `test_the_terminal_fence_immediately_precedes_the_first_response_byte`;
  - `test_post_teardown_filesystem_failure_replaces_the_prepared_success`;
  - `test_unpublishable_prepared_response_never_reaches_the_wire`;
  - `test_prepared_disposal_failure_publishes_nothing`;
  - `test_owner_over_release_fails_closed`; and
  - `test_recovered_cleanup_faults_reach_a_production_diagnostic`;
- rejected-candidate and legacy reversal;
- deliberate proof/disposal/release regressions and live-socket probes;
- exact affected `tests.test_codefire_cli` IDs;
- the complete `tests.test_codefire_cli` module;
- `unittest discover -s tests -p 'test_python_*.py'`;
- `check_source_docs.py`;
- `check_project_docs.py`; and
- `git diff --check`.

Use a new reviewer-specific `/tmp` prefix; never trust or overwrite the
worker's paths. Use monotonic/performance-counter duration rather than wall
clock. The cited repository timed-gate helper is absent from the candidate;
record that process deviation and use an auditable monotonic wrapper without
changing the repository.

The manager observed 228 whole-CLI tests with two failures:

- `test_completion_command_and_install_completion`; and
- `test_install_script_installs_codefire`.

Both report that Cargo is required, both reproduce identically at the
immutable base, and `tests/test_codefire_cli.py` plus `install.sh` are
byte-identical at the base and candidate. This is not permission to call the
whole module green. Independently repeat the two IDs at candidate and base,
record that `cargo`/`rustc` are absent, and distinguish the inherited
environment failure from candidate behavior. Any additional failure, changed
reason, or failed affected in-scope test is `REQUEST_CHANGES`.

Cargo, Docker, Install, Dogfood, unrestricted all-language discovery, and
Python-composition integration gates remain manager-owned. Do not install a
toolchain or alter the environment to make evidence green.

## Decision and handoff

Write and commit exactly the declared report. Include:

- recomputed immutable identity;
- row-by-row F1--F3, recorded-residual, and seven-prior-finding dispositions;
- production call graph and owner/effect/fence/state timelines;
- mutation and reversal evidence;
- validation and whole-CLI environment discrimination;
- integration overlaps and unavailable evidence; and
- one literal verdict.

`ACCEPT` requires exact identity, every blocking/recorded/inherited item
closed, non-vacuous production-reachable counterexamples, one terminal
outcome, no success with debt or permits, all in-scope gates independently
green, the two inherited CLI failures exactly explained at base, and no new
same-root residual. `ACCEPT_WITH_NOTES` is forbidden. Otherwise return
`REQUEST_CHANGES`.

Acceptance is scoped to immutable candidate `b1640467…`. It does not authorize
integration, Close, Install, Dogfood, accepted Upload/MR-write bindings,
shared-primitives preflight, Python composition, or any successor protocol.
