---
tags: [pipeline-run:tdd-ro5u-2026-11-19-kaya-7ma-status-digest-unification, pipeline-step:green]
---

GREEN 7ma: cmd_status digest = sha256_hex(&content) — the same one-shot shared with build (read_build_ledger) and check (verification); streaming hasher + sha2 import removed; buffer retained for size + tail byte. Gates: cli 9, status_build 4, cli_pipeline 27, full 30 suites green, clippy clean, ah run-tests 150/0. Repairing one mangled import line during the swap (compiler-caught).
