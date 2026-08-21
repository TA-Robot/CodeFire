# AI Task Packet

Status: completed

Packet ID: MS-agent-local-contract-replay

Schema version: human-v1

Risk score: 5

Reasoning tier: high

## Objective

As the same source-blind Claude session that found the contract blocker, prove the corrected public contract by taking one new bounded Algorithm change through `status -> edit -> focused test -> scan -> extinguish -> verify -> commit -> status` without reading CodeFire product source or receiving a private command correction.

This is node 5 of active M1 `MS-agent-local-contract`. The implementation session must not perform this replay.

## Admitted runner

Use the public CLI name exactly as emitted by action argv:

```text
PATH prefix: /workspace/worktrees/ms-agent-local-contract-seam-20260821/target/release
command: codefire
codefire hardlink: /workspace/worktrees/ms-agent-local-contract-seam-20260821/target/release/codefire
provenance binary: /workspace/worktrees/ms-agent-local-contract-seam-20260821/target/release/codefire-rs
binary sha256: 207fae943c09c2f739611e37050cbc83f34d3b53318fcefaf9d7304ba03d708b
build HEAD: 4d0f755dfb6f7fc5421dfd27cd66f90adebd4396
Rust input manifest: 95ee643bfdc6d62bb1aeaf7c304c74fb4978be089e190de3b3e6debfe37f2066 (99 files)
provenance: /workspace/worktrees/ms-agent-local-contract-seam-20260821/target/codefire-ai/rust-build-provenance.json
provenance sha256: 3d4cc3a1ae06515d63b17facdf3d89dacab9dd900fd7bf2b969e58a7885a2feb
toolchain: cargo 1.97.0 / rustc 1.97.0
```

The hardlink and `codefire-rs` are the same inode and bytes. Prepend the directory to `PATH`; do not substitute another executable for action argv element 0. Stop if any identity above differs.

## Bounded dogfood change

Canonical open directory:

```text
/workspace/project/subprojects/algorithm-evolution-agent-lab
```

Two currently clean tracked files and their admitted pre-edit SHA-256 values:

```text
src/evoagent/statistics.py  8488cfd9656e8bf4047adebac0d5fb906438f6517949b0ef1228d9639c353994
tests/test_statistics.py    4844701e33489d670e324fcd27af97a73226317eb9d793ebf3eec767bbcec542
```

Implement one behavior: reject non-finite confidence-interval inputs instead of returning a NaN/inf report.

1. Immediately after `numeric_values = tuple(float(value) for value in values)`, add a guard using the already imported `math.isfinite`; raise `ValueError("values must be finite")` when any converted value is non-finite.
2. Add one focused unittest method to `ConfidenceIntervalEstimatorTests` that checks `nan`, positive infinity, and negative infinity with `subTest` and `assertRaisesRegex(ValueError, "values must be finite")`.

Caller-supplied action values:

```text
resolution: addressed
rationale: Reject non-finite confidence interval inputs
commit message: Reject non-finite confidence inputs
```

These are task inputs, not command syntax. Obtain flags, ordering, path binding, ready/parameterized state, and argv structure only from help/envelopes/action execution metadata.

## Scope

- the two exact dogfood files above;
- the existing CodeFire state under `/workspace/project/subprojects/.codefire` and marker as written by the named loop;
- this packet for the evidence Result.

Every other `/workspace/project` path is preservation-only. The already dirty `src/evoagent/external_validity.py` is the prior sealed change and must remain untouched.

## Non-Goals

- reading `crates/**`, the Python CodeFire implementation, or CodeFire tests;
- editing CodeFire product/schema/docs or repairing the action contract;
- using the compatibility `command` string as a shell command when `execution` exists;
- changing CFB-133, CFR-169, any other issue, or any later milestone;
- full Algorithm tests, Retrospect, remote commands, install, build, or workspace Rust gates;
- cleaning/staging/committing the surrounding `/workspace/project` Git tree.

## Procedure

