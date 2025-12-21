/// <summary>
/// File-based storage writer that appends records to a JSON Lines file.
/// 
/// Initial implementation with:
/// - Append-only writes
/// - Basic file rotation (size-based)
/// - Thread-safe concurrent writes
/// </summary>

using System.Text.Json;
using System.Text.Json.Serialization;
using System.Text;
using System.Diagnostics.Metrics;
using Microsoft.Extensions.Logging;

namespace MonitoringServer.Storage;

/// <summary>
/// File storage configuration.
/// </summary>
public sealed class FileStorageConfig
{
    /// <summary>Base directory for storage files</summary>
    public required string BaseDirectory { get; init; }

    /// <summary>File name prefix (default: "monitoring")</summary>
    public string FilePrefix { get; init; } = "monitoring";

    /// <summary>Maximum file size before rotation in bytes (default: 100 MB)</summary>
    public long MaxFileSizeBytes { get; init; } = 100 * 1024 * 1024;
}

/// <summary>
/// File-based storage writer using JSON Lines format.
/// 
/// Each record is written as a single-line JSON object.
/// Thread-safe with locking for concurrent append operations.
/// </summary>
public sealed class FileStorageWriter : IStorageWriter, IAsyncDisposable, IDisposable
{
    private readonly FileStorageConfig _config;
    private readonly ILogger<FileStorageWriter> _logger;
    private readonly SemaphoreSlim _writeLock = new(1, 1);
    private StreamWriter? _currentWriter;
    private string? _currentFilePath;
    private long _currentFileSize;

    // Cached newline byte count for the writer encoding to avoid repeated computation per append
    private readonly int _newlineByteCount;
    private int _disposed = 0; // 0 = not disposed, 1 = disposed

    private readonly JsonSerializerOptions _jsonOptions = new()
    {
        WriteIndented = false,
        DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull
    };

    // Metrics: expose counters for monitoring append success/failure
    private static readonly Meter s_meter = new("MonitoringServer.Storage.FileStorageWriter");
    private readonly Counter<long> _appendSuccessCounter;
    private readonly Counter<long> _appendFailureCounter;

    // In-memory thread-safe counters for quick inspection/testing
    private long _successfulAppendCount;
    private long _failedAppendCount;

    // Public read-only accessors
    public long SuccessfulAppendCount => System.Threading.Volatile.Read(ref _successfulAppendCount);
    public long FailedAppendCount => System.Threading.Volatile.Read(ref _failedAppendCount);

    public FileStorageWriter(
        FileStorageConfig config,
        ILogger<FileStorageWriter> logger)
    {
        _config = config ?? throw new ArgumentNullException(nameof(config));
        _logger = logger ?? throw new ArgumentNullException(nameof(logger));

        // Initialize metrics
        _appendSuccessCounter = s_meter.CreateCounter<long>(
            "file_storage_append_success",
            description: "Number of successful append operations");
        _appendFailureCounter = s_meter.CreateCounter<long>(
            "file_storage_append_failure",
            description: "Number of failed append operations");

        // Ensure base directory exists
        Directory.CreateDirectory(_config.BaseDirectory);

        // Cache newline byte count for the chosen encoding (UTF-8)
        _newlineByteCount = Encoding.UTF8.GetByteCount(Environment.NewLine);
    }

    /// <summary>
    /// Append a record to the current file.
    /// 
    /// Automatically rotates to a new file if size threshold is exceeded.
    /// Thread-safe with internal locking.
    /// Best-effort with at-most-once semantics: append errors are logged and suppressed (no retries).
    /// </summary>
    public async Task AppendAsync(
        StorageRecord record,
        CancellationToken cancellationToken = default)
    {
        using (await AcquireLockAsync(cancellationToken))
        {
            try
            {
                // Rotate file if needed
                if (_currentWriter == null || _currentFileSize >= _config.MaxFileSizeBytes)
                {
                    await RotateFileAsync();
                }

                // Serialize record to JSON (single line)
                var json = JsonSerializer.Serialize(record, _jsonOptions);
                
                // Append to file
                await _currentWriter!.WriteLineAsync(json);
                // Update in-memory size estimate based on encoding, without forcing a flush.
                // Writer uses UTF-8, so this correctly accounts for multi-byte characters.
                _currentFileSize += _currentWriter.Encoding.GetByteCount(json) + _newlineByteCount;

                _logger.LogDebug(
                    "Appended record from agent {AgentId} to {FilePath}",
                    record.AgentInstanceId,
                    _currentFilePath);

                // Update success metrics
                Interlocked.Increment(ref _successfulAppendCount);
                _appendSuccessCounter.Add(1);
            }
            catch (Exception ex)
            {
                _logger.LogError(
                    ex,
                    "Failed to append record from agent {AgentId}",
                    record.AgentInstanceId);

                // Update failure metrics
                Interlocked.Increment(ref _failedAppendCount);
                _appendFailureCounter.Add(1);

                // Suppress the error to preserve at-most-once semantics; callers should rely on diagnostics/logs.
            }
        }
    }

