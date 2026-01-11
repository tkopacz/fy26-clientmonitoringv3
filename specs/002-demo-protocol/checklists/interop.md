# Interop Checklist: Demo Protocol Console Apps

**Purpose**: Validate that the requirements for the Rust→file→.NET demo are complete, clear, consistent, and measurable ("unit tests for English", not implementation tests).
**Created**: 2026-01-11
**Feature**: ../spec.md

## Requirement Completeness

- [x] CHK001 Are the Rust producer CLI inputs fully specified (output path, frame count default, and allowed ranges)? [Completeness, Spec §FR-004, Spec §FR-010, Quickstart §1]
- [x] CHK002 Is the Rust producer’s output-file collision behavior explicitly specified (overwrite vs version-stamp) and consistent with the acceptance scenarios? [Clarity, Spec §User Story 1 / Acceptance Scenario 2]
- [x] CHK003 Are the exact bytes written to disk defined as a complete protocol frame (including endianness and CRC coverage), not just “serialized payload”? [Completeness, Spec §FR-009, Contract §Frame, Contract §CRC32]
- [x] CHK004 Are the rules for multi-frame files explicitly specified as pure frame concatenation (no delimiter/headers), including default of exactly one frame? [Completeness, Spec §FR-010, Contract §Multiple Frames]
- [x] CHK005 Is the demo message type scope explicitly limited to Snapshot only, including whether any future expansion is intentionally out of scope? [Completeness, Spec §FR-011, Contract §Compatibility]
- [x] CHK006 Is “latest supported protocol version” concretely defined so an implementer can identify it unambiguously (e.g., a named constant or source of truth)? [Clarity, Spec §FR-012]
- [x] CHK007 Are the producer’s “print data” requirements specific about which Snapshot fields must be printed (and whether envelope metadata must also be printed)? [Gap, Spec §FR-002, Spec §FR-007]
- [x] CHK008 Are the consumer’s “print data” requirements specific about which decoded fields must be printed and how multiple frames are separated/labelled? [Gap, Spec §FR-003, Spec §FR-013, Quickstart §2]
- [x] CHK009 Are deterministic demo-payload requirements strong enough to guarantee stable output bytes across repeated runs (including message_id/timestamps if present)? [Gap, Spec §SC-004, Spec §FR-007, Data Model §Envelope]
- [x] CHK010 Is the max frame size / `length` limit specified in requirements (or explicitly documented as an assumption) to prevent ambiguous oversized input handling? [Gap, Data Model §Frame]

## Requirement Clarity

- [x] CHK011 Is the framing definition unambiguous about endianness (`len` big-endian; CRC little-endian) and about what exactly is CRC’d (body only)? [Clarity, Spec §FR-009, Contract §Frame, Contract §CRC32]
- [x] CHK012 Is “fail on invalid framing” defined in a way that distinguishes (a) invalid length, (b) premature EOF, and (c) trailing non-frame bytes? [Clarity, Spec §FR-005, Spec §FR-013, Contract §Multiple Frames]
- [x] CHK013 Is the “decode until EOF” requirement explicit about success conditions (EOF at frame boundary) vs error conditions (EOF mid-frame)? [Clarity, Spec §FR-013, Contract §Multiple Frames]
- [x] CHK014 Is payload compression behavior for the demo explicitly specified (always uncompressed vs optional compressed frames), including how it should appear in console output? [Ambiguity, Data Model §Envelope, Contract §Body]
- [x] CHK015 Are numeric field constraints and formatting requirements specified (e.g., float formatting precision) so “visual compare” is objective and repeatable? [Gap, Spec §FR-007, Spec §SC-002]

## Requirement Consistency

- [x] CHK016 Do the spec’s framing requirements match the contract exactly (field order, endianness, CRC input), without conflicting statements elsewhere? [Consistency, Spec §FR-009, Contract §Frame]
- [x] CHK017 Are the quickstart commands consistent with the required deterministic file naming/location convention? [Consistency, Spec §FR-004, Quickstart §1, Quickstart §2]
- [x] CHK018 Are multi-frame requirements consistent between spec and contract (concatenation model, decode semantics, error on incomplete trailing bytes)? [Consistency, Spec §FR-010, Spec §FR-013, Contract §Multiple Frames]

## Acceptance Criteria Quality

- [x] CHK019 Does the spec define which fields constitute an “exact match” between producer and consumer output (so SC-002 is objectively verifiable)? [Measurability, Spec §SC-002, Spec §FR-007]
- [x] CHK020 Is the “<5 minutes” success criterion grounded in explicit prerequisites and a bounded number of steps (so it’s not environment-dependent or ambiguous)? [Clarity, Spec §SC-001, Quickstart §Prerequisites]
- [x] CHK021 Does the determinism outcome define the scope (same OS only vs cross-OS) and acceptable sources of variance (timestamps, GUIDs, float formatting)? [Ambiguity, Spec §SC-004]

## Scenario Coverage

- [x] CHK022 Are requirements present for the default single-frame run and for the multi-frame option (including how the count maps to frames written)? [Coverage, Spec §FR-010, Spec §User Story 1 / Acceptance Scenario 3]
- [x] CHK023 Are requirements present for consuming a file with multiple frames, including how each frame is identified in output? [Coverage, Spec §FR-013, Spec §User Story 2 / Acceptance Scenario 3]
- [x] CHK024 Are missing-file requirements complete (error message content + exit code semantics, not just “fails”)? [Gap, Spec §FR-005, Spec §User Story 2 / Acceptance Scenario 2]

## Edge Case Coverage

- [x] CHK025 Is the empty-file behavior specified (message + exit code) and consistent with the broader “invalid file” requirements? [Gap, Spec §Edge Cases, Spec §FR-005]
- [x] CHK026 Is truncated input specified with enough detail to avoid ambiguous handling (truncate at length, within body, or within CRC)? [Completeness, Spec §Edge Cases, Spec §FR-013, Contract §Multiple Frames]
- [x] CHK027 Are “extra trailing bytes” requirements explicit about whether the consumer must reject any non-frame trailing bytes vs ignore them? [Clarity, Spec §FR-013]
- [x] CHK028 Are path edge cases specified (spaces, non-ASCII) in a way that results in explicit CLI and documentation requirements? [Coverage, Spec §Edge Cases, Spec §FR-004]
- [x] CHK029 Is unsupported protocol version behavior specified (how the consumer detects it, error text expectations, and exit status)? [Gap, Spec §Edge Cases, Spec §FR-005, Spec §FR-012]

## Non-Functional Requirements

- [x] CHK030 Is the “offline only” requirement explicit enough to exclude accidental network dependencies (telemetry, lookups, downloads), and is it reflected in quickstart/troubleshooting? [Completeness, Spec §FR-008]
- [x] CHK031 Are platform/OS support expectations stated for both demos (at least which OSes are “in scope” for the success criteria)? [Gap, Plan §Target Platform]

## Dependencies & Assumptions

- [x] CHK032 Are build/tool prerequisites bounded with minimum versions (or an explicit “latest” policy) to keep the quickstart reproducible? [Gap, Spec §Assumptions and dependencies, Quickstart §Prerequisites]
- [x] CHK033 Are output file location assumptions specified (working directory, relative paths, directory creation expectations, permissions)? [Gap, Spec §FR-004]

## Ambiguities & Conflicts

- [x] CHK034 Is the printed console output contract defined strongly enough that two independent implementations will converge (field order, labels, units), avoiding subjective “looks comparable”? [Ambiguity, Spec §FR-007, Spec §SC-002]
