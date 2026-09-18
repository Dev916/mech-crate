---
title: "FRP in Python"
category: patterns
languages: [python]
complexity: advanced
use_cases:
  - "building cancellable reactive graphs in Python"
  - "bounding asyncio queues and anyio streams for backpressure"
  - "choosing between reactivex and plain async generators"
  - "testing event sequences with virtual time"
summary: "Cancellable, backpressured FRP graphs in Python with reactivex 5, asyncio structured concurrency, and anyio or trio channels."
provenance: researched
researched: 2026-09-18
sources:
  - https://rxpy.readthedocs.io/en/latest/index.html
  - https://pypi.org/project/reactivex/
  - https://github.com/ReactiveX/RxPY/blob/master/CHANGELOG.md
  - https://rxpy.readthedocs.io/en/latest/reference_subject.html
  - https://rxpy.readthedocs.io/en/latest/reference_scheduler.html
  - https://rxpy.readthedocs.io/en/latest/testing.html
  - https://docs.python.org/3/library/asyncio-queue.html
  - https://docs.python.org/3/library/asyncio-task.html
  - https://anyio.readthedocs.io/en/stable/streams.html
  - https://anyio.readthedocs.io/en/stable/api.html
  - https://anyio.readthedocs.io/en/stable/testing.html
  - https://trio.readthedocs.io/en/stable/reference-core.html
  - https://trio.readthedocs.io/en/stable/reference-testing.html
  - https://peps.python.org/pep-0789/
  - https://textual.textualize.io/guide/reactivity/
  - https://starlette.dev/responses/
  - https://starlette.dev/websockets/
  - https://docs.djangoproject.com/en/6.0/topics/signals/
  - https://aiostream.readthedocs.io/en/stable/presentation.html
  - https://pytest-asyncio.readthedocs.io/en/latest/concepts.html
  - https://hypothesis.readthedocs.io/en/latest/stateful.html
  - https://returns.readthedocs.io/en/latest/pages/result.html
  - https://psygnal.readthedocs.io/en/stable/
  - https://blinker.readthedocs.io/en/stable/
  - https://github.com/dbrattli/aioreactive
  - https://pypi.org/project/aioreactive/
  - https://rxpy.readthedocs.io/en/latest/reference_operators.html
  - https://docs.python.org/3/library/dataclasses.html
  - https://docs.python.org/3/library/asyncio-dev.html
---

# Appendix: FRP in Python

State of practice as of 2026-09. Foundations (streams over time, effects at the edges, backpressure policy) live in [appendix-frp-general.md](appendix-frp-general.md); this is the Python member of that family, and [appendix-streams.md](appendix-streams.md) covers the transport side.

## Libraries and Building Blocks
- `reactivex` (RxPY) is "a library for composing asynchronous and event-based programs using observable collections and pipable query operators in Python" [1]. Current release 5.1.0, requiring Python `>=3.10,<4.0` [2].
- v5 kept `pipe()` and added chaining: operators "are now available as methods on Observable in addition to the existing `pipe()` style", with "zero breaking changes from 4.x" [3]. 5.1.0 added `tap` as an alias for `do_action` [3].
- Subjects bridge imperative events into the graph. `Subject` broadcasts each notification to all subscribed observers; `BehaviorSubject` "Represents a value that changes over time" and hands new observers the last or initial value; `ReplaySubject` also serves future observers, "subject to buffer trimming policies"; `AsyncSubject` emits only the last value before close [4].
- Schedulers decide where work runs. `AsyncIOScheduler` and `AsyncIOThreadSafeScheduler` live in `reactivex.scheduler.eventloop`; `TrampolineScheduler`, `CurrentThreadScheduler`, `ThreadPoolScheduler`, `VirtualTimeScheduler` and `HistoricalScheduler` live in `reactivex.scheduler` [5]. "You should never schedule timeouts using the TrampolineScheduler, as it will block the thread while waiting" [5].
- Push and pull are duals, and Python has both. aioreactive puts it directly: `AsyncObservable` "may be seen as the dual or opposite of AsyncIterable", and `to_async_iterable` lets you "flip around" to `async for`; because "a push by the `AsyncObservable` will await the pull by the `AsyncIterator`", that conversion is itself backpressure [25]. aioreactive's latest release is 0.20.0 from 2024-09-28 [26], so it is the quiet branch, not the default choice.
- anyio memory object streams are the async-agnostic channel: `create_memory_object_stream(max_buffer_size: float = 0, item_type: object = None)` [10]. "By default, memory object streams are created with a buffer size of 0. This means that `send()` will block until there's another task that calls `receive()`" [9]. An unbounded buffer via `math.inf` is possible but "not recommended" [9]. Streams clone, and each end "is only considered closed once all of its clones have been closed" [9].
- trio channels are the same shape: `open_memory_channel(max_buffer_size)` takes an int or `math.inf`, and "Choosing a sensible value here is important to ensure that backpressure is communicated promptly and avoid unnecessary latency ... If in doubt, use 0" [12]. `statistics()` exposes `current_buffer_used` and `peak_buffer_used` for queue-depth metrics [12].
- Signal libraries are callbacks, not observables. blinker "provides fast & simple object-to-object and broadcast signaling for Python objects" with receivers "automatically disconnected ... via weak referencing"; `send()` calls receivers and `send_async()` awaits them [24]. psygnal is a "Pure python callback/event system modeled after Qt Signals", compiled with mypyc because "signals are often emitted frequently", and converts dataclasses, attrs, pydantic or msgspec objects into evented ones [23]. Neither offers operators, so neither composes into a graph.
- Typed results at the boundary: `returns` supplies `Result` with `Success` and `Failure`, a `@safe` decorator "to convert exception-throwing function to Result container", and `.unwrap()` / `.value_or(default)` for extraction [22]. The stdlib alternative is a frozen dataclass, which `@dataclass(frozen=True)` makes immutable and hashable [28], discriminated by `match`; the example below takes that route.

