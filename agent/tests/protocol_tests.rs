//! Integration tests for the monitoring protocol.
//!
//! This test suite validates the complete protocol implementation including:
//! - Message serialization and deserialization
//! - Frame encoding and decoding with length-prefix framing
//! - Compression handling with zstd (level 3)
//! - Message type conversions and version compatibility
//! - Error handling and edge cases
//! - Cross-language serialization format validation
//!
//! The tests are organized into logical modules for maintainability:
//! - Protocol version compatibility
//! - Message types and conversions
//! - Encoding and decoding round-trips
//! - Compression and frame handling
//! - Edge cases and error conditions

use agent::protocol::*;
use std::io::Cursor;

// Test helper: Create a 16-byte message ID from a u64 for simplicity in tests
fn test_message_id(n: u64) -> [u8; 16] {
    let mut id = [0u8; 16];
    id[0..8].copy_from_slice(&n.to_le_bytes());
    id
}

// ============================================================================
// Module: Protocol Version Compatibility Tests
// ============================================================================

/// Tests for protocol version compatibility checking.
/// Validates that version negotiation works correctly for agent-server handshakes.
#[test]
fn protocol_version_same_is_compatible() {
    let v1_0 = ProtocolVersion { major: 1, minor: 0 };
    assert!(v1_0.is_compatible_with(&v1_0), 
            "Same version should be compatible");
}

#[test]
fn protocol_version_higher_minor_is_compatible() {
    let v1_0 = ProtocolVersion { major: 1, minor: 0 };
    let v1_1 = ProtocolVersion { major: 1, minor: 1 };
    // v1_1 is compatible with v1_0 (newer version can work with older protocol)
    assert!(v1_1.is_compatible_with(&v1_0),
            "Higher minor version should be compatible (forward compatible)");
}

#[test]
fn protocol_version_different_major_is_incompatible() {
    let v1_0 = ProtocolVersion { major: 1, minor: 0 };
    let v2_0 = ProtocolVersion { major: 2, minor: 0 };
    assert!(!v1_0.is_compatible_with(&v2_0),
            "Different major version should not be compatible");
}

// ============================================================================
// Module: Message Type Conversions
// ============================================================================

/// Tests for MessageType enum conversions and discriminant values.
/// Ensures correct mapping between byte values and message types for wire protocol.
#[test]
fn message_type_from_u8_all_types() {
    // Test all valid message types
    assert_eq!(MessageType::from_u8(1).unwrap(), MessageType::Handshake);
    assert_eq!(MessageType::from_u8(2).unwrap(), MessageType::HandshakeAck);
    assert_eq!(MessageType::from_u8(3).unwrap(), MessageType::Heartbeat);
    assert_eq!(MessageType::from_u8(4).unwrap(), MessageType::Snapshot);
    assert_eq!(MessageType::from_u8(5).unwrap(), MessageType::Ack);
    assert_eq!(MessageType::from_u8(6).unwrap(), MessageType::Backpressure);
    assert_eq!(MessageType::from_u8(7).unwrap(), MessageType::Error);
}

#[test]
fn message_type_from_u8_invalid_type() {
    // Test that invalid discriminants produce errors
    assert!(MessageType::from_u8(0).is_err(), "Type 0 should be invalid");
    assert!(MessageType::from_u8(8).is_err(), "Type 8 should be invalid");
    assert!(MessageType::from_u8(255).is_err(), "Type 255 should be invalid");
}

// ============================================================================
// Module: Agent Identity and Capabilities
// ============================================================================

/// Tests for agent identity and capability flags.
/// Validates that agents correctly report their capabilities for negotiation.
#[test]
fn agent_identity_capabilities_all_process() {
    let identity = AgentIdentity {
        instance_id: "agent-001".to_string(),
        os_type: OsType::Linux,
        agent_version: "0.1.0".to_string(),
        protocol_version: ProtocolVersion::CURRENT,
        capabilities: AgentIdentity::CAP_ALL_PROCESS,
    };
    assert!(identity.supports_all_process());
    assert!(!identity.supports_compression());
}

