# ADR-017: Make a remote project one contained, recoverable publication domain

- Status: Accepted and implemented; independent source/install/dogfood close review passed
- Date: 2026-07-19
- Decision owners: Remote mutation / Store / Recovery
- Related: [CFR-165](../docs/development/code-review-issues/cfr-165.md), [CFR-174](../docs/development/code-review-issues/cfr-174.md), [CFR-175](../docs/development/code-review-issues/cfr-175.md), RVC-026, RVC-030, RVC-044, RVC-050, RVC-056, ADR-010, ADR-016

## Context

File and HTTP remote mutations currently check idempotency before their resource lock and publish the result after
mutable state. Two callers can observe the same missing key, both mutate, and overwrite one another's receipt. A
process can stop after branch or merge-request publication but before the receipt. Review reads the MR before its
lock; apply spans generation, objects, branch, MR, and receipt without owning one publication authority. The
specialized `FileLock` uses create-new/unlink pathnames and is crash-stale and ABA-prone.

Python GC uses a separate lock and can sweep from an obsolete root snapshot while a Rust or Python writer publishes.
Its unlink loop and JSONL audit have no recoverable plan. The first ADR draft covered only keyed Rust mutations and
left Python writers, unkeyed multi-write operations, lock-free readers, cross-project roots, and GC crash recovery
outside the proof. Independent review correctly blocked that design.

The same characterization found two prerequisite defects. Apply uses the current source branch head rather than the
immutable head approved by the MR. File-remote components and MR IDs are not all normalized into contained authority
paths. These are CFR-174 and CFR-175 and belong to this root because an unbound subject or path makes any lock and
transaction proof false.

The CFR-164 installed binary reproduced the same-key race deterministically at
`/tmp/codefire-cfr165-counterexample._aq99nnn/results.json`: two different review payloads using one key both returned
success, only one effect/receipt survived, and replay of the other successful request became a conflict.

## Decision

### 1. A pinned `RemoteProject` owns every authority path

Rust introduces `remote/project.rs`. A `RemoteProject` is a pinned no-follow capability over one canonical remote
project and is the only object allowed to derive mutable or transaction paths. It owns:

- a version-1 `codefire_remote_project` descriptor with a stable project ID, protocol generation, canonical org/app
  components, and creation provenance;
- pinned identities for the project root and authority directories;
- bounded, regular-file-only, no-follow reads and atomic capability-relative writes;
- typed `RemoteAuthority` values such as `Branch`, `MergeRequest`, `Generation`, `PendingTransaction`,
  `IdempotencyReceipt`, `GcAudit`, and `RequestNonceCache`.

Org, app, branch, MR ID, and every external path component pass one strict Rust/Python golden contract after percent
decoding. Empty, current/parent, absolute/prefix, separator, backslash, and control components are rejected before
signature nonce publication, layout creation, lock ownership metadata, or any other write. Arbitrary `Path` values
never enter the publication reducer.

New-project creation is serialized by a persistent kernel-backed bootstrap lock under the canonical projects
namespace. The descriptor and complete base layout are staged, synced, and atomically published before the project
gate is usable. Legacy layout adoption requires administrative quiescence because a new binary cannot prove that an
old create-new-lock binary is absent. The migration records that limitation; a protocol-generation marker is not
treated as proof that a legacy process does not exist.

### 2. One project reader/writer gate defines visibility

Each remote project has one persistent advisory reader/writer gate inode.

- Every Rust and Python file-server mutation and GC takes the gate exclusively.
- Every Rust and Python authority read that combines or dereferences branch/MR/object state takes it shared.
- HTTP clients do not lock local storage; the serving Rust/Python handler takes the gate.
- After a shared lock is acquired, a reader checks the fixed pending authority before its first domain read. A pending
  operation produces a typed recovery response instead of a partial view.
- After an exclusive lock is acquired, a mutator reloads and validates every precondition from the pinned project.

The exclusive gate deliberately serializes mutations within one local project. Read-only commands remain concurrent,
and no compliant reader can observe an apply between branch and MR writes. A future finer-grained implementation may
replace this only if it preserves the same pending-authority and visibility contract.

