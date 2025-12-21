/// Binary protocol definitions and encoding for monitoring agent.
///
/// This module implements the versioned binary protocol with:
/// - Framing: length-prefix + message type
/// - Envelope: version, platform, timestamps, agent ID
/// - zstd compression (level 3, negotiated)
/// - At-most-once delivery semantics

use serde::de::Error as SerdeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::io::{self, Read, Write, Cursor};

/// Protocol version (MAJOR.MINOR format).
///
/// - MAJOR: Breaking changes to message structure
/// - MINOR: Backward-compatible additions (optional fields)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolVersion {
    pub major: u8,
    pub minor: u8,
}

impl ProtocolVersion {
    /// Current protocol version: 1.0
    pub const CURRENT: Self = Self { major: 1, minor: 0 };

    /// Check if this version is compatible with another version.
    ///
    /// Compatible if major versions match and minor >= other.minor.
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        self.major == other.major && self.minor >= other.minor
    }
}

/// Message types in the protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageType {
    /// Initial handshake from agent to server
    Handshake = 1,
    /// Server acknowledgment of handshake
    HandshakeAck = 2,
    /// Periodic keepalive/heartbeat
    Heartbeat = 3,
    /// Monitoring snapshot with CPU/memory/processes
    Snapshot = 4,
    /// Message acknowledgment (success/failure)
    Ack = 5,
    /// Backpressure signal from server
    Backpressure = 6,
    /// Error notification
    Error = 7,
}

impl MessageType {
    /// Convert from u8 discriminant
    pub fn from_u8(value: u8) -> Result<Self, ProtocolError> {
        match value {
            1 => Ok(MessageType::Handshake),
            2 => Ok(MessageType::HandshakeAck),
            3 => Ok(MessageType::Heartbeat),
            4 => Ok(MessageType::Snapshot),
            5 => Ok(MessageType::Ack),
            6 => Ok(MessageType::Backpressure),
            7 => Ok(MessageType::Error),
            _ => Err(ProtocolError::InvalidMessageType(value)),
        }
    }

    /// Convert to u8 discriminant
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

impl Serialize for MessageType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(self.to_u8())
    }
}

impl<'de> Deserialize<'de> for MessageType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        MessageType::from_u8(value).map_err(SerdeError::custom)
    }
}

/// Operating system type for agent identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum OsType {
    Windows = 1,
    Linux = 2,
}

/// Agent identity information sent during handshake.
///
/// Includes instance ID, OS type, version, and capability flags.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentIdentity {
    /// Unique instance identifier for this agent
    pub instance_id: String,
    /// Operating system type
    pub os_type: OsType,
    /// Agent version string
    pub agent_version: String,
    /// Protocol version supported by this agent
    pub protocol_version: ProtocolVersion,
    /// Capability flags (bit 0: supports all-process mode, bit 1: compression)
    pub capabilities: u32,
}

impl AgentIdentity {
    /// Capability flag: supports all-process mode
    pub const CAP_ALL_PROCESS: u32 = 0x01;
    /// Capability flag: supports zstd compression
    pub const CAP_COMPRESSION: u32 = 0x02;

    /// Check if agent supports all-process mode
    pub fn supports_all_process(&self) -> bool {
        (self.capabilities & Self::CAP_ALL_PROCESS) != 0
    }

    /// Check if agent supports compression
    pub fn supports_compression(&self) -> bool {
        (self.capabilities & Self::CAP_COMPRESSION) != 0
    }
}

/// Single process sample in a snapshot.
///
/// Ordered by CPU usage (descending) when truncation is applied.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessSample {
    /// Process ID
    pub pid: u32,
    /// Process name
    pub name: String,
    /// CPU usage percentage (0.0 - 100.0)
    pub cpu_percent: f32,
    /// Memory usage in bytes (RSS)
    pub memory_bytes: u64,
    /// Optional command line (may be omitted for privacy/performance)
    pub cmdline: Option<String>,
}

