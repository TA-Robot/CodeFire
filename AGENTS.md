# AGENTS.md（CodeFire本体開発用）

この `AGENTS.md` は **CodeFire本体リポジトリ** を改修するための開発ガイドである。

CodeFireは、仕様、設計、コード、テスト、証跡を整合した単位で封印するための開発管理システムである。実装者は、単にテストを通すだけではなく、履歴の整合性、object identity、traceability、診断品質、性能、メモリ効率まで含めて高い基準で扱う。

## 対象

- CLI本体: `codefire`
- テスト: `tests/`
- インストール/包装: `install.sh`, `pyproject.toml`, `setup.py`
- ドキュメント: `README.md`, `docs/`, `adr/`, `schemas/`
- 開発管理: `docs/development/`

## 開発姿勢

常にコードは **AtCoder red レベルの最高レベルのコードを作成すること** を心がける。

ここでいう AtCoder red レベルとは、短く巧妙なコードを書くことではない。制約を正確に読み、入力規模を見積もり、最適なアルゴリズムとデータ構造を選び、境界条件に強く、計算量とメモリ使用量を説明でき、長期保守に耐える実装へ落とし込む水準を指す。

CodeFireでは特に、以下を妥協しない。

- 正しさ
- 計算オーダ
- メモリ利用オーダ
- object identityの安定性
- repository破損時の安全性
- diagnosticsの明確さ
- testability
- migration compatibility

## 計算量へのこだわり

- 実装前に入力規模を想定する。
  - ファイル数
  - Atom数
  - Trace Link数
  - Fire数
  - object store内object数
  - commit ancestry長
  - remote branch / MR数
  - verification outputサイズ
- 主要処理の時間計算量を説明できるようにする。
  - 例: `O(files + lines)`, `O(V + E)`, `O(objects)`, `O(n log n)`。
- 無自覚な `O(n^2)` を避ける。
  - nested loopを書く場合は、対象サイズが小さいこと、または代替より妥当なことを確認する。
  - lookupは原則として `dict` / `set` / index map を使う。
- 同じファイル、JSON、YAML、objectを何度もparseしない。
- graph traversalは訪問済みsetを持つ。
- commit ancestry探索、object graph検証、TraceGraph検証は、cycle、missing reference、壊れたobjectに耐える。
- sortが必要な箇所では、安定性、決定性、コストを意識する。
- 大規模repoで支配的になる処理を見積もる。
  - 小さいfixtureで速いだけでは十分ではない。

## メモリ利用へのこだわり

- 全件読み込みが必要かを常に疑う。
- 巨大ログ、artifact、binary、verification outputを無制限にメモリへ載せない。
- stream処理、chunk処理、summary化で済む場合はそれを選ぶ。
- 同じpayloadのコピーを増やさない。
- object store、Atom index、TraceGraph、diagnosticsの保持量を見積もる。
- external artifactは原則としてpayload本体ではなく、hash、size、path/URI、summaryを扱う。
- JSON outputは便利だが、巨大診断を無制限に出さない。
  - limit、pagination、summary、detailsの分離を検討する。

## アルゴリズムとデータ構造

問題をまず抽象化する。

- TraceGraph: directed graph
- object graph: typed reference graph
- commit ancestry: DAG
- branch head update: atomic state transition
- duplicate Atom detection: set membership
- diff: sequence alignment + semantic impact
- merge: common ancestor + 3-way merge + consistency recovery
- policy evaluation: rule result collection

選択指針:

- duplicate検出はhash setを使う。
- reachabilityはDFS/BFSで訪問済みsetを持つ。
- topologicalな前提を置くなら、cycle時の挙動を定義する。
- diff/mergeはtext diff、Atom diff、TraceGraph diff、Fire impact diffを混同しない。
- path処理は正規化、相対/絶対、URL encode、slash入りbranch名、platform差を意識する。
- lock処理は原子性、timeout、stale lock診断、idempotencyを意識する。

## CodeFire固有の不変条件

以下は破ってはいけない。

- object IDはcanonical payloadから決まる。
- object storeはimmutableである。
- sealed commitは妥当なroot objectを指す。
- branch headは検証済みsealed commitにだけ向く。
- branch head更新はatomicである。
- open fireがblockingな状態ではcommitできない。
- stale resolutionはcommit blockerになる。
- duplicate Atom IDはcommit blockerになる。
- 壊れた履歴を通常操作へ広げない。
- server-side verificationはclient-provided verificationを信頼しない。
- remote mutationは認証、署名、branch protection、sealed commit validationを通す。

