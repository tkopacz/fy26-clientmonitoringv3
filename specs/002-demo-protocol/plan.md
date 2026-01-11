# Implementation Plan: Demo Protocol Console Apps

**Branch**: `002-demo-protocol` | **Date**: 2026-01-11 | **Spec**: [/specs/002-demo-protocol/spec.md](specs/002-demo-protocol/spec.md)
**Input**: Feature specification from `/specs/002-demo-protocol/spec.md`

## Summary

Create two small console apps demonstrating protocol interoperability via a file artifact:
1) Rust generates one (default) or many (optional) Snapshot protocol frames, prints a human-readable representation, and writes the exact framed bytes to disk.
2) .NET reads the file, decodes frames until EOF, prints decoded snapshots, and fails on invalid framing/CRC/incomplete trailing bytes.

This demo reuses the existing shared wire contract already implemented in Rust (`agent/src/protocol.rs`) and .NET (`server/Protocol/*`).

## Technical Context

**Language/Version**: Rust (edition 2021; MSRV 1.75; local toolchain rustc 1.92.0) + .NET (projects target net8.0; local SDK 10.0.101)  
**Primary Dependencies**: Rust: `serde`, `bincode`, `zstd`, `crc32fast`; .NET: `ZstdSharp`/`ZstdSharp.Port`, `Force.Crc32`  
**Storage**: Local filesystem demo artifact in `tmp/` (no server/network)  
**Testing**: `cargo test --package agent`; `dotnet test server/Tests/MonitoringServer.Tests.csproj` (+ add focused tests for multi-frame decode and demo frame generation helpers)  
**Target Platform**: Demo runs on developer workstation; Rust demo supports Linux/Windows; .NET demo runs on Linux/macOS/Windows (SDK permitting)  
**OS Scope (for success criteria)**: Producer is in-scope on Windows + Linux; consumer is in-scope on Linux + Windows + macOS (SDK permitting)
**Project Type**: Dual project (Rust crate + .NET solution), plus two demo entrypoints  
**Performance Goals**: Not performance-critical; keep demo deterministic and quick to run (<1s typical)  
**Constraints**: Offline-capable; file contains full protocol frames `[len:u32 BE][body][crc32:u32 LE]`; message type is Snapshot; version is latest supported by repo; consumer decodes all frames until EOF and fails on incomplete trailing bytes  
**Scale/Scope**: A single local file with 1..N frames for demo and debugging; no production ingestion, no networking

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*


- Minimal Footprint Agent (Rust, Windows/Linux): Demo logic is non-critical and lives in a dedicated binary entrypoint; no impact to runtime sampling loop.
- Scalable Linux .NET Server: Demo consumer is separate from server runtime; when reusing protocol decoding, it preserves streaming decode semantics.
- Binary Protocol Contract: Demo writes/reads the exact framed bytes using existing `FrameCodec` implementations; confirms versioning and field semantics.
- Test Discipline (Unit + Combination/Integration): Add small tests around demo frame generation and multi-frame decode behavior to keep regressions visible.
- Storage Abstraction: Not applicable; demo uses local file I/O only and does not change server storage interfaces.

## Project Structure

### Documentation (this feature)

```text
specs/002-demo-protocol/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)
```text
agent/
├── src/
│   ├── protocol.rs
│   ├── main.rs
│   └── bin/
│       └── demo_protocol_producer.rs   # NEW: writes demo frame(s) to tmp/
└── tests/
    └── protocol_tests.rs

server/
├── Protocol/
│   ├── FrameCodec.cs
│   └── Messages.cs
├── MonitoringServer.csproj
├── Tests/
│   └── MonitoringServer.Tests.csproj
└── DemoProtocolConsumer/               # NEW: console app reading file and decoding frames
    ├── DemoProtocolConsumer.csproj
    └── Program.cs

tmp/
└── demo-protocol.bin                   # DEFAULT output path (generated)
```

**Structure Decision**: Keep demos co-located with their language stacks:
- Rust producer is a dedicated `agent` binary (`agent/src/bin/…`) so it can reuse the Rust protocol types and `FrameCodec` without introducing a new workspace member.
- .NET consumer is a small console project under `server/` with a project reference to `MonitoringServer.csproj` so it can reuse `MonitoringServer.Protocol.FrameCodec` and message models.

## Phase Plan

### Phase 0 — Outline & Research (outputs: research.md)

- Confirm the exact on-disk artifact matches the real wire framing implemented in both stacks.
- Document choices: single frame default, multi-frame option, snapshot-only payload, latest protocol version.

### Phase 1 — Design & Contracts (outputs: data-model.md, contracts/*, quickstart.md)

- Describe the demo file format contract: endian layout, CRC calculation input, and where compression applies.
- Define the demo snapshot contents (deterministic field values and process list) sufficient to validate decoding.
- Provide quickstart commands to run Rust producer then .NET consumer.

### Phase 2 — Implementation Planning (not executed here)

- Implement the two console apps and add targeted tests:
  - Rust: a helper to build a deterministic `SnapshotPayload`; ensure `FrameCodec::encode` output decodes back.
  - .NET: a loop decoding frames until EOF; test multi-frame decode with a `MemoryStream` and known bytes.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No constitution violations are required for this feature.
