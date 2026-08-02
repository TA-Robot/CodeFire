# CFR-168 / AIA-046 — Artifact Export, stdout, Policy Binding, Source Re-read: Final Exact Re-review

Recorded: 2026-08-02
Role: fresh independent exact reviewer (control-plane safety), report-only
Review type: exact candidate review over the complete five-commit repair chain
Decision authority: reviewer proposal only
Integration: **this review does not integrate.** It performs no merge, no
rebase, no cherry-pick, and no mutation of any implementation, documentation,
tool, or evidence path. It adds exactly one file.

Every fact below was recomputed from Git, from the source, or from a live
probe. No prompt statement and no previous report was accepted as given.

## 1. Candidate identity, topology, and path scope

Recomputed with `git log --format='%H %T %P'` and `git status --porcelain`.

| Position | Commit | Tree |
|---|---|---|
| L (parent) | `6db7944b26399c3bc43867f12ee9b3f260988621` | `69b9f7414ef0104a5782493f8731e1cd3636795f` |
| fix A — Cargo uplift hardlink admission | `14fc108c67ad9e8305655a46b6490d08eb7db73f` | `8d4742de33995df9619f56f3f25b30cfc0e0d56b` |
| fix B — durable gate result bytes on stdout | `a45358a92895d6fca21899b1693cdf653fce9aef` | `691b5403fa0c3cb14b0ac54fe61ccb390977bcac` |
| fix C — policy identity across the gate user namespace | `3d23af7da893eb1f4e8657fc972379b10ea7818f` | `96908f2b56d9903e6b9032ba431cc39aa9f9a341` |
| fix D — full-content source re-read | `21c863e5cf8f5776b5966b3cb76c676c7135545d` | `5fb61e5d35714f9c0cab5a7480f3b00dd941198b` |
| **candidate (after HEAD)** | **`54028a136943a9d5f9608e19d3855c0edae46adf`** | **`e4057e1b6f16b12fed45ad0b50bc4c6014ca7ebd`** |

Linear, single-parent, no merge. Each commit's recomputed parent is exactly its
predecessor above. The candidate matches the mandated commit and tree byte for
byte. `git status --porcelain` is empty before and after this review; `git
ls-files -v` shows no non-`H` index flag.

Changed paths over `6db7944..54028a1` — exactly four, recomputed with
`git diff --numstat`:

| Path | + | − | Kind | Blob at candidate |
|---|---|---|---|---|
| `docs/development/ai/source-keyed-cargo-gates.md` | 75 | 3 | contract | `f57b9a7840cfa74ff53581cc2786af4c2a518ac3` |
| `docs/development/tools/build_source_bound_rust.py` | 197 | 10 | consumer | `a134c23d810509d78057f71ce289d459b2956b3e` |
| `docs/development/tools/run_source_keyed_cargo_gate.py` | 172 | 27 | gate | `a156b03074bec91708bd1becc2f116563346f36f` |
| `docs/development/tools/test_ai_tools.py` | 1451 | 0 | tests | `a31b5367f48290426a5f227ca0b068195d14c3a4` |

Range patch SHA-256 (`git diff 6db7944 54028a1`):
`86fec952532fc37fa1631885f50e2071b2f8867a136a2d53a03a09544409a1c1`.

Per-step patch digests:

| Range | Paths | +/− | Patch SHA-256 |
|---|---|---|---|
| `6db7944..14fc108` | 3 | 387 / 4 | `cbd9cf43a2b9b9a2d9bbac638de9c5ca78e47704fc0f26cac6bea9d188e3cb08` |
| `14fc108..a45358a` | 3 | 301 / 11 | `7d3f87c9c8c271dc046e90ab297d17e5b5cc48196c168c9aa9f5379a5e09a72c` |
| `a45358a..3d23af7` | 3 | 368 / 6 | `c60a0f0a445873a81bd0244cb038913961adfc5c08c28e70567ef12dcb2aef16` |
| `3d23af7..21c863e` | 4 | 496 / 48 | `4080c4f227a9567bc6361462e93c5eb86a39d1bd330383f113234f101be22c89` |
| `21c863e..54028a1` | 4 | 384 / 12 | `ac4761e75ade3280a571e8f85a9a9403065acdc16b6026d45b844cf4ae0ba735` |

No path outside these four is touched at any step. `crates/**`, `tests/**`,
`schemas/**`, `install.sh`, `pyproject.toml`, `setup.py` and the four canonical
source docs are untouched, so the AGENTS.md canonical-source-doc and Rust
workspace obligations are not engaged by the diff — though
`check_source_docs.py` was still run and passes. Each of the four changed paths
has a reviewed surface below; there is no `out_of_scope` escape for any of them.

### Path topology

This review worktree is a linked worktree of `/workspace/project/.git`
(`gitdir: /workspace/project/.git/worktrees/cfr168_aia046_artifact_export_final_rereview_20260802`),
nested beneath the source worktree's `.claude-second-agent/`, which
`.gitignore:18` ignores (`git check-ignore -v` confirms). It therefore cannot
enter the candidate's HEAD tree, cannot dirty the source worktree, and cannot
enter a `git archive` snapshot. The source worktree
`.../cfr168_aia046_artifact_export_hardlink_repair_20260802` is at exactly
`54028a1` / `e4057e1b…` and `git status --porcelain` there is empty. Each of the
four changed paths hashes identically in that worktree and at the candidate
commit — no live/committed drift.

