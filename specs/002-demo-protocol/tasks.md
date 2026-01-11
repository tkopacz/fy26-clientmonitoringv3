# Tasks: Demo Protocol Console Apps

**Input**: Design documents from `/specs/002-demo-protocol/`
**Prerequisites**: plan.md, spec.md

**Tests**: Mandatory per the constitution. Include unit tests for new demo helpers and combination/integration-style tests for end-to-end decode flows (multi-frame, truncation, trailing bytes, max length).

**Organization**: Tasks are grouped by user story to enable independent implementation and verification of each story.

## Format: `- [ ] [TaskID] [P?] [Story?] Description with file path`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[US#]**: Which user story this task belongs to (US1, US2, US3)

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create the minimal project structure to implement the demos without affecting existing runtime code.

- [x] T001 Create new Rust demo binary entrypoint at agent/src/bin/demo_protocol_producer.rs
- [x] T002 Create new .NET console project folder at server/DemoProtocolConsumer/ with DemoProtocolConsumer.csproj and Program.cs
- [x] T003 Add server/DemoProtocolConsumer/DemoProtocolConsumer.csproj to fy26-clientmonitoringv3.sln

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared decisions and “source of truth” alignment that both demos depend on.

- [x] T004 [P] Confirm Rust demo uses the repository’s latest protocol version constant in agent/src/protocol.rs (no hard-coded version)
- [x] T005 [P] Confirm .NET demo uses the repository’s latest protocol version constant in server/Protocol/Messages.cs (no hard-coded version)
- [x] T006 [P] Define a single documented default demo file path `tmp/demo-protocol.bin` (ensure both binaries default to it) in agent/src/bin/demo_protocol_producer.rs and server/DemoProtocolConsumer/Program.cs
- [x] T007 Document demo framing expectations (len endian, CRC endian, CRC coverage) in specs/002-demo-protocol/quickstart.md (only if any behavior differs from current doc)

**Checkpoint**: Foundation ready — user story implementation can begin.

---

## Phase 3: User Story 1 - Produce a demo payload file (Rust) (Priority: P1) 🎯 MVP

**Goal**: Rust produces deterministic Snapshot frame bytes to disk and prints a human-readable summary.

**Independent Test**: Run `cargo run -p agent --bin demo_protocol_producer` and verify it prints payload details and writes `tmp/demo-protocol.bin` (non-empty, stable across runs).

- [x] T008 [US1] Implement CLI arg parsing (`--out`, `--count`) with defaults in agent/src/bin/demo_protocol_producer.rs
- [x] T009 [US1] Implement deterministic demo Snapshot payload builder (fixed values, includes a process list) in agent/src/bin/demo_protocol_producer.rs
- [x] T010 [US1] Implement deterministic envelope metadata for the demo (fixed message_id, fixed timestamp, fixed agent_id/platform, compressed flag choice) in agent/src/bin/demo_protocol_producer.rs
- [x] T011 [US1] Encode Snapshot message into full framed bytes `[len:u32 BE][body][crc32:u32 LE]` using FrameCodec in agent/src/bin/demo_protocol_producer.rs
- [x] T012 [US1] Write 1..N concatenated frames to the output file (default N=1) and ensure parent directories exist in agent/src/bin/demo_protocol_producer.rs
- [x] T013 [US1] Print a human-readable summary of the demo envelope + payload (per spec FR-014/FR-014a/FR-014b) and frame count + output path to stdout in agent/src/bin/demo_protocol_producer.rs
- [x] T014 [US1] Define version-stamp behavior for existing output file (predictable suffix + log final path) in agent/src/bin/demo_protocol_producer.rs
- [x] T015 [US1] Manual verification: run the producer per specs/002-demo-protocol/quickstart.md and confirm output file is created and stable across two runs

**Checkpoint**: Producer can generate a deterministic demo artifact on disk.

---

## Phase 4: User Story 2 - Consume and print a demo payload file (.NET) (Priority: P2)

**Goal**: .NET reads the Rust-produced file, decodes all frames until EOF, prints decoded Snapshot payload(s), and fails clearly on invalid input.

**Independent Test**: Run `dotnet run --project server/DemoProtocolConsumer -- --in tmp/demo-protocol.bin` and confirm decoded output matches the Rust producer’s printed values.

