## ADDED Requirements

### Requirement: Bounded content-derived title attributes

A `capture.artifact.observed` event MAY carry two content-derived data
attributes: `title` (the artifact's first `# ` heading, trimmed and bounded
to 80 characters, cut on a character boundary) and `title_digest` (the
first 16 hex characters of the SHA-256 digest of the bounded title's UTF-8
bytes). Both SHALL be absent when the artifact is empty or carries no
`# ` heading.

#### Scenario: Title and digest derive deterministically

- **WHEN** the same artifact is captured twice with unchanged content
- **THEN** the `title` and `title_digest` attributes are byte-identical

#### Scenario: Multibyte title truncated without char boundary panic

- **WHEN** a heading of multibyte characters exceeds the bounded length
- **THEN** the title is cut on a character boundary and capture never
  panics

### Requirement: Event id independent of content

The `capture.artifact.observed` event id SHALL remain derived from path
inputs only — tool, version, project, timestamp, reference, and the
path-identity payload (kind and path) — never from file content.
Ownership decision: bounded content-derived attribute != text ownership;
wai keeps full text.

#### Scenario: Content edit does not move the event id

- **WHEN** only an artifact's heading text changes and it is re-captured
- **THEN** the event id is unchanged — id derivation stays path stem +
  digest, and only the title attribute moves
