# CFR-168 Python HTTP accepted-input composition — exact preflight, manager publication

Status: durable publication of a completed read-only preflight. Sections 0-7 are
authored by the CFR-168 Plan/FREEZE authority lane (manager) on 2026-08-02.
Section 8 onward is the preflight agent's own formal handoff body, preserved
byte-for-byte and never edited.

**This publication authorizes no product byte.** The preflight authorized
nothing, in its own words. The manager disposition that releases a bounded
composition writer is a separate artifact:
`docs/development/ai/task-packets/cfr168-python-http-accepted-input-composition-20260802.md`
and its typed companion
`docs/development/ai/task-packets/json/cfr168-python-http-accepted-input-composition-20260802.json`.
Read that packet, not this file, for what is currently authorized.

Manager lane constraints honoured while producing this file: Claude Opus 5 only;
no subagents, no Codex/Fugu, no Cargo/Rust, no product test, no install, no
dogfood, no network; the external capture was read without modification; no
product, runtime, test or packaging byte was touched by this commit.

---

## 0. Capture provenance, recomputed

The preflight ran as an external read-only agent lane whose transcript and event
log live outside this repository. This manager lane read them without writing to
them and recomputed every digest below from the bytes on disk.

| Field | Value | Recomputed |
|---|---|---|
| Agent | `cfr168_python_composition_exact_preflight_20260802` | match |
| Session (`session_id` field and `session_id` file) | `8434420d-dfbf-4ff4-b288-ee846dec92ba` | match |
| Capture timestamp (`ts`) | `2026-08-02T15:42:10.597976+00:00` | match |
| `is_error` | `false` | match |
| Transcript SHA-256 | `b2d80ec5d94b363efcc31aa8ff757030958b6556b80bd8e877bfc4e92407e9c1` | match |
| Events SHA-256 | `bdd9f401f5c7b891e6c9c949ad575317a875f9c98ff6ae8f78db857c189272f1` | match |
| Exact `jq -j .response` bytes SHA-256 | `06308884e0de0c87828c5e8fe96c32f872864035e22ced74ad0576f6cf4caf29` | match |
| Response size | 64,017 bytes, 674 lines (final line unterminated) | recomputed |
| Transcript record shape | one JSON object, keys `agent`, `cd`, `is_error`, `prompt`, `response`, `session_id`, `ts` | recomputed |

External capture paths, quoted exactly for audit and not tracked in this
repository:

```text
.../worktrees/cfr168_cargo_gate_canonical_port_20260802/.claude-second-agent/
  45ecdffc648a74131bed1edb1c993da8cfbaf142a4936d38443768d178092f1e/agents/
  cfr168_python_composition_exact_preflight_20260802/logs/transcript.jsonl
.../same directory/logs/events.jsonl
```

The preflight's own recorded working directory (`cd`) was its sibling worktree
`cfr168_python_composition_exact_preflight_20260802`, not this one. It reported a
clean tree, empty index, no stash, no commit and no report file, and it used
`git hash-object` without `-w`, so it wrote no Git object. Publishing this body
here is the first durable repository record of that lane.

## 1. Bases: the preflight's prospective base, and the current canonical base

| Role | Commit | Tree |
|---|---|---|
| Exact preflight prospective base (`HEAD` during the preflight) | `fb891b01f45e4d1963c48f06b827f8335150dace` | `2dadc3088c14fb624fb30d8060eba7466297959a` |
| Its parent | `41dea1488bf04c8a2bc1c1d1ad3df7ac9b2570f6` | `47b6bc83350fff67386ec92c273f74c75a2d9471` |
| Current canonical base, and the base of this commit | `a972cde890e0d621aaced6aca4b45835871da975` | `76a7342031f560a116916cfe978979674841c75f` |
| Its parent | `1a9e01d462ed39599a1d8cf0b864264cf0861d18` | — |

`a972cde` is the canonical head that carries the integrated AIA-046 source-keyed
Cargo control and the complete O1 evidence chain. Every `fb891b0`-relative value
in section 8 has been revalidated against `a972cde` in sections 2 and 3 below.

The manager's post-integration exact gate on `a972cde` was
`python3 docs/development/tools/check_ai_development.py --timeout 180`, which
passed 7/7 in 53.188 s. **That is the manager integration lane's measurement, not
a run performed by this Plan/FREEZE lane.** This lane's own validation runs are
recorded in the task packet and in the commit that publishes it.

## 2. `fb891b0` -> `a972cde` is docs-only, confined to the Cargo/O1 authority chain

Recomputed on this worktree:

- `git rev-list --count fb891b0..a972cde` = **4**; `git rev-list --merges fb891b0..a972cde | wc -l` = **0**.
- The four commits are `8316956` (fresh exact review of the AIA-046 canonical Cargo port), `38caf67` (publish source `1bbcfde` ACCEPT and rebind Cargo port authority), `1a9e01d` (correct premature canonical wording in the O1 publication) and `a972cde` (fresh exact review of the O1 source-ACCEPT authority rebind).
- `git diff --name-status fb891b0 a972cde` yields exactly **5** paths, all under `docs/`:

| Op | Path | Nature |
|---|---|---|
| `A` | `docs/development/ai/swarm-results/aia-source-keyed-cargo-gate-canonical-port-rereview-claude-20260802/review.md` | Cargo port review report |
| `A` | `docs/development/ai/swarm-results/aia-source-keyed-cargo-gate-fourth-closure-rereview-claude-20260802/review.md` | Cargo source ACCEPT report |
| `A` | `docs/development/ai/swarm-results/aia-source-keyed-cargo-source-accept-authority-rebind-rereview-claude-20260802/review.md` | O1 authority-rebind review report |
| `M` | `docs/development/ai/task-packets/cfr168-source-keyed-cargo-canonical-port-20260731.md` | Cargo port packet authority rebind |
| `M` | `docs/development/current-cycle-burn-down-plan.md` | lifecycle authority, closure-mode item 1 |

- `git diff --name-only fb891b0 a972cde -- codefire tests/ crates/ Cargo.toml Cargo.lock install.sh demo.sh pyproject.toml setup.py schemas/` = **0 paths**.

So the divergence is entirely Cargo/O1 report and authority documentation plus
the burn-down item that records it. No product, runtime, test or packaging byte
moved, which is exactly the condition the preflight's `BL-3` said would keep its
product-side conclusions intact under any docs-only amendment of `fb891b0`.

## 3. Revalidated composition target paths and blobs on `a972cde`

Every path the preflight's plan reads from the base, or writes into the base,
revalidated at `a972cde` against its `fb891b0` value.

### 3.1 Base-side target paths that must be unchanged

| Path | `fb891b0` | `a972cde` | State |
|---|---|---|---|
| `codefire` (mode `100755` at both) | `61214a21fd866efc0d9a8888dfbfc8e707039d48` | `61214a21fd866efc0d9a8888dfbfc8e707039d48` | unchanged |
| `tests/test_codefire_cli.py` | `10317e13deb531d04b9df44e756d7b8659b31c82` | `10317e13deb531d04b9df44e756d7b8659b31c82` | unchanged |
| `docs/development/source-architecture.md` | `0fe3dc8470ee2a477afae7fb874fe645071908e2` | `0fe3dc8470ee2a477afae7fb874fe645071908e2` | unchanged |
| `docs/development/command-source-trace.md` | `5feeaff7784cea83ad5db9205ddc648fe89703f6` | `5feeaff7784cea83ad5db9205ddc648fe89703f6` | unchanged |
| `docs/development/change-playbook.md` | `12d7fc6ab41177454a3f8c2b3cb0ee156055f83f` | `12d7fc6ab41177454a3f8c2b3cb0ee156055f83f` | unchanged |
| `docs/development/source-data-model-catalog.md` | `77623025f184a89a87cbcfb6447fb082edc77733` | `77623025f184a89a87cbcfb6447fb082edc77733` | unchanged |
| `docs/development/code-review-issues/cfr-168.md` | `3d9a4df9c19eff20a0d38bef4ff2c6a75a919bb8` | `3d9a4df9c19eff20a0d38bef4ff2c6a75a919bb8` | unchanged |

The four union base blobs are additionally byte-identical to `cb13da0`, so the
three-way base the preflight chose is still unambiguous on `a972cde`.
`codefire` is byte-identical at `5254e3d`, `16d0129`, `41dea14`, `fb891b0` and
`a972cde` — the canonical governance and Cargo lineage still never touched
product.

### 3.2 Target paths that must be absent at the base

Absent at both `fb891b0` and `a972cde`, so each remains a true `ADD`:

`tests/test_python_http_doctor_decision_work.py` ·
`tests/test_python_http_branch_decision_work.py` ·
`tests/test_python_http_mr_list_decision_work.py` ·
`tests/test_python_http_upload_accounted_traversal.py` ·
`docs/development/ai/swarm-results/cfr168-python-http-upload-live-route-cleanup-rereview-claude-20260802/review.md` ·
`docs/development/ai/swarm-batches/cfr168-python-http-upload-live-route-cleanup-repair-20260802.md` ·
`docs/development/ai/swarm-batches/cfr168-python-http-upload-zero-residual-repair-20260730.md` ·
`docs/development/ai/swarm-batches/cfr168-python-http-upload-zero-residual-repair-20260730.json` ·
`docs/development/ai/swarm-results/cfr168-python-http-upload-high2-third-repair-20260731/result.md` ·
`docs/development/ai/swarm-results/cfr168-python-http-upload-zero-residual-review-closure-repair-20260731/result.md` ·
`docs/development/ai/swarm-results/cfr168-python-http-upload-zero-residual-review-closure-repair-20260731/result-2.md`

### 3.3 Source-side blobs the writer will consume

| Source | Path | Blob |
|---|---|---|
| `b164046` | `codefire` (ours) | `f30575a697c1b525653c13738987ccdb5c0151ea` |
| `cb13da0` | `codefire` (3-way base) | `03966af23dd860c5235832320a8f1977ecd6495a` |
| `0f43415` | `codefire` (theirs) | `b984c1e7e393af658a618c96270d33d39bf8eeaa` |
| `b164046` | `tests/test_codefire_cli.py` | `b23c43ced6ccb69c0bb553f9856c1a7b2145e770` |
| `b164046` | `tests/test_python_http_doctor_decision_work.py` | `0f993edd6c6a57ee5ca0d15abc04e2260c3d48cd` |
| `b164046` | `tests/test_python_http_branch_decision_work.py` | `c0269ee80e00265d0144520ab975ce7e4bc9b7e3` |
| `b164046` | `tests/test_python_http_mr_list_decision_work.py` | `0d49841a2e682e661d169274e571e198f3408aa7` |
| `0f43415` | `tests/test_python_http_upload_accounted_traversal.py` (36 tests, taken) | `829836bc0411bf82030286e6f1b0e3d58e574e14` |
| `cb13da0` = `b164046` | same path (10 tests, duplicate omitted) | `141dcbd4ee1f8727fbe146faef481cf746c8c73f` |
| `cb13da0` / `b164046` / `0f43415` | `docs/development/source-architecture.md` | `0fe3dc8470ee` / `acc6c98cf73f2c8d698924b53485b10d67de83e6` / `cac0adb178be417fc197af1c3187fa14805f3082` |
| `cb13da0` / `b164046` / `0f43415` | `docs/development/command-source-trace.md` | `5feeaff7784c` / `a7bd13baff7aa06f867918e3225ca7c62c6d1f10` / `809629ddb96aed3d026d813f9f123fe2482dc1f7` |
| `cb13da0` / `b164046` / `0f43415` | `docs/development/change-playbook.md` | `12d7fc6ab411` / `a14c39647a5cda1e9859610d9bb1cf04cb8feab1` / `3ce2eb489661576d4dc24e8b4c0bdbdaf19b7646` |
| `cb13da0` / `b164046` / `0f43415` | `docs/development/source-data-model-catalog.md` | `77623025f184` / `e416eefeb26ba8e5791ea889fdf05b8a3576be40` / `a10683a226ec1385fc4d1d96b11f37d064414204` |
| `c54e5ce` | `…upload-live-route-cleanup-rereview-claude-20260802/review.md` | `216ee2680872c43dcd7b657e3a8d4312dbd9b1e9`, SHA-256 `46e7c3e4…` |
| `0f43415` | `…upload-live-route-cleanup-repair-20260802.md` | `250703ae50afcf842359abebc813cea717679c6a`, SHA-256 `756bf200…` |
| `0f43415` | `…upload-zero-residual-repair-20260730.md` | `46254b17badce37fdbec130967a0329bbb9afac7` |
| `0f43415` | `…upload-zero-residual-repair-20260730.json` | `4b4c6bfffcd2ab77f1e27d1345d543f58ddea143` |
| `0f43415` | `…upload-high2-third-repair-20260731/result.md` | `7cfa8bbe441b243e28fb8340fa249a9ed53a5005` |
| `0f43415` | `…upload-zero-residual-review-closure-repair-20260731/result.md` | `7100966bf76b1ada44d797e8787eb8377a18f6fd` |
| `0f43415` | `…upload-zero-residual-review-closure-repair-20260731/result-2.md` | `7169efbf1a0eb578aa92a71797f54abe9eb0299d` |

