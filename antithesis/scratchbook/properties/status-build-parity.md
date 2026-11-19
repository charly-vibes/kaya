# status-build-parity — status and build digest the same bytes and agree on the record

Evidence file for [`property-catalog.md`](../property-catalog.md) (P3). Repo kaya @ `cb8be97696b59f99909ba7f39a10daa2819294bc`.

**Disposition (pinned):** `tests/status_build_properties.rs` via hegeltest nx4, subprocess state machine ACCEPTED (cost ~0.07 s; the machine owns its tempdir git repo; deterministic exhaustive mutation alphabet so the `ci` profile loses nothing). Scoped by the build-acceptance classifier; includes the ledger-immutability oracle. The `status.events` accounting cases and the build-envelope gap are tickets kaya-g11 / kaya-8yw / kaya-7ma.

| | |
|---|---|
| **Type** | Safety (envelope parity) |
| **Invariant** | For a fixed ledger byte sequence `B` in a fixed git state, one `kaya status` run and one `kaya build --out <tmp> [--allow-uncommitted]` run over the same bytes either both accept — then `status.data.digest == build.data.ledger_digest`, `status.data.bytes == |B|`, `status.data.pending_proposals == build.data.pending_proposals` — or the ledger is outside build's acceptance set, in which case `build.ok == false` with a labeled error. Neither run changes `B` on disk or writes anything outside `--out`. |
| **Check mode** | Subprocess state machine (`assert_cmd`/`Command`), 2 binary spawns per step. |
| **Seam** | None usable for the full property; partial pure seam exists (see below). |

## Are the digests over identical bytes? Yes — by construction

Both paths SHA-256 the **entire file**, and only the file:

- **status** (src/main.rs:420-466): `File::open` (read-only, src/main.rs:422), one streaming pass in a 64 KiB buffer (`STATUS_BUFFER_BYTES`, src/main.rs:41, buffer at :430), `hasher.update(chunk)` per chunk (src/main.rs:443), envelope emits `"digest": format!("{:x}", hasher.finalize())` (src/main.rs:459) and `"bytes"` (src/main.rs:458). Streaming SHA-256 ≡ one-shot SHA-256 over the same bytes — mathematically identical.
- **build** (src/main.rs:597-740): `read_build_ledger` reads the whole file once with `fs::read` (src/main.rs:917-927), then `let digest = sha256_hex(&bytes)` (src/main.rs:944), where `sha256_hex` is `format!("{:x}", Sha256::digest(bytes))` (src/ledger.rs:336-338). Emitted as `"ledger_digest"` (src/main.rs:738).

Same bytes, same algorithm, no salt, no clock, no normalization → **digest field parity is green by construction** whenever both commands read the same file snapshot. `kaya check` is a third observer of the identical function: `sha256_hex(&bytes)` over `fs::read` bytes (src/verification/mod.rs:577-590; documented in src/verification/facts.rs:46-47, surfaced as `facts.events`/`ledger_digest` in the check envelope at src/main.rs:852-853).

## Can either path mutate the ledger or derived state on read? No — the suspected lead is refuted at this pin (negative result)

- **status**: two reads only — `File::open` + streaming `read` (src/main.rs:422-447) and a second full `fs::read` inside `pending_proposals()` (src/main.rs:474-499). No `OpenOptions::write`, no `set_len`, no directory creation. No writes exist on the path.
- **build**: reads the ledger once (src/main.rs:917), spawns `git ls-files --error-unmatch` and `git show HEAD:.kaya/ledger.jsonl` (src/main.rs:963-995, both read-only), and writes **only under `--out`**: `index.md` (src/main.rs:619-623), `experiments/*.md` (:636-649), `understanding.md` (:668-671), `overview.md` (:683-686), reuse documents (:700-704). `flag_stale_authored` (src/main.rs:875-912) only *reads back* the file it just compiled; it never rewrites ("The document is read back after compilation, never rewritten", src/main.rs:870-871).
- The mutation-on-read machinery the ticket suspected **does exist in the library but is not on either command's path**: `Ledger::recover` (src/ledger.rs:275-315) appends a newline to a complete-but-unterminated final event (`write_all(b"\n")`, src/ledger.rs:293) or truncates an unparseable tail (`file.set_len`, src/ledger.rs:308), reached only via `load_inner` (src/ledger.rs:227) ← `load`/`load_optional` (:218, :222) ← `read_recovered` (:90) and `append`. Its callers are `cmd_promote` (src/main.rs:1074), `cmd_challenge` (:1095), `cmd_acknowledge` (:1197), and `append` itself — **not** `cmd_status`/`cmd_build`, which bypass `Ledger` entirely. So: `status` does not mutate at this commit; the recovery-write hazard is real but lives on the promote/challenge/acknowledge/append paths (ticket kaya-1y9).

