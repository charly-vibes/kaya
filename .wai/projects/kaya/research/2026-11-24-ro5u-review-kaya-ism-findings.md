RO5U kaya-ism review (reviewer, CONVERGED stage 5, verdict NEEDS_REVISION): C:1 H:2 M:3 L:6 — every claim source-verified by reviewer (model.rs/ledger.rs/wai.rs/event.rs/main.rs greps; zero redaction code in src/ confirmed).
[CORR-001][CRITICAL] Req1 "exactly one event" vs nothing-appended scenarios; Req2 "exactly text+key" vs supersedes — quantifiers unsatisfiable as written. FIXED: at-most-one + envelope outcomes + supersedes clause + scenario 1.7 scoped.
[CORR-002][HIGH] agent_actor_recorded "visible in every projection" false (no projection renders actors; change promises zero projection work). FIXED: "carried on the stored event".
[CORR-003][HIGH] race risk + task asserted resolution nothing provides (two effective entity.created events; resolve_effective folds only supersedes; attributes empty). FIXED: true outcome + normative tie-break (newest stored event for subject id in ledger order) + task aligned.
[CORR-004][MEDIUM] id payload misattributed as "the adapters' derivation". FIXED: derivation scheme wording.
[CLAR-001][MEDIUM] "every seam shipped" false + pattern list only partially mirrored. FIXED: filter named the one unshipped seam; full inherited list (incl. ASIA, PuTTY/PGP) normative.
[EDGE-001][MEDIUM] lower bounds missing. FIXED: key 1-64 [a-z0-9-], empty text refused kaya.capture.empty_text, new scenario.
LOWs fixed: door wording, existing-ledger preconditions on all four command scenarios, seed-key treated-as-absent scenario, ledger-internal reference clause, door-list drift, sixth-arm off-by-one.
Spec now 7 requirements / 25 scenarios; tasks 1.1-1.26 one-per-scenario.
