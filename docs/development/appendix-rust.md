Appendix A: Rust Idioms and Patterns

Purpose
Guidance for a functional-core Rust stack with explicit effects at the edges. No heavy code here—only structure, filenames, and checklists.

A1. Project Shape
	•	Crates or folders: core (pure domain), infra (adapters), app (HTTP/CLI).
	•	Boundaries: domain logic is pure and dependency-free; all IO in adapters.

A2. Domain Modeling
	•	Use enums for closed state sets; newtypes for identity and constrained values.
	•	No Option/Result leakage across layers without mapping to domain types.
	•	Typestates where practical for valid-only transitions.

A3. Ports and Adapters
	•	Define traits in core/ports for persistence, messaging, external services.
	•	Implement adapters in infra/* (e.g., SQL, HTTP clients, queues).
	•	Keep mapping in infra/mappers to isolate transport shapes from domain.

A4. Effects and Concurrency
	•	All non-determinism (time, rand, UUID, IO) injected at edges.
	•	Concurrency via async tasks and message passing; model cancellation/timeouts explicitly.
	•	Avoid shared mutable state; prefer structured ownership and channels.

A5. Error Strategy
	•	Domain errors are enumerations with meaningful variants.
	•	Centralized boundary mapping from library errors to domain errors.
	•	No panics in domain paths; fail fast only at process boundaries.

A6. Testing
	•	Unit tests for pure functions in core.
	•	Property tests for invariants and decoders/encoders.
	•	Contract tests for core/ports against adapter fakes; integration tests for app.
	•	Model-based tests for state machines where applicable.

A7. Observability
	•	Structured logs with correlation IDs.
	•	Metrics: transition counters, state occupancy, latency histograms.
	•	Traces on cross-service calls.

A8. Build & CI
	•	Quality gates: format, lint, typecheck, test, coverage threshold.
	•	Example targets you will add later: run, test, lint, check, ci.

A9. File Map (placeholders to fill later)
	•	core/errors.rs
	•	core/model/*.rs
	•	core/ports/*.rs
	•	core/services/*.rs
	•	infra/repos/*.rs
	•	infra/clients/*.rs
	•	app/http/*.rs
	•	app/main.rs

A10. When to add code
	•	Only after modeling a transition table or ADR exists for the feature.
	•	Place code samples in this appendix under clearly labeled subsections when ready.

A11. Design Principles Ctd
        •       Enums + pattern matching for ADTs; tokio + channels (CSP); property tests with proptest; TLA+ for protocols; Axum with typed extractors at boundaries; use tower layers for cross-cutting concerns.
