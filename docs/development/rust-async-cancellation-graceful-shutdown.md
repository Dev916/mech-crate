---
title: "Rust Async Cancellation and Graceful Shutdown Patterns"
category: concurrency
languages: [rust]
complexity: advanced
use_cases:
  - "reasoning about what happens when an async task is cancelled"
  - "writing cancellation-safe select! loops without losing data"
  - "implementing bounded graceful shutdown for a server or daemon"
  - "avoiding futurelock and cancellation-correctness bugs"
summary: "How async cancellation works in Rust (cooperative drop-at-await), what cancellation safety means, and the Tokio patterns for graceful shutdown — CancellationToken, TaskTracker/JoinSet, bounded drain, and the futurelock hazard."
provenance: researched
researched: 2026-07-18
sources:
  - https://tokio.rs/tokio/topics/shutdown
  - https://docs.rs/tokio/latest/tokio/macro.select.html
  - https://docs.rs/tokio-util/latest/tokio_util/sync/struct.CancellationToken.html
  - https://docs.rs/tokio-util/latest/tokio_util/task/task_tracker/struct.TaskTracker.html
  - https://docs.rs/tokio/latest/tokio/task/struct.JoinSet.html
  - https://docs.rs/tokio/latest/tokio/time/fn.timeout.html
  - https://sunshowers.io/posts/cancelling-async-rust/
  - https://rfd.shared.oxide.computer/rfd/0609
  - https://smallcultfollowing.com/babysteps/blog/2022/01/27/panics-vs-cancellation-part-1/
---

# Rust Async Cancellation and Graceful Shutdown Patterns

Purpose: a working model of how cancellation actually happens in async Rust, what "cancellation safety" means precisely, and the concrete Tokio building blocks for shutting a service down cleanly. This complements the concurrency-primitives material (`appendix-rust-concurrency.md`) and the stream-cancellation notes in `appendix-frp-rust.md`; here the focus is async tasks, `select!`, and daemon/server lifecycle.

> All code below is **illustrative** (not compiled against a pinned toolchain in this repo). Type/method names are quoted from the cited primary sources. Citations use `[n]` keyed to the `sources` list in the frontmatter.

## 1. The cancellation model: cooperative, by dropping a future

Async cancellation in Rust is not a signal, an exception, or preemption. A future is an inert state machine that makes progress only when polled; **you cancel it by dropping it** [7][2]. Because a future can only be dropped while it is suspended at an `.await` point, "any Rust future can be cancelled at any await point" [7] — and only at those points. Between await points the code runs to completion; there is no mechanism to interrupt it mid-statement.

When a future is dropped, Rust runs the `Drop` impls of the values currently held on its suspended stack — and nothing more [9]. There is **no async `Drop`** in stable Rust, so you cannot run `.await`-ing cleanup as part of cancellation; async drop "would let you run async code when a future is cancelled ... [but] doesn't exist anything in safe Rust today" for that [7]. This is the root cause of most cancellation pain: the natural place to flush/commit/rollback is an async call, and cancellation is exactly the moment you cannot make one.

### Cancellation looks like a panic from the inside

Niko Matsakis frames cancellation as analogous to unwinding: from inside an async fn, "cancellation looks like the `await` call panicking – it unwinds the stack, running the destructors for all values" [9]. The difference is origin — a panic comes from executing code, a cancellation is "injected from the outside when the async fn's result is no longer needed" [9]. The correctness consequence is the same as exception safety: it is easy to hold invariants at the *start* of each operation but "really, really hard to ensure that those invariants hold all the time," and cancellation can strike during the window where an invariant is temporarily broken [9].

## 2. Cancellation safety

**Definition (Tokio):** "If you have a future that has not yet completed, then it must be a no-op to drop that future and recreate it." [2] Equivalently (sunshowers): cancel safety is "the property of a future that can be cancelled without any side effects" [7]. It is a **local** property of a single future.

sunshowers usefully separates two levels [7]:

- **Cancel safety** — local: dropping the partially-driven future loses no data and leaves no side effect.
- **Cancel correctness** — global: a bug requires *three* things together: (a) a cancel-*unsafe* future, (b) that future is actually cancelled, and (c) the cancellation violates a system invariant. Remove any one and you are fine.

### Why `select!` is where this bites

`tokio::select!` polls several futures concurrently and, when one branch completes, **drops all the others** [7]. In a loop this repeats every iteration, so a cancel-unsafe branch silently loses work each time a sibling wins the race. Tokio's own guidance: "When using `select!` in a loop to receive messages from multiple sources, you should make sure that the receive call is cancellation safe to avoid losing messages." [2]

### The cancel-safe / cancel-unsafe method lists (Tokio)

Tokio documents which of its methods are safe to use as a `select!` branch [2]:

| Cancellation-safe (safe in `select!`) | NOT cancellation-safe (data-loss risk) |
|---|---|
| `mpsc::Receiver::recv`, `UnboundedReceiver::recv` | `AsyncReadExt::read_exact` |
| `broadcast::Receiver::recv` | `AsyncReadExt::read_to_end` / `read_to_string` |
| `watch::Receiver::changed` | `AsyncWriteExt::write_all` |
| `TcpListener::accept`, `UnixListener::accept` | |
| `signal::unix::Signal::recv` | |
| `AsyncReadExt::read`, `AsyncReadExt::read_buf` | |
| `AsyncWriteExt::write`, `AsyncWriteExt::write_buf` | |

A separate class is safe from *data loss* but drops **queue fairness / position** if cancelled: `Mutex::lock`, `RwLock::read`/`write`, `Semaphore::acquire`, `Notify::notified` [2]. Cancelling these loses your place in the wait queue.

The `_buf`/`_exact` split is the key intuition: `read_buf`/`write_buf`/`write_all_buf` are cancel-safe because progress lives in the *caller-owned buffer* that survives the drop, whereas `read_exact`/`write_all` track progress *inside the future* that gets thrown away [2][7]. sunshowers' rule of thumb: prefer `write_all_buf` over `write_all` in cancellable contexts [7].

### Making a `select!` loop safe

Options, in rough order of preference [7]:

1. Use only cancel-safe methods as branches (see table).
2. **Pin the future outside the loop** and pass `&mut fut` so the loop *resumes* the same future instead of restarting it each iteration — but see the futurelock hazard in §7.
3. Move cancel-unsafe work **into a spawned task**, so the work "runs to completion even if the connection is closed" [7], and only `select!` on cancel-safe signals.
4. Rewrite the API to be cancel-safe (e.g. `Sender::reserve()` to obtain a permit, then a non-cancellable `send`) [7].

## 3. Futures vs. tasks: two different cancellation stories

This distinction trips up almost everyone:

- **Dropping a future cancels it immediately** — the state machine stops being polled [7].
- **Dropping a task handle does *not* cancel the task.** "Tasks are driven by the runtime ... With Tokio, dropping a handle to a task does not cause it to be cancelled." [7] A spawned task keeps running detached. To stop it you must call `JoinHandle::abort()` (which cancels at the next await point).

A consequence for shutdown: `tokio::spawn` gives you *detachment*, not *lifetime containment*. Spawned tasks can outlive the scope that created them, which is precisely the "detached task" problem structured concurrency exists to solve (see §4).

## 4. Structured task groups: `JoinSet` vs `TaskTracker`

Both let you own a group of spawned tasks and wait for them, but their **drop semantics are opposite** — choose deliberately.

### `JoinSet` (tokio)

A collection of tasks with the same return type `T`; `join_next()` yields results "in the order they complete," not spawn order [5]. Management: `abort_all()` aborts every task (you still drain with `join_next()`), and the async `shutdown()` "is equivalent to calling `abort_all()` and then calling `join_next()` in a loop until it returns `None`" [5]. **Critically: "When the `JoinSet` is dropped, all tasks in the `JoinSet` are immediately aborted."** [5] Use it when you want results back and when dropping the set *should* kill the work.

### `TaskTracker` (tokio_util)