## 2. Supersession of the stale `42433c8` ACCEPT

`42433c8d1a0b663dee3cf3e45b895d70e3b85c50` (report path
`swarm-results/cfr168-aia046-artifact-export-stdout-exact-review-20260802/review.md`)
is **not an ancestor of this candidate**: `git merge-base --is-ancestor` is
false; it sits on `claude-agent/cfr168_aia046_artifact_export_stdout_exact_review_20260802`
as a report-only child of `a45358a`. It is **explicitly superseded** and does
not carry forward to `54028a1`, on two independent grounds.

**Ground 1 — different subject.** It bound `6db7944..a45358a` (2 commits, 3
paths, tree `691b5403…`) and stated so. The candidate is `6db7944..54028a1`
(5 commits, 4 paths, tree `e4057e1b…`). Under the operating protocol, any
candidate byte change creates a new exact subject; its verdict cannot transfer.

**Ground 2 — refuted disposition (`new_counterexample`).** Its "Residuals —
Recorded, Not Blocking" §1 dispositioned the same-tick mid-copy window as
nonblocking on the reasoning that the exposure is identical on the parent and
that "a future doc-only change could qualify the sentence with the tick
condition; nothing in the code needs to change." I reproduced that window on
this host's real clock, with no mocking, against `3d23af7` — the last tree
before the content re-read and the direct successor of the tree that report
accepted:

```text
3d23af7 (real clock, 120 attempts): stat_oracle_blind=3  BLOCK=117
                                    EXPORTED=3  torn_bytes_published=3
54028a1 (real clock, 120 attempts): stat_oracle_blind=0  BLOCK=120
                                    EXPORTED=0  torn_bytes_published=0
```

On three of 120 attempts the pre-repair export published a 3 MiB
A-prefix/B-suffix file equal to neither version of the source, with the before
and after stat tuples byte-identical. That is a reachable, reproducible
data-integrity failure in the release artifact-export path — `MF-2` — not a
recordable residual, and the correction was in code (`21c863e`), not in
documentation. The stale ACCEPT is therefore superseded on the fixed
`new_counterexample` reopen trigger, and additionally on
`materially_new_evidence` for the exact `nonblocking` disposition it bound.

Per the protocol, the historical record is preserved as written: `42433c8`
remains an immutable reviewer proposal against its own subject, and this report
does not rewrite it. Its residual §2 (uncommitted follow-on edits in the sibling
worktree) is separately closed: those edits are now `3d23af7`, `21c863e` and
`54028a1`, and that worktree is clean.

## 3. The five fixes

### Fix A — Cargo uplift hardlink admission and destination strictness (`14fc108`)

`export_bound_artifact` replaces `st_nlink != 1` with
`st_nlink < 1 or st_nlink > MAX_EXPORT_SOURCE_ARTIFACT_LINKS` (`= 2`), and the
`artifact-not-private-executable` detail now carries
`regular` / `mode` / `nlink` / `nlink_maximum`. `EXPORTED` gains
`source_nlink_maximum` and `source_cargo_uplift_hardlinked`.

Nothing adjacent is relaxed, verified by reading the function rather than by
inference: the source is opened `O_RDONLY|O_CLOEXEC|O_NOFOLLOW` relative to a
pinned directory descriptor, so a symlinked source fails at the kernel with
`ELOOP` before any stat; `S_ISREG` and `S_IXUSR` still hold; the logical-byte
bound, free-floor, and export deadline are unchanged; `nlink >= 3` is `BLOCK`.
The destination is unchanged and independently strict — created
`O_WRONLY|O_CREAT|O_EXCL|O_CLOEXEC|O_NOFOLLOW` at `0o700`, `fchmod(0o700)`
re-applied, written by descriptor copy rather than by `link`, and the
post-rename predicate requires `S_ISREG`, `st_nlink == 1`, `(dev, ino)` equal to
the staged descriptor, and `st_size == size`. A pre-existing destination must be
regular, euid-owned and single-link, or the export is `unsafe-prior-artifact-export`.

Live corroboration: `source_nlink = 2`, `source_nlink_maximum = 2`,
`source_cargo_uplift_hardlinked = true`, `export_nlink = 1`,
`export_mode = 448` (`0o700`), and the exported inode differs from the source
inode.

### Fix B — exact durable result bytes on stdout (`a45358a`)

`publish_reserved_result` now returns the exact committed bytes; `main` tracks
them in `durable_raw` on the PASS, `GateBlocked`, and internal-error paths, and
`emit_durable_result` writes them through `sys.stdout.buffer`.

Single-representation check, recomputed by grep over the whole gate module:
the only writes to standard output in the file are lines 8208–8210 inside
`emit_durable_result`. Every parent-tree
`print(json.dumps(..., indent=2, sort_keys=True))` call site is gone; the two
surviving `print(...)` calls in `main` are `file=sys.stderr`. There is no
`return` inside `main`'s `try` body, so `durable_raw` is assigned on every path
that reaches an emission.

