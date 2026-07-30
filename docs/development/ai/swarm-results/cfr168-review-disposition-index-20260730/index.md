# CFR-168 Independent Review Disposition Index

Date: 2026-07-30

This index binds every committed independent `review.md` currently present
under `docs/development/ai/swarm-results/` to one explicit disposition. It
prevents three different states from being conflated:

- an exact candidate accepted by review;
- a rejected candidate with an active or queued repair;
- a rejected alternative line that must not be integrated and has been
  superseded by a different implementation path.

A report is immutable evidence. `ACCEPT` applies only to its exact reviewed
commit and does not automatically integrate that commit. `REQUEST_CHANGES`
forbids product integration until an additive successor receives a fresh
independent decision. Under the current zero-residual policy, accepted
candidate Should/Could findings may still require a successor before final
CFR-168 closure.

## Disposition table

| Review | Integrated report commit / SHA-256 | Decision | Exact disposition and successor |
|---|---|---|---|
| [`claude-cfr168-review-20260729`](../claude-cfr168-review-20260729/review.md) | `14a24a5` / `d41ae00cfecd71fed6567c553a614ce61ac6087ced9e95b905d6a8572826362d` | `REQUEST_CHANGES` | Reject and do not port the stacked `aa5e57d` alternative line wholesale. Canonical product snapshot `e2a218e` is not its descendant and omits the reviewed execution-evidence owner. Current focused lanes replace that design; task contracts explicitly forbid importing it. |
| [`cfr168-python-http-doctor-branch-independent-review-20260730`](../cfr168-python-http-doctor-branch-independent-review-20260730/review.md) | `4e651a7` / `5d76f8a5d1efd3e75fea88464ad01108666e00e714dd11b488bd3c3c99cd93a2` | `REQUEST_CHANGES` | Corrected in isolated Doctor+Branch composition `701469f`; exact successor received the accepted rereview below. Product integration remains predecessor-gated. |
| [`cfr168-python-http-doctor-branch-composition-rereview-20260730`](../cfr168-python-http-doctor-branch-composition-rereview-20260730/review.md) | `c57215e` / `c7a1665fdf4a6e1104bbab18ec8051c048ace1d25e865952bbbcd35a06eb19c0` | `ACCEPT` | Exact corrected composition accepted. It is validation input for the later Python spine, not permission to merge an entire worktree or bypass upload/MR predecessor reviews. |
| [`cfr168-python-http-mr-list-review-claude-20260730`](../cfr168-python-http-mr-list-review-claude-20260730/review.md) | `2d26695` / `95617c3cb48320e7256d23ae658f4cbedb18f9c4f8e9a7803a0417972f28042c` | `REQUEST_CHANGES` | Produced repaired candidate `38da898`; that exact candidate was independently rejected again below. |
| [`cfr168-python-http-mr-list-zero-residual-rereview-fugu-20260730`](../cfr168-python-http-mr-list-zero-residual-rereview-fugu-20260730/review.md) | `78b127c` / `6840035553871f5251e32d67bb5b2f9d6c6ee061b7ab3106e94de373ec97b20d` | `REQUEST_CHANGES` | Parent-authority absence, error-origin classification, retryable cleanup, late metadata, final publication gap, and two oracle defects are frozen in `cfr168-python-http-mr-list-review-closure-repair-20260730.md`. Claude Opus 5 repair is active; final acceptance requires manager composition with the accepted MR-write/Upload successors. |
| [`cfr168-python-http-mr-mutation-review-claude-fallback-20260730`](../cfr168-python-http-mr-mutation-review-claude-fallback-20260730/review.md) | `0da5855` / `925cbcdbd5e414b962a92ea9af3a20622f0881da664617b74cb964236b0a4681` | `REQUEST_CHANGES` | Initial zero-residual successor produced exact candidate `6dd4527`; that candidate was independently rejected again below. |
| [`cfr168-python-http-mr-mutation-zero-residual-rereview-fugu-20260730`](../cfr168-python-http-mr-mutation-zero-residual-rereview-fugu-20260730/review.md) | `84175bd` / `214e42fc8f36108f45e17b13e4343bfc2b56cc89b72be5c1258f5b497c9ec6ef` | `REQUEST_CHANGES` | F1–F7 are frozen in `cfr168-python-http-mr-write-review-closure-repair-20260730.md` and its validated manifest. The next Claude implementation waits for a free light lane and Python write-set ordering. |
| [`cfr168-python-http-upload-scale-review-claude-fallback-20260730`](../cfr168-python-http-upload-scale-review-claude-fallback-20260730/review.md) | `c734b55` / `534f098348e72fdc377b56ebf561a931e53284055a1f8a0e37d6d5393d956486` | `REQUEST_CHANGES` | Repair produced clean exact candidate `76baff8`. Its report-only Fugu Ultra rereview packet is registered by canonical commits `4537cf9` and `8a8267d`; the exact-candidate review is active. No product integration is permitted before its decision. |
| [`cfr168-rust-file-ancestry-review-claude-fallback-20260730`](../cfr168-rust-file-ancestry-review-claude-fallback-20260730/review.md) | `30bbb46` / `64a4831ea0a639d7f87fe0b08312677efcab8bfe014f0d5ba72806e9caceecf1` | `REQUEST_CHANGES` | Replacement repair produced clean exact candidate `02b0d63`. Its report-only Fugu Ultra rereview packet is registered by canonical commits `ecefdf2` and `ab119b0` and is queued behind the single-Fugu upload/storage order. |
| [`cfr168-rust-file-doctor-review-claude-fallback-20260730`](../cfr168-rust-file-doctor-review-claude-fallback-20260730/review.md) | `e9480c7` / `c1b39ffc6f25cf9f19007af3cd5336421400e467e6e3c28969de775b86ed8045` | `ACCEPT` | Exact product target `52a025e` is accepted, but four Should and four Could items remain mandatory under zero-residual closure. The collision-free Claude Opus 5 replacement repair is active under `cfr168-rust-file-doctor-zero-residual-repair-r1-20260730`. |
| [`cfr168-rust-http-mr-preflight-review-claude-20260730`](../cfr168-rust-http-mr-preflight-review-claude-20260730/review.md) | `e28c90d` / `f30c22aa34559829254b6ca305449abc874b653c8b3dbc767248fdbbc5fe0a5a` | `REQUEST_CHANGES` | Exact target `71854e2` is product-sound but lacks a non-vacuous central ordering barrier and has lifecycle/boundary residuals. F1–F5 and R1–R4 are now active in the Claude Opus 5 closure repair from exact task base `7e4e685`. |
| [`cfr168-rust-storage-independent-review-fugu-20260730`](../cfr168-rust-storage-independent-review-fugu-20260730/review.md) | `4380985` / `628a37f51cad4710eeccb8c9b34286b9e41332a59a421068a6a1aaa61ed4140a` | `REQUEST_CHANGES` | F1–F9 repair produced clean exact candidate `cdcb9bd`. Its report-only Fugu Ultra rereview packet is registered by canonical commits `0981524` and `f7bf266` and is queued immediately after the active upload review. |
| [`development-flow-kgi-kpi-independent-review-claude-20260730`](../development-flow-kgi-kpi-independent-review-claude-20260730/review.md) | `b307b47` / `02a0d259c55121379f07c2c1d893d4a29dd546b16e6f237659a5081c60fd3336` | `REQUEST_CHANGES` | First repair candidate `0ae583e` passes its official gates but fails eight manager counterexamples. Exact Fugu rereview is ready; the shared-primitives successor packet is committed with status `blocked` until that report is bound. |

