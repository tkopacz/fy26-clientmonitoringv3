/// <summary>
/// Integration tests for file storage writer.
/// 
/// Tests cover:
/// - Append operations
/// - File rotation
/// - Concurrent writes
/// - Flush behavior
/// </summary>

using Microsoft.Extensions.Logging.Abstractions;
using MonitoringServer.Protocol;
using MonitoringServer.Storage;
using Xunit;

namespace MonitoringServer.Tests.Storage;

public class FileStorageTests : IDisposable
{
    private readonly string _testDirectory;

    public FileStorageTests()
    {
        _testDirectory = Path.Combine(
            Path.GetTempPath(),
            $"monitoring-test-{Guid.NewGuid()}");
        Directory.CreateDirectory(_testDirectory);
    }

    public void Dispose()
    {
        if (Directory.Exists(_testDirectory))
        {
            Directory.Delete(_testDirectory, recursive: true);
        }
    }

    [Fact]
    public async Task FileStorage_AppendRecord_CreatesFile()
    {
        var config = new FileStorageConfig
        {
            BaseDirectory = _testDirectory,
            FilePrefix = "test"
        };

        using var storage = new FileStorageWriter(config, NullLogger<FileStorageWriter>.Instance);

        var record = CreateTestRecord("agent-001");
        await storage.AppendAsync(record);
        await storage.FlushAsync();

        // Verify file was created
        var files = Directory.GetFiles(_testDirectory, "test-*.jsonl");
        Assert.Single(files);

        // Verify content
        var content = await File.ReadAllTextAsync(files[0]);
        Assert.Contains("agent-001", content);
        Assert.Contains("test-process", content);
    }

    [Fact]
    public async Task FileStorage_AppendMultipleRecords_WritesAll()
    {
        var config = new FileStorageConfig
        {
            BaseDirectory = _testDirectory,
            FilePrefix = "test"
        };

        using var storage = new FileStorageWriter(config, NullLogger<FileStorageWriter>.Instance);

        // Write multiple records
        for (int i = 0; i < 10; i++)
        {
            var record = CreateTestRecord($"agent-{i:D3}");
            await storage.AppendAsync(record);
        }

        await storage.FlushAsync();

        // Verify all records written
        var files = Directory.GetFiles(_testDirectory, "test-*.jsonl");
        Assert.Single(files);

        var lines = await File.ReadAllLinesAsync(files[0]);
        Assert.Equal(10, lines.Length);
    }

    [Fact]
    public async Task FileStorage_FileRotation_CreatesNewFile()
    {
        var config = new FileStorageConfig
        {
            BaseDirectory = _testDirectory,
            FilePrefix = "test",
            MaxFileSizeBytes = 1024 // Small size to trigger rotation
        };

        using var storage = new FileStorageWriter(config, NullLogger<FileStorageWriter>.Instance);

        // Write enough records to trigger rotation
        for (int i = 0; i < 50; i++)
        {
            var record = CreateTestRecord($"agent-{i:D3}");
            await storage.AppendAsync(record);
        }

        await storage.FlushAsync();

        // Should have created multiple files due to rotation
        var files = Directory.GetFiles(_testDirectory, "test-*.jsonl");
        Assert.True(files.Length > 1, "Expected file rotation to create multiple files");
    }

    [Fact]
    public async Task FileStorage_ConcurrentWrites_ThreadSafe()
    {
        var config = new FileStorageConfig
        {
            BaseDirectory = _testDirectory,
            FilePrefix = "concurrent"
        };

        using var storage = new FileStorageWriter(config, NullLogger<FileStorageWriter>.Instance);

        // Concurrent writes from multiple tasks
        var tasks = Enumerable.Range(0, 100)
            .Select(i => Task.Run(async () =>
            {
                var record = CreateTestRecord($"agent-{i:D3}");
                await storage.AppendAsync(record);
            }))
            .ToArray();

        await Task.WhenAll(tasks);
        await storage.FlushAsync();

        // Verify all records written
        var files = Directory.GetFiles(_testDirectory, "concurrent-*.jsonl");
        var totalLines = files.Sum(f => File.ReadLines(f).Count());
        Assert.Equal(100, totalLines);
    }

    private static StorageRecord CreateTestRecord(string agentId)
    {
        return new StorageRecord
        {
            AgentInstanceId = agentId,
            ReceivedTimestampSecs = DateTimeOffset.UtcNow.ToUnixTimeSeconds(),
            Snapshot = new SnapshotPayload
            {
                WindowStartSecs = 1703174400,
                WindowEndSecs = 1703174410,
                TotalCpuPercent = 50.0f,
                MemUsedBytes = 7_500_000_000,
                MemTotalBytes = 8_000_000_000,
                Processes = new List<ProcessSample>
                {
                    new()
                    {
                        Pid = 1234,
                        Name = "test-process",
                        CpuPercent = 25.0f,
                        MemoryPercent = 1.25f,
                        MemoryBytes = 100_000_000,
                        Cmdline = "/usr/bin/test"
                    }
                },
                Truncated = false
            }
        };
    }
}
