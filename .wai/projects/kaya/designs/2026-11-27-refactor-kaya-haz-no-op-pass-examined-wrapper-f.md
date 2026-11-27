---
tags: [pipeline-run:tdd-ro5u-2026-11-27-kaya-haz-adopt-genesis-envelope-and-cli, pipeline-step:refactor]
---

REFACTOR kaya-haz: no-op pass — examined wrapper for tidying candidates (dual kind/envelope_kind in shell is load-bearing for the genesis mapping; pub warnings/hints/meta fields preserve the pre-adoption API and are read by the custom Serialize; struct-update syntax in Envelope::error already minimal). No changes; tests stay green from GREEN step run. No separate tidy commit needed.