1. Preflight runner/provenance hashes, the two clean-file hashes/status, `/workspace/project` HEAD/status counts, and the CodeFire marker/control-file hashes. Require `codefire status --path <dogfood> --json` to report `open-clean`, Fire 0, and base `CF-COMMIT-373468dbaf827a6cc2bae9a4` before the edit. Do not run `open`.
2. Apply only the bounded edit above.
3. Run the focused test:

   ```bash
   PYTHONPATH=src python3 -m unittest tests.test_statistics -v
   ```

4. Discover and run `scan --json` from public help. Require at least one real `extinguish_fire` action so the repaired boundary is exercised.
5. For every real scan-reported Fire, consume that Fire's `execution` object, not its `command` string:
   - require `schema=codefire.action_execution.v1` and `ready=false`;
   - materialize `argv_template` by filling `resolution=addressed` and adding the rationale input using the published `required_inputs[].cli_flag` and `required_input_groups` contract;
   - execute the resulting argv directly with no shell parsing and no path/flag supplied from this packet;
   - require each extinguish result's `verify` action to be `ready=true` with a complete argv containing the canonical `--path`.
6. After all real Fires are resolved, execute the last returned ready verify argv directly. Require a passed decision and a `commit` action with `ready=false`, a path-bound `argv_template`, and required input `message`.
7. Fill only the caller-authored message value above through the action's published input contract and execute the resulting argv directly.
8. Run public `status --json` after commit. Record the sealed identity, require `open-clean` / Fire 0, and compare preservation hashes/status. Only the two admitted dogfood paths may be newly dirty in `/workspace/project`.
9. Update only this packet with full evidence and commit it on the agent branch.

If an execution object is absent, claims the wrong readiness, loses the path, cannot be materialized from its own input metadata, or produces a loop after success, stop and record `block-m1`. Do not repair it.

## Validation

The required validation is exactly the identity preflight, focused 3-case module test, CodeFire action-driven loop, post-status/preservation comparison, plus:

```bash
python3 docs/development/tools/check_project_docs.py
git diff --check
```

No wider gate is allowed.

## Handoff

- runner/provenance/source identity;
- pre-state envelope and preservation baseline;
- exact two-file edit and focused test count;
- scan Fire IDs and the structured actions actually consumed;
- materialized argv arrays (not just shell strings) for extinguish, verify, and commit;
- readiness/input/path assertions;
- sealed commit identity and post-state;
- preservation result and any `block-m1`;
- next legal action: independent other-family audit only.

## Result

Status **completed**. The corrected public contract carried one new bounded Algorithm change from `status` to a sealed commit with **no** `block-m1`, no product source read, and no private command correction. Sealed identity: **`CF-COMMIT-2792b54ad730e6d644668c38`**, parent `CF-COMMIT-373468dbaf827a6cc2bae9a4`.

Run by: Claude Opus 5, the same source-blind session that recorded the node 1 blocker, 2026-08-21, worktree `m1-contract-probe-20260821` at `7df5655` (integration merge containing the focused fix `4d0f755`). The implementation session did not perform this replay.

### Source-blind discipline

`crates/**`, the Python CodeFire implementation, and CodeFire tests were not read in this session, before or after the fix. Every flag, ordering, path binding, readiness state, and argv structure came from `codefire <cmd> --help` and from `execution` metadata inside the live envelopes. The only product files read were the two admitted Algorithm dogfood files, which are the change target, not CodeFire source. The three caller-supplied values (`addressed`, the rationale, the commit message) were treated as task inputs and injected only through each action's published input contract.

Compatibility `command` strings were never executed as shell commands. Every mutation ran as an argv array through `subprocess.run(argv, shell=False)`. The materializer filled `argv_template` placeholders and appended flags **only** from that action's own `required_inputs[].cli_flag` and `required_input_groups`; it supplied no path and no flag from this packet.

### 1. Identity preflight — all match, nothing stopped