### 3.4 Negative space, revalidated on `a972cde`

- The 5 canonical-newer management docs the writer must never check out from a candidate: `docs/development/README.md`, `docs/development/history.md`, `docs/development/codefire-improvement-cycle.md` and `docs/development/ai/swarm-batches/cfr168-recovery-execution-ledger-20260730.md` are byte-identical to their `fb891b0` blobs; `docs/development/current-cycle-burn-down-plan.md` has advanced on canonical (`9707dd81c831` -> `eec0a389c0a3`) and advances once more in this commit. Canonical remains the newer authority on all five.
- The 18 already-identical candidate docs are still identical: 8 MR-list-lineage docs match `b164046` exactly, and 10 Upload-lineage docs match `0f43415` exactly.
- The accepted reports are present and unchanged at `a972cde`: Doctor/Branch `e5096bbb3a4da181b0bee471619de61f11b15635` / SHA-256 `c7a1665fdf4a6e1104bbab18ec8051c048ace1d25e865952bbbcd35a06eb19c0`; MR-list `ec7a09b795e0c3df22b6394ae03c465aa90f42c7` / SHA-256 `5c952a9a5c162bfbb93522bb644fcba44ce32ddf3628501e6f94308899fe760e`; shared-primitives preflight `ec2a264142c6625dadd4809d99cf18d3371e9847` / SHA-256 `ca1a1451bfc1b25ed1c54f1b012d4d0e508ef0a96f88401e61284c6d156a4441`.
- Independently recomputed on this worktree: `merge-base(701469f, b164046) = 701469f` (Doctor/Branch is an ancestor of MR-list); duplicate stable patch-id `9889888cedeba7344fb987ff5891cf728d9c7e36` for both `git show 0703f71` and `git diff 5254e3d cb13da0`; `git diff --binary 5790463..0f43415 | sha256sum` = `4ba34a6d92c5a6084f52fcdb978c0a4f0a5687b3fcac4bf019e8b2982dbf2fcb`; `git diff --binary 16d0129..b164046 -- codefire tests | sha256sum` = `2e9653fb511e7ad9f35fc56bb91a27d9bfa87fcd780f04af31b44b68d8a87c48`; all-paths variant = `567d98be14b61146cce1372047c486ea49904fcb0416d61f6f76975f4b9e9125`; `git grep C14-LIVE` over `a972cde` returns nothing, so the `C14-LIVE-*` handles are still new.

Conclusion: **the composition input graph the preflight measured is unchanged on
`a972cde`.** The preflight's product-side plan is directly executable against
the current canonical base.

## 4. Published body identity

The body in section 8 is the exact formal handoff, taken from the response
beginning at the first occurrence of the handoff title line and running through
the final `Bottom line` paragraph, which is also the end of the response.

| Property | Value |
|---|---|
| Start offset inside the exact response bytes | 373 |
| Body size | **63,644 bytes** |
| Body lines | **662** (661 newlines; the last line is unterminated in the source response) |
| Body SHA-256 | **`6342030bf0686a741869f9701fe71935ffd858dfba2db28b2a9c24d8f6a1085f`** |
| Discarded prefix | the 373 bytes of narration before the handoff title, five short progress lines and a `---` rule |

Byte-equality is mechanically checkable on this file. The body region begins at
the first byte of the handoff title line and runs to the end of the file, less
the single terminating newline this repository's Markdown convention adds:

```bash
python3 - <<'PY'
import hashlib
raw = open('docs/development/ai/swarm-results/cfr168-python-composition-exact-preflight-claude-20260802/preflight.md','rb').read()
marker = '# CFR\u2011168 PYTHON HTTP \u2014 EXACT COMPOSITION PREFLIGHT (read\u2011only)'.encode()
body = raw[raw.index(marker):].rstrip(b'\n')
print(len(body), hashlib.sha256(body).hexdigest())
PY
# expect: 63644 6342030bf0686a741869f9701fe71935ffd858dfba2db28b2a9c24d8f6a1085f
```

Nothing inside the body was normalized, reflowed, relabelled or corrected. Its
non-ASCII typography (U+2011 non-breaking hyphens, em dashes, curly quotes) is
part of the preserved bytes.

## 5. Historical prospective-base facts versus current manager disposition

Section 8 is a frozen record of what was true at `fb891b0` on 2026-08-02. Where
its premises have since moved, the current disposition is stated here and the
body is left alone.

| Body statement (historical, at `fb891b0`) | Current manager disposition (at `a972cde`) |
|---|---|
| §1.1 / `BL-3`: "`fb891b0` is still under fresh exact review… Everything here is conditional on that ACCEPT." | Superseded by evidence, not by edit. The Cargo port `fb891b0` holds a fresh exact `ACCEPT` at review `8316956`; the source `1bbcfde` `ACCEPT` is published at `38caf67`; the O1 authority rebind has its own fresh exact review at `a972cde`. Canonical `a972cde` carries that whole chain. The body's contingency is discharged, and its own fallback condition — that any docs-only amendment preserves the product blobs — held exactly (§2, §3.1). |
| §1.1 and throughout: base is `fb891b0`, tree `2dadc308…` | The composition base is now **`a972cde`**, tree `76a7342…`. Every base-dependent product value was revalidated and is unchanged (§3). Base-dependent *docs inventory* values did move: the preflight's `D2` figure of 683 Markdown files at `fb891b0` is **686** at `a972cde`, before this commit's own additions. |
| `BL-1`: "the shared-primitives preflight is not formally resolved… Reading the gate literally, Python composition is not yet authorized." | Adjudicated by the manager in the companion task packet, per the body's own recommendation that the manager record the adjudication rather than let a writer self-authorize it. `B1` is resolved **only** for the accepted-input composition writer; `B2` is released **only** as the composition resolution action and stays OPEN until the composed candidate earns a fresh exact `ACCEPT`; `B3` stays OPEN and routes to `C14-LIVE-A2`. The original shared-primitives verdict remains `BLOCK` and is preserved unchanged. |
| `BL-2`: burn-down status wording is manager-owned; §7.1's "optional 19th path" is `current-cycle-burn-down-plan.md`, recommended for deferral to the manager. | Accepted and discharged here. The manager writes the burn-down line in this same commit and **removes** `docs/development/current-cycle-burn-down-plan.md` from the writer's write set entirely. The frozen writer set is still 19 paths, but path 19 is now the writer's own new result report, not the burn-down. |
| §7.2: review path uses a `<provider>` placeholder. | Frozen to exactly one path: `docs/development/ai/swarm-results/cfr168-python-composition-exact-review-claude-20260802/review.md`. |
| §8.4 `D5`: `check_ai_development.py` default `--timeout 60` fails at `fb891b0` on `tool_tests`; `--timeout 300` passes 7/7. | Consistent with the manager integration gate on `a972cde`, which passed 7/7 at `--timeout 180` in 53.188 s. The packet therefore requires `--timeout 180` and permits a larger timeout only with a recorded reason. Still pre-existing environment timing, still not the writer's to fix. |
| §10: "`C14-LIVE-*` IDs do not yet exist in-tree." | Still true at `a972cde` (`git grep C14-LIVE` returns nothing). The composition writer introduces them as new OPEN handles. |

## 6. Errata recorded against the body, deliberately not corrected in it

The body is preserved verbatim, so the following stay as written. Each is a
label or arithmetic slip whose surrounding conclusion this manager lane
independently recomputed and confirmed.

1. **§6 `W2` pseudocode typo.** The line reads `git hash-object codefier`. The correct command is `git hash-object codefire`; the expected blob `f5e0b6db7c15aeadd3025ae4f038e301a7634caf` is unaffected. The writer must run the corrected command.
2. **§3.1 docs-path count.** The row for `cb13da0..0f43415` says "**24** docs paths", and §3.1's own correction paragraph repeats "24". Recomputed, `git diff --name-status cb13da0..0f43415` yields **27** paths: 2 product and **25** docs. The 25 enumerated paths in that same row are complete and correct — only the headline integer is one low. §4.2's classification of the Upload side sums to exactly 25 (4 union + 5 canonical-newer omit + 10 already-identical + 6 `ADD` from `0f43415`), so no path is unclassified.
3. **§4.2 heading "26 paths from the two lineages".** Recomputed, the union of docs paths across both lineages is **33** distinct paths (12 from `16d0129..b164046`, 25 from `cb13da0..0f43415`, sharing the 4 canonical source docs). The section's own subtotals — 4 union, 5 canonical-newer omit, 18 already-identical (8 MR-list + 10 Upload), 6 `ADD` from `0f43415` plus 1 from `c54e5ce` — sum to 33 and 34 respectively and are individually correct; the heading integer is the only defect.
4. **§4.2 parenthetical placing the terminal MR-list `ACCEPT` `ec7a09b7…` among the three `mr-list` reports carried by the candidate lineage.** Recomputed, that report is **absent** at `b164046` and present at `a972cde` from a separate report commit; the three `mr-list` swarm-results actually added by `16d0129..b164046` are `…mr-list-review-claude-20260730`, `…mr-list-review-closure-rereview-claude-20260730` and `…mr-list-zero-residual-rereview-fugu-20260730`. The operative conclusion — nothing to do on any of those paths, all identical at the base — is unchanged.
5. **Stale `BL-3` wording** as described in section 5, row 1. Preserved as the historical record of the gate the preflight was under.

No other numeric claim in the body failed recomputation in this lane. The
identity, ancestry, patch-id, path-set, blob and report-digest values that this
lane re-derived all matched (§3.4).

## 7. What the preflight authorized

**Nothing.** The body says so twice, in §1.1 and in its §12 confidence table
("Composition is *authorized* to proceed now — **Low, do not assume**"). It
freezes no candidate, accepts nothing, integrates nothing, and closes nothing.
It is a read-only measurement of an executable plan.

