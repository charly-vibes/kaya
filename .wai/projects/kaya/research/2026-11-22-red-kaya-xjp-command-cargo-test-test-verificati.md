---
tags: [pipeline-run:tdd-ro5u-2026-11-22-kaya-xjp-verification-facts-census, pipeline-step:red]
---

RED kaya-xjp: command=cargo test --test verification -> 28 passed, 7 failed.

Expected failures, all missing behavior and not setup breakage:
1. check_facts_turu_source: the turu bundle import DID land (events=14: 13 model + 1 capture.entry.observed; the projection emitted reuse-handoff.knowledge-brief, so the entry built a Source node), but events_by_source holds only {git,model,test-runner,wai} - there is no turu key. source_of() has no arm for capture.entry.observed.
2. check_required_source_turu_absent: exit 2 (usage: --require-source turu is not one of the canonical sources) instead of the expected exit 1 naming turu with reason no-events.
3. check_required_source_turu_satisfied: exit 2 for the same reason instead of the expected exit 0.
4. The same key-set assertion now fails in the pre-existing facts tests (check_facts_reported, check_facts_test_runner_source, check_sources_and_kinds_canonical, check_facts_property) because the local test mirror SOURCES grew from 4 to 5 entries - they fail on the missing turu key, not on a broken fixture.

Contracts declared in the change overlay .espectacular/changes/add-turu-source-row/kaya-verification/ with the cargo test flags bound: turu-entries-count-under-the-turu-source -> check_facts_turu_source; required-turu-source-satisfied -> check_required_source_turu_satisfied; required-turu-source-absent -> check_required_source_turu_absent.