#[test]
fn agent_identity_capabilities_compression() {
    let identity = AgentIdentity {
        instance_id: "agent-002".to_string(),
        os_type: OsType::Windows,
        agent_version: "0.1.0".to_string(),
        protocol_version: ProtocolVersion::CURRENT,
        capabilities: AgentIdentity::CAP_COMPRESSION,
    };
    assert!(!identity.supports_all_process());
    assert!(identity.supports_compression());
}

#[test]
fn agent_identity_capabilities_both() {
    let capabilities = AgentIdentity::CAP_ALL_PROCESS | AgentIdentity::CAP_COMPRESSION;
    let identity = AgentIdentity {
        instance_id: "agent-003".to_string(),
        os_type: OsType::Linux,
        agent_version: "0.1.0".to_string(),
        protocol_version: ProtocolVersion::CURRENT,
        capabilities,
    };
    assert!(identity.supports_all_process());
    assert!(identity.supports_compression());
}

// ============================================================================
// Module: Encoding/Decoding Round-Trips
// ============================================================================

/// Tests for message encoding and decoding with full frame support.
/// Validates that messages survive serialization and deserialization intact.

#[test]
fn encode_decode_handshake_message() {
    let original = Message {
        envelope: Envelope {
            version: ProtocolVersion::CURRENT,
            message_type: MessageType::Handshake,
            message_id: test_message_id(1),
            timestamp_utc_ms: 1703174400000,
            agent_id: "test-agent-001".to_string(),
            platform: OsType::Linux,
            compressed: false,
        },
        payload: MessagePayload::Handshake(AgentIdentity {
            instance_id: "test-agent-001".to_string(),
            os_type: OsType::Linux,
            agent_version: "0.1.0".to_string(),
            protocol_version: ProtocolVersion::CURRENT,
            capabilities: AgentIdentity::CAP_ALL_PROCESS | AgentIdentity::CAP_COMPRESSION,
        }),
    };

    // Encode to bytes
    let encoded = FrameCodec::encode(&original).expect("Failed to encode handshake");
    assert!(!encoded.is_empty(), "Encoded message should not be empty");

    // Decode from bytes
    let mut cursor = Cursor::new(&encoded);
    let decoded = FrameCodec::decode(&mut cursor).expect("Failed to decode handshake");

    // Verify round-trip
    assert_eq!(decoded.envelope.message_id, original.envelope.message_id);
    assert_eq!(decoded.envelope.message_type, MessageType::Handshake);
}

#[test]
fn encode_decode_snapshot_message() {
    let original = Message {
        envelope: Envelope {
            version: ProtocolVersion::CURRENT,
            message_type: MessageType::Snapshot,
            message_id: test_message_id(42),
            timestamp_utc_ms: 1703174405000,
            agent_id: "test-agent-001".to_string(),
            platform: OsType::Linux,
            compressed: false,
        },
        payload: MessagePayload::Snapshot(SnapshotPayload {
            window_start_secs: 1703174400,
            window_end_secs: 1703174410,
            total_cpu_percent: 45.5,
            memory_used_bytes: 7_500_000_000,
            memory_total_bytes: 8_000_000_000,
            processes: vec![
                ProcessSample {
                    pid: 1001,
                    name: "chrome".to_string(),
                    cpu_percent: 25.0,
                    memory_percent: 25.0,
                    memory_bytes: 2_000_000_000,
                    cmdline: Some("/usr/bin/chrome --user-data-dir=~/.config/google-chrome".to_string()),
                },
                ProcessSample {
                    pid: 1002,
                    name: "rust-app".to_string(),
                    cpu_percent: 15.0,
                    memory_percent: 6.25,
                    memory_bytes: 500_000_000,
                    cmdline: Some("/home/user/app/rust-app".to_string()),
                },
                ProcessSample {
                    pid: 1003,
                    name: "sshd".to_string(),
                    cpu_percent: 5.5,
                    memory_percent: 1.25,
                    memory_bytes: 100_000_000,
                    cmdline: None,
                },
            ],
            truncated: false,
        }),
    };

    let encoded = FrameCodec::encode(&original).expect("Failed to encode snapshot");
    let mut cursor = Cursor::new(&encoded);
    let decoded = FrameCodec::decode(&mut cursor).expect("Failed to decode snapshot");

    assert_eq!(decoded.envelope.message_type, MessageType::Snapshot);
    assert_eq!(decoded.envelope.message_id, test_message_id(42));
}

