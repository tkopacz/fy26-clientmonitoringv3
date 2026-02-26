# Tasks: Body-Only Message Compression

**Input**: Design documents from `/specs/003-compression-algorithms/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Protocol tests are included because the feature specification defines independent test criteria per user story and measurable outcomes.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Align implementation surfaces and verification entry points before codec changes

- [ ] T001 Confirm baseline protocol test entry points in `agent/tests/protocol_tests.rs` and `server/Tests/Protocol/ProtocolTests.cs`
- [ ] T002 Document feature task scope and execution checkpoints in `specs/003-compression-algorithms/quickstart.md`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared protocol guardrails that all stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T003 Add/align shared size guard constants (`maxFrameBytes`, `maxDecompressedBodyBytes`) in `agent/src/protocol.rs`
- [ ] T004 [P] Add/align shared size guard constants (`MaxFrameBytes`, `MaxDecompressedBodyBytes`) in `server/Protocol/FrameCodec.cs`
- [ ] T005 [P] Ensure protocol error metadata structure carries `messageType` and `messageId` for decode failures in `server/Protocol/Messages.cs`

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Parse and route messages without decompressing headers (Priority: P1) 🎯 MVP

**Goal**: Keep envelope/header decode path independent from body decompression for compressed and uncompressed frames

**Independent Test**: Decode one compressed and one uncompressed frame and verify header metadata is available before body decode/decompression is attempted.

### Tests for User Story 1

- [ ] T006 [P] [US1] Add Rust test proving envelope fields are parsed before payload decompression in `agent/tests/protocol_tests.rs`
- [ ] T007 [P] [US1] Add .NET protocol test proving envelope fields are parsed before payload decompression in `server/Tests/Protocol/ProtocolTests.cs`

### Implementation for User Story 1

- [ ] T008 [US1] Refactor Rust decode flow to parse envelope header before conditional body decompression in `agent/src/protocol.rs`
- [ ] T009 [US1] Refactor .NET decode flow to parse envelope header before conditional body decompression in `server/Protocol/FrameCodec.cs`
- [ ] T010 [US1] Ensure compressed flag controls body-only handling (never header bytes) in Rust codec paths in `agent/src/protocol.rs`
- [ ] T011 [US1] Ensure compressed flag controls body-only handling (never header bytes) in .NET codec paths in `server/Protocol/FrameCodec.cs`

**Checkpoint**: User Story 1 is fully functional and testable independently

---

## Phase 4: User Story 2 - Negotiate compression safely across versions (Priority: P2)

**Goal**: Use compression only when both peers advertise capability support while preserving identical header parsing behavior

**Independent Test**: Verify mutual-support session uses compressed bodies and mixed-support session remains uncompressed with no decompression attempt.

### Tests for User Story 2

- [ ] T012 [P] [US2] Add Rust negotiation test for mutual-support vs mixed-support compression behavior in `agent/tests/protocol_tests.rs`
- [ ] T013 [P] [US2] Add .NET negotiation test validating no decompression when compression is not negotiated in `server/Tests/Protocol/ProtocolTests.cs`

### Implementation for User Story 2

- [ ] T014 [US2] Enforce sender-side compressed body emission only when `CAP_COMPRESSION` is mutually enabled in `agent/src/protocol.rs`
- [ ] T015 [US2] Enforce receiver-side no-decompression path when compression negotiation is disabled in `agent/src/protocol.rs`
- [ ] T016 [US2] Enforce receiver-side no-decompression path when compression negotiation is disabled in `server/Protocol/FrameCodec.cs`
- [ ] T017 [US2] Keep message type discriminants and envelope decode parity unchanged while applying negotiation gating in `server/Protocol/Messages.cs`

**Checkpoint**: User Stories 1 and 2 work independently

---

## Phase 5: User Story 3 - Fail safely on malformed compressed bodies (Priority: P3)

**Goal**: Surface explicit decompression and size-cap errors with envelope identity metadata while preserving stream/frame stability

**Independent Test**: Inject invalid compressed payload and decompression-size overflow; verify explicit errors include message identity and subsequent valid frames still decode.

### Tests for User Story 3

- [ ] T018 [P] [US3] Add Rust test for malformed compressed body error with preserved `messageType`/`messageId` in `agent/tests/protocol_tests.rs`
- [ ] T019 [P] [US3] Add Rust test for decompressed size-cap rejection (`maxDecompressedBodyBytes`) in `agent/tests/protocol_tests.rs`
- [ ] T020 [P] [US3] Add .NET test for malformed compressed body error metadata and continued frame processing in `server/Tests/Protocol/ProtocolTests.cs`
- [ ] T021 [P] [US3] Add .NET test for decompressed size-cap rejection (`MaxDecompressedBodyBytes`) in `server/Tests/Protocol/ProtocolTests.cs`

### Implementation for User Story 3

- [ ] T022 [US3] Implement explicit decompression-failure and decompressed-size-cap error outcomes in Rust decode path in `agent/src/protocol.rs`
- [ ] T023 [US3] Implement explicit decompression-failure and decompressed-size-cap error outcomes in .NET decode path in `server/Protocol/FrameCodec.cs`
- [ ] T024 [US3] Preserve envelope identity metadata (`messageType`, `messageId`) in server error messages for body-related failures in `server/Protocol/Messages.cs`
- [ ] T025 [US3] Ensure frame boundary integrity after body decode failure in .NET loop behavior tests in `server/Tests/Protocol/DemoProtocolDecodeLoopTests.cs`

**Checkpoint**: All user stories are independently functional

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final parity validation, protocol governance artifacts, and measurable performance validation across stories

- [ ] T026 [P] Update compression behavior contract details and record rationale in `specs/003-compression-algorithms/contracts/body-compression-protocol.md`
- [ ] T027 Run full Rust and .NET protocol validation commands from `specs/003-compression-algorithms/quickstart.md`
- [ ] T028 [P] Regenerate and validate cross-language protocol fixture compatibility in `agent/tests/protocol_tests.rs` and `server/Tests/Protocol/ProtocolTests.cs`
- [ ] T029 Add protocol versioning decision record (version bump or no-bump rationale) in `specs/003-compression-algorithms/research.md`
- [ ] T030 Add migration notes for mixed-version rollout and operator impact in `specs/003-compression-algorithms/quickstart.md`
- [ ] T031 [P] Add Rust measurement test for SC-002 corpus size reduction (payloads ≥ 4 KiB) in `agent/tests/protocol_tests.rs`
- [ ] T032 [P] Add .NET measurement test for SC-002 corpus size reduction (payloads ≥ 4 KiB) in `server/Tests/Protocol/ProtocolTests.cs`
- [ ] T033 Compare compressed vs baseline failure-rate delta (≤ 0.1 percentage points) and document results in `specs/003-compression-algorithms/quickstart.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - blocks all user stories
- **User Stories (Phase 3+)**: Depend on Foundational completion
  - **US1 (P1)** should ship first as MVP
  - **US2 (P2)** depends on US1 decode-path stability
  - **US3 (P3)** depends on US1/US2 decode and negotiation behavior
