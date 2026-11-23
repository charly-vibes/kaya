## ADDED Requirements

### Requirement: Document IR materialization

The compiler SHALL materialize a deterministic document IR between the
ledger-model state and every rendered projection, and SHALL compile every
rendered projection from it. Each record SHALL carry a stable id, a content
hash, and its event ids; records SHALL cover generated documents (family and
kind), authored blocks, and composed records; edges SHALL link a record to
the event ids, cited ids, source artifacts, and contained records it depends
on.

#### Scenario: Containment edges carry transitive impact

- **WHEN** a composed record — a publication page or collection — holds a
  document record
- **THEN** the composed record holds an edge to that document record, so a
  change reaching the document reaches the composed record's impact set

#### Scenario: Identical state yields identical IR

- **WHEN** the document IR is materialized twice from identical ledger,
  graph, and document-source state
- **THEN** both materializations are identical — same record ids, hashes,
  edges, and validation state

#### Scenario: Provenance complete per record

- **WHEN** a generated record is derived from ledger events
- **THEN** the record names those event ids, and the emitted source map for
  its rendered block is exactly that provenance

#### Scenario: Authored block referenced, not copied

- **WHEN** an authored block is modeled in the IR
- **THEN** its record names the document and the block, carries the region
  hash and its cited ids, and stores none of the authored bytes

#### Scenario: IR independent of build caches

- **WHEN** a build cache, stale sidecar, or rendered copy outside the
  document sources is deleted and the IR is materialized
- **THEN** the IR is identical to the materialization before the deletion

#### Scenario: Structure read from the IR, not from rendered markdown

- **WHEN** a structural consumer needs document structure
- **THEN** it reads the IR; rendered markdown is never reparsed to recover
  records or dependencies