Terminal-path coverage, each executed:

| Path | stdout source | rc |
|---|---|---|
| PASS | bytes returned by `publish_reserved_result` | 0 |
| BLOCK after reservation | bytes returned by `publish_reserved_result` | 3 |
| internal error after reservation | bytes returned by `publish_reserved_result` | 2 |
| publication failure (`OSError`/`ValueError`) | `bytes(reservation["raw"])` — verified to be exactly what the reservation left on disk | 2 |
| BLOCK before any reservation | `unreserved_result_bytes` — one bounded rendering, no file exists | 3 |

Independent live differential, run outside the unit suite on isolated `git
archive` trees with the exact required interpreter flags:

```text
54028a1  --timeout 0 -> rc=3, stdout 118 bytes, 1 newline (trailing only)
         {"blockers":["invalid-gate-timeout"],"decision":"BLOCK",...}
6db7944  --timeout 0 -> rc=3, stdout 143 bytes, 8 newlines (indent=2)
stdout(parent) == stdout(candidate): False; no result file created on either
```

### Fix C — outer vs gate user-namespace UID policy binding (`3d23af7`)

`POLICY_IDENTITY_FIELDS` splits into `POLICY_INVARIANT_IDENTITY_FIELDS`
(`bytes`, `sha256`, `device`, `inode`, `mode`, `nlink`, `mtime_ns`, `ctime_ns`
— all still compared for strict equality) and
`POLICY_NAMESPACE_RELATIVE_IDENTITY_FIELDS` (`uid` — recorded from both sides as
`outer_uid` / `gate_namespace_uid`, never compared across the boundary). A new
`policy-supervisor` member binds the gate's own code, and the payload schema
moves to `codefire.rust_build_outer_policy_binding.v2`.

The defect being repaired is real and total, not cosmetic: the gate is launched
as `setpriv --pdeathsig SIGKILL /usr/bin/unshare -Urpf --mount-proc
--kill-child=SIGKILL --propagation private`, so inside it every launcher-owned
file reads back as uid 0 and every unmapped owner as the kernel overflow uid.
Strict `uid` equality across that boundary is a property no real invocation can
have; the parent's `validate_outer_policy_binding` would raise on every release
build.

Ownership is not weakened, verified on both sides in source:

| Member | Outer reader | Gate reader | euid check |
|---|---|---|---|
| `policy-build-source-bound` | `_bootstrap_file_identity(owner_controlled=True)` | `_preload_policy_module` (`st_uid != geteuid()` → refuse) | both sides, locally |
| `policy-ai-contracts` | same | same | both sides, locally |
| `policy-supervisor` | `running_source_gate_supervisor_identity` → `owner_controlled=True` | `running_supervisor_identity` → `_bootstrap_file_identity(owner_controlled=True)` | both sides, locally |
| `python` | `owner_controlled=False` | `owner_controlled=False` | **none, deliberately** — root-owned system file |

`python` still must be regular, single-link, non-group/world-writable, and is
still bound on every invariant field including its SHA-256. No member's
ownership is ever validated against the other side's value. A missing or
non-integer `uid` on either side is rejected rather than published as a null.

### Fix D — full-content source re-read and its exact narrow guarantee (`21c863e`, narrowed by `54028a1`)

After the copy the export seeks the same pinned descriptor to zero, re-reads and
hashes the whole source under the same logical-byte bound and export deadline,
takes a third `fstat`, and requires the re-read length and digest to equal the
staged copy and all three stat identities to agree. `source_reread_bytes` and
`source_reread_sha256` are recorded. `digest` is accumulated over exactly the
chunks handed to `write_all`, so "staged bytes" is the byte sequence actually
written to the destination, not merely the sequence read.

**The overclaim removal in `54028a1` is verified and is the load-bearing part of
this commit.** `21c863e` said a mutation before or during the copy "cannot
survive the comparison". That is false. The candidate's contract and the code
comment now state:

- what it establishes — the published bytes equal one complete post-copy re-read
  with all three stat identities agreeing, so the export cannot publish a view
  it never read as a whole;
- what it does not — detection of every mutation before or during the copy;
- that both reads are lock-free, so nothing excludes a concurrent writer or
  proves quiescence; a writer that can time both reads, or write the staged
  sequence back before the re-read, is not excluded;
- that **each of those needs the same-UID actor the boundary section already
  places outside this gate** — stated in the contract, in the code comment, and
  reinforced by the pre-existing boundary bullet "The manager, evidence root,
  registry, and cleanup paths assume no hostile process with the same host UID…
  No-follow/inode checks turn ordinary drift into `BLOCK`; they are not
  authentication against that actor."

The string `cannot survive the comparison` is absent from the candidate tree,
and `test_reread_guarantee_is_stated_narrowly` pins its absence together with
all four positive clauses. I confirmed the residual is not merely narrow but
*accurate*, by unconditional live probe: a complete stable replacement installed
before the export opens the source is exported 120 times out of 120 with
`source_reread_sha256` equal to the replacement. The contract describes
precisely this and calls it out as unclosed.

### Fix E — mandatory `source_reread` evidence at every consumer (`54028a1`)

