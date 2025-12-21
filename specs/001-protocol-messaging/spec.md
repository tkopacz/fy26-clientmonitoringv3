# Feature Specification: Binary Protocol & IO Core

**Feature Branch**: `001-protocol-messaging`  
**Created**: 2025-12-21  
**Status**: Draft  
**Input**: User description: "build specification for protocol and implement key components for rust and .net to send and receive messages"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Ingest binary snapshots end-to-end (Priority: P1)

An operator runs the .NET Linux server and receives binary monitoring
snapshots from deployed Rust agents (Windows/Linux) that include CPU,
memory, and top processes (or all processes when requested) and writes
them through the storage interface (initially append-to-file) without
drops.

**Why this priority**: Core value is reliable telemetry delivery; no
other functionality matters if ingest fails.

**Independent Test**: Deploy one agent against the server, send a
handshake and snapshots, verify storage append counts match sent
messages and decoding preserves fields.

**Acceptance Scenarios**:

1. **Given** the server is running and a fresh agent connects, **When**
   the agent sends handshake then a snapshot, **Then** the server
   acknowledges the handshake and persists the decoded snapshot via the
   storage interface.
2. **Given** a server backpressure window is triggered, **When** the
   agent sends additional snapshots, **Then** the server signals
   backpressure and the agent throttles without data loss.

---

### User Story 2 - Version-safe protocol evolution (Priority: P2)

A protocol maintainer extends the message schema (optional fields or new
message types) and both old and new agents can interact with the server
without crashes or mis-interpretation.

**Why this priority**: Enables safe rollout and backward compatibility
as protocol evolves.

**Independent Test**: Simulate mixed-version agents (v1, v1.1 with new
optional fields) sending to the server; verify decoding succeeds, extras
are ignored or recorded, and no process aborts.

**Acceptance Scenarios**:

1. **Given** an agent declares protocol version 1.1, **When** it sends a
   snapshot with an optional field, **Then** the server stores base
   fields and safely ignores or records the extension without failure.
2. **Given** an older agent declares protocol version 1.0, **When** the
   server is on version 1.1, **Then** the handshake completes and the
   server processes the known fields without rejecting the connection.

---

### User Story 3 - Health, errors, and diagnostics (Priority: P3)

An operator can see when agents fall behind or send malformed frames;
the system surfaces errors and recovers without crashing either side.

**Why this priority**: Observability and graceful failure keep the
system debuggable and reliable at scale.

**Independent Test**: Inject corrupted frames and delayed acks in a test
run; verify the agent retries or drops per policy, server logs and
increments error counters, and both remain alive.

**Acceptance Scenarios**:

1. **Given** the server receives a corrupted frame, **When** checksum or
   length validation fails, **Then** the server rejects that frame,
   emits a protocol error message, and keeps the connection alive if
   safe to do so.
2. **Given** the agent detects repeated backpressure signals, **When**
   threshold is met, **Then** the agent reduces send rate while still
   delivering periodic heartbeats to prove liveness.

### Edge Cases

- Handshake version mismatch between agent and server (e.g., agent newer
  than server) and the chosen downgrade/compatibility behavior.
- Oversized payloads when sending all processes (exceeding target 64 KB
  typical snapshot); how to truncate, segment, or reject.
- Partial frames or stream cuts (TCP half messages); need framing and
  reassembly rules.
- Backpressure signaling lost in-flight; agent over-sends before seeing
  signal.
- Storage append failures or rotation events while ingesting.
- Network latency spikes causing heartbeat timeouts and reconnects.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The agent MUST collect CPU usage, memory usage, and top-N
  processes by CPU and memory with an option to send all processes on
  request; macOS agents are out of scope.
- **FR-002**: A versioned binary protocol MUST define framing (length
  prefix and message type), envelope metadata (protocol version,
  platform, timestamps), and payload schemas for handshake, heartbeat,
  snapshot, error, and backpressure signals.
- **FR-003**: The protocol MUST support backward-compatible evolution
  via optional fields and version negotiation; unknown optional fields
  MUST be safely ignored or recorded without failing the session.
- **FR-004**: Handshake MUST include agent identity (instance id, OS
  type, agent version), protocol version, and capabilities (supports
  all-process option, compression if allowed) and receive server ack
  before snapshots are accepted.
- **FR-005**: Snapshots MUST include sampling window start/end,
  aggregated CPU/memory, and per-process entries (pid, name, cpu%, mem%/
  rss, optional command line when permitted by platform); ordering or
  truncation rules MUST be defined when payload exceeds size targets.
- **FR-006**: Transport sessions MUST include keepalive/heartbeat and
  backpressure signaling so the server can slow senders without
  disconnects.
- **FR-007**: Reliability semantics MUST implement at-most-once
  delivery: snapshot messages include ids for correlation, are not
  retried after a nack or timeout, and errors are surfaced via metrics
  and logs.
- **FR-008**: Transport MAY run plaintext for local/dev use only, with a
  configuration switch to enable TLS/mTLS in future iterations; default
  posture must document risks and prohibit plaintext in production
  deployments.
- **FR-009**: Payload size safeguards MUST apply zstd compression
  (level 3) when negotiated via capability flag, then truncate to top-N
  processes to fit within the 64 KB target; truncation rules must be
  deterministic and signaled in metadata when applied.
- **FR-010**: The server MUST decode, validate, and route messages
  through an ingestion pipeline that applies backpressure, batching, and
  validation before invoking the storage interface.
- **FR-011**: The storage layer MUST be accessed via an interface;
  initial implementation appends decoded records to a file with a
  documented rotation policy; business logic cannot depend on file I/O.
- **FR-012**: Observability MUST include structured logs and counters for
  message counts, error types, and backpressure events on both agent and
  server; diagnostics collection MUST be lightweight.
- **FR-013**: Tests MUST cover unit and combination flows: encode/decode
  round-trip per message type, handshake → snapshot → ack chain, and
  backpressure handling; mocks/fakes may be used to isolate transport or
  storage.

### Key Entities *(include if feature involves data)*

- **ProtocolEnvelope**: Contains version, message type, length, and
  envelope metadata (timestamps, platform, agent id).
- **AgentIdentity**: Agent instance id, OS type, agent version,
  capability flags (all-process allowed, compression if any).
- **SnapshotPayload**: Sampling window, aggregate CPU/memory, list of
  process samples (pid, name, cpu%, mem%/rss, optional cmdline, ordering
  by cpu%).
- **BackpressureSignal**: Server-to-agent message indicating throttle
  level or pause/resume instructions.
- **Ack/Nack**: Correlates to message ids; carries status and optional
  error code.
- **StorageRecord**: Canonical decoded record persisted through the
  storage interface; independent of storage medium.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Handshake + first snapshot end-to-end completes within
  2 seconds in 95% of attempts on a modest link (no packet loss).
- **SC-002**: With top-process mode, average snapshot payload ≤ 64 KB
  and decode success ≥ 99.5%; oversize handling follows defined policy
  without crashing.
- **SC-003**: Agent CPU overhead p95 ≤ 2% and memory ≤ 25 MB steady
  state while sending snapshots every 10 seconds.
- **SC-004**: Server sustains 1,000 concurrent agent sessions with <1%
  dropped/failed messages under backpressure and retains logs/metrics to
  diagnose any failures.
