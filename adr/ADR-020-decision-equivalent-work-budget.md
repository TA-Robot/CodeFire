# ADR-020: Decision-Equivalent Work Budget and Graph Evaluation

Status: Accepted

Date: 2026-07-19

Owners: Core, CLI, Store, Remote, and Metrics Maintainers

Related: CFR-168, RVC-014, RVC-015, RVC-029, RVC-042, RVC-047, RVC-048, RVC-058

## Context

CodeFire bounds many individual files and output samples, but a bounded response does not imply
bounded internal work. Current source still has four distinct amplification paths:

- scan calls a full Trace Link scan for every changed Atom;
- targeted ancestry builds a complete ancestor map, while signature history uses recursive calls;
- branch/storage/Doctor paths recreate sealed-validation state instead of sharing one command-scoped
  validated subject;
- HTTP bundle routes materialize every reachable object as JSON and serialize the complete response
  before the client-side 16 MiB limit can reject it.

Optimizing these paths without an equivalence contract could hide corrupt objects, change graph
tie-breaking, or report a partial traversal as complete. Wall-clock thresholds alone cannot
distinguish legitimate memoization from skipped validation.

## Proposed Decision

### One work vocabulary

Introduce advertised capability profile `codefire.decision-work.v1` with privacy-safe aggregate
counters:

- files and bytes read/hashed;
- Atoms, rules, and Trace Links indexed or evaluated;
- graph nodes and edges discovered, read, validated, revisited, and served from command memo;
- object-record bytes read and response bytes encoded;
- maximum queue depth and terminal reason (`complete`, `early_match`, `budget_exceeded`,
  `corrupt_subject`, `subject_changed`, `deadline_exceeded`). Business-policy rejection that
  completes its bounded decision work is not mislabeled as corruption.

Counters are checked `u64` values. A counter is charged before the corresponding allocation, queue
insertion, parse, validation, or output append. Overflow is a budget failure rather than saturation
or wraparound. Counters describe correctness work and remain outside the authoritative domain
result. Enabling or disabling metrics cannot change the traversal, result, diagnostic, ordering, or
budget enforcement.

### Budget profiles

One mutable `DecisionWorkLedger` is created for the complete command or HTTP request and passed by
mutable reference through validation, query, cache reuse, encoding, staging, and every branch/head.
Nested evaluators cannot create fresh ledgers or reset phase counters. Each phase records a
monotonic delta, while limit enforcement uses command/request cumulative totals; this prevents
per-head, per-root, or validate-then-encode budget multiplication.

Work that must execute after a mutation visibility point uses a ledger-bound
`DecisionWorkPermit`. `reserve` removes the declared upper bound from available headroom without
recording work, `consume_reserved` records only actual work, and cancellation returns unused
headroom. `seal_prepublication` then rejects ordinary `apply`; only permit consumption, concurrent
memory release, and the terminal transition remain legal. A permit is bound to one non-cloneable
ledger and cannot be consumed by another request.

The ledger distinguishes unique node discovery, record read, semantic validation, edge occurrence,
frontier insertion attempt, revisit, memo hit/miss, current logical depth, and maximum frontier.
An occurrence is charged even when its target is already visited; unique-node read/validation is
charged only on the first non-memo visit in the bound session. Exact-limit work succeeds. The first
attempt that would exceed a limit fails with `observed=limit+1` before its effect. Phase counters are
views over the same cumulative charges, not separately enforceable allowances.

Logical depth is the number of graph edges from a traversal root; a root has depth zero. Logical
retained bytes are the compact canonical JSON bytes of documents and output buffers owned
concurrently, plus `256 + utf8(node_id).len()` bytes for each retained traversal or frontier frame.
This portable accounting unit is intentionally independent of allocator and runtime object layout.

Every ledger has an explicit immutable profile. Candidate initial local limits follow the
existing worktree envelope: at most 100,000 graph nodes, 400,000 discovered edges, 100,000 logical
depth, 1 GiB of aggregate record/file bytes, 1 GiB of concurrently retained logical bytes, and
1 GiB of concurrently temporary logical bytes. They
are not adopted until the required 1x/3x/10x
and boundary evidence passes. Per-record 64 MiB limits remain stricter.

The HTTP profile freezes 64 KiB request/response header bytes, 16 MiB request body bytes, 16 MiB
canonical encoded response bytes, 32 MiB logical retained serialized bytes, 16 MiB temporary bytes,
100,000 object records, a 30-second request deadline, 32 active global connections, and zero queued
connections. Rust/Python use compact UTF-8 canonical JSON (sorted object keys, no insignificant
whitespace, no ASCII escaping) for every byte-limit decision. Negative, malformed, duplicate, or
overflowing `Content-Length` fails before body read. A route must stop before publishing response
headers, nonce/layout/temp creation, or mutation effects. HTTP overflow returns a typed 413
resource-budget diagnostic with observed/limit/phase and bounded partition or retry guidance; it
never sends a syntactically successful partial bundle.