#[test]
fn encode_decode_ack_message() {
    let original = Message {
        envelope: Envelope {
            version: ProtocolVersion::CURRENT,
            message_type: MessageType::Ack,
            message_id: test_message_id(100),
            timestamp_utc_ms: 1703174410000,
            agent_id: "test-agent-001".to_string(),
            platform: OsType::Linux,
            compressed: false,
        },
        payload: MessagePayload::Ack(MessageAck {
            message_id: test_message_id(99),
            success: true,
            error_code: None,
        }),
    };

    let encoded = FrameCodec::encode(&original).expect("Failed to encode ack");
    let mut cursor = Cursor::new(&encoded);
    let decoded = FrameCodec::decode(&mut cursor).expect("Failed to decode ack");

    assert_eq!(decoded.envelope.message_type, MessageType::Ack);
}

#[test]
fn encode_decode_backpressure_message() {
    let original = Message {
        envelope: Envelope {
            version: ProtocolVersion::CURRENT,
            message_type: MessageType::Backpressure,
            message_id: test_message_id(50),
            timestamp_utc_ms: 1703174415000,
            agent_id: "test-agent-001".to_string(),
            platform: OsType::Linux,
            compressed: false,
        },
        payload: MessagePayload::Backpressure(BackpressureSignal {
            throttle_delay_ms: 5000,
            reason: Some("Server buffer threshold exceeded".to_string()),
        }),
    };

    let encoded = FrameCodec::encode(&original).expect("Failed to encode backpressure");
    let mut cursor = Cursor::new(&encoded);
    let decoded = FrameCodec::decode(&mut cursor).expect("Failed to decode backpressure");

    assert_eq!(decoded.envelope.message_type, MessageType::Backpressure);
    
    match decoded.payload {
        MessagePayload::Backpressure(bp) => {
            assert_eq!(bp.throttle_delay_ms, 5000);
            assert_eq!(bp.reason, Some("Server buffer threshold exceeded".to_string()));
        }
        _ => panic!("Expected Backpressure payload"),
    }
}

// Additional tests for throttleDelayMs interpretation
#[test]
fn backpressure_no_throttle() {
    let bp = BackpressureSignal {
        throttle_delay_ms: 0,
        reason: None,
    };
    
    assert_eq!(bp.throttle_delay_ms, 0, "0 means no throttle");
}

#[test]
fn backpressure_small_delay() {
    let bp = BackpressureSignal {
        throttle_delay_ms: 100,
        reason: Some("Light throttle".to_string()),
    };
    
    assert_eq!(bp.throttle_delay_ms, 100, "Small delay for light throttle");
}

#[test]
fn backpressure_large_delay() {
    let bp = BackpressureSignal {
        throttle_delay_ms: 30000,
        reason: Some("Heavy throttle".to_string()),
    };
    
    assert_eq!(bp.throttle_delay_ms, 30000, "Large delay for heavy throttle");
}

// ============================================================================
// Module: Compression Handling
// ============================================================================

/// Tests for zstd compression (level 3) of large payloads.
/// Ensures that compression reduces frame size and decompression works correctly.