The later disposition — the `B1`/`B2`/`B3` adjudication, the exact composition
base binding, the frozen 19-path writer set, the single review path, and the
statement that MR-write remains blocked — is supplied by the manager task packet
named at the top of this file, and by nothing in section 8.

## 8. Preflight handoff body, verbatim

Everything below this line is the preflight agent's exact output, unmodified.

---

# CFR‑168 PYTHON HTTP — EXACT COMPOSITION PREFLIGHT (read‑only)

**No files were changed. Worktree is clean; HEAD is still `fb891b01f45e4d1963c48f06b827f8335150dace`; index empty; no stash; no commit; no report file. All simulation ran in `/tmp/cfr168pf`, now removed. `git hash-object` was used without `-w`, so no objects were written.**

Lane constraints honoured: Claude Opus 5 only; no subagents, no Codex/Fugu, no Cargo/Rust, no product tests, no install/dogfood, no network, no repository mutation.

---

## 1. Exact authorities

### 1.1 Prospective composition base

| Field | Value |
|---|---|
| Base HEAD | `fb891b01f45e4d1963c48f06b827f8335150dace` |
| Base tree | `2dadc3088c14fb624fb30d8060eba7466297959a` |
| Parent (canonical) | `41dea1488bf04c8a2bc1c1d1ad3df7ac9b2570f6` (tree `47b6bc83350fff67386ec92c273f74c75a2d9471`) |
| Additive commits over canonical | 1 · merges 0 |
| Subject | `docs(cfr168): port accepted AIA-046 source-keyed Cargo control onto canonical` |
| Product/runtime/test paths changed `16d0129..fb891b0` | **0** (245 changed paths, all docs/management) |
| `fb891b0:codefire` | `61214a21fd866efc0d9a8888dfbfc8e707039d48`, mode `100755` |

**Decisive base property:** `codefire` is byte‑identical at `5254e3d`, `16d0129`, `41dea14` and `fb891b0` (`61214a21…`). The same holds for all four canonical source docs and `tests/test_codefire_cli.py`. The entire canonical governance/Cargo lineage never touched product. This makes every merge base below unambiguous.

`fb891b0` is still under fresh exact review. **This preflight authorizes nothing.** A later report‑only Cargo review commit, if ACCEPT, touches only `docs/development/ai/swarm-results/…` and must affect no product/runtime path — consistent with `fb891b0` itself being docs‑only.

### 1.2 Governing lifecycle authority

`docs/development/current-cycle-burn-down-plan.md` → *“CFR‑168 closure‑mode checkpoint (2026‑08‑02)”*, frozen order, item **2**:

> *repair and rereview Python Upload/CFB‑138, then **resolve the shared‑primitives preflight before MR‑write and Python composition***

and item 6: *compose accepted Python and Rust inputs once per runtime before starting the predecessor‑gated successors…*

Root row 6 (`CFR-168 + CFB-136 + CFB-138`) state is **`Implement in progress`**. Composition must not alter that row to any closed state. See §9 BL‑1.

### 1.3 Accepted Python inputs — resolved, not assumed

**Input 1 — Doctor + Branch composition**

| Field | Value |
|---|---|
| Candidate | `701469f91dd0c4c52c5628fb3e9d9e9d0ab0c714` |
| Tree | `dd63dd560514aef08925cc8b1dbe815bafcceef4` |
| Parent | `7e87665b8158a37f1d25b449de7831cf9420f545` |
| Product base | `16d012938730fdb2b092b7f8c5cd601b53b81a1f` (tree `93e2ff77…`) |
| Additive commits / merges | 4 / **0** |
| Subject | `test: update branch-list deadline expectation` (clean, frozen) |
| ACCEPT report | `docs/development/ai/swarm-results/cfr168-python-http-doctor-branch-composition-rereview-20260730/review.md` |
| Report blob / SHA‑256 | `e5096bbb3a4da181b0bee471619de61f11b15635` / `c7a1665fdf4a6e1104bbab18ec8051c048ace1d25e865952bbbcd35a06eb19c0` |
| Verdict | **`ACCEPT` for the corrected Doctor + Branch composition only** (Fugu 1.1 Ultra, 2026‑07‑30) |
| Predecessor | `…doctor-branch-independent-review-20260730/review.md`, blob `3824d840…`, sha256 `5d76f8a5…`, **`REQUEST_CHANGES`** — superseded |
| **Report already present in `fb891b0`** | **yes, identical blob** |

⚠️ **Scope carve‑out, quoted verbatim from that ACCEPT:** *“This verdict neither accepts nor rejects the adjacent upload slice from `0703f71b9800ff19ce52d01bcd46c8eddb323df2`; that surface remains under its separate review and CFB‑138 gate.”* This is why the duplicate‑omit proof in §2.3 must rest on the **Upload** acceptance chain, never on this report.

**Input 2 — MR‑list**

| Field | Value |
|---|---|
| Candidate | `b16404676639336f705feffd9af4857d4a1fcc69` |
| Tree | `36185e320338b67e1610d82fcc81cba310c36026` |
| Parent | `091dc2a562ffcb94557b974ac1828c80d09e7ccc` |
| Product base | `16d0129…` (same as Input 1) |
| Additive commits / merges | 24 / **0** |
| Subject | `docs(cfr168): narrow the fallible-boundary claim and record the publication state machine` (clean, frozen) |
| Terminal ACCEPT report | `docs/development/ai/swarm-results/cfr168-python-http-mr-list-final-publication-closure-rereview-claude-20260731/review.md` |
| Report blob / SHA‑256 | `ec7a09b795e0c3df22b6394ae03c465aa90f42c7` / `5c952a9a5c162bfbb93522bb644fcba44ce32ddf3628501e6f94308899fe760e` |
| Verdict | **`ACCEPT`**, *“Scoped to immutable candidate `b164046…` only. It does not authorize integration, Close, Install, Dogfood, accepted Upload/MR‑write bindings, shared‑primitives preflight, Python composition, or any successor protocol.”* |
| Candidate card | `…mr-list-final-publication-closure-repair-candidate-ready-20260731/candidate.md` |
| **Report + card already present in `fb891b0`** | **yes, identical blobs** |

`merge-base(701469f, b164046) == 701469f`. **`b164046` strictly descends from `701469f`.** Input 1 is fully subsumed by Input 2 — there is nothing separate to replay for Doctor/Branch.

**Input 3 — Upload live‑route cleanup**

| Field | Value |
|---|---|
| Candidate | `0f4341561ce95935600bf1398b8a5fbdb206daba` |
| Tree | `da356c2a041e57eea9192ab917f3a293239c60e7` ✔ matches subject |
| Parent | `e7312ef3d25e27b89db35bf5080048e0e4a04a34` (tree `5718defd2e6e767bb4f9ec175a9a197b6e9417cd`) |
| Task base | `5790463864881ee28c372e9bd0b8cd8725c9dc70` (tree `d8c09084…`), subject `docs(cfr168): freeze Upload live-route cleanup repair` |
| Additive commits / merges in `5790463..0f43415` | **2** (`e7312ef`, `0f43415`) / **0** ✔ |
| Changed paths | **4, all `M`** ✔ |
| ACCEPT report commit | `c54e5ce4ab9e8f58f1db88ae59edceae96653058` (parent `0f43415`, docs‑only, 1 file `A`, 747 insertions) |
| Report path | `docs/development/ai/swarm-results/cfr168-python-http-upload-live-route-cleanup-rereview-claude-20260802/review.md` |
| Report blob | `216ee2680872c43dcd7b657e3a8d4312dbd9b1e9` |
| Report SHA‑256 | `46e7c3e41ac1b132766657e445e017b392ce6a0b72eb46f909b13755fc8a9b3c` ✔ **recomputed, exact match** |
| Verdict | **`ACCEPT`** (Claude Opus 5, 2026‑08‑02) |
| `git diff --binary 5790463..0f43415` SHA‑256 | `4ba34a6d92c5a6084f52fcdb978c0a4f0a5687b3fcac4bf019e8b2982dbf2fcb` ✔ **independently recomputed, matches the report’s own table** |
| **Report present in `fb891b0`** | **NO — must be added** |
| Immediate predecessor | `3475a8a` → `…upload-third-repair-rereview-claude-20260731/review.md`, **`REQUEST_CHANGES` on the live-route limb** — **already in `fb891b0`, identical blob, and SUPERSEDED by `c54e5ce`** |

Prior‑finding dispositions carried by the ACCEPT: `BLOCKER-1` CLOSED (preserved), **`HIGH-2` CLOSED — all three limbs**, `HIGH-3` CLOSED (preserved), `MEDIUM-4` CLOSED (preserved).

**Stale‑state warning honoured:** open‑audit artifacts in `fb891b0` (`cfr168-open-execution-audit-20260731/open-items.v1.json`, `cfr168-review-bottleneck-open-item-audit-claude-20260731/audit.md`, `cfr168-review-disposition-index-20260730/index.md`) and the shared‑primitives preflight all still record `HIGH-2`/`B2` as **behavior‑open against `b164046`**. That is *correct about `b164046`* and *stale about the composed tree*. §5 shows the composed tree closes it. Do not treat those files as authority over `c54e5ce`.

---

## 2. Ancestry / patch‑id graph

### 2.1 Actual graph (all lineages linear, zero merges anywhere)

```
5254e3dc8e3cbab3330c2a34469eb56750edf626   "chore: add canonical HTTP upload traversal task"
│   tree 6bc0d28b…   codefire 61214a21   ← COMMON PRODUCT BASE
│
├── UPLOAD LINEAGE (30 commits) ─────────────────────────────────────────────
│    3153d75  wip(recovery): preserve interrupted HTTP upload traversal   ┐ initial
│    cb13da0  test(recovery): make the three interrupted … decisive       ┘ Upload
│      │        codefire 03966af2 · upload-test 141dcbd4 · 10 tests
│      ⋮  (26 commits: CFB-138 scale, zero-residual, review-closure, 3 repairs)
│    3475a8a  REQUEST_CHANGES on the live-route limb            (report-only)
│    5790463  freeze Upload live-route cleanup repair    ← TASK BASE (packet-only)
│    e7312ef  drain commit-source cleanup at both live call sites
│    0f43415  close C14 cleanup precedence and bound the live-route retry  ★ACCEPTED
│    c54e5ce  exact rereview — ACCEPT                            (report-only)
│
└── canonical/management (26 commits, ZERO product paths) ──────────────────
     16d012938730fdb2b092b7f8c5cd601b53b81a1f  "chore: register Doctor and branch independent review"
     │   codefire 61214a21  (UNCHANGED from 5254e3d)
     │
     ├── DOCTOR/MR-LIST LINEAGE ──────────────────────────────────────────
     │    5e3b0c7  probe: compose Python HTTP Doctor ledger
     │    0703f71  probe: compose Python HTTP upload traversal   ◆ DUPLICATE
     │    7e87665  probe: compose Python HTTP branch session
     │    701469f  test: update branch-list deadline expectation ★ACCEPTED (Input 1)
     │      ⋮  (20 commits: MR-list session → zero-residual → closure → final publication)
     │    b164046  narrow the fallible-boundary claim …          ★ACCEPTED (Input 2)
     │        codefire f30575a6 · upload-test 141dcbd4 (still 10 tests)
     │
     └── canonical governance/Cargo (219 more commits, ZERO product paths)
          41dea14  rebaseline closure critical path
          fb891b0  port accepted AIA-046 source-keyed Cargo control ★PROSPECTIVE BASE
              codefire 61214a21  (STILL UNCHANGED)
```

