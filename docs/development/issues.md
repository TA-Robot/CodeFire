# 未解決の課題

開いている課題だけを載せる。直したら行を消し、`history.md` に1行残す。詳細は `issues/` 。

| ID | 概要 | 優先 | 扱い |
|---|---|---|---|
| STALE-BULK | stale resolution をまとめて更新する手順が案内されない（batch は `refresh` 項目を受け付けるが、template は open fire がないと作れない） | 低 | 件数が多くて詰まったら |
| VERIFY-WORDING | `Change coverage` 行の語（unclassified spans、file_only_no_span_claim）が利用者に読めない | 低 | 次の周で詰まったら |
| STATUS-STATE | 未scanの変更があっても `status` の `State: open-clean` 行はそのまま（下に案内行は出る） | 低 | 保留 |
| METHOD-ATOMS | `scan` が `CODE:path::method:...` 形式の自動Atomを説明なしで並べる。Atom ID の名前と中身のずれ（例 dedups → rejects）は検出対象外 | 低 | 保留 |
| NEW-ATOM-LINKS | 新しいAtomを足しても `scan` は trace link を書くよう促さない（必須リンク不足は既存分と合算の警告数だけ）。link の書式もCLIからは分からない | 中 | 次に直す |
| MERGE-REVIEW | semantic conflict の確認用に、merge結果と各枝の差分を見る手段がCLIにない（試用エージェントは自前で事前コピーを取って比較した） | 中 | 次に直す |
| MERGE-EXIT | conflict が残っても `merge` は exit 0（`merge --continue` と `verify` は非0）。スクリプトが見落とす | 低 | 仕様として決める |
| BATCH-RATIONALE | batch template は全Fire共通の rationale 1つ。Fireごとの理由はJSONを手で編集するしかない | 低 | 保留 |
| IMPORT-RESIDUE | `import` 成功後も `.codefire-import-transaction.json` と `.codefire-import.lock` がopen directoryに残る。意図した記録か未整理かを確認する | 低 | 段階4で詰まったら |
| [CFB-136](issues/cfb-136.md) | command cache が同一inodeのobject書換えを検証済みとして再利用する | 中 | 現HEADで再現してから判断 |
| [CFR-169](issues/cfr-169.md) | lifecycle/object/test の責務が巨大moduleに残る | 中 | 段階5 |
| [CFR-170](issues/cfr-170.md) | next action が全commandで実行可能な形になっていない | 中 | 段階4で詰まったら |
| FLAKY-HTTP | 高負荷時（load average 15–22）に HTTP のテストが不安定。`missing_upload_generic_body_limit_dominates_before_bootstrap_dispatch` が1回失敗、`http_upload_bootstrap_payload_limit_is_missing_only_and_pre_effect` が全体実行で2回、10分以上終わらなかった（単体と他の全体実行8回は成功）。原因は未特定 | 中 | 凍結範囲だが `scripts/check.sh` を止めるので、再発したら調べる |
| [CFB-133](issues/cfb-133.md) | `migrate apply` 成功後も消化済みplanのactionを返す | 低 | 凍結範囲 |
| [CFR-168](issues/cfr-168.md) | graph/object/HTTP の処理量に上限がない | 低 | 凍結範囲（HTTP側） |
| [CFR-171](issues/cfr-171.md) | consistency claim と certificate の種別が機械可読に束縛されない | 低 | 保留 |
| [CFR-172](issues/cfr-172.md) | 読み取り専用での導入を検証する実験がない | 低 | 保留 |
| [CFR-173](issues/cfr-173.md) | 進捗・Fireの価値を検証するpilot手順がない | 低 | 保留 |

未確認: Python版テストは旧環境で649件中61件失敗（約9分）と記録されている。凍結範囲のため未調査。
