# 未解決の課題

開いている課題だけを載せる。直したら行を消し、`history.md` に1行残す。詳細は `issues/` 。

| ID | 概要 | 優先 | 扱い |
|---|---|---|---|
| [CFB-140](issues/cfb-140.md) | stale resolution の更新手順が fire ID を出さず、初回commitで行き止まる | 高 | 段階3で直す |
| DEMO-RS | `demo.sh` がRust CLIで `open` の時点で失敗する（Python版前提） | 高 | 段階3で直す |
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
