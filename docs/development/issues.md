# 未解決の課題

開いている課題だけを載せる。直したら行を消し、`history.md` に1行残す。詳細は `issues/` 。

| ID | 概要 | 優先 | 扱い |
|---|---|---|---|
| STALE-BULK | stale resolution をまとめて更新する手順が案内されない（batch は `refresh` 項目を受け付けるが、template は open fire がないと作れない） | 低 | 件数が多くて詰まったら |
| HELP-FLOW | `--help` と `scan` の出力が「scan → extinguish → verify → commit」の順序や Fire の意味を教えない。試用エージェントは対象プロジェクトの文書から流れを知った | 中 | 次に直す |
| VERIFY-WORDING | `verify` が必須リンク不足7件でも「passed」と出す理由（policyで許可）を言わない。`Change coverage` 行の語（unclassified spans など）が読めない | 中 | 次に直す |
| PERF-LOOP | Algorithm Lab（183ファイル）で scan 4–12秒、verify 10–34秒、commit 最大72秒（高負荷時の値を含む）。進行表示もない | 中 | 負荷の低い状態で測り直してから判断 |
| IMPORT-RESIDUE | `import` 成功後も `.codefire-import-transaction.json` と `.codefire-import.lock` がopen directoryに残る。意図した記録か未整理かを確認する | 低 | 段階4で詰まったら |
| [CFB-136](issues/cfb-136.md) | command cache が同一inodeのobject書換えを検証済みとして再利用する | 中 | 現HEADで再現してから判断 |
| [CFR-169](issues/cfr-169.md) | lifecycle/object/test の責務が巨大moduleに残る | 中 | 段階5 |
| [CFR-170](issues/cfr-170.md) | next action が全commandで実行可能な形になっていない | 中 | 段階4で詰まったら |
| FLAKY-HTTP | `http::tests::missing_upload_generic_body_limit_dominates_before_bootstrap_dispatch` が高負荷時にまれに失敗する（2026-11-10 に全体実行3回中1回、単体5回は成功） | 低 | 凍結範囲。再発が続けば調べる |
| [CFB-133](issues/cfb-133.md) | `migrate apply` 成功後も消化済みplanのactionを返す | 低 | 凍結範囲 |
| [CFR-168](issues/cfr-168.md) | graph/object/HTTP の処理量に上限がない | 低 | 凍結範囲（HTTP側） |
| [CFR-171](issues/cfr-171.md) | consistency claim と certificate の種別が機械可読に束縛されない | 低 | 保留 |
| [CFR-172](issues/cfr-172.md) | 読み取り専用での導入を検証する実験がない | 低 | 保留 |
| [CFR-173](issues/cfr-173.md) | 進捗・Fireの価値を検証するpilot手順がない | 低 | 保留 |

未確認: Python版テストは旧環境で649件中61件失敗（約9分）と記録されている。凍結範囲のため未調査。