Tracks tasks to wait on them **without collecting return values and without aborting on drop** [4]. API: `spawn()`, then `close()` ("allows `wait` futures to complete"), then `wait()` which "waits until this `TaskTracker` is both closed and empty" [4]. It is more memory-efficient than `JoinSet` because "once tasks exit, they are immediately removed" [4]. And explicitly: **"Unlike `JoinSet`, dropping a `TaskTracker` does not abort the tasks."** [4] Its documented purpose is graceful shutdown "together with `CancellationToken`" — the token signals stop, the tracker waits for finish [4].

| | `JoinSet` | `TaskTracker` |
|---|---|---|
| Keeps return values | Yes | No (dropped on exit) [4] |
| Drop aborts tasks | **Yes** [5] | **No** [4] |
| Wait for all | loop `join_next()` until `None` [5] | `close()` + `wait()` [4] |
| Force-cancel all | `abort_all()` / `shutdown()` [5] | (pair with `CancellationToken`) [4] |
| Best for | bounded fan-out you own and may abort | graceful drain of long-lived workers |

## 5. `CancellationToken` (tokio_util): the shutdown signal

`CancellationToken` is a cloneable, **level-triggered** cancellation flag — once cancelled it stays cancelled [3].

- `cancel()` — "Cancel the `CancellationToken` and all child tokens which had been derived from it," waking all waiters; "once the call to `cancel` returns, all child nodes have been fully cancelled" [3].
- `cancelled()` — returns a future that resolves when cancellation is requested, and "will complete immediately if the token is already cancelled" [3]. `cancelled_owned()` is the same but owns the token. Both are cancel-safe, so they are ideal `select!` branches [3].
- `child_token()` — parent → child propagation only: a child "will get cancelled whenever the current token gets cancelled," but "cancelling a child token does not cancel the parent token" [3]. This builds a shutdown *tree* (cancel the root to stop everything; cancel a subtree to stop just that subsystem).
- `drop_guard()` — returns a `DropGuard` that "will cancel this token (and all its children) on drop unless disarmed" [3]; RAII cancellation tied to a scope.
- `run_until_cancelled(fut)` — runs `fut`, returning `Some(result)` normally or `None` if cancelled first (the future is dropped on cancel), with a slight fairness bias "towards the future completion" [3].

**When to reach for `CancellationToken` vs a channel** [1][3]: use the token for a one-way, broadcast, level-triggered *stop* signal with hierarchical scopes — its `cancelled()` future is cancel-safe and its tree structure maps onto subsystem shutdown. Use `watch` when observers need the latest *value* (not just "stopped"); use `broadcast`/`mpsc` when you need to deliver discrete *messages* or count consumers. The canonical Tokio shutdown tutorial uses `CancellationToken` for the "tell everyone to stop" step [1].

## 6. Graceful shutdown sequencing for a server/daemon

Tokio decomposes shutdown into three parts [1]:

1. **Figure out when to shut down.**
2. **Tell every part of the program to shut down.**
3. **Wait for every part to finish.**

The canonical wiring [1]:

- **Detect:** `tokio::signal::ctrl_c()` for Ctrl-C; on Unix also select over `signal::unix::signal(SignalKind::terminate())` for SIGTERM (what orchestrators send). For multiple internal shutdown causes, `select!` over the signal plus an `mpsc` channel [1].
- **Tell:** a `CancellationToken`, cloned into every task; tasks `select!` on `token.cancelled()` alongside their real work and clean up when it fires; call `token.cancel()` once to fan out [1].
- **Wait:** a `TaskTracker` — `tracker.spawn()` each worker, then at shutdown `tracker.close()` and `tracker.wait()` until all finish [1][4].