`artifact_export_reread_errors` requires `source_reread_bytes` and
`source_reread_sha256` to be present, correctly typed (`bool` excluded from the
`int` check), and equal to the exported `bytes` / `sha256`. It is called from
both acceptance sites and only those two exist, verified by grepping the whole
tool directory for `artifact_export` consumers:

1. `validate_source_keyed_gate_chain` (line 1492), reached from
   `validate_provenance_payload` → `validate_locked_canonical_provenance` →
   `ai_contracts.py:969`, which is the `rust-source-bound-binary` review
   capability gate;
2. `build_source_bound_rust.main` (line 3757), the release launcher's own
   in-run acceptance.

`test_every_export_consumer_requires_the_pair` inspects the source of both to
prevent a future site from silently dropping the call, which is the only way to
cover the launcher's inline acceptance without a real gate run.

## 4. Live evidence — inspected, not mutated

All four artifacts were verified by `sha256sum` and `stat` before and after
every read; every digest matches the mandate, and every digest is unchanged at
the end of this review.

| Artifact | Bytes | Mode / nlink | SHA-256 |
|---|---|---|---|
| provenance `target/codefire-ai/rust-build-provenance.json` | 6455 | 600 / 1 | `d4f34a6bb81fdd7ead8d0fb76751d3335f393e2167c8c3a9d071da0dd29123d0` |
| gate result `…/rust-release-build-ddcfad8a1d7b4b94fa4daf94-0.json` | 37703 | 400 / 1 | `19a79f1be1b2938d9b82b3fd5385aa1a1a320246dc5b6267816ff79e813f573f` |
| gate log `…-0.log` | 3345 | 400 / 1 | `cf322384704f81e003aa33c1978b72210a1a2cab1859d298b888561b7722bb29` |
| binary `target/release/codefire-rs` | 14457880 | 700 / 1 | `09d9fa2561de4b687200baa4b9fe54a4159e43def1a7a3cd28dc05c2741741e4` |

Note that the gate result is **not** the one the stale review examined (it saw
37577 bytes / `b067eb7d…` bound to tree `691b5403…`); this is a later run bound
to the candidate.

Bindings recomputed:

- Provenance `base_revision = 54028a136943a9d5f9608e19d3855c0edae46adf`. Gate
  result `candidate.head_revision = 54028a1…`. Log first line binds
  `head_revision = 54028a1…`, `tree = e4057e1b…`. All three name the exact
  candidate.
- Result: `decision = PASS`, `blockers = []`, `candidate_unchanged = true`,
  `argv = ["cargo","build","--release","--locked","-p","codefire-cli","--bin","codefire-rs"]`.
- `log_binding.bytes = 3345`, `log_binding.sha256 = cf322384…` — recomputed from
  the log file and equal.
- Provenance `source_keyed_gate_result_bytes/sha256` and
  `build_log_bytes/sha256` equal the measured files.
- **Fix A live:** `source_nlink = 2`, `source_nlink_maximum = 2`,
  `source_cargo_uplift_hardlinked = true`, `export_nlink = 1`,
  `export_mode = 448`.
- **Fix D live (source re-read pair):**
  `source_reread_bytes = 14457880 = bytes`;
  `source_reread_sha256 = 09d9fa25… = sha256 =` the independently re-hashed
  binary on disk. The pair is present, consistent, and matches the artifact.
- **Fix B live:** the result file is 37703 bytes containing exactly one newline,
  at the end — the compact sorted UTF-8 durable rendering, not the `indent=2`
  form. Its existence alongside the provenance means the launcher's
  `gate_process.stdout != gate_raw` byte comparison passed on a real run.
- **Fix C live:** `outer_policy_binding.schema =
  codefire.rust_build_outer_policy_binding.v2`, with
  `outer_uid → gate_namespace_uid` of `1000 → 0` for
  `policy-build-source-bound`, `policy-ai-contracts` and `policy-supervisor`
  (`owner_validation = local-effective-uid`) and `0 → 65534` for `python`
  (`owner_validation = pinned-system-file-not-euid-checked`). This is direct
  evidence that the pre-fix strict `uid` comparison could never pass.
  The three recorded policy digests equal the candidate blobs exactly:

  | Member | Recorded SHA-256 | Candidate file |
  |---|---|---|
  | `policy-supervisor` | `c38984e7e61e51853b8e024721c3505c14fd8d322867b8598a448e8fbdd6ddfc` | `run_source_keyed_cargo_gate.py` ✓ |
  | `policy-build-source-bound` | `9eec2e312d090a6289a4fab6d882cadd32df4634a81b3d598f66d2584bd4d9f7` | `build_source_bound_rust.py` ✓ |
  | `policy-ai-contracts` | `e5eb305b93c59da89d71ed5509810958e63de3956fd7945072e2008db45561bf` | `ai_contracts.py` ✓ |

- **Fix E live, end to end:** `validate_locked_canonical_provenance(root,
  expected_revision="54028a1…")` executed against the real provenance under the
  real evidence lock returns identity `6455` / `d4f34a6b…` and
  **`errors: []`**. Four in-memory tamperings of the live export record are each
  rejected: stripping `source_reread_bytes` or `source_reread_sha256` →
  *"missing source re-read evidence"*; contradicting either →
  *"source re-read evidence differs from the exported bytes"*. The unmodified
  record returns `[]`.
