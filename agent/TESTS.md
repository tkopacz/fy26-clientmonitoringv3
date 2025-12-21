# Rust Protocol Tests Documentation

## Overview

The Rust test suite consists of **29 tests** organized into two categories:

1. **Unit Tests** (10 tests) - Inline in `agent/src/protocol.rs`
   - Located in the `#[cfg(test)]` module within the protocol module
   - Test core protocol functionality

2. **Integration Tests** (19 tests) - Standalone file `agent/tests/protocol_tests.rs`
   - Located in the `tests/` directory (auto-discovered by Cargo)
   - Comprehensive end-to-end protocol validation
   - Better organized for long-term maintainability

## Test Organization

### Integration Tests Structure

The integration tests are organized into 7 logical modules:

#### Module 1: Protocol Version Compatibility (3 tests)
- `protocol_version_same_is_compatible` - Same version is compatible
- `protocol_version_higher_minor_is_compatible` - Higher minor version is forward compatible
- `protocol_version_different_major_is_incompatible` - Major version mismatch is incompatible

**Purpose**: Validates version negotiation for agent-server handshakes

#### Module 2: Message Type Conversions (2 tests)
- `message_type_from_u8_all_types` - All 7 message types convert correctly
- `message_type_from_u8_invalid_type` - Invalid discriminants are rejected

**Purpose**: Ensures correct mapping between byte values and message types

#### Module 3: Agent Identity & Capabilities (3 tests)
- `agent_identity_capabilities_all_process` - CAP_ALL_PROCESS flag
- `agent_identity_capabilities_compression` - CAP_COMPRESSION flag  
- `agent_identity_capabilities_both` - Both capabilities can be combined

**Purpose**: Validates agent capability reporting for negotiation

#### Module 4: Encoding/Decoding Round-Trips (5 tests)
- `encode_decode_handshake_message` - Handshake serialization
- `encode_decode_snapshot_message` - Snapshot with 3 processes
- `encode_decode_ack_message` - Message acknowledgment
- `encode_decode_backpressure_message` - Backpressure signal

**Purpose**: Validates messages survive serialization/deserialization intact

#### Module 5: Compression Handling (1 test)
- `encode_decode_with_compression` - Compression reduces frame size, decompresses correctly

**Purpose**: Validates zstd compression (level 3) for large payloads

#### Module 6: Frame Size Validation (1 test)
- `frame_size_validation_oversized_payload` - Oversized frames are rejected

**Purpose**: Prevents memory exhaustion attacks with size constraints

#### Module 7: Cross-Language Serialization (1 test)
- `cross_language_serialization_snapshot` - Generates test data for .NET validation

**Purpose**: Creates binary protocol test data for cross-language compatibility

#### Module 8: Edge Cases & Error Handling (3 tests)
- `heartbeat_message_minimal_payload` - Minimal keepalive message
- `error_message_with_details` - Error with code and description
- `snapshot_with_no_processes` - Empty process list handling
- `snapshot_with_truncation_flag` - Truncated snapshot handling

**Purpose**: Graceful handling of boundary cases and error conditions

## Running Tests

### Run All Rust Tests
```bash
cd /home/tkopacz/fy26-clientmonitoringv3
cargo test --package agent
```

### Run Only Integration Tests
```bash
cargo test --test protocol_tests
```

### Run Only Unit Tests
```bash
cargo test --lib --package agent
```

### Run Specific Test
```bash
cargo test --test protocol_tests protocol_version_same_is_compatible
```

### Run with Verbose Output
```bash
cargo test --test protocol_tests --verbose
```

### Run All Tests (Rust + .NET)
```bash
./run-all-tests.sh
./run-all-tests.sh --verbose
```

## Test Coverage

### Message Types (7/7 covered)
- ✅ Handshake - Full encode/decode test
- ✅ HandshakeAck - Implicit via unit tests
- ✅ Heartbeat - Minimal payload test
- ✅ Snapshot - Full encode/decode with processes
- ✅ Ack - Full encode/decode test
- ✅ Backpressure - Full encode/decode test
- ✅ Error - Full encode/decode test

### Protocol Features
- ✅ Version compatibility checking
- ✅ Capability flags (all-process, compression)
- ✅ Frame length-prefix encoding (big-endian u32)
- ✅ bincode serialization/deserialization
- ✅ zstd compression (level 3)
- ✅ Frame size constraints (256 KB max)
- ✅ Process sampling (multiple processes per snapshot)
- ✅ Optional fields (cmdline in process samples)
- ✅ Truncation flags (when process list is too large)

### Test Data Scenarios
- Comprehensive process data (100+ processes for compression testing)
- Empty process lists (minimal data)
- Large cmdline arguments (path + flags)
- No cmdline (system processes)
- Multiple snapshots with different message IDs
- Cross-language serialization artifacts

## Documentation

Each test includes:
1. **Module-level documentation** - Explains the testing concern
2. **Test function documentation** - Describes what is being validated
3. **Assertion comments** - Clarifies what each assertion checks
4. **Sample data** - Realistic values (PIDs, process names, CPU/memory usage)

Example:
```rust
#[test]
fn protocol_version_higher_minor_is_compatible() {
    let v1_0 = ProtocolVersion { major: 1, minor: 0 };
    let v1_1 = ProtocolVersion { major: 1, minor: 1 };
    // v1_1 is compatible with v1_0 (newer version can work with older protocol)
    assert!(v1_1.is_compatible_with(&v1_0),
            "Higher minor version should be compatible (forward compatible)");
}
```

## Best Practices

1. **Separation of Concerns**
   - Unit tests remain in `src/` for quick feedback
   - Integration tests in `tests/` for comprehensive validation

2. **Clear Naming**
   - Test names describe the exact scenario being tested
   - Use `_` to separate concerns (e.g., `encode_decode_` prefix)

3. **Realistic Data**
   - Process samples use realistic PIDs, names, and resource values
   - Multiple processes test collection functionality
   - Large payloads verify compression benefits

4. **Error Path Testing**
   - Invalid message types are rejected
   - Oversized frames trigger errors
   - Incompatible versions are detected

5. **Documentation**
   - Module-level docs explain testing philosophy
   - Test comments explain non-obvious details
   - Assertion messages provide context

## Adding New Tests

To add a new test to `agent/tests/protocol_tests.rs`:

1. Add test under appropriate module section
2. Include clear documentation of what's being tested
3. Use descriptive test name following existing patterns
4. Include assertion messages for clarity
5. Use realistic test data where applicable
6. Run `cargo test --test protocol_tests` to verify

Example template:
```rust
#[test]
fn test_new_feature() {
    // Setup
    let message = Message { /* ... */ };
    
    // Test action
    let result = FrameCodec::encode(&message).expect("encode failed");
    
    // Verify
    assert!(!result.is_empty(), "Encoded message should not be empty");
    assert_eq!(result.len(), expected_size, "Frame size matches expectations");
}
```

## Related Files

- **Implementation**: `agent/src/protocol.rs` (729 lines)
- **Library Exports**: `agent/src/lib.rs`
- **Unit Tests**: `agent/src/protocol.rs` (lines ~350-600)
- **Integration Tests**: `agent/tests/protocol_tests.rs` (600+ lines, this file)
- **Test Runner**: `run-all-tests.sh`

## Status

✅ All 19 integration tests passing  
✅ All 10 unit tests passing  
✅ 29 total Rust protocol tests  
✅ Cross-language serialization data generated for .NET tests
