# CodeFire Executable Source Mental Model

この文書は、CodeFireのsourceを開く前に、実装がどのような関数列、型、永続化境界で動いているかを再現するための実装像である。

目的は「設計を読む」ではなく「このドキュメントだけで、sourceの形をかなり具体的に想像できる」状態を作ることにある。詳細な型辞書は `source-data-model-catalog.md`、関数帯の索引は `source-code-map.md`、復元手順は `source-code-reconstruction-guide.md` を参照する。

## 1. Program Shape

CodeFireはRust workspaceで、実行時には次の4層だけがある。

```text
argv/text/json/http
  -> codefire-cli
      command parser
      repository/open-directory orchestration
      command-specific renderers
  -> codefire-core
      Atom extraction
      Trace Graph
      required-link policy
      Fire, Resolution, Verification
  -> codefire-store
      canonical JSON
      immutable object records
      sealed commit validation
  -> codefire-util
      hash, hex, time, path helpers
```

依存方向は常に下向きである。`core` はCLI outputやfilesystem stateを知らない。`store` はAtomやFireの意味を知らない。`cli` が両方を呼び、`.codefire` とopen directoryを更新する。

## 2. Runtime State You Should Picture

通常のopen directoryには、次の3種類のstateが同時に存在する。

```text
repository root
  .codefire/
    branches/<branch>
      branch head and opened directory registry
    objects/
      immutable records grouped by object kind
    active/<open-id>/
      scan.json
      fires.json
      resolutions.json
      verification.json
      state.json

open directory
  .codefire-open
    repo root, branch, open id, base commit
  working files
    docs, source, tests, codefire.links.yaml, policy files
```

`commit` はworking filesとactive stateをsealed objectに変換し、branch headを進め、active stateをcleanに戻す。`scan` と `verify` はsealed historyを変更せず、active stateだけを更新する。

## 3. Local Workflow Call Graphs

### scan

```text
main.rs::run
  -> local_workflow_handler("scan")
  -> run_scan_command(args)
     -> parse_path_json_args(args, "scan")
     -> run_scan(path)
        -> compute_scan(path, persist=true)
           -> open_context(path)
              reads .codefire-open and branch/open registry
           -> build_atom_index(open_dir)
              extracts Markdown and explicit cf-atom source atoms
           -> load_base_atom_index(objects, base_commit)
              validates base commit and reads prior atom index object
           -> current_trace_graph(open_dir)
              reads codefire.links.yaml / trace link files
           -> parse_trace_policy(open_dir)
              loads required-link policy
           -> build_scan_result(current, base, graph, policy, resolutions)
              creates changed atoms and missing required-link fires
           -> write active scan.json, fires.json, state.json
     -> render_scan or automation::scan_data_json
```

`scan` の実装像は、Atom diffとTrace policy checkである。source hashやlinksが変わると `changed_atoms` が増え、required linkが欠けるとfireが生まれる。

### verify

```text
run_verify_command(args)
  -> verification::parse_verify_args
  -> run_verify(path)
     -> compute_verify(path, persist=true)
        -> compute_scan(path, persist=true)
        -> parse_verification_policy(open_dir)
        -> run_verification_commands(open_dir, policy.commands)
        -> build_verification(scan, resolutions, external checks, policy)
        -> persist_verification_result_state
  -> verification::print_verification or automation::verification_data_json
```

`verify` は `scan` の上位である。missing link fires、未解決fire、stale resolution、verification command失敗をcommit blockerに変換する。

### extinguish

```text
run_extinguish_command(args)
  -> parse_extinguish_args(args)
  -> run_extinguish(options)
     -> open_context
     -> load active fires
     -> validate evidence refs against object store
     -> build_resolution(request, current atom index, trace graph)
     -> remove or mark matching active fire
     -> write resolutions.json and fires.json
     -> set active state based on remaining fires
```

