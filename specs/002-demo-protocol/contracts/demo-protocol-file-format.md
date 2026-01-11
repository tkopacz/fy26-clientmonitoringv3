# Contract: Demo Protocol File Format

This contract defines the on-disk bytes written by the Rust demo producer and read by the .NET demo consumer.

## Overview

- The file is a concatenation of 1..N frames.
- Default is one frame.
- Frames are identical to the protocol’s on-the-wire framing.

## Frame

Each frame is:

1. `length` (4 bytes): unsigned 32-bit integer, **big-endian**
2. `body` (`length` bytes): encoded envelope + payload bytes
3. `crc32` (4 bytes): unsigned 32-bit integer, **little-endian**, computed over `body` bytes only

### Max Length (Demo Constraint)

- `length` MUST be ≤ 256 KiB (262,144 bytes).
- Consumers MUST treat larger frames as invalid and fail the run.

### CRC32

- Algorithm: standard CRC32 (same as `crc32fast` in Rust and `Force.Crc32` in .NET)
- Input: `body` bytes only

## Body

`body = envelope || payload_bytes`

- Envelope is encoded first and is never compressed.
- Payload bytes may be zstd-compressed when the envelope indicates compression.

## Multiple Frames

- Producer may write multiple frames concatenated back-to-back.
- Consumer MUST decode frames sequentially until EOF.
- If EOF occurs mid-frame (e.g., cannot read a full length/body/crc), the consumer MUST treat this as an error.

## Invalid Trailing Bytes

- If the file contains any trailing bytes that do not form a complete valid frame, the consumer MUST treat this as an error.

## Compatibility

- Message type: Snapshot only.
- Protocol version: repository “latest supported” version.
