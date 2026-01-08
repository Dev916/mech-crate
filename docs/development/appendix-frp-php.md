# Appendix: FRP in PHP

Purpose: bring FRP semantics to PHP services with RxPHP + event loops (ReactPHP/Amp) and Munus for typed values.

## Libraries and Building Blocks
- RxPHP (`reactivex/rxphp`) for Observables/operators; `Subject`, `BehaviorSubject`, `ReplaySubject`.
- Event loops: ReactPHP or Amp as schedulers; integrate timers/IO.
- Munus `Option`/`Either` for domain purity; map Rx streams to typed results at boundaries.

## Core Patterns
- Intent stream (Subject) → reducer (`scan`/`reduce`) → state BehaviorSubject; effects emit new intents.
- Scheduler: `EventLoopScheduler` backed by ReactPHP loop; keep IO async.
- Backpressure: use `bufferWithCount`, `bufferWithTime`, or `sample`; avoid unbounded `ReplaySubject`.
- Error strategy: wrap errors as domain types; prefer `retryWhen` with capped backoff; no naked exceptions in streams.
- Resource safety: `takeUntil`/`finally` for lifecycle; dispose subscriptions on shutdown.

## Example: intent + effect loop
```php
<?php
use React\EventLoop\Factory as Loop;
use Rx\Observable;
use Rx\ObserverInterface;
use Rx\Scheduler\EventLoopScheduler;
use Rx\Subject\Subject;
use Munus\Control\Either;

$loop = Loop::create();
$scheduler = new EventLoopScheduler($loop);

$fetchData = fn(string $id) => \React\Promise\resolve("data-$id"); // replace with real IO

$intents = new Subject(); // {type: load|loaded|failed, id?, data?, error?}

$effects = $intents
  ->flatMap(function ($intent) use ($scheduler, $fetchData) {
    if ($intent['type'] !== 'load') {
      return Observable::of($intent);
    }
    return Observable::fromPromise(($fetchData)($intent['id'])) // promise returning string
      ->map(fn($data) => ['type' => 'loaded', 'data' => $data])
      ->catch(fn($err) => Observable::of(['type' => 'failed', 'error' => (string)$err]))
      ->subscribeOn($scheduler);
  });

$state = Observable::merge([$intents, $effects])
  ->scan(function ($state, $intent) {
    return match ($intent['type']) {
      'load' => ['status' => 'loading'],
      'loaded' => ['status' => 'ready', 'data' => $intent['data']],
      'failed' => ['status' => 'error', 'error' => $intent['error']],
      default => $state,
    };
  }, ['status' => 'idle']);

$state->subscribe(new class implements ObserverInterface {
  public function onNext($value) { echo "state: " . json_encode($value) . PHP_EOL; }
  public function onError(\Throwable $e) { echo "error: {$e->getMessage()}\n"; }
  public function onCompleted() { echo "completed\n"; }
});

$intents->onNext(['type' => 'load', 'id' => '123']);
$loop->run();
```

## Integration Notes
- Laravel/Symfony HTTP: keep domain pure; controllers translate HTTP ↔ intents/events; fold `Either` to responses.
- Queues/async: wrap queue consumers as Observables; use Subjects to push into reducers.
- Persistence: log intent/event stream for replay; hydrate BehaviorSubject from snapshots.

## Testing
- Use `TestScheduler` for deterministic time; marble tests for operator chains.
- Table-driven tests for reducers; property tests for decoders/encoders before entering streams.

## Anti-Patterns
- Blocking IO inside operators; unbounded replay subjects; mixing raw arrays with domain models; throwing inside map.

## Checklist
- Scheduler chosen; bounded buffers; error taxonomy; cancellation/disposal path; observability hooks (timeline logs, metrics).
