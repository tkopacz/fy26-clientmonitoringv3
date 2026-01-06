# Data Model

## Entities

### ProtocolEnvelope
- Fields: `version_major` (u8), `version_minor` (u8), `message_type` (u8 discriminant 1–7), `timestamp_utc` (i64), `agent_id` (string/uuid), `platform` (enum: windows, linux), `capabilities` (bit flags: compression, all_processes).
- Relationships: Wraps one payload (Handshake, Snapshot, Heartbeat, Backpressure, Ack, Error).
- Validation: Frame length prefix matches bytes read; message_type within known range.

### Handshake
- Fields: `min_version_major` (u8), `min_version_minor` (u8), `max_version_major` (u8), `max_version_minor` (u8), `agent_version` (string), `agent_id` (string/uuid), `os` (enum), `supports_all_processes` (bool), `supports_compression` (bool), `timestamp_utc` (i64).
- Validation: min_version <= max_version; agent_id non-empty.
- State: Must precede snapshots; acknowledged by server.

### HandshakeAck
- Fields: `chosen_version_major` (u8), `chosen_version_minor` (u8), `compression_selected` (bool), `timestamp_utc` (i64).
- Validation: chosen_version within the overlap of agent/server supported ranges.

### Snapshot
- Fields: `snapshot_id` (u64 or uuid), `window_start` (i64), `window_end` (i64), `cpu_total_pct` (f32), `mem_used_bytes` (u64), `mem_total_bytes` (u64), `processes` (list of ProcessSample).
- Segmentation fields (only when snapshot is segmented): `part_index` (u16, 0-based), `part_count` (u16, total parts).
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
- Fields: `throttle_delay_ms` (u32, 0 = none), `reason` (string), `timestamp_utc` (i64).
- Validation: throttle_delay_ms within a configured range; reason optional text length cap (e.g., 256 chars).

### Ack
- Fields: `message_id` (u64), `status` (enum: ok|error), `error_code` (optional string), `timestamp_utc` (i64).
- Validation: message_id present; status known.

### ErrorFrame
- Fields: `code` (string), `detail` (string), `timestamp_utc` (i64).
- Validation: code non-empty.

## State Transitions
- Handshake → (Ack) → Snapshot* → (Ack/Backpressure) → Heartbeat as keepalive.
- Backpressure throttle delay (milliseconds) dictates snapshot send rate.
- Errors do not close session by default; agent may reconnect on repeated errors.

## Segmentation & Acks
- Each snapshot part is sent as its own message/frame with a unique `message_id` (acked individually).
- All parts of a segmented snapshot share the same `snapshot_id` and include `part_index`/`part_count`.
- Server persists the snapshot once after full reassembly; server de-dupes on `message_id`.
