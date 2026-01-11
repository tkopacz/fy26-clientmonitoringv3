/// <summary>
/// Binary protocol definitions and decoding for monitoring server.
/// 
/// Implements the versioned binary protocol with:
/// - Framing: length-prefix + message + CRC32 checksum
/// - Envelope: version, platform, timestamps, agent ID
/// - zstd decompression
/// - At-least-once delivery semantics with retry and de-duplication
/// </summary>

namespace MonitoringServer.Protocol;

/// <summary>
/// Protocol version (MAJOR.MINOR format).
/// </summary>
public readonly struct ProtocolVersion : IEquatable<ProtocolVersion>
{
    public byte Major { get; init; }
    public byte Minor { get; init; }

    /// <summary>
    /// Current protocol version: 1.0
    /// </summary>
    public static readonly ProtocolVersion Current = new() { Major = 1, Minor = 0 };

    /// <summary>
    /// Check if this version is compatible with another version.
    /// Compatible if major versions match and minor >= other.minor.
    /// </summary>
    public bool IsCompatibleWith(ProtocolVersion other) =>
        Major == other.Major && Minor >= other.Minor;

    public bool Equals(ProtocolVersion other) =>
        Major == other.Major && Minor == other.Minor;

    public override bool Equals(object? obj) =>
        obj is ProtocolVersion other && Equals(other);

    public override int GetHashCode() => HashCode.Combine(Major, Minor);

    public static bool operator ==(ProtocolVersion left, ProtocolVersion right) =>
        left.Equals(right);

    public static bool operator !=(ProtocolVersion left, ProtocolVersion right) =>
        !left.Equals(right);

    public override string ToString() => $"{Major}.{Minor}";
}

/// <summary>
/// Message types in the protocol.
/// </summary>
public enum MessageType : byte
{
    Handshake = 1,
    HandshakeAck = 2,
    Heartbeat = 3,
    Snapshot = 4,
    Ack = 5,
    Backpressure = 6,
    Error = 7
}

/// <summary>
/// Operating system type for agent identity.
/// </summary>
public enum OsType : byte
{
    Windows = 1,
    Linux = 2
}

/// <summary>
/// Agent identity information sent during handshake.
/// </summary>
public sealed class AgentIdentity
{
    /// <summary>Capability flag: supports all-process mode</summary>
    public const uint CapAllProcess = 0x01;
    /// <summary>Capability flag: supports zstd compression</summary>
    public const uint CapCompression = 0x02;

    /// <summary>Unique instance identifier for this agent</summary>
    public required string InstanceId { get; init; }

    /// <summary>Operating system type</summary>
    public required OsType OsType { get; init; }

    /// <summary>Agent version string</summary>
    public required string AgentVersion { get; init; }

    /// <summary>Protocol version supported by this agent</summary>
    public required ProtocolVersion ProtocolVersion { get; init; }

    /// <summary>Capability flags</summary>
    public required uint Capabilities { get; init; }

    /// <summary>Check if agent supports all-process mode</summary>
    public bool SupportsAllProcess => (Capabilities & CapAllProcess) != 0;

    /// <summary>Check if agent supports compression</summary>
    public bool SupportsCompression => (Capabilities & CapCompression) != 0;
}

/// <summary>
/// Single process sample in a snapshot.
/// Ordered by CPU usage (descending) when truncation is applied.
/// </summary>
public sealed class ProcessSample
{
    /// <summary>Process ID</summary>
    public required uint Pid { get; init; }

    /// <summary>Process name</summary>
    public required string Name { get; init; }

    /// <summary>CPU usage percentage (0.0 - 100.0)</summary>
    public required float CpuPercent { get; init; }

    /// <summary>Memory usage percentage of total system memory (0.0 - 100.0)</summary>
    public required float MemoryPercent { get; init; }

    /// <summary>Memory usage in bytes (RSS)</summary>
    public required ulong MemoryBytes { get; init; }

    /// <summary>Optional command line</summary>
    public string? Cmdline { get; init; }
}

/// <summary>
/// Monitoring snapshot payload.
/// Contains aggregated CPU/memory metrics and per-process samples.
/// </summary>
public sealed class SnapshotPayload
{
    /// <summary>Sampling window start timestamp (Unix epoch seconds)</summary>
    public required long WindowStartSecs { get; init; }

    /// <summary>Sampling window end timestamp (Unix epoch seconds)</summary>
    public required long WindowEndSecs { get; init; }

    /// <summary>Aggregate CPU usage percentage (0.0 - 100.0)</summary>
    public required float TotalCpuPercent { get; init; }

