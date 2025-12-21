/// <summary>
/// Storage interface abstraction for persisting monitoring records.
/// 
/// Decouples business logic from storage implementation.
/// Initial implementation: append to file with rotation policy.
/// </summary>

namespace MonitoringServer.Storage;

using Protocol;

/// <summary>
/// Canonical storage record persisted through the storage interface.
/// Independent of storage medium.
/// </summary>
public sealed class StorageRecord
{
    /// <summary>Agent instance ID that sent this record</summary>
    public required string AgentInstanceId { get; init; }

    /// <summary>Timestamp when record was received by server (Unix epoch seconds)</summary>
    public required long ReceivedTimestampSecs { get; init; }

    /// <summary>Snapshot payload data</summary>
    public required SnapshotPayload Snapshot { get; init; }
}

/// <summary>
/// Storage interface for persisting monitoring records.
/// 
/// Implementations must be thread-safe for concurrent writes.
/// </summary>
public interface IStorageWriter
{
    /// <summary>
    /// Append a record to storage.
    /// 
    /// At-most-once semantics: failures are logged but not retried.
    /// </summary>
    /// <param name="record">Record to persist</param>
    /// <param name="cancellationToken">Cancellation token</param>
    Task AppendAsync(StorageRecord record, CancellationToken cancellationToken = default);

    /// <summary>
    /// Flush any buffered records to durable storage.
    /// </summary>
    Task FlushAsync(CancellationToken cancellationToken = default);
}
