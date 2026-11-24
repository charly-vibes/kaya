---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-j00-release-0-4-0, pipeline-step:red]
---

RED kaya-j00: command=python3 scripts/check-release-claims.py after bumping Cargo.toml to 0.4.0. Expected failure observed, exit 1: 'docs/src/status.md does not name v0.4.0' and 'the Release row status does not name v0.4.0'. The gate bites - this is the failing fixture the doc updates must satisfy.
