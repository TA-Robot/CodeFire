# CFR-168 Python HTTP Upload + MR-list Composition Preflight

Recorded: 2026-07-31  
Role: portfolio-manager mechanical preflight  
Comparison canonical: `61459bc`  
Product mutation: none  
Integration authority: none

## Disposition

This record proves only that the pending Upload product delta and the already
accepted MR-list product delta can be ported, in the frozen composition order,
onto the accepted Doctor+Branch scaffold without a Git conflict and without
failing the selected composed-tree gates.

It does **not**:

- accept Upload candidate `446cf4a`;
- integrate either candidate;
- close CFR-168, CFB-136, or CFB-138;
- discharge the four manager-routed MR-list observations;
- supply the shared-primitives `READY` report;
- authorize MR-write implementation or Python composition; or
- replace a fresh review of the eventual exact composition commit.

Upload remains pending in exact Claude session
`239dd18d-bdd7-4828-b503-336e02ba752a`. The source candidate and all
preflight trees remain unusable if that review returns `REQUEST_CHANGES`.

## Exact Inputs

### Accepted scaffold

| Field | Exact value |
|---|---|
| Doctor+Branch scaffold | `701469f91dd0c4c52c5628fb3e9d9e9d0ab0c714` |
| Scaffold tree | `dd63dd560514aef08925cc8b1dbe815bafcceef4` |
| Accepted report | `c57215e243839b224a641251fc9e3bf092f7e116` |
| Embedded Upload reference | product/test slice represented by `cb13da08e82c2502920f735d27617e61f978527a` |

The scaffold's Upload test blob and all four Upload source-document blobs
equal the `cb13da0` blobs. Its `codefire` blob differs because it also contains
the accepted Doctor+Branch composition.

### Pending Upload source

| Field | Exact value |
|---|---|
| Reference-range base | `cb13da08e82c2502920f735d27617e61f978527a` |
| Pending candidate | `446cf4a97b63e46d011a03083ef59354b9d68178` |
| Candidate parent | `a531494be9b3ee5c6eef550521dc6f2d5b3176f7` |
| Candidate tree | `3ac8fa8bc5aa38a1f137fd523db8285980d69692` |
| Latest immutable rejecting report | `b3de0d7`; SHA-256 `3df1db92d6adcef9d318e23182fb9c62519f6eacb091266ced9616626f64cf1f` |
| Current review | absent after provider quota; exact resume required |
| Allowed source patch SHA-256 | `d4159433f0dc53ee9819eadfaac553f85c0c63f7a249be8206a8bc129ccc6476` |
| Allowed source patch size | 6 paths, `+4460/-122` |

The allowed Upload delta contains only:

```text
codefire
docs/development/change-playbook.md
docs/development/command-source-trace.md
docs/development/source-architecture.md
docs/development/source-data-model-catalog.md
tests/test_python_http_upload_accounted_traversal.py
```

Management files, reports, packets, ledgers, issue files, and history from the
candidate branch were excluded.

The Upload test file grows from 10 to 33 unique `test_*` methods. The sorted
name-set SHA-256 values are:

- base: `1b42b665ad3d810fc5acd69dea07883235d3cf879384f84ecc5349494817005f`;
- candidate: `9f96645edd3e82fc4b79eb600c5d3d6b2863671ec5dc16d12c18066f9c1911f6`.

No test name is removed.

### Accepted MR-list source

| Field | Exact value |
|---|---|
| Product-range base | `701469f91dd0c4c52c5628fb3e9d9e9d0ab0c714` |
| Accepted candidate | `b16404676639336f705feffd9af4857d4a1fcc69` |
| Candidate parent | `091dc2a562ffcb94557b974ac1828c80d09e7ccc` |
| Candidate tree | `36185e320338b67e1610d82fcc81cba310c36026` |
| Accepted report | `ee6517a`; SHA-256 `5c952a9a5c162bfbb93522bb644fcba44ce32ddf3628501e6f94308899fe760e` |
| Allowed source patch SHA-256 | `04836fe7beb80825aa09a5b3238f7701c76e83cea0a80da822be105cb436acbd` |
| Allowed source patch size | 6 paths, `+6915/-128` |

The allowed MR-list delta contains `codefire`, the same four source documents,
and new `tests/test_python_http_mr_list_decision_work.py`. Its 49 unique
`test_*` methods have sorted name-set SHA-256
`16943aeae4d3da52b92027c71f8558caeb74486b52eb5bc53fe26e92860fdb21`.

## Mechanical Port

The manager created detached scratch worktree
`/tmp/cfr168-upload-port-preflight-20260731` at exact scaffold `701469f`.
No persistent project worktree or branch was added.

### Stage 1 — pending Upload

The binary allowed-path patch from `cb13da0..446cf4a` applied with
`git apply --3way --index`:

- return code: 0;
- six paths applied cleanly;
- unmerged paths: 0;
- result tree: `e8f282d949b38c71f021d3d83d51172a14f1a5fa`;
- scaffold-relative staged patch SHA-256:
  `5246d3cb899379cd30e9a8cf5701bb1dcb16260119cf6e54db23684afd7fa10a`;