## 実装スタイル

- parse、validate、plan、apply、renderを分ける。
- filesystem writeの前にoperation planを作る。
- domain logicとCLI表示を分ける。
- human outputとmachine-readable outputを混同しない。
- errorは種類を分ける。
  - user error
  - config error
  - repository corruption
  - policy violation
  - verification failure
  - internal invariant violation
- 例外的な処理には理由を残す。
- コメントは「なぜその設計か」「どの不変条件を守っているか」に使う。

## 差分・マージ品質

Gitと比べて現在のCodeFireのdiff/mergeはまだ薄い。v0.6ではRust再実装と合わせて、以下を意識する。

- Myers / patience / histogram-style diffを比較可能にする。
- rename/copy detectionではsimilarity scoreを出す。
- binary fileはpayload diffではなくhash/size/kind summaryを出す。
- Atom diffを出す。
- TraceGraph diffを出す。
- Fire impact diffを出す。
- policy impact diffを出す。
- merge dry-runで書き込み前に影響を提示する。
- semantic conflictは候補検出までに留め、自動解決しない。
- review-packはfile diff、Atom diff、Trace diff、verification、next actionsをまとめる。

## Automation-Friendly Interface

CodeFireはAI agent運用機能を持たない。agentの起動、編成、分業、プロンプト管理、marketplaceは対象外である。

一方で、外部自動化ツール、人間のwrapper、CI、editor integrationが安全にCodeFireを叩けるよう、v0.6では以下を重視する。

- stable `--json`
- stable exit code taxonomy
- `codefire context`
- machine-readable `next_actions`
- dry-run / operation plan
- batch operation
- idempotency key
- `--wait-lock` / `--lock-timeout`
- evidence capture
- read-only `codefire explain`

実装時は、human outputよりmachine-readable contractの安定性を重視する。

## Rust v0.6方針

v0.6ではRust実装をdefault CLIへ移行する計画である。

守ること:

- Python版はreference implementation / fallbackとして残す。
- Rust write pathの前にcanonical JSON / object ID golden testを作る。
- Python-created repoをRustが読めることを先に保証する。
- Rust-created objectをPython版が最低限 `doctor` / `show` できる互換性を保つ。
- object format extensionはversioned extensionとして追加し、既存fieldの意味を変えない。
- installer切替はmigration checkが揃ってから行う。

参照:

- `docs/development/v0.6-rust-rewrite-plan.md`
- `docs/development/todo-checklist.md`
- `docs/development/bug-backlog.md`

## テスト方針

変更範囲に応じて、最小ではなく十分なテストを走らせる。

`codefire` 本体を触ったら:

```bash
python3 -m py_compile codefire tests/test_codefire_cli.py
python3 -m unittest discover -s tests -v
./demo.sh
```

installやpackagingを触ったら:

```bash
bash -n install.sh
./install.sh --prefix /usr/local
codefire --help
```

Rust workspace追加後は:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

docs-only変更ではテスト不要。ただしMarkdownの表、リンク、TODO/historyの整合性は確認する。

## CodeFire dogfooding運用

CodeFire自身の不具合、バグに近いUX、改善点を見つけたら:

- `docs/development/bug-backlog.md` に `CFB-*` として記録する。
- 実装タスクに落とせる場合は `docs/development/todo-checklist.md` に `CF-*` として追加する。
- 方針や履歴は `docs/development/history.md` に残す。

開発対象がCodeFire管理repoの場合は、原則として:

```bash
codefire scan
codefire verify --details
codefire extinguish ...
codefire commit -m "..."
```

Git管理側のCodeFire本体では、docs/code/testsの変更を小さく分けてgit commitする。

## 禁止・注意

- ユーザーの未コミット変更を勝手に戻さない。
- `git reset --hard` や `git checkout --` で破壊的に戻さない。
- 依存追加を軽く扱わない。
- 大きなrefactorを小さなbug fixに混ぜない。
- object format互換性を壊す変更を無計画に入れない。
- semantic conflictの自動解決をv0.6に入れない。
- AI agent運用機能やmarketplace機能をCodeFire本体に入れない。
