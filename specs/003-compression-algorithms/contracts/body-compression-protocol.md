# Contract: Body-Only Compression Protocol Behavior

## Scope

This contract defines how compression is represented and processed for protocol messages in this repository. It applies to both Rust (`agent/src/protocol.rs`) and .NET (`server/Protocol/FrameCodec.cs`).

## Frame-Level Contract

- Frame layout remains unchanged:
  - `lengthPrefix`: 4 bytes, unsigned big-endian (`u32`)
  - `body`: `lengthPrefix` bytes
  - `crc32`: 4 bytes, unsigned little-endian (`u32`) computed over `body`
- `body` MUST NOT exceed `maxFrameBytes=262144`.
- If `body` exceeds cap, receiver rejects frame without attempting decompression.

## Envelope/Body Boundary Contract

- Envelope fields are always uncompressed and must be parseable before body processing:
  - `version.major`
  - `version.minor`
  - `messageType`
  - `messageId`
  - `timestampUtcMs`
  - `agentId`
  - `platform`
  - `compressed`
- `compressed` controls body handling only.
- Body bytes are interpreted as:
  - raw payload bytes when `compressed=false`
  - zstd-compressed payload bytes when `compressed=true`

## Negotiation Contract

- Compression is capability-gated.
- Compressed bodies are allowed only when both peers support compression capability.
- If compression is not mutually negotiated:
  - sender MUST emit `compressed=false`
  - receiver MUST decode payload without decompression attempt

## Decompression Safety Contract

- Receiver enforces `maxDecompressedBodyBytes=262144`.
- If decompressed output would exceed this cap, decompression is aborted and body is invalid.
- If decompression fails for any reason, receiver emits explicit error outcome and includes envelope identity metadata (`messageType`, `messageId`) for diagnostics.

## Error Handling Contract

- Failure categories covered by this feature:
  - On-wire frame too large
  - CRC mismatch
  - Decompression failed
  - Decompressed body too large
  - Payload decode failed after successful decompression
- For body-related failures, receiver preserves already-parsed envelope metadata in diagnostics.
- Receiver must not reinterpret subsequent frame boundaries because of a body decode failure.

## Cross-Language Compatibility Contract

- Message type discriminants remain fixed:
  - `1=Handshake`, `2=HandshakeAck`, `3=Heartbeat`, `4=Snapshot`, `5=Ack`, `6=Backpressure`, `7=Error`
- Compression algorithm remains zstd level 3 for sender-side compression.
- Any change to compression fields/limits requires mirrored updates in Rust and .NET plus interoperability tests.
