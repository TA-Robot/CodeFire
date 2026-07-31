# CFR-168 / CFB-138 Python HTTP Upload Review-Closure Candidate-Ready Card

Captured: 2026-07-31T03:40:06Z

Status: `READY_FOR_REVIEW`

This is manager-recomputed identity, provenance, and worker-validation
evidence. It is not an acceptance decision and does not authorize
integration.

## Exact Git identity

| Field | Manager-recomputed value |
|---|---|
| Candidate worktree | `/workspace/worktrees/cfr168-python-http-upload-zero-residual-repair-base/.claude-second-agent/5cecc4984d163f4eedc2809fe8bbe2c69e79e2cadcbd5d123667ee8b01641505/worktrees/cfr168-python-http-upload-zero-residual-repair-20260730-upload-zero-residual-repair` |
| Branch | `claude-agent/cfr168-python-http-upload-zero-residual-repair-20260730-upload-zero-residual-repair` |
| Continuation base / merge base | `b19c4346c0437ed28bdfe2fb6af1dd466abe9f8b` / `b19c4346c0437ed28bdfe2fb6af1dd466abe9f8b` |
| Parent | `f0ecd25ed18005d3f50b54dd254b7d05e678afb3` |
| Head / checkout HEAD | `467b0c27715e424ae18ffb5664da053b55915e2d` / `467b0c27715e424ae18ffb5664da053b55915e2d` |
| Tree | `3b129b9e41ae45defe4d6e7c43da04b016f07404` |
| Additive commits / merge commits | `3` / `0` |
| Binary full-index patch SHA-256 | `a71ea9276361cd426c0eecbf06c900034b5129849df281c8b8f881ffa0f63dc1` |
| Tracked/untracked state | clean; porcelain output empty after provider terminal |
| Manager decision | `READY_FOR_REVIEW` |

The manager ran the canonical
`docs/development/tools/capture_candidate_identity.py` implementation from the
canonical manager checkout against the terminal candidate. The helper's exact
SHA-256 is
`c4606a4b48af8f844a9b42b32e681cbc028df8b884fe75a66dcb6fafc16557b9`.
The helper is not part of this product candidate. The reviewer must recompute
the same values from a fresh detached scratch checkout and may also reproduce
the algorithm with ordinary read-only Git commands.

## Exact changed paths

| Status | Path | Added | Deleted | Base blob | Head blob | Head bytes | Head SHA-256 |
|---|---|---:|---:|---|---|---:|---|
| `M` | `codefire` | 156 | 29 | `cc2c52aff4a11ebd80b2f22e37482b1700c0d738` | `e72b263b2717c45e84087ce0094f04cc2c74c2ca` | 1,402,507 | `8df9193e3700ce4d0e47f2c4ff8bef231c92d7b6a8d723f204ade9c7f1d0be93` |
| `A` | `docs/development/ai/swarm-results/cfr168-python-http-upload-zero-residual-review-closure-repair-20260731/result.md` | 277 | 0 | — | `7100966bf76b1ada44d797e8787eb8377a18f6fd` | 16,259 | `a36149570fafcb7e38cc838738796b8e2b8c49095bab2325e1b136a3e31cb361` |
| `M` | `docs/development/change-playbook.md` | 16 | 0 | `fcb1514713644ae346e367358220852c64280725` | `727fb6d81a28d7078e50ff9602be2821c3cd87dd` | 36,636 | `792fc05bebdc4621ea5092c3724a349ed5313956e8296a647b34cd5bebec6a65` |
| `M` | `docs/development/command-source-trace.md` | 6 | 2 | `847d19441cac15bc89217a002d9cab67891555ec` | `6814208991d837d6992535576e44fe4e249978fb` | 40,678 | `986984327245a060d00fafcf9a12a85eb070a7bc8cc80685de12d0c1cbcec4e4` |
| `M` | `docs/development/source-architecture.md` | 30 | 5 | `6a4f8d4afb5312d18029ef0754fcedc16b6bab77` | `fc707bd2aee8ce88b864a658383fbf27a53405af` | 103,662 | `a1e2f19fc13cf8985f1d03390ad8d6f7d8e7573a0677b5cb2d121e3d071e0e6e` |
| `M` | `docs/development/source-data-model-catalog.md` | 2 | 2 | `ea357917bfa18e13e3a58fb3c4ce0092cb8a0241` | `619a70e7854ea14fd0942db3b4b2043eb8db113d` | 93,150 | `5ccab09f3c1f6a6ae84f3a7c6dc6b837fa5b2f699968dae6ba2457040606b9c0` |
| `M` | `tests/test_python_http_upload_accounted_traversal.py` | 806 | 2 | `ceeeebe8be2b581b2ba216dc349ced12624ad4c3` | `010727908196179cb08c059e03e3cb54ddf7834c` | 143,181 | `c58135d4fb9341c39fe4ceaf335dab41db97b45e49ce512c30e6b3cdca7cda90` |