## Core Patterns
- Intent -> reducer -> effect: a hot `Subject` of intents [4], `scan` folding to state, and an effect layer that maps intents back into intents [1].
- Backpressure is a bounded buffer chosen up front. `asyncio.Queue`: "If maxsize is less than or equal to zero, the queue size is infinite. If it is an integer greater than `0`, then `await put()` blocks when the queue reaches maxsize until an item is removed by `get()`" [7]. anyio and trio expose the same knob on their streams [9][12].
- Cancellation is structured concurrency, not a flag. `asyncio.TaskGroup` (added 3.11) cancels the remaining tasks "The first time any of the tasks belonging to the group fails with an exception other than `asyncio.CancelledError`", then combines failures in an `ExceptionGroup` [8]. `asyncio.timeout` (added 3.11) "will cancel the current task and handle the resulting `asyncio.CancelledError` internally, transforming it into a `TimeoutError`" [8]. On anyio use `CancelScope`, `move_on_after` and `fail_after` [10]; trio has the same three, where `move_on_after` swallows `Cancelled` and `fail_after` raises `TooSlowError` [12].
- Never swallow `CancelledError`. "If end-user code is suppressing cancellation by catching `CancelledError`, it needs to call this method [`uncancel()`] to remove the cancellation state" [8]; otherwise task groups and timeouts silently misbehave.
- Shutdown is explicit. `Queue.shutdown()` (added 3.13) means "Future calls to `put()` raise `QueueShutDown`" and, "Once the queue is empty, future calls to `get()` will raise `QueueShutDown`" [7]. Pair it with `take_until`, which "Returns the values from the source observable sequence until the other observable sequence produces a value" [27], so the reactive half completes at the same instant the queue closes.
- Eager tasks remove scheduling overhead for effects that finish without blocking: with `loop.set_task_factory(asyncio.eager_task_factory)`, "coroutines begin execution synchronously during `Task` construction. Tasks are only scheduled on the event loop if they block"; added 3.12 [8].
- Effects stay at the edges. Operators are "pipable query operators" over a sequence [1], and "Blocking (CPU-bound) code should not be called directly" on the loop [29], so IO belongs in tasks that feed results back as intents rather than inside a `map`.
- Time is a scheduler, not a `sleep`. `VirtualTimeScheduler` "should work with either datetime/timespan or ticks as int/int" and `HistoricalScheduler` "uses datetime for absolute time and timedelta for relative time" [5]; on trio, `MockClock` defaults to rate 0.0, so "the clock only advances through manuals calls to `jump()`" [13].

The same boundary in anyio form, with no reactivex in the pipeline [9] (illustrative):

