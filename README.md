# Protocol Implementation

Binary protocol for agent-server communication with:
- **Agent**: Rust (Windows/Linux)
- **Server**: .NET 8 C# (Linux)
- **Encoding**: bincode-compatible binary format
- **Compression**: zstd level 3 (negotiated via capability flags)
- **Reliability**: At-most-once delivery semantics
- **Storage**: Interface-based with file append implementation

## Project Structure

```
fy26-clientmonitoringv3/
├── agent/                      # Rust agent (Windows/Linux)
│   ├── src/
│   │   ├── lib.rs             # Library exports
│   │   ├── main.rs            # Agent binary entry point
│   │   └── protocol.rs        # Protocol definitions, encoding, framing
│   └── Cargo.toml
├── server/                     # .NET server (Linux)
│   ├── Protocol/
│   │   ├── Messages.cs        # Protocol message types
│   │   └── FrameCodec.cs      # Frame encoding/decoding
│   ├── Storage/
│   │   ├── IStorageWriter.cs  # Storage interface
│   │   └── FileStorageWriter.cs # File-based implementation
│   ├── Tests/
│   │   ├── Protocol/
│   │   │   └── ProtocolTests.cs
│   │   └── Storage/
│   │       └── FileStorageTests.cs
│   └── MonitoringServer.csproj
└── Cargo.toml                 # Rust workspace
```

## Message Types

1. **Handshake** (agent → server): Identity, version, capabilities
2. **HandshakeAck** (server → agent): Handshake confirmation
3. **Heartbeat**: Keepalive
4. **Snapshot**: CPU, memory, process list
5. **Ack**: Message acknowledgment
6. **Backpressure**: Throttle signal
7. **Error**: Error notification

## Wire Format

```
Frame: [4-byte length (big-endian)][payload]
Payload: bincode-serialized Message (optionally zstd-compressed)

Message:
  - Envelope (version, type, ID, timestamp, compressed flag)
  - Payload (variant based on message type)
```

## Running Tests

### Rust (agent)
```bash
cd /home/tkopacz/fy26-clientmonitoringv3
cargo test --package agent
```

### .NET (server)
```bash
cd /home/tkopacz/fy26-clientmonitoringv3/server
dotnet test Tests/MonitoringServer.Tests.csproj
```

## Demo Protocol (Rust → file → .NET)

See [specs/002-demo-protocol/quickstart.md](specs/002-demo-protocol/quickstart.md) for the end-to-end demo:

- Rust producer: `cargo run -p agent --bin demo_protocol_producer`
- .NET consumer: `dotnet run --project server/DemoProtocolConsumer`

## Key Features

- **Version negotiation**: Major/minor compatibility checking
- **Capability flags**: All-process mode, compression support
- **Size constraints**: Max 256 KB uncompressed, target 64 KB compressed
- **Truncation**: Deterministic top-N process selection with metadata flag
- **Storage abstraction**: Interface allows future backend swapping
- **Thread safety**: File storage writer uses semaphore for concurrent access
- **Professional documentation**: Inline comments for functions and modules
