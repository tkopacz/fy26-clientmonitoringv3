# Implementation Plan: Body-Only Message Compression

**Branch**: `003-compression-algorithms` | **Date**: 2026-02-27 | **Spec**: [/specs/003-compression-algorithms/spec.md](specs/003-compression-algorithms/spec.md)
**Input**: Feature specification from `/specs/003-compression-algorithms/spec.md`

## Summary

Implement body-only compression semantics across Rust and .NET protocol codecs so envelope/header bytes remain always readable before decompression. Keep compression opt-in via negotiated capability, enforce both on-wire and decompressed size caps, and preserve diagnostic metadata (`messageType`, `messageId`) for decompression failures.

## Technical Context

**Language/Version**: Rust (edition 2021, toolchain 1.92.0 in workspace) and .NET 8 (server projects targeting `net8.0`)  
**Primary Dependencies**: Rust: `zstd`, `crc32fast`; .NET: `ZstdSharp`, `Force.Crc32`  
**Storage**: Existing append-only storage interface in server (`IStorageWriter`/`FileStorageWriter`), no schema/storage change required  
**Testing**: `cargo test --package agent`; `dotnet test server/Tests/MonitoringServer.Tests.csproj`; cross-language fixture round-trip tests in existing protocol suites  
**Target Platform**: Rust agent on Windows/Linux and .NET server on Linux (dev runs also supported on macOS/Windows)  
**Project Type**: Dual-stack protocol library/runtime within existing Rust + .NET monorepo  
**Performance Goals**: Preserve header parse-before-decompress behavior and maintain existing target frame profile (`targetSnapshotBytes` ≈ 64 KiB); compression should reduce payload size for larger snapshots without increasing decode failure rate  
**Constraints**: Must keep message type discriminants 1-7 stable; keep frame format `[len:u32 BE][body][crc32:u32 LE]`; enforce both `maxFrameBytes=262144` and `maxDecompressedBodyBytes=262144`; compression only when both peers negotiate support; include protocol versioning decision and migration notes per constitution protocol-change governance  
**Scale/Scope**: Protocol changes limited to envelope/body processing and validation paths in `agent/src/protocol.rs`, `server/Protocol/FrameCodec.cs`, and associated tests/contracts

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Pre-Research Gate Review

- Minimal Footprint Agent (Rust, Windows/Linux): PASS — change is bounded to encode/decode logic, no additional sampling loops or blocking I/O.
- Scalable Linux .NET Server: PASS — server continues reading envelope first and can fail early on oversized/corrupt frames.
- Binary Protocol Contract: PASS — preserves versioned binary framing and discriminant parity while clarifying body-only compression boundary.
- Test Discipline (Unit + Combination/Integration): PASS — plan includes Rust, .NET, and cross-language interoperability tests.
- Storage Abstraction (Interface-first): PASS — no storage coupling changes.

### Post-Design Gate Review

- All constitution gates remain PASS after Phase 1 artifacts (`research.md`, `data-model.md`, `contracts/`, `quickstart.md`).
- No justified violations required.

## Project Structure

### Documentation (this feature)

```text
specs/003-compression-algorithms/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── body-compression-protocol.md
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
agent/
├── src/
│   └── protocol.rs
└── tests/
    └── protocol_tests.rs

server/
├── Protocol/
│   ├── Messages.cs
│   └── FrameCodec.cs
└── Tests/
    └── Protocol/
        └── ProtocolTests.cs

specs/
└── 003-compression-algorithms/
    ├── plan.md
    ├── research.md
    ├── data-model.md
    ├── quickstart.md
    └── contracts/
```

**Structure Decision**: Keep all implementation in existing protocol modules for both stacks and document interface semantics under `specs/003-compression-algorithms/contracts/`. No new runtime project boundaries are introduced.

## Phase Plan

### Phase 0 — Research

- Confirm current envelope-first decoding pattern in both stacks and codify body-only compression boundary.
- Define decompression guard strategy for explicit max decompressed body cap.
- Align negotiation behavior with existing capability bit (`CAP_COMPRESSION`).

### Phase 1 — Design & Contracts

- Model protocol entities and validation/state transitions for compression-enabled messages.
- Define contract for envelope fields, compression indicator, negotiation, and failure outcomes.
- Provide quickstart verification flow covering: negotiated compression, fallback when unsupported, malformed compressed payload, and size-cap rejection.

### Phase 2 — Task Planning (next command)

- Break implementation into Rust codec changes, .NET codec changes, and parity tests.

## Complexity Tracking

No constitution violations are required for this feature.
