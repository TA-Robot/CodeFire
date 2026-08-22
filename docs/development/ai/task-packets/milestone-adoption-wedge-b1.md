# AI Task Packet

Status: completed

Packet ID: MS-adoption-wedge-b1

Schema version: human-v1

Risk score: 6

Reasoning tier: high

## Objective

Classify the exact smallest public-contract correction justified by the first adoption experiment's
`narrow` decision: document and machine-bind only the existing `branch list --json` and `show --json`
payload facts the source-blind consumer relied on, without implementing the full CFR-171 descriptor.

## Scope

- integration evidence in `milestone-adoption-wedge-run.md` and preserved consumer/controller roots;
- current `branch list` / `show` parse, validation, typed data, render, capabilities, public schema,
  docs, and focused tests;
- this packet's Result only.

This is a read-only classification. Do not implement, edit another packet, run a consumer retry, or
mutate any repository.

## Questions

1. What exact current payload shapes/versions/types/fields did Q1–Q7 cite, and which are already typed
   versus ad-hoc JSON?
2. What is the smallest backward-compatible way for public schemas/capabilities to bind the two
   command-specific `data` shapes while retaining the shared command-result envelope?
3. Which fields are required for repo/branch/head/base/claim/missing-authority comprehension, and which
   unrelated branch/show fields must remain outside the admitted guarantee?
4. What exact product/docs/schema/test write set and focused validations prove the contract, including
   unknown/additional-field/version behavior?
5. Does any Store/object/certificate persistence change become necessary? If yes, classify `block-m1`
   and stop; the admitted slice is intended to be projection/schema/docs only.

## Validation

Set Status `completed` or `blocked`, classify findings `block-m1` / `park` / `drop`, and run:

```bash
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_source_docs.py
git diff --check
```

## Non-Goals

- full CFR-171 lossless cross-surface descriptor, persisted format, Store validator, or tamper corpus;
- changing actual claim semantics, branch state, commit objects, certificate authority, or command
  behavior beyond machine-readable contract binding;
- Python/HTTP parity, toolchain acquisition, review-pack/import, Git bridge, telemetry, agent platform;
- rerunning or rescoring the consumer before a fix is integrated;
- CFR-168/CFB-136 or M2–M5.

## Handoff

Return fixed HEAD, exact current payload map, recommended schema/capability strategy, required and
excluded fields, exact write/test set, compatibility/invariant argument, issue verdict, and next legal
action. Commit only this packet.

## Result

Status **completed**. Classification at exact HEAD
`0d414931c46c3ca89eadf1d46481067b3e573d16` (merge of integration `11d0c8c` as `0d41493`).
**No Store/object/certificate persistence change is necessary.** **No `block-m1` that stops the
slice.** B1 is a projection/schema/docs/capability advertisement of JSON these two commands **already
emit**. Full CFR-171, Python/HTTP, toolchain acquisition, FORMAT_CONTRACTS, and later slots stay
**`park`**. Changing `show.certificate` from a string to an object is **`drop`** (breaking).

This packet did not implement, rerun, rescore, or mutate any repository. First-run evidence is frozen
in `milestone-adoption-wedge-run.md` (score 7/7, `narrow`, `undocumented_field_reliance` 7 / 2
payload families). Write set: this packet only.

Run by: persistent Grok 4.6 node-2 session, 2026-08-22, worktree
`ms-adoption-experiment-20260822`. Public requirement was already fixed by nodes 1–2 and confirmed
by the consumer run; this lane mapped only `branch list` / `show` parse, typed data, render,
capabilities, public schemas, docs, and focused tests.

### 1. Current payload map (Question 1)

Envelope (already bound): `codefire.command_result.v1`. `data` is an unconstrained object.
`capabilities.commands[].output_schema` is that envelope ID for every JSON command, including
`branch` and `show`. Envelope `repo` answered Q1 and is already in the docs pack.

Q1–Q7 citations vs current producers (first-run C.5, 25 cited paths, 7 undocumented):

