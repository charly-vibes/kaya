---
tags: [pipeline-run:tdd-ro5u-2026-11-19-kaya-g11-status-events-count, pipeline-step:orient]
---

ORIENT kaya-g11 P2: status.events is a newline count (src/main.rs:445): blank lines inflate it, an unterminated-but-complete final line is not counted even though status's own pending_proposals parses it and build admits it (src/main.rs:479-486,1036-1041). Fix: events = lines that parse as kaya.event/v0 (aligning with pending_proposals + build admission); blank lines counted in a NEW data.blank_lines field (never inflating events); unterminated final line counted as an event when it parses, with replay_integrity still naming recovered-truncated-tail. Status stays a cheap non-failing read (invalid lines skipped as today, still not counted as events). Digest/bytes unchanged (whole-file sha256). Re-scope: the Hegel status_build suite may now assert the FIXED status.events semantics alongside the ticket fix (it previously excluded g11).