Merge bases: `merge-base(fb891b0, b164046) = merge-base(fb891b0, 701469f) = 16d0129`; `merge-base(fb891b0, 0f43415) = merge-base(b164046, 0f43415) = 5254e3d`; `5254e3d` is an ancestor of `16d0129`. Because `codefire` is the same blob at both, **the effective common product base is a single blob, not a range.**

### 2.2 Duplicate‑equivalent commits — stable patch‑id proof

| Object | `git patch-id --stable` |
|---|---|
| `git show 0703f71` (single commit, Doctor/MR‑list lineage) | **`9889888cedeba7344fb987ff5891cf728d9c7e36`** |
| `git diff 5254e3d cb13da0` (squash of `3153d75`+`cb13da0`, Upload lineage) | **`9889888cedeba7344fb987ff5891cf728d9c7e36`** |
| `git show 3153d75` alone | `8376abdf056db843c72d0fad4e3e836a315d3346` — *not* equivalent alone |
| `git show cb13da0` alone | `8a82303219d78b22c5e342a8956d1b688d8f91ee` — *not* equivalent alone |

Manager’s prior measurement of `9889888c…` is **confirmed exactly**, and refined: the equivalent of `0703f71` is the **pair** `3153d75 + cb13da0`, not `cb13da0` alone. Both sides touch exactly `M codefire` + `A tests/test_python_http_upload_accounted_traversal.py`.

Strengthened beyond patch‑id (which normalizes hunk headers): normalizing away only `@@` line numbers and `index` lines, the two patch texts are **byte‑for‑byte identical**, context lines included.

**Answer to required item 4: YES.** The initial Upload product patch is already present in the accepted Doctor/MR‑list lineage, at `0703f71`, which is an ancestor of both `701469f` and `b164046`.

### 2.3 Which duplicate is omitted from replay, and why

**Omit the Upload lineage’s `5254e3d..cb13da0` limb. Keep `0703f71` as it arrives inside `b164046`.**

Proof of safety, in three independent parts:

1. **Content:** the two are the same patch (§2.2), so “which copy” cannot change bytes.
2. **Realized state:** `tests/test_python_http_upload_accounted_traversal.py` is blob `141dcbd4ee1f` at **all** of `0703f71`, `7e87665`, `701469f`, `b164046` **and** `cb13da0`. The MR‑list lineage never advanced that module past the initial patch; the Upload lineage’s `cb13da0` state is the *identical blob*. So `b164046`’s copy and `cb13da0` are literally the same object — the omission is a no‑op on that path.
3. **Authority:** the omitted copy is not orphaned of acceptance. `0f43415`’s ACCEPT covers a tree whose ancestry contains that exact patch, and CFB‑138 gates governed it throughout. The Doctor/Branch ACCEPT explicitly does **not** cover `0703f71` (§1.3), so it is deliberately *not* cited here.

Consequence: the replay range for Upload is **`cb13da0..0f43415`**, not `5254e3d..0f43415` and not `5790463..0f43415`.

Note the accepted candidate range `5790463..0f43415` is only the **last 2** of 30 Upload commits. Composing only it would leave 26 accepted Upload commits behind and produce a `codefire` that no review ever saw. The correct replay unit is the full post‑initial delta `cb13da0..0f43415`, whose head is the accepted `0f43415` tree.

---

## 3. Path sets and binary patch SHA‑256

All values from `git diff --binary` with default options, hashed with `sha256sum`.

| ID | Range / scope | Paths | SHA‑256 |
|---|---|---|---|
| **P1** | `16d0129..b164046`, product only (`codefire tests`) | 6 | `2e9653fb511e7ad9f35fc56bb91a27d9bfa87fcd780f04af31b44b68d8a87c48` |
| P1a | `16d0129..b164046`, all paths | 18 | `567d98be14b61146cce1372047c486ea49904fcb0416d61f6f76975f4b9e9125` |
| **P2** | `5790463..0f43415`, all paths *(accepted candidate range)* | 4 | `4ba34a6d92c5a6084f52fcdb978c0a4f0a5687b3fcac4bf019e8b2982dbf2fcb` ✔ *matches the ACCEPT report* |
| P2p | `5790463..0f43415`, product only | 2 | `581a65e1e755fb2653932604d4d661320b7d9718326d600cb486f5d7484dd97f` |
| **P3** | `cb13da0..0f43415`, product only *(replay unit)* | 2 | `410fb69b4f147f12dbfc0055c05ba844d38e1eb4692ff223cf93c3d59c64e290` |
| P3d | `cb13da0..0f43415`, the two canonical source docs | 2 | `78edcf0851ee869a07d071825e8c36ac44f07d014bf10404e053dfe6fe3c1413` |
| PD | `16d0129..701469f`, product only | 5 | `51ad535e2ea7b1fc3dd0b335ed328c24b69b3d7539e7da6646cdf6607e087544` |
| PU | `5e3b0c7..0703f71`, product only *(duplicate, kept)* | 2 | `d6761b6f7754f57185ba95733336a8ac6bbfa1f0234fe587c061a1c70f696946` |
| PU′ | `5254e3d..cb13da0`, product only *(duplicate, omitted)* | 2 | `35b3a732fe267025ddf6af943ab47a3e8bff92dfc39b341270efb6ead7879811` |
| **PC** | **`fb891b0` → composed, product only** | 6 | `ed451aed40cccc613d3ca3d2500bc4f857027432a0883a3ceb73723cf6b1c80f` |

PU ≠ PU′ as raw text (differing `index` lines and hunk offsets) while sharing patch‑id `9889888c…` — exactly what §2.2 predicts.

**PC derivation (reproduce exactly, or it will not match):** materialize `fb891b0:{codefire, tests/test_codefire_cli.py}` into dir `A/` (`chmod 755 A/codefire`) and the six §4.1 target blobs into `B/`, then
`git diff --no-index --binary A B | sed -E 's#<abs-parent>/##g' | sha256sum`.
Because that value is prefix‑sensitive, **prefer the blob identities in §4.1 as the authoritative composed‑product proof.** PC is a convenience cross‑check only.

### 3.1 Required item 5 — recomputed

The manager’s statement is **correct as written, once “product delta” is read as product‑path‑restricted.**

| Scope | Product/runtime/test paths | Additional paths |
|---|---|---|
| `cb13da0..0f43415` — *post‑initial Upload successor* | `codefire`, `tests/test_python_http_upload_accounted_traversal.py` — **exactly the two expected** | **24 docs paths** (`README.md`, 6 swarm‑batches, 6 swarm‑results, `bug-backlog.md`, `bug-issues/README.md`, `A bug-issues/cfb-138.md`, `change-playbook.md`, `code-review-issues/cfr-168.md`, `codefire-improvement-cycle.md`, `command-source-trace.md`, `current-cycle-burn-down-plan.md`, `history.md`, `issue-root-cause-map.json`, `source-architecture.md`, `source-data-model-catalog.md`) |
| `5790463..0f43415` — *exact accepted candidate range* | `codefire`, `tests/test_python_http_upload_accounted_traversal.py` | **exactly** `docs/development/source-architecture.md`, `docs/development/command-source-trace.md` ✔ as stated |

Correction to add: the wider replay range carries **24** docs paths, not 2. §4.2 classifies all of them — 10 are already byte‑identical in `fb891b0`, 4 need semantic union, 5 are canonical‑newer and must be omitted, 6 are absent evidence to add. That is the difference between a correct composition and clobbering the governance lineage.

---

## 4. Path‑by‑path merge table

### 4.1 Product / runtime / test — 6 paths, all resolved

| Path | Base blob (`fb891b0`) | Class | Source | Target blob | Target SHA‑256 |
|---|---|---|---|---|---|
| `codefire` (100755) | `61214a21fd86` | **SEMANTIC UNION** (3‑way, see §5) | ours `b164046` `f30575a6` ⊕ theirs `0f43415` `b984c1e7` over base `cb13da0` `03966af2` | **`f5e0b6db7c15aeadd3025ae4f038e301a7634caf`** | `c1aab6acfec90d41378ced3e55809d72e7a0be7f84afc695a9be543de36230d7` |
| `tests/test_codefire_cli.py` | `10317e13deb5` | **BASE_EQUAL clean apply** — base ≡ `16d0129` ≡ `cb13da0` ≡ `0f43415`; only OURS changed it | copy `b164046` | `b23c43ced6ccb69c0bb553f9856c1a7b2145e770` | `0e15fae60ab9d3a54333720e30e2edfba2457d9b108f19e75e5854e0a35c6427` |
| `tests/test_python_http_doctor_decision_work.py` | absent | **ADD** | copy `b164046` | `0f993edd6c6a57ee5ca0d15abc04e2260c3d48cd` | `4b6c65cbe80ab6251a023a58f660b9b9434aa8046cc8244b016844dc0dd68302` |
| `tests/test_python_http_branch_decision_work.py` | absent | **ADD** | copy `b164046` | `c0269ee80e00265d0144520ab975ce7e4bc9b7e3` | `bf2f8d0343fc074d0fb771446cdd6142794333cfca426127c8e6588e412a18c5` |
| `tests/test_python_http_mr_list_decision_work.py` | absent | **ADD** | copy `b164046` | `0d49841a2e682e661d169274e571e198f3408aa7` | `98938b9bfdb1bc4b9add8150618737ee66c315c5b59f7d97ea8023e9edf83474` |
| `tests/test_python_http_upload_accounted_traversal.py` | absent | **ADD, DUPLICATE OMIT + newest‑wins** — `b164046`’s copy is blob `141dcbd4` (10 tests) ≡ `cb13da0`; take `0f43415`’s 36‑test successor instead. No 3‑way merge needed *because ours ≡ base on this path.* | copy `0f43415` | `829836bc0411bf82030286e6f1b0e3d58e574e14` | `31fb356171c2ef08b691bcdc467827321a6c89a9d969574537970acf7effc7fa` |

Only **one** path in the entire composition requires a real 3‑way merge: `codefire`.

### 4.2 Docs — 26 paths from the two lineages

**Semantic union (4 paths).** Base blob is identical to `cb13da0` on all four, so the 3‑way base is unambiguous. All four merge with **zero conflicts**:

| Path | base = `cb13da0` = `fb891b0` | ours `b164046` | theirs `0f43415` | merged blob | lines b/o/t/m |
|---|---|---|---|---|---|
| `docs/development/source-architecture.md` | `0fe3dc8470ee` | `acc6c98cf73f` | `cac0adb178be` | `d8d7aa9881b5` | 744 / 823 / 842 / **921** |
| `docs/development/command-source-trace.md` | `5feeaff7784c` | `a7bd13baff7a` | `809629ddb96a` | `0a8d2a886c88` | 239 / 294 / 269 / **324** |
| `docs/development/change-playbook.md` | `12d7fc6ab411` | `a14c39647a5c` | `3ce2eb489661` | `9a3afd7dd932` | 355 / 433 / 399 / **477** |
| `docs/development/source-data-model-catalog.md` | `77623025f184` | `e416eefeb26b` | `a10683a226ec` | `2b68347eaa35` | 490 / 532 / 492 / **534** |