/// Monitoring snapshot payload.
///
/// Contains aggregated CPU/memory metrics and per-process samples.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapshotPayload {
    /// Sampling window start timestamp (Unix epoch seconds)
    pub window_start_secs: i64,
    /// Sampling window end timestamp (Unix epoch seconds)
    pub window_end_secs: i64,
    /// Aggregate CPU usage percentage (0.0 - 100.0)
    pub total_cpu_percent: f32,
    /// Aggregate memory usage in bytes
    pub total_memory_bytes: u64,
    /// Process samples (ordered by CPU, truncated if needed)
    pub processes: Vec<ProcessSample>,
    /// True if process list was truncated to fit size cap
    pub truncated: bool,
}

/// Backpressure signal from server to agent.
///
/// Instructs agent to slow or pause sending.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackpressureSignal {
    /// Throttle level (0 = normal, 1 = slow down, 2 = pause)
    pub level: u8,
    /// Optional pause duration in seconds
    pub pause_secs: Option<u32>,
}

/// Message acknowledgment with status and optional error code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageAck {
    /// ID of the message being acknowledged
    pub message_id: u64,
    /// Success flag
    pub success: bool,
    /// Optional error code if success=false
    pub error_code: Option<u32>,
}

/// Protocol envelope containing message metadata.
///
/// Wraps the payload with version, type, ID, timestamps, and compression flag.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Envelope {
    /// Protocol version
    pub version: ProtocolVersion,
    /// Message type
    pub message_type: MessageType,
    /// Unique message ID for correlation
    pub message_id: u64,
    /// Timestamp when message was created (Unix epoch seconds)
    pub timestamp_secs: i64,
    /// True if payload is zstd-compressed
    pub compressed: bool,
}

/// Top-level message container.
///
/// Combines envelope with the actual message payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    /// Message envelope with metadata
    pub envelope: Envelope,
    /// Message payload (varies by type)
    pub payload: MessagePayload,
}

/// Message payload variants.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MessagePayload {
    Handshake(AgentIdentity),
    HandshakeAck,
    Heartbeat,
    Snapshot(SnapshotPayload),
    Ack(MessageAck),
    Backpressure(BackpressureSignal),
    Error { code: u32, message: String },
}

/// Protocol errors.
#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("Invalid message type: {0}")]
    InvalidMessageType(u8),
    #[error("Incompatible protocol version")]
    IncompatibleVersion,
    #[error("Frame too large: {0} bytes (max {1})")]
    FrameTooLarge(usize, usize),
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Compression error: {0}")]
    Compression(String),
}

impl From<bincode::Error> for ProtocolError {
    fn from(err: bincode::Error) -> Self {
        ProtocolError::Serialization(err.to_string())
    }
}

/// Maximum uncompressed frame size (256 KB safeguard).
///
/// IMPORTANT: This value must be kept in sync with the corresponding
/// `FrameCodec.MaxFrameSize` constant in the C# implementation.
/// If you change this limit here, update the C# value as well to avoid
/// protocol incompatibilities between agent and server.
const MAX_FRAME_SIZE: usize = 256 * 1024;

/// Target compressed frame size (64 KB)
pub const TARGET_FRAME_SIZE: usize = 64 * 1024;

/// Wire format framing and encoding.
///
/// Framing: [length: u32 big-endian][message bytes]
/// Message bytes: bincode-encoded Message, optionally zstd-compressed
pub struct FrameCodec;