    /// <summary>Memory currently in use (bytes)</summary>
    public required ulong MemUsedBytes { get; init; }

    /// <summary>Total system memory (bytes)</summary>
    public required ulong MemTotalBytes { get; init; }

    /// <summary>Process samples (ordered by CPU, truncated if needed)</summary>
    public required List<ProcessSample> Processes { get; init; }

    /// <summary>True if process list was truncated to fit size cap</summary>
    public required bool Truncated { get; init; }
}

/// <summary>
/// Backpressure signal from server to agent.
/// Instructs agent to throttle its send rate by applying a delay.
/// </summary>
public sealed class BackpressureSignal
{
    /// <summary>
    /// Throttle delay in milliseconds (0 = no throttle).
    /// Agent applies: effectiveIntervalMs = max(configuredSnapshotIntervalMs, throttleDelayMs)
    /// </summary>
    public required uint ThrottleDelayMs { get; init; }

    /// <summary>Optional reason string for logging/diagnostics</summary>
    public string? Reason { get; init; }
}

/// <summary>
/// Message acknowledgment with status and optional error code.
/// </summary>
public sealed class MessageAck
{
    /// <summary>ID of the message being acknowledged (opaque 16-byte identifier)</summary>
    public required byte[] MessageId { get; init; }

    /// <summary>Success flag</summary>
    public required bool Success { get; init; }

    /// <summary>Optional error code if success=false</summary>
    public uint? ErrorCode { get; init; }
}

/// <summary>
/// Protocol envelope containing message metadata.
/// Wraps the payload with version, type, ID, timestamps, and compression flag.
/// </summary>
public sealed class Envelope
{
    /// <summary>Protocol version</summary>
    public required ProtocolVersion Version { get; init; }

    /// <summary>Message type</summary>
    public required MessageType MessageType { get; init; }

    /// <summary>Unique message ID for correlation (opaque 16-byte identifier)</summary>
    public required byte[] MessageId { get; init; }

    /// <summary>Timestamp when message was created (UTC Unix epoch milliseconds)</summary>
    public required long TimestampUtcMs { get; init; }

    /// <summary>Agent instance identifier</summary>
    public required string AgentId { get; init; }

    /// <summary>Platform (OS type)</summary>
    public required OsType Platform { get; init; }

    /// <summary>True if payload is zstd-compressed</summary>
    public required bool Compressed { get; init; }
}

/// <summary>
/// Message payload variants.
/// </summary>
public abstract record MessagePayload
{
    public sealed record Handshake(AgentIdentity Identity) : MessagePayload;
    public sealed record HandshakeAck : MessagePayload;
    public sealed record Heartbeat : MessagePayload;
    public sealed record Snapshot(SnapshotPayload Payload) : MessagePayload;
    public sealed record Ack(MessageAck AckData) : MessagePayload;
    public sealed record Backpressure(BackpressureSignal Signal) : MessagePayload;
    public sealed record Error(uint Code, string Message) : MessagePayload;
}

/// <summary>
/// Top-level message container.
/// Combines envelope with the actual message payload.
/// </summary>
public sealed class Message
{
    /// <summary>Message envelope with metadata</summary>
    public required Envelope Envelope { get; init; }

    /// <summary>Message payload (varies by type)</summary>
    public required MessagePayload Payload { get; init; }
}

/// <summary>
/// Protocol errors.
/// </summary>
public class ProtocolException : Exception
{
    public ProtocolException(string message) : base(message) { }
    public ProtocolException(string message, Exception inner) : base(message, inner) { }
}

public class InvalidMessageTypeException : ProtocolException
{
    public byte Value { get; }
    public InvalidMessageTypeException(byte value)
        : base($"Invalid message type: {value}")
    {
        Value = value;
    }
}

public class IncompatibleVersionException : ProtocolException
{
    public IncompatibleVersionException()
        : base("Incompatible protocol version") { }
}

public class FrameTooLargeException : ProtocolException
{
    public int Size { get; }
    public int MaxSize { get; }
    public FrameTooLargeException(int size, int maxSize)
        : base($"Frame too large: {size} bytes (max {maxSize})")
    {
        Size = size;
        MaxSize = maxSize;
    }
}

public class Crc32MismatchException : ProtocolException
{
    public uint Expected { get; }
    public uint Actual { get; }
    public Crc32MismatchException(uint expected, uint actual)
        : base($"CRC32 checksum mismatch: expected 0x{expected:X8}, got 0x{actual:X8}")
    {
        Expected = expected;
        Actual = actual;
    }
}
