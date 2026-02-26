# Quickstart: Validate Body-Only Compression

## Prerequisites

- Rust toolchain available (`cargo`)
- .NET SDK available (`dotnet`)
- Repository root: `fy26-clientmonitoringv3`

## 1) Run protocol test baselines

```bash
./run-all-tests.sh --verbose
```

Expected: Existing Rust and .NET test suites pass before changes.

## 2) Validate header parse without decompression dependency

Run targeted protocol tests after implementation updates:

```bash
cargo test --package agent protocol
cd server && dotnet test Tests/MonitoringServer.Tests.csproj --filter Protocol
```

Expected:
- Envelope fields decode for both compressed and uncompressed messages.
- Receiver reads compression indicator before body decompression.

## 3) Validate negotiation behavior

Test both sessions:

- mutual compression support -> `compressed=true` payloads allowed
- single-sided support -> payloads remain uncompressed

Expected: No decompression attempt when capability negotiation does not enable compression.

## 4) Validate failure behavior for malformed compressed body

Use/extend protocol tests that inject invalid compressed body with `compressed=true`.

Expected:
- explicit decompression-related error
- diagnostics include `messageType` and `messageId`
- subsequent valid frames still decode correctly

## 5) Validate dual size limits

Add/execute tests for:

- on-wire body over `maxFrameBytes=262144` -> immediate rejection before decompression
- decompressed payload over `maxDecompressedBodyBytes=262144` -> decompression abort and body-invalid outcome

Expected: deterministic rejection path for both cap violations.