| Path relied on for a correct answer | Command | Producer | Typed today? |
|---|---|---|---|
| envelope `repo` | C2–C5 | `CommandResultWire` / `command_result_envelope` | yes, envelope schema |
| C2 doctor `data.{type,mode,checked.*,skipped_checks,issues,issue_counts}` | C2/C6 | `doctor::report` | yes, documented; **not B1** |
| C2/C7 `next_actions` (`inspect_legacy_branch_profile`, absent `execution`) | C2/C6 | envelope | yes, documented; **not B1** |
| **C3 `data.branches`** | C3 | `BranchListDataWire` + `BranchesWire` / `BorrowedBranchResult` | **serde-typed wire; no public schema** |
| **C4/C5 `data.show.commit`** | C4/C5 | `view::show_commitish_data` `json!` | **ad-hoc JSON; no public schema** |
| **C4/C5 `data.show.parents`** | C4/C5 | same | ad-hoc |
| **C4/C5 `data.show.certificate`** | C4/C5 | `commit.certificate.result` projected as a **string** | ad-hoc |
| **C4/C5 `data.show.signature`** | C4/C5 | present/summary/details object | ad-hoc |
| **C4/C5 `data.show.signature.present`** | C4/C5 | `bool` | ad-hoc |
| **C4 `data.show.target`** | C4 | argv echo | ad-hoc |

Exact current `data` shapes (do not change them in B1):

**`branch list --json`** — serde `BranchListDataWire` in `main.rs`; envelope `command` is
`branch-list` (hyphen), capabilities `name` is `branch`. Domain type `cli_model::Branch`.

```json
{
  "type": "codefire_branch_list",
  "version": 1,
  "branches": [{"name": "<branch>", "head": "CF-COMMIT-…", "state": "closed|open-clean|…"}]
}
```

Optional `metrics` is omitted unless `--metrics` (`skip_serializing_if = None`). No `opened` array
(that is `codefire_local_list`, a different command). `state` is the branch **record** field; the
fixture had `main.state=open-clean` with `opened=0`. That is a docs hazard, not a missing field.

**`show --json`** — untyped `json!` in `view.rs::show_commitish_data`; envelope `command` is `show`.

```json
{
  "type": "codefire_show_result",
  "version": 1,
  "show": {
    "target": "<argv>",
    "label": "<resolved label>",
    "commit": "CF-COMMIT-…",
    "message": "…",
    "parents": ["CF-COMMIT-…"],
    "files": 0,
    "atoms": 0,
    "certificate": "passed",
    "roots": {},
    "signature": {"present": false, "summary": "(none)", "details": null}
  }
}
```

`certificate` is **only** `commit.certificate.result` (`passed` / `failed` / legacy `consistent` /
fallback `"unknown"`). There is **no** assessment `subject`, `profile`, or `phase` on this surface.
Q5/Q6 composed claim class from `signature.present=false` plus doctor
`legacy_branch_head_semantic_profile` warnings. Adding those claim fields to `show` is **not** B1.

Parse/validate owners (unchanged): `main` branch-list routing + `branch_refs` sealed-head session;
`main::parse_show_args` + `view` + `codefire_store::validate_sealed_commit`. Apply/write: none
(read-only). Render: `BranchListDataWire` / `show_commitish_data` → envelope.

Existing tests that already pin the shapes (do not rerun from this packet):

- `tests.rs::command_capabilities_are_shared_by_help_completion_and_metrics_parser`
- `tests.rs::branch_list_uses_one_session_for_many_head_shared_dag_and_reports_aggregate_work`
- `tests.rs::show_and_diff_local_branches` (`type`, `show.target`, `certificate` string, `signature.present`)
- `tests.rs::branch_output_preflight_accepts_exact_retained_limit_and_rejects_plus_one` (envelope `command=branch-list`)
- `compatibility.rs::support_manifest_advertises_actual_decision_work_profiles_without_overclaiming_coverage`

### 2. Smallest backward-compatible binding (Question 2)

Keep the shared envelope. Do **not** retarget `output_schema` (that ID is the envelope for every JSON
command). Do **not** tighten `command-result-v1` `data` to a `oneOf` (that would reject doctor/status/
verify/…). Do **not** add a `schema` field inside the two payloads (`type`+`version` already identify
them). Do **not** add FORMAT_CONTRACTS rows (these are command `data`, not persisted object formats).

Recommended strategy (additive, v1-compatible):

1. Check in two public **data** schemas:
   - `schemas/public/branch-list-v1.schema.json` — `$id` `codefire.branch_list.v1`
   - `schemas/public/show-result-v1.schema.json` — `$id` `codefire.show_result.v1`
2. Advertise them in `compatibility::support_manifest_json()["schemas"]` (the rule: a schema is not
   advertised until checked in).