Note `change-playbook.md` and `source-data-model-catalog.md` are byte‑identical at `5790463` and `0f43415` — their Upload‑side edits land *before* the accepted range but *inside* the accepted lineage. Since the composition ships the full `0f43415` product state, AGENTS.md rule 41 requires the docs to describe that full state, so include all four. Merged blobs above are the exact `git merge-file --diff3 ours base theirs` outputs.

**Management‑evidence‑only OMIT — canonical is newer authority (5 paths).** `fb891b0` has diverged forward on each; the Upload‑side edits predate `5790463` and canonical has since superseded them. Never `checkout` these from a candidate:

`docs/development/README.md` · `docs/development/current-cycle-burn-down-plan.md` · `docs/development/history.md` · `docs/development/codefire-improvement-cycle.md` · `docs/development/ai/swarm-batches/cfr168-recovery-execution-ledger-20260730.md`

**Already IDENTICAL in `fb891b0` — omit, nothing to do (18 paths).**
From MR‑list lineage: all 5 `swarm-batches/cfr168-python-http-mr-list-*` and all 3 `swarm-results/cfr168-python-http-mr-list-*` (incl. the terminal ACCEPT `ec7a09b7…`).
From Upload lineage: `swarm-batches/…upload-staging-scale-20260730.md`, `…upload-zero-residual-review-closure-repair-20260731.md`, `swarm-results/…upload-scale-review-claude-fallback-20260730/review.md`, `…upload-third-repair-rereview-claude-20260731/review.md` *(the superseded `REQUEST_CHANGES`)*, `…upload-zero-residual-rereview-fugu-20260730/review.md`, `bug-backlog.md`, `bug-issues/README.md`, `bug-issues/cfb-138.md`, `code-review-issues/cfr-168.md`, `issue-root-cause-map.json`.

**ADD — immutable acceptance evidence absent from base (7 paths).**

| Path | Source | Blob | Kind |
|---|---|---|---|
| `docs/development/ai/swarm-results/cfr168-python-http-upload-live-route-cleanup-rereview-claude-20260802/review.md` | `c54e5ce` | `216ee2680872c43dcd7b657e3a8d4312dbd9b1e9` | **terminal ACCEPT for `0f43415`**, sha256 `46e7c3e4…` |
| `docs/development/ai/swarm-batches/cfr168-python-http-upload-live-route-cleanup-repair-20260802.md` | `0f43415` | `250703ae50afcf842359abebc813cea717679c6a` | accepted task packet, sha256 `756bf200138f6066dc1f2ac13d4933cf3e4a4228742bfc079d0c1ce3da2dabe9`, 6,395 bytes — the packet the ACCEPT binds to |
| `docs/development/ai/swarm-results/cfr168-python-http-upload-high2-third-repair-20260731/result.md` | `0f43415` | — | implementer result for the HIGH‑2 third repair |
| `docs/development/ai/swarm-results/cfr168-python-http-upload-zero-residual-review-closure-repair-20260731/result.md` | `0f43415` | — | implementer result |
| `docs/development/ai/swarm-results/cfr168-python-http-upload-zero-residual-review-closure-repair-20260731/result-2.md` | `0f43415` | — | implementer result |
| `docs/development/ai/swarm-batches/cfr168-python-http-upload-zero-residual-repair-20260730.md` | `0f43415` | — | accepted packet |
| `docs/development/ai/swarm-batches/cfr168-python-http-upload-zero-residual-repair-20260730.json` | `0f43415` | — | accepted packet (machine) |

**Immutable acceptance evidence** (must add): the ACCEPT report `216ee268…` and its bound packet `250703ae…`.
**Supporting implementer evidence** (should add, completes the chain the ACCEPT cites): the 3 `result*.md` and 2 zero‑residual packets.
**Stale / superseded** (already in base, leave untouched, do **not** delete): `…upload-third-repair-rereview-claude-20260731/review.md` (`REQUEST_CHANGES`) and the three open‑audit artifacts named in §1.3.
**`.claude-second-agent` / agent‑state / log / worktree debris:** none exists to import. Verified `.gitignore` covers `.claude-second-agent/`, `.claude-worktrees/`, `.codex-second-agent/`, `.fugu-*`, `.dev-swarm/`; and `git ls-tree -r` over `701469f`, `b164046`, `0f43415`, `c54e5ce`, `fb891b0` yields the **same 4** matches in every tree — `.codex/agents/{high,medium,ultra}.toml` and `.codex/config.toml`, pre‑existing tracked config, **not debris**. Zero debris risk on these inputs.

---

## 5. Overlapping `codefire` hunks and test modules

### 5.1 The 3‑way merge, and why `cb13da0:codefire` is the right base

`cb13da0:codefire` (`03966af2`) is not a literal git‑ancestor blob of `b164046:codefire`, but it **is** the correct base *in patch space*: `b164046 = base ⊕ (Doctor+Branch+MR‑list)` and `0f43415 = base ⊕ (Upload successor)`, because the initial Upload patch is already inside `base` on both sides and is textually identical there (§2.2). This is precisely what makes the duplicate collapse rather than double‑apply.

`git merge-file -p --diff3 ours base theirs`:

```
exit 0 · conflicts 0 · markers 0
base 33,633  ours 35,791  theirs 35,129  →  merged 37,287
37,287 == 35,791 + (35,129 − 33,633)   ← exactly additive
```

### 5.2 Hunk disjointness

OURS: **60 hunks**, base ranges `1432–2310` (JSON/accounting helpers, remote project gate) and `11365–15519` (`AccountedHTTPBundle`, `collect_http_bundle_payload`, `handle_http_request_merge`, `make_http_handler`, `_PreparedHTTPResponseOwner`, `_HTTPMutationResponseSlot`, Doctor payload owners).
THEIRS: **52 hunks**, base ranges `1082–1087`, `2722–3340`, `6616–6728`, `11346–11360`, `12224–12528` (`handle_http_upload`), `20913–23456`, `28954+`.

Closest approach is THEIRS `11346–11360` vs OURS `11365+` — a **5‑line gap**, adjacent but non‑overlapping, and the merge confirms it.

### 5.3 Semantic disjointness — the decisive proof

Textual non‑conflict is necessary, not sufficient. Verified at Python‑AST granularity on the merged file:

| Check | Result |
|---|---|
| `python3 -m py_compile` on the merged `codefire` | **PASS** |
| Top‑level units (def/class) changed by OURS | 43 |
| Top‑level units changed by THEIRS | 18 |
| Units changed by **both** | **NONE** |
| OURS‑changed units not byte‑identical in merged | **NONE** |
| THEIRS‑changed units not byte‑identical in merged | **NONE** |
| Units untouched by both that drifted in merged | **NONE** (of 412) |
| Units present in a side but missing from merged | **NONE** |
| Qualified‑name collisions (incl. `Class.method`), 77 added by OURS vs 48 by THEIRS | **NONE** |
| Duplicate qualified definitions anywhere in merged | **NONE** |
| Module‑level statements: `merged == union(ours, theirs)` | **True** — 84 → 102; 10 added by OURS, 8 by THEIRS, 0 removed, 0 extra, 0 missing |
| Module‑level **evaluation order** (`SEALED_SESSION_CONTROL_UPPER` at 25118 needs `SealedCommitValidationSession` 23940, `OBJECT_PREFIXES` 1592, `SEALED_SESSION_PATH_BYTES` 25110) | **OK** — all deps defined before use |

**Each accepted change lands exactly once, byte‑identical to its accepted source, with zero collateral drift.** This is the strongest available evidence short of executing the suites.

One AST signal needs interpretation: OURS “removes” `AccountedHTTPBundle.__del__` and `AccountedHTTPBundle.close`. Inspected — it is a **hoist, not a removal**: OURS extracts `class _AccountedHTTPProjection(dict)` with `_OWNER_LABEL`, and `AccountedHTTPBundle` becomes `class AccountedHTTPBundle(_AccountedHTTPProjection)`, inheriting both. THEIRS adds `__del__`/`close` only on its own new upload classes and never references `AccountedHTTPBundle`. No behavioural loss.

### 5.4 The B2 / C14 resolution — measured

The shared‑primitives preflight holds `C14` behavior‑open at `b164046:7047`–`7049` and `B2` on `b164046:6994`–`6999`. Occurrences of the inverted form `raise cleanup_error from run_error`:

| Tree | Count | Locations |
|---|---|---|
| base `cb13da0` | 2 | `6722` in **`enforce_remote_commit_signature_policy_accounted`** (live upload route), `8571` in `run_server_verification` |
| ours `b164046` | 2 | `7049` in `enforce_remote_commit_signature_policy_accounted` ← **the exact cited B2 site**, `8898` in `run_server_verification` |
| theirs `0f43415` | 1 | `9326` in `run_server_verification` |
| **merged** | **1** | **`9653` in `run_server_verification`** |

**The composition eliminates the `enforce_remote_commit_signature_policy_accounted` inversion — the precise defect that makes `b164046` and the shared‑primitives preflight contradict each other.** The composition *is* the resolution mechanism for B2’s product half. The one surviving inversion is in `run_server_verification` / `AccountedServerVerification`, which the ACCEPT explicitly lists as *“untouched; not in the diff — separately routed composition blocker.”* It is **pre‑existing on both lineages**, not composition‑introduced. It must be named and left OPEN (§8, `C14-LIVE-A1`).

`_drained_commit_source_cleanup_error` — the new synchronous drain helper — appears **5×** in `theirs` and **5×** in `merged`: both live call sites (`is_ancestor_from_objects_accounted`, `enforce_remote_commit_signature_policy_accounted`) carry it through intact.

### 5.5 Test modules

Sufficiency of applying **only** the post‑initial Upload delta: **YES**, on this specific evidence — `tests/test_python_http_upload_accounted_traversal.py` is blob `141dcbd4` at `cb13da0` *and* at `b164046`, so ours ≡ base on that path and taking `0f43415`’s blob is a clean fast‑forward, not a merge. Growth: `141dcbd4` 10 → `d9f5bf96` 33 (`5790463`) → `5e1a3bb4` 34 (`e7312ef`) → `829836bc` 36 (`0f43415`).

| Module | Source | Methods |
|---|---|---|
| `test_python_http_doctor_decision_work.py` | `b164046` | 19 |
| `test_python_http_branch_decision_work.py` | `b164046` | 13 |
| `test_python_http_mr_list_decision_work.py` | `b164046` | 49 |
| `test_python_http_upload_accounted_traversal.py` | `0f43415` | 36 |
| `test_codefire_cli.py` | `b164046` | 228 (unchanged count; one Branch deadline expectation corrected) |

Arithmetic, cross‑validated against the ACCEPT report’s own `457/457`:

```
test_python_*  : fb891b0 421  ·  0f43415 457 (=421+36) ✔  ·  b164046 512 (=421+19+13+49+10)  ·  701469f 463
composed       : 421 + 19 + 13 + 49 + 36 = 538 test_python_* in 34 modules
total          : 538 + 228 = 766 in 35 modules   (base: 649 in 31)
```

**Cross‑module residual risk, unresolvable by static analysis:** `test_codefire_cli.py` is taken from `b164046`, which was validated against `b164046:codefire` — a build **without** the Upload successor delta. The Upload lineage never modified that file, and its version is base‑identical, so no accepted review has ever executed `b164046:test_codefire_cli.py` against `merged:codefire`. This is the single genuine composition‑only unknown. It is why V6/V7 in §7 are mandatory, not optional.

### 5.6 Does the MR‑list lineage already include Doctor/Branch and initial Upload?

