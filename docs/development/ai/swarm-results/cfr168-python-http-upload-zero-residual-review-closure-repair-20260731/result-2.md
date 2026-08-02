# CFR-168 / CFB-138 Python HTTP Upload — Second Review-Closure Repair

Implementer: Claude Opus 5 (`claude-opus-5`), original implementer session
`7baa1275-0ee2-476a-af46-1907386ebceb`, resumed in its existing isolated worktree.

Prior candidate (reviewed): `467b0c27715e424ae18ffb5664da053b55915e2d`
Report answered: Fugu `57d6792106f09c17a1d154aedb5a64d342e5d19c23db7e9963c46061c868c132`, verdict
`REQUEST_CHANGES`, **HIGH-2** and **MEDIUM-4** open; **BLOCKER-1** and **HIGH-3** closed.

**Implementer evidence only. No acceptance is claimed.** A fresh independent rereview is requested;
this lane did not review its own candidate.

## HIGH-2 — retryable exactly-once control release

The prior candidate erased the claim, the amount, and the closed/acquired owner state *before* the
fallible `DecisionWorkLedger.release`. A pre-effect failure stranded the bytes with nothing left
owed, so the retry was a silent no-op.

Three release sites were rewritten around one transition,
`_AccountedUploadSourceMemory._release_ledger_exactly_once`: it compares the ledger's scalar before
and after, runs the caller's `consume` **only** when the ledger really moved, and always propagates
the failure.

| Site | Prior behaviour | Repaired behaviour |
|---|---|---|
| `_AccountedUploadSourceMemory.close` (retained control, 11,456 B) | `_acquired`/`_closed` cleared first; strand on fault | owner stays open+acquired until the ledger gives the reservation back |
| `_AccountedUploadSourceMemory._release_control_memory` (exact dimension) | claim decremented first | claim decremented only on a proven commit |
| `SealedCommitValidationSession._release_control_scalar` (ordered projection) | scalar zeroed first | scalar cleared only once the owner accepted the release |

`validate_sealed_commit` additionally gained primary-error precedence, so a cleanup fault no longer
masks the decision that caused the unwind.

Measured, injecting one pre-effect `ledger.release` failure per path:

| Path | Fault raised | Ledger after fault | Still owned | After retry (`owner.close()`) | Second close |
|---|---|---|---|---|---|
| `validate_sealed_commit` | yes | `retained_bytes: 11,456` | 11,456 | `{0, 0}`, no permits | no double release |
| `validate_remote_json_publication_identity` | yes | `retained_bytes: 11,456` | 11,456 | `{0, 0}`, no permits | no double release |
| `preflight_remote_json_publication` | yes | `retained_bytes: 11,456` | 11,456 | `{0, 0}`, no permits | no double release |
| ordered projection (480 B on a 53-document closure) | yes | `{0, 0}` — the session's own cleanup retried the still-owed release | 0 | `{0, 0}`, no permits | no double release |

## MEDIUM-4 — charge-verified pre-allocation boundary

`list(self.documents)` now runs behind `SealedCommitValidationSession._charged_ordered_projection`,
which charges, then calls the named `_charged_allocation_boundary`, then records what is owed, then
allocates. The boundary refuses an allocation whose charge cannot be **witnessed**:
`_control_charge_witness` treats `RESERVED_DIMENSIONS` (whose upper is already on the ledger) as
self-witnessing and requires every other dimension to be visible on the caller's ledger itself.

The regression observes the memo's `__iter__` — the only place this route iterates it — so
"before allocation" is proved directly rather than inferred:

| Case | Outcome | Projection allocated |
|---|---|---|
| healthy | completes, zero debt | yes, once |
| projection charge removed | `CFError: ... ordered object-id projection is not charged before allocation` | **no** |
| charge never reaches the caller's ledger | same typed boundary error, as the *primary* error | **no** |

## Mutation matrix

Each mutation applied alone to a clean export of the head tree at `/tmp/mut`, run, reversed, and the
tree proved byte-identical to baseline `codefire` SHA-256
`63a9be232d064afa022432537f0686e57d47845b1f5654b929a69acec7a872fa` before the next.

| Mutation | Focused result | Failing test | Time |
|---|---|---|---:|
| `m1_no_projection_claim` | FAILED (4 f / 51 e) | boundary + projection tests | 29.609s |
| `m9_no_source_temporary` | FAILED (1 f / 20 e) | boundary + projection tests | 87.733s |
| `m10_no_allocation_boundary` | FAILED (1) | `test_ordered_projection_allocation_requires_a_proved_charge` | 93.295s |
| `m11_erase_before_release` | FAILED (3) | `test_control_release_is_retryable_and_exactly_once` | 86.780s |

`m1` and `m9` — the two the prior report found detected only *after* allocation — now fail at the
boundary, with the in-process oracle asserting the memo was never iterated. `m10` and `m11` pin the
two new production mechanisms precisely, one failing test each.

## Preserved

`BLOCKER-1` and `HIGH-3` remain closed and are re-exercised by
`test_completed_quarantine_recovery_owns_the_request_budget`,
`test_dropping_the_recovery_handoff_publishes_an_expired_retry` and
`test_publication_root_proof_is_atomically_single_use`, all green. Every earlier upload property —
one request-owned sealed snapshot, the 1,201-commit 409, the deadline fences, the streamed canonical
encoder, descriptor bounds, exact response bytes, nonce/idempotency — is unchanged.

## Gates

| Command | Result |
|---|---|
| `py_compile codefire tests/test_python_http_upload_accounted_traversal.py tests/test_python_sealed_production.py tests/test_codefire_cli.py` | PASS |
| `unittest tests.test_python_http_upload_accounted_traversal` | 31/31 PASS, 86.813s |
| `unittest` ancestry / signature / signature-terminal-freeze / sealed-production / external-production | 59/59 PASS, 27.069s |
| `unittest discover -s tests -p 'test_python_*.py'` | 452/452 PASS, 283.579s |
| `check_source_docs.py` | PASS — 44 modules, 34 commands, 12 object kinds |
| `check_project_docs.py` | PASS |
| `git diff --check` | PASS |

No Cargo/Rust, Docker, install, dogfood, or repository-wide validation was run in this lane.

## Request

Fresh independent exact-candidate rereview. No acceptance, integration, issue closure, capability
completion, or zero-residual claim is made from this implementer evidence.