`extinguish` はfireを消すだけではない。消した理由、対象Atom、evidence ref、resolution basisをactive stateに残し、後続 `verify` がstale判定できるようにする。

### commit

```text
run_commit_command(args)
  -> parse_commit_args(args)
  -> run_commit(options)
     -> open_context
     -> compute_verify(open_dir, persist=true)
     -> reject if blockers remain unless policy permits
     -> build_manifest(objects, open_dir)
        stores blob objects and content manifest
     -> store atom index, trace graph, fire ledger,
        resolution ledger, verification, policy roots
     -> commit_payload(parent ids, root object ids, message, signature)
     -> codefire-store::store_object("commit", payload)
     -> save_branch_record(branch head = new commit)
     -> reset_active(active, "clean")
```

`commit` の正体は「working tree snapshot + semantic roots + verification result」を1つのsealed objectへ束ねる処理である。Git互換の差分保存ではなく、CodeFire独自の意味付きsnapshotを保存する。

## 4. Main File Shapes

### `crates/codefire-cli/src/main.rs`

`main.rs` はまだ大きいが、内部は次の順で並ぶ。

| Zone | Typical symbols | What mutates |
|---|---|---|
| dispatch | `main`, `run`, command match | none directly |
| local workflow wrappers | `run_scan_command`, `run_verify_command`, `run_extinguish_command`, `run_commit_command` | active state through runners |
| rendering | `print_status`, `render_scan`, JSON envelope helpers | stdout only |
| parsers | `parse_*_args`, option helpers | none |
| repo lifecycle | `init_repo`, `open_branch_from`, `clone_branch` | `.codefire`, open dirs |
| merge/patch | `merge_branch`, `import_patch` | open dir and active state |
| local workflow core | `compute_scan`, `compute_verify`, `run_extinguish`, `run_commit` | active state, objects, branch head |
| repo services | `open_context`, `build_manifest`, `materialize_commit`, `list_branches` | read/write depending on helper |
| low-level IO | `write_json_atomic`, locks, `read_json`, path helpers | filesystem primitives |

変更時の判断:

- parserがfilesystemを書き始めたら責務違反。
- rendererがdomain decisionを持ち始めたら責務違反。
- `main.rs` に新しい長いfeature runnerを足す場合、同時にmodule抽出候補を記録する。

### `crates/codefire-core/src/lib.rs`

`core` は状態を保存しない純粋寄りのsemantic engineである。

```text
input files and config
  -> Config
  -> AtomIndex
  -> TraceGraph
  -> TracePolicy / VerificationPolicy
  -> ScanResult
  -> Verification
  -> Resolution basis helpers
```

主要な型の意味:

| Type | Imagine it as |
|---|---|
| `Atom` | requirement/design/code/testなどの追跡単位 |
| `AtomIndex` | current source treeから抽出したAtom table |
| `TraceLink` | `from -> to` の意味付き辺 |
| `TraceGraph` | trace link集合と参照整合性 |
| `TracePolicy` | required link ruleの集合 |
| `Fire` | required traceやsemantic consistencyが欠けた未解決警告 |
| `Resolution` | fireを消した理由とevidence |
| `Verification` | commit可能かどうかの判定結果 |

### `crates/codefire-store/src/lib.rs`

`store` はobject IDとsealed validationのための層である。

```text
payload
  -> canonical_json(payload)
  -> sha256(type_tag + NUL + canonical_json)
  -> object id prefix + digest prefix
  -> ObjectRecord { type, id, payload }
  -> .codefire/objects/<kind>/<id>.json
```

この層で守るべき不変条件:

- file name、record id、payload digestが一致する。
- commit objectは参照するroot objectが存在する。
- branch headはsealed commitだけを指す。
- canonical JSONが変わればobject idも変わる。

## 5. Module Ownership In Source Terms