impl FrameCodec {
    /// Encode a message into a framed byte buffer.
    ///
    /// Steps:
    /// 1. Serialize message to bytes (bincode)
    /// 2. Compress if envelope.compressed=true (zstd level 3)
    /// 3. Prepend 4-byte length (big-endian)
    ///
    /// Returns: framed bytes ready to write to socket
    pub fn encode(message: &Message) -> Result<Vec<u8>, ProtocolError> {
        let mut body = Vec::with_capacity(128);

        // Envelope header layout: multi-byte envelope fields (message_id, timestamp_secs) are encoded in
        // little-endian; single-byte fields (version bytes, message type, compressed flag) have no
        // endianness. The 4-byte frame length prefix is big-endian (see encode docs), while the envelope
        // and payload multi-byte fields are little-endian. The .NET FrameCodec must read/write the same
        // layout.
        body.push(message.envelope.version.major);
        body.push(message.envelope.version.minor);
        body.push(message.envelope.message_type.to_u8());
        body.extend_from_slice(&message.envelope.message_id.to_le_bytes());
        body.extend_from_slice(&message.envelope.timestamp_secs.to_le_bytes());
        body.push(if message.envelope.compressed { 1 } else { 0 });

        // Serialize payload based on message type
        let mut payload_bytes = Vec::with_capacity(256);
        match &message.payload {
            MessagePayload::Handshake(identity) => {
                write_string(&mut payload_bytes, &identity.instance_id);
                payload_bytes.push(identity.os_type as u8);
                write_string(&mut payload_bytes, &identity.agent_version);
                payload_bytes.push(identity.protocol_version.major);
                payload_bytes.push(identity.protocol_version.minor);
                payload_bytes.extend_from_slice(&identity.capabilities.to_le_bytes());
            }
            MessagePayload::HandshakeAck => {}
            MessagePayload::Heartbeat => {}
            MessagePayload::Snapshot(snapshot) => {
                payload_bytes.extend_from_slice(&snapshot.window_start_secs.to_le_bytes());
                payload_bytes.extend_from_slice(&snapshot.window_end_secs.to_le_bytes());
                payload_bytes.extend_from_slice(&snapshot.total_cpu_percent.to_le_bytes());
                payload_bytes.extend_from_slice(&snapshot.total_memory_bytes.to_le_bytes());

                let count = snapshot.processes.len() as u64;
                payload_bytes.extend_from_slice(&count.to_le_bytes());
                for process in &snapshot.processes {
                    payload_bytes.extend_from_slice(&process.pid.to_le_bytes());
                    write_string(&mut payload_bytes, &process.name);
                    payload_bytes.extend_from_slice(&process.cpu_percent.to_le_bytes());
                    payload_bytes.extend_from_slice(&process.memory_bytes.to_le_bytes());
                    write_optional_string(&mut payload_bytes, process.cmdline.as_deref());
                }

                payload_bytes.push(if snapshot.truncated { 1 } else { 0 });
            }
            MessagePayload::Ack(ack) => {
                payload_bytes.extend_from_slice(&ack.message_id.to_le_bytes());
                payload_bytes.push(if ack.success { 1 } else { 0 });
                write_optional_u32(&mut payload_bytes, ack.error_code);
            }
            MessagePayload::Backpressure(bp) => {
                payload_bytes.push(bp.level);
                write_optional_u32(&mut payload_bytes, bp.pause_secs);
            }
            MessagePayload::Error { code, message } => {
                payload_bytes.extend_from_slice(&code.to_le_bytes());
                write_string(&mut payload_bytes, message);
            }
        }

        // Compress payload bytes if requested; envelope stays uncompressed
        let encoded_payload = if message.envelope.compressed {
            zstd::encode_all(payload_bytes.as_slice(), 3)
                .map_err(|e| ProtocolError::Compression(e.to_string()))?
        } else {
            payload_bytes
        };

        // Final buffer = envelope + payload (compressed or not)
        body.extend_from_slice(&encoded_payload);

        // Check size constraints
        if body.len() > MAX_FRAME_SIZE {
            return Err(ProtocolError::FrameTooLarge(body.len(), MAX_FRAME_SIZE));
        }

        // Build frame: [length:u32][body]
        let mut frame = Vec::with_capacity(4 + body.len());
        frame.extend_from_slice(&(body.len() as u32).to_be_bytes());
        frame.extend_from_slice(&body);

        Ok(frame)
    }

