# Feature Specification: Demo Protocol Console Apps

**Feature Branch**: `002-demo-protocol`  
**Created**: 2026-01-11  
**Status**: Draft  
**Input**: User description: "build a sample CONSOLE app that will 1. Serialize on rust. print data, save to FILE 2. Deserialize in .NET, print data"

## Clarifications

### Session 2026-01-11

- Q: What should the Rust app write to the file (i.e., what exact byte format is the “demo artifact”)? → A: Full protocol frame bytes (length prefix + CRC + encoded envelope + payload).
- Q: Should the demo file contain one frame or multiple frames? → A: Default to exactly one full frame; provide an option to write multiple concatenated frames.
- Q: Which protocol message type should the demo frame(s) contain? → A: Snapshot message only.
- Q: Which protocol version should the Rust demo app encode in the snapshot frame? → A: Always write the latest supported protocol version.
- Q: When the input file contains multiple concatenated full frames, what should the .NET app do? → A: Decode and print all frames until EOF; fail if trailing bytes don’t form a full frame.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Produce a demo payload file (Rust) (Priority: P1)

A developer runs a small Rust console application that creates a representative sample snapshot payload, serializes it into the repository’s protocol format, prints a human-readable summary to the console, and writes the serialized bytes to a file on disk.

**Why this priority**: A deterministic “producer” artifact is the foundation for cross-language validation and repeatable demos.

**Independent Test**: Run the Rust console app; confirm it prints the payload and writes a file that is non-empty and stable across runs.

**Acceptance Scenarios**:

1. **Given** a clean working directory, **When** the Rust demo app is run, **Then** it prints the payload fields to stdout and writes a single output file at a documented path.
2. **Given** an existing output file, **When** the Rust demo app is run again, **Then** it MUST version-stamp the output file in a predictable way (e.g., `demo-protocol-001.bin`, `demo-protocol-002.bin`) and clearly log the final path.
3. **Given** the user requests multiple frames, **When** the Rust demo app is run, **Then** it writes multiple full protocol frames concatenated in a single file and logs how many frames were written.

---

### User Story 2 - Consume and print a demo payload file (.NET) (Priority: P2)

A developer runs a .NET console application that reads the file produced by the Rust demo app, deserializes it, and prints the decoded envelope + snapshot payload fields to the console in a stable format per FR-014.

**Why this priority**: Validates that the protocol can be produced and consumed across languages, and gives a simple repro for debugging encoding issues.

**Independent Test**: Run the .NET demo app against a known-good file; confirm it prints decoded values and indicates successful parsing.

**Acceptance Scenarios**:

1. **Given** a valid file produced by the Rust demo app, **When** the .NET demo app is run, **Then** it prints the decoded fields and exits successfully.
2. **Given** a missing input file, **When** the .NET demo app is run, **Then** it prints a clear error message and exits with a non-success code.
3. **Given** a file containing multiple concatenated full frames, **When** the .NET demo app is run, **Then** it decodes and prints all frames until EOF and fails if the file ends with an incomplete trailing frame.
4. **Given** an empty input file, **When** the .NET demo app is run, **Then** it prints a clear error message and exits with a non-success code.
5. **Given** a file containing at least one full valid frame followed by extra trailing bytes that do not form a complete valid frame, **When** the .NET demo app is run, **Then** it fails with a clear error message and exits with a non-success code.

---

### User Story 3 - Repeatable end-to-end demo (Priority: P3)

A developer can follow short instructions to run the Rust producer and then the .NET consumer (in either order with the file in place) and visually verify that the printed decoded values match what was produced.

**Why this priority**: Makes the demo usable for onboarding, troubleshooting, and PR validation.

**Independent Test**: Follow the documented steps to run both apps and compare output for at least one known payload.

**Acceptance Scenarios**:

1. **Given** a fresh clone and the required build tools installed, **When** a developer follows the documented demo steps, **Then** they can run producer then consumer successfully within 5 minutes.

---

### Edge Cases

- Input file exists but is empty.
- Input file is truncated or has extra trailing bytes.
- Input file contains multiple frames and ends with an incomplete trailing frame.
- Payload contains unexpected values (very large numbers, empty strings, non-ASCII text).
- Consumer reads a file produced by a different protocol version than it supports.
- File paths that include spaces or non-ASCII characters.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The project MUST include a Rust console application that creates a demo payload and serializes it using the repository’s protocol encoding rules.
- **FR-002**: The Rust demo app MUST print the payload it is about to serialize (human-readable) and MUST write the serialized bytes to a file on disk.
- **FR-003**: The project MUST include a .NET console application that reads the serialized file, deserializes it using the same protocol rules, and prints the decoded payload (human-readable).
- **FR-004**: Both apps MUST use a documented, deterministic file naming and location convention so the consumer can be pointed at the producer output without guesswork.
- **FR-004a**: File path handling MUST be consistent:
	- If a relative path is provided, it is interpreted relative to the current working directory.
	- The producer MUST create parent directories for the output path if they do not exist.
	- Both apps MUST accept paths containing spaces and non-ASCII characters.
	- Both apps MUST log the resolved input/output path they used.
- **FR-005**: When deserialization fails (invalid file, unsupported version, corrupt data), the .NET demo app MUST emit an error message and exit with a non-success status.
- **FR-005c**: Exit codes MUST be deterministic:
	- `0`: Success
	- `2`: Usage / invalid CLI args
	- `10`: `MissingFile`
	- `11`: `EmptyFile`
	- `12`: `InvalidFrame`
	- `13`: `TrailingBytes`
	- `14`: `CrcMismatch`
	- `15`: `UnsupportedVersion`
	- `16`: `FrameTooLarge`
