---
tags: [pipeline-run:tdd-ro5u-2026-11-19-kaya-8tn-codec-hardening, pipeline-step:green]
---

GREEN 8tn: parse_line refuses duplicate keys at any object depth via Deserializer visitor (label kaya.event.duplicate_key naming key + object path; escaped lookalikes parse); to_line returns Result<String, EventError> refusing non-finite floats (kaya.event.non_finite, finite output unchanged); 128-level depth boundary documented in module header. Callers migrated mechanically (append maps write error via event label; adapters split errors; test expects). Gates: full suite 30 suites green, clippy clean, ah run-tests 150/0. Deviation: NaN red-test unwritable as specified — serde_json 1 Value cannot represent non-finite (json! collapses to null pre-write; 1e999 refused at parse as kaya.event.json); the guard + doc test document the boundary; arbitrary_precision would be the lever and is out of ticket scope.
