# Decisions（重要な判断ログ）

管理者が「なぜそうしたか」を短く残すためのログです（箇条書きでOK）。
target project へコピーするときは、workspace 内の `docs/agents/decisions.md` として配置してください。

- YYYY-MM-DD: `<<判断>>`
  - 背景: `<<背景>>`
  - 代替案: `<<代替案>>`
  - 影響: `<<影響>>`

- 2026-06-06: Object IDの表示digest長を新規objectでは24 hexへ拡張し、既存12 hex IDは読み取り互換として残す
  - 背景: 12 hexは小規模運用では扱いやすいが、大量artifact/evidenceを扱う実験自動化用途では衝突余裕が不足する
  - 代替案: 12 hexを維持する、16 hexへだけ拡張する、完全64 hexを表示IDにする
  - 影響: 新規objectは `CF-*-<24 hex>` になる。既存 `CF-*-<12 hex>` recordはhash、record object_id、filenameが一致していれば読み取れる。mixed repositoryは参照先ID文字列をそのまま保持する
