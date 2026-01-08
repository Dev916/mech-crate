# Appendix: Functional Reactive Programming (General)

Purpose: durable, law-abiding FRP foundation that scales across stacks. Focus on streams over time, explicit effects, and emergent behavior from composition.

## Mental Model
- Everything is a time-varying value: discrete events (streams/observables) and continuous-ish signals/behaviors.
- Purity + referential transparency: operators are pure; effects live at the edges.
- Dataflow graphs, not callbacks: describe what flows, not when to push.
- Monads for time and effects: `IO/Task`, `Either/Result`, `Option/Maybe` to model async, failure, and absence.
- Emergence: derive higher-level signals by combining primitives (e.g., `combineLatest`, `scan`, `withLatestFrom`), keep the graph declarative.

## Core Building Blocks
- Producers: cold vs hot streams; single-shot vs multi-shot; replay vs behavior (last value).
- Operators: map/filter/reduce, window/buffer/sample/throttle/debounce, merge/concat/switch/exhaust/zip, share/replay/refCount, retry/backoff.
- Schedulers/Executors: control where and when work runs (event loop, worker pool, async runtime).
- Subjects/Signals: bridge imperative events into streams; avoid overuse by preferring pure operator graphs.
- Resource safety: `using`/`finally`/`takeUntil` for lifetimes; cancellation is first-class.

## Design Principles
- Functional core, reactive shell: pure reducers compute next state; reactive shell wires IO.
- Type the timeline: model cancellation, timeouts, backpressure, and errors explicitly.
- Decode at the edge: all external inputs decoded to domain types; outputs encoded at boundaries.
- Backpressure strategy chosen up front: buffer with bounds, drop (latest or oldest), sample, throttle, or propagate demand.
- Avoid shared mutable state; drive UI/services from signals and reducers.
- Observability: log timeline events with correlation IDs; measure operator latency and queue depth.

## Patterns to Favor
- `scan` as reducer: fold events into state; emit state snapshots; test with table-driven cases.
- `switchMap`/`flatMapLatest` for “latest wins” workflows; `exhaustMap` for in-flight protection; `mergeMap` with concurrency caps for parallel work.
- `shareReplay` (bounded) for multicasting expensive sources; reset on error when appropriate.
- `retryWhen` with jittered backoff and max attempts; emit structured errors, not strings.
- Gate effects: map to intent -> validate -> reduce -> effect stream; keep effects cancellable.
- Time-aware invariants: property tests over event sequences; simulate clocks to prove ordering.

## Architecture Blueprint
- Inputs: UI events, messages, HTTP, queues → decoded to intents/events (typed).
- State: reducer (`scan`) producing domain state; persists snapshots or event log if needed.
- Effects: intent-driven effect layer returning streams/tasks; bridge results back as events.
- Concurrency: use schedulers/runtimes that support cancellation; never block.
- Composition: small graphs per feature; compose via higher-level streams rather than global buses.

## Anti-Patterns
- Hidden mutation inside operators; unbounded subjects with no lifecycle; ignoring cancellation.
- Mixing transport DTOs inside domain streams; swallowing errors; unbounded `shareReplay`.
- Tightly coupling UI widgets to subjects instead of passing intents.

## Testing and Verification
- Deterministic schedulers/fake clocks; marble tests for operator chains.
- Property-based tests over event sequences and reducers.
- Contract tests at boundaries (decoders/encoders, adapters).

## Adoption Path
- Start with intent → reducer → effect loop for one workflow.
- Standardize backpressure + cancellation policy.
- Introduce shared operator helpers and lint rules to ban ad-hoc subjects.
- Add observability hooks (timeline logs, metrics) and simulation tests.