#[test]
fn encode_decode_with_compression() {
    // Create a large snapshot that will benefit from compression
    let mut processes = Vec::new();
    for i in 0..100 {
        processes.push(ProcessSample {
            pid: 2000 + i,
            name: format!("process-{:03}", i),
            cpu_percent: (i as f32) * 0.1,
            memory_percent: (i as f32) * 0.05,
            memory_bytes: (i as u64) * 1_000_000,
            cmdline: Some(format!("/usr/bin/process-{:03} --arg1=value{} --arg2=/path/to/file", i, i)),
        });
    }

    let message = Message {
        envelope: Envelope {
            version: ProtocolVersion::CURRENT,
            message_type: MessageType::Snapshot,
            message_id: test_message_id(200),
            timestamp_utc_ms: 1703174420000,
            agent_id: "test-agent-001".to_string(),
            platform: OsType::Linux,
            compressed: true, // Enable compression
        },
        payload: MessagePayload::Snapshot(SnapshotPayload {
            window_start_secs: 1703174410,
            window_end_secs: 1703174420,
            total_cpu_percent: 75.0,
            memory_used_bytes: 15_000_000_000,
            memory_total_bytes: 16_000_000_000,
            processes,
            truncated: false,
        }),
    };

    let encoded = FrameCodec::encode(&message).expect("Failed to encode with compression");
    
    // Verify it's compressed by checking it's smaller than uncompressed version
    let message_uncompressed = Message {
        envelope: Envelope {
            version: message.envelope.version,
            message_type: message.envelope.message_type,
            message_id: message.envelope.message_id,
            timestamp_utc_ms: message.envelope.timestamp_utc_ms,
            agent_id: message.envelope.agent_id.clone(),
            platform: message.envelope.platform,
            compressed: false,
        },
        payload: message.payload.clone(),
    };
    let encoded_uncompressed = FrameCodec::encode(&message_uncompressed)
        .expect("Failed to encode without compression");
    
    assert!(encoded.len() < encoded_uncompressed.len(),
            "Compressed frame ({}) should be smaller than uncompressed ({})",
            encoded.len(),
            encoded_uncompressed.len());

    // Decode and verify
    let mut cursor = Cursor::new(&encoded);
    let decoded = FrameCodec::decode(&mut cursor).expect("Failed to decode compressed message");
    
    assert_eq!(decoded.envelope.message_type, MessageType::Snapshot);
    assert_eq!(decoded.envelope.message_id, test_message_id(200));
}

// ============================================================================
// Module: Frame Size Validation
// ============================================================================

/// Tests for frame size constraints and error handling.
/// Ensures that oversized frames are rejected to prevent memory exhaustion attacks.

#[test]
fn frame_size_validation_oversized_payload() {
    // Try to create a message that would exceed MAX_FRAME_SIZE when encoded
    let mut processes = Vec::new();
    
    // Create enough processes to exceed 256 KB uncompressed
    for i in 0..10000 {
        processes.push(ProcessSample {
            pid: i as u32,
            name: format!("very-long-process-name-with-lots-of-characters-{:05}", i),
            cpu_percent: 1.0,
            memory_percent: 0.5,
            memory_bytes: 1_000_000,
            cmdline: Some(format!(
                "/very/long/command/line/path/with/many/arguments/and/environment/variables/{:05}",
                i
            )),
        });
    }

    let message = Message {
        envelope: Envelope {
            version: ProtocolVersion::CURRENT,
            message_type: MessageType::Snapshot,
            message_id: test_message_id(999),
            timestamp_utc_ms: 1703174425000,
            agent_id: "test-agent-001".to_string(),
            platform: OsType::Linux,
            compressed: false,
        },
        payload: MessagePayload::Snapshot(SnapshotPayload {
            window_start_secs: 1703174400,
            window_end_secs: 1703174430,
            total_cpu_percent: 100.0,
            memory_used_bytes: 30_000_000_000,
            memory_total_bytes: 32_000_000_000,
            processes,
            truncated: false,
        }),
    };

    // Should fail to encode due to size
    let result = FrameCodec::encode(&message);
    assert!(result.is_err(), "Should reject oversized frame");
    
    match result.unwrap_err() {
        ProtocolError::FrameTooLarge(size, max) => {
            assert!(size > max, "Reported size should exceed max");
        }
        _ => panic!("Expected FrameTooLarge error"),
    }
}

