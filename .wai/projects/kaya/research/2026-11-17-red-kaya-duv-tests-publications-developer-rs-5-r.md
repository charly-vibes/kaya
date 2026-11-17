---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-duv-developer-products, pipeline-step:red]
---

RED kaya-duv: tests/publications_developer.rs (5 regression guards over the developer admission rows, already green against the library) and the CLI test publish_declares_every_product_and_reports_justified_omissions, which is red because cmd_publish declares only overview/reference/project-site: expected pages [developer-guide, getting-started, overview, project-site, reference] and omitted [concept, example, faq, how-to, troubleshooting]. Fact-once ownership checked: getting-started sorts before overview, so it owns orientation.project-overview and the overview page cross-references it. Existing publish test updated accordingly. No setup errors.
