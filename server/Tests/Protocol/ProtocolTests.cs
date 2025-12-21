/// <summary>
/// Unit tests for protocol message encoding and decoding.
/// 
/// Tests cover:
/// - Version compatibility
/// - Message type conversion
/// - Encode/decode round-trips for all message types
/// - Compression handling
/// - Frame size validation
/// </summary>

using MonitoringServer.Protocol;
using Xunit;
using Xunit.Sdk;

namespace MonitoringServer.Tests.Protocol;

public class ProtocolTests
{
    [Fact]
    public void ProtocolVersion_Compatibility_SameVersion()
    {
        var v1_0 = new ProtocolVersion { Major = 1, Minor = 0 };
        Assert.True(v1_0.IsCompatibleWith(v1_0));
    }

    [Fact]
    public void ProtocolVersion_Compatibility_HigherMinor()
    {
        var v1_0 = new ProtocolVersion { Major = 1, Minor = 0 };
        var v1_1 = new ProtocolVersion { Major = 1, Minor = 1 };

        // Higher minor compatible with lower
        Assert.True(v1_1.IsCompatibleWith(v1_0));
        // Lower minor NOT compatible with higher
        Assert.False(v1_0.IsCompatibleWith(v1_1));
    }

    [Fact]
    public void ProtocolVersion_Compatibility_DifferentMajor()
    {
        var v1_0 = new ProtocolVersion { Major = 1, Minor = 0 };
        var v2_0 = new ProtocolVersion { Major = 2, Minor = 0 };

        // Different major NOT compatible
        Assert.False(v2_0.IsCompatibleWith(v1_0));
        Assert.False(v1_0.IsCompatibleWith(v2_0));
    }

    [Fact]
    public void AgentIdentity_Capabilities_AllProcess()
    {
        var identity = new AgentIdentity
        {
            InstanceId = "test-001",
            OsType = OsType.Linux,
            AgentVersion = "0.1.0",
            ProtocolVersion = ProtocolVersion.Current,
            Capabilities = AgentIdentity.CapAllProcess
        };

        Assert.True(identity.SupportsAllProcess);
        Assert.False(identity.SupportsCompression);
    }

    [Fact]
    public void AgentIdentity_Capabilities_Compression()
    {
        var identity = new AgentIdentity
        {
            InstanceId = "test-001",
            OsType = OsType.Linux,
            AgentVersion = "0.1.0",
            ProtocolVersion = ProtocolVersion.Current,
            Capabilities = AgentIdentity.CapCompression
        };

        Assert.False(identity.SupportsAllProcess);
        Assert.True(identity.SupportsCompression);
    }

    [Fact]
    public void AgentIdentity_Capabilities_Both()
    {
        var identity = new AgentIdentity
        {
            InstanceId = "test-001",
            OsType = OsType.Linux,
            AgentVersion = "0.1.0",
            ProtocolVersion = ProtocolVersion.Current,
            Capabilities = AgentIdentity.CapAllProcess | AgentIdentity.CapCompression
        };

        Assert.True(identity.SupportsAllProcess);
        Assert.True(identity.SupportsCompression);
    }

    [Fact]
    public async Task FrameCodec_EncodeDecodeHandshake()
    {
        var identity = new AgentIdentity
        {
            InstanceId = "agent-123",
            OsType = OsType.Windows,
            AgentVersion = "0.1.0",
            ProtocolVersion = ProtocolVersion.Current,
            Capabilities = AgentIdentity.CapCompression
        };

        var message = new Message
        {
            Envelope = new Envelope
            {
                Version = ProtocolVersion.Current,
                MessageType = MessageType.Handshake,
                MessageId = 1,
                TimestampSecs = 1703174400,
                Compressed = false
            },
            Payload = new MessagePayload.Handshake(identity)
        };

        // Encode
        var frame = FrameCodec.Encode(message);
        Assert.True(frame.Length > 4); // At least length prefix + payload

        // Decode
        using var ms = new MemoryStream(frame);
        var decoded = await FrameCodec.DecodeAsync(ms);

        Assert.Equal(MessageType.Handshake, decoded.Envelope.MessageType);
        Assert.Equal(1UL, decoded.Envelope.MessageId);
        
        var handshake = Assert.IsType<MessagePayload.Handshake>(decoded.Payload);
        Assert.Equal("agent-123", handshake.Identity.InstanceId);
        Assert.Equal(OsType.Windows, handshake.Identity.OsType);
        Assert.True(handshake.Identity.SupportsCompression);
    }

