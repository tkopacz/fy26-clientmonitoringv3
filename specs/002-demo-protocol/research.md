# Research Notes

## Demo Protocol Decision Summary

This document captures the decisions needed to implement the demo protocol console apps in a way that exercises the real cross-language wire contract.

## Demo file format
- Decision: The Rust producer writes full protocol frame bytes to disk and the .NET consumer reads and decodes the same framing.
- Rationale: This validates the actual on-the-wire format (length + CRC + envelope + payload) and catches endian/CRC/layout mismatches.
- Alternatives considered: Writing only the message body (would skip framing/CRC); writing only the payload (would skip envelope/versioning).

## Framing + CRC details
- Decision: Frame layout is `[length:u32 big-endian][body bytes][crc32:u32 little-endian]`.
- Rationale: This matches the existing implementations:
  - Rust: `agent/src/protocol.rs` (`FrameCodec::encode` / `FrameCodec::decode`)
  - .NET: `server/Protocol/FrameCodec.cs` (`FrameCodec.Encode` / `FrameCodec.DecodeAsync`)
- Alternatives considered: Little-endian length prefix (would break the current .NET decoder).

## Message type
- Decision: Demo frames contain Snapshot messages only.
- Rationale: Snapshot is the most representative message for this repository and exercises mixed field types plus a collection (process list).
- Alternatives considered: Handshake-only (too trivial); handshake+snapshot (adds ordering/state that’s not needed for file-based demo).

## Protocol version
- Decision: Always encode using the latest supported protocol version for the repository (`ProtocolVersion::CURRENT` / `ProtocolVersion.Current`).
- Rationale: Keeps the demo aligned with current protocol code and avoids hard-coding an older version.
- Alternatives considered: Allow version override (useful later, but not required for the initial demo).

## Single-frame default, multi-frame option
- Decision: Rust defaults to writing exactly one frame, and provides an option to write multiple concatenated frames.
- Rationale: One frame is simplest for onboarding; multiple frames validates stream-like behavior and repeated decode.
- Alternatives considered: Always multi-frame (more moving parts for first-time users).

## Consumer behavior for multiple frames
- Decision: The .NET demo decodes and prints all frames until EOF and fails if the file ends with incomplete trailing bytes.
- Rationale: Mirrors streaming decode semantics and prevents silent data loss.
- Alternatives considered: Stop after first frame (would make the producer “multi-frame option” less useful); ignore trailing bytes (would hide corruption).

## Code placement
- Decision: Implement the Rust producer as a dedicated `agent` binary (`agent/src/bin/demo_protocol_producer.rs`) and the .NET consumer as a new console project under `server/` with a project reference to `MonitoringServer.csproj`.
- Rationale: Maximizes reuse of the existing codec and message models without introducing a new Rust workspace member or duplicating the protocol implementation in .NET.
- Alternatives considered: Separate Rust crate in workspace (more plumbing); a .NET standalone project duplicating protocol code (risk of drift).
