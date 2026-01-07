/// <summary>
/// Wire format framing and decoding for binary protocol.
/// 
/// Framing: [4-byte length][message bytes][4-byte CRC32]
/// Message bytes: custom-encoded Message, optionally zstd-compressed
/// </summary>

using System.Buffers.Binary;
using System.Runtime.Serialization;
using System.Text;
using ZstdSharp;
using Force.Crc32;

namespace MonitoringServer.Protocol;

/// <summary>
/// Frame codec for encoding and decoding protocol messages.
/// </summary>
public static class FrameCodec
{
    /// <summary>
    /// Maximum uncompressed frame size (256 KB safeguard).
    /// Must be kept in sync with Rust MAX_FRAME_SIZE in agent/src/protocol.rs (line 264).
    /// </summary>
    public const int MaxFrameSize = 256 * 1024;

    /// <summary>Target compressed frame size (64 KB)</summary>
    public const int TargetFrameSize = 64 * 1024;

    /// <summary>
    /// Decode a message from a stream.
    /// 
    /// Steps:
    /// 1. Read 4-byte length (big-endian)
    /// 2. Read payload bytes
    /// 3. Read and validate CRC32 checksum (4 bytes, little-endian)
    /// 4. Decompress if needed (detected via envelope)
    /// 5. Deserialize message (custom bincode-compatible format)
    /// </summary>
    /// <param name="stream">Input stream positioned at frame start</param>
    /// <param name="cancellationToken">Cancellation token</param>
    /// <returns>Decoded message</returns>
    public static async Task<Message> DecodeAsync(
        Stream stream,
        CancellationToken cancellationToken = default)
    {
        // Read length prefix (4 bytes, big-endian)
        var lengthBuffer = new byte[4];
        await ReadExactAsync(stream, lengthBuffer, cancellationToken);
        var payloadLength = BinaryPrimitives.ReadUInt32BigEndian(lengthBuffer);

        // Validate length
        if (payloadLength > MaxFrameSize)
        {
            throw new FrameTooLargeException((int)payloadLength, MaxFrameSize);
        }

        // Read payload
        var payload = new byte[payloadLength];
        await ReadExactAsync(stream, payload, cancellationToken);

        // Read CRC32 checksum (4 bytes, little-endian)
        var crcBuffer = new byte[4];
        await ReadExactAsync(stream, crcBuffer, cancellationToken);
        var expectedCrc = BinaryPrimitives.ReadUInt32LittleEndian(crcBuffer);

        // Validate CRC32
        var actualCrc = Crc32Algorithm.Compute(payload);
        if (actualCrc != expectedCrc)
        {
            throw new Crc32MismatchException(expectedCrc, actualCrc);
        }

        // Deserialize message (check compression flag first)
        var message = DeserializeMessage(payload);

        return message;
    }

    /// <summary>
    /// Encode a message into a framed byte buffer.
    /// 
    /// Steps:
    /// 1. Serialize message to bytes (bincode-compatible)
    /// 2. Compress if envelope.compressed=true (zstd level 3)
    /// 3. Compute CRC32 checksum
    /// 4. Build frame: [length:u32 BE][payload][crc32:u32 LE]
    /// </summary>
    /// <param name="message">Message to encode</param>
    /// <returns>Framed bytes ready to write to socket</returns>
    public static byte[] Encode(Message message)
    {
        // Serialize message
        var serialized = SerializeMessage(message);

        // Compress if requested
        var payload = message.Envelope.Compressed
            ? CompressPayload(serialized)
            : serialized;

        // Check size constraints
        if (payload.Length > MaxFrameSize)
        {
            throw new FrameTooLargeException(payload.Length, MaxFrameSize);
        }

        // Compute CRC32 checksum
        var checksum = Crc32Algorithm.Compute(payload);

        // Build frame: [length:u32 BE][payload][crc32:u32 LE]
        var frame = new byte[4 + payload.Length + 4];
        BinaryPrimitives.WriteUInt32BigEndian(frame.AsSpan(0, 4), (uint)payload.Length);
        payload.CopyTo(frame, 4);
        BinaryPrimitives.WriteUInt32LittleEndian(frame.AsSpan(4 + payload.Length, 4), checksum);

        return frame;
    }

    /// <summary>
    /// Write a framed message to a stream.
    /// </summary>
    public static async Task WriteAsync(
        Stream stream,
        Message message,
        CancellationToken cancellationToken = default)
    {
        var frame = Encode(message);
        await stream.WriteAsync(frame, cancellationToken);
        await stream.FlushAsync(cancellationToken);
    }