Cross-project request-merge acquires all involved project gates in stable project-ID order, using shared mode for a
source-only project and exclusive mode for the target. Equal source/target identities collapse to one exclusive gate.
One absolute deadline covers the full ordered acquisition; timeout is not restarted per lock.

Rust uses `fs2` advisory locks on persistent no-follow, exactly-one-link regular files. Unix Python uses
`fcntl.flock` on the same inode and verifies `lstat`/`fstat` device, inode, mode, and link count before and after
acquisition. Python file-server mutation and GC fail closed on platforms where the same shared/exclusive primitive and
no-follow identity checks cannot be implemented. Unlock never unlinks the inode. Owner metadata is a non-authoritative
atomic sidecar, bounded to 16 KiB; malformed, absent, symlinked, non-regular, FIFO, or oversized metadata is ignored.
For shared holders it describes only the last observer, never the complete ownership set.

The signature nonce-cache lock migrates to the same persistent kernel-backed lock primitive. Authentication cannot
remain blocked by the old crash-stale `FileLock` after the publication locks are fixed.

### 3. Every mutation has one internal durable journal

All upload, request-merge, review, apply, and GC operations use an internal transaction whether or not the caller
supplies an idempotency key. The project has one O(1)-discoverable `pending_transaction.json`, valid only while its
exclusive gate protects it. A transaction-specific directory holds bounded write postimages or chunk manifests.

The normalized request binds authenticated actor, operation, stable project identities, semantic inputs, and any
caller key. Signature bytes, nonce, lock options, transport locator, and rendering fields are not semantic request
identity. User keys are bounded values stored only inside the record; v2 receipt filenames use a domain-separated
hash of actor, operation, and key, preventing cross-actor key squatting and unbounded/raw filenames.

The reducer proceeds as follows:

1. Validate or recover the fixed pending authority under the exclusive gate. A different request cannot replace it.
2. Stage every bounded postimage/manifest, sync each file and its directory, and validate their hashes.
3. Atomically publish and sync a version-1 `remote_publication_transaction` marker before the first mutable-authority
   write. It binds operation ID, request hash, actor/key scope, project IDs, exact preimages, ordered typed writes,
   object roots, immutable result, manifest hashes, and phase/progress.
4. For each authority write, accept only its exact preimage or its already-published exact postimage. Apply, sync, and
   then advance progress. A write-before-progress interruption is the only accepted local gap.
5. Validate all final postimages and sealed object roots.
6. If keyed, atomically publish a compact version-2 completed idempotency receipt containing request/result/effect
   hashes and the original immutable result.
7. Mark the transaction complete, remove only matching transaction-owned staging, and release the fixed pending
   authority last.

At retry, the same normalized request rolls forward any recorded pre/post mixture and returns the fixed result. A
different request, malformed/future marker, resource mismatch, unexpected current image, missing/corrupt manifest, or
foreign staging is preserved and rejected before recovery writes. A keyed retry after pending release replays the
receipt. An unkeyed retry can recover while its pending owner remains, but after successful pending release an equal
request is a new operation. A crash after release but before the caller receives the response may therefore cause an
unkeyed repeated review; this unavoidable acknowledgement ambiguity is explicit, while internal branch/MR authority
is always consistent.

Error-side stale detection is read-only. Review/apply return a conflict without opportunistically writing
`status=stale`; derived list status may still report stale. Thus an error is not an unjournaled mutation.

Version-1 remote idempotency records remain strict completed receipts only. They are never upgraded in place or used
for prepared recovery. Replay is available only when the historical semantic payload can still be reconstructed;
legacy file/HTTP upload records intentionally retain their former transport-specific limitation.

### 4. Approval binds a versioned immutable MR subject

Request-merge constructs a version-1 `codefire_merge_request_subject` containing MR ID, stable source/target project
IDs, canonical branch identities, exact `source_head`, exact `target_head_at_request`, and a canonical source object-set
digest. The MR stores the subject and recomputable digest. Each review stores the exact subject digest it approved or
rejected. Apply requires the current subject to recompute exactly and the effective approving review to bind that same
digest. A legacy approved MR without this subject is not silently trusted; it requires a new review that materializes
and approves the current versioned subject.

URLs remain display/routing locators and are not authority identities. Apply never reads the current source branch as
the approved head.

For cross-project MRs, request-merge copies and validates the complete source graph into the target object store while
holding the ordered source/target gates and before publishing a recoverable MR transaction. Content-addressed escrow
objects may be written before the pending marker because they are non-authoritative and safe to orphan. The marker is
published only after the complete graph exists and validates in the target. The target MR and pending transaction then
root that graph locally; apply reads only target-local escrow. No cross-project GC lease remains after request-merge.

### 5. Upload and apply publish fixed write sets

File and HTTP adapters authenticate, normalize, and stage their incoming graph, then call shared operation-specific
planners in `remote/operations.rs`. The canonical upload object set is sorted by object ID, rejects duplicate/conflicting
records, and binds one object-set digest on both transports.

Generation is planned once under the exclusive gate and is an ordinary journaled authority postimage. Upload writes
generation then branch. Apply writes generation, target branch, then MR, all inside one recorded reducer; shared readers
cannot interleave. Apply's result head, branch head, MR `applied_head`, and reviewed subject head must be identical.

Object publication is bounded and content-addressed. Large object sets use hashed, bounded manifest chunks rather than
embedding all records in the pending JSON. Mutable retention metadata is not written into content-addressed object
records; it belongs to a separate versioned retention index.

### 6. GC is a durable transaction, not a best-effort unlink loop

GC uses the same fixed pending authority and exclusive gate. Its staged plan binds:

- operation/plan ID, actor, policy digest, generation, and dry-run mode;
- typed roots with per-root reason, including branch, MR source/target/applied, and pending object roots;
- a canonical root fingerprint and complete object-inventory fingerprint;
- bounded chunk manifests of candidates, each candidate's object ID, record hash, pinned file identity, and retention
  decision/reason;
- ordered progress and the immutable audit result.

Unknown root, reference, transaction, object, or policy versions block collection. Immediately before the first delete,
GC recomputes policy, roots, inventory, and plan fingerprints. The persisted planning timestamp must be an integer,
nonnegative, and no later than the trusted resume wall clock before age retention uses it. Each delete reopens the
exact regular file no-follow, checks identity and record hash, renames it no-replace to a plan-owned quarantine,
reopens/rechecks that quarantine, unlinks relative to the pinned object directory, syncs, then records progress. A
quarantine or delete-before-progress gap accepts only exact plan-owned evidence or exact absence. Partial sweep retry
resumes the same plan while later mutations remain blocked by the pending authority.

The authoritative audit is an atomic exact-or-create `gc/<plan-id>.json` record. JSONL is a derived view and cannot be
the exactly-once authority. Audit publication is journaled before pending release. Dry-run and live GC share one planner;
dry-run publishes no deletion but may publish its audit through the same transaction.

Rust readers/mutators validate and diagnose a pending Python-created GC transaction. The owning Python GC command can
resume it. A later Rust GC executor may consume the same schema, but this ADR does not claim a new public Rust GC
command.

### 7. File and HTTP share reducers and fault vocabulary

`remote/project.rs` owns containment and I/O, `remote/locks.rs` the stable gate, `remote/publication.rs` the generic
journal, and `remote/operations.rs` the four operation-specific planners. `remote.rs` and `http.rs` become adapters;
`main.rs` loses the old `FileLock` and gains no remote domain logic. Python mirrors the persisted protocol and golden
normalization/subject/result vectors; Python file-server writers participate in the same gate/journal or fail closed.
Python new object construction uses the current 24-hex public ID while authenticated transfer/read retains explicit
12/24-hex compatibility, matching the Rust store. The cross-runtime gate includes actual Rust-to-Python and
Python-to-Rust receipt replay with byte-for-byte branch/generation nonmutation.