```text
PATH prefix: /workspace/worktrees/ms-agent-local-contract-seam-20260821/target/release
command -v codefire -> <PATH prefix>/codefire
codefire            inode 1331446, 3 links, 14603344 bytes, sha256 207fae943c09c2f739611e37050cbc83f34d3b53318fcefaf9d7304ba03d708b
codefire-rs         inode 1331446 (same inode, same bytes), sha256 207fae943c09c2f739611e37050cbc83f34d3b53318fcefaf9d7304ba03d708b
provenance sha256   3d4cc3a1ae06515d63b17facdf3d89dacab9dd900fd7bf2b969e58a7885a2feb
provenance content  base_revision 4d0f755dfb6f7fc5421dfd27cd66f90adebd4396, ok true,
                    source_manifest_sha256 95ee643bfdc6d62bb1aeaf7c304c74fb4978be089e190de3b3e6debfe37f2066, source_file_count 99,
                    cargo 1.97.0 (c980f4866 2026-06-30), rustc 1.97.0 (2d8144b78 2026-07-07),
                    build_command "cargo build --release --locked -p codefire-cli --bin codefire-rs"
```

Every value equals the packet's admitted identity. `argv` element 0 was `codefire` resolved from the prepended PATH; no executable was substituted. No build and no install was run.

Pre-edit target files, both clean in Git and matching their admitted hashes:

```text
src/evoagent/statistics.py  8488cfd9656e8bf4047adebac0d5fb906438f6517949b0ef1228d9639c353994  (admitted match)
tests/test_statistics.py    4844701e33489d670e324fcd27af97a73226317eb9d793ebf3eec767bbcec542  (admitted match)
```

Preservation baseline in `/workspace/project`: `HEAD=f1644122134d479333cf422fdb1d48b8fa92dd8e`, porcelain 246 lines (`114 M`, `132 ??`). The prior sealed change `src/evoagent/external_validity.py` was already dirty at sha256 `873aa601542dec4337f17d4f05779d86e26ebc36ae286dc76cc2d420fe10c2e7`.

CodeFire control-file baseline:

```text
.codefire-open  60b133b3eeefcc79e20bda32843261897713b0d6e86b2f0d0111cd19862690d6
repo.json       bf6d978b0eece0f185c6995630339326f56513a0c8645af3e7e984ad69a7f054
opened/main     f217d0cfabb97e3ed58938e0931decde41a85a65e8cdbdd1f715938f7e038c5c
branches/main   7781697ea2a08a41f8bae9a9cf2217f154261d67e1fa4f94653f410a519fa339
active/open_4c320ed42560  contains only state.json
```

Required pre-state, from `codefire status --path <dogfood> --json` (exit 0):

```text
branch main | state open-clean | base CF-COMMIT-373468dbaf827a6cc2bae9a4 | open_fires 0 | next_actions []
```

`open` was **not** invoked.

### 2. Exact two-file edit

`src/evoagent/statistics.py`, guard placed immediately after the `numeric_values` conversion, using the already imported `math.isfinite`:

```diff
         numeric_values = tuple(float(value) for value in values)
+        if not all(math.isfinite(value) for value in numeric_values):
+            raise ValueError("values must be finite")
         mean = sum(numeric_values) / len(numeric_values)
```

`tests/test_statistics.py`, one focused method added to `ConfidenceIntervalEstimatorTests`:

```diff
+    def test_confidence_interval_estimator_rejects_non_finite_values(self):
+        for label, value in (("nan", float("nan")), ("inf", float("inf")), ("-inf", float("-inf"))):
+            with self.subTest(value=label):
+                with self.assertRaisesRegex(ValueError, "values must be finite"):
+                    ConfidenceIntervalEstimator().estimate([0.80, 0.82, value])
```

Post-edit hashes: `statistics.py` `74b674cb4966b0237a2517b4f897b5cece6670ae515665ef3fb606edf5208dce`, `tests/test_statistics.py` `6af9eda817af0732d8559ecfbda4ac2e1a2ba4c0890eb60d62a73649835f7d88`. Combined diff is 8 insertions, 0 deletions, 2 files. No other file was touched.