- **Binary smoke:** `codefire-rs --version` → `codefire foundation 1`, rc 0;
  `--help` renders the command list. Digest, mode `700` and `nlink 1` unchanged
  after the smoke.

## 5. Validations

No Cargo, rustc, rustfmt, `cargo fmt`, `install.sh`, Docker, dogfood, or network
execution was performed. Nothing under `/workspace/.cache/codefire-gate-evidence`
or the source worktree was written.

### Test suite

| Where | Command | Result |
|---|---|---|
| candidate worktree | `python3 -m unittest test_ai_tools` | **`Ran 254 tests in 68.563s … OK`** — 0 failures, 0 errors |

Per-commit counts in isolated `git archive` trees under `/tmp` (not Git
checkouts, so a constant 4 errors + 1 skip appear on **every** commit including
the parent — `test_v2_canonical_port_packet_identities_match_git_objects`
(×2), `test_provenance_destination_is_ignored_untracked_target_only`,
`test_source_manifest_is_stable_and_covers_workspace_inputs`; all four are
environment-caused and identical across the chain):

| Commit | Tests run | New |
|---|---|---|
| `6db7944` (parent) | 218 | — |
| `14fc108` | 226 | +8 |
| `a45358a` | 232 | +6 |
| `3d23af7` | 240 | +8 |
| `21c863e` | 245 | +5 |
| `54028a1` | 254 | +9 |

Total +36 tests; no test is deleted (`test_ai_tools.py` is +1451/−0).

### Negative controls

Five directed controls, each a single reverting edit in a throwaway `/tmp` tree
built by `git archive`; the candidate worktree was never touched. Baseline for
these trees is 0 failures + the 4 constant environment errors.

| # | Reverted control | Result | Delta beyond baseline |
|---|---|---|---|
| A | `MAX_EXPORT_SOURCE_ARTIFACT_LINKS = 2` → `1` | 8 failures, 10 errors | 14 |
| B | full-content re-read comparison neutered (`if False:`) | 3 failures, 4 errors | 3 |
| C | `uid` restored to `POLICY_INVARIANT_IDENTITY_FIELDS` | 8 failures, 7 errors | 11 |
| D | `emit_durable_result` → `print(json.dumps(..., indent=2))` | 5 failures, 4 errors | 5 |
| E | `artifact_export_reread_errors` → `return []` | 16 failures, 4 errors | 16 |

Each control's failures are confined to the surface its fix owns. Control B is
deliberately narrow and is the honest number: it breaks exactly the two tests
that require torn-view rejection
(`test_mid_copy_mutation_is_caught_when_the_stat_oracle_is_blind` at both link
counts and `test_mutation_between_the_copy_and_the_reread_is_caught`) while
leaving `test_complete_stable_replacement_is_published_undetected` passing —
which is correct, because that test characterises the *limit* of the guarantee
and must pass with or without the check.

### Reviewer probes (mine; not covered by the writer's tests)

1. **ctime granularity, measured, no mocking.** `/workspace` (ext4): min
   3.9674 ms, median 3.9674 ms over 40 observed ticks. `/tmp` (tmpfs): median
   3.9680 ms over 40. Consistent with a 250 Hz tick.
2. **Real-clock torn-copy differential** (120 attempts each, 3 MiB two-link
   source, mutation through the sibling link after the first destination write,
   mtime restored) — the table in §2. `3d23af7`: 3/120 blind, 3 torn
   publications. `54028a1`: 0/120 blind, 0 publications, 120 `BLOCK`.
3. **Stable-replacement residual, real clock**, 120 attempts on `54028a1`: 120
   exports of the replacement, 0 blocks — unconditional, not clock-dependent,
   exactly as the contract states.
4. **Re-read cost**: 678 MB/s measured over the live 14.5 MB binary → a
   projected 0.8 s for the 512 MiB logical bound inside the 120 s export
   deadline.
5. **`F_SETLEASE` feasibility** — see O-3.
6. **Live pre-reservation BLOCK stdout differential** — see fix B above.
7. **Live consumer chain and four tamperings** — see §4.
8. **Binary smoke** — see §4.

### Repository checkers

| Command | Result |
|---|---|
| `python3 docs/development/tools/check_ai_contracts.py` | `AI contract check passed.` |
| `python3 docs/development/tools/check_source_docs.py` | `passed: 44 modules, 34 commands, 12 object kinds` |
| `python3 docs/development/tools/check_ai_acceleration_docs.py` | `AI acceleration docs check passed.` |
| `python3 docs/development/tools/check_ai_development.py --timeout 600` | 7 × `PASS`, `AI development gate passed.` |

## 6. Observations, impact and actionability

Every observation carries a compact screen over all impact axes. `unknown` is
never treated as neutral; no axis is omitted. Zero observations are
acceptance-critical-pending, so no disposition is left null.

### O-1 — the stale `42433c8` ACCEPT and its residual §1