    [Fact]
    public async Task FrameCodec_EncodeDecodeSnapshot()
    {
        var snapshot = new SnapshotPayload
        {
            WindowStartSecs = 1703174400,
            WindowEndSecs = 1703174410,
            TotalCpuPercent = 25.5f,
            TotalMemoryBytes = 8_000_000_000,
            Processes = new List<ProcessSample>
            {
                new()
                {
                    Pid = 1234,
                    Name = "test-process",
                    CpuPercent = 15.2f,
                    MemoryBytes = 100_000_000,
                    Cmdline = "/usr/bin/test"
                },
                new()
                {
                    Pid = 5678,
                    Name = "another-process",
                    CpuPercent = 10.3f,
                    MemoryBytes = 50_000_000,
                    Cmdline = null
                }
            },
            Truncated = false
        };

        var message = new Message
        {
            Envelope = new Envelope
            {
                Version = ProtocolVersion.Current,
                MessageType = MessageType.Snapshot,
                MessageId = 42,
                TimestampSecs = 1703174410,
                Compressed = false
            },
            Payload = new MessagePayload.Snapshot(snapshot)
        };

        // Encode and decode
        var frame = FrameCodec.Encode(message);
        using var ms = new MemoryStream(frame);
        var decoded = await FrameCodec.DecodeAsync(ms);

        Assert.Equal(MessageType.Snapshot, decoded.Envelope.MessageType);
        
        var decodedSnapshot = Assert.IsType<MessagePayload.Snapshot>(decoded.Payload);
        Assert.Equal(1703174400, decodedSnapshot.Payload.WindowStartSecs);
        Assert.Equal(2, decodedSnapshot.Payload.Processes.Count);
        Assert.Equal(1234u, decodedSnapshot.Payload.Processes[0].Pid);
        Assert.False(decodedSnapshot.Payload.Truncated);
    }

    [Fact]
    public async Task FrameCodec_EncodeDecodeBackpressure()
    {
        var message = new Message
        {
            Envelope = new Envelope
            {
                Version = ProtocolVersion.Current,
                MessageType = MessageType.Backpressure,
                MessageId = 200,
                TimestampSecs = 1703174420,
                Compressed = false
            },
            Payload = new MessagePayload.Backpressure(new BackpressureSignal
            {
                Level = 2,
                PauseSecs = 30
            })
        };

        var frame = FrameCodec.Encode(message);
        using var ms = new MemoryStream(frame);
        var decoded = await FrameCodec.DecodeAsync(ms);

        var backpressure = Assert.IsType<MessagePayload.Backpressure>(decoded.Payload);
        Assert.Equal((byte)2, backpressure.Signal.Level);
        Assert.Equal(30u, backpressure.Signal.PauseSecs);
    }

    [Fact]
    public async Task FrameCodec_EncodeDecodeAck()
    {
        var message = new Message
        {
            Envelope = new Envelope
            {
                Version = ProtocolVersion.Current,
                MessageType = MessageType.Ack,
                MessageId = 300,
                TimestampSecs = 1703174430,
                Compressed = false
            },
            Payload = new MessagePayload.Ack(new MessageAck
            {
                MessageId = 42,
                Success = false,
                ErrorCode = 1001
            })
        };

        var frame = FrameCodec.Encode(message);
        using var ms = new MemoryStream(frame);
        var decoded = await FrameCodec.DecodeAsync(ms);

        var ack = Assert.IsType<MessagePayload.Ack>(decoded.Payload);
        Assert.Equal(42UL, ack.AckData.MessageId);
        Assert.False(ack.AckData.Success);
        Assert.Equal(1001u, ack.AckData.ErrorCode);
    }

