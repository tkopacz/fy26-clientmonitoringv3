# Research Notes

## MessageType discriminants
- Decision: Encode `MessageType` as explicit numeric discriminants 1–7 aligned across Rust and .NET; serialize using `repr(u8)`/custom serde on Rust to emit the same values used by .NET.
- Rationale: Prior default serde/bincode layout emitted variant order that mismatched the .NET decoder, causing Snapshot frames to be read as Heartbeat.
- Alternatives considered: Adjust .NET decoder mapping (would break existing fixtures); switch to string enums (wasteful on wire).

## Compression default
- Decision: Keep zstd level 3 as the negotiated compression level when capability flag is set; otherwise send plain frames.
- Rationale: Level 3 is a balanced default and already used in both stacks; avoids new tuning risk for this fix.
- Alternatives considered: Higher levels (too slow); gzip (incompatible with current code paths).

## Snapshot size guardrails
- Decision: Retain 64 KB target for typical snapshot payloads; prefer truncating process list before encoding when estimated to exceed target, and log when truncation occurs.
- Rationale: Matches success criteria and keeps decode predictable; avoids fragmentation/segmentation work in this iteration.
- Alternatives considered: Splitting snapshots into multiple frames (larger change); raising limit (risk to backpressure).

## Fixture regeneration
- Decision: Regenerate the Rust-produced cross-language fixture after fixing `MessageType` serialization and rerun .NET test `CrossLanguageSnapshotDecode`.
- Rationale: Fixture currently encodes Snapshot with wrong discriminant; must reflect fixed wire contract to keep parity tests meaningful.
- Alternatives considered: Hand-edit fixture (error-prone); disable test (reduces safety net).
