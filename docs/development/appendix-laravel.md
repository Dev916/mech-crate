Appendix B: Laravel Idioms and Patterns

Purpose
Laravel with a functional bias: pure domain, explicit ports, Eloquent at the edge.

B1. Project Shape
	•	app/Domain/* for value objects, DTOs, services, ports (interfaces).
	•	app/Infra/* for Eloquent repositories, HTTP clients, mappers.
	•	app/App/* for controllers, jobs, events, listeners.

B2. Domain Modeling
	•	Backed enums for finite states; value objects for constrained primitives.
	•	DTOs are immutable transport within the domain layer.
	•	No Eloquent models inside Domain.

B3. Ports and Adapters
	•	Define ports in Domain/Ports.
	•	Implement with Eloquent or external clients in Infra.
	•	Keep mappers translating between DB/HTTP payloads and domain DTOs.

B4. Error and Result Handling
	•	Prefer Result-like wrappers or explicit return objects instead of throwing internally.
	•	Convert exceptions at edges (controllers, jobs) into HTTP responses or retries.

B5. Validation & Decoding
	•	Form Requests validate and map inputs to simple DTO arrays.
	•	Edge decoders only; domain receives already-validated values.

B6. Testing
	•	Feature tests for controllers and routes.
	•	Unit tests for domain services and value objects.
	•	Contract tests for adapters using fakes or sqlite/in-memory strategies.
	•	Property tests for core invariants where practical.

B7. Quality Gates
	•	Static analysis (Larastan), formatter (Pint), security advisories.
	•	CI runs: analyze, format check, tests with coverage, migration check.

B8. Observability
	•	Structured logging for transitions and domain events.
	•	Metrics via counters and timers; include queue latency and retries.
	•	Correlation IDs threaded through requests and jobs.

B9. File Map (placeholders to fill later)
	•	app/Domain/Value/*
	•	app/Domain/Dto/*
	•	app/Domain/Services/*
	•	app/Domain/Ports/*
	•	app/Infra/Repos/*
	•	app/Infra/Clients/*
	•	app/Http/Controllers/*
	•	app/Http/Requests/*
	•	tests/Unit/*, tests/Feature/*

B10. When to add code
	•	After a brief transition table or ADR exists for the workflow.
	•	Keep all code demonstrations here under clear headings when you’re ready.

B11. Design Principles Ctd
        •       Modular monolith with bounded contexts (modules/packages); queues with outbox table (DB or Kafka); Horizon + idempotency middleware; Temporal via PHP SDK or run worker services in another lang and call via gRPC; events for projections; policy classes backed by OPA/Cedar if you need ABAC.