**Yes, all three.** `merge-base(701469f, b164046) == 701469f`, and `16d0129..b164046` contains `5e3b0c7` (Doctor), `0703f71` (Upload initial), `7e87665` (Branch), `701469f` (test correction). Input 1 needs no separate replay; `b164046` alone carries it.

---

## 6. Proposed writer procedure

Semantic apply throughout. **Pseudocode only — nothing below was executed.** No `git cherry-pick` of any range: both candidate lineages carry management docs that would clobber the canonical governance/Cargo lineage.

```
# --- W0. verify start state; abort on any mismatch -----------------------
git rev-parse HEAD                      # expect fb891b01f45e4d1963c48f06b827f8335150dace
git rev-parse HEAD^{tree}               # expect 2dadc3088c14fb624fb30d8060eba7466297959a
git rev-parse HEAD:codefire             # expect 61214a21fd866efc0d9a8888dfbfc8e707039d48
git status --porcelain                  # expect empty

# --- W1. product fast-forwards from b164046 (base blob is identical) ----
git checkout b164046 -- codefire \
    tests/test_codefire_cli.py \
    tests/test_python_http_doctor_decision_work.py \
    tests/test_python_http_branch_decision_work.py \
    tests/test_python_http_mr_list_decision_work.py

# --- W2. Upload successor delta onto codefire (3-way; base = cb13da0) ---
#   NOT `git apply` against the working tree: the working codefire is now
#   b164046's, and the delta's context is cb13da0's.
git show cb13da0:codefire > /tmp/w/base
git show 0f43415:codefire > /tmp/w/theirs
cp codefire /tmp/w/ours
git merge-file -p --diff3 /tmp/w/ours /tmp/w/base /tmp/w/theirs > codefire
chmod 755 codefire                       # preserve mode 100755
#   expect: exit 0, zero '<<<<<<<' markers
git hash-object codefier                 # expect f5e0b6db7c15aeadd3025ae4f038e301a7634caf

# --- W3. upload test module: newest wins, duplicate omitted -------------
git checkout 0f43415 -- tests/test_python_http_upload_accounted_traversal.py
#   expect blob 829836bc0411bf82030286e6f1b0e3d58e574e14 (36 tests, not b164046's 10)

# --- W4. four canonical source docs: semantic union, base = cb13da0 -----
for d in source-architecture command-source-trace change-playbook source-data-model-catalog; do
  git show cb13da0:docs/development/$d.md > /tmp/w/$d.base
  git show b164046:docs/development/$d.md > /tmp/w/$d.ours
  git show 0f43415:docs/development/$d.md > /tmp/w/$d.theirs
  git merge-file -p --diff3 /tmp/w/$d.ours /tmp/w/$d.base /tmp/w/$d.theirs \
      > docs/development/$d.md
done
#   expect zero conflicts and blobs
#     source-architecture      d8d7aa9881b5   command-source-trace  0a8d2a886c88
#     change-playbook          9a3afd7dd932   source-data-model     2b68347eaa35

# --- W5. evidence ADDs only; never the 5 canonical-newer management docs -
git checkout c54e5ce -- \
  docs/development/ai/swarm-results/cfr168-python-http-upload-live-route-cleanup-rereview-claude-20260802/review.md
git checkout 0f43415 -- \
  docs/development/ai/swarm-batches/cfr168-python-http-upload-live-route-cleanup-repair-20260802.md \
  docs/development/ai/swarm-batches/cfr168-python-http-upload-zero-residual-repair-20260730.md \
  docs/development/ai/swarm-batches/cfr168-python-http-upload-zero-residual-repair-20260730.json \
  docs/development/ai/swarm-results/cfr168-python-http-upload-high2-third-repair-20260731/result.md \
  docs/development/ai/swarm-results/cfr168-python-http-upload-zero-residual-review-closure-repair-20260731/result.md \
  docs/development/ai/swarm-results/cfr168-python-http-upload-zero-residual-review-closure-repair-20260731/result-2.md
#   FORBIDDEN, do not check out from any candidate:
#     docs/development/README.md  current-cycle-burn-down-plan.md  history.md
#     codefire-improvement-cycle.md  ai/swarm-batches/cfr168-recovery-execution-ledger-20260730.md

# --- W6. hand-authored edits (see 6.1) ---------------------------------
#   O1 citation in the two merged source docs; C14-LIVE-A1/A2/SUCCESSOR
#   registration in code-review-issues/cfr-168.md; composition status line
#   in current-cycle-burn-down-plan.md. Edit in place — never checkout.

# --- W7. validate (section 7), then freeze (section 7.4) ---------------
```

### 6.1 Hand‑authored edits — exact semantic union markers

**`docs/development/source-architecture.md`.** Both sides are **pure insertions at disjoint anchors**; no reconciliation needed, no heading collisions (verified: 0 headings added by either side, 0 duplicate headings in merged).

- THEIRS inserts **98 lines after base line 390**, anchor: the paragraph ending `…escape through this handoff, and later signature/verification/digest/ancestry/pu…`. Contains the Upload C14/`_drained_commit_source_cleanup_error` narrative including *“That synchronous guarantee is scoped to those two commit‑source routes and to nothing else.”* — **preserve that scoping sentence verbatim; it is what keeps the ACCEPT honest.**
- OURS inserts **79 lines after base line 714**, anchor: `…one final stdout write. Subject drift releases every snapshot/projection and rec…`. Contains `_AccountedHTTPProjection` / `AccountedHTTPBundle` and the MR‑list publication state machine.
- Merged: 921 lines = 744 + 79 + 98.

**`docs/development/command-source-trace.md`.**

- THEIRS: line 81 → 10 lines (upload three‑first‑effects deadline poll, single‑use publication‑root proof) and line 97 → 22 lines (`SealedCommitValidationSession`, `_AccountedUploadCommitSource`, retryable cleanup, `_drained_commit_source_cleanup_error`, the C14 precedence rule, and *“preflightがbehaviour-open `B2`として記録していたものをここで解消する”* — the explicit B2 resolution claim, which §5.4 independently confirms).
- OURS: **pure insertion of 55 lines after base line 220**, anchor `…pre-effect closeだけがexact hidden identityを削除し、effect後はreducer…`, plus new heading `### Filesystem authority failures on read and mutation routes` (unique, no collision).
- Merged: 324 lines = 239 + 55 + 30. Disjoint (81/97 ≪ 220).

**Cargo / AIA governance preservation.** `fb891b0`’s AIA‑046 source‑keyed Cargo content lives in `docs/development/ai/source-keyed-cargo-gates.md`, `ai/swarm-batches/`, `ai/swarm-results/`, `docs/development/tools/run_source_keyed_cargo_gate.py` and `current-cycle-burn-down-plan.md` item 1. **None** of those paths appears in any candidate lineage’s diff, and W1–W5 never touch them. Preservation is structural, not merely intended.

**Required O1 remediation (composition‑owned, and now cheap).** The ACCEPT’s Observation O1 says the candidate docs cite `C14`/`B2`/“shared‑primitives” with *“no shared-primitives document, no `C14` record and no `B2` record anywhere in the repository, and none at the base.”* **That was true at the reviewer’s base `5790463` and is FALSE at `fb891b0`.** The authority exists on the canonical lineage:

`docs/development/ai/swarm-results/cfr168-python-http-shared-primitives-preflight-claude-20260731/preflight.md`
blob `ec2a264142c6`, SHA‑256 `ca1a1451bfc1b25ed1c54f1b012d4d0e508ef0a96f88401e61284c6d156a4441`, introduced by `6792f86`; `C14` defined in §2 per‑clause register, `B2` in §0 decision table. Present at `41dea14` and `fb891b0`; **absent** at `16d0129`, `cb13da0`, `b164046`, `5790463`, `0f43415`.

So O1 resolves by **adding a relative link to that existing path** at the `C14`/`B2` mentions in the two merged source docs. Do **not** author a new authority. The reviewer’s alternative (“restate the rule without the labels”) is now the inferior option — the labels are resolvable on this base.

---

## 7. Exact allowed write set

### 7.1 Writer — one bounded lane, 18 paths

| # | Path | Op |
|---|---|---|
| 1 | `codefire` | 3‑way union (mode `100755`) |
| 2 | `tests/test_codefire_cli.py` | clean apply from `b164046` |
| 3 | `tests/test_python_http_doctor_decision_work.py` | ADD from `b164046` |
| 4 | `tests/test_python_http_branch_decision_work.py` | ADD from `b164046` |
| 5 | `tests/test_python_http_mr_list_decision_work.py` | ADD from `b164046` |
| 6 | `tests/test_python_http_upload_accounted_traversal.py` | ADD from `0f43415` (duplicate omit) |
| 7 | `docs/development/source-architecture.md` | union + O1 citation |
| 8 | `docs/development/command-source-trace.md` | union + O1 citation |
| 9 | `docs/development/change-playbook.md` | union |
| 10 | `docs/development/source-data-model-catalog.md` | union |
| 11 | `docs/development/ai/swarm-results/cfr168-python-http-upload-live-route-cleanup-rereview-claude-20260802/review.md` | ADD from `c54e5ce` |
| 12 | `docs/development/ai/swarm-batches/cfr168-python-http-upload-live-route-cleanup-repair-20260802.md` | ADD from `0f43415` |
| 13 | `docs/development/ai/swarm-batches/cfr168-python-http-upload-zero-residual-repair-20260730.md` | ADD from `0f43415` |
| 14 | `docs/development/ai/swarm-batches/cfr168-python-http-upload-zero-residual-repair-20260730.json` | ADD from `0f43415` |
| 15 | `docs/development/ai/swarm-results/cfr168-python-http-upload-high2-third-repair-20260731/result.md` | ADD from `0f43415` |
| 16 | `docs/development/ai/swarm-results/cfr168-python-http-upload-zero-residual-review-closure-repair-20260731/result.md` | ADD from `0f43415` |
| 17 | `docs/development/ai/swarm-results/cfr168-python-http-upload-zero-residual-review-closure-repair-20260731/result-2.md` | ADD from `0f43415` |
| 18 | `docs/development/code-review-issues/cfr-168.md` | hand edit — register `C14-LIVE-A1/A2/SUCCESSOR` as **OPEN** |

**Optional 19th, manager‑gated:** `docs/development/current-cycle-burn-down-plan.md` — one status line recording the Python composition, root row 6 left at `Implement in progress`. It is the highest‑risk path in the set (lifecycle authority, and BL‑1/BL‑2 unresolved). Recommend deferring to the manager.

**Explicitly forbidden to the writer:** the 5 canonical‑newer management docs (§4.2); any `crates/`, `Cargo.*`, `install.sh`, `pyproject.toml`, `setup.py`, `demo.sh`, `schemas/`, `adr/`, `README.md` path; the 18 already‑identical docs; deleting the superseded `REQUEST_CHANGES` report or the stale open‑audit artifacts; any `.claude-second-agent`/`.claude-worktrees`/`.codex-*`/`.fugu-*`/`.dev-swarm` path.

### 7.2 Fresh composition review — report‑only, 1 path

`docs/development/ai/swarm-results/cfr168-python-composition-exact-review-<provider>-20260802/review.md` — ADD only.

Reviewer must independently recompute: the §3 patch SHA‑256s, the §4.1 blob identities, the §2.2 patch‑id, the §5.4 inversion census, the §5.3 AST disjointness, and the §5.5 counts. Reviewer must not write any path in §7.1.

### 7.3 No same‑file sibling writers

