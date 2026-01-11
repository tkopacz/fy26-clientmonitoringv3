use crate::protocol::{
    Envelope, FrameCodec, Message, MessagePayload, MessageType, OsType, ProcessSample,
    ProtocolVersion, SnapshotPayload,
};
use std::fmt::Write as _;
use std::io;
use std::path::{Path, PathBuf};

pub const DEFAULT_DEMO_PATH: &str = "tmp/demo-protocol.bin";
pub const MAX_FRAME_COUNT: u32 = 1000;

pub const EXIT_USAGE: i32 = 2;

#[derive(Debug, Clone)]
pub struct ProducerArgs {
    pub out_path: PathBuf,
    pub count: u32,
}

impl ProducerArgs {
    pub fn parse_from_env() -> Result<Self, String> {
        let mut out_path: PathBuf = DEFAULT_DEMO_PATH.into();
        let mut count: u32 = 1;

        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--out" => {
                    let value = args
                        .next()
                        .ok_or_else(|| "--out requires a <path> value".to_string())?;
                    out_path = PathBuf::from(value);
                }
                "--count" => {
                    let value = args
                        .next()
                        .ok_or_else(|| "--count requires a <n> value".to_string())?;
                    count = value
                        .parse::<u32>()
                        .map_err(|_| format!("Invalid --count value: {value}"))?;
                }
                "-h" | "--help" => {
                    return Err("help".to_string());
                }
                other => {
                    return Err(format!("Unknown argument: {other}"));
                }
            }
        }

        if !(1..=MAX_FRAME_COUNT).contains(&count) {
            return Err(format!(
                "--count must be in 1..={MAX_FRAME_COUNT} (got {count})"
            ));
        }

        Ok(Self { out_path, count })
    }

    pub fn usage() -> String {
        format!(
            "demo_protocol_producer --out <path> --count <n>\n\nDefaults:\n  --out   {DEFAULT_DEMO_PATH}\n  --count 1\n\nConstraints:\n  1 <= --count <= {MAX_FRAME_COUNT}\n"
        )
    }
}

pub fn build_demo_message(platform: OsType) -> Message {
    let envelope = Envelope {
        version: ProtocolVersion::CURRENT,
        message_type: MessageType::Snapshot,
        message_id: demo_message_id(),
        timestamp_utc_ms: 1703174410000,
        agent_id: "demo-agent-ž".to_string(),
        platform,
        compressed: false,
    };

    let payload = SnapshotPayload {
        window_start_secs: 1703174400,
        window_end_secs: 1703174410,
        total_cpu_percent: 12.345_f32,
        memory_used_bytes: 1_500_000_000,
        memory_total_bytes: 8_000_000_000,
        processes: vec![
            ProcessSample {
                pid: 1234,
                name: "demo-π".to_string(),
                cpu_percent: 1.234_f32,
                memory_percent: 0.321_f32,
                memory_bytes: 50_000_000,
                cmdline: Some("/usr/bin/demo --mode=π".to_string()),
            },
            ProcessSample {
                pid: 5678,
                name: "worker".to_string(),
                cpu_percent: 0.500_f32,
                memory_percent: 0.111_f32,
                memory_bytes: 75_000_000,
                cmdline: None,
            },
        ],
        truncated: false,
    };

    Message {
        envelope,
        payload: MessagePayload::Snapshot(payload),
    }
}

fn demo_message_id() -> [u8; 16] {
    // 16 deterministic bytes.
    [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
        0x0f,
    ]
}

pub fn encode_demo_frame_bytes(
    message: &Message,
) -> Result<Vec<u8>, crate::protocol::ProtocolError> {
    FrameCodec::encode(message)
}

