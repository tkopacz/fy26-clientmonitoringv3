---
description: "Task list for Binary Protocol & IO Core"
---

# Tasks: Binary Protocol & IO Core

**Input**: Design documents from `/specs/001-protocol-messaging/`

**Tests**: Per the Constitution and FR-013, tests are MANDATORY. Write tests first and ensure they fail before implementation when practical.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Establish a clean baseline and ensure build/test workflow is reproducible.

- [ ] T001 Run baseline suite using run-all-tests.sh and record failures in specs/001-protocol-messaging/tasks.md
- [ ] T002 [P] Add a minimal local dev run note to specs/001-protocol-messaging/quickstart.md (TCP server + agent sender)
- [ ] T003 [P] Add/confirm a repo-level protocol decision summary in specs/001-protocol-messaging/research.md (ensure matches spec Clarifications)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared protocol contract + core codec behavior required by all stories.

**⚠️ CRITICAL**: No user story work should begin until this phase is complete.

- [ ] T004 Define envelope metadata fields (messageId, agent id, platform, per-message timestampUtc) in agent/src/protocol.rs and server/Protocol/Messages.cs
- [ ] T005 Update framing serialization/deserialization to include length prefix + CRC32 checksum validation and the updated envelope fields in agent/src/protocol.rs and server/Protocol/FrameCodec.cs
- [ ] T006 [P] Update Rust protocol unit tests for envelope field round-trips in agent/tests/protocol_tests.rs
- [ ] T007 [P] Update .NET protocol unit tests for envelope field round-trips in server/Tests/Protocol/ProtocolTests.cs
- [ ] T008 Align SnapshotPayload schema to spec (memUsed + memTotal + window + aggregates) in agent/src/protocol.rs and server/Protocol/Messages.cs
- [ ] T009 Update snapshot encode/decode in agent/src/protocol.rs and server/Protocol/FrameCodec.cs to match the aligned SnapshotPayload schema
- [ ] T010 [P] Update snapshot round-trip tests in agent/tests/protocol_tests.rs and server/Tests/Protocol/ProtocolTests.cs to match the aligned SnapshotPayload schema
- [ ] T011 Replace backpressure pause/level semantics with `throttleDelayMs` (milliseconds) semantics in agent/src/protocol.rs and server/Protocol/Messages.cs
- [ ] T012 Update backpressure encode/decode logic in agent/src/protocol.rs and server/Protocol/FrameCodec.cs
- [ ] T013 [P] Update backpressure tests in agent/tests/protocol_tests.rs and server/Tests/Protocol/ProtocolTests.cs
- [ ] T014 [P] Update tests to assert `throttleDelayMs` interpretation consistently (0, small value, large value) in agent/tests/protocol_tests.rs and server/Tests/Protocol/ProtocolTests.cs

**Checkpoint**: Foundation ready — user story implementation can begin.

---

## Phase 3: User Story 1 - Ingest binary snapshots end-to-end (Priority: P1) 🎯 MVP

**Goal**: A runnable .NET Linux server accepts connections, performs handshake gating, ingests snapshots from a Rust agent, applies backpressure, and persists records via the storage interface without drops.

**Independent Test**: Run server + one agent; send handshake then snapshot(s); verify storage append count matches persisted snapshots and decoded fields are preserved.

### Tests for User Story 1 (MANDATORY)

- [ ] T015 [P] [US1] Add failing tests for segmented snapshot payload fields in agent/tests/protocol_tests.rs
- [ ] T016 [P] [US1] Add failing tests for segmented snapshot payload fields in server/Tests/Protocol/ProtocolTests.cs
- [ ] T017 [P] [US1] Add failing tests for server snapshot de-dup by messageId in server/Tests/Protocol/ProtocolTests.cs
- [ ] T018 [P] [US1] Add failing tests for server snapshot reassembly by snapshotId/partIndex/partCount in server/Tests/Protocol/ProtocolTests.cs
- [ ] T019 [P] [US1] Add failing tests for storage error propagation (nack on append failure) in server/Tests/Storage/FileStorageTests.cs
- [ ] T020 [P] [US1] Add deterministic tests asserting retry/backoff timing behavior in agent/tests/retry_backoff_tests.rs
- [ ] T021 [P] [US1] Add compression negotiation tests (on/off) and round-trip tests for compressed payloads in agent/tests/protocol_tests.rs and server/Tests/Protocol/ProtocolTests.cs
- [ ] T022 [P] [US1] Add `maxFrameBytes` enforcement tests (reject too-large frames) in agent tests and server/Tests/Protocol/ProtocolTests.cs

### Implementation for User Story 1

- [ ] T023 [P] [US1] Audit current agent collection coverage vs FR-001 (CPU, mem, top-N, all-process option) and document gaps in specs/001-protocol-messaging/research.md
- [ ] T024 [P] [US1] Add/align agent collection APIs used by protocol sender (top-N + all-process) in agent/src/lib.rs (or a dedicated module) and wire into agent/src/main.rs
- [ ] T025 [P] [US1] Add deterministic unit tests for collection result shaping (top-N selection, ordering, truncation determinism) in agent/tests/collection_tests.rs
- [ ] T026 [US1] Add snapshot segmentation fields (snapshotId, partIndex, partCount) to SnapshotPayload in agent/src/protocol.rs
- [ ] T027 [US1] Implement snapshot segmentation in agent encoder (split oversized all-process snapshots into parts) in agent/src/protocol.rs
- [ ] T028 [US1] Add snapshot segmentation fields (snapshotId, partIndex, partCount) to SnapshotPayload in server/Protocol/Messages.cs
- [ ] T029 [US1] Update server snapshot encode/decode for segmentation fields in server/Protocol/FrameCodec.cs
- [ ] T030 [US1] Add a structured HandshakeAck payload (server-selected compression + chosen protocol version) in agent/src/protocol.rs and server/Protocol/Messages.cs
- [ ] T031 [US1] Update handshake encode/decode for HandshakeAck payload in agent/src/protocol.rs and server/Protocol/FrameCodec.cs
- [ ] T032 [US1] Implement zstd compression/decompression gated by handshake capability + server selection in HandshakeAck in agent/src/protocol.rs and server/Protocol/FrameCodec.cs
- [ ] T033 [US1] Add supported protocol range fields to handshake (minVersion/maxVersion) in agent/src/protocol.rs and server/Protocol/Messages.cs
- [ ] T034 [US1] Change storage interface to surface append success/failure in server/Storage/IStorageWriter.cs
- [ ] T035 [US1] Update FileStorageWriter to return failure on append errors (do not suppress) in server/Storage/FileStorageWriter.cs
- [ ] T036 [US1] Update storage tests to reflect new storage semantics in server/Tests/Storage/FileStorageTests.cs
- [ ] T037 [US1] Create a runnable TCP server entrypoint in server/Program.cs and set OutputType=Exe in server/MonitoringServer.csproj
- [ ] T038 [US1] Implement per-connection session handling (handshake gating, decode loop, error handling) in server/Protocol/SessionHandler.cs
- [ ] T039 [US1] Implement server messageId de-duplication (per-connection) in server/Protocol/SessionHandler.cs
- [ ] T040 [US1] Implement server snapshot reassembly buffer keyed by snapshotId and persist only after full reassembly in server/Protocol/SessionHandler.cs
- [ ] T041 [US1] Implement ack-per-part behavior and nack/error responses on failures in server/Protocol/SessionHandler.cs
- [ ] T042 [US1] Implement snapshot persistence batching (flush by maxBatchSize/maxBatchDelayMs) before calling storage in server/Protocol/SessionHandler.cs
- [ ] T043 [P] [US1] Add tests for batching flush-by-size and flush-by-time behavior in server/Tests/Protocol/SessionHandlerTests.cs
- [ ] T044 [US1] Implement throttleDelayMs backpressure emission when server-side buffers exceed threshold in server/Protocol/SessionHandler.cs
- [ ] T045 [US1] Implement a minimal Rust agent sender that connects, handshakes, sends snapshots, and retries until ack (timeout/disconnect) in agent/src/main.rs
- [ ] T046 [US1] Verify retry/backoff defaults are documented in specs/001-protocol-messaging/quickstart.md and align wording with spec
- [ ] T047 [US1] Add an in-process server session integration test (handshake→segmented snapshot→persist) using in-memory streams and fake storage in server/Tests/Protocol/SessionHandlerTests.cs

**Checkpoint**: User Story 1 is functional and independently testable.

---

## Phase 4: User Story 2 - Version-safe protocol evolution (Priority: P2)

**Goal**: Extend protocol schema safely so older and newer agents can interoperate with the server without crashes or misinterpretation.

**Independent Test**: Simulate mixed versions (e.g., v1.0 and v1.1 with an extra optional field) sending to server; verify decode succeeds and unknown extensions are ignored or recorded.

### Tests for User Story 2 (MANDATORY)

- [ ] T048 [P] [US2] Add a Rust fixture that encodes a v1.1 snapshot with an appended optional field and verify v1.0 decode ignores it in agent/tests/protocol_tests.rs
- [ ] T049 [P] [US2] Add a .NET test that decodes a v1.1 snapshot containing an appended optional field and verifies base fields decode correctly in server/Tests/Protocol/ProtocolTests.cs
- [ ] T050 [P] [US2] Add handshake version negotiation tests (highest mutually supported) in agent/tests/protocol_tests.rs and server/Tests/Protocol/ProtocolTests.cs

### Implementation for User Story 2

- [ ] T051 [US2] Implement handshake version negotiation to highest mutually supported and return chosen version in HandshakeAck in server/Protocol/SessionHandler.cs
- [ ] T052 [US2] Update agent handshake to send supported range and adopt server-chosen version from HandshakeAck in agent/src/protocol.rs and agent/src/main.rs
- [ ] T053 [US2] Define an optional v1.1 extension field appended to SnapshotPayload and implement encode in agent/src/protocol.rs
- [ ] T054 [US2] Update server snapshot decode to ignore trailing bytes and optionally parse v1.1 extensions when present in server/Protocol/FrameCodec.cs

