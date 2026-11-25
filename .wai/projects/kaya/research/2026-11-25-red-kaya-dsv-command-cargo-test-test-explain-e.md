---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-dsv-author-missing-reader-documents, pipeline-step:red]
---

RED kaya-dsv: command=cargo test --test explain; expected failure=all three tests fail because the kaya binary has no explain subcommand (empty stderr rejection at invocation, after init succeeded - harness setup is sound); observed=failures: explain_names_kind_feeder_seed_path_and_dependents, explain_refuses_unknown_name_with_closed_vocabularies, explain_without_argument_lists_omissions_with_reasons; 0 passed 3 failed in 0.02s.
