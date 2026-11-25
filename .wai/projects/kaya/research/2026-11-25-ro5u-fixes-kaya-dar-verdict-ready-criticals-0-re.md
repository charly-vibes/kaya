---
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-dar-pretender-target-exclude-glob-fix, pipeline-step:fix-review]
---

RO5U-FIXES kaya-dar: verdict READY CRITICALS=0 (review attached to green artifact). Two P3-LOW findings, both accepted as-is: (1) mutual_supersedes_cycle_is_refused lines().count()==1 is helper-implied redundancy - kept deliberately as the min_assertions-visible direct assertion the red finding demanded; (2) glob-twin drift risk mitigated by in-array comment pinning the 0.7.0 collect_path walk-relative matching quirk. No code fixes required.