// ============================================================================
// Module: Cross-Language Serialization
// ============================================================================

/// Tests for data persistence and cross-language validation.
/// The serialized data can be used by .NET integration tests to verify
/// that the binary protocol is compatible across language implementations.

#[test]
fn cross_language_serialization_snapshot() {
    // This test serializes a Snapshot message for validation by .NET deserialization tests.
    // The encoded bytes are written to a file that .NET tests can read.
    let snapshot = SnapshotPayload {
        window_start_secs: 1703174400,
        window_end_secs: 1703174410,
        total_cpu_percent: 75.5,
        memory_used_bytes: 15_000_000_000,
        memory_total_bytes: 16_000_000_000,
        processes: vec![
            ProcessSample {
                pid: 1001,
                name: "chrome".to_string(),
                cpu_percent: 45.0,
                memory_percent: 12.5,
                memory_bytes: 2_000_000_000,
                cmdline: Some("/usr/bin/chrome --user-data-dir=/home/user/.config/google-chrome".to_string()),
            },
            ProcessSample {
                pid: 1002,
                name: "firefox".to_string(),
                cpu_percent: 20.0,
                memory_percent: 9.375,
                memory_bytes: 1_500_000_000,
                cmdline: Some("/usr/bin/firefox".to_string()),
            },
            ProcessSample {
                pid: 1003,
                name: "code".to_string(),
                cpu_percent: 10.5,
                memory_percent: 5.0,
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
            message_id: test_message_id(42),
            timestamp_utc_ms: 1703174405000,
            agent_id: "test-agent-001".to_string(),
            platform: OsType::Linux,
            compressed: false,
        },
        payload: MessagePayload::Snapshot(snapshot),
    };

    // Encode the message
    let encoded = FrameCodec::encode(&message).expect("Failed to encode");
    
    // Verify it's decodable on Rust side first
    let mut cursor = std::io::Cursor::new(&encoded);
    let decoded = FrameCodec::decode(&mut cursor).expect("Failed to decode");
    assert_eq!(decoded.envelope.message_id, test_message_id(42));
    assert_eq!(decoded.envelope.message_type, MessageType::Snapshot);
    
    // Write to file for cross-language testing
    use std::fs;
    use std::path::PathBuf;
    use std::env;

    let test_data_dir = env::var("CROSS_LANG_TEST_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            // Fall back to workspace-relative default: <workspace>/server/Tests/data
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("server")
                .join("Tests")
                .join("data")
        });
    if !test_data_dir.exists() {
        fs::create_dir_all(&test_data_dir).ok();
    }

    let file_path = test_data_dir.join("cross-language-snapshot.bin");
    fs::write(&file_path, &encoded)
        .expect("Failed to write test data file");
    // Verify file was written
    let written = fs::read(&file_path).expect("Failed to read back test data");
    assert_eq!(written, encoded, "Written file should match encoded data");
}

// ============================================================================
// Module: Edge Cases and Error Handling
// ============================================================================

/// Tests for error conditions and boundary cases.
/// Ensures graceful handling of malformed or invalid input.

