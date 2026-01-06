# Implementation Plan: Protocol Messaging & Cross-Language Compatibility

**Branch**: `001-protocol-messaging` | **Date**: 2025-12-21 | **Spec**: [/specs/001-protocol-messaging/spec.md](specs/001-protocol-messaging/spec.md)
**Input**: Feature specification from `/specs/001-protocol-messaging/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Align Rust agent and .NET server on the binary protocol contract (framing, envelopes, payloads) with cross-language encode/decode parity. Immediate defect: Rust `MessageType` serialization emits the wrong discriminant on the wire, causing .NET to decode Snapshot frames as Heartbeat. Fix enum serialization to use explicit 1–7 discriminants, regenerate cross-language fixtures, and validate via Rust and .NET tests.

## Technical Context

**Language/Version**: Rust 1.92.0 (agent); .NET 8.0.122 SDK (server)  
**Primary Dependencies**: Rust: bincode, serde, zstd (level 3); .NET: System.Text.Json, ZstdSharp  
**Storage**: File append via storage interface (initial implementation)  
**Testing**: cargo test (unit + integration), xUnit (.NET), cross-language fixture-based test  
**Target Platform**: Rust agent on Windows/Linux; .NET server on Linux  
**Project Type**: Dual projects (Rust crate `agent`, .NET server `MonitoringServer`)  
**Performance Goals**: Snapshot frame target ≤ 64 KB typical; handshake→first snapshot ≤ 2s p95; agent overhead ≤2% CPU p95 / ≤25 MB steady per constitution  
**Constraints**: At-least-once delivery for snapshots (retry until ack) with server de-dupe by messageId; backpressure via throttle level; compression optional via capability flag; segment oversized all-process snapshots; no macOS agent  
**Scale/Scope**: Thousands of concurrent agents; protocol evolution must remain backward compatible

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- Binary Protocol Contract: Length-prefixed framing, explicit message types, version negotiation, optional fields; cross-language parity required (enum discriminant fix). **Status: In scope**
- Minimal Footprint Agent: Rust, low overhead, compression optional, truncation for oversize frames. **Status: In scope**
- Scalable Linux .NET Server: Async decode pipeline, backpressure, ingestion via storage interface. **Status: In scope**
- Test Discipline: Unit + integration + cross-language fixture; keep failing deserialization test to prove fix. **Status: Must stay green**
- Storage Abstraction: File-append via interface; no business logic coupling. **Status: In scope**

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
agent/                # Rust agent crate
├── src/protocol.rs   # Protocol types, encode/decode
├── src/lib.rs
└── tests/protocol_tests.rs  # Integration + cross-language fixture writer

server/               # .NET server
├── MonitoringServer.csproj
├── Protocol/         # Message, envelope, codec
└── Tests/            # xUnit tests (ProtocolTests.cs)

run-all-tests.sh      # Orchestrates Rust + .NET test runs
```

**Structure Decision**: Dual-project layout (Rust agent crate + .NET server) with shared binary protocol contract validated via cross-language fixtures. Tests live alongside each project; integration tests under `agent/tests`, xUnit under `server/Tests`.

## Complexity Tracking

No constitution violations planned; no additional justification required.
