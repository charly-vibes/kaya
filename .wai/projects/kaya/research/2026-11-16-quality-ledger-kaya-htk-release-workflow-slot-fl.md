---
tags: [pipeline-run:tdd-ro5u-2026-11-16-kaya-htk-release-workflow, pipeline-step:quality-ledger]
---

QUALITY LEDGER kaya-htk: release workflow slot (fleet standard §1).

Changed:
- .github/workflows/release.yml (new): Release, tag v* only; build-linux/darwin/windows + test gate; publish needs all four; workflow permissions contents: read, publish contents: write; 18 action refs at full SHAs; tag==Cargo.toml gate; five canonical archives + checksums.txt + provenance.json; idempotent release create/upload; tap/bucket/crates channels gated on release success, retried, aggregated in the job summary; crates.io token-gated (disabled, kaya-dgz).
- scripts/update-homebrew.py, scripts/update-scoop.py (new): render channel artifacts from checksums.txt; reject tag/version mismatch, missing platform digests (no partial writes), tolerate ./ prefixes, byte-identical re-render; formula test block uses real #{bin} interpolation (the wai/ddl template escapes it).
- tests/release_scripts.rs (new): 7 behavioral tests over fixtures (render, missing platform, dot-prefix, tag mismatch, idempotence, url->digest pairing).
- .gitignore: /bins/, /dist/ (release artifacts must not enter the crate or the tree).
- README.md (Release badge), docs/src/status.md (release row + crates.io deviation), CHANGELOG.md.

Verified:
- cargo test: 154 passed, 0 failed.
- just ci: green (fmt, clippy -D warnings, mdbook, ah check, spk lint, openspec --strict).
- actionlint (+shellcheck) clean.
- ruby -ryaml structural assertions: step order, ids, gates, publish permissions.
- cargo publish --dry-run --allow-dirty --locked: packages + verifies, 0 bins/dist entries.
- local packaging simulation (fake bins): archives carry exec bit, checksums.txt, provenance.json (6 artifacts), formula passes ruby -c, scoop manifest templated; tag mismatch exits 1 without writing.

Review: RO5U verdict fail -> all 6 findings fixed (1 high, 3 medium, 2 low). No deferrals.

Risks:
- release.yml has never executed on GitHub (no tag pushed; tagging is a release action requiring authorization).
- First real tag will be the first exercise of the crates.io/tap/bucket paths and the Windows staging; secrets CARGO_REGISTRY_TOKEN and TAP_GITHUB_TOKEN are not configured yet, so both channels will report 'disabled'.
- Fleet template bug found: ddl/wai generate formulas with system "\#{bin}/tool" (escaped interpolation) so brew test would exec a literal '#{bin}/tool'. kaya's template is correct; the fleet scripts still need the fix (not in this repo).

Next: ship-close (commit ticket files, close kaya-htk, export beads), then propose [38;2;56;56;56m╭────────────────────────────────────╮[m
[38;2;56;56;56m│[m lefthook  v2.2.2   hook:  [1mpre-push[m [38;2;56;56;56m│[m
[38;2;56;56;56m╰────────────────────────────────────╯[m
│  core.hooksPath is set globally to '~/.git-hooks' 
│  Custom hooks paths are not supported by default. 
│                                                   
│  Reset core.hooksPath automatically:              
│    lefthook install --reset-hooks-path            
│                                                   
│  Remove it manually:                              
│     git config --unset-all --global core.hooksPath
│                                                   
│  Force installation into current hooks path:      
│    lefthook install --force                       
│                                                   
│    Target:                                        
│    ~/.git-hooks                                   
│  Skipping hook sync: 
│  bd (skip) no matching push files
│  ddl-ah-check (skip) no matching push files
[38;2;56;56;56m  ────────────────────────────────────[m
summary: (done in 3.75 seconds) and, separately, tag authorization.
