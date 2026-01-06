# Implementation Plan: Protocol Messaging & Cross-Language Compatibility

**Branch**: `001-protocol-messaging` | **Date**: 2026-01-06 | **Spec**: [/specs/001-protocol-messaging/spec.md](specs/001-protocol-messaging/spec.md)
**Input**: Feature specification from `/specs/001-protocol-messaging/spec.md`

## Summary

Implement and validate a compact, versioned binary protocol shared by the Rust agent and .NET server, including: explicit length-prefixed framing, version negotiation, handshake gating, optional zstd compression (level 3), segmented all-process snapshots with reassembly, at-least-once snapshot delivery (retry until ack) with server de-duplication by `messageId`, and throttle-level backpressure signaling. Ensure cross-language parity via encode/decode round-trips and fixture-based interoperability tests.

## Technical Context

**Language/Version**: Rust (agent) + .NET 8 (server)  
**Primary Dependencies**: Rust: `serde`, `bincode`, `zstd`, `chrono`; .NET: `ZstdSharp.Port`, `Microsoft.Extensions.Logging.Abstractions`  
**Storage**: File append via storage interface (`server/Storage`)  
**Testing**: `cargo test --package agent`; `dotnet test server/Tests/MonitoringServer.Tests.csproj`; cross-language fixture interoperability  
**Target Platform**: Rust agent on Windows/Linux; .NET server on Linux  
**Project Type**: Dual project (Rust crate + .NET server)  
**Performance Goals**: Typical top-process snapshot ≤ 64 KB; handshake→first snapshot ≤ 2s p95; agent overhead p95 ≤ 2% CPU and ≤ 25 MB memory steady state  
**Constraints**: At-least-once snapshot delivery with server de-dupe by `messageId`; handshake negotiates highest mutually supported protocol version; optional zstd compression (level 3) via capability flag; oversized all-process snapshots are segmented (common `snapshotId`, part index/count) and each part is acked; backpressure is expressed as a throttle level; plaintext transport is local/dev only (no production plaintext)  
**Scale/Scope**: Support 1,000 concurrent agent sessions; protocol evolution must be backward compatible via optional fields and versioning

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- Minimal Footprint Agent (Rust, Windows/Linux): Sampling and encoding are bounded (size targets, segmentation, optional compression); avoid blocking I/O on critical paths.
- Scalable Linux .NET Server: Use async I/O and an ingestion pipeline with backpressure and batching; avoid large allocations (streaming decode).
- Binary Protocol Contract: Explicit framing and message types; version negotiation; optional fields for evolution; includes platform + sampling timestamps.
- Test Discipline (Unit + Combination/Integration): Unit tests per message; integration tests for handshake→snapshot→ack and cross-language fixture parity.
- Storage Abstraction: Storage stays behind an interface; file-append remains a replaceable implementation.

## Project Structure

### Documentation (this feature)

```text
specs/001-protocol-messaging/
├── plan.md              # This file (/speckit.plan output)
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
agent/
├── src/
│   ├── lib.rs
│   ├── main.rs
│   └── protocol.rs
└── tests/
    └── protocol_tests.rs

server/
├── MonitoringServer.csproj
├── Protocol/
│   ├── FrameCodec.cs
│   └── Messages.cs
├── Storage/
│   ├── FileStorageWriter.cs
│   └── IStorageWriter.cs
└── Tests/
    ├── MonitoringServer.Tests.csproj
    ├── Protocol/
    │   └── ProtocolTests.cs
    └── Storage/
        └── FileStorageTests.cs

run-all-tests.sh
```

**Structure Decision**: Keep protocol encode/decode logic in each language stack, validated by shared discriminants and fixture-based tests.

## Complexity Tracking

No constitution violations are required for this feature.
