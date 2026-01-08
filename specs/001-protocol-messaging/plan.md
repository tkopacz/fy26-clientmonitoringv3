# Implementation Plan: Binary Protocol & IO Core

**Branch**: `001-protocol-messaging` | **Date**: 2026-01-08 | **Spec**: [specs/001-protocol-messaging/spec.md](specs/001-protocol-messaging/spec.md)
**Input**: Feature specification from [specs/001-protocol-messaging/spec.md](specs/001-protocol-messaging/spec.md)

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Deliver a compact, versioned binary protocol and IO core that enables Rust agents (Windows/Linux/macOS) to reliably send monitoring snapshots to a scalable Linux .NET server using at-least-once semantics (ack + retry) with server-side de-duplication by `messageId`, backpressure via `throttleDelayMs`, and optional zstd compression.

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->



**Language/Version**: Rust (toolchain pinned by repo; see quickstart), .NET 8  
**Primary Dependencies**: Rust: Tokio (async IO), `bytes`, `crc32fast` (CRC32/ISO-HDLC), `zstd` (optional compression), `serde` (message model); .NET: .NET 8, async IO (Sockets/Pipelines), zstd binding as needed  
**Storage**: Files (append-to-file via storage interface)  
**Testing**: `cargo test`, `dotnet test` (unit + combination/integration)  
**Target Platform**: Linux server (.NET 8); agents on Windows/Linux/macOS (Rust)  
**Project Type**: Multi-project repo (Rust agent + .NET server + shared protocol concepts)  
**Performance Goals**: Agent p95 CPU ≤ 2%, steady-state memory ≤ 25 MB; typical snapshot payload ≤ 64 KiB (top-N mode)  
**Constraints**: Frame format: 4-byte LE length + 4-byte LE CRC32 + envelope+payload; `maxFrameBytes = 262144`; deterministic retry/backoff (no jitter)  
**Scale/Scope**: Server targets 1,000+ concurrent sessions; supports batching + backpressure


## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- ✅ Minimal Footprint Agent: Rust client supports Windows, Linux, and macOS. Capability-gated support is allowed (macOS may omit all-process/cmdline based on platform/permissions; must advertise via handshake).
- ✅ Scalable Linux .NET Server: Server runs on Linux using .NET 8 with async IO, batching, and backpressure.
- ✅ Binary Protocol Contract: Compact, versioned binary framing with CRC32 validation before decode; explicit discriminants; version negotiation; optional fields.
- ✅ Test Discipline: Unit + combination/integration tests (encode/decode round-trips, handshake → snapshot → ack, backpressure).
- ✅ Storage Abstraction: Storage access via interface; initial append-to-file implementation.

## Project Structure

### Documentation (this feature)

```text
specs/001-protocol-messaging/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)
<!--
  ACTION REQUIRED: Replace the placeholder tree below with the concrete layout
  for this feature. Delete unused options and expand the chosen structure with
  real paths (e.g., apps/admin, packages/something). The delivered plan must
  not include Option labels.
-->

```text
agent/                      # Rust monitoring agent
├── src/
└── tests/

server/                     # .NET server + protocol + storage
├── MonitoringServer.csproj
├── Protocol/
└── Storage/

server/Tests/                # .NET tests
specs/001-protocol-messaging/ # This feature spec + design docs
```

**Structure Decision**: Multi-project repository (Rust agent + .NET server) with protocol contract enforced by shared spec and cross-language tests.

## Phased Plan

### Phase 0: Research (complete)

- See [specs/001-protocol-messaging/research.md](specs/001-protocol-messaging/research.md) for key protocol decisions (discriminants, retry semantics, segmentation, backpressure, compression).

### Phase 1: Design & Contracts (complete, aligned)

- Data model: [specs/001-protocol-messaging/data-model.md](specs/001-protocol-messaging/data-model.md)
- Interop/diagnostics contract: [specs/001-protocol-messaging/contracts/protocol-openapi.yaml](specs/001-protocol-messaging/contracts/protocol-openapi.yaml)
- Quickstart: [specs/001-protocol-messaging/quickstart.md](specs/001-protocol-messaging/quickstart.md)

### Phase 2: Implementation Planning (next)

- Implement/verify framing + CRC32 validation (Rust + .NET)
- Implement handshake negotiation + capability flags (all-process, cmdline included, compression)
- Implement snapshot encode/decode + segmentation + reassembly
- Implement ack/retry + server-side de-dupe window (per-session)
- Implement backpressure emission + agent throttle application
- Ensure tests: unit + integration parity + cross-language fixture regeneration

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |
