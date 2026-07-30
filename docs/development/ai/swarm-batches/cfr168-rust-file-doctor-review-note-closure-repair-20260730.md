# CFR-168 Rust File Doctor Review-Note Closure Repair

Use Claude Opus 5 (`claude-opus-5`) as the bounded implementer. Work only in
the assigned worktree and declared write set. Add commits; do not amend,
rebase, merge, integrate, or change issue state.

## Exact ancestry and evidence

- accepted predecessor: `52a025ecda3fe2f5ad8b5b2a485562e7c16ab6de`
- eight-residual repair candidate:
  `1a04490806b59f8c39b5c1cf03f42d06aa750a98`
- accepting exact rereview report:
  `1c555ff3f426d7fa6874db92ceb71515a8d908e6`
- report SHA-256:
  `f7c52b05e312cc131fdfc2e2c0d9d486f5ee372b5ec93a85fdb44a95fb3ce1b9`
- report:
  `docs/development/ai/swarm-results/cfr168-rust-file-doctor-zero-residual-rereview-claude-20260730/review.md`

The rereview accepts all four prior Should and four prior Could closures, but
records three new notes. CFR-168 uses a stricter zero-residual completion
policy, so resolve the notes without reopening any accepted invariant.

## Required closure

### N1 — close the production panic boundary deliberately

`observe_file_remote_doctor_failure` currently uses a production `panic!` when
failure cleanup cannot restore zero ownership, including an open permit.

1. Preserve fail-closed behavior: an ownership invariant violation must never
   publish a normal report, replace the primary failure silently, or continue
   with hidden debt.
2. Prefer a typed terminal invariant failure propagated through the real
   `cf file doctor` route when ownership can be settled safely. Do not replace
   the panic with a normal error that leaves an open permit or untracked debt.
3. If an impossible internal open-permit state genuinely requires process
   abort, isolate it behind one explicit invariant boundary, prove every
   production Store/Doctor path prevents that state, and document why a typed
   return cannot preserve the ownership contract. A generic `panic!` plus an
   injected test-only permit is not sufficient reasoning.
4. Preserve the original primary error, terminal reason, observer ordering,
   semantic tree identity, and release-build enforcement.
5. Test debug and release behavior through the production factory and route.
   Deliberately regress the chosen boundary and prove the oracle fails.

### N2 — reserve before digest and path projection allocation

`read_doctor_json` currently creates the 64-byte hex digest and
`path.display().to_string()` before the corresponding retained fence frame is
claimed.

1. Compute a conservative allocation upper without first allocating the
   covered digest/path projection.
2. Reserve/claim the correct retained or temporary dimension and real
   `DoctorGrowthSite` before each allocation, then retain or release it
   symmetrically.
3. Bound non-UTF-8 and maximum-length paths without formula slack that rejects
   an otherwise admissible record.
4. Make the allocation/claim order observable at the actual production site;
   deleting or moving the claim after either allocation must fail a focused
   test for the intended reason.
5. Preserve exact record digest/fence semantics and existing diagnostics.

### N3 — fence unavailable subjects, not only parsed bytes

When metadata or bounded reading fails, the scan currently records an invalid
diagnostic but no fence subject. A document can become readable before
publication while the stale invalid conclusion is still emitted.

1. Represent an unavailable subject explicitly. Bind the pinned parent
   directory capability, leaf name, observed file-kind/identity information
   when available, operation boundary, and typed failure origin/class without
   converting denial or I/O failure into absence.
2. At the final `RecordFence`, reperform the same bounded authority operation.
   Unavailable→readable, missing→created, replaced parent, symlink,
   identity/kind change, different failure origin, or changed readable bytes
   is `subject_changed` and must fail before publication.
3. A still-unavailable subject may preserve the original diagnostic only when
   the pinned authority and typed unavailable state are unchanged.
4. Cover metadata failure, bounded-read failure, permission denial through a
   deterministic seam, disappearance, creation, symlink replacement,
   unreadable→readable, and multi-fault precedence.
5. Every detected change leaves the semantic repository tree byte-identical,
   zero debt, no permit, and one terminal observation.
6. Deliberately remove the unavailable-subject record and final recheck; the
   production-path oracle must fail.

## Preservation and composition

- Preserve the accepted production factory/profile/label, growth-site
  attribution, gate fields, full subject fence, retired record cap,
  retryable cleanup, release enforcement, and all eight deliberate-regression
  oracles from `1a04490`.
- Preserve quick/full behavior, HTTP delegation, error precedence, exact CLI
  output, and unrelated File/Store/HTTP behavior.
- Do not claim the command-wide CLI envelope/text-output accounting item
  closed. It remains owned by the later response-projection packet.
- Keep shared source documents factual. Do not edit manager packets, reviews,
  ledgers, issue state, or unrelated modules.

## Required evidence

1. exact base/head/tree/patch/changed paths and clean state;
2. one explicit disposition and production proof for N1, N2, and N3;
3. exact/minus-one/plus-one allocation boundaries and path-size cases for N2;
4. unavailable-state transition and multi-fault matrix for N3;
5. debug and release N1 evidence;
6. all prior 20 focused File Doctor tests and eight reversals still pass/fail
   as intended;
7. new N1–N3 deliberate regressions fail for the intended reason;
8. full authorized CLI/core, strict Clippy, format, source-doc, and diff
   gates with the pinned Rust toolchain.

Use focused tests while iterating. Run broad validation once after the exact
candidate is stable. Commit every intended source/test/document change and
leave the worktree clean. End with exact handoff evidence; do not integrate.