    /// <summary>
    /// Read exact number of bytes from stream (throws on EOF).
    /// </summary>
    private static async Task ReadExactAsync(
        Stream stream,
        byte[] buffer,
        CancellationToken cancellationToken)
    {
        var offset = 0;
        var remaining = buffer.Length;

        while (remaining > 0)
        {
            var read = await stream.ReadAsync(
                buffer.AsMemory(offset, remaining),
                cancellationToken);

            if (read == 0)
            {
                throw new EndOfStreamException(
                    $"Unexpected end of stream: expected {buffer.Length} bytes, got {offset}");
            }

            offset += read;
            remaining -= read;
        }
    }

    /// <summary>
    /// Compress payload using zstd level 3.
    /// </summary>
    private static byte[] CompressPayload(byte[] data)
    {
        using var compressor = new Compressor(3); // zstd level 3
        return compressor.Wrap(data).ToArray();
    }

    /// <summary>
    /// Decompress payload using zstd.
    /// </summary>
    private static byte[] DecompressPayload(byte[] data)
    {
        using var decompressor = new Decompressor();
        return decompressor.Unwrap(data).ToArray();
    }

    /// <summary>
    /// Serialize message to bytes (bincode-compatible format).
    /// 
    /// Implements the subset of the Rust agent's bincode wire format that this server relies on.
    /// When modifying this method, ensure the produced wire format remains compatible with the
    /// corresponding Rust implementation (see agent/src/protocol.rs).
    /// For details on keeping MessageType discriminants synchronized with the Rust definition, see
    /// the documentation on the MessageType enum (in Messages.cs) and agent/src/protocol.rs.
    /// </summary>
    private static byte[] SerializeMessage(Message message)
    {
        using var ms = new MemoryStream();
        using var writer = new BinaryWriter(ms);

        // Write envelope
        writer.Write(message.Envelope.Version.Major);
        writer.Write(message.Envelope.Version.Minor);
        writer.Write((byte)message.Envelope.MessageType);
        writer.Write(message.Envelope.MessageId);
        writer.Write(message.Envelope.TimestampSecs);
        writer.Write(message.Envelope.Compressed);

        // Write payload based on type
        WritePayload(writer, message.Payload);

        return ms.ToArray();
    }

    /// <summary>
    /// Deserialize message from bytes (bincode-compatible format).
    /// </summary>
    private static Message DeserializeMessage(byte[] data)
    {
        using var ms = new MemoryStream(data);
        using var reader = new BinaryReader(ms);

        // Read envelope
        var version = new ProtocolVersion
        {
            Major = reader.ReadByte(),
            Minor = reader.ReadByte()
        };
        var messageType = (MessageType)reader.ReadByte();
        var messageId = reader.ReadUInt64();
        var timestampSecs = reader.ReadInt64();
        var compressed = reader.ReadBoolean();

        var envelope = new Envelope
        {
            Version = version,
            MessageType = messageType,
            MessageId = messageId,
            TimestampSecs = timestampSecs,
            Compressed = compressed
        };

        // Read payload based on type
        MessagePayload payload;
        if (compressed)
        {
            // If compressed, decompress the remaining data first
            var compressedPayload = reader.ReadBytes((int)(ms.Length - ms.Position));
            var payloadData = DecompressPayload(compressedPayload);
            using var payloadMs = new MemoryStream(payloadData);
            using var payloadReader = new BinaryReader(payloadMs);
            payload = ReadPayload(payloadReader, messageType);
        }
        else
        {
            payload = ReadPayload(reader, messageType);
        }

        return new Message
        {
            Envelope = envelope,
            Payload = payload
        };
    }

    /// <summary>
    /// Write message payload based on message type.
    /// </summary>
    private static void WritePayload(BinaryWriter writer, MessagePayload payload)
    {
        switch (payload)
        {
            case MessagePayload.Handshake handshake:
                WriteString(writer, handshake.Identity.InstanceId);
                writer.Write((byte)handshake.Identity.OsType);
                WriteString(writer, handshake.Identity.AgentVersion);
                writer.Write(handshake.Identity.ProtocolVersion.Major);
                writer.Write(handshake.Identity.ProtocolVersion.Minor);
                writer.Write(handshake.Identity.Capabilities);
                break;

            case MessagePayload.HandshakeAck:
                // No payload
                break;

            case MessagePayload.Heartbeat:
                // No payload
                break;

            case MessagePayload.Snapshot snapshot:
                writer.Write(snapshot.Payload.WindowStartSecs);
                writer.Write(snapshot.Payload.WindowEndSecs);
                writer.Write(snapshot.Payload.TotalCpuPercent);
                writer.Write(snapshot.Payload.TotalMemoryBytes);
                writer.Write((ulong)snapshot.Payload.Processes.Count);
                foreach (var process in snapshot.Payload.Processes)
                {
                    writer.Write(process.Pid);
                    WriteString(writer, process.Name);
                    writer.Write(process.CpuPercent);
                    writer.Write(process.MemoryBytes);
                    WriteOptionalString(writer, process.Cmdline);
                }
                writer.Write(snapshot.Payload.Truncated);
                break;

            case MessagePayload.Ack ack:
                writer.Write(ack.AckData.MessageId);
                writer.Write(ack.AckData.Success);
                WriteOptionalUInt32(writer, ack.AckData.ErrorCode);
                break;

            case MessagePayload.Backpressure backpressure:
                writer.Write(backpressure.Signal.Level);
                WriteOptionalUInt32(writer, backpressure.Signal.PauseSecs);
                break;

            case MessagePayload.Error error:
                writer.Write(error.Code);
                WriteString(writer, error.Message);
                break;

            default:
                throw new SerializationException($"Unknown payload type: {payload.GetType()}");
        }
    }

