# Data Model: Body-Only Message Compression

## Entities

## 1) EnvelopeHeader

- Description: Uncompressed metadata prefix used for routing, validation, and diagnostics.
- Fields:
  - `version.major: u8`
  - `version.minor: u8`
  - `messageType: u8` (must map to discriminants 1..7)
  - `messageId: [16]byte`
  - `timestampUtcMs: i64`
  - `agentId: string`
  - `platform: u8` (`1=Windows`, `2=Linux`)
  - `compressed: bool`
- Validation Rules:
  - Header MUST decode before body decompression.
  - `messageType` outside 1..7 is invalid.
  - Header fields remain uncompressed for both compressed and uncompressed messages.

## 2) MessageBody

- Description: Structured payload bytes for message-specific content (handshake/snapshot/ack/etc.).
- Fields:
  - `encodedBytes: byte[]`
  - `encoding: enum {Plain, Zstd}` inferred from `EnvelopeHeader.compressed`
  - `decodedPayload: MessagePayload` after optional decompression and parse
- Validation Rules:
  - If `compressed=false`, decode payload directly.
  - If `compressed=true`, payload MUST be decompressed before structured decode.
  - Decompression failure marks body invalid and surfaces explicit protocol error.

## 3) CompressionNegotiation

- Description: Session-level capability outcome determining whether compressed payloads are permitted.
- Fields:
  - `agentSupportsCompression: bool` (`CAP_COMPRESSION`)
  - `serverSupportsCompression: bool`
  - `compressionEnabled: bool` (`agentSupportsCompression && serverSupportsCompression`)
- Validation Rules:
  - Sender MUST NOT emit compressed payloads when `compressionEnabled=false`.
  - Receiver MUST NOT attempt decompression when `compressionEnabled=false`.

## 4) SizeGuards

- Description: Frame and decompression safety bounds.
- Fields:
  - `maxFrameBytes: int = 262144`
  - `maxDecompressedBodyBytes: int = 262144`
- Validation Rules:
  - Reject frame before decode if on-wire frame body exceeds `maxFrameBytes`.
  - Abort decompression and fail body decode if output exceeds `maxDecompressedBodyBytes`.

## 5) ProtocolErrorOutcome

- Description: Explicit decode/decompression failure record for diagnostics.
- Fields:
  - `errorKind: enum {FrameTooLarge, CrcMismatch, DecompressionFailed, DecompressedBodyTooLarge, PayloadDecodeFailed}`
  - `messageType: u8`
  - `messageId: [16]byte`
  - `details: string`
- Validation Rules:
  - For body failures, include at least `messageType` and `messageId`.
  - Error handling MUST not corrupt stream framing for subsequent frames.

## Relationships

- `EnvelopeHeader 1 -> 1 MessageBody` per decoded message.
- `CompressionNegotiation 1 -> N EnvelopeHeader` over session lifetime.
- `SizeGuards` apply to every frame/body decode operation.
- `ProtocolErrorOutcome` may be emitted for any `EnvelopeHeader`/`MessageBody` pair that fails validation.

## State Transitions

## Session Compression State

- `Unknown` -> `Enabled` when both peers advertise compression capability.
- `Unknown` -> `Disabled` when either peer lacks compression capability.
- `Enabled` and `Disabled` are stable for session duration.

## Message Decode State

- `ReadFramePrefix` -> `ValidateFrameSize`
- `ValidateFrameSize` -> `ReadFrameBody` (if <= `maxFrameBytes`) else `Reject(FrameTooLarge)`
- `ReadFrameBody` -> `ValidateCrc`
- `ValidateCrc` -> `ParseEnvelopeHeader` (if valid) else `Reject(CrcMismatch)`
- `ParseEnvelopeHeader` -> `DecodeBodyPlain` (if `compressed=false`)
- `ParseEnvelopeHeader` -> `DecompressBody` (if `compressed=true`)
- `DecompressBody` -> `DecodeBodyStructured` (if decompression succeeds and size <= cap)
- `DecompressBody` -> `Reject(DecompressionFailed|DecompressedBodyTooLarge)` on failure
- `DecodeBodyStructured` -> `MessageAccepted` on success or `Reject(PayloadDecodeFailed)` on parse failure