    /// <summary>
    /// Flush buffered writes to disk.
    /// </summary>
    public async Task FlushAsync(CancellationToken cancellationToken = default)
    {
        using (await AcquireLockAsync(cancellationToken))
        {
            if (_currentWriter != null)
            {
                await _currentWriter.FlushAsync(cancellationToken);
                _logger.LogDebug("Flushed storage to {FilePath}", _currentFilePath);
            }
        }
    }

    /// <summary>
    /// Rotate to a new file.
    /// 
    /// File naming: {prefix}-{timestamp:yyyyMMdd-HHmmss-fff}.jsonl
    /// </summary>
    private async Task RotateFileAsync()
    {
        // Close current writer if open
        if (_currentWriter != null)
        {
            await _currentWriter.DisposeAsync();
            _logger.LogInformation(
                "Rotated storage file: {FilePath} ({SizeBytes} bytes)",
                _currentFilePath,
                _currentFileSize);
        }

        // Create new file with millisecond precision to avoid collisions
        var timestamp = DateTime.UtcNow.ToString("yyyyMMdd-HHmmss-fff");
        _currentFilePath = Path.Combine(
            _config.BaseDirectory,
            $"{_config.FilePrefix}-{timestamp}.jsonl");

        _currentWriter = new StreamWriter(
            _currentFilePath,
            append: false,
            encoding: System.Text.Encoding.UTF8,
            bufferSize: 8192);

        _currentFileSize = 0;

        _logger.LogInformation("Created new storage file: {FilePath}", _currentFilePath);
    }

    public async ValueTask DisposeAsync()
    {
        if (Interlocked.Exchange(ref _disposed, 1) != 0)
        {
            return; // Already disposed
        }

        await _writeLock.WaitAsync();
        try
        {
            if (_currentWriter != null)
            {
                await _currentWriter.DisposeAsync();
                _currentWriter = null;
            }
        }
        finally
        {
            _writeLock.Release();
        }
        _writeLock.Dispose();

    }

    /// <summary>
    /// Acquire the write lock asynchronously and return a disposable that releases it.
    /// </summary>
    private async Task<IDisposable> AcquireLockAsync(CancellationToken cancellationToken = default)
    {
        await _writeLock.WaitAsync(cancellationToken);
        return new LockReleaser(_writeLock);
    }

    /// <summary>
    /// Helper struct to release the semaphore when disposed.
    /// </summary>
    private struct LockReleaser : IDisposable
    {
        private readonly SemaphoreSlim _semaphore;
        private bool _disposed;

        public LockReleaser(SemaphoreSlim semaphore)
        {
            _semaphore = semaphore ?? throw new ArgumentNullException(nameof(semaphore));
        }

        public void Dispose()
        {
            if (_disposed)
            {
                return;
            }

            _disposed = true;

            try
            {
                _semaphore.Release();
            }
            catch (ObjectDisposedException)
            {
                // Semaphore was already disposed, ignore
            }
        }
    }

    public void Dispose()
    {
        if (Interlocked.Exchange(ref _disposed, 1) != 0)
        {
            return; // Already disposed
        }

        // Use synchronous disposal - safe for synchronous contexts
        // For async contexts, prefer DisposeAsync()
        _writeLock.Wait();
        try
        {
            if (_currentWriter != null)
            {
                _currentWriter.Dispose();
                _currentWriter = null;
            }
        }
        finally
        {
            _writeLock.Release();
        }
        _writeLock.Dispose();
    }
}