Evidence: §2, probe 2.
Impact — frozen requirements/invariants: **affected** (the earlier disposition
would have carried a reproducible torn-artifact publication into integration).
Users/workflows/owners: **affected** (release provenance consumers; owner =
review manager). Categories: `correctness` **affected**, `data` **affected**,
`state` unaffected, `authority` unaffected, `api_compatibility` unaffected,
`operations` unaffected, `performance` unaffected, `resource` unaffected,
`maintainability` unaffected, `development_flow` **affected** (a superseded
verdict must not be cited as closure). Reachability: proved, 3/120 on the real
host clock. Likelihood: low per attempt, unbounded under retry. Blast radius:
the published release binary. Severity: **high**. Reversibility: the published
export is replaced on the next successful run; a consumed torn binary is not
recoverable from the evidence chain. Recurrence likelihood: closed by `21c863e`
plus a pinned regression test. Evidence confidence: high; reproducible;
availability **available**. Repair cost: already paid.
Disposition: **`duplicate`** of the canonical effect now closed by `21c863e` —
recorded here only to supersede the earlier `nonblocking` label. No new repair.
No required correction against the current candidate, whose behaviour is proved
correct. Causal class: null.

### O-2 — `54028a1` narrowing of the re-read guarantee

Evidence: §3 fix D, probe 3, `test_reread_guarantee_is_stated_narrowly`.
Every axis: **unaffected** except `correctness` and `development_flow`, which
are **positively affected** — the contract now matches the code, the same-UID
actor is explicitly bounded outside the local trusted-source model in three
places, and a regression test forbids the overclaim's return. Evidence
availability **available**.
Disposition: **no observation against the candidate.** Recorded as
affirmatively verified.

### O-3 — `F_SETLEASE` proposal: independently **confirmed** as `nonblocking_improvement`

The manager's adjudication proposes `nonblocking_improvement` because compliant
Cargo writers are excluded by the global lock and a same-UID actor can replace
the whole unsigned evidence chain. I **confirm** the disposition and confirm
both reasons, and I add measured evidence that makes the conclusion stronger
than either reason alone.

Both manager reasons verified:

- The export runs under the evidence → global → registry → artifact-namespace
  lock order, and the contract's global lock "serializes all Cargo gates". No
  compliant concurrent Cargo writer can exist during the export. **Confirmed.**
- The boundary section already states that a same-UID actor "can race
  directory-entry cleanup or replace a complete unsigned result/log/provenance
  chain", and that no-follow/inode checks "are not authentication against that
  actor". Replacing the evidence chain strictly dominates tampering with the
  export source. **Confirmed.**

Measured additions — a lease would not close either documented residual:

| Probe | Measurement |
|---|---|
| `F_SETLEASE(F_RDLCK)` on `/tmp` (tmpfs) and `/workspace` (ext4) | succeeds; `F_GETLEASE` = `F_RDLCK` |
| rename-swap of a different inode over the leased path | completed in 0.1 ms; **lease still held**, `F_GETLEASE` still `F_RDLCK` |
| replacement completed *before* the export's open | 120/120 exported (probe 3) — no lease can reach back before its own acquisition |
| in-place open-for-write during the copy window | already 120/120 `BLOCK` on the candidate (probe 2) |
| lease holder with no `SIGIO` handler | **killed by `SIGIO`** (process terminated, `I/O possible`) |
| lease holder with a handler; conflicting open on the sibling link | writer's `open()` blocked 5.009 s until manual release; `/proc/sys/fs/lease-break-time = 45` |

So the only window a lease adds coverage for is the one the content re-read
already closes, while the two documented residuals — a replacement finished
before the open, and an atomic rename-swap — are untouched by it. Adopting it
would instead introduce a real operational hazard: up to 45 s of forced blocking
inside a 120 s export deadline while the export holds the evidence, global,
registry, and namespace locks, and a mandatory `SIGIO` disposition change to
avoid killing the manager mid-export.
Impact: `correctness` unaffected (adds no coverage the candidate lacks),
`operations` **affected if adopted** (lock-held stall), `resource` unaffected,
`authority` unaffected (does not move the same-UID boundary),
`api_compatibility` unaffected, `data` unaffected, `state` unaffected,
`performance` unaffected, `maintainability` unaffected, `development_flow`
unaffected. Reachability of the covered case: already blocked. Evidence
availability **available**.
Disposition: **`nonblocking_improvement`**, confirming the manager. Under the
operating protocol no must-fix trigger is proved — `MF-2` fails because the
residual is explicitly placed outside accepted scope by the boundary section,
and `MF-1` fails because no frozen clause requires writer exclusion; the
contract affirmatively disclaims it. **Scope is not expanded.** Causal class:
null. Routed as a future, separately-owned item only if the batch ever needs a
hostile-source lane, which the contract already names as needing a separate
service UID plus append-only/signed evidence — a strictly stronger control than
a lease.

### O-4 — `acceleration-master-plan.md` AIA-021 / AIA-046 rows omit the three new behaviours

