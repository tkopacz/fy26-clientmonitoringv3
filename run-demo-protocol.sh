#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
run-demo-protocol.sh [OUT] [COUNT]

Runs the Rust demo producer to generate a demo protocol file, then runs the .NET demo consumer
to read and print it.

Args:
  OUT    Output file path (default: tmp/demo-protocol.bin)
  COUNT  Number of frames to write (default: 1)

Examples:
  ./run-demo-protocol.sh
  ./run-demo-protocol.sh tmp/demo-protocol.bin 5
EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

OUT="${1:-tmp/demo-protocol.bin}"
COUNT="${2:-1}"

if [[ -z "$OUT" ]]; then
  echo "UsageError: OUT must be non-empty" >&2
  usage >&2
  exit 2
fi

if ! [[ "$COUNT" =~ ^[0-9]+$ ]]; then
  echo "UsageError: COUNT must be an integer (got '$COUNT')" >&2
  usage >&2
  exit 2
fi

OUT_DIR="$(dirname -- "$OUT")"
if [[ "$OUT_DIR" != "." ]]; then
  mkdir -p -- "$OUT_DIR"
fi

# Avoid the producer auto-versioning the output path (e.g. demo-protocol-001.bin)
# by ensuring the target does not already exist.
rm -f -- "$OUT"

echo "==> Generating demo file via Rust producer"
cargo run -p agent --bin demo_protocol_producer -- --out "$OUT" --count "$COUNT"

if [[ ! -f "$OUT" ]]; then
  echo "RunnerError: producer did not create output file: '$OUT'" >&2
  exit 1
fi

if [[ ! -s "$OUT" ]]; then
  echo "RunnerError: output file is empty: '$OUT'" >&2
  exit 1
fi

echo "==> Reading demo file via .NET consumer"
dotnet run --project server/DemoProtocolConsumer -- --in "$OUT"
