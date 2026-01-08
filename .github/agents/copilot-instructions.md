# fy26-clientmonitoringv3 Development Guidelines

Auto-generated from all feature plans. Last updated: 2025-12-21

## Active Technologies
- Rust (agent) + .NET 8 (server) + Rust: `serde`, `bincode`, `zstd`, `chrono`; .NET: `ZstdSharp.Port`, `Microsoft.Extensions.Logging.Abstractions` (001-protocol-messaging)
- File append via storage interface (`server/Storage`) (001-protocol-messaging)
- Rust (toolchain pinned by repo; see quickstart), .NET 8 + Rust: Tokio (async IO), `bytes`, `crc32fast` (CRC32/ISO-HDLC), `zstd` (optional compression), `serde` (message model); .NET: .NET 8, async IO (Sockets/Pipelines), zstd binding as needed (001-protocol-messaging)
- Files (append-to-file via storage interface) (001-protocol-messaging)

- Rust 1.92.0 (agent); .NET 8.0.122 SDK (server) + Rust: bincode, serde, zstd (level 3); .NET: System.Text.Json, ZstdSharp (001-protocol-messaging)

## Project Structure

```text
src/
tests/
```

## Commands

cargo test [ONLY COMMANDS FOR ACTIVE TECHNOLOGIES][ONLY COMMANDS FOR ACTIVE TECHNOLOGIES] cargo clippy

## Code Style

Rust 1.92.0 (agent); .NET 8.0.122 SDK (server): Follow standard conventions

## Recent Changes
- 001-protocol-messaging: Added Rust (toolchain pinned by repo; see quickstart), .NET 8 + Rust: Tokio (async IO), `bytes`, `crc32fast` (CRC32/ISO-HDLC), `zstd` (optional compression), `serde` (message model); .NET: .NET 8, async IO (Sockets/Pipelines), zstd binding as needed
- 001-protocol-messaging: Added Rust (agent) + .NET 8 (server) + Rust: `serde`, `bincode`, `zstd`, `chrono`; .NET: `ZstdSharp.Port`, `Microsoft.Extensions.Logging.Abstractions`

- 001-protocol-messaging: Added Rust 1.92.0 (agent); .NET 8.0.122 SDK (server) + Rust: bincode, serde, zstd (level 3); .NET: System.Text.Json, ZstdSharp

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
