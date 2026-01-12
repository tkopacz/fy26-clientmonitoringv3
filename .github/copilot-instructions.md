# Copilot instructions

Ignore all markdown files in a-copilot-demos directory

## Big Picture
- Two halves: Rust agent (Windows/Linux) under [../agent/src](../agent/src) and .NET 8 server under [../server](../server); binary protocol connects them (bincode payloads, optional zstd level 3).
- Frames: `[len:u32 BE][payload]`; envelope carries version, message type (1–7), message ID, timestamp, compressed flag; keep size caps (target ~64 KiB, hard cap 256 KiB) aligned across stacks.
- Message types: Handshake → HandshakeAck, Heartbeat, Snapshot, Ack, Backpressure, Error. Maintain discriminants 1–7 consistently in [../agent/src/protocol.rs](../agent/src/protocol.rs) and [../server/Protocol/Messages.cs](../server/Protocol/Messages.cs).
- Version negotiation is major/minor compatible; capability flags cover all-process and compression; at-most-once delivery semantics; snapshot truncation is deterministic top-N with flag in metadata.
- Storage on the server is abstracted via [../server/Storage/IStorageWriter.cs](../server/Storage/IStorageWriter.cs) with file append impl [../server/Storage/FileStorageWriter.cs](../server/Storage/FileStorageWriter.cs); thread-safe via semaphore.

## Workflows & Commands
- Full test sweep: [../run-all-tests.sh](../run-all-tests.sh) (Rust first, then .NET). Use `--verbose` for full output.
- Rust tests: `cargo test --package agent`; unit tests live inline in [../agent/src/protocol.rs](../agent/src/protocol.rs), integration suite in [../agent/tests/protocol_tests.rs](../agent/tests/protocol_tests.rs) (covers version negotiation, encoding/decoding, compression, backpressure, size guards, cross-language fixture generation).
- .NET tests: from [../server](../server) run `dotnet test Tests/MonitoringServer.Tests.csproj`; filter `CrossLanguageSnapshotDecode` when validating the generated fixture.
- Demo file round-trip: [../run-demo-protocol.sh](../run-demo-protocol.sh) (producer writes, consumer reads). Under the hood: `cargo run -p agent --bin demo_protocol_producer -- --out <file> --count <n>` then `dotnet run --project server/DemoProtocolConsumer -- --in <file>`.
- Manual end-to-end: `dotnet run --project server/MonitoringServer.csproj` (listens on 8080), then `RUST_LOG=info cargo run` from [../agent](../agent) to send handshake + snapshots; inspect server logs and persisted files.
- Regenerate cross-language fixture after protocol changes: `cargo test -- test_writes_cross_language_fixture` in [../agent](../agent), then rerun .NET tests so [../server/Tests/Protocol/ProtocolTests.cs](../server/Tests/Protocol/ProtocolTests.cs) consumes the updated artifact.

## Conventions & Integration Details
- Keep zstd compression optional and signaled via the compressed flag; use level 3 on the agent and decompress only when flagged.
- Maintain discriminant alignment and schema parity; add new fields with backward-compatible defaults where possible; guard against oversized frames (>256 KiB) in both frame codec implementations ([../agent/src/protocol.rs](../agent/src/protocol.rs), [../server/Protocol/FrameCodec.cs](../server/Protocol/FrameCodec.cs)).
- Defaults to preserve: `ackTimeoutMs = 2000`, exponential backoff 500 → 30000 ms (no jitter), `targetSnapshotBytes = 65536`, `maxFrameBytes = 262144`, `throttleDelayMs` conveys backpressure (0 = none).
- Demo protocol files include len-prefix + CRC32 over body; consumer exits with specific codes (MissingFile, EmptyFile, InvalidFrame, TrailingBytes, CrcMismatch, UnsupportedVersion, FrameTooLarge) as documented in [../specs/002-demo-protocol/quickstart.md](../specs/002-demo-protocol/quickstart.md).
- Plaintext transport is for dev only; production must require explicit opt-in before starting in plaintext (see guardrail tasks).
- Toolchain expectations: Rust 1.92.0, .NET SDK 8.0.122, zstd available for both stacks.

## When Editing Protocol/Storage
- Update both Rust and C# definitions together; mirror validation logic (length checks, CRC/zstd handling, version compatibility) and keep message type discriminants stable.
- Extend tests first: add coverage in [../agent/tests/protocol_tests.rs](../agent/tests/protocol_tests.rs) and corresponding cases in [../server/Tests/Protocol](../server/Tests/Protocol); ensure cross-language fixture still decodes.
- Preserve storage contract (append-only writer) and semaphore-based concurrency in [../server/Storage/FileStorageWriter.cs](../server/Storage/FileStorageWriter.cs) when adding new sinks.
