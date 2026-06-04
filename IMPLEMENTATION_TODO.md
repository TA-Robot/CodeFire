# Implementation TODO

## Phase 1

- [x] Implement canonical JSON serialization
- [x] Implement object hash calculation
- [x] Implement immutable object store
- [x] Implement repository init
- [x] Implement branch object and branch head update

## Phase 2

- [x] Implement open materialization
- [x] Implement `.codefire-open`
- [x] Implement open registry
- [x] Implement copied directory detection
- [x] Implement close and close --discard
- [x] Implement local clone

## Phase 3

- [x] Implement codefire.yaml parser
- [x] Implement Markdown Atom extractor
- [x] Implement Markdown ADR/ops Atom extractor
- [x] Implement OpenAPI JSON operation Atom extractor
- [x] Implement OpenAPI YAML operation Atom extractor
- [x] Implement SQL DB schema table Atom extractor
- [x] Implement SQL DB schema column Atom extractor
- [x] Implement SQL DB schema index Atom extractor
- [x] Implement SQL DB schema view Atom extractor
- [x] Implement SQL DB schema trigger Atom extractor
- [x] Implement SQL DB schema function Atom extractor
- [x] Implement SQL DB schema procedure Atom extractor
- [x] Implement SQL DB schema sequence Atom extractor
- [x] Implement SQL DB schema materialized view Atom extractor
- [x] Implement SQL DB schema type Atom extractor
- [x] Implement Python Atom extractor
- [x] Implement JavaScript/TypeScript Atom extractor
- [x] Implement Go Atom extractor
- [x] Implement Java Atom extractor
- [x] Implement C# Atom extractor
- [x] Implement Rust Atom extractor
- [x] Implement Kotlin Atom extractor
- [x] Implement PHP Atom extractor
- [x] Implement Ruby Atom extractor
- [x] Implement Swift Atom extractor
- [x] Implement C/C++ Atom extractor
- [x] Implement pytest Atom extractor
- [x] Implement AtomIndex generation
- [x] Implement duplicate Atom ID detection

## Phase 4

- [x] Implement codefire.links.yaml parser
- [x] Implement TraceGraph
- [x] Implement required link policy
- [x] Implement custom required link policy from codefire.policy.yaml

## Phase 5

- [x] Implement scan
- [x] Implement changed atom detection
- [x] Implement fire propagation
- [x] Implement active fire ledger
- [x] Implement obsolete auto fire
- [x] Implement manual fire

## Phase 6

- [x] Implement extinguish
- [x] Implement resolution basis
- [x] Implement stale resolution check

## Phase 7

- [x] Implement verify command runner
- [x] Implement commit sealer
- [x] Implement consistency certificate
- [x] Implement branch head update after commit

## Phase 8

- [x] Implement common ancestor search
- [x] Implement file-level merge
- [x] Implement merge fire generation
- [x] Implement merge commit parents

## Phase 9

- [x] Implement local show/diff
- [x] Implement file-backed remote upload/list/clone/show/diff
- [x] Implement merge request create/list/review/apply
- [x] Implement sealed commit validation on upload
- [x] Implement server-side verification command execution
- [x] Implement remote permission policy
- [x] Implement remote branch protection policy
- [x] Implement remote token authentication
- [x] Implement remote object GC and retention policy
- [x] Implement generation-aware remote GC retention
- [x] Record remote GC dry-run/deletion audit log
- [x] Implement HTTP remote upload/list/clone protocol
- [x] Implement HTTP merge request create/list/review/apply protocol
- [x] Implement HTTP remote show/diff
- [x] Implement HTTP remote doctor/gc
- [x] Implement optional TLS transport for HTTP remote
- [x] Restrict server-side verification cwd/env/timeout
- [x] Implement hashed remote token storage
- [x] Implement HMAC commit signatures and remote signature policy
- [x] Implement commit signature key rotation policy
- [x] Implement HMAC remote request signatures
- [x] Implement remote request signature nonce replay cache
- [x] Implement install script and demo script
- [x] Implement bash/zsh shell completion generation and install
