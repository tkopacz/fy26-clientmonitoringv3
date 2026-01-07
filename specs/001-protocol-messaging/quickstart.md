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
- Backpressure uses `throttleDelayMs` (milliseconds; 0 = no throttle) and segmented snapshot parts are acked individually; keep these semantics stable across protocol version bumps.

## Security: plaintext dev-only

- Plaintext transport is allowed for local/dev only.
- Production deployments MUST NOT run plaintext.
- The server MUST refuse to start in plaintext mode unless explicitly opted-in (guardrail; see tasks).

## Defaults (deterministic)

- Retry policy: `ackTimeoutMs = 2000`; exponential backoff 500 → 30000ms; no jitter.
- Size guardrails: `targetSnapshotBytes = 65536` (64 KiB typical payload target); `maxFrameBytes = 262144` (256 KiB hard cap).