## Current review queue

1. Finish the active Upload exact-candidate Fugu review and integrate only its
   immutable report.
2. Reuse the single Fugu permit in order for the prepared Storage and
   File-ancestry exact-candidate reviews; keep the KGI/KPI review behind those
   CFR-168 product candidates.
3. Collect the three active Claude repairs: File Doctor, Rust HTTP MR
   preflight closure, and Python MR-list closure. Each completed candidate
   requires a fresh exact report-only review.
4. Keep MR-write repair behind upload/MR-list and shared Python composition
   ownership ordering.
5. Integrate product commits only after exact acceptance, then run composed
   broad gates before Install, Dogfood, Close, and Retrospect.

## Root-finding coverage map

The earlier current-state closure audit remains the root review. Its findings
are routed as follows; a routed finding is not yet a closed finding.

| Root finding | Current owner |
|---|---|
| M1 — Python upload full-history decision is unbounded/undeadlined | Active Python upload repair plus active Rust/Python reusable file-ancestry repair; final composed upload review must prove the live path. |
| M2 — HTTP Doctor used a second request ledger | Corrected Doctor+Branch composition `701469f`, independently accepted at `c57215e`; composed-spine review still required. |
| M3 — uncovered Python HTTP routes and mutating pre-effect response | Upload candidate `76baff8` awaits exact rereview; MR-list repair is active; MR-write closure is queued behind shared Python composition; `POST …/gc` remains explicitly predecessor-gated on their response/session result. |
| M4 — unbudgeted Rust local/file surfaces | Storage and file ancestry active; File Doctor restart queued; local `cf merge`, `cf scan`, clone/storage/GC consumers remain predecessor-gated with exact symbols in the recovery ledger. |
| M5 — upload signature history outside accounting | Active upload repair owns signature policy/history under the same request ledger and final fence. |
| M6 — tests concentrated on non-production callers | Every current packet requires production-call reachability, exact-candidate reversal, and full snapshot/zero-debt evidence; final composition must rerun broad gates. |
| S1 — cache metrics omit/contradict invalidation | Blocked until all validation-session consumers are stable, then owned by the final cache/command projection packet. |
| S2 — metrics-orthogonality test cannot falsify its claim | Same final metrics/projection packet, with an independent deliberate-regression oracle. |
| S3 — no canonical asymptotic order table | Final stable-counter documentation packet after all selected production call graphs are fixed. |
| S4 — Rust MR create/review response preflight | `cfr168-rust-http-mr-preflight-review-closure-repair-20260730` is validated and queued. |
| S5 — one-command metrics projection and stale capability value | Final command-wide projection/conformance packet, followed by capability re-advertisement only after all gates pass. |

The seven missing Close-evidence items remain manager-serialized after product
composition: clean-HEAD Install with hashes, installed boundary/scale
reversals, canonical Algorithm Dogfood, full cross-runtime corpus, all M4
surface gates, Retrospect, and final capability re-advertisement.

## Closure invariant

Final CFR-168 review closure requires every row to end in one of two terminal
states:

- exact accepted product commit integrated and revalidated on the composed
  head; or
- exact rejected/superseded candidate permanently excluded, with no live
  product byte or claimed evidence depending on it.

An active, queued, blocked, partial, interrupted, or report-only state is not
terminal.