Evidence: the AIA-046 and AIA-021 rows describe the gate and release wrapper but
mention neither the two-link export-source admission, nor the durable-stdout
single representation, nor the mandatory `source_reread` pair; the four changed
paths do not include the master plan.
Impact: `maintainability` **affected** (summary currency); every other category
**unaffected** — no row *contradicts* the new behaviour (I grepped the whole
`docs/` tree: no stale "single-link export source" claim survives outside the
updated contract), the authoritative contract for this subsystem is
`source-keyed-cargo-gates.md` and it is updated, and
`check_ai_acceleration_docs.py` passes. Reachability: documentation reader only.
Severity: **low**. Reversibility: trivial. Evidence availability **available**.
Disposition: **`nonblocking_improvement`**. No frozen rule requires per-repair
master-plan rows; AGENTS.md's canonical-source-doc obligation covers the four
product docs, which are untouched and pass their checker. Causal class: null.

### O-5 — contract states ctime granularity as "about every 3.8 ms"; measured 3.967 ms

Evidence: probe 1.
Impact: `maintainability` **affected** (a measured constant is ~4 % low); every
other category **unaffected** — every conclusion resting on it depends on
quantisation existing, not on its exact value, and the candidate's oracle is
content equality, which the comment itself says "does not depend on winning a
timestamp tick". Severity: **low**. Reproducible; availability **available**.
Disposition: **`nonblocking_improvement`**. Causal class: null.

### O-6 — two `GateBlocked` classes still escape `main` without emitting stdout

Evidence: `publish_reserved_result` can raise `GateBlocked(
"result-payload-exceeds-bound")` from `rendered_result` before its own `try`,
and `GateBlocked("result-publication-rollback-failed")` from the rollback path.
Neither is caught by the handlers' `except (OSError, ValueError)`, so both
propagate out of `main`.
Impact: `correctness` unaffected — control flow here is **byte-identical to the
parent** (`6db7944` skipped its `print` in exactly the same way), and it fails
closed: stdout is empty, the result path still holds the mode-`0400` canonical
`BLOCK` reservation, and the launcher's `gate_process.stdout != gate_raw`
comparison raises before any decision is read. `data`, `state`, `authority`,
`api_compatibility`, `performance`, `resource` unaffected. `operations`
**affected** at low severity (a traceback and rc 1 instead of a structured
result). `maintainability` **affected** (the contract sentence "every terminal
gate path — PASS, `BLOCK`, internal error, and the reservation fallback after a
failed publication — writes the exact bytes it made durable" enumerates four
paths and these are exits that never made bytes durable, so I do not read it as
contradicted; but it can be read as exhaustive). Reachability: requires a
result payload over the 1 MiB cap, or a rollback failure. Severity: **low**.
Evidence availability **available**.
Disposition: **`nonblocking_improvement`**. Not introduced, not widened by any
of the five commits; no `MF` trigger. Causal class: null. (The same effect was
recorded in the superseded `42433c8`; it is retained here as a distinct
observation because that report is not in this lineage.)

### O-7 — pre-`main` bootstrap refusal emits no JSON

Evidence: invoking the gate without the exact `-I -S -B -X
pycache_prefix=/dev/null` flags raises `RuntimeError` at module import, rc 1,
empty stdout, no result file.
Impact: `operations` **affected** at low severity; all other categories
**unaffected**; fails closed at the consumer for the same reason as O-6.
Pre-existing and untouched by the candidate; the release launcher always passes
the exact flags (verified in `source_keyed_build_argv`).
Disposition: **`out_of_scope`** — outside the frozen candidate scope, with no
in-scope or cross-cutting violation proved. Retained and routed; it removes no
changed path from coverage. Causal class: null.

### O-8 — destination mode is not re-asserted after the rename

Evidence: the post-rename `exported` predicate checks `S_ISREG`, `nlink == 1`,
`(dev, ino)` and `size`, but not mode. `fchmod(0o700)` is applied to the staged
descriptor before `fsync`, and `os.rename` preserves the inode.
Impact: `correctness` unaffected in practice — the rename cannot change the
inode's mode, the consumer independently requires `S_IXUSR` and
non-group/other-writable ancestors on the published binary and cross-checks the
recorded `export_mode`, and the live export records `export_mode = 448` against
a binary that re-stats to `700`. All other categories unaffected;
`maintainability` **affected** at low severity. Unchanged by all five commits.
Disposition: **`nonblocking_improvement`**. Causal class: null.

### O-9 — the re-read doubles export read I/O

Evidence: probe 4 — 678 MB/s measured; 0.8 s projected for the 512 MiB logical
bound inside the unchanged 120 s `ARTIFACT_EXPORT_TIMEOUT_SECONDS`. The contract
discloses that the re-read runs "under the same logical-byte bound and export
deadline".
Impact: `performance` and `resource` **unaffected** (measured, not assumed); all
other categories unaffected. No frozen operations/resource bound is breached, so
`MF-4` does not fire.
Disposition: **`invalid_or_unsubstantiated`** as a concern — measurement
contradicts it. Recorded so the axis is not left unknown. Causal class: null.

### O-10 — two launcher-side supervisor identity readers

Evidence: `running_source_gate_supervisor_identity` (new in `3d23af7`) and the
pre-existing `open_source_gate_supervisor` both assert supervisor identity from
the launcher with independently written predicate lists.
Impact: I checked for present divergence and found none — both require
`S_ISREG`, `st_uid == geteuid()`, `nlink == 1`, `mode & 0o022 == 0`, a ≤ 1 MiB
size (`BUILD_BOOTSTRAP_MAX_BYTES == 1024 * 1024`, matching the inline literal),
and `(dev, ino)` agreement between the descriptor and a no-follow `os.stat`. So
`correctness`, `authority` and every other category are **unaffected** today;
`maintainability` **affected** at low severity (a future tightening applied to
one and not the other would diverge silently). Evidence availability
**available**.
Disposition: **`nonblocking_improvement`**. Not tied to any frozen criterion, no
pre-candidate measurable violation, no reachable failure. Causal class: null.

