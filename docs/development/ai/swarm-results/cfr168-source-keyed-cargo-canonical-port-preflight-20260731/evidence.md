# CFR-168 Source-Keyed Cargo Canonical-Port Preflight

Recorded: 2026-07-31  
Role: portfolio-manager read-only preflight  
Decision authority: none  
Port execution: forbidden until the packet admission blocker is satisfied

## Disposition

This record fixes mechanical input facts for a possible future semantic port
of source-keyed Cargo candidate
`4e5c0e7f058cd406cb7a183ecda8b3a07f3ee912`. It does not review or accept that
candidate, does not authorize a port, and does not infer closure from its green
manager gates.

The exact independent reviewer session
`d089c8ae-8364-49af-bcd1-b9bf4b741ea1` still has no report. If its eventual
decision is `REQUEST_CHANGES`, these facts remain historical input only and the
port packet must be rebound to the accepted successor.

## Frozen Identities

| Subject | Commit | Tree |
|---|---|---|
| historical source-range base | `893cbcad2b1f2a8455a5c256246c8110d3fcbdd6` | `daeb9075f9999d87caaf93a1a9d8cc4e266d9ad6` |
| candidate | `4e5c0e7f058cd406cb7a183ecda8b3a07f3ee912` | `6417400cb7b5a3507f85f38d66c0573992ad4007` |
| canonical comparison snapshot | `9c1eefdea5dd591aa832625320516f2b3d9138af` | `85db20ca5f05c0a48ca038922731af21b8b77203` |

The candidate worktree was porcelain-clean at
`2026-07-31T10:18:01Z`; the manager revalidation record binds that observation
and the exact worktree.

The binary source-range patch has SHA-256
`6194825df9517165ee8fb517246c11220b850ff7877d29d88d30a644bf2bbb7c`
and contains:

```text
19 files changed, 21858 insertions(+), 284 deletions(-)
5 added paths
14 modified paths
```

The count describes the exact finite Git range; it is not a quota, quality
score, or substitute for per-path semantic disposition.

## Canonical Collision Preflight

At comparison snapshot `9c1eefd`, 17 of the 19 source-range paths have the
same state as the historical base:

- all five source-added paths are absent from both historical base and
  canonical; and
- twelve source-modified paths have an identical historical-base/canonical
  blob.

Only two paths already diverge on canonical and therefore require an explicit
three-way semantic disposition:

| Path | Historical base blob | Candidate blob | Canonical snapshot blob |
|---|---|---|---|
| `docs/development/ai/swarm-batches/cfr168-rust-file-doctor-review-note-closure-rereview-claude-20260730.md` | `8a31d96909464d2b2d87882dec879f31c62c31af` | `0d6769a6b055d74ff94d635c82ae43c213467f88` | `850ce81e3a2eaad9e593c6f1e9c92d67e012f5e0` |
| `docs/development/ai/swarm-batches/cfr168-rust-http-mr-preflight-evidence-closure-rereview-claude-20260730.md` | `7f7f14466778896fa4504a9aa280f007eb1b9880` | `1d01e9369a93964ef889da11cab6a803e66d17b2` | `eea642a152236cd03e0c27b2c904837e4f813c59` |

This is not permission to copy the other 17 candidate blobs. Canonical can
advance before port execution, and source-range files contain historical
current-state prose. The implementer must recompute all 19 base/candidate/
then-current-canonical blob identities after admission and start from
canonical text.

## Test-Name Census

The following normalization was applied independently to the historical base
and candidate versions of `docs/development/tools/test_ai_tools.py`:

```bash
git show <revision>:docs/development/tools/test_ai_tools.py \
  | rg '^\s+(async\s+)?def test_' \
  | sed -E 's/^\s*(async\s+)?def ([^(]+).*/\2/' \
  | sort -u
```

| Set | Unique names | SHA-256 of newline-terminated sorted set |
|---|---:|---|
| historical base | 65 | `8a0877c73ad014b9fbc96656667f0aa4beabd9fa827b00b2303101f38040538f` |
| candidate | 168 | `74dd287f105ab3c6ff919cbf7715e690342d95524216e1e91b9d6205e533b81e` |
| candidate minus base | 104 | `c49179f121f9e3853dd6c471af3b60bba3affe5fd091bb694cfc6c20ba887dcf` |
| base minus candidate | 1 | `d59c4837781dab901b61d0f43f81a5624f12efdbf97a57c0d30d501c3bb9e689` |

The one removed name is:

```text
test_shared_low_memory_process_passes
```

It was not silently deleted. The candidate replaces the formerly permissive
shared-root assertion with:

```text
test_unkeyed_shared_root_process_is_blocked
test_source_keyed_target_namespace_passes
```

Therefore the future port must:

1. preserve every one of the 104 candidate-introduced names exactly once;
2. explicitly disposition the removed permissive test and both replacement
   tests;
3. preserve every unrelated test present on then-current canonical;
4. report the complete source, canonical-base, and port name sets and digests;
   and
5. execute the source-gate classes separately with nonzero selection.

No total test count can prove these requirements. For example, one dropped
test and one duplicated or unrelated test can leave a plausible total.

## Recompute Contract

After candidate acceptance and before any edit, the future implementer must
recompute and report:

- source-range patch digest and path set;
- base/candidate/then-current-canonical blob for every source-range path;
- normalized source, canonical-base, added, removed, and proposed-port test
  name sets;
- every path whose canonical blob changed since comparison snapshot
  `9c1eefd`; and
- whether the candidate result/document claims still match the exact accepted
  review.

Any mismatch is a port input requiring explicit disposition, not a reason to
update this immutable preflight retroactively.

