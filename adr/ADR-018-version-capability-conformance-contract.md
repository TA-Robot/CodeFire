# ADR-018: Version, Capability, and Conformance Contract

Status: Implemented

Date: 2026-07-19

Owners: CLI, Store, and Compatibility Maintainers

Related: CFR-166, CFB-134, RVC-006/007/013/017/020/022/032/033/049/053/055/057/059

## Context

CodeFire already rejects unsupported commit/root versions, but the decision was distributed. Remote
branch and merge-request readers could consume semantic fields before checking a version, top-level
object extensions were dropped by transfer, capabilities described only command-level JSON/metrics,
and the Rust default Doctor sent `cf://` and HTTP targets into local repository discovery. Rust and
Python had no checksum corpus or independent reader proving the same allow/deny result.

An additive field cannot be assumed safe. A future field may be a required blocker or an object
reference. Conversely, rejecting every unknown field prevents noncritical metadata forwarding. The
runtime therefore needs an explicit criticality and preservation rule rather than relying on serde or
dict behavior.

## Decision

### One support manifest

Rust owns `codefire.support.v1`, advertised inside `codefire.capabilities.v1`. Each registered format
states its current/readable versions and whether a missing version is an explicit legacy case or a
denial. Unknown versions deny before semantic fields are consumed. The manifest also publishes the
canonical JSON profile, public-ID widths, semantic validation profile, runtime roles, and checked
schema IDs.

The command registry additionally declares target classes, output schema, and pagination mode. Help,
parser, capability, and schema tests must agree. Python remains the reference/fallback runtime; it may
read/write the shared format profile but is not command-manifest authority.

### Extension rule

Object payload identity remains `SHA-256(type || NUL || canonical(payload))`. Non-identity top-level
metadata is retained in `ObjectRecordDocument`. Standard extension descriptors contain namespace,
name, version, criticality, optional references, and optional payload.

- Unknown critical extensions deny before use.
- Unknown noncritical extensions are preserved exactly as JSON values during copy/bundle/write.
- Declared `CF-*` references join reachability, transfer, Doctor, and GC traversal.
- Immutable existing-record reuse requires exact extension-map equality; it never silently merges or
  drops metadata.
- Historical top-level transport metadata remains noncritical and preserved according to its existing
  owner rule.

### Checked schemas and corpus

The public base family is checked in under `schemas/public/`: command result, capabilities, remote
Doctor, page, and page cursor. `schemas/conformance/manifest-v1.json` binds every selected vector by
SHA-256 and names writer/reader expectations. Rust, Python fallback, and a stdlib-only independent
consumer must produce the same accept/deny result. Canonical JSON vectors also bind canonical-byte
SHA-256, so equal parsing alone is insufficient.

The corpus is deliberately small and representative, not a claim that every product payload already
has a complete public schema. Adding a public/persisted family requires adding its version decision,
schema where public, negative vector, and all applicable runtime readers in the same change.

### Snapshot cursor

`codefire.page_cursor.v1` is a bounded canonical-hex payload containing collection identity,
complete ordered-result snapshot digest, offset, and limit. `codefire.page.v1` exposes the same
snapshot and returned/total counts. A cursor from a different collection, malformed/future cursor,
changed limit, or changed result snapshot denies and instructs the caller to restart. Unpaged calls
retain the historical full-result behavior. Remote `list` and `request-list` are the first consumers.

### Remote Doctor

Doctor classifies its target before local discovery. File remotes enter their shared project gate;
HTTP(S) uses `GET /v1/projects/{org}/{app}/doctor`. Both return
`codefire.remote_doctor.v1` with transport/profile, project/layout state, bounded issue samples,
record/reference counters, skipped quick checks, and outcome. Missing projects are reports rather
than I/O exceptions. Future branch/MR versions, malformed objects, missing references, and invalid
sealed roots are typed diagnostics. Rust accepts the same report from the Python HTTP fallback.

## Dependency Decision

`codefire-cli` now declares `serde` with derive support directly. Serde was already a transitive
workspace dependency; the direct dependency adds no runtime service or network surface. It is used
only to serialize/deserialize the typed cross-runtime remote Doctor report. A manual `Value` decoder
was rejected because it would duplicate required/optional field logic and weaken compile-time shape
review. Remove the direct dependency if the report type moves into an existing serde-owning shared
crate or if a generated schema decoder replaces it; `serde_json` alone remains insufficient for the
derive.

## Consequences

- Capabilities become machine-actionable and falsifiable rather than a flat feature list.
- Remote readers fail earlier and more consistently on future records.
- Noncritical extension transport no longer loses data or hidden references.
- Cursor replay never mixes two collection snapshots silently.
- Rust default and Python HTTP fallback share one remote Doctor payload.
- Complete schema coverage for every command payload remains incremental; the support manifest must
  not advertise a schema that is not checked in.

## Rejected Alternatives

- Accept every unknown field: can discard a future blocker or reference.
- Reject every unknown field: prevents safe noncritical forwarding and gradual interoperability.
- Keep runtime-specific capability lists: preserves the drift that caused CFB-134.
- Offset-only cursors: silently duplicate/omit records after concurrent mutation.
- Implement remote Doctor as a temporary local clone: adds unnecessary writes and changes the
  diagnostic subject.

## Verification

- legacy/current/future branch and object-extension decision vectors;
- noncritical extension preservation and extension-only reference traversal;
- capability registry/help/parser/schema agreement;
- cursor concatenation, cross-collection/limit/mutation/malformed rejection;
- file remote missing/healthy/corrupt/future-version reports;
- Rust server HTTP report and Rust client consumption of Python HTTP report;
- full Rust/Python gates, fresh Install, installed counterexample reversal, real read-only probes,
  canonical Algorithm Dogfood, independent close review, and ledger synchronization before Fixed.
