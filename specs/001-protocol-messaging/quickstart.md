# Quickstart

## Prerequisites
- Rust 1.92.0 toolchain
- .NET SDK 8.0.122
- zstd library available for both stacks

## Steps
1. Build and test Rust agent:
   - `cd agent`
   - `cargo test`
2. Build and test .NET server:
   - `cd server`
   - `dotnet test`
3. Regenerate cross-language fixture after protocol changes:
   - `cd agent`
   - `cargo test -- test_writes_cross_language_fixture`
   - Copy/commit updated fixture consumed by .NET tests if applicable.
4. Run cross-language validation:
   - `cd server`
   - `dotnet test --filter CrossLanguageSnapshotDecode`
5. Optional end-to-end check:
   - Start server (`dotnet run` under `server`)
   - Run agent sample sending handshake + snapshot (add `RUST_LOG=info` for diagnostics)

## Notes
- Ensure `MessageType` discriminants remain aligned (1–7) across Rust and .NET when modifying protocol.rs or decoder logic.
- Keep the storage interface contract intact; append-to-file is the reference implementation.
- Backpressure and heartbeat semantics should remain stable across protocol version bumps.
