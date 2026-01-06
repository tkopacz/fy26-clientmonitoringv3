<!--
Sync Impact Report
- Version change: N/A → 1.0.0
- Modified principles:
	- [PRINCIPLE_1_NAME] → Minimal Footprint Agent (Rust, Windows/Linux)
	- [PRINCIPLE_2_NAME] → Scalable Linux .NET Server
	- [PRINCIPLE_3_NAME] → Binary Protocol Contract
	- [PRINCIPLE_4_NAME] → Test Discipline (Unit + Combination/Integration)
	- [PRINCIPLE_5_NAME] → Storage Abstraction (Interface-first)
- Added sections: none (filled existing placeholders for Sections 2 and 3)
- Removed sections: none
- Templates requiring updates:
	- ✅ Updated: .specify/templates/tasks-template.md (tests now mandatory per constitution)
	- ✅ Aligned: .specify/templates/plan-template.md (Constitution Check gates sourced from this file)
	- ✅ Aligned: .specify/templates/spec-template.md (user stories and acceptance tests align)
	- ✅ Aligned: .specify/templates/agent-file-template.md (no conflicting guidance)

# Client Server Monitoring Constitution
<!-- Project: fy26-clientmonitoringv3 -->


### Minimal Footprint Agent (Rust, Windows/Linux)
The monitoring agent MUST be implemented in Rust and support Windows
and Linux. It MUST minimize CPU and memory usage through efficient
sampling, zero-copy where practical, preallocated buffers, and avoiding
blocking I/O on critical paths. Collection scope includes CPU usage,
memory usage, and top processes by CPU and memory, with an option to
report all processes when requested. Do not build client for macOS.

Rationale: Rust enables predictable performance and low overhead across
platforms; careful resource discipline keeps the agent unobtrusive.

### Scalable Linux .NET Server
The server MUST run on Linux using .NET, utilize async I/O, and be
designed for horizontal scalability. It MUST implement batching,
backpressure, and concurrency-safe ingestion pipelines to handle large
numbers of agents. Processing MUST prefer streaming and incremental
parsing to avoid unnecessary memory growth.

Rationale: Async .NET on Linux provides strong performance and
operational maturity; backpressure and batching ensure stable scaling.

### Binary Protocol Contract
Communication between agent and server MUST use a compact, versioned
binary protocol with explicit framing, minimal allocations, and clear
field semantics. The protocol MUST support backward-compatible
extensions via versioning and optional fields. Messages MUST include
agent platform metadata and sampling window timestamps for consistency.

Rationale: A compact, versioned protocol reduces bandwidth and CPU,
while enabling future evolution without breaking clients.

### Test Discipline (Unit + Combination/Integration)
All modules and functions MUST have unit tests. Additionally, tests MUST
cover combined logic and integration flows (e.g., agent collection →
encoding → server decode → storage). Mocks and fakes MAY be used to
isolate components. Tests MUST be deterministic and executed on every
build (CI and local), with failure required before implementation in TDD
workflow where practical.

Rationale: Comprehensive testing ensures reliability and guards against
regressions in performance and correctness.

### Storage Abstraction (Interface-first)
Storage MUST be accessed via an interface boundary to allow swapping
backends. The initial implementation MUST append records to a file with
rotation policy documented and configurable. No business logic may
depend on a specific storage implementation.

Rationale: An interface boundary enables future storage migration (e.g.,
databases, streams) without coupling application logic to persistence.

## Additional Constraints & Performance Standards

- Agent CPU overhead: p95 ≤ 2% on typical workloads; transient spikes
	≤ 5% during full process scans.
- Agent memory overhead: steady-state ≤ 25 MB; transient ≤ 50 MB during
	all-process enumeration.
- Binary message size: compact representation; typical snapshot ≤ 64 KB
	when reporting top processes; may scale with all-process option.
- Server ingestion throughput: MUST sustain concurrent connections from
	thousands of agents with graceful degradation via backpressure.
- Observability: Structured logs with correlation IDs; lightweight
	tracing for ingestion stages; diagnostics emitted only when useful.
- Documentation: Functions, modules, and key code paths MUST include
	professional-grade comments describing intent, invariants, and key
	trade-offs.

## Development Workflow & Quality Gates

- Build Always: Every module/function includes tests executed in local
	builds and CI before merge.
- Test Types: Unit tests for each function/module; combination and
	integration tests for end-to-end logic; mocks/fakes permitted.
- Reviews: PRs MUST demonstrate compliance with principles and include
	rationale for any deviations, with performance data for sensitive
	paths.
- Protocol Changes: Require version bump, migration notes, and tests for
	backward compatibility.
- Storage Contract: Any new storage implementation MUST pass the common
	storage interface contract tests.
- Guidance: For runtime development guidance, see
	[.specify/templates/agent-file-template.md](.specify/templates/agent-file-template.md).

The constitution supersedes other practices when conflicts arise. Any
amendment MUST include: documentation of changes, impact analysis,
migration plan if applicable, and updated version per semantic rules
below.

- Compliance Reviews: All PRs/reviews MUST verify constitution
	compliance. Complexity MUST be justified in PR descriptions.
- Versioning Policy (Constitution):
	- MAJOR: Backward-incompatible governance changes or principle removal
		or redefinition.
	- MINOR: New principle/section added or materially expanded guidance.
	- PATCH: Clarifications, typos, or non-semantic refinements.
- Review Cadence: Quarterly compliance review with adjustments based on
	operational feedback and test coverage metrics.

**Version**: 1.0.0 | **Ratified**: 2025-01-06 | **Last Amended**: 2025-12-21