**Checkpoint**: Protocol evolution is safe and mixed-version operation is testable.

---

## Phase 5: User Story 3 - Health, errors, and diagnostics (Priority: P3)

**Goal**: Surface malformed frames, timeouts, and backpressure events via logs/counters and recover without crashing.

**Independent Test**: Inject corrupted frames and delayed acks; verify server emits protocol errors and stays alive, and agent retries/throttles while keeping heartbeats.

### Tests for User Story 3 (MANDATORY)

- [ ] T055 [P] [US3] Add tests for invalid message type / corrupt payload producing Error responses in server/Tests/Protocol/ProtocolTests.cs
- [ ] T056 [P] [US3] Add tests for heartbeat handling (send/receive) in agent/tests/protocol_tests.rs and server/Tests/Protocol/ProtocolTests.cs
- [ ] T057 [P] [US3] Add tests for throttleDelayMs backpressure affecting agent send loop timing in agent/tests/protocol_tests.rs

### Implementation for User Story 3

- [ ] T058 [US3] Implement protocol Error responses on decode/validation failures (without killing session when safe) in server/Protocol/SessionHandler.cs
- [ ] T059 [US3] Add lightweight counters for message counts, error types, and backpressure events in server/Protocol/SessionHandler.cs
- [ ] T060 [US3] Implement agent heartbeat emission and server heartbeat timeout detection in agent/src/main.rs and server/Protocol/SessionHandler.cs
- [ ] T061 [US3] Add lightweight agent-side counters/logging for retries, acks, and backpressure in agent/src/main.rs

**Checkpoint**: Diagnostics exist and failure modes are survivable.

---

## Phase 6: Polish & Cross-Cutting Concerns

- [ ] T062 [P] Verify “Security: plaintext dev-only” section exists in specs/001-protocol-messaging/quickstart.md and align wording with spec
- [x] T063 [P] Align protocol docs and contract field names/discriminants in specs/001-protocol-messaging/spec.md and specs/001-protocol-messaging/contracts/protocol-openapi.yaml (MessageType mapping 1–7, Backpressure uses throttleDelayMs not throttleLevel)
- [ ] T064 Enforce a runtime guardrail: server refuses to start in plaintext mode unless an explicit opt-in is set (e.g., env var) in server/Program.cs
- [ ] T065 [P] Add a test for the plaintext guardrail (startup/config test) in server/Tests/PlaintextGuardrailTests.cs
- [ ] T066 Run quickstart validation steps from specs/001-protocol-messaging/quickstart.md and ensure run-all-tests.sh passes
- [ ] T067 [P] Remove outdated at-most-once wording in agent/src/protocol.rs, server/Protocol/Messages.cs, and server/Storage/IStorageWriter.cs doc comments
- [ ] T068 [P] Add/verify intent + invariant comments for protocol framing/codec/session handling in agent/src/protocol.rs, server/Protocol/FrameCodec.cs, and server/Protocol/SessionHandler.cs
- [ ] T069 [P] Add a cross-language fixture interoperability test (Rust encodes → .NET decodes) in agent/tests/protocol_tests.rs and server/Tests/Protocol/InteropFixtureTests.cs

---

## Dependencies & Execution Order

### Phase Dependencies

- Setup (Phase 1) → Foundational (Phase 2) → US1 (Phase 3) → US2 (Phase 4) → US3 (Phase 5) → Polish (Phase 6)

### User Story Dependency Graph

- US1 (P1) is the MVP and should land first.
- US2 (P2) builds on the same message formats but can be developed after the Phase 2 codec/schema foundation is stable.
- US3 (P3) depends on the runnable session handler loop from US1.

Graph:

US1 → US2
US1 → US3

### Parallel Opportunities

- Any task marked [P] can be done in parallel (different files / minimal coupling).
- Within a user story: tests can be written in parallel; agent vs server changes can often proceed in parallel once the schema is agreed.

---

## Parallel Execution Examples

### User Story 1

- [P] Agent-side segmentation tests + implementation in agent/src/protocol.rs and agent/tests/protocol_tests.rs
- [P] Server-side reassembly tests + implementation in server/Protocol/FrameCodec.cs, server/Protocol/SessionHandler.cs, and server/Tests/Protocol/*
- [P] Storage failure propagation tests + IStorageWriter/FileStorageWriter updates in server/Storage/* and server/Tests/Storage/*

### User Story 2

- [P] Optional-field decode tests in server/Tests/Protocol/ProtocolTests.cs
- [P] Handshake negotiation tests in agent/tests/protocol_tests.rs

### User Story 3

- [P] Corrupt frame tests in server/Tests/Protocol/ProtocolTests.cs
- [P] Heartbeat tests in agent/tests/protocol_tests.rs

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1 + Phase 2
2. Complete Phase 3 (US1) end-to-end
3. Validate using run-all-tests.sh and the US1 independent test scenario

### Incremental Delivery

- Land US1 first, then add US2, then add US3, keeping each story independently testable.
