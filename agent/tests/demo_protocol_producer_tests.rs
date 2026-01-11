use agent::demo_protocol::{build_demo_message, encode_demo_frame_bytes};
use agent::protocol::{FrameCodec, MessagePayload, OsType};
use std::io::Cursor;

#[test]
fn demo_message_is_deterministic() {
    let message_a = build_demo_message(OsType::Linux);
    let message_b = build_demo_message(OsType::Linux);

    assert_eq!(message_a.envelope.version.major, 1);
    assert_eq!(message_a.envelope.version.minor, 0);
    assert_eq!(message_a.envelope.compressed, false);
    assert_eq!(message_a.envelope.agent_id, "demo-agent-ž");

    assert_eq!(message_a, message_b);

    let frame_a = encode_demo_frame_bytes(&message_a).expect("encode A");
    let frame_b = encode_demo_frame_bytes(&message_b).expect("encode B");

    assert_eq!(frame_a, frame_b);
}

#[test]
fn demo_frame_round_trips_through_framecodec() {
    let message = build_demo_message(OsType::Linux);
    let frame = encode_demo_frame_bytes(&message).expect("encode");

    let mut cursor = Cursor::new(frame);
    let decoded = FrameCodec::decode(&mut cursor).expect("decode");

    assert_eq!(decoded.envelope.version, message.envelope.version);
    assert_eq!(decoded.envelope.message_type, message.envelope.message_type);
    assert_eq!(decoded.envelope.message_id, message.envelope.message_id);
    assert_eq!(
        decoded.envelope.timestamp_utc_ms,
        message.envelope.timestamp_utc_ms
    );
    assert_eq!(decoded.envelope.agent_id, message.envelope.agent_id);
    assert_eq!(decoded.envelope.platform, message.envelope.platform);
    assert_eq!(decoded.envelope.compressed, message.envelope.compressed);

    let MessagePayload::Snapshot(snapshot) = decoded.payload else {
        panic!("expected Snapshot payload");
    };

    assert_eq!(snapshot.processes.len(), 2);
    assert_eq!(snapshot.processes[0].pid, 1234);
    assert_eq!(snapshot.processes[1].cmdline, None);
    assert_eq!(snapshot.truncated, false);
}
