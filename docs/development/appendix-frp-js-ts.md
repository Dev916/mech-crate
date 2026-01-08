# Appendix: FRP in JavaScript / TypeScript

Purpose: high-leverage FRP stacks for browsers and Node, centered on RxJS + fp-ts for typed effects and domain purity.

## Libraries and Building Blocks
- RxJS (Observables, Subjects, schedulers); focus on pipeable operators.
- fp-ts `Either`/`Option` + `Task`/`TaskEither` to model errors and async; bridge to Observables when needed.
- io-ts (or zod) for edge decoding before values enter streams.

## Core Patterns
- Intent → reducer (`scan`) → state signal, with effect streams feeding new intents.
- Multicast expensive sources with `shareReplay({ bufferSize: n, refCount: true })`; bound `n`.
- Concurrency choices: `switchMap` (latest wins), `exhaustMap` (drop if busy), `mergeMap` (parallel with `concurrency`), `concatMap` (ordered).
- Backpressure: `throttleTime`/`debounceTime` for UI chatter; `bufferTime`/`bufferCount` with bounds for bursts; prefer `sample` over unbounded `shareReplay`.
- Error strategy: convert to typed domain errors; use `retryWhen` + jitter backoff + cap; surface terminal errors via explicit channels.
- Scheduling: main thread for UI bindings; `queueScheduler` for deterministic unit tests; `animationFrameScheduler` for visual updates; `asyncScheduler` for timers.

## Example: intent loop with effects
```typescript
import { Subject, merge, from, of } from 'rxjs';
import { map, scan, switchMap, catchError, startWith, shareReplay } from 'rxjs/operators';
import * as TE from 'fp-ts/TaskEither';
import { pipe } from 'fp-ts/function';

type Intent = { type: 'load'; id: string } | { type: 'loaded'; data: string } | { type: 'failed'; error: string };
type State = { status: 'idle'|'loading'|'ready'|'error'; data?: string; error?: string };

const intents = new Subject<Intent>();

const fetchData = (id: string) =>
  pipe(
    TE.tryCatch(
      () => fetch(`/api/items/${id}`).then(r => r.text()),
      (e) => new Error(String(e))
    )
  );

const effects$ = intents.pipe(
  switchMap(intent => intent.type === 'load'
    ? from(fetchData(intent.id)()).pipe(
        map(data => ({ type: 'loaded', data } as Intent)),
        catchError(err => of({ type: 'failed', error: err.message }))
      )
    : of(intent)
  )
);

const state$ = merge(intents, effects$).pipe(
  scan((state: State, intent: Intent) => {
    switch (intent.type) {
      case 'load': return { status: 'loading' };
      case 'loaded': return { status: 'ready', data: intent.data };
      case 'failed': return { status: 'error', error: intent.error };
    }
  }, { status: 'idle' }),
  startWith({ status: 'idle' }),
  shareReplay({ bufferSize: 1, refCount: true })
);
```

## UI Binding
- React: subscribe once per feature; prefer `useSyncExternalStore` or `from(state$)` with `useObservable`. Keep hooks thin; avoid per-render subscriptions.
- Vue/Svelte/Nuxt: expose derived signals as stores; update reactivity via a single subscription.

## Testing
- Marble tests for operator chains; fake schedulers for time.
- Reducer/state tests table-driven; property tests over intent sequences.
- Contract tests for decoders/encoders feeding streams.

## Anti-Patterns
- Unbounded subjects; mixing fetch promises inside operators without cancellation; `shareReplay(Infinity)`.
- Swallowing errors in `catchError` without telemetry; side-effects in `map` instead of `tap`.

## Checklist
- Types at edges; bounded replay; explicit concurrency choice; cancellation path; metrics on slow operators; docs for intent/reducer graph.