The product/test/source-document paths are inside the continuation packet's
declared implementation scope. The result record is the immutable provider
handoff added by the final documentation commit.

## Additive checkpoint history

1. `f2ebb32c26629cadca6a8c0967a212c1a91ff688`, tree
   `8c37630318ab3320c55dcf952fa4ea91edf2f7f1` — close the three
   product findings from the rejected rereview.
2. `f0ecd25ed18005d3f50b54dd254b7d05e678afb3`, tree
   `df4cae6ad3a69085a16cf1bdff352b1ce686763d` — bind the closure
   with non-vacuous regression and mutation oracles.
3. `467b0c27715e424ae18ffb5664da053b55915e2d`, tree
   `3b129b9e41ae45defe4d6e7c43da04b016f07404` — record the exact
   implementer result and terminal evidence.

The range is linear and additive. No commit was amended, rebased, squashed,
or merged.

## Review and continuation binding

| Evidence | Exact value |
|---|---|
| Rejected predecessor | `76baff84cb1192e11b26dcc669fb800f7b13fa30`; tree `fcc65a4729f5029556e0266971d94b6321421f09` |
| Prior independent reviewer | Fugu Ultra 1.1 (`fugu-ultra-v1.1`) |
| Prior independent-review commit | `35983261e1abd32045e1ff6cbd7dd7dfff5e806d` |
| Prior report copied into candidate lineage | `fe7cf94388393bd9b0457564d0116406f2c8844e` |
| Prior report path | `docs/development/ai/swarm-results/cfr168-python-http-upload-zero-residual-rereview-fugu-20260730/review.md` |
| Prior report blob / SHA-256 | `1fd3de9b48e04d29b56f36d79584a0a03c22bcff` / `7632552eb0ca60addca0d83fe3ba53ead642a7cbee82937b1b7b37754f6cb2cf` |
| Prior verdict | `REQUEST_CHANGES` |
| Continuation packet commit | `b19c4346c0437ed28bdfe2fb6af1dd466abe9f8b` |
| Continuation packet path | `docs/development/ai/swarm-batches/cfr168-python-http-upload-zero-residual-review-closure-repair-20260731.md` |
| Continuation packet blob / SHA-256 | `b7031007c17fcb78ea94491458a622b9841930f2` / `9c46f5585e1e83748651ced1e3384db4a4eb2201c966032de9211f79bc8ef64e` |
| Implementer / model / role | Claude / `claude-opus-5` / original implementation repair |
| Original and resumed exact session | `7baa1275-0ee2-476a-af46-1907386ebceb` |
| Implementer result path | `docs/development/ai/swarm-results/cfr168-python-http-upload-zero-residual-review-closure-repair-20260731/result.md` |
| Implementer result commit | `467b0c27715e424ae18ffb5664da053b55915e2d` |
| Implementer result blob / SHA-256 | `7100966bf76b1ada44d797e8787eb8377a18f6fd` / `a36149570fafcb7e38cc838738796b8e2b8c49095bab2325e1b136a3e31cb361` |
| Exact resumed-session transcript SHA-256 | `d93d7899fa996ff176485f43324e8f42172b87d67ef1d1e13aabdfb299f7df8e` |
| Transcript terminal size / mtime | 5,633,375 bytes / `2026-07-31T03:32:54.490478278Z` |
| Provider terminal | completed; resume manager PID `1318933` and provider/gate processes terminal |