## Where status and build genuinely disagree (predicted red components → tickets)

1. **`status.events` is a newline count, not an event count** (src/main.rs:445: `events += chunk.iter().filter(|b| *b == b'\n').count()`). Three divergences fall out (ticket kaya-g11):
   - A **blank line** adds 1 to `status.events`, while build refuses the ledger outright (`parse_line("")` fails → `kaya.ledger.record_invalid`, src/main.rs:1031-1043).
   - A **complete-but-unterminated final line** adds 0 to `status.events` and sets `replay_integrity: "recovered-truncated-tail"` (src/main.rs:449-451) — yet status's *own* `pending_proposals` parses that same line (split on `\n` includes the tail, src/main.rs:479-486), and build *admits* it as an event (`split_inclusive` + `strip_suffix(b"\n").unwrap_or(raw)`, src/main.rs:1036-1041). So on the untracked/`--allow-uncommitted` path build serves N events while status reports `events: N-1` and simultaneously counts the Nth event in `pending_proposals`. Digest still agrees (same bytes). Build admitting a crash tail the library itself refuses to serve, without any recovery report, is its own ticket (kaya-8yw).
   - A **duplicate event id** appended by hand: status stays `ok` with a digest; build refuses `kaya.ledger.id_reuse` (src/main.rs:1047-1056).
2. **Acceptance sets differ**: `status.accepted ⊃ build.accepted`. Status is deliberately "a cheap, non-failing read" that skips unparseable lines (src/main.rs:468-472); build refuses non-UTF-8 (:1032-1035), invalid records (:1036-1043), reused ids (:1047-1056), and tracked-ledger growth that doesn't extend HEAD on a line boundary (`verify_committed_ledger`, src/main.rs:997-1013). Therefore the parity invariant must be **scoped to build-accepted ledgers** (hence the build-acceptance classifier), or the unaccepted half becomes the separate property "status ok ⇒ at worst a labeled build refusal, never a digest mismatch".
3. **Build's envelope exposes no event count** (src/main.rs:728-739: `out, recomputed, unchanged, omitted, stale, sources, pending_proposals, ledger_digest` only), so envelope-level *event-count* parity is unobservable today; the comparable shared fields are `ledger_digest` and `pending_proposals`. `check` is the only command whose envelope carries `events` (src/main.rs:852). Ticket kaya-7ma: expose `events` in the build envelope.

## Generator sketch (subprocess state machine)

- **State**: ledger bytes `B` in a tempdir; profile ∈ {committed (git init + `git add .kaya/ledger.jsonl` + commit after each mutation), untracked (no git repo or untracked file + `--allow-uncommitted`)}. Seed `B` with a fixed valid 3-event ledger (`kaya init` + `import model` from a fixed seed file).
- **Actions** (each is a byte-level file mutation, deliberately including cases no writer would produce): `AppendValidEvent` (serialize a generated `Event` + `\n`), `AppendDuplicateLine` (byte-identical copy of an existing line), `CorruptLastLine` (flip one byte inside the final line, keep `\n`), `DeleteByte` (drop one byte mid-line → invalid UTF-8 or invalid JSON), `TruncateMidLine` (drop k < last-line-length bytes → unterminated tail).
- **Step**: run `kaya status`, then `kaya build --out <fresh tmp> [--allow-uncommitted]`; hash `B` before and after.
- **Oracle**:
  1. Ledger immutability: `sha256(B_before) == sha256(B_after)` after both runs; no entries created outside `--out` (mutation-on-read refuted continuously, not just once).
  2. Classifier agreement: an explicit in-test predicate `build_accepts(B, profile)` — newline-terminated ∧ every line a valid unique-id event ∧ (untracked ∨ extends committed HEAD on a line boundary). Where true: both `ok == true`, `status.digest == build.ledger_digest`, `status.bytes == |B|`, `status.pending_proposals == build.pending_proposals`. Where false: `build.ok == false` with a non-empty `error.label`; status may still be `ok`.
  3. Exactly one pretty-JSON envelope on each stdout (envelope.rs:63-70) with `process.exit == 0 ⇔ ok == true` (the envelope-exit-code-agreement property, free on every spawn).
