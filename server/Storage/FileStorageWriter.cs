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
public sealed class FileStorageWriter : IStorageWriter, IDisposable
{
    private readonly FileStorageConfig _config;
    private readonly ILogger<FileStorageWriter> _logger;
    private readonly SemaphoreSlim _writeLock = new(1, 1);
    private StreamWriter? _currentWriter;
    private string? _currentFilePath;
    private long _currentFileSize;

    private readonly JsonSerializerOptions _jsonOptions = new()
    {
        WriteIndented = false,
        DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull
    };

    public FileStorageWriter(
        FileStorageConfig config,
        ILogger<FileStorageWriter> logger)
    {
        _config = config ?? throw new ArgumentNullException(nameof(config));
        _logger = logger ?? throw new ArgumentNullException(nameof(logger));

        // Ensure base directory exists
        Directory.CreateDirectory(_config.BaseDirectory);
    }

    /// <summary>
    /// Append a record to the current file.
    /// 
    /// Automatically rotates to a new file if size threshold is exceeded.
    /// Thread-safe with internal locking.
    /// </summary>
    public async Task AppendAsync(
        StorageRecord record,
        CancellationToken cancellationToken = default)
    {
        await _writeLock.WaitAsync(cancellationToken);
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
            _currentFileSize += json.Length + Environment.NewLine.Length;

            _logger.LogDebug(
                "Appended record from agent {AgentId} to {FilePath}",
                record.AgentInstanceId,
                _currentFilePath);
        }
        catch (Exception ex)
        {
            _logger.LogError(
                ex,
                "Failed to append record from agent {AgentId}",
                record.AgentInstanceId);
            // At-most-once semantics: log error but don't throw
        }
        finally
        {
            _writeLock.Release();
        }
    }

    /// <summary>
    /// Flush buffered writes to disk.
    /// </summary>
    public async Task FlushAsync(CancellationToken cancellationToken = default)
    {
        await _writeLock.WaitAsync(cancellationToken);
        try
        {
            if (_currentWriter != null)
            {
                await _currentWriter.FlushAsync();
                _logger.LogDebug("Flushed storage to {FilePath}", _currentFilePath);
            }
        }
        finally
        {
            _writeLock.Release();
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

    public void Dispose()
    {
        _writeLock.Wait();
        try
        {
            _currentWriter?.Dispose();
            _writeLock.Dispose();
        }
        finally
        {
            // Lock already disposed
        }
    }
}
