# Research Notes

## Protocol Design Decision Summary

This document captures the key design decisions that shaped the binary protocol specification. All decisions below are reflected in the spec clarifications (see `spec.md` Clarifications section) and functional requirements.

## MessageType discriminants
- Decision: Encode `MessageType` as explicit numeric discriminants 1–7 aligned across Rust and .NET; serialize using `repr(u8)`/custom serde on Rust to emit the same values used by .NET.
- Rationale: Prior default serde/bincode layout emitted variant order that mismatched the .NET decoder, causing Snapshot frames to be read as Heartbeat.
- Alternatives considered: Adjust .NET decoder mapping (would break existing fixtures); switch to string enums (wasteful on wire).

## Version negotiation
- Decision: During handshake, negotiate the highest mutually supported protocol version and proceed; if there is no overlapping supported version, fail handshake with an explicit error.
- Rationale: Enables safe rollouts and mixed-version fleets without brittle coupling.
- Alternatives considered: Reject all mismatches (operationally painful); force either server or agent version (creates one-sided compatibility burden).

## Snapshot delivery semantics
- Decision: Snapshot delivery is at-least-once: the agent retries on timeout/disconnect until an ack is received; the server de-dupes by `messageId` so storage does not double-count.
- Rationale: Matches “without drops” goals while keeping implementation practical.
- Alternatives considered: At-most-once (may drop snapshots under faults); exactly-once (requires significantly more state/coordination).

## Compression default
- Decision: Keep zstd level 3 as the negotiated compression level when capability flag is set; otherwise send plain frames.
- Rationale: Level 3 is a balanced default and already used in both stacks; avoids new tuning risk for this fix.
- Alternatives considered: Higher levels (too slow); gzip (incompatible with current code paths).

## Snapshot size guardrails
- Decision: Retain 64 KiB `targetSnapshotBytes = 65536` for typical snapshot payloads and enforce `maxFrameBytes = 262144` (256 KiB hard cap). If an all-process snapshot is still oversized after negotiated zstd compression, segment it into multiple parts that share a common `snapshotId` and include part index/count for reassembly.
- Rationale: Preserves “all processes” functionality without silent data loss while keeping typical snapshots small.
- Alternatives considered: Truncation only (drops process data); raising size limit (risks memory/backpressure); rejecting oversize (breaks “all processes” use case).

## Backpressure signaling
- Decision: Server-to-agent backpressure is expressed as `throttleDelayMs` (unsigned integer milliseconds; 0 = no throttle) that the agent applies to its snapshot send rate.
- Rationale: Simple to implement and test across Rust and .NET and avoids complex credit systems.
- Alternatives considered: Pause/resume (more state transitions); credit-based flow control (more protocol state).

## Fixture regeneration
- Decision: Regenerate the Rust-produced cross-language fixture after fixing `MessageType` serialization and rerun .NET test `CrossLanguageSnapshotDecode`.
- Rationale: Fixture currently encodes Snapshot with wrong discriminant; must reflect fixed wire contract to keep parity tests meaningful.
- Alternatives considered: Hand-edit fixture (error-prone); disable test (reduces safety net).

## Alignment with Spec Clarifications

All decisions above are consistent with the clarifications recorded in `spec.md` (Session 2026-01-06):
- ✅ Snapshot delivery semantics (at-least-once + server de-dupe)
- ✅ Version negotiation (highest mutually supported)
- ✅ Oversize snapshot handling (segmentation with snapshotId + reassembly)
- ✅ Backpressure signaling (throttleDelayMs numeric)
- ✅ Segmented snapshot acks (per-part acknowledgment)
