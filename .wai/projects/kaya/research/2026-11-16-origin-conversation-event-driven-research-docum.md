# Origin conversation: event-driven research documentation system

Kaya's design originates from a November 2026 research conversation on making
documentation a projection of development history (transcript:
`~/cotorreo/raw/_conversations/chatgpt/6afa5381*` — renamed from the working
name "REVEAL" to "kaya").

Key decisions captured from that conversation:

1. **Integrate, don't duplicate.** Kaya consumes existing tool outputs (wai,
   dont, whisper, specodelic) through adapters. Each tool retains authority
   over its own domain; there are no competing copies of authoritative
   information.
2. **Append-only JSONL ledger** with the `kaya.event/v0` schema —
   human-inspectable, git-versionable, incrementally processable.
3. **Three epistemic categories:** events (tool-produced), assertions
   (human or AI proposal), conclusions (human-approved interpretation). An AI
   may never silently promote a proposal into an established finding.
4. **Generated vs authored sections** in managed blocks
   (`kaya:begin/end generated:*`, `kaya:authored`) — documentation updates
   without overwriting human interpretation, and stale evidence can be
   flagged.
5. **Workspace-first scope** with namespace-qualified experiment identities
   so multiple repositories can connect later.
6. **MVP 1 validation:** a static research compiler that reconstructs a
   useful dossier from existing wai/git/test artifacts — kaya itself becomes
   the first subject of the method it supports.

Rejected alternatives: isolated documentation app, workflow manager (wai's
territory), graph database (query patterns unknown), automatic paper
generation (encourages unsupported narrative).

Full design: `docs/src/index.md`.

