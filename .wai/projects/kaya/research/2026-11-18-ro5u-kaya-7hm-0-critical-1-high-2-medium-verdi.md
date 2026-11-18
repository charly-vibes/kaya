---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-7hm-authored-staleness, pipeline-step:ro5u-review]
---

RO5U kaya-7hm: 0 Critical, 1 High, 2 Medium. Verdict incorrect.
HIGH 1: acknowledgment coverage is unversioned; after acknowledging an id, a later correction of the same id is silently suppressed and cannot be re-acknowledged. Fix: record the reviewed superseding event id(s) in the ack event and only treat an id as covered while its latest correction is the reviewed one; equivalence must include the reviewed map so a newer correction is a new review. Add the re-correction test.
MEDIUM 2: two-segment grammar contradicts the deployed kaya-event invariant and the refusal text. Fix: keep >=3 segments plus one explicit documented exception for `verification.acknowledged`; update the message and restore the run.completed refusal case.
MEDIUM 3: acknowledgment labels are not canonical; absolute/non-canonical document paths append an ack that never matches. Fix: canonicalize the document against cwd (strip root, refuse outside-root) for both ack and build labels; add an absolute-path test.