Two lanes, disjoint path sets, zero intersection. Sequential: writer completes and freezes a candidate **before** the review lane starts. The review lane reads an immutable candidate SHA and writes exactly one new file under its own result directory.

### 7.4 Post‑composition candidate freeze

```
git add -A && git commit          # one commit, subject e.g.
#   "feat(cfr168): compose accepted Python HTTP Doctor, Branch, MR-list and Upload inputs"
#   body: name every input SHA, the omitted duplicate patch-id 9889888c…,
#         the C14-LIVE-A1/A2/SUCCESSOR OPEN residuals, and state explicitly
#         that this closes neither CFR-168 nor CFB-138 nor global C14.
CAND=$(git rev-parse HEAD)
git rev-parse HEAD^{tree}
git rev-list --count fb891b0..HEAD          # expect 1
git rev-list --merges fb891b0..HEAD | wc -l # expect 0
git diff --name-status fb891b0..HEAD        # expect exactly the 18 (or 19) paths
git diff --binary fb891b0..HEAD | sha256sum
git diff --binary fb891b0..HEAD -- codefire tests | sha256sum   # expect ed451aed… family; see §3 note
python3 docs/development/tools/capture_candidate_identity.py    # per repo convention
git status --porcelain                      # expect empty
```
Record the candidate SHA, tree, patch SHA‑256, the 6 product blob ids, and both test counts (`538` / `766`) in a candidate card. The freeze commit must **not** be created by the review lane.

---

## 8. Validation matrix

None of these was run here — the lane is preflight and read‑only. Marked ✅ are already **statically confirmed** in this preflight and need only re‑confirmation on the real tree.

### 8.1 Identity and patch proof

| ID | Command | Expected |
|---|---|---|
| I1 | `git rev-parse HEAD^{tree}` | new composed tree, recorded |
| I2 | `git rev-list --count fb891b0..HEAD` / `--merges … \| wc -l` | `1` / `0` |
| I3 | `git diff --name-status fb891b0..HEAD` | exactly the §7.1 set, no extras |
| I4 | `git hash-object codefire` | `f5e0b6db7c15aeadd3025ae4f038e301a7634caf` ✅ |
| I5 | `git ls-tree HEAD -- codefire` | mode `100755` ✅ |
| I6 | `for f in <4 docs>; do git hash-object $f; done` | `d8d7aa9881b5…`, `0a8d2a886c88…`, `9a3afd7dd932…`, `2b68347eaa35…` ✅ |
| I7 | `sha256sum tests/test_python_http_upload_accounted_traversal.py` | `31fb3561…` (36 tests, not `b164046`’s 10) ✅ |
| I8 | `git diff --binary fb891b0..HEAD \| sha256sum` | record; product‑only cross‑check vs `ed451aed…` per §3 note |
| I9 | `git show <report>:… \| sha256sum` | `46e7c3e4…` ✅ and packet `756bf200…` ✅ |
| I10 | `git diff --check` · `git status --porcelain` | clean / empty |
| I11 | `grep -c '<<<<<<<\|>>>>>>>\|=======' codefire docs/development/{source-architecture,command-source-trace,change-playbook,source-data-model-catalog}.md` | `0` everywhere ✅ |

### 8.2 Focused Python

| ID | Command | Expected |
|---|---|---|
| V1 | `python3 -m py_compile codefire tests/test_codefire_cli.py tests/test_python_http_doctor_decision_work.py tests/test_python_http_branch_decision_work.py tests/test_python_http_mr_list_decision_work.py tests/test_python_http_upload_accounted_traversal.py` | PASS ✅ (`codefire` already compiles) |
| V2 | `python3 -m unittest tests.test_python_http_upload_accounted_traversal -v` | **36/36 OK** (`0f43415` measured 36/36 in 105.4s) |
| V3 | `python3 -m unittest tests.test_python_http_mr_list_decision_work -v` | **49/49 OK** |
| V4 | `python3 -m unittest tests.test_python_http_doctor_decision_work tests.test_python_http_branch_decision_work -v` | **32/32 OK** (19+13) |
| V5 | `python3 -m unittest tests.test_python_ancestry_owner tests.test_python_signature_owner tests.test_python_signature_terminal_freeze tests.test_python_sealed_production tests.test_python_external_production -v` | **59/59 OK** — the Upload ACCEPT’s neighbour set, unchanged by composition |

### 8.3 Broad Python — the composition‑specific gates

| ID | Command | Expected |
|---|---|---|
| **V6** | `python3 -m unittest discover -s tests -p 'test_python_*.py'` | **538/538 OK** in 34 modules (base 421 + 19 + 13 + 49 + 36). ~5–8 min. |
| **V7** | `python3 -m unittest tests.test_codefire_cli -v` | **228/228 OK** — **the mandatory composition‑only gate (§5.5).** No accepted review has run `b164046`’s copy against `merged:codefire`. `-k upload` (27/27) is a useful fast pre‑check but **is not a substitute**. |
| V8 | `python3 -m unittest discover -s tests -v` | **766/766 OK** in 35 modules — the AGENTS.md ¶60 gate |
| V9 | *module‑level execution check* — any invocation that imports `codefire` (V1 only compiles; V6/V7 cover this implicitly) | PASS. Static order verified ✅, but `SEALED_SESSION_CONTROL_UPPER` evaluates `SealedCommitValidationSession.control_state_upper(...)` at import time, so it must be executed at least once. |
| V10 | `./demo.sh` | AGENTS.md ¶62. Record honestly if not run. |

### 8.4 Docs and governance checkers

| ID | Command | Expected |
|---|---|---|
| D1 | `python3 docs/development/tools/check_source_docs.py` | PASS — `44 modules, 34 commands, 12 object kinds` (verified PASS at `fb891b0`; composition adds no Rust module/command/object kind) |
| D2 | `python3 docs/development/tools/check_project_docs.py` | PASS — `683` Markdown at `fb891b0` → **`690`** after the 7 doc ADDs (`+1` more with the review report) |
| D3 | `python3 docs/development/tools/check_ai_contracts.py` | PASS (verified at base) |
| D4 | `python3 docs/development/tools/check_ai_acceleration_docs.py` | PASS (verified at base) |
| **D5** | `python3 docs/development/tools/check_ai_development.py` **`--timeout 300`** | PASS. ⚠️ **The default `--timeout 60` FAILS at `fb891b0` today** — `tool_tests` runs `python3 -m unittest docs/development/tools/test_ai_tools.py`, which takes **60–63s** against a 60.0s default. Root‑caused: `{"name":"tool_tests","ok":false,"returncode":null,"error":"timeout after 60.0s"}`. Standalone it is `Ran 218 tests … OK`. **Pre‑existing, environment‑timing, not composition‑caused, and not the writer’s to fix.** Confirmed `--timeout 300` → all 7 PASS. |
| D6 | `git diff fb891b0..HEAD -- docs/development/ai/source-keyed-cargo-gates.md docs/development/tools/run_source_keyed_cargo_gate.py` | **empty** — Cargo/AIA governance untouched |
| D7 | `grep -n 'shared-primitives\|C14\|B2' docs/development/{source-architecture,command-source-trace}.md` | every mention resolves to the in‑tree authority path (O1 remediation landed) |

Do **not** run `check_swarm_admission.py`, `check_review_terminalization.py`, `check_review_recurrence.py` blind — each requires mandatory args (`--resource-class/--provider/--provider-active/--provider-window`; `--events --report`; `--record`) and exits `2` without them. They gate lane admission and successor‑review recurrence, not this composition’s content.

### 8.5 Non‑vacuity and reversals

Semantic composition can silently produce oracles that pass for the wrong reason. Re‑establish, on the composed tree, the reversals the accepted reports already pinned:

| ID | Check | Expected |
|---|---|---|
| R1 | Nine single‑edit reversals `r1`–`r6b` and `m2` named in the Upload ACCEPT §Gates | **each FAILS at the boundary it names** |
| R2 | The Upload ACCEPT’s production accounting probe — `make_http_handler` → `handle_one_request` `finally`, read through `_decision_work_observer` — real socket POST with one real `close(2)` fault | `current = {}`, `open_permit_count = 0`, `terminal_reason = 'complete'`, `0` response bytes, `0` durable bytes. Pre‑repair triple was `retained_bytes = 11,456 / 0 / None`. |
| R3 | `close(2)` call‑count bound: **8** faults armed, both live routes | exactly **2** `close` calls on the faulting source, exactly **1** on every other source |
| R4 | C14 precedence by **exception object identity**, not type or message | primary decision is the outer exception; first cleanup error attaches as `__cause__` |
| R5 | MR‑list reversals from the `5c952a9a…` ACCEPT | each FAILS at its named boundary |
| R6 | Branch deadline reversal — revert `test_codefire_cli.py:6273-6280` to the pre‑fix `200 OK`/zero‑poll expectation | **FAILS**, proving the corrected expectation is load‑bearing (this was Doctor/Branch `MUST-1`) |
| **R7** | **Composition‑specific inversion census:** `grep -n 'raise cleanup_error from run_error' codefire` | **exactly 1 occurrence, inside `run_server_verification` only.** `0` occurrences in `enforce_remote_commit_signature_policy_accounted`. ✅ statically verified — a second occurrence there means the Upload delta was lost. |
| **R8** | `grep -c '_drained_commit_source_cleanup_error' codefire` | **5** ✅ |
| R9 | Three‑run stability of the three new Upload live‑route tests (real socket, real server thread, `InjectedClock`) | 3× `3/3 OK`, no port/PID/timing flake |

### 8.6 Not run in this lane, and not to be claimed

No Cargo/Rust (`cargo fmt/clippy/test`), no `install.sh`, no dogfood, no network, no Docker, no product test. Record each as **not run**, never as PASS.

---

## 9. Review contract for the fresh composition review

- **Subject:** the frozen composed candidate SHA + tree from §7.4, over base `fb891b0…` (tree `2dadc308…`), plus the 18/19‑path write set. Report‑only; single write path per §7.2.
- **Independence:** fresh agent, session and worktree. No inherited implementer conversation, WIP, `/tmp` state or conclusion. Reviewer‑owned scratch prefix; must not call the candidate’s own test helpers when reproducing oracles.
- **Must independently recompute, not accept from this handoff:** patch‑id `9889888c…`; the §3 patch SHA‑256s (esp. `4ba34a6d…` and `2e9653fb…`); the six §4.1 product blob ids; the four §4.2 merged doc blob ids; the §5.4 inversion census; the §5.3 AST disjointness (43 / 18 / 0 overlap / 0 drift / 0 collisions); the module‑level union; the §5.5 counts `538` / `766`.
- **Must verify the negative space:** zero product paths differ between `16d0129` and `fb891b0`; the 5 canonical‑newer management docs are byte‑identical to `fb891b0`; the 18 already‑identical docs unchanged; Cargo/AIA governance untouched (D6); no `.claude-second-agent`/log/worktree debris.
- **Must confirm the duplicate is omitted exactly once:** `tests/test_python_http_upload_accounted_traversal.py` has 36 methods, not 10 and not 46; the initial Upload patch appears once in `codefire`.
- **Must adjudicate BL‑1 and BL‑2 (§10) explicitly.** A verdict that ignores the burn‑down ordering gate or the root‑row status is incomplete.
- **Verdict scoping — mandatory language.** Any ACCEPT must state that it is scoped to the immutable composed candidate only and does **not** close CFR‑168, CFB‑136, CFB‑138, all of Upload, global C14, MR‑write, integration, Install, Dogfood, or any predecessor‑gated successor; and must not move root row 6 off `Implement in progress`. Reject any `Close`/`Closed`/`complete`/`zero-residual` phrasing applied to the root, the cycle, or global C14.
- **Must reproduce, not infer, R2/R3/R4 and V7.** V7 is the gate no prior review has executed.
- Two consecutive required‑correction rounds on the same mechanism return that mechanism to Plan/Preflight, per the burn‑down control.

