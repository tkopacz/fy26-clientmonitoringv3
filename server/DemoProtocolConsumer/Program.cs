using System.Globalization;
using MonitoringServer.Protocol;

namespace DemoProtocolConsumer;

internal static class Program
{
    private const int ExitSuccess = 0;
    private const int ExitUsage = 2;

    private const int ExitMissingFile = 10;
    private const int ExitEmptyFile = 11;
    private const int ExitInvalidFrame = 12;
    private const int ExitTrailingBytes = 13;
    private const int ExitCrcMismatch = 14;
    private const int ExitUnsupportedVersion = 15;
    private const int ExitFrameTooLarge = 16;

    private const string DefaultInputPath = "tmp/demo-protocol.bin";

    public static async Task<int> Main(string[] args)
    {
        string inputPath;
        try
        {
            inputPath = ParseInputPath(args);
        }
        catch (UsageException ex)
        {
            if (ex.Message == "help")
            {
                Console.WriteLine(UsageText());
                return ExitSuccess;
            }

            Console.Error.WriteLine($"UsageError: {ex.Message}\n\n{UsageText()}");
            return ExitUsage;
        }

        var resolvedInput = Path.GetFullPath(inputPath);

        if (!File.Exists(resolvedInput))
        {
            WriteError(
                category: "MissingFile",
                inputPath: resolvedInput,
                reason: "File does not exist");
            return ExitMissingFile;
        }

        var info = new FileInfo(resolvedInput);
        if (info.Length == 0)
        {
            WriteError(
                category: "EmptyFile",
                inputPath: resolvedInput,
                reason: "File is empty");
            return ExitEmptyFile;
        }

        var decodedAny = false;
        var frameIndex = 0;

        try
        {
            await using var stream = File.OpenRead(resolvedInput);

            Console.WriteLine($"Reading in='{resolvedInput}'");

            while (stream.Position < stream.Length)
            {
                frameIndex++;
                var message = await FrameCodec.DecodeAsync(stream);
                decodedAny = true;

                var expected = ProtocolVersion.Current;
                if (message.Envelope.Version != expected)
                {
                    var found = message.Envelope.Version;
                    WriteError(
                        category: "UnsupportedVersion",
                        inputPath: resolvedInput,
                        reason: $"Found {found.Major}.{found.Minor}, expected {expected.Major}.{expected.Minor}");
                    return ExitUnsupportedVersion;
                }

                PrintMessage(message, frameIndex);
            }

            return ExitSuccess;
        }
        catch (FrameTooLargeException ex)
        {
            WriteError(
                category: "FrameTooLarge",
                inputPath: resolvedInput,
                reason: ex.Message);
            return ExitFrameTooLarge;
        }
        catch (Crc32MismatchException ex)
        {
            WriteError(
                category: "CrcMismatch",
                inputPath: resolvedInput,
                reason: ex.Message);
            return ExitCrcMismatch;
        }
        catch (EndOfStreamException ex)
        {
            var category = decodedAny ? "TrailingBytes" : "InvalidFrame";
            var code = decodedAny ? ExitTrailingBytes : ExitInvalidFrame;
            WriteError(
                category: category,
                inputPath: resolvedInput,
                reason: ex.Message);
            return code;
        }
        catch (Exception ex)
        {
            WriteError(
                category: "InvalidFrame",
                inputPath: resolvedInput,
                reason: ex.Message);
            return ExitInvalidFrame;
        }
    }

    private static string ParseInputPath(string[] args)
    {
        var inputPath = DefaultInputPath;

        for (var i = 0; i < args.Length; i++)
        {
            var arg = args[i];
            switch (arg)
            {
                case "--in":
                    if (i + 1 >= args.Length)
                    {
                        throw new UsageException("--in requires a <path> value");
                    }
                    inputPath = args[++i];
                    break;
                case "-h":
                case "--help":
                    throw new UsageException("help");
                default:
                    throw new UsageException($"Unknown argument: {arg}");
            }
        }

        return inputPath;
    }

    private static string UsageText() =>
        "DemoProtocolConsumer --in <path>\n\nDefaults:\n  --in   tmp/demo-protocol.bin\n\nExit codes:\n  0  Success\n  2  Usage / invalid CLI args\n  10 MissingFile\n  11 EmptyFile\n  12 InvalidFrame\n  13 TrailingBytes\n  14 CrcMismatch\n  15 UnsupportedVersion\n  16 FrameTooLarge\n";

    private static void PrintMessage(Message message, int frameIndex1Based)
    {
        Console.WriteLine($"Frame {frameIndex1Based}:");

        var env = message.Envelope;

        Console.WriteLine($"version_major={env.Version.Major}");
        Console.WriteLine($"version_minor={env.Version.Minor}");
        Console.WriteLine($"message_type={env.MessageType}");
        Console.WriteLine($"message_id={ToLowerHex(env.MessageId)}");
        Console.WriteLine($"timestamp_utc_ms={env.TimestampUtcMs}");
        Console.WriteLine($"agent_id={env.AgentId}");
        Console.WriteLine($"platform={env.Platform}");
        Console.WriteLine($"compressed={env.Compressed.ToString().ToLowerInvariant()}");

        if (message.Payload is not MessagePayload.Snapshot snapshot)
        {
            throw new InvalidOperationException("Expected Snapshot payload");
        }

        var p = snapshot.Payload;
        Console.WriteLine($"window_start_secs={p.WindowStartSecs}");
        Console.WriteLine($"window_end_secs={p.WindowEndSecs}");
        Console.WriteLine($"total_cpu_percent={FormatF32(p.TotalCpuPercent)}");
        Console.WriteLine($"memory_used_bytes={p.MemUsedBytes}");
        Console.WriteLine($"memory_total_bytes={p.MemTotalBytes}");
        Console.WriteLine($"process_count={p.Processes.Count}");

        for (var i = 0; i < p.Processes.Count; i++)
        {
            var idx = i + 1;
            var proc = p.Processes[i];

            Console.WriteLine($"process[{idx}].pid={proc.Pid}");
            Console.WriteLine($"process[{idx}].name={proc.Name}");
            Console.WriteLine($"process[{idx}].cpu_percent={FormatF32(proc.CpuPercent)}");
            Console.WriteLine($"process[{idx}].memory_percent={FormatF32(proc.MemoryPercent)}");
            Console.WriteLine($"process[{idx}].memory_bytes={proc.MemoryBytes}");
            Console.WriteLine($"process[{idx}].cmdline={(proc.Cmdline is null ? "<absent>" : proc.Cmdline)}");
        }

        Console.WriteLine($"truncated={p.Truncated.ToString().ToLowerInvariant()}");
    }

    private static string FormatF32(float value) => value.ToString("F3", CultureInfo.InvariantCulture);

    private static string ToLowerHex(byte[] bytes) => Convert.ToHexString(bytes).ToLowerInvariant();

    private static void WriteError(string category, string inputPath, string reason) =>
        Console.Error.WriteLine($"{category}: path='{inputPath}' reason='{reason}'");

    private sealed class UsageException(string message) : Exception(message);
}
