# Data Model

## Entities

### ProtocolEnvelope
- Fields: `version` (u16), `message_type` (u8 discriminant 1–7), `length` (u32), `timestamp_utc` (i64), `agent_id` (string/uuid), `platform` (enum: windows, linux), `capabilities` (bit flags: compression, all_processes).
- Relationships: Wraps one payload (Handshake, Snapshot, Heartbeat, Backpressure, Ack, Error).
- Validation: Length matches payload bytes; message_type within known range.

### Handshake
- Fields: `protocol_version` (u16), `agent_version` (string), `agent_id` (string/uuid), `os` (enum), `supports_all_processes` (bool), `supports_compression` (bool), `timestamp_utc` (i64).
- Validation: protocol_version >= minimum supported; agent_id non-empty.
- State: Must precede snapshots; acknowledged by server.

### Snapshot
- Fields: `window_start` (i64), `window_end` (i64), `cpu_total_pct` (f32), `mem_used_bytes` (u64), `mem_total_bytes` (u64), `processes` (list of ProcessSample), `truncated` (bool), `compression_applied` (bool).
- Relationships: Contains many ProcessSample entries.
- Validation: window_end >= window_start; process count > 0; totals consistent.
- State: Sent after successful handshake; may be throttled by backpressure.

### ProcessSample
- Fields: `pid` (u32), `name` (string), `cpu_pct` (f32), `mem_pct` (f32), `rss_bytes` (u64), `cmdline` (optional string), `rank` (u16 by cpu_pct).
- Validation: cpu_pct within 0–100; mem_pct within 0–100.

### Heartbeat
- Fields: `timestamp_utc` (i64), `agent_id` (string/uuid).
- Validation: none beyond schema; lightweight keepalive.

### Backpressure
- Fields: `level` (enum: none|reduce|pause), `reason` (string), `timestamp_utc` (i64).
- Validation: level known; reason optional text length cap (e.g., 256 chars).

### Ack
- Fields: `message_id` (u64), `status` (enum: ok|error), `error_code` (optional string), `timestamp_utc` (i64).
- Validation: message_id present; status known.

### ErrorFrame
- Fields: `code` (string), `detail` (string), `timestamp_utc` (i64).
- Validation: code non-empty.

## State Transitions
- Handshake → (Ack) → Snapshot* → (Ack/Backpressure) → Heartbeat as keepalive.
- Backpressure level dictates snapshot send rate; pause halts snapshots until lifted.
- Errors do not close session by default; agent may reconnect on repeated errors.