Logical retained memory is charged by canonical encoded bytes owned concurrently by the route,
including incoming records, validation frames, collected records, and the response buffer. It is a
portable deterministic ceiling rather than a claim about allocator overhead. Values retained beyond
a parse or validation step transfer from temporary accounting to this exact canonical unit before the
next effect. Temporary bytes cover staged files and bounded raw-read, parser, validation-clone, and
serializer scratch that is released within the phase; on-disk/raw input scratch uses its physical byte
length, while a transient clone of an already parsed value uses its compact canonical byte length.
Local branch-reference profile v1 reserves `2 * P` temporary bytes before reading and parsing a
physical `P`-byte document (one `P` raw buffer plus one `P` parser-scratch unit), then acquires the
compact canonical JSON size as retained bytes before releasing that temporary reservation.
Directory membership retains `256 + encoded_name.len()` bytes per entry until final revalidation;
the iterator itself reserves a 4 KiB temporary scratch unit before each step.
This pre-effect scratch reservation may independently exhaust the temporary limit, but it may not be
misreported as retained logical bytes or use allocator/runtime `size_of` values. A tiny upload must
validate through an incoming-first/existing-store overlay or a
closure-only bounded stage; copying the complete existing object store into a temp directory is
forbidden and does not become acceptable merely because the temp limit eventually stops it.

The generated support manifest and command capabilities advertise the profile ID, every enforced
dimension, exact limit, applicable command/phase, continuation support, and typed exit code 60
(`resource_budget_exceeded`). Budgets are compatibility/runtime contract values, not
repository-controlled relaxation knobs. Increasing a limit is additive capacity. Decreasing it,
changing charge semantics, or adding a newly required dimension requires a new capability profile,
schema/conformance vectors, and scale evidence. A runtime that cannot enforce the selected profile
must reject the command before traversal or mutation rather than silently using different limits.

### Evaluators and memoization

Trace evaluation compiles a deterministic adjacency/count index once and preserves current
direct-only, bidirectional Fire semantics. CFR-168 does not adopt transitive impact propagation.
Cycle/closure experiments remain explicit future policy work.

Commit/object traversal becomes iterative and accepts query-specific termination:

- `is_ancestor` may stop at the first match only inside a command session whose complete reachable
  head graph has already passed sealed/corruption validation under the same pinned Store subject.
  A cold query completes validation first, so an early matching parent cannot hide a corrupt sibling;
- signature history visits each reachable parent once without call-stack growth;
- common-ancestor selection retains the existing distance, maximum-side-distance, generation, and
  public-ID tie-break and may not early-stop unless the same result is proven;
- reachable object transfer validates each record once and charges its encoded size before keeping
  or writing it.

Store `validate_sealed_commit_inner` and Python `validate_sealed_commit` are replaced by explicit
iterative enter/exit frames before the depth profile is claimed. Scheduling and error precedence are
compatibility behavior: sealed validation and signature history use depth-first parent-array order
(reverse-push on a LIFO stack); object traversal preserves current LIFO reference scheduling and
sorts successful bundle records by public ID; ancestry remains FIFO parent-array order; common
ancestor retains its complete tie-break. Trace adjacency retains source Link order, duplicate links,
and the current single self-loop entry. Golden fixtures freeze both successful order and the first
reported corruption/budget boundary.

A command-scoped validation session is keyed by pinned objects-root/subdirectory identities, public
object ID, full object-record document digest and size (including every extension and declared
reference), validator support-manifest digest, and operation snapshot. The snapshot binds the
captured branch/head set, remote/signature policy and keyset digests when applicable, and active/open
snapshot identity for local workflows. It distinguishes
`fully_validated_reachable` from merely `parsed` or `path_matched`; only the first state permits a
query early-stop that would otherwise skip corruption detection. It may reuse a parsed validated
record only as a retained immutable value inside that command subject: memo hits use the initially
parsed bytes and never re-read an ambient mutable path. Before command success, and immediately
before mutation preflight completes, the session performs one bounded final revalidation of every
memoized document by re-reading its complete bytes and checking the recorded digest and size.
Snapshot-revalidation records and bytes are separate ledger charges. Any drift fails with a typed
subject-change outcome; the next cold command reads the new value. Thus semantic validation remains
at most one pass per unique document plus memo lookups, while I/O is explicitly bounded to the
initial read and one final revalidation. Same-path, same-inode, same-size, same-mtime, and
extension-only tampering are explicit drift/corruption fixtures. Persistent mtime/size-only caches
are not adopted: they could hide same-size tampering and create a second authority.

### Failure contract

Budget exhaustion is distinct from corruption and business-policy failure. It is a typed blocking
outcome and process exit 60 containing profile, dimension, observed value, limit, phase, applicable
command, and whether a continuation is supported. The complete diagnostic survives HTTP server
encoding, client decoding, `RemoteDiagnostic`, automation envelope, and text rendering without
being reduced to an error string. A command that requires a complete graph cannot return `ok=true`,
`healthy`, `passed`, or a mutation plan after exhaustion. Read-only paged surfaces may return
continuation only when the token binds the same validated snapshot and cumulative budget semantics.

