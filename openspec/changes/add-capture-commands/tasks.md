# Tasks: add-capture-commands

Proposal-only change: fix the capture-command surface and the commitments
the implementation must honour. No production code.

## 1. Implementation-ticket work items (one per scenario in the kaya-capture spec)

The follow-up ticket (kaya-w9t) implements these; each item names the
scenario it must verify. Command handler shared by all four subcommands:
redact → bound → compare against the key's effective text (the newest
stored event for its subject id in ledger order) → derive id → append or
correct.

- [ ] 1.1 `note_command_creates_an_observation_event` — one
      `model.entity.created` event, subject type `Observation`, envelope
      `appended`.
- [ ] 1.2 `question_command_creates_a_question_event` — subject type
      `Question`.
- [ ] 1.3 `decision_command_creates_a_decision_event` — subject type
      `Decision`.
- [ ] 1.4 `finding_command_creates_a_finding_event` — subject type
      `Finding`.
- [ ] 1.5 `capture_against_a_missing_ledger_refused` — `missing-ledger`,
      names `kaya init`, appends nothing.
- [ ] 1.6 `capture_subject_id_is_namespace_qualified` —
      `<project>/<TYPE>/<key>` subject id.
- [ ] 1.7 `capture_data_carries_exactly_text_and_key` — first capture's
      data is exactly `{text, key}`; a correction's data is exactly
      `{text, key, supersedes}`; no interpretation keys.
- [ ] 1.8 `capture_provenance_names_kaya_and_the_subcommand` — tool
      `kaya`, reference `<subcommand>/<key>`.
- [ ] 1.9 `capture_text_over_bound_refused_without_truncation` — 2001
      chars refused with `kaya.capture.bound_exceeded`, no event.
- [ ] 1.10 `capture_key_over_bound_or_off_alphabet_refused` — key > 64
      chars or outside `[a-z0-9-]` refused, no event.
- [ ] 1.11 `capture_empty_key_or_empty_text_refused` — empty key refused
      with `kaya.capture.bound_exceeded` naming `key`; empty text
      refused with `kaya.capture.empty_text`; no event.
- [ ] 1.12 `pasted_api_key_replaced_in_place_with_marker` — `sk-` token
      becomes `[redacted]`, surrounding words intact, token nowhere in
      the ledger.
- [ ] 1.13 `redaction_filter_covers_the_inherited_pattern_list` — every
      inherited family (`sk-`, `ghp_`, `github_pat_`, AWS `AKIA` and
      `ASIA`, Slack, JWT, private-key armour including PGP and
      `PuTTY-User-Key-File`, `token: value`) redacted, nothing else in
      the text changes.
- [ ] 1.14 `high_entropy_run_treated_as_secret` — 40-char base64 run
      redacted.
- [ ] 1.15 `benign_text_survives_redaction_unchanged` — text naming
      labels like `kaya.ledger.id_reuse` stored byte-identically.
- [ ] 1.16 `redacted_text_hashes_deterministically` — same pasted secret
      twice, same timestamp: same id, second append a byte-exact
      duplicate.
- [ ] 1.17 `distinct_captures_never_share_an_id` — different texts, same
      second: different ids.
- [ ] 1.18 `key_without_stored_text_treated_as_absent` — a key whose
      stored events carry no `data.text` (e.g. seed-imported attributes)
      counts as absent; the capture is a first capture.
- [ ] 1.19 `unchanged_recapture_appends_nothing` — same key, same
      effective redacted text: `duplicate`, no append.
- [ ] 1.20 `edited_capture_emits_a_superseding_correction` — changed
      text under an existing key: one event with `data.supersedes`
      naming the prior effective id.
- [ ] 1.21 `correction_preserves_the_subject_id` — node id never
      changes across corrections.
- [ ] 1.22 `correction_replay_appends_nothing` — deterministic
      generation-salted correction id; re-run is a duplicate.
- [ ] 1.23 `tool_actor_refused` — `--actor-type tool` refused with
      `kaya.capture.actor_required`, no event.
- [ ] 1.24 `default_actor_is_human` — omitted flag yields actor type
      `human`.
- [ ] 1.25 `agent_actor_recorded` — `--actor-type agent` recorded on the
      stored event.
- [ ] 1.26 Concurrent-capture test (design risk): two captures of the
      same key from two processes — the ledger lock serializes appends,
      both events survive, and the newest event in ledger order is the
      key's effective text.

## 2. Follow-up tickets to file

- 2.1 kaya-w9t — AFK implementation of the capture commands per section
  1 (this change's unblock target). (to be filed)
- 2.2 Validate the redaction pattern list against a corpus of real
  interactive paste; extend it with the misses. (to be filed)
- 2.3 Dogfooding: capture kaya's own next few decisions and findings
  with the commands; confirm decision records render acceptably with the
  pre-relationship staleness marker. (to be filed)
- 2.4 Relationship flags on capture (`--derives`, `--cites`) — revisit
  after dogfooding; seed files cover the closed grammar today. (filed
  when picked up)
- 2.5 Burst capture / stdin batching (`kaya note -`) — per-line
  redaction reporting and atomic refusal rules before any batch lands.
  (filed when picked up)

## 3. Verify (orchestrator)

- [ ] 3.1 `openspec validate --changes --strict` passes.
- [ ] 3.2 `git diff --stat src/` is empty — no production code in this
      change.
- [ ] 3.3 `cargo test` and `cargo clippy --all-targets -- -D warnings`
      stay green.
