Appendix C: FSM (Finite State Machines)

Purpose
Standardize how application states and complex workflows are modeled as FSMs across stacks.

C1. Non-Negotiables
	•	All non-trivial workflows are FSMs or statecharts.
	•	State is a closed set; transitions are named events with optional guards and actions.
	•	Side-effects attach to transitions; no direct mutation from effects.
	•	Invalid transitions are prevented by types or rejected with explicit reasons.

C2. Modeling Checklist
	•	States: mutually exclusive, include terminal states.
	•	Events: small, well-named vocabulary.
	•	Context: minimal, immutable, versioned for persistence.
	•	Transitions: defined for each (state, event) pair with guards and actions.
	•	Invariants: documented and, where possible, enforced by types.
	•	Time: model timers and deadlines as events (e.g., EXPIRE, TIMEOUT).
	•	Retries: explicit attempt counts and terminal conditions.

C3. Testing Strategy
	•	Table-driven tests for the transition function.
	•	Property tests for invariants and reachability.
	•	Model-based tests: random event sequences with oracles.
	•	Contract tests for actions at IO boundaries.

C4. Observability
	•	Emit one structured log per transition: from, to, event, guards evaluated, actions scheduled, context hash, correlation ID.
	•	Metrics: per-transition counters, per-state occupancy, dwell time distributions.

C5. Persistence and Rehydration
	•	Persist entity ID, state (enum string), context snapshot (versioned), updated_at.
	•	Rehydrate the machine on read; never infer state from scattered columns.
	•	Use DB constraints for state values where supported; otherwise enforce in repositories.

C6. Documentation Artifacts
	•	Transition table included in PRs and stored in /docs/machines/<machine-name>.md.
	•	Lightweight diagram (e.g., Mermaid or PNG export) checked in alongside the table.
	•	ADR noting tradeoffs, guards, and failure modes for each machine.

C7. Minimal Transition Table Template (text-only)
	•	Columns to include:
	•	From State
	•	Event
	•	Guard(s)
	•	To State
	•	Action(s)
	•	Notes
	•	Rules: guards are pure predicates; actions are named effects; document rejection cases explicitly.

C8. Review Gate
	•	Any PR changing a workflow must include an updated transition table, tests for new pairs, and an updated diagram reference.
	•	No merge if totality or determinism is broken.

C9. Where code lives
	•	Rust machine files under core/machines or app/machines.
	•	Laravel machine files under app/Domain/<Context>/Fsm with enums for states/events.
	•	Shared generators or scripts (if any) documented here when added.