- staged size: `+4460/-122`.

The resulting Upload test blob is exactly candidate blob
`d9f5bf96ac6c0737162aa56bf38fc7565b74f127`.

### Stage 2 — accepted MR-list

After making a temporary detached checkpoint for the Stage 1 tree, the binary
allowed-path patch from `701469f..b164046` applied with
`git apply --3way --index`:

- return code: 0;
- existing paths applied cleanly;
- the new MR-list test file used direct-add fallback as expected;
- unmerged paths: 0;
- result tree: `55d7eeef81fb6f0976b9ea8aedf2df06beb08122`;
- Stage-1-relative staged patch SHA-256:
  `1206c17c83df91f6573ea1af657c4e7b1c85ea341668790696c1351be186567a`.

The resulting MR-list test blob is exactly candidate blob
`0d49841a2e682e661d169274e571e198f3408aa7`.

The final scaffold-relative cumulative patch has SHA-256
`9fe4ca2ab3b0281209f6e456f8088d0bb1d012842de021370d638f41b5846202`
and changes seven paths by `+11465/-340`:

| Path | Added | Deleted |
|---|---:|---:|
| `codefire` | 3584 | 315 |
| `docs/development/change-playbook.md` | 122 | 0 |
| `docs/development/command-source-trace.md` | 75 | 2 |
| `docs/development/source-architecture.md` | 158 | 0 |
| `docs/development/source-data-model-catalog.md` | 45 | 1 |
| `tests/test_python_http_mr_list_decision_work.py` | 4700 | 0 |
| `tests/test_python_http_upload_accounted_traversal.py` | 2781 | 22 |

These counts describe an exact Git comparison. They are not a quality target,
review quota, or authorization to land a large patch without semantic review.

## Gate Evidence

Every timed command used monotonic duration, a finite timeout, a complete log,
and a digest-bound result.

### Scaffold + pending Upload

| Gate | Result | Count | Duration | Result SHA-256 | Log SHA-256 |
|---|---|---:|---:|---|---|
| Doctor + Branch + Upload focused modules | pass | 65 | 99.699760362 s | `b5e98020dd63e9b379499a2a4e535aa660156dca60ac74bcbce3a27d545c837a` | `e88300ea624e9e41d1b8996862f8dbaa5c9fea0984e1e2ca1b9b737a63df4183` |
| Python compile | pass | 5 paths | 0.568286211 s | `cbf9d5294d1feff30f985165718fd2a0bfed7f01a86b4062fe7e16ce6ecea543` | empty-log SHA-256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| Python-pattern discovery | pass | 486 | 281.446580768 s | `840ba63351b4fcce6481ef888f5317d304235406c50606d4967e9de5afc8eb7c` | `5f8047980b3e26b4f8662f119b6fdc2ee258ebd314ffb2717cb2559f80a08e28` |

### Scaffold + pending Upload + accepted MR-list

| Gate | Result | Count | Duration | Result SHA-256 | Log SHA-256 |
|---|---|---:|---:|---|---|
| Doctor + Branch + Upload + MR-list focused modules | pass | 114 | 184.409245916 s | `27373470fd906c5efb82d229626ce19015bd5e59e4f543a10ec96f7f52a0d9db` | `450d4c7154dd1e80d2951e148cba34c25544704dac99598f0042c27f011e9f7e` |
| Python-pattern discovery | pass | 535 | 382.457736053 s | `c7e20e068da98f922c4f113f244a70e6c7aa187da05cfb88bb065cb5f02d1e60` | `3daa637b5523e9d577763ff7f670076a5c920e3aaf9cbf19198ccbdd9ba2e24d` |
| Source-doc checker | pass | 44 modules, 34 commands, 12 object kinds | 0.216177668 s | `88d29f4de8a9f7c04d603632555c8020b3dc83d49456efeb802b62e53c6c5375` | `a3679b77701e02f0697f91c6c53a362f7ab9d372ed700d042c1a73ec329cdb03` |
| Project-doc checker | pass | 534 Markdown files | 0.668074099 s | `b9a8a3ba03f926f4f0e59a38110d83e5fbb52132c171b9713b6327310a317da7` | `58bf12798714cba3806a633aa568de5d7d87d5f321f0c4842ca741a35329a3c8` |
| `git diff --cached --check` | pass | n/a | n/a | n/a | n/a |

No timed result detected a wall-clock discontinuity.

## Consequence

If and only if exact Upload candidate `446cf4a` receives a fresh `ACCEPT`, the
manager can use the two recorded source ranges as semantic-port inputs without
spending another lane rediscovering their path/test census. The actual port
must still:

1. start from then-current authorized composition base;
2. recompute every blob, patch, and test-name set;
3. exclude all candidate management/report history;
4. disposition the four accepted MR-list observations;
5. consume the committed shared-primitives preflight before MR-write;
6. run focused gates after every delta and broad gates on the stable head; and
7. receive a fresh independent exact-composition review.

If Upload returns `REQUEST_CHANGES`, this preflight remains negative
compatibility evidence only. Do not port, integrate, or repair from its
speculative result tree.

