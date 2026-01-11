# Quickstart: Demo Protocol Console Apps

This demo creates a protocol file in Rust and reads it in .NET.

## Prerequisites

- Rust toolchain installed (`rustc`, `cargo`), version >= 1.75 (MSRV)
- .NET SDK installed (`dotnet`), version >= 8.0

Supported OS:

- Rust producer: Windows and Linux
- .NET consumer: Linux, Windows, and macOS (SDK permitting)

## 1) Generate a demo file (Rust)

Default: write one Snapshot frame to `tmp/demo-protocol.bin`.

```bash
cargo run -p agent --bin demo_protocol_producer
```

Optional: write multiple frames (example: 5) and a custom output path.

```bash
cargo run -p agent --bin demo_protocol_producer -- --out tmp/demo-protocol.bin --count 5
```

Notes:

- `--count n` writes exactly `n` concatenated frames (default `1`, allowed `1..=1000`).
- If your path contains spaces, quote it, e.g. `--out "tmp/demo protocol.bin"`.
- The canonical demo output is uncompressed (`compressed = false`).

## 2) Read and print the demo file (.NET)

```bash
dotnet run --project server/DemoProtocolConsumer -- --in tmp/demo-protocol.bin
```

Notes:

- If your path contains spaces, quote it, e.g. `--in "tmp/demo protocol.bin"`.

Expected behavior:

- Prints decoded Snapshot fields for each frame
- Exits non-zero for missing file, invalid CRC, invalid framing, or incomplete trailing bytes

Exit codes:

- `0`: Success
- `2`: Usage / invalid CLI args
- `10`: MissingFile
- `11`: EmptyFile
- `12`: InvalidFrame
- `13`: TrailingBytes
- `14`: CrcMismatch
- `15`: UnsupportedVersion
- `16`: FrameTooLarge

## Troubleshooting

- If the consumer reports a CRC mismatch, confirm the producer wrote `[len:u32 BE][body][crc32:u32 LE]` and computed CRC32 over `body` bytes only.
- If the consumer reports end-of-stream, the file may be truncated or include an incomplete trailing frame.
- If the consumer reports `TrailingBytes`, ensure the file contains only full frames (no extra bytes after the last frame).
- If the consumer reports `UnsupportedVersion`, regenerate the file with the current Rust producer (the demo always writes the repository’s current version).
