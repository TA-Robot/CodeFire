# project/AGENTS.md（サブエージェント用：configured workspace 内のみを対象にする指示テンプレ）

この `project/AGENTS.md` は、**対象プロジェクトへコピーして `AGENTS.md` として使う、サブエージェント（implementer/reviewer/triage）向け指示テンプレ**です。  
目的は「サブエージェントが **configured workspace 内だけ**を見て作業する」運用を徹底することです。

> 注意: OS レベルの強制隔離ではありません。  
> ただし、`codex-second-agent workspace init <path>` を必須にし、sub-agent の `--cd` / `--add-dir` を workspace 内だけに制限することで、誤ったスコープ拡張を防ぎます。

## codex-second-agent とは？（簡単に）

- `codex exec` を「セッションID自動保持」「agent別worktree」「ログ保存」付きで呼び出すラッパーです
- この基盤リポジトリでは `scripts/codex-second-agent` が実体です
- 実運用では通常 `codex-second-agent` を PATH から実行します
  - PATH に無い場合は、管理者が基盤リポジトリ側の実体パスを指定して起動してください

---

## スコープ（最重要）

- **見てよい/触ってよい**: **現在の workspace 配下のみ**
- **見ない/触らない**:
  - リポジトリルート配下の `scripts/` / `.devcontainer/` / `docs/` / `README.md` / `AGENTS*.md` など
  - workspace 外のファイルを参照してよいか迷ったら、必ず管理者へ質問して止まる

## docs/agents（管理者が整備する前提）

- 管理者は **workspace 内の** `docs/agents/` を整備します（runbook / decision log / plan / tickets）
- 管理者は target project の `.gitignore` に `.codex-second-agent/` と `.codex-worktrees/` を入れておいてください
- サブエージェントは **参照してよい**（ただし更新が必要なら管理者に提案）
- wrapper は sub-agent の `--cd` / `--add-dir` を workspace 外へ出せません
  - runbook / ticket / decision log も workspace 内から読める場所に置くのを既定にしてください

## 役割別の期待

- **implementer**
  - 現在の workspace 直下のみで実装し、対象プロジェクトのテスト/ビルドが通る状態まで持っていく
  - `git status -sb` を確認し、必要ならコミットしてブランチ先端を進める
- **reviewer**
  - 現在の workspace 直下だけの差分レビューを行う（Must/Should/Nice）
- **triage**
  - 現在の workspace 直下のログ/コードから原因切り分け。必要な追加情報があれば管理者に要求

## 実行コマンド（管理者が使う想定）

### implementer を project 固定で起動

```bash
# 最初に一度だけ、対象プロジェクト（git repo root）を保存する
codex-second-agent workspace init <path-to-project-git>

mkdir -p .codex-second-agent/nohup
cat <<'PROMPT' | nohup codex-second-agent --agent implementer --post-git-status - > .codex-second-agent/nohup/implementer.out 2>&1 &
あなたは implementer です。
- 作業対象は workspace 直下のみ（= 管理者が `workspace init <path-to-project-git>` で固定した対象プロジェクト）
- 変更は現在の worktree 配下のみに限定

要件:
- ...
制約:
- 依存追加は事前承認
完了条件:
- 対象プロジェクトのテストが通る
PROMPT
echo "pid=$!"
```

### reviewer を project 固定で起動

```bash
# 対象プロジェクトを保存済みであること（未設定なら init する）
codex-second-agent workspace init <path-to-project-git>

mkdir -p .codex-second-agent/nohup
cat <<'PROMPT' | nohup codex-second-agent --agent reviewer - > .codex-second-agent/nohup/reviewer.out 2>&1 &
あなたは reviewer です。
- レビュー対象は workspace 直下（= 対象プロジェクト worktree）の差分のみ
- 指摘は Must/Should/Nice に分ける
PROMPT
echo "pid=$!"
```

## ログ/スコープ確認（迷子防止）

```bash
codex-second-agent --agent implementer paths
codex-second-agent --agent implementer status --verbose
codex-second-agent --agent implementer doctor
```

`workspace init <path-to-project-git>` を使っている場合、`paths` の `effective_cd` は **対象 project 用 worktree のルート** を指します。  
親リポジトリを workspace にしたまま一部ディレクトリへ絞る運用では、`-- --cd <<workdir>>` を付け、その場合は `effective_cd` が `.../<<workdir>>` になります。
`workspace_valid: no` が出たら、保存済み workspace が壊れているので `workspace init <path-to-project-git>` をやり直してください。

## 実運用メモ（ハマりどころ）

- `nohup` の標準出力ファイルは **空のまま**になることがあります。進捗・成果物の回収は基本 **`transcript.jsonl` を見る**運用が安定します。
- sub-agent は `--cd` / `--add-dir` で workspace 外を指定できません。必要な前提は `docs/agents/` へ置くか、チケット本文へ転記してください。
- reviewer は長引くことがあるので、バックグラウンド実行では `timeout` 付きにするのがおすすめです:

```bash
mkdir -p .codex-second-agent/nohup
cat <<'PROMPT' | nohup timeout 120s codex-second-agent --agent reviewer - > .codex-second-agent/nohup/reviewer.out 2>&1 &
対象コミット: <hash>
出力: Must/Should/Nice
PROMPT
echo "pid=$!"
```

## 補足（よくある誤解）

- **worktreeが project 配下に作られるわけではありません**  
  `codex-second-agent` は基本的に「workspace（git root）単位」で agent worktree を作ります。  
  `codex-second-agent workspace init <path-to-project-git>` を使うと、**対象 project 側の Git を workspace として扱う**ようになります。
- `workspace init <path-to-project-git>` を使った後に workspace 外の `--cd` / `--add-dir` を付けることはできません。
  project 側 Git を workspace にした場合は、通常 `--cd` を付けません。