### 3. Focused test — 3 tests, 3 non-finite subcases

Command from the dogfood directory: `PYTHONPATH=src python3 -m unittest tests.test_statistics -v`, exit 0.

```text
test_confidence_interval_estimator_rejects_non_finite_values ... ok
test_confidence_interval_estimator_single_value_has_no_width ... ok
test_confidence_interval_estimator_summarizes_repeated_runs ... ok

Ran 3 tests in 0.001s

OK
```

The new method covers `nan`, `+inf`, and `-inf` as three `subTest` cases against `assertRaisesRegex(ValueError, "values must be finite")`.

### 4. Scan — two real Fires, boundary exercised

`scan --help` gave the flags (`[path|--path <path>] [--json] [--metrics] [--full] ...`). `codefire scan --path <dogfood> --json` exited 0 with `branch_state: open-burning`, `changed_atom_count: 4`, `open_fire_count: 2`, `non_atom_changed_files: []`.

```text
FIRE-26974231BF82  fire_sha256_26974231bf82b6232ee43e7e79c29768  atom_changed  CODE-ConfidenceIntervalEstimator -> REQ-SOTA-004  required
FIRE-F530E483B5AF  fire_sha256_f530e483b5af5bb31e68f1a5e9faf9ed  atom_changed  TEST-confidence-interval-estimator-summarizes-repeated-runs -> REQ-SOTA-004  required
```

Changed atoms: `CODE-ConfidenceIntervalEstimator`, `CODE:src/evoagent/statistics.py::method:ConfidenceIntervalEstimator.estimate`, `TEST-confidence-interval-estimator-summarizes-repeated-runs`, `TEST:tests/test_statistics.py::method:ConfidenceIntervalEstimatorTests.test_confidence_interval_estimator_rejects_non_finite_values`.

Action ids returned: `context_changed`, `verify`, `extinguish_batch_template`, `context_fire`, `extinguish_fire`, `context_fire`, `extinguish_fire`. Two real `extinguish_fire` actions, one per Fire, so the repaired boundary was genuinely exercised.

### 5. Extinguish — structured execution consumed, not the string

Both `extinguish_fire` actions carried an identical execution contract, asserted before use:

```text
schema = codefire.action_execution.v1   (asserted)
ready  = false                          (asserted)
argv_template contains the canonical --path /workspace/project/subprojects/algorithm-evolution-agent-lab
```

`argv_template` for `FIRE-26974231BF82`, verbatim, with the typed placeholder object rather than a text token:

```json
["codefire","extinguish","FIRE-26974231BF82","--resolution",{"input":"resolution"},
 "--path","/workspace/project/subprojects/algorithm-evolution-agent-lab","--json"]
```

Published input contract used for materialization:

```json
"required_inputs": [
  {"name":"resolution","cli_flag":"--resolution","required":true,"value_kind":"string",
   "accepts":"extensible_string","cli_default":"addressed","examples":["addressed","changed","no-change-required"],
   "notes":"resolution type is an extensible string; the CLI does not constrain it to a closed enum. If the flag is omitted the parser defaults to addressed. no-change-required additionally requires --rationale when policy requires it."},
  {"name":"rationale","cli_flag":"--rationale","group":"rationale_or_evidence","required":false,"value_kind":"string"},
  {"name":"evidence","cli_flag":"--evidence","group":"rationale_or_evidence","required":false,"value_kind":"string"},
  {"name":"evidence_ref","cli_flag":"--evidence-ref","group":"rationale_or_evidence","required":false,"value_kind":"string"}
],
"required_input_groups": [
  {"name":"rationale_or_evidence","min_selected":1,
   "notes":"supply at least one of --rationale, --evidence, or --evidence-ref"}
]
```

