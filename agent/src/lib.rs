pub mod demo_protocol;
/// Agent library exports
///
/// Provides protocol encoding, framing, and core monitoring agent functionality.
pub mod protocol;

pub use protocol::{
    AgentIdentity, BackpressureSignal, Message, MessageAck, MessageType, ProcessSample,
    ProtocolError, ProtocolVersion, SnapshotPayload,
};
