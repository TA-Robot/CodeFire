# CFR-168 Python HTTP Doctor + Branch Composition Re-review

Review date: 2026-07-30

Reviewer: Fugu 1.1 Ultra (`fugu-ultra-v1.1`)

Review worktree base: `22927cece7d48a90581c747d63f526114abddebd`

**Verdict: `ACCEPT` for the corrected Doctor + Branch composition only.**

No new blocking finding was found. Prior `MUST-1` is closed exactly at
`701469f91dd0c4c52c5628fb3e9d9e9d0ab0c714`; the six `codefire` conflict blocks use the required
combined `read_route_phase` resolution; and the exact-target focused, neighboring, compilation,
documentation, and diff-hygiene gates pass. This verdict neither accepts nor rejects the adjacent
upload slice from `0703f71b9800ff19ce52d01bcd46c8eddb323df2`; that surface remains under its separate review and
CFB-138 gate.

The completed 463-case evidence is the Python-pattern command, not unrestricted discovery. The
manager's exact-target unrestricted `python3 -m unittest discover -s tests -v` run was still active
when this report was finalized and remains a mandatory integration gate. That pending broader gate
does not reopen a Doctor/Branch finding in the reviewed corrected composition.

## Exact subject

| Role | Revision | Disposition |
|---|---|---|
| Pre-composition management/product base | `16d012938730fdb2b092b7f8c5cd601b53b81a1f` | comparison base |
| Doctor composition commit | `5e3b0c769f84e10a57592c3c9bf229e61cd37ebe` | reviewed |
| Adjacent upload composition commit | `0703f71b9800ff19ce52d01bcd46c8eddb323df2` | composition neighbor only; outside verdict |
| Branch composition/conflict-resolution commit | `7e87665b8158a37f1d25b449de7831cf9420f545` | reviewed |
| Mandatory canonical test correction / exact target | `701469f91dd0c4c52c5628fb3e9d9e9d0ab0c714` | reviewed and accepted for Doctor + Branch |
| Prior independent review, integrated copy | `4e651a79fcf22980be7ecdbb64c576c3941aa5a1` | prior `REQUEST_CHANGES`; `MUST-1` rechecked here |

The source reviewer commit `8b395b0a3ae3436e76fc5b300733ed75cef790bc` contains the identical
prior-review report blob.

The composition chain is linear:

```text
16d012938730fdb2b092b7f8c5cd601b53b81a1f
  -> 5e3b0c769f84e10a57592c3c9bf229e61cd37ebe
     -> 0703f71b9800ff19ce52d01bcd46c8eddb323df2
        -> 7e87665b8158a37f1d25b449de7831cf9420f545
           -> 701469f91dd0c4c52c5628fb3e9d9e9d0ab0c714
```

`codefire` is byte-identical between `7e87665b...` and `701469f...`; the final commit changes only
the canonical CLI expectation. The Doctor source/test blobs in `5e3b0c7...` are byte-identical to
the independently reviewed completed Doctor target `01a7c862fb9825878e580229d1b265ae3a48115a`.

## Finding disposition

### Prior `MUST-1` — `CLOSED EXACTLY`

The corrected method
`CodeFireCliTests.test_python_http_bundle_deadline_boundaries_and_poll_phases` now asserts all required
properties for an already-expired branch-list request:

- wire starts with `HTTP/1.0 503 Service Unavailable`;
- no `HTTP/1.0 200 OK` appears;
- payload `kind` is `request_deadline_exceeded` and `exit_code` is `60`;
- first diagnostic phase is `branch list planning`;
- deadline `poll_count` is exactly `1`;
- terminal reason is `deadline_exceeded`;
- `open_permit_count` is `0` and `assert_http_work_closed` proves zero current debt.

The method passes on the exact target. Production reaches this result at the first
`handle_http_list` deadline poll, before project lookup, permit acquisition, or response publication.

### New findings — `NONE`

No new Must, Should, or Could finding was identified in the Doctor/Branch corrected composition.

## Conflict-resolution verdict

**`PASS`.** The six prior-review conflict surfaces are resolved with the required combined shape:

1. `read_route_phase(suffix)` preserves exact precedence and labels: bundle, Doctor, Branch list.
2. Response-body freezing uses that helper and preserves the upload fallback
   `upload response encoding`.
3. Pre-encoding polling derives `encoding_phase` from the combined helper.
4. Pre-header polling derives and checks `header_phase` for every active read route.
5. Pre-publication polling derives and checks `publication_phase`, preventing late success wire.
6. `handle_one_request` resets `_bundle_deadline_active`, `_doctor_deadline_active`,
   `_list_deadline_active`, and `_mutation_deadline_active` to `False` for every request.

The route arms then activate only their own flag and pass the handler's exact
`self.decision_work`/`self.request_deadline`. `AccountedHTTPBranchList` remains an accepted bounded
freezer mapping type. Representative bundle and upload-mutation response-owner tests pass, so the
resolution did not silently replace the pre-existing bundle or mutation response behavior.

## Composition preservation

**`PASS`.** Static inspection plus the dedicated suites found no dropped Doctor or Branch contract:

- Doctor keeps the supplied request ledger rather than creating a hidden second ledger, unwinds only
  Doctor-owned deltas, preserves direct compatibility, and carries deadlines through route,
  authority/object work, diagnostics, encoding, headers, and publication.
- Doctor corrupted paths are projected to strings before accounting/publication; quick/full domain
  output and diagnostic precedence remain covered.
- Branch keeps one request ledger and one `SealedCommitValidationSession`, sorted authority capture,
  shared ancestry validation, projection ownership, exact/minus-one limits, deterministic deadline
  phases, and final `session.finalize(terminate=False)` snapshot revalidation.
- Branch failures remain typed and atomic for absent/empty/many authority, malformed records,
  invalid sealed heads, shared-parent corruption, same-metadata content changes, and extension-only
  changes; no partial 200 response or ownership debt is published.
- Bundle behavior and upload mutation-response preflight/cleanup remain green in neighboring tests.

## Commands and results

All runtime commands below were run from a `git archive` snapshot of exact target
`701469f91dd0c4c52c5628fb3e9d9e9d0ab0c714`; no product/test file was modified.

| Command | Result |
|---|---|
| `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tests.test_python_http_doctor_decision_work -v` | **19/19 OK**, 28.814s reported by unittest |
| `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tests.test_python_http_branch_decision_work -v` | **13/13 OK**, 22.331s |
| Corrected `test_python_http_bundle_deadline_boundaries_and_poll_phases` | **1/1 OK**, 2.959s |
| Bundle neighbors: `test_python_http_bundle_uses_one_request_ledger_and_preserves_wire`, `test_python_http_bundle_work_limits_are_atomic` | **2/2 OK** |
| Mutation neighbors: `test_python_upload_response_preflight_precedes_effect_and_retries_durable_result`, `test_python_mutation_response_owner_rejects_foreign_and_retries_cleanup` | **2/2 OK**; four-neighbor group 11.257s |
| `python3 -m py_compile codefire tests/test_python_http_doctor_decision_work.py tests/test_python_http_branch_decision_work.py tests/test_codefire_cli.py` | pass |
| `python3 docs/development/tools/check_source_docs.py` | pass: 44 modules, 34 commands, 12 object kinds |
| `python3 docs/development/tools/check_project_docs.py` | pass: 539 Markdown files inventoried |
| `git diff --check 16d012938730fdb2b092b7f8c5cd601b53b81a1f 5e3b0c769f84e10a57592c3c9bf229e61cd37ebe -- codefire tests/test_python_http_doctor_decision_work.py` | pass |
| `git diff --check 0703f71b9800ff19ce52d01bcd46c8eddb323df2 7e87665b8158a37f1d25b449de7831cf9420f545 -- codefire tests/test_python_http_branch_decision_work.py` | pass |
| `git diff --check 7e87665b8158a37f1d25b449de7831cf9420f545 701469f91dd0c4c52c5628fb3e9d9e9d0ab0c714 -- tests/test_codefire_cli.py` | pass |
| `git diff --check 16d012938730fdb2b092b7f8c5cd601b53b81a1f 701469f91dd0c4c52c5628fb3e9d9e9d0ab0c714` | pass |

## Python-pattern and full-suite evidence

The manager's completed exact-target evidence is sufficient corroboration for the **Python-pattern
gate**:

```text
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tests -p 'test_python_*.py'
701469f91dd0c4c52c5628fb3e9d9e9d0ab0c714
463/463 passed in 296.500 seconds
```

This reviewer did not rerun that 463-case sweep and does not claim it as reviewer-executed. The
revision is the exact reviewed tree and the command is explicit. Local loader inventory independently
confirms that this pattern selects 463 unique cases.

It is **not** the `AGENTS.md` full-suite command: unrestricted
`python3 -m unittest discover -s tests -v` selects 691 cases at this target, including 228 cases in
`test_codefire_cli.py`. At report finalization, the manager's unrestricted run was active with:

- working directory `/workspace/worktrees/cfr168-python-composition-probe`;
- `HEAD == 701469f91dd0c4c52c5628fb3e9d9e9d0ab0c714` and a clean worktree;
- command `python3 -m unittest discover -s tests -v`;
- configured Cargo/Rustup environment rooted at `/tmp/codefire-rustup`.

Its result therefore remains a mandatory pre-integration gate and must not be inferred from the
463-case result. The manager owns that broader gate; this report does not claim it completed.

## Residual risk

1. Unrestricted exact-target Python discovery was still running when this review finalized. Any
   failure must block integration and be triaged before the manager lands the composition.
2. The adjacent upload product slice remains outside this verdict and still requires its independent
   review plus CFB-138 acceptance; the neighboring mutation checks here are composition-preservation
   evidence only.
3. This verdict is limited to the exact Doctor + Branch corrected composition at `701469f...`; it
   does not authorize unrelated CFR-168 surfaces or integration into main.

## Next action

Manager: wait for the active unrestricted exact-target Python suite, require a green result, then
integrate only the reviewed commits in the serialized order while preserving the exact combined
`read_route_phase` resolution. Keep upload acceptance governed by its separate review/CFB-138 gate.