Test-only failpoints are named by operation and durable phase: gate acquired, escrow/object staging, plan staged,
pending published, each authority/delete write before/after, progress publication, postimage validation, receipt/audit
publication, complete, and pending release. Barrier hooks never run while holding their registration mutex and are not
public runtime features.

## Consequences

- Same-key/same-payload concurrency has one effect/result; different payload conflicts before domain writes.
- Keyless crashes cannot leave permanent branch/MR divergence, although response-acknowledgement deduplication still
  requires a caller key.
- Lock-compliant readers never observe a partial multi-authority publication.
- Process exit releases every lock without stale-path breaking or unlink ABA.
- Apply cannot widen an approval to a newer source head or a changed MR subject.
- Cross-project approved objects become target-local escrow and remain target GC roots.
- GC survives every delete/audit interruption and cannot race a cooperating Rust/Python writer.
- Mutation serialization reduces same-project write parallelism. This is accepted for the local file server; bounded
  read concurrency remains, and later optimization must preserve the protocol rather than bypass it.
- Python file-server remote mutation/GC is unavailable on a platform where its standard library cannot prove the same
  advisory-lock and no-follow identity contract. HTTP clients may still use a compliant Rust server.
- This is local-filesystem coordination, not distributed consensus or a network-filesystem lease.
- The local containment model coordinates CodeFire writers through the project gate. It detects and preserves
  observed path/inode swaps but does not claim isolation from an actively racing, non-cooperating process with the
  same filesystem credentials; pathname removal cannot provide that stronger boundary by itself.

## Alternatives considered

- Add an in-lock idempotency recheck only: rejected; it leaves crash gaps, partial apply, stale locks, and GC races.
- Keep keyed transactions only: rejected; unkeyed multi-record operations can remain permanently divergent.
- Keep normal reads lock-free and scan transaction directories: rejected; absence-check/read races still expose partial
  authority and bounded discovery is unclear.
- Fine-grained branch/MR locks immediately: rejected for this revision because multiple pending indexes and reader
  visibility complicate the proof. One project RW gate is the correctness baseline.
- Break stale path locks using PID/time: rejected because neither proves ownership and unlink permits ABA.
- Apply current source when it remains a fast-forward: rejected because ancestry does not prove review.
- Cross-project root leases: rejected in favor of target-local immutable escrow.
- Use JSONL as GC audit authority: rejected because append and sweep cannot be recovered atomically.
- Treat equal unkeyed payloads as implicit idempotency: rejected because identical intentional reviews are distinct.

## Verification

- Strict component golden matrix and no-write containment tests across Rust/Python/file/HTTP.
- Project bootstrap, stable ID, no-follow/root/parent identity, legacy-quiescence diagnostics, process-kill reacquire,
  persistent inode, sidecar bound, symlink/hard-link/inode-swap, and one absolute timeout.
- Shared-reader/exclusive-writer barriers proving branch/MR/object readers block or receive typed pending authority.
- Upload/merge/review/apply barrier matrix for file/file, HTTP/HTTP, and file/HTTP with same/different keyed payloads and
  unkeyed crash recovery.
- Every journal write-before-progress and phase boundary: foreign request raw nonmutation, exact owner roll-forward,
  stable immutable result, and no phase rewind.
- MR subject tamper, legacy re-review, source advance after approval, target move, cross-project escrow, and target GC
  survival.
- GC root/inventory fingerprint change, unknown root/reference, partial first/middle/last delete, post-delete/pre-audit,
  exact audit creation, JSONL derivation, and retry.
- v1 receipt readability where reconstructable and explicit legacy file/HTTP limitation fixtures.
- Full Rust/Python gates, fresh install, installed counterexample reversal, real read-only probes, canonical dogfood, and
  independent High review before CFR-165/CFR-174/CFR-175 close.

Revision 1 was independently blocked because it omitted Python writers, unkeyed recovery, reader pending visibility,
approval-subject binding, cross-project root ownership, and GC crash recovery. Revision 2 added those invariants and
received `ACCEPT_FOR_IMPLEMENTATION` on 2026-07-19. Source Close still requires the implementation and verification
matrix above; accepting the ADR is not closure evidence by itself.
