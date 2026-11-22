## MODIFIED Requirements

### Requirement: Source authority retained

An adapter SHALL read its source's records and SHALL never write to the
source's store, and SHALL never author content that duplicates what the
source already owns — kaya holds projections and interpretation, the source
holds its own truth.

### Requirement: Bounded content-derived attributes

A bounded content-derived attribute != text ownership; wai keeps full
text: the wai adapter MAY copy only a bounded first-heading title and its
digest from a record's own structure, and SHALL never copy the artifact
body or any arbitrary text. A bounded first-heading title is an observation
of the file's own structure, not an interpretation.

#### Scenario: Adapter writes nothing to its source

- **WHEN** any adapter runs against its source
- **THEN** the source's file tree is byte-identical before and after the
  adapter completes

#### Scenario: No duplicate authoring

- **WHEN** the same wai decision is captured twice
- **THEN** the second capture is acknowledged as a duplicate of the first
  (by source reference), not a second authored copy

#### Scenario: First hash-space heading becomes the title

- **WHEN** the wai adapter captures an artifact whose text carries a line
  starting with `# `
- **THEN** the event's `data` carries `title` (the heading's remainder,
  trimmed) and `title_digest` (the first 16 hex of sha256 over the title
  bytes), and the artifact body stays wai's truth

#### Scenario: Multibyte title truncated without char boundary panic

- **WHEN** the first heading's text contains multibyte characters and is
  longer than the bounded title length
- **THEN** the captured title is cut on a character boundary and the
  capture never panics

#### Scenario: Title attributes omitted when file empty or heading-less

- **WHEN** an artifact is empty or carries no `# ` heading line
- **THEN** the event's `data` omits the `title` and `title_digest` keys
  entirely

#### Scenario: Long title truncated to the bounded constant

- **WHEN** the first heading's text exceeds the bounded title length
  (`TITLE_MAX_LEN`, 80 characters)
- **THEN** the captured title is the first 80 characters of the heading

#### Scenario: Renamed path event id keeps stem digest stability

- **WHEN** only an artifact's heading text changes between captures
- **THEN** the event id is unchanged — id derivation stays tool, version,
  project, timestamp, reference, and the path-identity payload, never file
  content