```python
import anyio
from anyio.streams.memory import MemoryObjectReceiveStream


async def fold(recv: MemoryObjectReceiveStream[int]) -> None:
    total = 0
    async for item in recv:
        total += item
    print("total", total)


async def main() -> None:
    # A bounded buffer is the backpressure policy: send() waits when it is full.
    send, recv = anyio.create_memory_object_stream[int](max_buffer_size=8)
    async with anyio.create_task_group() as tg:
        tg.start_soon(fold, recv)
        with send:
            for i in range(1000):
                await send.send(i)
    print("peak buffer", recv.statistics().max_buffer_size)
```

## Example: asyncio + reactivex intent loop
reactivex supplies the operator algebra and asyncio supplies the concurrency: `AsyncIOScheduler` takes the running loop [5], while the bounded queue [7], the task group and the timeout [8] stay stdlib. This runs to completion on Python 3.13 with reactivex 5.1.0 [2].

```python
"""Intent -> reducer -> effect loop on asyncio + reactivex. Python 3.13."""

import asyncio
from dataclasses import dataclass, replace

from reactivex import Subject
from reactivex import operators as ops
from reactivex.scheduler.eventloop import AsyncIOScheduler


@dataclass(frozen=True, slots=True)
class Load:
    key: str

@dataclass(frozen=True, slots=True)
class Loaded:
    key: str
    value: str

@dataclass(frozen=True, slots=True)
class Failed:
    key: str
    reason: str

Intent = Load | Loaded | Failed

@dataclass(frozen=True, slots=True)
class State:
    status: str = "idle"
    ok: tuple[str, ...] = ()
    err: tuple[str, ...] = ()


def reduce(state: State, intent: Intent) -> State:
    """Pure, total over the Intent union. No IO, no awaits, no exceptions."""
    match intent:
        case Load():
            return replace(state, status="loading")
        case Loaded(key=key, value=value):
            return replace(state, status="ready", ok=(*state.ok, f"{key}={value}"))
        case Failed(key=key, reason=reason):
            return replace(state, status="error", err=(*state.err, f"{key}:{reason}"))


DELAYS = {"alpha": 0.01, "beta": 0.02, "boom": 0.03}


async def fetch(key: str) -> str:
    """The only IO in the program. It lives at the edge, never in an operator."""
    await asyncio.sleep(DELAYS[key])
    if key == "boom":
        raise RuntimeError("upstream refused")
    return f"value-{key}"


async def main() -> None:
    scheduler = AsyncIOScheduler(asyncio.get_running_loop())
    inbox: asyncio.Queue[Intent] = asyncio.Queue(maxsize=2)  # the backpressure boundary
    intents: Subject[Intent] = Subject()
    shutdown: Subject[None] = Subject()
    done = asyncio.Event()

    states = intents.pipe(
        ops.scan(reduce, State()),
        ops.distinct_until_changed(),
        ops.take_until(shutdown),
        ops.share(),
    )

    def observe(state: State) -> None:
        print(f"state: {state.status} ok={list(state.ok)} err={list(state.err)}")
        if len(state.ok) + len(state.err) == len(DELAYS):
            done.set()

    state_sub = states.subscribe(
        on_next=observe,
        on_completed=lambda: print("graph completed"),
    )

    async with asyncio.TaskGroup() as group:

        async def run_effect(intent: Load) -> None:
            try:
                value = await fetch(intent.key)
            except RuntimeError as exc:
                await inbox.put(Failed(intent.key, str(exc)))
            else:
                await inbox.put(Loaded(intent.key, value))

        effects = intents.pipe(
            ops.filter(lambda intent: isinstance(intent, Load)),
            ops.observe_on(scheduler),  # effect tasks are created on the loop
        )
        effect_sub = effects.subscribe(
            on_next=lambda intent: group.create_task(run_effect(intent))
        )

        async def pump() -> None:
            """Drain the bounded inbox into the hot intent stream."""
            while True:
                try:
                    intent = await inbox.get()
                except asyncio.QueueShutDown:  # Python 3.13
                    return
                intents.on_next(intent)
                inbox.task_done()

        group.create_task(pump())
        for key in DELAYS:
            await inbox.put(Load(key))

        try:
            async with asyncio.timeout(2.0):
                await done.wait()
        except TimeoutError:
            print("timed out waiting for effects")

        inbox.shutdown()  # unblocks pump with QueueShutDown
        shutdown.on_next(None)  # completes the state graph via take_until
        effect_sub.dispose()

    state_sub.dispose()


if __name__ == "__main__":
    asyncio.run(main())
```

