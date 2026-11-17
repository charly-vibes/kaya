---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-scq-status-release-architecture, pipeline-step:fix-review]
---

RO5U-FIXES kaya-scq: the export id tokenizer keeps '@' so every Source reference stays kaya/Source/<path>@<commit> (24/24 exported Source ids now carry the commit; counts unchanged at 77/6/1); publication.json regenerated and idempotent. The bound format_attribution test now compares each product's exported kinds to its admission row restricted to backed documents, in declaration order, compares rendered-section ids to that product's own export ids, and requires cross-reference sections to carry none. CLI attribution derives the section universe from admission, drops the unreachable guard, and asserts the newly omitted products have no page files after their backing document disappears. cmd_publish's doc comment names the eleven products. Verified: 23 test targets green, fmt/clippy clean, publish idempotent, ah check --run-tests 13 passed.
