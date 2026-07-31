# CFR-168 KGI/KPI Control-Plane Composition Preflight

Recorded: 2026-07-31  
Role: portfolio-manager read-only preflight  
Decision authority: none  
Composition execution: forbidden until both packet admission rows are satisfied

## Disposition

This record fixes mechanical input facts for a possible future semantic port
of KGI/KPI candidate
`753c8db103579d021795d3fe6a97c5d45d4a111a` onto an accepted Cargo-enabled
canonical head. It does not review or accept the KGI/KPI candidate or the Cargo
candidate, and it does not authorize either port.

The exact KGI/KPI reviewer session
`60548555-ff29-4fd9-9360-2820a663ee54` still has no report. If it eventually
returns `REQUEST_CHANGES`, the composition packet must bind its accepted
successor and recompute this census.

## Frozen Identities

| Subject | Commit | Tree |
|---|---|---|
| KGI/KPI source-range base | `9dd0da4f2ae55742ee51c1671aab6308994de24c` | `c3319a78239eb4e25d9e98984d82ecd722c20653` |
| KGI/KPI candidate | `753c8db103579d021795d3fe6a97c5d45d4a111a` | `213e16a033aeb2e8eb3599be4ffe725b1d7ad344` |
| canonical comparison snapshot | `c78f1023c3aa80b27baf1731cfa67ba816fb5d9e` | `bdc65463a9663dfa96f04f23ad6163ec99cd88f3` |
| protected Cargo source candidate | `4e5c0e7f058cd406cb7a183ecda8b3a07f3ee912` | `6417400cb7b5a3507f85f38d66c0573992ad4007` |

The KGI/KPI candidate worktree was porcelain-clean at
`2026-07-31T10:18:01Z`; the post-reboot manager evidence binds the exact
worktree and observation.

The binary KGI/KPI source-range patch has SHA-256
`09b7c18fe0651541e16fbc6ca8aea5e8dcd128fe221284efee0e1406dc43eb95`
and contains:

```text
17 files changed, 7790 insertions(+), 447 deletions(-)
3 added paths
14 modified paths
```

These are finite Git-range facts, not a target size or evidence of review
quality.

## Canonical Collision Preflight

At comparison snapshot `c78f102`, eleven source-range paths had the same state
as the historical KGI/KPI base. Six had already diverged and require a
then-current semantic three-way disposition:

| Path | KGI/KPI base blob | KGI/KPI candidate blob | Canonical snapshot blob |
|---|---|---|---|
| `docs/development/ai/README.md` | `2a5ec1d5677304af1c1e040674584e71038203e0` | `d1a7e71bf4332b22e7aab381db3f79fc9c1ee2bb` | `aa061c9624e05c5b28b6f996a916a28af1569695` |
| `docs/development/ai/acceleration-master-plan.md` | `c8e30a39041cd154f4f13851ca4bb4f97bf11e1b` | `37edc4970676ce961547e73f03d671d08e0e16a1` | `fb13789236a4d1b624723bf1e3259cf56dc80f80` |
| `docs/development/ai/development-flow-kgi-kpi.md` | `b7dc0066ff6e19c984088159241b08e8a3e32c1f` | `127a335e95e94c07d499218f17bacde046a6998b` | `5c250733aea3cd7d0d9f22cf1cfd58aea293e70a` |
| `docs/development/ai/task-packets/development-flow-kgi-kpi-zero-residual-repair-20260730.md` | `f9054f5ab8c3f65cd3ad663b42dcc9c18972e6d4` | `046b0d3b1c886d5052e0ac1a9a8d1dc7bbb18e5b` | `0d7dd64b56adf940ef7cf9fd7bcbbc818a6622d8` |
| `docs/development/history.md` | `3a8431baed772eedaf54ef919999856a852515ad` | `61360a424557221ccd770063c3ef86cb9ba3b5a0` | `660898a027c6f5485f6a43d34d65ef389aec4153` |
| `docs/development/tools/test_ai_tools.py` | `c067d47ea50aab002254b933f81eb29ef3663da6` | `c52633013e05da8b37704679360ba1b668ebfe24` | `906a448d10242fa58d9ed91daef9ae769daf79ed` |

The comparison precedes the Cargo canonical port. After that port,
`ai_contracts.py` and the other four declared Cargo/KGI overlaps must also be
treated as protected Cargo collisions even if their current blobs happened to
match an old base here.

The implementer must recompute all 17 path blobs after both admission rows are
satisfied. Do not copy one of these candidate blobs over later canonical
incident, routing, review, or contract text.

## Test-Name Census

The same normalized method-name extraction used by the Cargo preflight gives:

| Set | Unique names | SHA-256 of newline-terminated sorted set |
|---|---:|---|
| KGI/KPI base | 42 | `85db77b3fd693bf97cded1f5be08b9636d6572cd1858a71e401db943f51a2dc3` |
| KGI/KPI candidate | 129 | `4d1f4c8f6b436c25fc58de3564be32fd1b4ca06a184953d3a4a318639bacac5a` |
| KGI/KPI candidate minus its base | 87 | `f59c45a138141e8d6b5a730a4807748b3a8e35ef99dea7849faf55c378c8a4ce` |
| KGI/KPI base minus its candidate | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |

Against Cargo candidate `4e5c0e7`:

- the KGI/KPI 87-name added set has zero names in common with the Cargo
  candidate set;
- the two candidate sets share exactly the 42 older base names; and
- their unique-name union contains 255 names with SHA-256
  `9ff20e639cf2da8343c9fd91d8892e18d586999c721046302f5982410da06adf`.

The union is a source-input lower bound, not an exact future composition
count. The Cargo canonical port or KGI/KPI conflict resolution may add valid
composition tests. Acceptance requires set inclusion and execution:

1. every accepted Cargo-port test name remains exactly once;
2. all 87 KGI/KPI-added names appear exactly once;
3. every later unrelated canonical test remains;
4. Cargo and KGI/KPI classes each execute separately with nonzero selection;
   and
5. a total such as 255 never substitutes for the name-set comparisons.

## Recompute Contract

After the accepted Cargo port is integrated and the KGI/KPI source receives
`ACCEPT`, the composition implementer must recompute:

- all source/canonical/port path blobs;
- Cargo-port, KGI/KPI source, added, removed, overlap, and proposed-composition
  test-name sets;
- the five declared Cargo/KGI file collisions plus any new canonical
  collisions;
- source and then-current policy/history claims that cannot be combined
  literally; and
- every counterexample and deliberate regression named by both accepted
  inputs.

Any mismatch is evidence requiring disposition, not permission to rewrite this
preflight or weaken one input contract.

