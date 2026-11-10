# 未解決の課題

開いている課題だけを載せる。直したら行を消し、`history.md` に1行残す。詳細は `issues/` 。

| ID | 概要 | 優先 | 扱い |
|---|---|---|---|
| MERGE-FILE-LEVEL | merge がファイル単位。両方の枝が同じファイルの別の行を変えただけで、ファイル全体が conflict marker で二重になる（Algorithm Lab では仕様書1,381行・設計書4,548行が丸ごと重複し、Atomが二重になって trace graph エラー706件） | 高 | 次に直す（行単位の3-way merge） |
| MERGE-GUIDANCE | conflict 後の `merge`・`verify`・`explain` が marker の存在、対象ファイル、`merge --continue` を案内しない。Atomが二重のときは exit 21 の別エラーになり、`status` は open-clean のまま | 高 | MERGE-FILE-LEVEL と一緒に直す |
| BRANCH-GUIDANCE | 枝の作り方が `branch --help` から分からない（`clone` が作る）。`clone` の後に `open` を案内しない。`open` にパスがないとき置き場所を提案しない。`close` と `discard` の使い分けが help から読めない | 中 | 次に直す |
| STALE-BULK | stale resolution をまとめて更新する手順が案内されない（batch は `refresh` 項目を受け付けるが、template は open fire がないと作れない） | 低 | 件数が多くて詰まったら |
| VERIFY-WORDING | `Change coverage` 行の語（unclassified spans、file_only_no_span_claim）が利用者に読めない | 低 | 次の周で詰まったら |
| STATUS-STATE | 未scanの変更があっても `status` の `State: open-clean` 行はそのまま（下に案内行は出る） | 低 | 保留 |
| METHOD-ATOMS | `scan` が `CODE:path::method:...` 形式の自動Atomを説明なしで並べる。Atom ID の名前と中身のずれ（例 dedups → rejects）は検出対象外 | 低 | 保留 |
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