    /// <summary>
    /// Read message payload based on message type.
    /// </summary>
    private static MessagePayload ReadPayload(BinaryReader reader, MessageType messageType)
    {
        return messageType switch
        {
            MessageType.Handshake => new MessagePayload.Handshake(new AgentIdentity
            {
                InstanceId = ReadString(reader),
                OsType = (OsType)reader.ReadByte(),
                AgentVersion = ReadString(reader),
                ProtocolVersion = new ProtocolVersion
                {
                    Major = reader.ReadByte(),
                    Minor = reader.ReadByte()
                },
                Capabilities = reader.ReadUInt32()
            }),

            MessageType.HandshakeAck => new MessagePayload.HandshakeAck(),

            MessageType.Heartbeat => new MessagePayload.Heartbeat(),

            MessageType.Snapshot => new MessagePayload.Snapshot(new SnapshotPayload
            {
                WindowStartSecs = reader.ReadInt64(),
                WindowEndSecs = reader.ReadInt64(),
                TotalCpuPercent = reader.ReadSingle(),
                TotalMemoryBytes = reader.ReadUInt64(),
                Processes = ReadProcessList(reader),
                Truncated = reader.ReadBoolean()
            }),

            MessageType.Ack => new MessagePayload.Ack(new MessageAck
            {
                MessageId = reader.ReadUInt64(),
                Success = reader.ReadBoolean(),
                ErrorCode = ReadOptionalUInt32(reader)
            }),

            MessageType.Backpressure => new MessagePayload.Backpressure(new BackpressureSignal
            {
                Level = reader.ReadByte(),
                PauseSecs = ReadOptionalUInt32(reader)
            }),

            MessageType.Error => new MessagePayload.Error(
                reader.ReadUInt32(),
                ReadString(reader)
            ),

            _ => throw new InvalidMessageTypeException((byte)messageType)
        };
    }

    private static List<ProcessSample> ReadProcessList(BinaryReader reader)
    {
        var count = reader.ReadUInt64();
        var processes = new List<ProcessSample>((int)count);

        for (ulong i = 0; i < count; i++)
        {
            processes.Add(new ProcessSample
            {
                Pid = reader.ReadUInt32(),
                Name = ReadString(reader),
                CpuPercent = reader.ReadSingle(),
                MemoryBytes = reader.ReadUInt64(),
                Cmdline = ReadOptionalString(reader)
            });
        }

        return processes;
    }

    private static void WriteString(BinaryWriter writer, string value)
    {
        var bytes = Encoding.UTF8.GetBytes(value);
        writer.Write((ulong)bytes.Length);
        writer.Write(bytes);
    }

    private static string ReadString(BinaryReader reader)
    {
        var length = reader.ReadUInt64();
        var bytes = reader.ReadBytes((int)length);
        return Encoding.UTF8.GetString(bytes);
    }

    private static void WriteOptionalString(BinaryWriter writer, string? value)
    {
        if (value is null)
        {
            writer.Write((byte)0);
        }
        else
        {
            writer.Write((byte)1);
            WriteString(writer, value);
        }
    }

    private static string? ReadOptionalString(BinaryReader reader)
    {
        var hasValue = reader.ReadByte() != 0;
        return hasValue ? ReadString(reader) : null;
    }

    private static void WriteOptionalUInt32(BinaryWriter writer, uint? value)
    {
        if (value is null)
        {
            writer.Write((byte)0);
        }
        else
        {
            writer.Write((byte)1);
            writer.Write(value.Value);
        }
    }

    private static uint? ReadOptionalUInt32(BinaryReader reader)
    {
        var hasValue = reader.ReadByte() != 0;
        return hasValue ? reader.ReadUInt32() : null;
    }
}