    /// Decode a message from a reader.
    ///
    /// Steps:
    /// 1. Read 4-byte length (big-endian)
    /// 2. Read payload bytes
    /// 3. Decompress if compressed (auto-detect zstd magic bytes)
    /// 4. Deserialize message (bincode)
    ///
    /// Returns: decoded Message
    pub fn decode<R: Read>(reader: &mut R) -> Result<Message, ProtocolError> {
        // Read length prefix
        let mut len_buf = [0u8; 4];
        reader.read_exact(&mut len_buf)?;
        let payload_len = u32::from_be_bytes(len_buf) as usize;

        // Validate length
        if payload_len > MAX_FRAME_SIZE {
            return Err(ProtocolError::FrameTooLarge(payload_len, MAX_FRAME_SIZE));
        }

        // Read payload
        let mut payload = vec![0u8; payload_len];
        reader.read_exact(&mut payload)?;

        let mut cursor = Cursor::new(&payload);

        // Envelope
        let mut version_buf = [0u8; 1];
        cursor.read_exact(&mut version_buf)?;
        let major = version_buf[0];
        cursor.read_exact(&mut version_buf)?;
        let minor = version_buf[0];

        let mut mt_buf = [0u8; 1];
        cursor.read_exact(&mut mt_buf)?;
        let message_type = MessageType::from_u8(mt_buf[0])?;

        let message_id = read_u64_le(&mut cursor)?;
        let timestamp_secs = read_i64_le(&mut cursor)?;
        let mut bool_buf = [0u8; 1];
        cursor.read_exact(&mut bool_buf)?;
        let compressed = bool_buf[0] != 0;

        // Remaining bytes are payload (possibly compressed)
        let remaining = payload[cursor.position() as usize..].to_vec();
        let payload_bytes = if compressed {
            zstd::decode_all(remaining.as_slice())
                .map_err(|e| ProtocolError::Compression(e.to_string()))?
        } else {
            remaining
        };

        let mut payload_cursor = Cursor::new(&payload_bytes);
        let payload = match message_type {
            MessageType::Handshake => {
                let instance_id = read_string(&mut payload_cursor)?;
                let os_type_raw = read_u8(&mut payload_cursor)?;
                let agent_version = read_string(&mut payload_cursor)?;
                let proto_major = read_u8(&mut payload_cursor)?;
                let proto_minor = read_u8(&mut payload_cursor)?;
                let capabilities = read_u32_le(&mut payload_cursor)?;

                MessagePayload::Handshake(AgentIdentity {
                    instance_id,
                    os_type: match os_type_raw {
                        1 => OsType::Windows,
                        2 => OsType::Linux,
                        other => return Err(ProtocolError::InvalidMessageType(other)),
                    },
                    agent_version,
                    protocol_version: ProtocolVersion {
                        major: proto_major,
                        minor: proto_minor,
                    },
                    capabilities,
                })
            }
            MessageType::HandshakeAck => MessagePayload::HandshakeAck,
            MessageType::Heartbeat => MessagePayload::Heartbeat,
            MessageType::Snapshot => {
                let window_start_secs = read_i64_le(&mut payload_cursor)?;
                let window_end_secs = read_i64_le(&mut payload_cursor)?;
                let total_cpu_percent = read_f32_le(&mut payload_cursor)?;
                let total_memory_bytes = read_u64_le(&mut payload_cursor)?;

                let process_count = read_u64_le(&mut payload_cursor)?;
                let mut processes = Vec::with_capacity(process_count as usize);
                for _ in 0..process_count {
                    let pid = read_u32_le(&mut payload_cursor)?;
                    let name = read_string(&mut payload_cursor)?;
                    let cpu_percent = read_f32_le(&mut payload_cursor)?;
                    let memory_bytes = read_u64_le(&mut payload_cursor)?;
                    let cmdline = read_optional_string(&mut payload_cursor)?;

                    processes.push(ProcessSample {
                        pid,
                        name,
                        cpu_percent,
                        memory_bytes,
                        cmdline,
                    });
                }

                let truncated = read_bool(&mut payload_cursor)?;

                MessagePayload::Snapshot(SnapshotPayload {
                    window_start_secs,
                    window_end_secs,
                    total_cpu_percent,
                    total_memory_bytes,
                    processes,
                    truncated,
                })
            }
            MessageType::Ack => {
                let message_id = read_u64_le(&mut payload_cursor)?;
                let success = read_bool(&mut payload_cursor)?;
                let error_code = read_optional_u32(&mut payload_cursor)?;

                MessagePayload::Ack(MessageAck {
                    message_id,
                    success,
                    error_code,
                })
            }
            MessageType::Backpressure => {
                let level = read_u8(&mut payload_cursor)?;
                let pause_secs = read_optional_u32(&mut payload_cursor)?;

                MessagePayload::Backpressure(BackpressureSignal { level, pause_secs })
            }
            MessageType::Error => {
                let code = read_u32_le(&mut payload_cursor)?;
                let message = read_string(&mut payload_cursor)?;
                MessagePayload::Error { code, message }
            }
        };

        Ok(Message {
            envelope: Envelope {
                version: ProtocolVersion { major, minor },
                message_type,
                message_id,
                timestamp_secs,
                compressed,
            },
            payload,
        })
    }