This resolves the node 1 first-failure directly. `resolution` is no longer an unexplained `<type>`: it is a named input with a `cli_flag`, a declared `cli_default`, examples, and an explicit `accepts: extensible_string` disclosure that the value space is deliberately open rather than a closed enum. Note also that the compatibility `command` string for this action (`codefire extinguish FIRE-... --resolution <resolution> --path ... --json`) omits any rationale/evidence flag, so per the action's own `min_selected: 1` group it would be insufficient on its own; the group contract is load-bearing, which is exactly why the `execution` object rather than the string was consumed.

**Materialized argv arrays actually executed** (`shell=False`, argv element 0 `codefire` from PATH):

```json
["codefire","extinguish","FIRE-26974231BF82","--resolution","addressed",
 "--path","/workspace/project/subprojects/algorithm-evolution-agent-lab","--json",
 "--rationale","Reject non-finite confidence interval inputs"]
```

```json
["codefire","extinguish","FIRE-F530E483B5AF","--resolution","addressed",
 "--path","/workspace/project/subprojects/algorithm-evolution-agent-lab","--json",
 "--rationale","Reject non-finite confidence interval inputs"]
```

Both exited 0, `ok: true`, `data.type: codefire_plan_result`, `dry_run: false`, `would_apply: true`.

```text
FIRE-26974231BF82 -> resolution_type addressed, has_rationale true, resolution_uid res_6d00f3e2c399
                     operations: update_fire_status(extinguished) on active/open_4c320ed42560/fires.json,
                                 append_resolution(res_6d00f3e2c399) on active/open_4c320ed42560/resolutions.json
FIRE-F530E483B5AF -> resolution_type addressed, has_rationale true, resolution_uid res_fa13d33c97a1
                     operations: update_fire_status(extinguished) on active/open_4c320ed42560/fires.json,
                                 append_resolution(res_fa13d33c97a1) on active/open_4c320ed42560/resolutions.json
```

Each extinguish returned exactly one action, `verify`, and both satisfied the required assertions:

```json
{"schema":"codefire.action_execution.v1","ready":true,
 "argv":["codefire","verify","--details","--json","--path","/workspace/project/subprojects/algorithm-evolution-agent-lab"]}
```

`ready=true` actions publish a complete `argv` array rather than an `argv_template`, and the canonical `--path` is present. This is the exact node 1 second failure — where the pre-fix envelope returned `codefire verify --details --json` with `target: {"branch":"main"}` and no path anywhere — now closed.

### 6. Verify — executed the last returned ready argv

Executed directly, no substitution:

```json
["codefire","verify","--details","--json","--path","/workspace/project/subprojects/algorithm-evolution-agent-lab"]
```

Exit 0, `ok: true`.

```text
result: passed
decision.outcome: passed | primary_reason: null | blocking_reasons: [] | warning_reasons: ["missing_required_links"]
open_required_fires: 0 | stale_resolutions: [] | failed_checks: [] | unresolved_merge_conflicts: []
trace_completeness_required: false
change_coverage_digest:    5f9101407a555cc65df357279601458eabe6279a005f1dd70023cdc0150e8549
effective_policy_digest:   12321a00d6c74b8318e44842095b33902d4fb41e06b57cec30fa5b858d54becb
coverage summary: changed_files 2, atom_files_with_unclassified_spans 2, non_atom_files 0, out_of_atom_files 0
  src/evoagent/statistics.py  modified  atom_change_with_unclassified_spans  CF-BLOB-825689d99a4e36b10c856673 -> CF-BLOB-aad1cdaebdfb5f692aee230c
  tests/test_statistics.py    modified  atom_change_with_unclassified_spans  CF-BLOB-bdbac131deec504e250bf09d -> CF-BLOB-c031a8ee338cb20e1e5fe60e
execution[unit-tests]: exit_code 0, success true, timed_out false, subject_unchanged true,
  profile trusted_local_shell_v1, phase verification_command, trust_kind local_unisolated,
  worktree_before == worktree_after == sha256:b5f385bbce74e784b128981f1972dd8a8742601c3b8baa1a32384e6281da9cb4,
  input_digest sha256:718953e33233121ca4953f4d45fc23edd65a86ec1a02899dc6d1b6507e1162e1,
  environment_digest sha256:7d706eb8f149e3831db22b2122e566169136e6ada0c5f3570d51ba2b49ab6e88
subject: live_open, base CF-COMMIT-373468dbaf827a6cc2bae9a4, branch main, open_instance_id open_4c320ed42560,
  repository_id repo_8a9e9ea836af, subject_digest sha256:7e1e6107cbcbce531b7e7378fb82f905c28f1d248dd6f60a46bf05ebd5b2ed15
```

