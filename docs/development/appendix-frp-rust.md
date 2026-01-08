# Appendix: FRP in Rust

Purpose: build deterministic, cancellable FRP graphs in Rust with `rxrust`, `futures::stream`, and async runtimes.

## Libraries and Building Blocks
- rxrust: Observables, Subjects, operators (`map`, `filter`, `merge`, `switch_latest`, `scan`, `take_until`, `share`, `retry`).
- tokio/async-std: runtime and timers; integrate via schedulers.
- Domain types: `Option`/`Result`/custom enums for typed errors and absence; no panics in stream paths.

## Core Patterns
- Intent channel → rxrust Observable → `scan` reducer to state; effects are async tasks producing events back into the stream.
- Backpressure: prefer bounded channels feeding Observables; use `throttle`, `sample`, or `buffer_with_count/time` equivalents; never unbounded.
- Cancellation: `take_until` with shutdown signal; drop handles to stop upstream work.
- Error strategy: map external errors to domain enums; `retry` with capped backoff; expose terminal errors as events.
- Multicast: `share` or `publish().ref_count()` for hot streams; avoid cloning heavy sources per subscriber.

## Example: tokio + rxrust intent loop
```rust
use rxrust::prelude::*;
use rxrust::scheduler::FuturesLocalSchedulerPool;
use tokio::time::{sleep, Duration};

#[derive(Clone)]
enum Intent { Load(String), Loaded(String), Failed(String) }

#[derive(Debug, Clone)]
struct State { status: Status, data: Option<String>, error: Option<String> }
#[derive(Debug, Clone)]
enum Status { Idle, Loading, Ready, Error }

async fn fetch_data(id: String) -> Result<String, String> {
    sleep(Duration::from_millis(50)).await;
    Ok(format!("data-{id}"))
}

#[tokio::main]
async fn main() {
    let intents = Subject::local(); // hot stream
    let mut pool = FuturesLocalSchedulerPool::new();
    let spawner = pool.spawner();

    let effects = intents.clone().flat_map(move |intent| {
        match intent {
            Intent::Load(id) => observable::from_future(fetch_data(id), spawner.clone())
                .map(|res| match res {
                    Ok(data) => Intent::Loaded(data),
                    Err(e) => Intent::Failed(e),
                }),
            other => observable::of(other),
        }
    });

    let state = intents.clone()
        .merge(effects)
        .scan(State { status: Status::Idle, data: None, error: None }, |state, intent| {
            match intent {
                Intent::Load(_) => State { status: Status::Loading, data: None, error: None },
                Intent::Loaded(data) => State { status: Status::Ready, data: Some(data), error: None },
                Intent::Failed(e) => State { status: Status::Error, data: None, error: Some(e) },
            }
        })
        .share(); // multicast state

    state.clone().subscribe(|s| println!("state: {:?}", s));

    intents.clone().next(Intent::Load("42".into()));
    pool.run(); // drive futures
}
```

## Integration Notes
- HTTP servers (Axum/Actix): decode requests to intents; stream responses where useful (SSE/WebSocket); keep domain pure, adapters handle IO.
- Message brokers: consume into Observables with bounded channels; effect layer produces ack/nack events.
- UI (Tauri/Leptos): bridge Observables to signals; keep reducers pure and side-effects explicit.

## Testing
- Use deterministic schedulers; simulate time with tokio `time::pause`.
- Property tests over reducers/state transitions; contract tests for adapter mapping.
- Ensure no `unwrap`/`expect` in stream paths; guard invariants with pattern matches.

## Anti-Patterns
- Blocking in async operators; cloning heavy upstream per subscriber; ignoring cancellation; unbounded `share` without shutdown.

## Checklist
- Bounded channels; explicit backpressure choice; typed errors; cancellation signal; metrics on operator latency and queue depth; docs for intent/reducer graph.