## Integration Notes
- Textual already is an FRP shell. Reactive attributes are declared with `reactive` (or `var` when no refresh is wanted), and assignment triggers an automatic refresh; `watch_<name>` methods react to changes, `validate_<name>` methods intercept and clamp incoming values, `compute_<name>` methods derive cached values from other reactives, `layout=True` and `recompose=True` widen what a change re-renders, and `data_bind()` propagates a parent reactive into children [15]. Mutating a collection inside a reactive needs `mutate_reactive()` to fire the machinery [15], and `data_bind()` is one-directional, parent to child [15].
- Starlette is where a stream meets the wire. `StreamingResponse` "Takes an async generator or a normal generator/iterator and streams the response body" [16]; server-sent events are third party, via `EventSourceResponse` from `sse-starlette` [16]. The `WebSocket` class "fulfils a similar role to the HTTP request, but ... allows sending and receiving data on a websocket", with `iter_text()`, `iter_bytes()` and `iter_json()` async iterators that exit on `WebSocketDisconnect` [17]. Those iterators are the natural intent source; a bounded per-connection queue is the natural outbound sink [7].
- Django signals are not FRP. They "allow certain senders to notify a set of receivers that some action has taken place", are "called one at a time, in the order they were registered", and adapt across the sync boundary: "Synchronous receivers will be called using `sync_to_async()` when invoked via `asend()`. Asynchronous receivers will be called using `async_to_sync()` when invoked via `send()`" [18]. "All built-in signals, except those in the async request-response cycle, are dispatched using `Signal.send()`" [18], so they are in-process callbacks with no operators, no backpressure and no cancellation.
- Data pipelines without observables: aiostream is "a collection of stream operators that can be combined to create asynchronous pipelines of operations", an "asynchronous version of itertools", with pipelining via `|`, repeatability, a safe iteration context via `async with` and the `stream` method, slicing, and concatenation via `+` [19].

## Testing
- Virtual time in reactivex: `from reactivex.testing import ReactiveTest, TestScheduler` [6]. `scheduler.start()` uses `created` 100, `subscribed` 200 and `disposed` 1000 by default [6]. Notifications are built with `ReactiveTest.on_next(time, value)`, `on_completed(time)` and `on_error(time, exception)` [6]. Marble diagrams are available through `marbles_testing()`, which yields `start, cold, hot, exp` [6].

```python
from reactivex import operators as ops
from reactivex.testing import ReactiveTest, TestScheduler


def test_scan_folds_intents() -> None:
    scheduler = TestScheduler()
    source = scheduler.create_hot_observable(
        ReactiveTest.on_next(210, 1),
        ReactiveTest.on_next(240, 2),
        ReactiveTest.on_completed(300),
    )
    results = scheduler.start(lambda: source.pipe(ops.scan(lambda acc, x: acc + x, 0)))
    assert results.messages == [
        ReactiveTest.on_next(210, 1),
        ReactiveTest.on_next(240, 3),
        ReactiveTest.on_completed(300),
    ]
```

- Async test runners: pytest-asyncio offers strict and auto discovery modes, where "In strict mode pytest-asyncio will only run tests that have the asyncio marker and will only evaluate async fixtures decorated with `@pytest_asyncio.fixture`", auto mode "automatically adds the asyncio marker to all asynchronous test functions", and "strict mode is the default mode" [20]. AnyIO ships its own plugin: mark with `pytest.mark.anyio` or request the `anyio_backend` fixture, and parameterise that fixture over `asyncio` and `trio` to run the same suite on both backends [11].
- Deterministic clocks on trio: `MockClock(rate=0.0, autojump_threshold=math.inf)` advances only via `jump(seconds)`, or automatically once "all tasks have been blocked for this many real seconds" if a threshold is set [13]. `wait_all_tasks_blocked()` "block[s] until there are no runnable tasks", which is how you let a graph settle before asserting [13].
- Property tests over event sequences: Hypothesis `RuleBasedStateMachine` generates sequences of `@rule` actions rather than single inputs, `Bundle` carries generated values between rules, `@invariant` marks "a function to be run after every step", and `@precondition` filters inapplicable rules; run it via the machine's `.TestCase` attribute or `run_state_machine_as_test()` [21].

