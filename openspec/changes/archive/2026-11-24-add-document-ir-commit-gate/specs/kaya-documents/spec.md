## MODIFIED Requirements

### Requirement: Doctest execution contracts

An example MAY declare a doctest execution contract: a runner, its steps, and
expected output. A declared doctest SHALL become a first-class IR record with
a stable id and a content hash; undeclared code SHALL remain prose. Expected
output SHALL change only through a human-recorded supersede event. Runs SHALL
execute through the controlled test-runner source in isolation and be
recorded as test-runner facts of kind `doctest`.

#### Scenario: Declared doctest is a first-class record

- **WHEN** a document source declares an example with runner, steps, and
  expected output
- **THEN** the IR carries a doctest record with a stable id and a content
  hash, and the rendered block keeps the example's authored form

#### Scenario: Undeclared code stays prose

- **WHEN** a fenced code block declares no execution contract
- **THEN** no doctest record is created for it and no runner is invoked

#### Scenario: Failing run never rewrites expected output

- **WHEN** a doctest run exits nonzero or produces different output
- **THEN** the run is recorded as a failed test-runner fact and the doctest
  record's expected output is unchanged

#### Scenario: Doctest counts under test-runner

- **WHEN** a doctest run is recorded
- **THEN** the verification report counts it under the `test-runner` source
  with kind `doctest`, and no other source names it
