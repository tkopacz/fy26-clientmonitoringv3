# Research: Body-Only Message Compression

## Decision 1: Keep compression boundary at payload bytes only

- Decision: Preserve current encode/decode structure where the envelope/header remains plaintext and only message payload bytes are compressed when `compressed=true`.
- Rationale: This guarantees the receiver can parse version, message type, message id, timestamp, platform, and compression flag before any decompression work.
- Alternatives considered:
  - Compress entire serialized message (envelope + payload): rejected because routing/validation would require decompression first.
  - Mixed field-level compression in envelope: rejected due to complexity and lower interoperability.

## Decision 2: Enforce dual size guards (wire and decompressed)

- Decision: Enforce `maxFrameBytes=262144` prior to decode/decompression and add explicit `maxDecompressedBodyBytes=262144` enforcement during decompression.
- Rationale: The dual guard addresses both oversized wire frames and decompression bomb risk while satisfying clarified requirement to cap both dimensions.
- Alternatives considered:
  - Only on-wire cap: rejected because highly compressed payloads can expand beyond safe memory limits.
  - Only decompressed cap: rejected because oversized frames should fail before expensive decoding.

## Decision 3: Keep capability-negotiated compression behavior

- Decision: Continue using handshake capability bit (`CAP_COMPRESSION`) and permit compressed payloads only when both peers advertise support.
- Rationale: Maintains backward compatibility with existing protocol negotiation and prevents unilateral compression from breaking older peers.
- Alternatives considered:
  - Always-on compression by protocol version: rejected because mixed-version fleets would fail.
  - Per-message trial-and-error decode fallback: rejected because it obscures protocol intent and failure diagnostics.

## Decision 4: Preserve message discriminants and framing unchanged

- Decision: Keep message type discriminants `1..7` and frame layout `[len:u32 BE][body][crc32:u32 LE]` unchanged.
- Rationale: Feature scope is compression placement/validation, not wire protocol redesign; stability avoids cross-language breaking changes.
- Alternatives considered:
  - Introduce new message type for compressed payloads: rejected as unnecessary schema churn.
  - Replace frame checksum or prefix format: rejected as out of scope and high-risk.

## Decision 5: Standardize decompression-failure diagnostics with envelope identity

- Decision: On decompression failure, treat body as invalid, raise explicit protocol error, and include envelope identity metadata (`messageType`, `messageId`) in diagnostics/test expectations.
- Rationale: Supports operator troubleshooting and aligns with fail-safe requirement while preserving stream alignment for subsequent frames.
- Alternatives considered:
  - Generic decode error without identity fields: rejected due to poor observability.
  - Hard-close session on first decompression error: rejected as overly disruptive for isolated corrupt frames.
