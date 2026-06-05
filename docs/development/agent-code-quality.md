# Agent Code Quality Standard

この文書は、CodeFire本体を改修するagentが守るコード品質基準を定義する。

## 基本姿勢

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

## 計算量

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

## メモリ利用

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

## テストと検証

- 小さい変更でも、壊れる可能性がある不変条件をテストする。
- 重要なアルゴリズムには正常系、境界系、異常系を用意する。
- object hash、canonical JSON、diff、merge、graph traversalはgolden testを重視する。
- 性能が重要な処理は、少なくとも簡易的な大規模fixtureで確認する。
- コマンドを触ったら、exit code、stdout、stderr、JSON出力、状態変化を確認する。