The seven `missing_required_links` warnings (`DES-AUTO-003`, `DES-AUTO-007`, `DES-PRODUCT-002`, `DES-PRODUCT-006`, `DES-SOTA-001`, `DES-SOTA-003`, `DES-SOTA-004`) are non-blocking under `trace_completeness_required: false`, unchanged from the prior loop and unrelated to this change.

Verify returned exactly one action, `commit`, meeting every required property: `ready: false`, path-bound `argv_template`, and required input `message`.

```json
{"schema":"codefire.action_execution.v1","ready":false,
 "argv_template":["codefire","commit","-m",{"input":"message"},
                  "--path","/workspace/project/subprojects/algorithm-evolution-agent-lab","--json"],
 "required_inputs":[{"name":"message","cli_flag":"-m","required":true,"value_kind":"string",
                     "notes":"caller-authored commit message"}]}
```

### 7. Commit — caller-authored message only

Materialized argv executed directly:

```json
["codefire","commit","-m","Reject non-finite confidence inputs",
 "--path","/workspace/project/subprojects/algorithm-evolution-agent-lab","--json"]
```

Exit 0, `ok: true`, `data.type: codefire_commit_result`.

```text
commit_id: CF-COMMIT-2792b54ad730e6d644668c38
applied: true | blocked: false | branch main | parents ["CF-COMMIT-373468dbaf827a6cc2bae9a4"]
message: "Reject non-finite confidence inputs"
changed_atom_count 4 | extinguished_fire_count 2 | active_resolution_count 2 | non_atom_changed_file_count 0
verification: result passed, open_required_fires 0, stale_resolutions 0, failed_checks 0,
              duplicate_atom_ids 0, missing_required_links 7, primary_reason null
change_coverage digest 5f9101407a555cc65df357279601458eabe6279a005f1dd70023cdc0150e8549 (identical to verify)
objects: blob, content_manifest, atom_index, trace_graph, fire_ledger, resolution_ledger, verification, policy, commit, branch
operations: begin_commit_transaction, store_objects, update_branch_head, update_open_registry,
            reset_active_state, clear_commit_transaction
transaction: marker active/open_4c320ed42560/commit_transaction.json, cleanup removed_after_success
```

`next_actions: []`. No action was returned after success.

### 8. Post-state, no loop, preservation

`codefire status --path <dogfood> --json`, exit 0:

```text
branch main | state open-clean | base CF-COMMIT-2792b54ad730e6d644668c38 | open_fires 0
scan_prediction: changed_count 0, open_fire_count 0
next_actions: []
```

Loop termination is machine-checkable at both ends: `commit` returned `next_actions: []` and the post-commit `status` returned `next_actions: []`. No action re-proposed consumed work, so no cycle after success.

Control files after the loop:

```text
.codefire-open  60b133b3eeefcc79e20bda32843261897713b0d6e86b2f0d0111cd19862690d6  (unchanged)
repo.json       bf6d978b0eece0f185c6995630339326f56513a0c8645af3e7e984ad69a7f054  (unchanged)
opened/main     5721d710f4371db88be26567d2a3c6afb3bc2458288b8c812b54ccd7556a42b2  (changed by the loop; state.last_known open-clean)
branches/main   bfd52e3525f6a6faf0db4f3d2d1d9d7fa3fa0d400f6561c3c5b8227e79e989f8  (changed by the loop; head now CF-COMMIT-2792b54ad730e6d644668c38)
active/open_4c320ed42560  contains only state.json ({"state":"open-clean"}) after reset_active_state
```