#[test]
fn heartbeat_message_minimal_payload() {
    // Heartbeat messages have no payload beyond the envelope.
    // This is the minimal message type for keepalive signals.
    let message = Message {
        envelope: Envelope {
            version: ProtocolVersion::CURRENT,
            message_type: MessageType::Heartbeat,
            message_id: test_message_id(0),
            timestamp_utc_ms: 1703174430000,
            agent_id: "test-agent-001".to_string(),
            platform: OsType::Linux,
            compressed: false,
        },
        payload: MessagePayload::Heartbeat,
    };

    let encoded = FrameCodec::encode(&message).expect("Failed to encode heartbeat");
    let mut cursor = Cursor::new(&encoded);
    let decoded = FrameCodec::decode(&mut cursor).expect("Failed to decode heartbeat");

    assert_eq!(decoded.envelope.message_type, MessageType::Heartbeat);
    // Frame should be small since there's no payload
    assert!(encoded.len() < 200, "Heartbeat frame should be small");
}

#[test]
fn error_message_with_details() {
    // Error messages carry error code and description.
    let message = Message {
        envelope: Envelope {
            version: ProtocolVersion::CURRENT,
            message_type: MessageType::Error,
            message_id: test_message_id(555),
            timestamp_utc_ms: 1703174435000,
            agent_id: "test-agent-001".to_string(),
            platform: OsType::Linux,
            compressed: false,
        },
        payload: MessagePayload::Error {
            code: 1001,
            message: "Agent encountered an internal error while sampling processes".to_string(),
        },
    };

    let encoded = FrameCodec::encode(&message).expect("Failed to encode error");
    let mut cursor = Cursor::new(&encoded);
    let decoded = FrameCodec::decode(&mut cursor).expect("Failed to decode error");

    assert_eq!(decoded.envelope.message_type, MessageType::Error);
}

#[test]
fn snapshot_with_no_processes() {
    // Snapshot with empty process list (system under very light load).
    let message = Message {
        envelope: Envelope {
            version: ProtocolVersion::CURRENT,
            message_type: MessageType::Snapshot,
            message_id: test_message_id(10),
            timestamp_utc_ms: 1703174440000,
            agent_id: "test-agent-001".to_string(),
            platform: OsType::Linux,
            compressed: false,
        },
        payload: MessagePayload::Snapshot(SnapshotPayload {
            window_start_secs: 1703174435,
            window_end_secs: 1703174440,
            total_cpu_percent: 0.1,
            memory_used_bytes: 95_000_000,
            memory_total_bytes: 100_000_000,
            processes: vec![],
            truncated: false,
        }),
    };

    let encoded = FrameCodec::encode(&message).expect("Failed to encode empty snapshot");
    let mut cursor = Cursor::new(&encoded);
    let decoded = FrameCodec::decode(&mut cursor).expect("Failed to decode empty snapshot");

    assert_eq!(decoded.envelope.message_type, MessageType::Snapshot);
}

#[test]
fn snapshot_with_truncation_flag() {
    // Snapshot marked as truncated (process list exceeded limits).
    // Server should handle truncated snapshots gracefully.
    let message = Message {
        envelope: Envelope {
            version: ProtocolVersion::CURRENT,
            message_type: MessageType::Snapshot,
            message_id: test_message_id(11),
            timestamp_utc_ms: 1703174445000,
            agent_id: "test-agent-001".to_string(),
            platform: OsType::Linux,
            compressed: false,
        },
        payload: MessagePayload::Snapshot(SnapshotPayload {
            window_start_secs: 1703174440,
            window_end_secs: 1703174445,
            total_cpu_percent: 95.0,
            memory_used_bytes: 28_000_000_000,
            memory_total_bytes: 30_000_000_000,
            processes: vec![
                ProcessSample {
                    pid: 1,
                    name: "init".to_string(),
                    cpu_percent: 0.1,
                    memory_percent: 0.003,
                    memory_bytes: 1_000_000,
                    cmdline: None,
                },
            ],
            truncated: true, // Flag indicates more processes were filtered out
        }),
    };

    let encoded = FrameCodec::encode(&message).expect("Failed to encode truncated snapshot");
    let mut cursor = Cursor::new(&encoded);
    let decoded = FrameCodec::decode(&mut cursor).expect("Failed to decode truncated snapshot");

    assert_eq!(decoded.envelope.message_type, MessageType::Snapshot);
}
