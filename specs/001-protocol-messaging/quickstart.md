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

## Local Dev Run (TCP server + agent sender)

To manually test the end-to-end flow:

1. **Terminal 1 - Start the server**:
   ```bash
   cd server
   dotnet run --project MonitoringServer.csproj
   ```
   The server will listen on `localhost:8080` (or configured port) for incoming agent connections.

2. **Terminal 2 - Run the agent**:
   ```bash
   cd agent
   RUST_LOG=info cargo run
   ```
   The agent will connect to the server, perform handshake, and send snapshots at the configured interval.

3. **Verify**:
   - Check server logs for handshake acknowledgment and snapshot receipt
   - Check agent logs for successful acks and any backpressure signals
   - Inspect storage output files (default: `./data/snapshots_*.bin`) to confirm persistence

4. **Test backpressure** (optional):
   - Configure a low buffer threshold on the server to trigger throttling
   - Observe agent adjusting its send rate according to `throttleDelayMs` values

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