- **Predicted red/green**: digest parity **green** (§ above); immutability **green**; pending-proposal parity **green** on build-accepted ledgers; **red expected** on the `status.events` accounting cases (blank line, unterminated tail) — those reds are ticket material (kaya-g11, kaya-8yw), not flakiness, and are scoped out of the property domain by the classifier. Hegel precedent: the eval ran a subprocess CLI state machine at 100 cases × 12 steps in 0.44 s with readable 4-step shrinks (reports/hegel-eval-2026-11-16/README.md §2; §4 puts this exact property in Phase 4).

## status-build-parity-pure-seam — is there a shared digest function that collapses the property? (P3b)

| | |
|---|---|
| **Seam found** | Partial. `sha256_hex` (src/ledger.rs:336-338) is the shared one-shot digest and is what build (src/main.rs:944) and check (src/verification/mod.rs:590) call. **Status does not use it** — it re-implements hashing inline as a streaming `Sha256` (src/main.rs:430-447, finalize at :459). |
| **If unified** | Pointing status's digest at `sha256_hex` (or extracting `fn ledger_envelope_facts(bytes: &[u8]) -> (events, bytes, digest, replay_integrity)` used by both) makes digest parity a tautology over identical bytes and reduces the property to: (a) both commands read the same file (no interleaved writer — trivial in a test), and (b) the acceptance predicates and envelope rendering agree. That residue is *only* exercisable at the binary boundary, because the interesting divergences live in `parse_ledger_bytes` (src/main.rs:1031) vs status's newline count (src/main.rs:445) and lenient `pending_proposals` (src/main.rs:472) — three independently written parsers of the same bytes. |
| **Recommendation** | **Keep the subprocess property** (matches the eval's Phase 4 placement); the measured spawn cost makes it effectively free (below). Do *not* settle for the pure seam alone: `sha256_hex(bytes) == sha256_hex(bytes)` proves nothing about the envelopes, refusal boundaries, or the three-parser disagreement, which is where every predicted red lives. The one-line unification of status's hasher onto `sha256_hex` is a cheap hardening commit to file alongside the property, not a replacement for it. |

## subprocess-cost-nx4 — accept the subprocess generator

- Measured baseline from the eval: **100 cases × up to 12 steps, each step spawning the real binary, ran in 0.44 s** (reports/hegel-eval-2026-11-16/README.md §2) ≈ 0.37 ms/spawn.
- This property, n×4 (4 ledger cases × ≤12 steps): 2 binary spawns per step (`status` + `build`) plus build's 1–2 git spawns (`ls-files` always; `show` only for tracked ledgers — src/main.rs:963-975) ⇒ worst case 4 × 12 × 4 = **192 spawns ≈ 0.07 s** of subprocess time. Even a 25×-per-case expansion stays under a second.
- The dominant cost is building the kaya dev-binary (one-time, already a test dependency) and Hegel's engine (`static-engine` cold build 16.2 s, README §2 — a suite-level cost, not per-property).
- **Verdict (pinned): accept.** The eval's "expensive" tag for `kaya/status-report-agrees-with-verified-read` was relative to in-process properties; at 4 cases the absolute cost is noise. Defer only if the case count grows to hundreds.

## ci-derandomization — determinism confirmed, no weakening

- Digest parity is a pure function of bytes: SHA-256 has no randomness, and neither envelope carries wall-clock or id-derived nondeterminism — status emits `ledger, events, bytes, digest, replay_integrity, pending_proposals` (src/main.rs:453-464), build emits only ledger-derived fields plus the `--out` path (src/main.rs:728-739). `new_id`/timestamps appear only inside generated ledger *content*, which the generator fixes.
- Under `CI=true`, Hegel selects the `ci` profile: **database off, derandomized per test name** (README §2). That removes database replay but not coverage **provided the generator enumerates the mutation alphabet deterministically** — the alphabet here is 5 atomic mutations × 2 git profiles, small enough to sweep exhaustively in one derandomized run. Recommendation (pinned): write the machine so one pass covers all 5 mutations on both profiles; the property then loses nothing under `ci` and stays replayable via a stored failing example locally.
- One test-infra caveat, not a property weakness: the *committed* profile depends on git state (`verify_committed_ledger`, src/main.rs:997), so the machine must own its tempdir repo (init/commit at known points) rather than inheriting the developer's worktree — this also keeps the property hermetic under CI.

## Open questions

- None that change the invariant; all divergences are ticketed (kaya-g11, kaya-8yw, kaya-7ma) and the status-hasher unification is a follow-up hardening commit, not an open question.