```rust
// Illustrative — Tokio graceful shutdown skeleton.
let token = CancellationToken::new();
let tracker = TaskTracker::new();

for _ in 0..worker_count {
    let token = token.clone();
    tracker.spawn(async move {
        loop {
            tokio::select! {
                // cancel-safe branch: stop signal
                _ = token.cancelled() => break,
                // real work must ALSO be cancel-safe, or moved into its own task
                msg = next_work() => handle(msg).await,
            }
        }
        // cleanup here runs on the task's own time (not during a Drop)
    });
}

// stop accepting new work, then trigger + bound the drain
tracker.close();
tokio::select! {
    _ = tokio::signal::ctrl_c() => {}
    // ... SIGTERM branch ...
}
token.cancel();                                   // step 2: tell everyone
tokio::time::timeout(GRACE, tracker.wait())       // step 3: bounded wait
    .await
    .unwrap_or_else(|_| { /* forced exit path */ });
```

Classic variant without `tokio_util`: give every task a clone of an `mpsc::Sender` and have the main task await `receiver.recv()`, which returns `None` only after **all** senders are dropped — i.e. all tasks have exited. It works because it piggybacks on Rust's drop semantics, but `TaskTracker` expresses the intent directly [1].

For HTTP servers, Hyper/axum expose the same shape via a per-connection graceful mechanism plus a shutdown future: stop accepting, let in-flight requests finish, then drop. Wire the same `CancellationToken`/signal into the server's `with_graceful_shutdown`-style hook and bound it with a timeout (see §7).

## 7. Timeouts and their interaction with cancellation

`tokio::time::timeout(dur, fut)` returns `Result<T, Elapsed>` — `Ok(T)` if `fut` finished in time, `Err(Elapsed)` otherwise [6]. On elapse it **cancels by dropping the wrapped future**: "Cancelling a timeout is done by dropping the future. No additional cleanup or other work is required." [6] It needs the timer driver (`enable_time`/`enable_all`) or it panics [6].

The sharp edge: a timeout **is** a cancellation, so it inherits all of §2. If `fut` is not cancel-safe, timing it out drops it mid-flight and any progress held inside the future is lost [6][2]. Wrapping a `read_exact` or `write_all` in `timeout` inside a retry loop is a data-loss bug. `Timeout::into_inner()` can recover the inner future if you want to resume rather than discard it [6].

Use `timeout` to **bound the drain** in step 3 above: never `tracker.wait().await` unbounded, or one stuck task hangs shutdown forever.

## 8. Futurelock (Oxide RFD 609, 2025)

A subtle, single-task self-deadlock distinct from a classic circular-wait deadlock. Futurelock occurs when "a resource owned by Future A is required for another Future B to proceed, while the Task responsible for both Futures is no longer polling A." [8]

Mechanism [8]: a task drives several futures concurrently (often via `select!` with a `&mut future`). One branch wins; the losing future is **kept alive but no longer polled**. If that parked future holds a shared resource — a `Mutex` guard, a channel permit — then another future that needs the resource can never get it, because the only task that could advance the resource-holder has stopped polling it. Unlike an ordinary deadlock between independent entities, "the task itself becomes the bottleneck."

RFD 609's mitigations [8]:

- **Spawn the work as a separate task** so it is polled independently — turn a held `&mut future` into a `JoinHandle` you `select!` on instead.
- Prefer `tokio::JoinSet` over `FuturesUnordered`/`FuturesOrdered` (JoinSet runs each future in its own task) [8].
- **Avoid `.await` inside a `select!` branch handler** that could interact with the parked futures.
- Do **not** paper over it by enlarging channel capacity — that hides the trigger without removing the hazard [8].

## 9. Anti-patterns

- **Cancel-unsafe branch in a `select!` loop** (`read_exact`, `write_all`, custom `recv`-and-buffer) → silent data loss every time a sibling wins [2][7].
- **Holding a `tokio::sync::Mutex` guard across `.await` inside `select!`** → invariant corruption on cancel and a futurelock trigger [7][8]. sunshowers: "avoid Tokio mutexes" for cross-await critical sections [7].
- **Assuming dropping a `JoinHandle` stops the task** — it does not; the task runs detached [7].
- **Detached `tokio::spawn` with no tracker/token** → no way to drain on shutdown; tasks are killed abruptly at process exit.
- **Relying on cleanup in a `Drop` impl that needs `.await`** — impossible; there is no async drop [7][9]. Do cleanup on the task's own timeline after observing the cancel signal.
- **Unbounded `wait()` at shutdown** → one wedged task hangs the whole process [6].
- **Growing channel capacity to "fix" a futurelock** [8].