For mutating HTTP routes, request parsing, authentication without nonce publication, complete graph
and response planning, budget validation, and canonical success/error body construction all precede
the publication transaction visibility point. Nonce publication joins that mutation transaction or
occurs only after budget preflight. A response-budget failure therefore cannot leave nonce, layout,
temp, branch, MR, receipt, or other authority effects. Transport failure after a committed visibility
point is an incomplete-framing/retry condition and is never represented as a partial successful graph.

The 30-second HTTP deadline is an availability control, not a deterministic ledger dimension.
Cancellation is checked throughout parsing, planning, validation, encoding, and once immediately
before response or mutation publication. Once a mutation visibility transaction begins, deadline
cancellation is disabled and the durable reducer must complete or recover; the server cannot report
a no-effect resource error after authority became visible. Deadline failures still use bounded typed
transport diagnostics, but do not claim deterministic decision-work equivalence.
Iterative Store validation and final retained-document revalidation accept a caller-owned monotonic
cancellation poll; HTTP bundle/upload also poll during object collection, upload preflight, signature
history, ancestry, canonical digest serialization, overlay preparation, and ordered-map construction.
HTTP merge-request create/list/review/apply use the request ledger for sealed validation, subject
graph/digest work, escrow traversal, ancestry, and signature history. Request-derived lock waits are
capped by the same absolute remaining deadline, including ordered multi-project acquisition.
Collected object documents transfer `2 * physical` parse scratch from Temporary to compact-canonical
Retained ownership and return an explicit release/publication owner. Escrow publication consumes that
owner under the same ledger; successful HTTP response completion releases all remaining route-owned
concurrent memory before the terminal transition. Overlay request clones reserve their aggregate
compact size before cloning and transfer it to Store-owned Retained bytes.

Rust default semantics are authoritative. The Python fallback uses the same profile, charge points,
ordering, diagnostic fields, and limits; its current missing generation tie-break for common
ancestors is an intentional CFR-168 compatibility correction, frozen by a cross-runtime conformance
vector. Python response reads/encoding, required-link evaluation, sealed/signature traversal, object
collection, and common-ancestor tie-breaking are part of this parity gate. A runtime-specific
fallback to unbounded work or weaker ancestry selection is unsupported.

Existing branch and request-list surfaces keep their snapshot-bound pagination. Complete sealed
object bundles cannot expose a continuation without changing graph-transfer atomicity, so v1 reports
`continuation_supported=false` and fails before response publication when the cumulative graph or
response budget is exceeded. This is the explicit disposition of the stream/page requirement:
bounded streaming is used internally, safe existing collections page externally, and atomic graph
surfaces fail rather than return a partial graph. A future resumable bundle protocol requires its own
versioned snapshot/object-set token and is not left as an untracked CFR-168 residual.

## Required Proof Before Adoption

- reference/indexed required-link and Trace adjacency results have identical order and diagnostics;
- iterative ancestry/signature/object traversal matches existing golden results on linear, diamond,
  wide merge, repeated-parent, missing, and corrupt graphs;
- metrics-disabled and metrics-enabled results are byte-equivalent after removing metrics;
- 1x/3x/10x scale fixtures remain inside the declared counter order, and every limit-minus-one,
  exact-limit, and limit-plus-one boundary is tested;
- HTTP overflow performs no partial response/mutation and does not allocate an unbounded encoded
  body;
- remote typed budget details survive server/client transport, and the same local/HTTP failure has
  the same kind, exit code, profile, dimension, observed, limit, phase, and continuation fields;
- a head with an immediately matching ancestor plus a corrupt sibling still reports corruption on a
  cold query; only reuse of a fully validated command session permits early termination;
- Rust/Python common-ancestor, required-link, sealed/signature, object, and HTTP golden vectors agree;
- one cumulative ledger proves that N heads/phases cannot each consume the full command budget;
- a tiny HTTP upload with many unreachable existing objects performs work proportional to the
  candidate closure and never copies the complete store;
- full Rust/Python compatibility, Install, installed reversal, and canonical Dogfood gates pass.

## Consequences

- Resource safety becomes a deterministic correctness contract rather than an elapsed-time guess.
- Deep signature history no longer depends on the process call stack.
- Shared-DAG work can be reduced without trusting a persistent cache.
- Existing repositories beyond a supported envelope fail explicitly instead of consuming
  unbounded resources; capacity changes must be reviewed as compatibility changes.
- Streaming/paging transports may be added behind the same snapshot and cumulative-budget contract,
  but CFR-168 does not require a new external service or graph database.

## Rejected Alternatives

- Elapsed-time-only gates: noisy and cannot show whether validation was skipped.
- More output truncation: does not bound traversal, parsing, hashing, or serialization.
- Recursive traversal plus a depth check: still duplicates reader/corruption semantics and retains
  stack dependence.
- Persistent path/mtime/size cache: insufficient integrity key and a new stale authority.
- Partial-success bundle on budget overflow: consumers cannot distinguish a complete sealed graph.