- **FR-006**: The canonical demo Snapshot MUST include at least: numeric fields (`i64`, `u64`, `f32`), at least one string containing non-ASCII characters, and a process collection with at least 2 entries; at least one process MUST include an optional `cmdline`, and at least one MUST omit it.
- **FR-007**: The demo output MUST be stable enough that a developer can visually compare “produced” vs “consumed” values and recognize mismatches.
- **FR-008**: The demo MUST not require any network connectivity or server process; file-based interchange is the only required integration point.
- **FR-009**: The Rust demo app MUST write a complete, self-contained protocol frame to disk (the same byte sequence that would be sent over the wire): length prefix + CRC + encoded envelope + payload.
- **FR-010**: The Rust demo app MUST default to writing exactly one full protocol frame to the output file, and MUST provide an option to write multiple full frames concatenated in the same file.
- **FR-010a**: The Rust producer CLI MUST support:
	- `--out <path>`: output file path (default: `tmp/demo-protocol.bin`)
	- `--count <n>`: number of frames to write (default: `1`, allowed: `1..=1000`)
	- Semantics: `--count n` writes exactly `n` concatenated frames.
- **FR-011**: The demo frame payload type MUST be a Snapshot message.
- **FR-012**: The Rust demo app MUST encode the snapshot frame using the latest protocol version supported by the repository.
- **FR-012a**: “Latest supported protocol version” MUST be sourced from the repository constants:
	- Rust: `ProtocolVersion::CURRENT`
	- .NET: `ProtocolVersion.Current`
- **FR-013**: The .NET demo app MUST decode concatenated frames until EOF and print each decoded snapshot; it MUST fail if any trailing bytes remain that do not constitute a complete valid frame.
- **FR-013a**: “Invalid framing” failures MUST be distinguishable:
	- `InvalidFrame`: invalid/unsupported frame structure or an invalid length prefix.
	- `InvalidFrame`: premature EOF while reading `length`, `body`, or `crc32`.
	- `TrailingBytes`: EOF reached after at least one successful frame decode, but remaining bytes do not form a complete valid frame.
- **FR-013b**: Unsupported protocol versions MUST be detected from the decoded envelope version:
	- If `version_major`/`version_minor` does not match the current supported version, the consumer MUST fail with category `UnsupportedVersion`, exit code `15`, and a reason that includes both the found version and the expected version.
- **FR-014**: Both apps MUST print the same canonical field set for comparison:
	- Envelope: `version_major`, `version_minor`, `message_type`, `message_id`, `timestamp_utc_ms`, `agent_id`, `platform`, `compressed`
	- Snapshot payload: `window_start_secs`, `window_end_secs`, `total_cpu_percent`, `memory_used_bytes`, `memory_total_bytes`, `process_count`, and for each process: `pid`, `name`, `cpu_percent`, `memory_percent`, `memory_bytes`, `cmdline` (present/absent), plus `truncated`.
- **FR-014a**: Printed output MUST list fields in the order defined in FR-014, and MUST include a frame index label when multiple frames are present.
- **FR-014b**: Printed output MUST be stable and culture-invariant: numeric values are base-10; floating-point values use `.` as the decimal separator and are printed with exactly 3 fractional digits.
- **FR-014c**: When printing multiple frames, the output MUST clearly separate frames using the prefix `Frame <index>:` where `<index>` is 1-based.
- **FR-005a**: Error output MUST include: an error category, the input file path, and a human-readable reason.
- **FR-005b**: Error categories MUST be one of: `MissingFile`, `EmptyFile`, `InvalidFrame`, `TrailingBytes`, `CrcMismatch`, `UnsupportedVersion`, `FrameTooLarge`.
- **FR-015**: The canonical demo output MUST be deterministic: the producer MUST use fixed values for all encoded envelope fields (including `message_id` and `timestamp_utc_ms`) and a fixed Snapshot payload so the emitted bytes are stable across repeated runs on the same platform.
- **FR-016**: The producer MUST NOT emit, and the consumer MUST reject, any frame whose `length` exceeds 256 KiB (262,144 bytes); the consumer MUST exit non-successfully with a clear error.
- **FR-017**: The canonical demo producer MUST write uncompressed payloads (`compressed = false`) for the demo file.

Assumptions and dependencies:

- The developer has the necessary build tools installed for Rust and .NET.
- OS support expectations:
	- Rust producer: Windows and Linux.
	- .NET consumer: Linux, Windows, and macOS (SDK permitting).
- Both apps have permission to read/write the demo file path on the local filesystem.

### Key Entities *(include if feature involves data)*

- **DemoPayload**: A representative snapshot object used solely for validating cross-language serialization and readability; contains a mix of values and identifiers.
- **SnapshotPayload**: The concrete snapshot message instance encoded into the demo frame(s) and decoded by the .NET consumer.
- **SerializedDemoFile**: The bytes written by the Rust demo app and consumed by the .NET demo app; contains a complete protocol frame (length + CRC + envelope + payload), including enough metadata to identify format/version.
- **ProtocolVersion**: The version identifier needed for the consumer to decide whether it can parse the file.
- **LatestSupportedProtocolVersion**: The protocol version the Rust demo uses by default; the current “latest supported” for the repo.
- **ConsoleOutput**: The human-readable rendering of the payload used to compare producer and consumer results.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A developer can generate a demo file and successfully decode it using the other app in under 5 minutes following the documented steps.
- **SC-002**: For the canonical demo file, the consumer prints values that match the producer output exactly for all fields printed per FR-014.
- **SC-003**: For a missing file and a corrupted file, the consumer provides a clear failure reason and exits non-successfully 100% of the time.
- **SC-004**: The demo can be run offline (no network dependency) and produces the same serialized file bytes for the canonical payload across repeated runs on the same platform.