## 10. Checklist

- [ ] Every `select!` branch is either cancel-safe or a spawned task; verified against Tokio's list [2].
- [ ] Long-lived workers `select!` on a `CancellationToken::cancelled()` branch [1][3].
- [ ] Shutdown detects SIGTERM *and* Ctrl-C [1].
- [ ] Tasks are owned by a `TaskTracker` (drain) or `JoinSet` (abortable), chosen by drop semantics [4][5].
- [ ] The drain wait is bounded by `tokio::time::timeout` with a defined forced-exit path [6].
- [ ] No `tokio::sync::Mutex` guard is held across an `.await` that sits in a `select!` [7][8].
- [ ] Cleanup lives after the cancel observation, never in an (impossible) async `Drop` [9].
- [ ] No parked `&mut future` holds a lock/permit another branch needs (futurelock) [8].

## Synthesis (inferred)

The following connections and recommendations are the author's synthesis across the cited sources, not direct quotations.

- **A single mental model unifies the whole topic.** Cancellation = "drop a suspended future," and dropping runs only sync destructors. Every rule here is a corollary: cancel safety exists because dropping mid-progress can lose in-future state; the async-drop gap exists because destructors can't `.await`; graceful shutdown is "arrange for cancellation to happen at a safe moment, on the task's own timeline, instead of abruptly." Teaching the drop model first makes the rest deducible rather than memorized.

- **Prefer moving cancel-unsafe work behind a task boundary over hand-proving cancel safety.** Cancel-correctness needs all three of (unsafe future, actually cancelled, invariant broken) [7]; spawning the work removes leg two (the work runs to completion, only the *await on its handle* is cancelled). This is usually cheaper and more robust than auditing every branch for the safe/unsafe list, and it composes with `JoinSet`/`TaskTracker` for the shutdown story.

- **`CancellationToken` + `TaskTracker` is the default pairing; reach for `JoinSet` only when you want return values or abort-on-drop.** The two `tokio_util` types are complementary by design [1][4] — token = "stop," tracker = "are we stopped yet." `JoinSet`'s abort-on-drop is a feature for bounded fan-out (drop the set to cancel a scatter/gather) but a footgun for long-lived workers you meant to drain.

- **Futurelock reads as the dynamic-lifetime shadow of a static borrow-checker guarantee.** In sync Rust, holding a lock guard while blocking is visible and the type system helps; in async, a *parked* future can hold a permit invisibly because "parked" is a runtime state the compiler can't see [8]. The practical takeaway that generalizes RFD 609: treat "a future that holds a resource and might stop being polled" as a hazard equal to "a lock held across a blocking call" — and the antidote is the same as structured concurrency's core promise (a task boundary makes lifetime/polling explicit).

- **Bound every shutdown phase.** Composing §6 and §7: shutdown should be a sequence of *bounded* steps (drain with `timeout`, then escalate to `abort`/`JoinSet::shutdown`, then process exit), so a single misbehaving worker degrades to a hard kill rather than an infinite hang. Cancellation cannot force a task past its next await point, so a forced-exit escape hatch is not optional.

## Further reading

- Yoshua Wuyts, "Async Cancellation" series (2021) — early framing of the problem: <https://blog.yoshuawuyts.com/async-cancellation-1/>
- Matthias Einwag, "Async/Await — The challenges besides syntax: Cancellation" — <https://gist.github.com/Matthias247/ffc0f189742abf6aa41a226fe07398a8>
- Niko Matsakis, "Async cancellation: a case study of pub-sub in mini-redis" — <https://smallcultfollowing.com/babysteps/blog/2022/06/13/async-cancellation-a-case-study-of-pub-sub-in-mini-redis/>
- HN discussions surfacing these sources: "Cancellations in async Rust" (<https://news.ycombinator.com/item?id=45464632>) and "Futurelock: A subtle risk in async Rust" (<https://news.ycombinator.com/item?id=45774086>).
