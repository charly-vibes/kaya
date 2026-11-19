---
sut_path: /var/home/sasha/para/areas/dev/gh/charly/kaya
commit: cb8be97696b59f99909ba7f39a10daa2819294bc
updated: 2026-11-19
external_references:
  - path: /var/home/sasha/para/areas/dev/gh/charly/reports/hegel-eval-2026-11-16/README.md
    why: Suite-level topology conclusion (local PBT, no container design) adopted here.
---

# Deployment topology

Same conclusion as the suite-level pass, applied to kaya: these are local,
single-user CLI properties executed by an in-repo test runner. The decision
under evaluation is Hegel (local property-based testing), not Antithesis
fault-injection, so no container topology is designed.

| Component | Instances | Why |
|---|---|---|
| test process (`cargo test` / `cargo nextest`) | 1 | Runs the Hegel properties |
| tempdir fixture | 1 per case | Isolates `.kaya/` ledger state |
| built `kaya` binary | 1 per case (subprocess property only) | Status/build envelope parity |
| network / peers | 0 | None exists |
