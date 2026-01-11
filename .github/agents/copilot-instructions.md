# fy26-clientmonitoringv3 Development Guidelines

Auto-generated from all feature plans. Last updated: 2025-12-21

## Active Technologies
- Rust (agent) + .NET 8 (server) + Rust: `serde`, `bincode`, `zstd`, `chrono`; .NET: `ZstdSharp.Port`, `Microsoft.Extensions.Logging.Abstractions` (001-protocol-messaging)
- File append via storage interface (`server/Storage`) (001-protocol-messaging)
- Rust (edition 2021; MSRV 1.75; local toolchain rustc 1.92.0) + .NET (projects target net8.0; local SDK 10.0.101) + Rust: `serde`, `bincode`, `zstd`, `crc32fast`; .NET: `ZstdSharp`/`ZstdSharp.Port`, `Force.Crc32` (002-demo-protocol)
- Local filesystem demo artifact in `tmp/` (no server/network) (002-demo-protocol)

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
- 002-demo-protocol: Added Rust (edition 2021; MSRV 1.75; local toolchain rustc 1.92.0) + .NET (projects target net8.0; local SDK 10.0.101) + Rust: `serde`, `bincode`, `zstd`, `crc32fast`; .NET: `ZstdSharp`/`ZstdSharp.Port`, `Force.Crc32`
- 001-protocol-messaging: Added Rust (agent) + .NET 8 (server) + Rust: `serde`, `bincode`, `zstd`, `chrono`; .NET: `ZstdSharp.Port`, `Microsoft.Extensions.Logging.Abstractions`

- 001-protocol-messaging: Added Rust 1.92.0 (agent); .NET 8.0.122 SDK (server) + Rust: bincode, serde, zstd (level 3); .NET: System.Text.Json, ZstdSharp

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
