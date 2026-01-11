using MonitoringServer.Protocol;
using Xunit;

namespace MonitoringServer.Tests.Protocol;

public class DemoProtocolDecodeLoopTests
{
    [Fact]
    public async Task DecodeAllFrames_MultiFrame_Succeeds()
    {
        var message = new Message
        {
            Envelope = new Envelope
            {
                Version = ProtocolVersion.Current,
                MessageType = MessageType.Snapshot,
                MessageId = TestMessageId(1),
                TimestampUtcMs = 1703174410000,
                AgentId = "demo-agent-ž",
                Platform = OsType.Linux,
                Compressed = false
            },
            Payload = new MessagePayload.Snapshot(new SnapshotPayload
            {
                WindowStartSecs = 1703174400,
                WindowEndSecs = 1703174410,
                TotalCpuPercent = 12.345f,
                MemUsedBytes = 1_500_000_000,
                MemTotalBytes = 8_000_000_000,
                Processes = new()
                {
                    new ProcessSample
                    {
                        Pid = 1234,
                        Name = "demo-π",
                        CpuPercent = 1.234f,
                        MemoryPercent = 0.321f,
                        MemoryBytes = 50_000_000,
                        Cmdline = "/usr/bin/demo --mode=π"
                    },
                    new ProcessSample
                    {
                        Pid = 5678,
                        Name = "worker",
                        CpuPercent = 0.500f,
                        MemoryPercent = 0.111f,
                        MemoryBytes = 75_000_000,
                        Cmdline = null
                    }
                },
                Truncated = false
            })
        };

        var frame = FrameCodec.Encode(message);
        var bytes = frame.Concat(frame).ToArray();

        using var stream = new MemoryStream(bytes);

        var decoded = new List<Message>();
        while (stream.Position < stream.Length)
        {
            decoded.Add(await FrameCodec.DecodeAsync(stream));
        }

        Assert.Equal(2, decoded.Count);
        Assert.All(decoded, m => Assert.Equal(MessageType.Snapshot, m.Envelope.MessageType));
    }

    [Fact]
    public async Task DecodeAllFrames_TruncatedFirstFrame_Fails()
    {
        var message = new Message
        {
            Envelope = new Envelope
            {
                Version = ProtocolVersion.Current,
                MessageType = MessageType.Snapshot,
                MessageId = TestMessageId(2),
                TimestampUtcMs = 1703174410000,
                AgentId = "demo-agent-ž",
                Platform = OsType.Linux,
                Compressed = false
            },
            Payload = new MessagePayload.Snapshot(new SnapshotPayload
            {
                WindowStartSecs = 1703174400,
                WindowEndSecs = 1703174410,
                TotalCpuPercent = 12.345f,
                MemUsedBytes = 1_500_000_000,
                MemTotalBytes = 8_000_000_000,
                Processes = new() { new ProcessSample { Pid = 1, Name = "x", CpuPercent = 1f, MemoryPercent = 1f, MemoryBytes = 1, Cmdline = null } },
                Truncated = false
            })
        };

        var frame = FrameCodec.Encode(message);
        var truncated = frame.Take(frame.Length - 1).ToArray();

        using var stream = new MemoryStream(truncated);
        await Assert.ThrowsAsync<EndOfStreamException>(() => FrameCodec.DecodeAsync(stream));
    }

    [Fact]
    public async Task DecodeAllFrames_ValidFramePlusTrailingJunk_FailsOnTrailingBytes()
    {
        var message = new Message
        {
            Envelope = new Envelope
            {
                Version = ProtocolVersion.Current,
                MessageType = MessageType.Snapshot,
                MessageId = TestMessageId(3),
                TimestampUtcMs = 1703174410000,
                AgentId = "demo-agent-ž",
                Platform = OsType.Linux,
                Compressed = false
            },
            Payload = new MessagePayload.Snapshot(new SnapshotPayload
            {
                WindowStartSecs = 1703174400,
                WindowEndSecs = 1703174410,
                TotalCpuPercent = 12.345f,
                MemUsedBytes = 1_500_000_000,
                MemTotalBytes = 8_000_000_000,
                Processes = new() { new ProcessSample { Pid = 1, Name = "x", CpuPercent = 1f, MemoryPercent = 1f, MemoryBytes = 1, Cmdline = null } },
                Truncated = false
            })
        };

        var frame = FrameCodec.Encode(message);
        var bytes = frame.Concat(new byte[] { 0xFF }).ToArray();

        using var stream = new MemoryStream(bytes);

        var decodedAny = false;
        while (stream.Position < stream.Length)
        {
            try
            {
                await FrameCodec.DecodeAsync(stream);
                decodedAny = true;
            }
            catch (EndOfStreamException)
            {
                Assert.True(decodedAny); // trailing bytes after at least one frame
                return;
            }
        }

        Assert.Fail("Expected trailing-bytes failure");
    }

    [Fact]
    public async Task DecodeAsync_LengthPrefixTooLarge_ThrowsFrameTooLarge()
    {
        var tooLarge = (uint)FrameCodec.MaxFrameSize + 1;

        // Frame starts with [len:u32 BE]; decoder should throw before attempting to read body.
        var bytes = new byte[4];
        bytes[0] = (byte)((tooLarge >> 24) & 0xFF);
        bytes[1] = (byte)((tooLarge >> 16) & 0xFF);
        bytes[2] = (byte)((tooLarge >> 8) & 0xFF);
        bytes[3] = (byte)(tooLarge & 0xFF);

        using var stream = new MemoryStream(bytes);
        await Assert.ThrowsAsync<FrameTooLargeException>(() => FrameCodec.DecodeAsync(stream));
    }

    [Fact]
    public void EmptyStream_IsEmpty()
    {
        using var stream = new MemoryStream(Array.Empty<byte>());
        Assert.Equal(0, stream.Length);
    }

    private static byte[] TestMessageId(ulong n)
    {
        var id = new byte[16];
        BitConverter.GetBytes(n).CopyTo(id, 0);
        return id;
    }
}