3. Add optional `data_schema` on `capabilities.commands[]` (capabilities-v1 currently
   `additionalProperties: false`, so the property must be listed). Emit it **only** for `branch` →
   `codefire.branch_list.v1` and `show` → `codefire.show_result.v1`. Omit on every other command.
4. Document the field lists in `docs/04_cli_spec.md` and `docs/automation_interface.md` (both were in
   the frozen 12-file docs pack; that is what `undocumented_field_reliance` scans).
5. Leave `view.rs` and `BranchListDataWire` bytes unchanged.

Unknown / additional-field / version contract to freeze in those schemas and docs:

| Rule | Binding |
|---|---|
| `type` | const `codefire_branch_list` / `codefire_show_result` |
| `version` | const `1`; consumers using the public schema **deny** any other version (align with `support.unknown_version=deny` for newly advertised schemas; no persisted missing-version legacy case) |
| required fields | only the comprehension set in §3 |
| currently emitted extras (`message`, `files`, `atoms`, `label`, `roots`, `metrics`, `signature.summary`/`details`) | listed as **optional** properties so today’s payloads validate |
| unknown future keys | `additionalProperties: true` on `data`, `show`, `branches[]`, and `signature` (preserve; do not treat command JSON as object-record critical extensions) |
| envelope | unchanged; `data` stays `{"type":"object"}` |

This is the pagination analog: `list` already advertises `pagination.page_schema` without changing
`output_schema`. `data_schema` is the same seam for non-paged commands.

Rejected alternatives: renaming envelope `command` `branch-list` → `branch` (**`drop`**, breaks
current JSON); emitting the full certificate object under `show.certificate` (**`drop`**, string→object
break); stuffing data schema IDs into `output_schema` (**`drop`**, changes the advertised envelope).

### 3. Required vs excluded fields (Question 3)

**Required for repo / branch / head / base / claim / missing-authority on this path** (must be
`required` in the new schemas and named in CLI spec + automation_interface):

| Need | Field | Notes |
|---|---|---|
| repository | envelope `repo` | already schema-bound; not duplicated into branch/show data |
| branch set | `data.branches` | array |
| branch identity | `branches[].name` | |
| head | `branches[].head` **and** `show.commit` | C3 vs C4 |
| base | `show.parents` | array of sealed parent IDs; empty on a root |
| claim result | `show.certificate` | **string** result; `consistent` is the documented legacy alias of `passed` |
| claim target echo | `show.target` | cited on C4; binds which commitish was asked |
| missing signed/portable authority | `show.signature` object with required `present` (boolean) | `false` plus `details: null` is the current unsigned projection |
| payload identity | `data.type`, `data.version` | already emitted |
| branch record state | `branches[].state` | always emitted; **must be documented** so the next consumer does not treat `open-clean` as a live open workspace |

**Excluded from the admitted guarantee** (may remain in the payload; optional in schema; not promised
as claim/proof):

- `show.label`, `show.message`, `show.files`, `show.atoms`, `show.roots`
- `signature.summary`, `signature.details` (optional; do not promote `details` to a standalone proof)
- `data.metrics` on branch list
- assessment `subject` / `profile` / `phase` (not emitted; composing them is **parked** CFR-171)
- doctor semantic-profile warnings (already documented; not a branch/show field)
- `list` / `codefire_local_list` / remote `codefire_remote_branch_list` (different types)
- review-pack, import, verify live data, HTTP twins

Do **not** add `subject`/`profile`/`phase` to `show` in this slice. The consumer already answered Q5/Q6
from `certificate` + `signature.present` + doctor warnings. Binding those existing fields is enough
for a contract-truthful rerun. Adding a lossless descriptor is full CFR-171.

### 4. Exact write/test set (Question 4)

Node 5d only, after 5c admission. No writes from this packet.

| File | Change |
|---|---|
| `schemas/public/branch-list-v1.schema.json` | **new** data schema |
| `schemas/public/show-result-v1.schema.json` | **new** data schema |
| `schemas/public/capabilities-v1.schema.json` | optional `data_schema` on `$defs.command` |
| `crates/codefire-cli/src/compatibility.rs` | `BRANCH_LIST_SCHEMA` / `SHOW_RESULT_SCHEMA` constants; append both to `support.schemas` |
| `crates/codefire-cli/src/command_registry.rs` | emit `data_schema` only for `branch` and `show` |
| `docs/04_cli_spec.md` | field list for `branch list --json` and `show --json`; certificate is the result string; `state` ≠ live open |
| `docs/automation_interface.md` | examples; `data_schema` vs `output_schema`; envelope `command=branch-list` |
| `docs/development/command-source-trace.md` | branch/show/capabilities rows: advertised data schema IDs |
| `docs/development/source-architecture.md` | `command_registry` / `compatibility` one-line advertisement duty |
| `docs/development/source-data-model-catalog.md` | public schema family now includes the two data IDs |