- [x] T016 [US2] Add project reference from server/DemoProtocolConsumer/DemoProtocolConsumer.csproj to server/MonitoringServer.csproj
- [x] T017 [US2] Implement CLI arg parsing (`--in`) with default path and clear usage text in server/DemoProtocolConsumer/Program.cs
- [x] T018 [US2] Implement file open + missing-file handling with error category + input path + reason (per spec FR-005a/FR-005b) and non-zero exit code in server/DemoProtocolConsumer/Program.cs
- [x] T019 [US2] Implement “decode frames until EOF” loop using server/Protocol/FrameCodec.cs from server/DemoProtocolConsumer/Program.cs
- [x] T020 [US2] Implement incomplete trailing bytes detection (error if EOF mid-frame) and propagate as error category + input path + reason (per spec FR-005a/FR-005b) + non-zero exit code in server/DemoProtocolConsumer/Program.cs
- [x] T021 [US2] Print decoded envelope + Snapshot payload fields (per spec FR-014/FR-014a/FR-014b) in a stable format (with frame index when multi-frame) in server/DemoProtocolConsumer/Program.cs
- [x] T022 [US2] Manual verification: run the consumer per specs/002-demo-protocol/quickstart.md against a producer-generated file and confirm it prints all frames and exits successfully
- [x] T023 [US2] Manual verification: truncate the input file and confirm the consumer fails with a clear end-of-stream / incomplete-frame error (non-zero exit)

**Checkpoint**: Consumer reliably decodes 1..N frames and rejects corrupted/truncated input.

---

## Phase 5: User Story 3 - Repeatable end-to-end demo (Priority: P3)

**Goal**: A developer can follow short instructions and complete the demo end-to-end quickly and offline.

**Independent Test**: Follow specs/002-demo-protocol/quickstart.md from a clean build and complete producer→consumer run within 5 minutes.

- [x] T024 [US3] Ensure quickstart commands match actual project/bin names and default paths in specs/002-demo-protocol/quickstart.md
- [x] T025 [US3] Ensure console outputs match spec print contract (FR-014/FR-014a/FR-014b) by refining output formatting in agent/src/bin/demo_protocol_producer.rs and server/DemoProtocolConsumer/Program.cs
- [x] T026 [US3] Add troubleshooting notes for common failures (CRC mismatch, truncated frame, unsupported version) in specs/002-demo-protocol/quickstart.md
- [x] T027 [US3] Manual verification: run the full quickstart flow and confirm produced vs consumed values match for all printed fields

**Checkpoint**: End-to-end demo is usable for onboarding/debugging.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Quality and maintenance tasks that affect multiple stories.

- [x] T028 [P] Run repository test suite and ensure no regressions: ./run-all-tests.sh (reference specs/002-demo-protocol/quickstart.md for demo commands)
- [x] T029 [P] Run formatting checks: cargo fmt --all and dotnet format (if configured) and ensure demo code matches repo style (files: agent/src/bin/demo_protocol_producer.rs, server/DemoProtocolConsumer/Program.cs)
- [x] T030 [P] Add a short note in README.md pointing to specs/002-demo-protocol/quickstart.md for the demo entrypoints

---

## Additional Test & Edge-Case Coverage (Required)

- [x] T031 [P] [US1] Add unit tests for deterministic payload/envelope generation in agent/tests/demo_protocol_producer_tests.rs (or extend agent/tests/protocol_tests.rs)
- [x] T032 [US1] Add combination test: encode demo frame(s) then decode with FrameCodec and assert envelope+payload values match expected constants in agent/tests/demo_protocol_producer_tests.rs
- [x] T033 [P] [US2] Add unit tests for decode loop behaviors (multi-frame success, truncation fails, empty file fails, trailing junk fails) in server/Tests/Protocol/
- [x] T034 [US2] Add unit test enforcing max frame length rejection (>256 KiB) in server/Tests/Protocol/
- [x] T035 [US2] Implement max length enforcement in server/DemoProtocolConsumer/Program.cs (reject frames >256 KiB with clear error)
- [x] T036 [US2] Manual verification: empty input file fails with clear error and non-zero exit
- [x] T037 [US2] Manual verification: valid frame + trailing junk byte(s) fails (non-zero exit, clear message)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — blocks story work
- **User Stories (Phase 3+)**: Depend on Foundational completion
- **Polish (Phase 6)**: Depends on desired user stories being complete

### User Story Dependencies

- **US1 (P1)**: Can start after Phase 2 — no dependencies on other stories
- **US2 (P2)**: Can start after Phase 2 — depends on US1 only for convenient manual verification (not for implementation)
- **US3 (P3)**: Depends on US1 + US2 being complete

---

## Parallel Execution Examples

### After Phase 2 (Foundational)

- Workstream A: Implement Rust producer in agent/src/bin/demo_protocol_producer.rs (US1)
- Workstream B: Scaffold .NET consumer in server/DemoProtocolConsumer/ (US2)

### Within US2

- CLI parsing + error messaging can be completed before wiring the full decode loop (all in server/DemoProtocolConsumer/Program.cs)

---

## Implementation Strategy

### MVP First (US1 Only)

1. Complete Phase 1 + Phase 2
2. Complete US1 (Phase 3)
3. Validate: run the producer twice and confirm byte stability + readable output

### Incremental Delivery

1. Add US2 (Phase 4) and validate decode against the generated file
2. Add US3 (Phase 5) and finalize quickstart + troubleshooting
