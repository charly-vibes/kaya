---
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-g9u-split-main-rs-into-command-modules-waves-tighten-pretender-ceiling, pipeline-step:plan]
---

PLAN kaya-g9u: mechanical extraction in waves, each closed green (cargo test after each). W1 src/commands/capture.rs<-cmd_capture+CaptureArgs. W2 src/commands/import.rs<-cmd_import+ImportSource+read_model_events. W3 src/commands/status.rs<-cmd_status+cmd_init. W4 src/commands/publication.rs<-cmd_build+cmd_ir+cmd_publish+cmd_check. W5 src/commands/verification.rs<-cmd_promote+cmd_challenge+cmd_acknowledge. W6 src/commands/doctest.rs<-cmd_doctest+cmd_supersede_doctest. W7 src/commands/impact.rs<-cmd_impact+cmd_disposition; then tighten pretender.toml file_lines_max to measured max + slim headroom + rewrite header note; pretender gate green. Acceptance: PRR header on every extracted module; tests 501+ green per wave. Clap CLI structs stay in main.rs; shared helpers pub(crate).
