## Plan: End-to-End Demo Runner Script

Create a single repo-root script that (1) runs the Rust demo producer to serialize frames into a demo file, then (2) runs the .NET demo consumer to parse that same file. This keeps the “happy path” one command, while handling common footguns (output directory missing, producer auto-versioning when file exists, missing file/empty file).

### Steps
1. Confirm CLI flags/defaults in [agent/src/bin/demo_protocol_producer.rs](./agent/src/bin/demo_protocol_producer.rs) and [server/DemoProtocolConsumer/Program.cs](./server/DemoProtocolConsumer/Program.cs).
2. Add a bash script (e.g., `run-demo-protocol.sh`) at repo root to run producer then consumer.
3. In the script, accept `OUT` and `COUNT` args, create parent dir, and `rm -f` the target to avoid auto “-001” versioning.
4. Run `cargo run -p agent --bin demo_protocol_producer -- --out "$OUT" --count "$COUNT"` then `dotnet run --project server/DemoProtocolConsumer -- --in "$OUT"`.
5. Add a small README snippet in [README.md](./README.md) (or in [specs/002-demo-protocol/quickstart.md](./specs/002-demo-protocol/quickstart.md)) showing one-line usage.

### Further Considerations
1. Output naming: always delete existing file (simpler) vs parse “final out path” from producer logs (safer).
2. Default path: keep both sides aligned on `tmp/demo-protocol.bin` to match docs and expectations.
3. Optional: add `--release` flags for faster repeated runs if desired.