## Anti-Patterns
- Blocking inside an operator: "if a function performs a CPU-intensive calculation for 1 second, all concurrent asyncio Tasks and IO operations would be delayed by 1 second" [29]. Push that work to `loop.run_in_executor()` [29].
- Catching `CancelledError` and returning normally without `uncancel()`, which breaks the enclosing task group or timeout [8].
- `yield` inside a `TaskGroup` or `asyncio.timeout` in an async generator: PEP 789 (status Draft) documents that this "leads to situations where the wrong task is canceled, timeouts are ignored, and exceptions are mishandled" [14].
- Unbounded buffers: `asyncio.Queue()` with the default `maxsize` is infinite [7], and `math.inf` on an anyio stream is explicitly "not recommended" [9].
- Timeouts on `TrampolineScheduler` or `CurrentThreadScheduler`, both of which block the thread while waiting [5].
- Treating blinker or psygnal as an event bus for domain flow: synchronous callbacks with no operators, no backpressure and no cancellation [23][24]. The same objection applies to Django signals [18].
- Unbounded `ReplaySubject` used as a cache: it serves "future observers, subject to buffer trimming policies" [4], so set the policy.

## Checklist
- One scheduler chosen per graph, and an `AsyncIOScheduler` bound to the running loop wherever operator work touches asyncio [5].
- Every queue and stream given an explicit bound, with `statistics()` or `qsize()` exported as a metric [7][9][12].
- Cancellation path proven: a task group or cancel scope owns every effect, and a shutdown signal completes the reactive half [8][10][12].
- Errors modelled as domain values (`Result`, or a frozen dataclass in the intent union) rather than exceptions crossing operators [22].
- Reducers pure and total, tested table-driven, with a Hypothesis state machine over intent sequences [21].
- Virtual time in tests, never `sleep` [6][13].

## Agreed vs folklore (compressed)
- **Agreed:** backpressure is a bounded buffer, and every mainstream Python channel says so in its own docs [7][9][12]; cancellation belongs to structured concurrency primitives rather than ad hoc flags [8][10][12][14]; time in tests is a scheduler or clock, not a sleep [6][13].
- **"asyncio is FRP":** false as stated. asyncio supplies queues, tasks, cancel scopes and timeouts [7][8], and none of the operator algebra (`scan`, `share`, `distinct_until_changed`, `take_until`) that the family doc treats as the point; that algebra comes from reactivex [1] or aiostream [19].
- **"RxPY is dead":** false. reactivex 5.1.0 is current on PyPI [2] and v5 added method chaining with "zero breaking changes from 4.x" [3]. The dormant project is aioreactive, last released 2024-09-28 [26].
- **"you need Rx to do reactive in Python":** false. Textual ships reactive descriptors, watchers, validators and computed attributes with no observable library [15]; anyio and trio give push streams with real backpressure [9][12]; aiostream gives operator pipelines over async iterators [19].

## Synthesis (inferred)
- **Reach for plain async generators plus `TaskGroup` first.** One producer, one consumer, a linear transform and a bounded queue cover most services, and every piece is stdlib, debuggable and typed. Reach for reactivex when the graph is genuinely a graph: multiple subscribers on one source, latest-wins semantics (`flat_map_latest` or `switch_latest`), fan-in from heterogeneous sources, or time operators (`debounce`, `sample`, `buffer_with_time`) that you would otherwise hand-roll with timers. The break-even point in practice is the second subscriber or the first time-shaping operator.
- **Do not let reactivex own concurrency.** Keep the loop, the queues, the task group and the timeouts in asyncio, and use `AsyncIOScheduler` only to place operator work on that loop. The moment the Observable graph is also the scheduler, cancellation stops being inspectable from the asyncio side.
- **Migration path out of callback soup.** (1) Name the intents: every callback argument becomes a frozen dataclass in one union. (2) Insert a bounded queue between the callback and the logic, so the callback only enqueues. (3) Move all state mutation into one pure reducer driven by that queue, keeping the old callbacks as thin adapters. (4) Move IO out of the reducer into effect tasks that enqueue result intents. (5) Only then, if the graph has branched, replace the hand-written pump with `Subject` plus `scan` plus `share`. Steps 1 to 4 are worth doing even if step 5 never happens.
- **Textual, Django and Starlette are shells, not cores.** Bind them to the reducer at the edge (reactive attribute writes, signal handlers, websocket iterators) and the same domain graph runs headless under test with a virtual clock.
