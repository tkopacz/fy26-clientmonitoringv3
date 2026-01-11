# Data Model

## Entities

### DemoProtocolFile
- Representation: A byte stream stored on disk.
- Structure: 1..N concatenated Frames (default N=1).
- Validation:
  - Reader processes frames sequentially until EOF.
  - Any incomplete frame at EOF is an error.

### Frame
- Layout: `[length:u32 big-endian][body:length bytes][crc32:u32 little-endian]`
- Fields:
  - `length`: number of bytes in `body`.
  - `body`: encoded Envelope + payload bytes (payload may be zstd-compressed depending on envelope flag).
  - `crc32`: CRC32 of `body` bytes only.
- Validation:
  - `length <= 256 KiB` (max frame size safeguard; normative constraint for this demo).
  - `crc32(body) == crc32`.

### Envelope
- Fields (in order):
  - `version_major` (u8)
  - `version_minor` (u8)
  - `message_type` (u8; Snapshot is 4)
  - `message_id` (16 bytes)
  - `timestamp_utc_ms` (i64 little-endian)
  - `agent_id` (string)
  - `platform` (u8 enum: Windows=1, Linux=2)
  - `compressed` (bool-like byte: 0/1)
- Validation:
  - `message_type` is recognized.
  - Version is the repository’s current supported version.

### SnapshotPayload
- Fields (in order):
  - `window_start_secs` (i64 little-endian)
  - `window_end_secs` (i64 little-endian)
  - `total_cpu_percent` (f32 little-endian)
  - `memory_used_bytes` (u64 little-endian)
  - `memory_total_bytes` (u64 little-endian)
  - `process_count` (u64 little-endian)
  - `processes` (ProcessSample * process_count)
  - `truncated` (bool-like byte: 0/1)
- Validation:
  - `window_end_secs >= window_start_secs`.
  - Percent fields are within 0..=100 for the demo payload.

### ProcessSample
- Fields (in order):
  - `pid` (u32 little-endian)
  - `name` (string)
  - `cpu_percent` (f32 little-endian)
  - `memory_percent` (f32 little-endian)
  - `memory_bytes` (u64 little-endian)
  - `cmdline` (optional string)

## Notes

- Strings follow the shared custom encoding already implemented in both stacks (see Rust `write_string`/`read_string` and .NET `WriteString`/`ReadString`).
- Compression (zstd) applies to payload bytes only when `Envelope.compressed == true`; the envelope itself is not compressed.
