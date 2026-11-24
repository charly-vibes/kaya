---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-w9t-capture-commands-implementation, pipeline-step:refactor]
---

REFACTOR-STEP kaya-w9t: tidy applied while tests stayed green — capture.rs owns redaction+bounds+id+decision in one module; four subcommands share cmd_capture (no duplication); clippy lints (needless borrow, unnecessary filter_map, identity map_err) and fmt fixed during green verification; review fixes kept module boundaries intact (redaction isolated, dispatch thin). Nothing further.
