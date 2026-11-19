---
tags: [pipeline-run:tdd-ro5u-2026-11-19-kaya-7ma-status-digest-unification, pipeline-step:orient]
---

ORIENT kaya-7ma P3: status re-implements sha256 streaming inline (src/main.rs hasher, STATUS_BUFFER_BYTES) while build and check call the shared sha256_hex (src/ledger.rs). Since g11, cmd_status already accumulates the full ledger bytes in content — point the digest at sha256_hex(&content) and drop the hasher; digest values identical by construction (parity properties are the oracle). Also retire the now-unused streaming-hash comments; keep the byte-size + tail-byte tracking. Red: none behavioral (internal unification); oracle = existing digest parity + bytes assertions stay green and a unit check digest == sha256_hex(bytes).