pub fn format_message_for_console(message: &Message, frame_index_1_based: usize) -> String {
    let mut out = String::new();

    let _ = writeln!(&mut out, "Frame {frame_index_1_based}:");

    // Envelope fields (FR-014 order)
    let _ = writeln!(&mut out, "version_major={}", message.envelope.version.major);
    let _ = writeln!(&mut out, "version_minor={}", message.envelope.version.minor);
    let _ = writeln!(
        &mut out,
        "message_type={}",
        format_message_type(message.envelope.message_type)
    );
    let _ = writeln!(
        &mut out,
        "message_id={}",
        format_message_id_hex(&message.envelope.message_id)
    );
    let _ = writeln!(
        &mut out,
        "timestamp_utc_ms={}",
        message.envelope.timestamp_utc_ms
    );
    let _ = writeln!(&mut out, "agent_id={}", message.envelope.agent_id);
    let _ = writeln!(
        &mut out,
        "platform={}",
        format_platform(message.envelope.platform)
    );
    let _ = writeln!(
        &mut out,
        "compressed={}",
        bool_to_lower(message.envelope.compressed)
    );

    // Snapshot fields (FR-014 order)
    if let MessagePayload::Snapshot(snapshot) = &message.payload {
        let _ = writeln!(&mut out, "window_start_secs={}", snapshot.window_start_secs);
        let _ = writeln!(&mut out, "window_end_secs={}", snapshot.window_end_secs);
        let _ = writeln!(
            &mut out,
            "total_cpu_percent={}",
            format_f32_3(snapshot.total_cpu_percent)
        );
        let _ = writeln!(&mut out, "memory_used_bytes={}", snapshot.memory_used_bytes);
        let _ = writeln!(
            &mut out,
            "memory_total_bytes={}",
            snapshot.memory_total_bytes
        );
        let _ = writeln!(&mut out, "process_count={}", snapshot.processes.len());

        for (i, p) in snapshot.processes.iter().enumerate() {
            let n = i + 1;
            let _ = writeln!(&mut out, "process[{n}].pid={}", p.pid);
            let _ = writeln!(&mut out, "process[{n}].name={}", p.name);
            let _ = writeln!(
                &mut out,
                "process[{n}].cpu_percent={}",
                format_f32_3(p.cpu_percent)
            );
            let _ = writeln!(
                &mut out,
                "process[{n}].memory_percent={}",
                format_f32_3(p.memory_percent)
            );
            let _ = writeln!(&mut out, "process[{n}].memory_bytes={}", p.memory_bytes);
            match &p.cmdline {
                Some(cmd) => {
                    let _ = writeln!(&mut out, "process[{n}].cmdline={cmd}");
                }
                None => {
                    let _ = writeln!(&mut out, "process[{n}].cmdline=<absent>");
                }
            }
        }

        let _ = writeln!(&mut out, "truncated={}", bool_to_lower(snapshot.truncated));
    }

    out
}

fn format_message_id_hex(bytes: &[u8; 16]) -> String {
    let mut s = String::with_capacity(32);
    for b in bytes {
        let _ = write!(&mut s, "{b:02x}");
    }
    s
}

fn format_message_type(t: MessageType) -> &'static str {
    match t {
        MessageType::Handshake => "Handshake",
        MessageType::HandshakeAck => "HandshakeAck",
        MessageType::Heartbeat => "Heartbeat",
        MessageType::Snapshot => "Snapshot",
        MessageType::Ack => "Ack",
        MessageType::Backpressure => "Backpressure",
        MessageType::Error => "Error",
    }
}

fn format_platform(p: OsType) -> &'static str {
    match p {
        OsType::Windows => "Windows",
        OsType::Linux => "Linux",
    }
}

fn bool_to_lower(b: bool) -> &'static str {
    if b {
        "true"
    } else {
        "false"
    }
}

fn format_f32_3(v: f32) -> String {
    format!("{v:.3}")
}

pub fn resolve_path_for_logging(path: &Path) -> io::Result<PathBuf> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    Ok(std::env::current_dir()?.join(path))
}

pub fn version_stamp_if_exists(path: &Path) -> io::Result<PathBuf> {
    if !path.exists() {
        return Ok(path.to_path_buf());
    }

    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("demo-protocol");
    let ext = path.extension().and_then(|e| e.to_str());

    for i in 1..=999_u32 {
        let candidate_name = match ext {
            Some(ext) if !ext.is_empty() => format!("{stem}-{i:03}.{ext}"),
            _ => format!("{stem}-{i:03}"),
        };
        let candidate = parent.join(candidate_name);
        if !candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "Could not find an available version-stamped output file name",
    ))
}