| Module | Think of it as | Main read inputs | Main outputs |
|---|---|---|---|
| `automation.rs` | machine-readable command adapter | command result structs | JSON envelope, diagnostics, next actions |
| `metrics.rs` | observability adapter | status/scan/verify/storage models | compact metrics JSON/text |
| `verification.rs` | verify CLI shell | argv, verification model | options and human text |
| `fire.rs`, `fire/batch.rs` | manual fire writers | atom index, active fires | active fire ledger updates |
| `batch.rs`, `extinguish_ux.rs` | fire resolution UX | active fire ledger, evidence refs | resolution requests |
| `evidence.rs`, `evidence/batch.rs` | evidence object producer | artifact path, URI, command output | evidence and artifact objects |
| `context.rs` | AI context pack builder | changed atoms, graph, source snippets | bounded context JSON |
| `explain.rs` | diagnostic reader | fire/atom/verification/storage refs | explanation JSON/text |
| `view.rs`, `view/*` | sealed history viewer | commitish objects | show/diff/review/patch text or files |
| `remote.rs`, `http.rs` | file-backed remote transport | local/remote objects and requests | remote branches/MRs/HTTP JSON |
| `signatures.rs` | trust policy | commit/request payloads and keys | signatures, verification, nonce cache |
| `doctor/*` | repository health checker | layout, objects, branch/open state | issue report |
| `storage.rs` | capacity report | object trees and remote dirs | size, object-type, retention report |
| `migration.rs` | upgrade planner | current layout | migration check/dry-run report |

## 6. Data Lifetimes

| Data | Created by | Mutated by | Becomes sealed when |
|---|---|---|---|
| working file content | user/tool | user/tool | `commit` stores manifest blobs |
| AtomIndex | `scan`/`commit` | recomputed, not edited | stored as root object in commit |
| TraceGraph | `scan`/`commit` | source link files change | stored as root object in commit |
| active fires | `scan`, `fire` | `scan`, `extinguish` | fire ledger object in commit |
| active resolutions | `extinguish` | `extinguish`, stale checks | resolution ledger object in commit |
| verification | `verify`, `commit` | recomputed | verification object in commit |
| evidence | `evidence add` | never, object is immutable | immediately object-stored |
| commit | `commit` | never | object ID is content identity |
| branch record | `init`, `commit`, remote apply | mutable pointer | never sealed itself |

## 7. How To Modify A Feature

### Add or change a local workflow command

Touch order:

1. `docs/04_cli_spec.md` for command contract.
2. `docs/development/command-source-trace.md` for execution path.
3. parser/options in `main.rs`, command module, or `cli_model.rs`.
4. runner in the owning module.
5. JSON envelope in `automation.rs` or command module.
6. tests in `crates/codefire-cli/src/tests.rs` or module-local tests.
7. source maps in this directory.

### Add a semantic rule

Touch order:

1. `docs/09_index_trace_policy.md` and consistency docs.
2. `codefire-core/src/lib.rs` policy model and builder.
3. automation diagnostics and metrics if user/tool-visible.
4. scan/verify/commit regression tests.
5. source-data catalog if a new JSON shape appears.

### Add an object kind

Touch order:

1. `docs/08_object_store.md`.
2. `codefire-store/src/lib.rs` object kind registry.
3. write/read path in `codefire-cli`.
4. doctor/storage checks.
5. sealed validation tests.

## 8. What Source Should Feel Like

If the docs are accurate, a reader should be able to predict these implementation facts before opening the files:

- `scan` computes and persists active semantic state but does not create a commit.
- `verify` calls scan-like logic and then converts unresolved semantic state into blockers.
- `extinguish` records evidence-backed resolution state and can become stale when the basis changes.
- `commit` is the only local command that advances branch head.
- `core` returns domain values; `cli` decides where to write them and how to render them.
- `store` never decides whether a fire is acceptable; it only decides whether an object is valid.
- remote operations copy and validate object graphs rather than trusting branch pointers.

When one of these predictions becomes false, update this document in the same change as the source.