The original scheduler state still says `running` because the repair was
resumed directly in the exact Claude session after that scheduler became
stale. The terminal Claude transcript, final handoff, clean Git state, exact
commits, and committed result are the binding evidence. The stale predecessor
scheduler state must not be used as candidate identity.

The fresh review must use a new Fugu agent, session, and worktree. It must not
reuse the prior Fugu reviewer or inherit the Claude implementer conversation,
temporary mutation trees, conclusions, or mutable logs.

## Required finding closure claimed by the implementer

| Prior finding | Worker evidence requiring independent re-falsification |
|---|---|
| BLOCKER-1: completed-quarantine recovery escapes the request ledger/deadline | The exact upload ledger and deadline are threaded through `execute_remote_json_publication` and `recover_remote_pending_quarantine`; recovered roots use that ledger/deadline; polls precede recovered-root validation, receipt replay, and quarantine cleanup. The 1.0s/100ms reversal now returns typed deadline with byte-identical authority and preserved quarantine. |
| HIGH-2: caller-owned sealed-validation APIs fail at ordered projection | The owner now acquires exact `temporary_bytes` before ordered projection allocation. Empty/one/27-record values are 56/64/272 bytes on this runtime. All three caller-owned APIs reportedly complete with ledger and ledger+deadline and end at zero debt/permits. |
| HIGH-3: publication-root proof is reusable | The proof now has `available`, `consuming`, `consumed`, and `revoked` states. State is claimed before the callback-capable deadline poll; alias, callback reentry, second use, stale/foreign inputs, and revocation reportedly preserve single consumption and typed precedence. |
| MEDIUM-4: mandatory reversal evidence is vacuous | Nine one-at-a-time mutations reportedly make focused tests non-green for named behavioral reasons, including ordered-projection claim removal, every prepared-coverage predicate, layout-poll removal, recovery handoff removal, and proof reuse. |

## Worker validation evidence

All recorded commands used `PYTHONDONTWRITEBYTECODE=1` in the implementation
worktree where applicable.

| Gate | Terminal worker evidence |
|---|---|
| Python compile | `codefire`, focused upload module, sealed-production module, and `tests/test_codefire_cli.py`: PASS |
| Focused upload | 29/29 PASS in 156.953 seconds |
| Neighboring owner suites | 59/59 PASS in 29.597 seconds |
| Broad Python-pattern suite | 450/450 PASS in 333.387 seconds |
| Exact selected legacy upload/nonce/receipt/Doctor/branch cases | 13/13 PASS in 169.748 seconds |
| Source docs | PASS: 44 modules, 34 commands, 12 object kinds |
| Project docs | PASS: 531 Markdown files |
| Patch hygiene | `git diff --check`: PASS; terminal worktree clean |

The implementer also recorded all nine deliberate mutations and their
individual focused failures, but those mutation trees and mutable `/tmp`
outputs are not trusted review evidence. A fresh reviewer must recreate them
independently.

## Evidence limits and composition overlap

- Cargo, Docker, Install, Dogfood, `./demo.sh`, unrestricted all-language
  discovery, repository-wide validation, and Python composition were not run.
- Independent raw-wire 16 MiB body and 64 KiB header exact/±1 fixtures were
  not re-authored by the implementer.
- The candidate changes shared publication and recovery signatures also
  reached by MR merge/review/apply. The active MR-list lane changes the same
  monolithic `codefire`, so final integration requires a separate
  composition review and conflict resolution.
- The worker's inventory reports 29 production `validate_sealed_commit`
  occurrences. Upload recovery is now accounted, while `handle_http_list`,
  `http_remote_branch`, and `validate_http_merge_request_record` remain
  separately owned HTTP surfaces.

## Manager disposition

`READY_FOR_REVIEW`

Only a fresh report-only Fugu Ultra 1.1 reviewer may return `ACCEPT` or
`REQUEST_CHANGES` for exact candidate
`467b0c27715e424ae18ffb5664da053b55915e2d`. Neither decision integrates
product bytes, resolves Python composition, closes CFR-168/CFB-138, or
authorizes Install, Dogfood, Close, or release.