    [Fact]
    public void FrameCodec_FrameTooLarge_ThrowsException()
    {
        // Create a snapshot with many processes to exceed MAX_FRAME_SIZE
        var largeSnapshot = new SnapshotPayload
        {
            WindowStartSecs = 1703174400,
            WindowEndSecs = 1703174410,
            TotalCpuPercent = 100.0f,
            TotalMemoryBytes = 32_000_000_000,
            Processes = Enumerable.Range(0, 10000)
                .Select(i => new ProcessSample
                {
                    Pid = (uint)i,
                    Name = $"very-long-process-name-{i}",
                    CpuPercent = 1.0f,
                    MemoryBytes = 1_000_000,
                    Cmdline = $"/very/long/command/line/path/number/{i}/with/many/args"
                })
                .ToList(),
            Truncated = false
        };

        var message = new Message
        {
            Envelope = new Envelope
            {
                Version = ProtocolVersion.Current,
                MessageType = MessageType.Snapshot,
                MessageId = 999,
                TimestampSecs = 1703174440,
                Compressed = false
            },
            Payload = new MessagePayload.Snapshot(largeSnapshot)
        };

        // Should throw FrameTooLargeException
        Assert.Throws<FrameTooLargeException>(() => FrameCodec.Encode(message));
    }

    /// <summary>
    /// Cross-language serialization test.
    /// Reads binary snapshot generated by Rust tests and verifies .NET can deserialize it.
    /// This validates protocol compatibility between Rust and .NET implementations.
    /// </summary>
    [Fact]
    public async Task CrossLanguage_DeserializeRustGeneratedSnapshot()
    {
        var testDataPath = Path.Combine(
            AppDomain.CurrentDomain.BaseDirectory,
            "..",
            "..",
            "..",
            "data",
            "cross-language-snapshot.bin"
        );

        // Ensure the test data file exists
        Assert.True(File.Exists(testDataPath), $"Cross-language test data file not found at: {testDataPath}");

        // Read the binary data generated by Rust
        using var stream = File.OpenRead(testDataPath);
        
        // Decode the frame using async API
        var message = await FrameCodec.DecodeAsync(stream);

        // Verify the message structure
        Assert.NotNull(message);
        
        // Message type should be Snapshot (4) - debug info
        if (message.Envelope.MessageType != MessageType.Snapshot)
        {
            throw new XunitException($"Expected Snapshot, but got {message.Envelope.MessageType} (this indicates the file was overwritten by another test)");
        }

        // Verify payload is a snapshot
        var snapshot = Assert.IsType<MessagePayload.Snapshot>(message.Payload);
        Assert.NotNull(snapshot.Payload);

        // Verify basic snapshot structure (matches what Rust test generates)
        Assert.NotEmpty(snapshot.Payload.Processes);
        Assert.Equal(3, snapshot.Payload.Processes.Count);

        // Verify window timestamps
        Assert.True(snapshot.Payload.WindowStartSecs > 0);
        Assert.True(snapshot.Payload.WindowEndSecs > snapshot.Payload.WindowStartSecs);

        // Verify CPU and memory aggregates exist
        Assert.True(snapshot.Payload.TotalCpuPercent >= 0);
        Assert.True(snapshot.Payload.TotalMemoryBytes >= 0);

        // Verify individual process samples
        foreach (var process in snapshot.Payload.Processes)
        {
            Assert.True(process.Pid > 0);
            Assert.NotEmpty(process.Name);
            Assert.True(process.CpuPercent >= 0);
            Assert.True(process.MemoryBytes >= 0);
        }
    }
}