Focused tests (extend existing; do not add workspace-wide gates):

- `command_capabilities_are_shared_by_help_completion_and_metrics_parser` — `branch`/`show`
  `data_schema` set; other commands omit it; `output_schema` remains `codefire.command_result.v1`
- `support_manifest_json` schemas list contains the two new `{id,path}` entries
- `show_and_diff_local_branches` — keep string `certificate` + `signature.present`; assert `version==1`
- branch-list wire test already equality-checks `type`/`version`/`branches{name,head,state}`
- **new assertions in those same tests** (no new crate): (a) version `2` / missing `type` / missing
  `branches` or `show.commit` fail a tiny required-field+const-version check against the checked-in
  schema `required`/`const` nodes; (b) an extra unknown key on `show` or a branch entry is still
  accepted (`additionalProperties: true`); (c) `--metrics` branch-list payload still has the required
  fields

Do **not** add `schemas/conformance/manifest-v1.json` vectors, Python readers, or
`minimal_consumer.py` cases in this slice (ADR-018 corpus expansion is **`park`**; Python/HTTP is an
explicit non-goal). Do **not** edit `view.rs`, `BranchListDataWire`, Store, certificate objects, or
issue/board files.

`python3 docs/development/tools/check_source_docs.py` and `check_project_docs.py` run on the 5d
commit. `cargo test -p codefire-cli --bin codefire-rs command_capabilities_are_shared_by_help_completion_and_metrics_parser show_and_diff_local_branches branch_list_uses_one_session_for_many_head_shared_dag_and_reports_aggregate_work` is the focused binary gate if 5c names cargo.

### 5. Store / persistence (Question 5)

**No.** Certificate agreement, assessment subjects, and sealed commit bytes stay in Store as they
are. `show` already projects `result` as a string; B1 documents that projection. A Store/object
change would be `block-m1` and would stop this slice; it is not required. **`park`** all persisted
format, tamper corpus, and lossless cross-surface descriptor work.

Compatibility / invariant argument: existing v1 readers ignore unknown envelope/`data` keys and
already parse these fields. Advertising schemas and `data_schema` is additive. Required fields are a
subset of fields always emitted at this HEAD. `additionalProperties: true` keeps extras and future
noncritical keys valid. Version remains `1`. No command behavior, no identity, no sealed commit, no
branch-head atomicity change.

### 6. Issue verdicts and next action

| Finding | Class | Why |
|---|---|---|
| B1 document + schema-bind `codefire_branch_list` / `codefire_show_result` + optional `data_schema` | admitted 5d slice after 5c; **not** Store `block-m1` | reproduced by 7 undocumented field paths; JSON already exists |
| Full CFR-171 lossless descriptor / show subject/profile/phase / certificate object | `park` | not emitted; not needed to bind the first-run answers |
| `show.certificate` string → object | `drop` | breaking type change |
| Rename envelope `branch-list` → `branch` | `drop` | breaks current JSON |
| FORMAT_CONTRACTS / conformance corpus / Python / HTTP / toolchain | `park` | packet non-goals; ADR-018 remainder |
| Doctor `open-clean` vs `opened=0` warning | `park` | docs of `state` is enough for the rerun; behavior change is a later UX residual |
| CFR-168 / CFB-136 / CFB-133 / CFR-169 / CFR-170 / M2–M5 | `park` | unchanged |

CFR-171 stays `investigating` in the ledger; only this B1 slice is classified for 5c. CFR-172 remains
the experiment (`narrow` until one identical rerun). CFR-173 metrics stay frozen.

Stall risks: implementing subject/profile/phase because the contract *wished* they existed; tightening
envelope `data`; pulling Python/HTTP because ADR-018 mentioned three runtimes; treating this
classification as permission to start 5d before 5c; rerunning the consumer on unbound payloads.

Next legal action: wait for node 5b and orchestrator 5c. 5c may admit **exactly** the write/test set
in §4. Do not implement from this packet. Do not rerun the consumer until that fix is integrated.