---

## 10. Routed residuals — remain OPEN, named as dependencies

Composition **must not** fix, close, or appear to close any of these. Register in `docs/development/code-review-issues/cfr-168.md` as **OPEN**. Note `C14-LIVE-*` IDs **do not yet exist in‑tree** (`git grep C14-LIVE` over `fb891b0` returns nothing) — the composition introduces them as new open handles onto residuals the ACCEPT already routed.

**`C14-LIVE-A1` — inherited constructor‑held‑traceback cleanup debt.** OPEN.
From the ACCEPT’s Residual routing: *“constructor unwinds do not attach the cleanup error as `__cause__` — inherited and unchanged in both `_open` and `validate_sealed_commit`; routed — the cause‑link half of the C14 contract is not yet realised there.”* Plus `F6` persistent teardown: no recovery, reproduced at `11,456` retained bytes, documented as not claimed. The composed `command-source-trace.md` states the mechanism honestly: `__del__` is best‑effort, *“raisedされたerrorのtracebackがsourceをpinしている間は何も返さず, 持続的に失敗するsession teardownはどちらのfinalizerでも回収されない.”*
**Also under A1:** the one surviving `raise cleanup_error from run_error` in `run_server_verification` (`merged:9653`) — pre‑existing on **both** lineages, listed by the ACCEPT as *“`AccountedServerVerification` — untouched; not in the diff — separately routed composition blocker.”* Not composition‑introduced (§5.4). Gate: R7 must find it exactly once, there and nowhere else.

**`C14-LIVE-A2` — in‑tree C14/B2 authority citation.** OPEN as a *contract* item; the **citation** is discharged by composition.
ACCEPT Observation O1: the canonical source docs cite an authority the reviewer found nowhere in‑tree. **That finding was true at base `5790463` and is false at `fb891b0`** — `…shared-primitives-preflight-claude-20260731/preflight.md` (blob `ec2a264142c6`, sha256 `ca1a1451…`) defines `C14` (§2 register) and `B2` (§0). Composition adds the link (§6.1 W6, gate D7). What stays OPEN: the preflight is a *review artifact*, not a canonical contract document; promoting `C14`/`B2` to a durable in‑tree contract, and the `shape-pending` state of `RetryableCleanup` (§5.5 of that preflight), remain unowned.

**`C14-LIVE-SUCCESSOR` — global C14 owners.** OPEN, predecessor‑gated, **explicitly not touched by composition.** Each is byte‑identical to base or to its accepted source in the composed tree:

| Surface | State |
|---|---|
| `AccountedServerVerification` / `run_server_verification` | untouched; carries the surviving inversion (→ A1) |
| handler `POST` / request‑loop `finally` | untouched; only **read** by the new observer test |
| signature / server / ancestry owners outside the two wrappers | untouched |
| planner / stage / sealed factories | untouched |
| `AccountedHTTPUploadAncestry`, `AccountedHTTPUploadSealedAncestry` (production‑unwired successors) | untouched |
| `validate_sealed_commit` (`0f43415:codefire:30389`) | already C14‑correct on precedence (`raise run_error` then `cleanup_error`); **no cause link, no synchronous retry**; owns its own release. Outside the two wrappers by design — the composed `source-architecture.md` says so: *“That synchronous guarantee is scoped to those two commit‑source routes and to nothing else.”* |
| `F7` `_open` handle orphaning | byte‑identical to base; routed owner‑hardening |
| `F9` `_control_charge_witness` | byte‑identical to base; deferred pending a recorded manager decision |
| `O2` live‑route oracle insensitivity to `m2` | oracle strength only; low actionability |
| `O3` `except BaseException as retry_error: detach(retry_error)` at `0f43415:codefire:7415-7416` — discards `KeyboardInterrupt`/`SystemExit` during retry | non‑blocking per the ACCEPT; carried into the composed tree unchanged; **name it, do not fix it here** |

**Also carried forward, unchanged:** CFB‑138 default‑profile upload scale; MR‑write; metrics/conformance; Install; Dogfood; terminal lifecycle. All predecessor‑gated per burn‑down items 3–6.

**Anti‑false‑closure rule for the writer.** No composition text may say `Close`, `Closed`, `complete`, `zero-residual`, or `ACCEPT` about CFR‑168, CFB‑136, CFB‑138, global C14, or the cycle. Only two closure claims are licensed by evidence, and both are narrow: (a) `HIGH-2` is CLOSED on all three limbs **for candidate `0f43415`**, per `46e7c3e4…`; (b) the `B2` product contradiction between `b164046` and the shared‑primitives preflight is **resolved on the composed tree**, per the §5.4 census. Root row 6 stays `Implement in progress`.

---

## 11. Blockers and unknowns

**BL‑1 — ordering gate. Manager decision required before the writer starts.** ⚠️
The burn‑down freezes: *“repair and rereview Python Upload/CFB‑138, **then resolve the shared‑primitives preflight before MR‑write and Python composition**.”* Upload repair + rereview is **done** (`c54e5ce` ACCEPT). The shared‑primitives preflight is **not formally resolved** — no result/adjudication artifact exists in `fb891b0` closing its `B2`, and its per‑clause register still records `C14` behavior‑open against `b164046`.
Substantively, this composition **is** the resolution of B2’s product half: `b164046`’s cited inversion at `7049` disappears (§5.4). But that is a *measurement made by this preflight*, not a manager adjudication. Reading the gate literally, Python composition is not yet authorized; reading it substantively, composition is the resolving act and the preflight’s `B2` should be adjudicated **as part of** the composition step.
**Recommendation:** manager records a one‑line adjudication — *B2’s product half is resolved by the Python composition; `RetryableCleanup` `shape-pending` and the C14 contract‑document half route to `C14-LIVE-A2`* — before the writer starts. Do not let the writer self‑authorize this.

**BL‑2 — burn‑down status wording, manager‑owned.** Whether the composition may add a status line to `current-cycle-burn-down-plan.md` (§7.1 optional path 19), and its exact wording. Root row 6 must stay `Implement in progress`. Recommend the manager write it, not the writer.

**BL‑3 — prospective base is not yet accepted.** `fb891b0` is under fresh exact review. Everything here is conditional on that ACCEPT. If the Cargo‑port review returns REQUEST_CHANGES and `fb891b0` is amended, **every** base‑dependent value must be recomputed — though note the product‑side conclusions are unusually robust: they depend only on `codefire`/4‑docs/`test_codefire_cli.py` remaining at their `16d0129` blobs, which any docs‑only amendment preserves. The later report‑only Cargo review commit, if ACCEPT, writes only under `docs/development/ai/swarm-results/` and must touch no product path.

**UNK‑1 — `test_codefire_cli.py` × merged `codefire` (highest residual risk).** §5.5. `b164046`’s copy has never been executed against a build containing the Upload successor delta. Statically clean (0 overlapping units, 0 drift across 412 units), but 228 CLI tests exercise integrated behaviour that unit‑granularity analysis cannot reach. **V7 is mandatory.** Not a blocker to starting — a blocker to any ACCEPT.

**UNK‑2 — reversal reproduction cost.** R1 needs the 9 single‑edit reversals `r1`–`r6b`/`m2` from the Upload ACCEPT plus the MR‑list set from `5c952a9a…`. Both reports name them but this lane did not extract the edit scripts. Budget reviewer time for reconstruction from the report text.

**UNK‑3 — no rereview batch manifest for the Upload ACCEPT.** `c54e5ce` added only `review.md`; no `…live-route-cleanup-rereview-claude-20260802.{json,md}` exists in any tree. The ACCEPT binds to the **repair** packet (`250703ae…`, sha256 `756bf200…`), which §7.1 adds. Minor evidence‑completeness gap, not a blocker — but if `check_swarm_admission`/`check_review_terminalization` are later run against this result directory, the missing manifest may surface.

**UNK‑4 — `check_ai_development` default‑timeout failure is pre‑existing.** §8.4 D5. Root‑caused to a 60.0s default vs a 60–63s `test_ai_tools.py` run; `--timeout 300` passes all 7 checks. Not composition‑caused. Flagged so no lane misreads it as a composition regression or tries to “fix” it inside this write set. Worth a separate maintenance item (raise the default, or split `tool_tests`).

**Nothing was invented.** Every SHA, tree, blob, patch‑id, path set, count and verdict above was recomputed in this worktree. Where a manager premise turned out incomplete (item 5’s docs scope, `cb13da0` alone as the duplicate) or where a review finding is stale against this base (O1), it is stated as such rather than papered over.

---

## 12. Confidence

| Claim | Confidence | Basis |
|---|---|---|
| Identities, trees, parents, merge counts, subjects, report paths/verdicts | **Very high** | Direct `git` recomputation; `46e7c3e4…` and `4ba34a6d…` match the ACCEPT report’s own independently‑computed table |
| Ancestry graph; `b164046` subsumes `701469f`; common product base is one blob | **Very high** | `merge-base` + blob equality at `5254e3d`/`16d0129`/`41dea14`/`fb891b0` |
| Duplicate equivalence `9889888c…` and which copy to omit | **Very high** | Patch‑id match **plus** byte‑identical normalized patch text **plus** identical realized blob `141dcbd4` on both sides |
| Path‑by‑path classification and the 18‑path write set | **High** | Exhaustive blob comparison of all 26 candidate docs paths + 6 product paths against `fb891b0` |
| `codefire` merges cleanly and each accepted change lands exactly once, unaltered | **High** | 0 conflicts; exact line arithmetic; `py_compile` PASS; 43/18 units with **0** overlap, **0** non‑identical, **0** drift across 412 untouched, **0** name collisions; module‑level `merged == union` |
| Composition resolves the cited `B2`/`C14` inversion | **High** | Inversion census 2→1 with the `enforce_remote_commit_signature_policy_accounted` site eliminated and the `run_server_verification` site correctly retained as a routed residual |
| Four source docs union cleanly with no semantic duplication | **High** | 0 conflicts; 0 heading collisions; 0 duplicate headings; hunk anchors disjoint; OURS is a pure insertion in both key docs |
| Expected test counts `538` / `766` | **High** | Per‑module recount cross‑validated against the ACCEPT’s measured `457/457` and `36/36` |
| Composed tree passes V6/V7/V8 | **Medium** | Static evidence is strong and uniformly favourable, but no test was executed in this lane. UNK‑1 is the specific exposure. |
| Composition is *authorized* to proceed now | **Low — do not assume** | BL‑1 (shared‑primitives ordering gate) and BL‑3 (`fb891b0` still under review) are both unresolved. This preflight authorizes nothing. |

**Bottom line.** The composition is unusually clean for its size: one 3‑way merge (`codefire`), four clean doc unions, four copies, one duplicate omission proved three independent ways, and a measured resolution of the exact defect that made two accepted trees contradict each other. The plan is executable as written. It should not start until the manager resolves **BL‑1**, and it cannot be accepted until **V7** and the **R1–R4** reversals run green on the frozen candidate.