    /// Write a framed message to a writer.
    ///
    /// Encodes the message and writes the complete frame.
    pub fn write<W: Write>(writer: &mut W, message: &Message) -> Result<(), ProtocolError> {
        let frame = Self::encode(message)?;
        writer.write_all(&frame)?;
        writer.flush()?;
        Ok(())
    }
}

fn write_string(buf: &mut Vec<u8>, value: &str) {
    let bytes = value.as_bytes();
    buf.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    buf.extend_from_slice(bytes);
}

fn write_optional_string(buf: &mut Vec<u8>, value: Option<&str>) {
    match value {
        Some(v) => {
            buf.push(1);
            write_string(buf, v);
        }
        None => buf.push(0),
    }
}

fn write_optional_u32(buf: &mut Vec<u8>, value: Option<u32>) {
    match value {
        Some(v) => {
            buf.push(1);
            buf.extend_from_slice(&v.to_le_bytes());
        }
        None => buf.push(0),
    }
}

fn read_u8<R: Read>(reader: &mut R) -> Result<u8, ProtocolError> {
    let mut buf = [0u8; 1];
    reader.read_exact(&mut buf)?;
    Ok(buf[0])
}

fn read_bool<R: Read>(reader: &mut R) -> Result<bool, ProtocolError> {
    Ok(read_u8(reader)? != 0)
}

fn read_u32_le<R: Read>(reader: &mut R) -> Result<u32, ProtocolError> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

fn read_u64_le<R: Read>(reader: &mut R) -> Result<u64, ProtocolError> {
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)?;
    Ok(u64::from_le_bytes(buf))
}

fn read_i64_le<R: Read>(reader: &mut R) -> Result<i64, ProtocolError> {
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)?;
    Ok(i64::from_le_bytes(buf))
}

fn read_f32_le<R: Read>(reader: &mut R) -> Result<f32, ProtocolError> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(f32::from_le_bytes(buf))
}

fn read_string<R: Read>(reader: &mut R) -> Result<String, ProtocolError> {
    let len = read_u64_le(reader)? as usize;
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    String::from_utf8(buf).map_err(|e| ProtocolError::Serialization(e.to_string()))
}

fn read_optional_string<R: Read>(reader: &mut R) -> Result<Option<String>, ProtocolError> {
    let has_value = read_bool(reader)?;
    if has_value {
        Ok(Some(read_string(reader)?))
    } else {
        Ok(None)
    }
}