## 7. Required corrections

**None.** No must-fix trigger is proved against `54028a1`:

| Trigger | Status |
|---|---|
| `MF-1` frozen requirement/invariant/guardrail violated | not proved — the contract now matches the code on every claim I could test, including the two overclaims `54028a1` removed |
| `MF-2` reachable material correctness/state/data/authority failure in accepted scope | not proved — 120/120 torn-view rejection on the real clock; the surviving replacement window is explicitly outside the accepted local trusted-source model |
| `MF-3` public API / persisted state / cross-generation contract broken | not proved — `artifact_export` and the policy binding gain fields; the binding's schema version is bumped `v1 → v2` rather than reinterpreted, and both consumers are updated in the same candidate |
| `MF-4` frozen operations/performance/resource/availability/recovery bound breached | not proved — measured 0.8 s worst-case re-read inside an unchanged 120 s deadline |
| `MF-5` required acceptance evidence absent/contradicted/invalid | not proved — live provenance validates with `errors: []` against the exact candidate revision under its real evidence lock |
| `MF-6` repair-introduced regression or reopened closed obligation | not proved — 254/254 pass; five directed negative controls are non-vacuous; every earlier closed surface I re-probed (destination strictness, prior-destination safety, no-follow open, stdout single representation) still holds |

No repair is authorized by this review, and none is requested.

## 8. Residual and non-goal boundaries

Recorded so this verdict is not mistaken for a clean bill. None is introduced
or widened by the five commits, and each is already written into the candidate's
own contract.

1. **A complete, stable replacement installed before the export opens the
   source is published.** Measured 120/120, unconditional. Documented verbatim
   and pinned by `test_complete_stable_replacement_is_published_undetected`.
   Requires the same-UID actor the boundary section excludes.
2. **Both reads are lock-free.** Nothing excludes a concurrent writer or proves
   quiescence; a writer that can time both reads, or write the staged sequence
   back before the re-read, is not excluded. Documented.
3. **Same-host-UID hostile or unsupervised actors are outside the model.** They
   can race directory-entry cleanup and replace the whole unsigned
   result/log/provenance chain — strictly more powerful than tampering with the
   export source. A stronger lane needs a separate service UID plus
   append-only/signed evidence, not a lease (O-3).
4. **Not hermetic.** Not a reproducible-build or dependency supply-chain
   attestation; the mount namespace is not a pivot-root sandbox; the capability
   drop is not a durable capability boundary; the live supervisor is not a
   cgroup or filesystem quota. All pre-existing and documented.
5. **Transcript validation, not a structured test protocol.** The non-vacuity
   claim holds only for reviewed/trusted candidate source.
6. **Evidence retention is two slots, not an archive.** A copied review
   provenance may expire after later canonical builds rotate its slot.
7. Non-goals of this review: no Cargo/rustc/rustfmt/install/Docker/dogfood/
   network execution; no mutation of any implementation, doc, tool, or evidence
   path; no integration; no repair; no expansion of the frozen scope to
   `F_SETLEASE` or to any other new control.

## 9. Verdict

**ACCEPT_FOR_INTEGRATION** — reviewer proposal, not integration authority.

The five fixes are correct, minimal, mutually consistent, and each is
independently corroborated by live evidence bound to the exact candidate tree
`e4057e1b6f16b12fed45ad0b50bc4c6014ca7ebd`. Fix A admits exactly the link shape
Cargo's uplift produces and nothing wider, keeping every neighbouring source and
destination predicate. Fix B makes the durable bytes the single representation
on every terminal path that reaches an emission, verified by live differential
against the parent. Fix C repairs a total, always-failing cross-namespace uid
comparison without weakening ownership on either side, and binds the gate's own
code as the member that authorizes trusting the gate's local check. Fix D closes
a reproducible torn-artifact publication — 3 in 120 real-clock attempts on the
pre-repair tree, 0 in 120 on the candidate — and, critically, `54028a1` states
what it closes and what it does not with no residual overclaim, placing the
same-UID actor explicitly outside the local trusted-source model. Fix E makes
the re-read pair a requirement at both consumers rather than decoration, proved
against the live provenance and against four directed tamperings of it.

The stale `42433c8` ACCEPT is **explicitly superseded**: it bound a shorter,
different subject, and its "not blocking" disposition of the same-tick window is
refuted by reproducible counterexample. Ten observations are recorded with full
impact screens; none is a required correction, and the `F_SETLEASE` proposal is
independently **confirmed** as `nonblocking_improvement` on measured grounds
stronger than the two the manager cited. Zero required corrections are currently
open for this exact candidate.

**This review does not integrate.** Integration remains a manager decision
requiring the machine recurrence gate and the manager's separate
impact/actionability/repair-authority artifact referencing this report's commit
and digest.