- **Polish (Phase 6)**: Depends on all implemented stories

### User Story Dependencies

- **US1 (P1)**: Starts after Foundational; no dependency on other user stories
- **US2 (P2)**: Starts after US1 establishes stable header/body boundary behavior
- **US3 (P3)**: Starts after US1 and US2 define decompression and negotiation paths

### Within Each User Story

- Tests are added before or alongside implementation and must pass before story completion
- Decode/control-flow changes in codec files precede error/metadata refinements
- Story-level checkpoint must pass before moving to the next priority

### Parallel Opportunities

- T004 and T005 can run in parallel in Phase 2
- US1 tests (T006, T007) can run in parallel
- US2 tests (T012, T013) can run in parallel
- US3 tests (T018–T021) can run in parallel
- Polish tasks T026 and T028 can run in parallel after implementation
- Performance measurement tasks T031 and T032 can run in parallel after core implementation

---

## Parallel Example: User Story 1

```bash
Task: "T006 [US1] Add Rust test proving envelope fields are parsed before payload decompression in agent/tests/protocol_tests.rs"
Task: "T007 [US1] Add .NET protocol test proving envelope fields are parsed before payload decompression in server/Tests/Protocol/ProtocolTests.cs"
```

## Parallel Example: User Story 2

```bash
Task: "T012 [US2] Add Rust negotiation test for mutual-support vs mixed-support compression behavior in agent/tests/protocol_tests.rs"
Task: "T013 [US2] Add .NET negotiation test validating no decompression when compression is not negotiated in server/Tests/Protocol/ProtocolTests.cs"
```

## Parallel Example: User Story 3

```bash
Task: "T018 [US3] Add Rust malformed compressed body error test in agent/tests/protocol_tests.rs"
Task: "T020 [US3] Add .NET malformed compressed body error metadata test in server/Tests/Protocol/ProtocolTests.cs"
Task: "T021 [US3] Add .NET decompressed size-cap rejection test in server/Tests/Protocol/ProtocolTests.cs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1 (Setup)
2. Complete Phase 2 (Foundational)
3. Complete Phase 3 (US1)
4. Validate US1 independently (header parse before decompression)
5. Demo/review before adding negotiation and failure hardening

### Incremental Delivery

1. Deliver US1 (header/body boundary)
2. Deliver US2 (safe capability negotiation)
3. Deliver US3 (malformed-body and dual-cap failure handling)
4. Finish with Polish phase parity checks and docs updates

### Parallel Team Strategy

1. Team aligns on Phase 1 and Phase 2 together
2. Then split by concern:
   - Rust path owner: `agent/src/protocol.rs` + Rust tests
   - .NET path owner: `server/Protocol/FrameCodec.cs` + .NET tests
   - Contract/test orchestration owner: protocol docs + cross-language fixture checks
