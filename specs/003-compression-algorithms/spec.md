# Feature Specification: Body-Only Message Compression

**Feature Branch**: `003-compression-algorithms`  
**Created**: 2026-02-26  
**Status**: Draft  
**Input**: User description: "add compression to message body not header"

## Clarifications

### Session 2026-02-26

- Q: Should we cap on-wire frame bytes, decompressed body bytes, or both? → A: Cap both.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Parse and route messages without decompressing headers (Priority: P1)

A protocol maintainer or operator can rely on message header/envelope fields being readable without decompressing the message body so the server can validate, route, and apply flow-control consistently.

**Why this priority**: If the server must decompress to read basic metadata, it adds avoidable CPU overhead, complicates error handling, and reduces observability.

**Independent Test**: Send one compressed and one uncompressed message; confirm the receiver can read header/envelope fields (version, message type, message id, timestamp, compression flag) for both without attempting decompression first.

**Acceptance Scenarios**:

1. **Given** a message is transmitted with compression enabled, **When** the receiver reads the frame, **Then** it can decode the header/envelope (including the compression indicator) without decompressing the message body.
2. **Given** a message is transmitted without compression, **When** the receiver reads the frame, **Then** it decodes the same header/envelope fields using the same parsing logic.

---

### User Story 2 - Negotiate compression safely across versions (Priority: P2)

A protocol maintainer can evolve compression behavior without breaking older agents/servers by making compression opt-in via capability negotiation and keeping header/envelope decoding stable.

**Why this priority**: Compression is a performance feature that must not create reliability regressions or compatibility breakages.

**Independent Test**: Simulate two sessions: one where both sides support compression and one where only one side does; verify the session either uses compression (mutual support) or stays uncompressed (no mutual support) while header parsing stays consistent.

**Acceptance Scenarios**:

1. **Given** both peers advertise support for body-only compression, **When** they complete handshake, **Then** they may send compressed bodies and the receiver correctly interprets the compression indicator.
2. **Given** only one peer supports compression, **When** they complete handshake, **Then** they proceed without compression and the receiver does not attempt to decompress.

---

### User Story 3 - Fail safely on malformed compressed bodies (Priority: P3)

An operator can diagnose corrupted/malformed compressed bodies without losing the ability to interpret message metadata, and the session remains stable when safe.

**Why this priority**: Compression introduces a new failure mode (decompression failure) that should be handled explicitly and observably.

**Independent Test**: Send a frame whose header indicates compression but whose body is invalid/corrupted; verify the receiver still extracts header metadata and produces a clear error outcome for the body.

**Acceptance Scenarios**:

1. **Given** a message indicates a compressed body, **When** decompression fails, **Then** the receiver surfaces a protocol error that includes the message id/type and does not misinterpret subsequent frames.

**Story sequencing note**: Each user story remains independently testable, but implementation is expected to follow priority order (P1 → P2 → P3) because later stories reuse the stable decode and negotiation paths established earlier.

### Edge Cases

- Compression indicator set but body is not compressed (or uses a different algorithm).
- Body decompresses successfully but decoded body content is invalid/corrupted.
- Oversized frames when compression is disabled (should be rejected consistently).
- Very small payloads where compression expands size; policy should still behave deterministically.

## Assumptions & Scope

- The protocol already has a concept of a header/envelope and a body/payload.
- Session negotiation already exists (or will exist) to agree on whether body-only compression is allowed.
- This feature defines *where* compression applies (body only), not *which* compression algorithm is used.

**Out of scope**:

- Changing framing, length-prefixing, or transport behavior.
- Changing message schemas unrelated to compression signaling.
- Adding new observability UX beyond emitting clear error outcomes for decompression failures.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The protocol MUST define a clear separation between the message header/envelope and the message body/payload.
- **FR-002**: If compression is used, it MUST be applied to the message body/payload only. The header/envelope MUST NOT be compressed.
- **FR-003**: The receiver MUST be able to decode header/envelope fields (including any compression indicator) without performing decompression.
- **FR-004**: Compression MUST be negotiated via capability signaling so that compression is only used when both peers support it.
- **FR-005**: The header/envelope MUST include an explicit indicator that the body/payload is compressed (or not) so the receiver can decide whether to decompress.
- **FR-006**: When a message indicates a compressed body, the receiver MUST attempt decompression of the body before decoding the body’s structured fields.
- **FR-007**: If decompression fails, the receiver MUST treat the message body as invalid and surface an explicit error outcome while preserving header metadata (at minimum: message type and message id) for diagnostics.
- **FR-008**: If compression is not negotiated, the sender MUST NOT send compressed bodies and the receiver MUST NOT attempt decompression.
- **FR-009**: Size limits MUST apply consistently regardless of whether compression is enabled. A frame that exceeds the maximum allowed size MUST be rejected deterministically.
- **FR-010**: The receiver MUST enforce both (a) an on-wire maximum frame size cap and (b) a maximum decompressed body size cap.
	- If the on-wire frame exceeds `maxFrameBytes`, the receiver MUST reject it without attempting decompression.
	- If the decompressed body would exceed `maxDecompressedBodyBytes`, the receiver MUST abort decompression and treat the body as invalid.
	- Defaults for this project: `maxFrameBytes = 262144` bytes and `maxDecompressedBodyBytes = 262144` bytes.
	- Rejections MUST surface an explicit error outcome that includes message identity metadata (at minimum: message type and message id).

### Key Entities *(include if feature involves data)*

- **Header/Envelope**: Uncompressed metadata required for decoding/routing (e.g., protocol version, message type, message id, timestamp, and compression indicator).
- **Body/Payload**: Structured message content which may be compressed when negotiated.
- **Compression Capability**: A negotiated property that determines whether body-only compression is allowed for the session.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Using a shared valid-frame corpus (compressed + uncompressed fixtures that pass framing, CRC, and schema checks), receivers extract required header/envelope metadata for 100% of corpus frames before any body decompression step.
- **SC-002**: Using a fixed corpus of at least 1,000 messages with payload size ≥ 4 KiB, body-only compression yields an average on-wire body byte reduction of at least 20% versus uncompressed transmission.
	- Baseline failure rate is measured from the uncompressed run on the same corpus.
	- Compressed-run failure rate MAY increase by at most 0.1 percentage points versus baseline.
- **SC-003**: In a deterministic negative-test suite of at least 20 malformed compressed frames, 100% of failures produce explicit error outcomes with message identity metadata and 100% of immediately subsequent valid frames are accepted.