fn read_optional_u32<R: Read>(reader: &mut R) -> Result<Option<u32>, ProtocolError> {
    let has_value = read_bool(reader)?;
    if has_value {
        Ok(Some(read_u32_le(reader)?))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// Test: Protocol version compatibility check
    #[test]
    fn test_protocol_version_compatibility() {
        let v1_0 = ProtocolVersion { major: 1, minor: 0 };
        let v1_1 = ProtocolVersion { major: 1, minor: 1 };
        let v2_0 = ProtocolVersion { major: 2, minor: 0 };

        // Same version compatible
        assert!(v1_0.is_compatible_with(&v1_0));
        // Higher minor compatible with lower
        assert!(v1_1.is_compatible_with(&v1_0));
        // Lower minor NOT compatible with higher
        assert!(!v1_0.is_compatible_with(&v1_1));
        // Different major NOT compatible
        assert!(!v2_0.is_compatible_with(&v1_0));
    }

    /// Test: Message type conversion to/from u8
    #[test]
    fn test_message_type_conversion() {
        assert_eq!(MessageType::Handshake.to_u8(), 1);
        assert_eq!(MessageType::from_u8(1).unwrap(), MessageType::Handshake);
        assert_eq!(MessageType::Snapshot.to_u8(), 4);
        assert_eq!(MessageType::from_u8(4).unwrap(), MessageType::Snapshot);
        assert!(MessageType::from_u8(99).is_err());
    }

    /// Test: Agent identity capability flags
    #[test]
    fn test_agent_identity_capabilities() {
        let identity = AgentIdentity {
            instance_id: "test-001".to_string(),
            os_type: OsType::Linux,
            agent_version: "0.1.0".to_string(),
            protocol_version: ProtocolVersion::CURRENT,
            capabilities: AgentIdentity::CAP_ALL_PROCESS | AgentIdentity::CAP_COMPRESSION,
        };

        assert!(identity.supports_all_process());
        assert!(identity.supports_compression());

        let identity_no_caps = AgentIdentity {
            capabilities: 0,
            ..identity
        };
        assert!(!identity_no_caps.supports_all_process());
        assert!(!identity_no_caps.supports_compression());
    }

    /// Test: Encode and decode handshake message (uncompressed)
    #[test]
    fn test_encode_decode_handshake() {
        let identity = AgentIdentity {
            instance_id: "agent-123".to_string(),
            os_type: OsType::Windows,
            agent_version: "0.1.0".to_string(),
            protocol_version: ProtocolVersion::CURRENT,
            capabilities: AgentIdentity::CAP_COMPRESSION,
        };

        let message = Message {
            envelope: Envelope {
                version: ProtocolVersion::CURRENT,
                message_type: MessageType::Handshake,
                message_id: 1,
                timestamp_secs: 1703174400,
                compressed: false,
            },
            payload: MessagePayload::Handshake(identity.clone()),
        };

        // Encode
        let frame = FrameCodec::encode(&message).unwrap();
        assert!(frame.len() > 4); // At least length prefix + some payload

        // Decode
        let mut cursor = Cursor::new(frame);
        let decoded = FrameCodec::decode(&mut cursor).unwrap();

        assert_eq!(decoded.envelope.message_type, MessageType::Handshake);
        assert_eq!(decoded.envelope.message_id, 1);
        match decoded.payload {
            MessagePayload::Handshake(decoded_identity) => {
                assert_eq!(decoded_identity.instance_id, identity.instance_id);
                assert_eq!(decoded_identity.os_type, identity.os_type);
            }
            _ => panic!("Expected Handshake payload"),
        }
    }

    /// Test: Encode and decode snapshot message (uncompressed)
    #[test]
    fn test_encode_decode_snapshot() {
        let snapshot = SnapshotPayload {
            window_start_secs: 1703174400,
            window_end_secs: 1703174410,
            total_cpu_percent: 25.5,
            total_memory_bytes: 8_000_000_000,
            processes: vec![
                ProcessSample {
                    pid: 1234,
                    name: "test-process".to_string(),
                    cpu_percent: 15.2,
                    memory_bytes: 100_000_000,
                    cmdline: Some("/usr/bin/test".to_string()),
                },
                ProcessSample {
                    pid: 5678,
                    name: "another-process".to_string(),
                    cpu_percent: 10.3,
                    memory_bytes: 50_000_000,
                    cmdline: None,
                },
            ],
            truncated: false,
        };

        let message = Message {
            envelope: Envelope {
                version: ProtocolVersion::CURRENT,
                message_type: MessageType::Snapshot,
                message_id: 42,
                timestamp_secs: 1703174410,
                compressed: false,
            },
            payload: MessagePayload::Snapshot(snapshot.clone()),
        };

        // Encode and decode
        let frame = FrameCodec::encode(&message).unwrap();
        let mut cursor = Cursor::new(frame);
        let decoded = FrameCodec::decode(&mut cursor).unwrap();

        assert_eq!(decoded.envelope.message_type, MessageType::Snapshot);
        match decoded.payload {
            MessagePayload::Snapshot(decoded_snapshot) => {
                assert_eq!(decoded_snapshot.window_start_secs, snapshot.window_start_secs);
                assert_eq!(decoded_snapshot.processes.len(), 2);
                assert_eq!(decoded_snapshot.processes[0].pid, 1234);
                assert_eq!(decoded_snapshot.truncated, false);
            }
            _ => panic!("Expected Snapshot payload"),
        }
    }

    /// Test: Encode with compression and decode
    #[test]
    fn test_encode_decode_compressed() {
        let snapshot = SnapshotPayload {
            window_start_secs: 1703174400,
            window_end_secs: 1703174410,
            total_cpu_percent: 50.0,
            total_memory_bytes: 16_000_000_000,
            processes: (0..100)
                .map(|i| ProcessSample {
                    pid: 1000 + i,
                    name: format!("process-{}", i),
                    cpu_percent: 0.5,
                    memory_bytes: 10_000_000,
                    cmdline: Some(format!("/usr/bin/app-{}", i)),
                })
                .collect(),
            truncated: false,
        };

        let message = Message {
            envelope: Envelope {
                version: ProtocolVersion::CURRENT,
                message_type: MessageType::Snapshot,
                message_id: 100,
                timestamp_secs: 1703174410,
                compressed: true, // Enable compression
            },
            payload: MessagePayload::Snapshot(snapshot.clone()),
        };

        // Encode (should compress)
        let frame = FrameCodec::encode(&message).unwrap();
        
        // Verify compression reduced size (heuristic: should be < uncompressed)
        let uncompressed_message = Message {
            envelope: Envelope {
                compressed: false,
                ..message.envelope
            },
            ..message.clone()
        };
        let uncompressed_frame = FrameCodec::encode(&uncompressed_message).unwrap();
        assert!(frame.len() < uncompressed_frame.len(), "Compression should reduce size");

        // Decode and verify
        let mut cursor = Cursor::new(frame);
        let decoded = FrameCodec::decode(&mut cursor).unwrap();

        assert_eq!(decoded.envelope.compressed, true);
        match decoded.payload {
            MessagePayload::Snapshot(decoded_snapshot) => {
                assert_eq!(decoded_snapshot.processes.len(), 100);
                assert_eq!(decoded_snapshot.processes[0].pid, 1000);
            }
            _ => panic!("Expected Snapshot payload"),
        }
    }

    /// Test: Backpressure signal encoding/decoding
    #[test]
    fn test_backpressure_signal() {
        let message = Message {
            envelope: Envelope {
                version: ProtocolVersion::CURRENT,
                message_type: MessageType::Backpressure,
                message_id: 200,
                timestamp_secs: 1703174420,
                compressed: false,
            },
            payload: MessagePayload::Backpressure(BackpressureSignal {
                level: 2,
                pause_secs: Some(30),
            }),
        };

        let frame = FrameCodec::encode(&message).unwrap();
        let mut cursor = Cursor::new(frame);
        let decoded = FrameCodec::decode(&mut cursor).unwrap();

        match decoded.payload {
            MessagePayload::Backpressure(signal) => {
                assert_eq!(signal.level, 2);
                assert_eq!(signal.pause_secs, Some(30));
            }
            _ => panic!("Expected Backpressure payload"),
        }
    }

    /// Test: Message acknowledgment
    #[test]
    fn test_message_ack() {
        let message = Message {
            envelope: Envelope {
                version: ProtocolVersion::CURRENT,
                message_type: MessageType::Ack,
                message_id: 300,
                timestamp_secs: 1703174430,
                compressed: false,
            },
            payload: MessagePayload::Ack(MessageAck {
                message_id: 42,
                success: false,
                error_code: Some(1001),
            }),
        };

        let frame = FrameCodec::encode(&message).unwrap();
        let mut cursor = Cursor::new(frame);
        let decoded = FrameCodec::decode(&mut cursor).unwrap();

        match decoded.payload {
            MessagePayload::Ack(ack) => {
                assert_eq!(ack.message_id, 42);
                assert_eq!(ack.success, false);
                assert_eq!(ack.error_code, Some(1001));
            }
            _ => panic!("Expected Ack payload"),
        }
    }

    /// Test: Frame size validation (reject oversized frames)
    #[test]
    fn test_frame_size_validation() {
        // Create a snapshot with many processes to exceed MAX_FRAME_SIZE
        let large_snapshot = SnapshotPayload {
            window_start_secs: 1703174400,
            window_end_secs: 1703174410,
            total_cpu_percent: 100.0,
            total_memory_bytes: 32_000_000_000,
            processes: (0..10000) // Large number of processes
                .map(|i| ProcessSample {
                    pid: i,
                    name: format!("very-long-process-name-{}", i),
                    cpu_percent: 1.0,
                    memory_bytes: 1_000_000,
                    cmdline: Some(format!("/very/long/command/line/path/number/{}/with/many/args", i)),
                })
                .collect(),
            truncated: false,
        };

        let message = Message {
            envelope: Envelope {
                version: ProtocolVersion::CURRENT,
                message_type: MessageType::Snapshot,
                message_id: 999,
                timestamp_secs: 1703174440,
                compressed: false,
            },
            payload: MessagePayload::Snapshot(large_snapshot),
        };

        // Should fail to encode due to size
        let result = FrameCodec::encode(&message);
        assert!(result.is_err());
        match result.unwrap_err() {
            ProtocolError::FrameTooLarge(size, max) => {
                assert!(size > max);
            }
            _ => panic!("Expected FrameTooLarge error"),
        }
    }

    #[test]
    fn test_cross_language_serialization_snapshot() {
        // This test serializes a Snapshot message for validation by .NET deserialization tests.
        // The encoded bytes are written to a file that .NET tests can read.
        let snapshot = SnapshotPayload {
            window_start_secs: 1703174400,
            window_end_secs: 1703174410,
            total_cpu_percent: 75.5,
            total_memory_bytes: 16_000_000_000,
            processes: vec![
                ProcessSample {
                    pid: 1001,
                    name: "chrome".to_string(),
                    cpu_percent: 45.0,
                    memory_bytes: 2_000_000_000,
                    cmdline: Some("/usr/bin/chrome --user-data-dir=/home/user/.config/google-chrome".to_string()),
                },
                ProcessSample {
                    pid: 1002,
                    name: "firefox".to_string(),
                    cpu_percent: 20.0,
                    memory_bytes: 1_500_000_000,
                    cmdline: Some("/usr/bin/firefox".to_string()),
                },
                ProcessSample {
                    pid: 1003,
                    name: "code".to_string(),
                    cpu_percent: 10.5,
                    memory_bytes: 800_000_000,
                    cmdline: None,
                },
            ],
            truncated: false,
        };

        let message = Message {
            envelope: Envelope {
                version: ProtocolVersion::CURRENT,
                message_type: MessageType::Snapshot,
                message_id: 42,
                timestamp_secs: 1703174405,
                compressed: false,
            },
            payload: MessagePayload::Snapshot(snapshot),
        };

        // Encode the message
        let encoded = FrameCodec::encode(&message).expect("Failed to encode");
        
        // Verify it's decodable on Rust side first
        let mut cursor = std::io::Cursor::new(&encoded);
        let decoded = FrameCodec::decode(&mut cursor).expect("Failed to decode");
        assert_eq!(decoded.envelope.message_id, 42);
        assert_eq!(decoded.envelope.message_type, MessageType::Snapshot);
        
        // Write to file for cross-language testing
        use std::fs;
        use std::path::PathBuf;
        
        let test_data_dir = PathBuf::from("../server/Tests/data");
        if !test_data_dir.exists() {
            fs::create_dir_all(&test_data_dir).ok();
        }
        
        let file_path = test_data_dir.join("cross-language-snapshot.bin");
        fs::write(&file_path, &encoded)
            .expect("Failed to write test data file");
        
        // Verify file was written
        let written = fs::read(&file_path).expect("Failed to read back test data");
        assert_eq!(written, encoded);
    }
}
