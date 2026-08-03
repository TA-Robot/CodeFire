# CFR-168 C14-LIVE-A1 Manager Adjudication

Date: 2026-08-03

## Decision subject

This adjudication binds the exact reviewed line ending at R5 commit
`7bd4594f01db6d8298ca242db04b0f71ce34f040`, tree
`d3bd3a2c60fab7e34e25726305733bf91a2fe134`, whose sole parent is W5
`c57ac240cc6dc2011f78ba6148123804b9aa4258`. R5 adds only
`review.md`, has terminal `REVIEW_VERDICT: ACCEPT`, and has raw report SHA-256
`2facd0caef19f993f7e33e5be07a6f4e80ee4b7855756f6d1b49d584565d1a08`.

The accepted A1 code/test subject is exact O1
`46d98466c702f111b58a3c1ea080518f75a59580`. It is a single-parent child of
F5, changes only two stale expected literals in
`tests/test_python_existing_terminal_owner.py`, and leaves the C4 `codefire`
blob byte-identical. C4 remains the two-path A1 product candidate at
`19010f742d358c5c03e19d091c4249420f9e53b9`.

## Evidence adjudication

The manager accepts the immutable ordered O1 artifacts produced by the stopped
predecessor session. Their origin is disclosed, the pre-run identity binds
exact clean O1, every artifact is content-addressed, W5 rechecked the complete
set, and R5 independently rehashed all artifacts and reproduced the paired
comparator. No conclusion depends on mutable session state or an inadmissible
candidate-worktree mutation log.

The paired C4/O1 discovery evidence contains the same 786 ordered method IDs.
Its only status changes are the two authorized `FAIL` to `ok` transitions; the
two remaining installer failures are byte-equivalent after root normalization
and are retained as environmental/unavailable, never PASS. The fresh AO-01 and
AO-02 controls are exact one-at-a-time O1-export mutations and restore the
complete export manifest.

## Observation dispositions

| Observation | Manager disposition | Repair authority |
|---|---|---|
| W5-O1 stale C4 adjacent oracles | `required_correction`, causal class `implementation_nonconformance`; closed by exact O1 and its paired/reversal evidence | none remains |
| W5-O2 900-second timeout | `out_of_scope` environmental/unavailable evidence; retained without conversion to PASS | none in A1 |
| W5-O3 absent-Cargo installer failures | `out_of_scope`; applicable Install evidence remains required after integration | none in A1 |
| W5-O4 historical candidate-worktree mutation | `required_correction`, causal class `implementation_nonconformance`; old logs remain inadmissible and fresh external-export reversals close the evidence defect | none remains |
| R5-N1 | `nonblocking_improvement`; the historical required correction receives the causal class above | none |
| R5-N2 and R5-N3 | `nonblocking_improvement`; raw command bytes and both environmental failures are explicit in immutable evidence/R5 | none |
| R5-N4 | `nonblocking_improvement`; immutable cross-session evidence is admitted for the reasons above | none |

No observation has unknown acceptance impact, no must-fix trigger remains, and
no further A1 repair or generic review is authorized.

## Integration authority and remaining scope

C14-LIVE-A1 is accepted as the exact source package represented by the reviewed
line above. The manager selects this line as the A1-integrated product base for
the next bounded composition step. A2/B3 must be rebound to the resulting
manager adjudication descendant and must recompute its response/wire tests
against A1 exception behavior. Whole-file `codefire` selection or textual
conflict resolution without the frozen composition contract remains forbidden.

This decision does not accept A2/B3, MR-write, any successor owner, the whole
CFR-168/CFB-138 root, Install, Dogfood/Evaluate, final Close, or Retrospect.

MANAGER_ADJUDICATION: ACCEPT_A1_SOURCE_PACKAGE