`/workspace/project` Git preservation:

```text
HEAD f1644122134d479333cf422fdb1d48b8fa92dd8e  (unchanged)
porcelain 248 lines (116 M, 132 ??) versus baseline 246 (114 M, 132 ??)
delta = exactly the two admitted dogfood paths:
   M subprojects/algorithm-evolution-agent-lab/src/evoagent/statistics.py
   M subprojects/algorithm-evolution-agent-lab/tests/test_statistics.py
prior sealed change untouched:
   M subprojects/algorithm-evolution-agent-lab/src/evoagent/external_validity.py
   sha256 873aa601542dec4337f17d4f05779d86e26ebc36ae286dc76cc2d420fe10c2e7  (identical to its node 3 post-edit value)
```

Nothing was staged, committed, cleaned, or reverted in `/workspace/project`. No untracked file count changed. No CodeFire product, schema, or doc file was edited, and no contract gap was repaired by this session.

### block-m1

**None.** No execution object was absent, none claimed the wrong readiness, none lost the path, all were materializable from their own input metadata, and no action produced a loop after success. Every one of the packet's stop conditions was checked as an explicit assertion rather than by inspection.

### Contract verdict against the node 1 findings

| Node 1 finding | State in this replay |
|---|---|
| `scan` → `extinguish` action was a placeholder string with no published value vocabulary | Closed. Typed `argv_template` with `{"input":"resolution"}`, plus `cli_flag`, `cli_default: addressed`, examples, and an explicit `accepts: extensible_string` disclosure. |
| `extinguish` → `verify` action dropped the required `--path` and carried no path in `target` | Closed. `ready: true` with a complete `argv` array containing the canonical `--path`. |
| No action declared readiness, argv form, or an input contract; schema typed `next_actions` as bare objects | Closed on this path. Every consumed action declares `schema: codefire.action_execution.v1`, a boolean `ready`, and either a complete `argv` or an `argv_template` plus `required_inputs` / `required_input_groups`. |
| `verify` → `commit` placeholder message | Correctly modelled as a caller-authored `required_inputs` entry with `cli_flag: -m`, not as an unexplained placeholder. |

The M1 exit condition this node exists to prove now holds for the local loop: a fresh worker chose every transition from help or from machine-readable actions with complete required arguments, read no CodeFire product source, and received no private command correction.

### Handoff

- runner/provenance/source identity: all admitted values matched exactly; single inode for `codefire` and `codefire-rs`; build `4d0f755`, manifest `95ee643b…` over 99 files;
- pre-state: `open-clean`, Fire 0, base `CF-COMMIT-373468dbaf827a6cc2bae9a4`; `open` not run;
- edit: two files, 8 insertions, hashes recorded above; focused test 3 tests / 3 non-finite subcases, exit 0;
- Fires: `FIRE-26974231BF82` and `FIRE-F530E483B5AF`, both `atom_changed` → `REQ-SOTA-004`, both resolved `addressed` with rationale;
- materialized argv arrays for both extinguishes, verify, and commit are recorded verbatim above; all executed with `shell=False`;
- readiness/input/path assertions: `ready=false` on both extinguish templates and on commit, `ready=true` with canonical `--path` on every verify action, all asserted programmatically;
- sealed identity `CF-COMMIT-2792b54ad730e6d644668c38`; post-state `open-clean`, Fire 0, `next_actions: []`;
- preservation: HEAD unchanged, only the two admitted paths newly dirty, prior sealed change byte-identical;
- no `block-m1`;
- next legal action: independent other-family audit of this replay only. This packet does not write the board, `history.md`, any issue file, or any other packet, and M1 does not exit until that audit reproduces the path.
