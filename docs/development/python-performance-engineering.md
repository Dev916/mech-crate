---
title: "Python Performance Engineering: Measure First, Then Free-Threading, JIT, Async, and Native Code"
category: patterns
languages: [python]
complexity: advanced
use_cases:
  - finding where a slow Python service actually spends time before changing any code
  - choosing between threads, processes, sub-interpreters, asyncio, and a free-threaded build
  - deciding whether to reach for NumPy, Cython, mypyc, numba, or Rust, and what the packaging costs
  - tuning a forked Python web service (worker model, GC around fork, serialisation, caching)
summary: "A measurement-first playbook for Python performance as of 2026-09: which profiler answers which question, what the 3.13/3.14/3.15 interpreter changes actually deliver (free-threading at a 5-10% single-thread penalty, a JIT that is still opt-in, tail-calling interpreter figures), a concurrency decision table, the native acceleration ladder and its packaging bill, and service-level tuning."
provenance: researched
researched: 2026-09-18
sources:
  - https://docs.python.org/3/whatsnew/3.14.html
  - https://docs.python.org/3.15/whatsnew/3.15.html
  - https://docs.python.org/3/whatsnew/3.13.html
  - https://devguide.python.org/versions/
  - https://peps.python.org/pep-0703/
  - https://peps.python.org/pep-0779/
  - https://peps.python.org/pep-0744/
  - https://peps.python.org/pep-0836/
  - https://peps.python.org/pep-0734/
  - https://peps.python.org/pep-0669/
  - https://peps.python.org/pep-0799/
  - https://docs.python.org/3.15/library/profiling.sampling.html
  - https://docs.python.org/3/library/profile.html
  - https://docs.python.org/3/library/sys.monitoring.html
  - https://docs.python.org/3/howto/perf_profiling.html
  - https://docs.python.org/3/library/tracemalloc.html
  - https://docs.python.org/3/library/timeit.html
  - https://pyperf.readthedocs.io/en/latest/analyze.html
  - https://pyperf.readthedocs.io/en/latest/system.html
  - https://github.com/benfred/py-spy
  - https://github.com/plasma-umass/scalene
  - https://py-free-threading.github.io/running-gil-disabled/
  - https://py-free-threading.github.io/tracking/
  - https://developers.redhat.com/articles/2026/09/14/python-314-free-threaded-build-now-available-rhel
  - https://docs.python.org/3/library/concurrent.futures.html
  - https://docs.python.org/3/library/asyncio-task.html
  - https://docs.python.org/3/library/asyncio-dev.html
  - https://github.com/MagicStack/uvloop
  - https://uvicorn.dev/deployment/
  - https://gunicorn.org/design/
  - https://docs.python.org/3/library/gc.html
  - https://numpy.org/doc/stable/user/basics.broadcasting.html
  - https://cython.readthedocs.io/en/latest/src/quickstart/overview.html
  - https://mypyc.readthedocs.io/en/latest/introduction.html
  - https://numba.readthedocs.io/en/stable/user/5minguide.html
  - https://pyo3.rs/latest/free-threading.html
  - https://docs.python.org/3/c-api/stable.html
  - https://docs.python.org/3/library/dataclasses.html
  - https://docs.python.org/3/library/array.html
  - https://docs.python.org/3/library/functools.html
  - https://docs.python.org/3/faq/programming.html
  - https://github.com/ijl/orjson
  - https://msgspec.dev/benchmarks.html
  - https://pydantic.dev/docs/validation/latest/concepts/performance/
  - https://fidget-spinner.github.io/posts/jit-reflections.html
---

# Python Performance Engineering: Measure First, Then Free-Threading, JIT, Async, and Native Code

State of practice as of 2026-09. The seed sources are the CPython What's New pages for 3.13, 3.14 and 3.15 [1][2][3] and the PEPs behind the runtime changes: free-threading (703, 779) [5][6], the JIT (744, 836) [7][8], multiple interpreters (734) [9], low-impact monitoring (669) [10] and the profiling package (799) [11]. Library claims come from each project's own documentation, and vendor benchmarks are labelled as such. Version context: 3.13 and 3.14 are in bugfix status (3.14 first released 2025-10-07), 3.15 is in prerelease with a first release date of 2026-10-01, and 3.10 is security-only until 2026-10 [4]. Inline `[n]` cites key to `sources`. Code is illustrative unless stated otherwise.

## 1. Measure first: pick the profiler that answers your question

The two stdlib profiling styles are not interchangeable. "Deterministic profiling is meant to reflect the fact that all function call, function return, and exception events are monitored, and precise timings are made for the intervals between these events"; "statistical profiling (which is not done by this module) randomly samples the effective instruction pointer, and deduces where time is being spent" [13]. Of the two tracing profilers shipped, `cProfile` "is recommended for most users; it's a C extension with reasonable overhead", while the pure-Python `profile` "adds significant overhead to profiled programs" [13]. In 3.15 `profile` is deprecated and slated for removal in 3.17 [2].

The cost of tracing is not only wall-clock. PEP 799 notes that tracing profilers instrument every function call and return and that this "can disable certain interpreter optimizations, such as those introduced by PEP 659" [11]. A cProfile run therefore measures a differently-optimised interpreter than the one in production, which is why hot-loop attributions from cProfile and from a sampler often disagree.

Python 3.15 reorganises the built-ins: `profiling.tracing` (relocated from `cProfile`, which stays as an alias) and `profiling.sampling`, a new statistical sampler named Tachyon [2][11]. Tachyon "enables low-overhead performance analysis of running Python processes without requiring code modification or process restart"; the docs claim "virtually zero overhead while achieving sampling rates of up to 1,000,000 Hz" and note "the target process is never stopped or paused during sampling" [2][12]. Default sampling rate is 1 kHz, adjustable with `-r/--sampling-rate`; modes are `wall` (default), `cpu`, `gil` and `exception`; outputs include pstats, collapsed stacks, flamegraph HTML, gecko, heatmap and a binary format for later replay [12].

```bash
# Python 3.15: sample a running service by PID, CPU time only, flamegraph out
python -m profiling.sampling attach 12345 --mode=cpu --flamegraph
# one-shot stack snapshot, no profiling session
python -m profiling.sampling dump 12345
# pre-3.15 equivalent, no code change, no restart
py-spy record --pid 12345 --native --subprocesses -o profile.svg
```

| Question | Reach for | Why |
|---|---|---|
| Where does a running production process spend wall time? | `profiling.sampling attach` (3.15) or `py-spy record --pid` | Both sample from outside the process; py-spy "doesn't run in the same process as the profiled Python program" and supports CPython 2.3-2.7 and 3.3-3.14 [12][20] |
| Is the time in Python or in the C extension underneath? | Scalene, or `py-spy --native` | Scalene "separates out time spent in Python from time in native code (including libraries)"; py-spy profiles native extensions with `--native` on some platforms [20][21] |
| Which line allocates, and how much is being copied? | Scalene | Line-level and per-function profiling, plus copy volume in MB/s "making it easy to spot inadvertent copying" [21] |
| Exact call counts for a short script | `profiling.tracing` / `cProfile` | Sampling docs steer you to deterministic profiling for scripts under a second, exact call counts, and micro-benchmarks where differences are 1-2% [12][13] |
| Where is memory retained between two points? | `tracemalloc` | `start()`, `take_snapshot()`, `compare_to()`; added in 3.4 [16] |
| Native frames, kernel time, whole-stack flame graphs on Linux | Linux `perf` | Support added in 3.12 via `-X perf`, `PYTHONPERFSUPPORT=1`, or `sys.activate_stack_trampoline("perf")` [15] |
| Custom, always-on instrumentation | `sys.monitoring` | Added in 3.12; per-location `DISABLE` means "a program can be run under a debugger with no overhead if the debugger disables all monitoring except for a few breakpoints" [14] |

Two practical notes on the last two rows. For `perf`, the interpreter should be built with frame pointers: without them "perf support still works but with higher overhead because Python must generate unwinding information dynamically and perf must use slower DWARF debugging information to unwind the stack" [15]. Python 3.15 makes this the default: "CPython is now built with frame pointers enabled by default (PEP 831). Pass `--without-frame-pointers` to opt out" [2]. For `sys.monitoring`, PEP 669's own measurement is that "if no events are active, this PEP should have a small positive impact on performance", with experiments showing "between 1 and 2% speedup" [10]. Separately, 3.14 added a "zero-overhead debugging interface that allows debuggers and profilers to safely attach to running Python processes without stopping or restarting them", exposed as `sys.remote_exec()`, plus `python -m asyncio ps PID` and `pstree PID` for live task introspection [1].

**Avoiding noisy benchmarks.** `timeit` "temporarily turns off garbage collection during timing", which makes runs comparable but hides GC cost; re-enable it with `gc.enable()` in the setup string when GC is part of what you are measuring [17]. `timeit` also advises taking the minimum, not the mean: "the lowest value gives a lower bound for how fast your machine can run the given code snippet; higher values in the result vector are typically not caused by variability in Python's speed, but by other processes interfering with your timing accuracy" [17]. For anything you will quote to other people, use `pyperf`: "If you run a benchmark without tuning the system, it's likely that you will get outliers: a few values much slower than the average", and it prefers "median and median absolute deviation (MAD) instead of mean and standard deviation" because those are "robust statistics which ignore outliers" [18]. `pyperf system tune` sets the CPU governor to `performance`, pins `scaling_min_freq` to the maximum, disables turbo boost, stops irqbalance and adjusts IRQ affinity, and sets the perf event max sample rate to 1; the docs say CPU isolation and pinning have "a significant impact on the stability of benchmarks" [19].

Checklist before you believe a number, assembled from the above: same machine, tuned system [19], median plus MAD over many runs [18], GC in whichever state matches production [17], and the same build flags as production, since the JIT, the tail-calling interpreter and the free-threaded build each move the baseline [1][2].

## 2. Interpreter changes, with the numbers the docs actually give

| Change | Status as of 2026-09 | The figure the primary source states |
|---|---|---|
| Free-threaded build (PEP 703) | Supported but optional since 3.14 (phase II) [1][6] | "The performance penalty on single-threaded code in free-threaded mode is now roughly 5-10%, depending on the platform and C compiler used" [1] |
| JIT (PEP 744) | Experimental, not in the default build [7] | 3.14: "the typical performance impact of enabling it can range from 10% slower to 20% faster, depending on workload" [1]. 3.15: "8-9% geometric mean performance improvement ... on x86-64 Linux", "12-13% speedup over the tail calling interpreter" on AArch64 macOS, with per-benchmark results "from roughly 15% slowdown to over 100% speedup"; the page adds "These results are not yet final" [2] |
| Tail-calling interpreter | Opt-in build variant in 3.14; default for python.org Windows 64-bit binaries in 3.15 [1][2] | 3.14: "a geometric mean of 3-5% faster on the standard pyperformance benchmark suite ... The baseline is Python 3.14 built with Clang 19, without this new interpreter" [1]. 3.15 on Windows: "between 15-20% speedup on the geometric mean of pyperformance on Windows x86-64 over the switch-case interpreter on an AMD Ryzen 7 5800X" [2] |
| Cycle GC | Generational again from 3.14.5 onward [1][2] | The incremental GC in 3.14.0-3.14.4 "has been reverted back to the generational GC from 3.13" after "a number of reports of significant memory pressure in production environments" [1] |
| Sub-interpreters (PEP 734) | `concurrent.interpreters` and `InterpreterPoolExecutor` new in 3.14 [1][9][25] | "Each interpreter has its own Global Interpreter Lock, so code running in one interpreter can run on one CPU core, while code in another interpreter runs unblocked on a different core" [25] |
| Allocator | 3.15 | "mimalloc is now used as the default allocator for raw memory allocations such as via `PyMem_RawMalloc()` for better performance on free-threaded builds" [2] |

**Free-threading, read precisely.** PEP 703 added the `--disable-gil` build configuration, which defines `Py_GIL_DISABLED`, and pays for thread safety with biased reference counting, deferred reference counting and immortal objects; the PEP's own pyperformance overhead table reported 6% one-thread / 8% multi-thread on Intel Skylake and 5% / 7% on AMD Zen 3 [5]. 3.13 shipped it as experimental, warning of "a substantial single-threaded performance hit", with a separate `python3.13t` executable and `sys._is_gil_enabled()` to check at runtime [3]. PEP 779 set the bar for phase II: a 15% single-threaded regression as the hard performance target and 20% memory overhead, accepted 2025-06-16 for 3.14 [6]. 3.14 delivered: the PEP 659 specializing adaptive interpreter is now enabled in free-threaded mode, the single-thread penalty is "roughly 5-10%", and "The free-threaded build of Python is now supported and no longer experimental" [1]. Phase III, free-threading as the default or sole build, is explicitly "still undecided" [1].

Three operational facts before you ship a free-threaded build:

1. **The GIL can come back under you.** Importing a C extension that does not declare free-threading support re-enables the GIL at runtime; `PYTHON_GIL=0` or `python -Xgil=0` forces it off [22][24]. Detect the build with `sysconfig.get_config_var("Py_GIL_DISABLED")` and the live state with `sys._is_gil_enabled()` [22].
2. **Thread safety is your problem, not the interpreter's.** Built-in `dict` and `list` have internal locks, but "sequences of operations are not atomic"; use `threading.Lock` or queues for shared mutable state [24].
3. **Ecosystem coverage is good but not universal.** The free-threading compatibility tracker lists support landing in NumPy 2.1.0, SciPy 1.15.0, pandas 2.2.3, PyTorch 2.6.0, Cython 3.1.0, pydantic 2.11.0 and cryptography 46.0.0 [23]. Pure Python needs no changes by design [23].

```python
import sys
import sysconfig


def threading_profile() -> dict[str, bool]:
    """Report the build and the live GIL state (illustrative)."""
    freethreaded = bool(sysconfig.get_config_var("Py_GIL_DISABLED"))
    gil_on = getattr(sys, "_is_gil_enabled", lambda: True)()
    return {"freethreaded_build": freethreaded, "gil_enabled": bool(gil_on)}
```

**The JIT, read precisely.** It is a copy-and-patch template JIT; PEP 744 states "the JIT is about as fast as the existing specializing interpreter on most platforms", costs roughly 10-20% more memory, and that "The JIT is currently not part of the default build configuration, and it is likely to remain that way for the foreseeable future" [7]. The bar it must clear to stop being experimental is a meaningful improvement "(realistically, on the order of 5%)" on at least one popular platform [7]. 3.13 exposed it via `--enable-experimental-jit` with "modest" improvements [3]; 3.14 ships it in the official macOS and Windows binaries but "it is not recommended for production use" and must be switched on with `PYTHON_JIT=1`, with `sys._jit.is_available()` for introspection [1]. 3.15's rebuilt JIT (LLVM 21 stencils, a new tracing frontend, basic register allocation) is where the 8-9% and 12-13% figures come from, and the docs still mark them as not final [2]. PEP 836, a draft targeting 3.16, says outright "This PEP does not propose declaring the JIT as supported immediately", and default enablement "would require a separate final approval from the Release Manager" [8]. Ken Jin, who is "primarily responsible for Python's JIT compiler's optimizer", wrote in 2025 that "CPython 3.13's JIT ranges from slower to the interpreter to roughly equivalent to the interpreter" and singled out "inaccurate coverage of the JIT" as a problem [45].

Decision rule for the interpreter layer: treat the JIT as a measurable experiment per workload, never as a default. Treat the tail-calling interpreter as a build-time choice (it needs Clang 19 or newer on x86-64 or AArch64, and PGO is "highly recommended") [1]. Treat free-threading as an architecture decision, because it changes what your code must guarantee.

## 3. Picking a concurrency model

The choice is driven by two questions: is the work I/O-bound or CPU-bound, and does it need to share objects?

| Model | Parallel on multiple cores? | Sharing | Reach for it when | Cost |
|---|---|---|---|---|
| `asyncio` | No (one thread) | Shared objects, one thread at a time | Many concurrent I/O waits, especially sockets | Any blocking call stalls everything [27] |
| Threads (GIL build) | No for Python bytecode | Shared objects | Blocking I/O in libraries without async APIs | Since 3.13 `ThreadPoolExecutor` defaults `max_workers` to `min(32, (os.process_cpu_count() or 1) + 4)`, a default the docs explain as one that "preserves at least 5 workers for I/O bound tasks" and "utilizes at most 32 CPU cores for CPU bound tasks which release the GIL" [25] |
| Threads (free-threaded build) | Yes | Shared objects, you provide the locking | CPU-bound work over shared in-memory state | 5-10% single-thread penalty; extensions can re-enable the GIL [1][24] |
| Sub-interpreters | Yes | Isolated; pickle or buffers across a queue | CPU-bound work with clean task boundaries, inside one process | Startup "has not been optimized yet", each interpreter "uses more memory than necessary", few sharing options "other than memoryview" [1] |
| Processes | Yes | Isolated; pickle | CPU-bound work with large independent tasks, or unsafe C libraries | `ProcessPoolExecutor` "allows it to side-step the Global Interpreter Lock" but "only picklable objects can be executed and returned" [25] |

Sub-interpreters are the new option and the least familiar. PEP 734 describes them as isolated: "interpreters never share objects (except in very specific cases with immortal, immutable builtin objects)"; queues move data, not references, and objects implementing the buffer protocol can share underlying data directly [9]. The 3.14 docs pitch them as "the isolation of processes with the efficiency of threads" [1], and `InterpreterPoolExecutor` is a `ThreadPoolExecutor` subclass whose workers each hold their own interpreter and own GIL [25]. The catch is the same as multiprocessing: "the worker serializes the callable and arguments using pickle when sending them to its interpreter" [25]. That makes them a poor fit for chatty, large-payload workloads and a good fit for coarse tasks.

Rule of thumb that falls out of the table: if the work is I/O-bound, do not reach past asyncio or a thread pool, which is the case the executor default is explicitly sized for [25]. If it is CPU-bound over large shared in-memory state, the free-threaded build is the only row that does not serialise, since both processes and sub-interpreters pickle their arguments [1][25]. If it is CPU-bound and cleanly partitionable, processes remain the portable answer because they "side-step the Global Interpreter Lock" on any build [25], with sub-interpreters as the same-process alternative whose current cost is slower startup and extra memory per interpreter [1].

## 4. Asyncio practice

**Never block the loop.** The asyncio docs are blunt: "Blocking (CPU-bound) code should not be called directly. For example, if a function performs a CPU-intensive calculation for 1 second, all concurrent asyncio Tasks and IO operations would be delayed by 1 second" [27]. Offload with `loop.run_in_executor()` against a thread, interpreter or process pool [27], or with `asyncio.to_thread()`, which is "primarily intended to be used for executing IO-bound functions/methods that would otherwise block the event loop if they were run in the main thread" [26]. Turn on debug mode in staging: "Callbacks taking longer than 100 milliseconds are logged", and the threshold is tunable via `loop.slow_callback_duration` [27].

**Prefer `TaskGroup` to `gather`.** Added in 3.11, "TaskGroup provides stronger safety guarantees than gather for scheduling a nesting of subtasks: if a task (or a subtask, a task scheduled by a task) raises an exception, TaskGroup will, while gather will not, cancel the remaining scheduled tasks" [26]. That is a correctness win that shows up as a performance win, because orphaned tasks keep consuming connections and CPU after the request that spawned them has failed.

**Consider the eager task factory, and understand what it changes.** With `loop.set_task_factory(asyncio.eager_task_factory)`, "coroutines begin execution synchronously during Task construction. Tasks are only scheduled on the event loop if they block. This can be a performance improvement as the overhead of loop scheduling is avoided for coroutines that complete synchronously. A common example where this is beneficial is coroutines which employ caching or memoization to avoid actual I/O when possible" [26]. The docs flag the trade explicitly: "Immediate execution of the coroutine is a semantic change ... the application's task execution order is likely to change" [26]. So it pays exactly when a large share of your awaits are cache hits, and it is a behaviour change you must test, not a free switch.

**uvloop** is the drop-in event loop built on libuv; its README claims "uvloop makes asyncio 2-4x faster" and it supports Python 3.8 or greater, with `uvloop.run(main())` the preferred entry point as of 0.18 [28].

```python
import asyncio

import uvloop


async def fetch(client, url: str) -> bytes:
    return await client.get(url)


async def main(client, urls: list[str]) -> list[bytes]:
    loop = asyncio.get_running_loop()
    loop.set_task_factory(asyncio.eager_task_factory)
    results: list[bytes] = []
    async with asyncio.TaskGroup() as tg:
        tasks = [tg.create_task(fetch(client, u)) for u in urls]
    results.extend(t.result() for t in tasks)
    # CPU-bound post-processing belongs off the loop
    return await asyncio.to_thread(lambda: [r.upper() for r in results])


def run(client, urls: list[str]) -> list[bytes]:
    return uvloop.run(main(client, urls))
```

## 5. The native acceleration ladder

Climb only as far as the measurement justifies. Each rung past the first turns a pure-Python distribution into a compiled one, which brings the Limited API and ABI decisions of section 5's last paragraph with it [37].

| Rung | What it buys | What it costs | Primary source says |
|---|---|---|---|
| NumPy vectorisation | The loop moves to C | Rewrite in array terms; intermediates can blow up memory | "Broadcasting provides a means of vectorizing array operations so that looping occurs in C instead of Python ... without making needless copies of data" [32] |
| numba | JIT compile numeric kernels via a decorator | NumPy-shaped code only; first-call compile time | "Numba is a just-in-time compiler for Python that works best on code that uses NumPy arrays and functions, and loops"; "Numba doesn't know about pd.DataFrame" [35] |
| mypyc | Compile annotated Python modules to C extensions | Needs type annotations; a build step | "often 1.5x to 5x faster when compiled", "5x to 10x" for code tuned for mypyc; "This is how mypy achieved a 4x performance improvement over interpreted Python" [34] |
| Cython | Full C-level control from Python-like source | A second language; type declarations are where the speed comes from | "The source code gets translated into optimized C/C++ code and compiled as Python extension modules"; speedups come "from optional static type declarations" [33] |
| Rust via PyO3 and maturin | A real systems language for the hot module | A Rust toolchain, a wheel matrix, and free-threading auditing | PyO3 has supported free-threaded builds "since version 0.23"; from 0.28 it "defaults to assuming Python modules created with it are thread-safe" [36] |

The ordering matters because the rungs differ in what they do to your build. NumPy and numba are ordinary runtime dependencies and change nothing about how you ship [32][35]. mypyc and Cython compile your modules into C extensions, so you now publish platform wheels [33][34]. PyO3 adds a Rust toolchain and, on free-threaded builds, an auditing obligation for `unsafe` code on top of that [36].

**The packaging bill.** A native extension can target the Limited API and ship one `abi3` wheel that works "across multiple minor Python versions", at the cost that `Py_LIMITED_API` "disables inlining, allowing stability as data structures improve" with "possibly reduced performance"; note also that "Python does not verify that extensions conform to the Stable ABI" [37]. Free-threading complicates this. PyO3's guide states "The free-threaded build uses a completely new ABI and there is not yet an equivalent to the limited API for the free-threaded ABI", requiring version-specific wheels [36]. Python 3.15 answers that with PEP 803: "C extensions that target the Stable ABI can now be compiled for the new Stable ABI for Free-Threaded Builds (also known as abi3t)" [2]. The migration is not free: it "usually requires some non-trivial changes to the source code", specifically PEP 697 style negative `basicsize` plus `PyObject_GetTypeData()`, and a switch from `PyInit_` to the PEP 793 `PyModExport_*` hook [2]. And as of the 3.15 docs, "these tools do not support abi3t" (Setuptools, meson-python, scikit-build-core, Maturin are named), so "Extensions that cannot switch to abi3t should continue to build for the existing Stable ABI (abi3) and the version-specific ABI for free-threading (cp315t) separately" [2].

**Rust extensions in a free-threaded world** need explicit work. Authors declare thread safety with `#[pymodule(gil_used = false)]` or opt out with `gil_used = true` if the `unsafe` code has not been audited; `pyclass` borrow checking "will raise exceptions (or in some cases panic) to enforce exclusive access for mutable borrows", which is more likely to bite without a GIL; calling CPython C APIs requires `Python::attach()`, and long-running work without CPython access must use `Python::detach()` "to avoid hanging other threads during garbage collection and similar global synchronization events" [36].

## 6. Data structure and memory tactics

Everything in this section is pure Python and needs no build step, so none of it carries the wheel and ABI cost of section 5 [37].

- **`__slots__` and `dataclass(slots=True)`** for high-cardinality record types. The dataclass parameter was added in 3.10; it generates `__slots__` and "a new class will be returned instead of the original one", raising `TypeError` if `__slots__` is already defined [38]. Two caveats: passing parameters to a base class `__init_subclass__()` with `slots=True` raises `TypeError`, and since 3.11 inherited field names are omitted from the generated `__slots__`, so "do not use `__slots__` to retrieve the field names of a dataclass. Use `fields()` instead" [38].
- **`array` and `memoryview`** when the data is homogeneous numbers and NumPy is too heavy a dependency. The `array` module provides "an object type that can compactly represent an array of basic values", and "Array objects also implement the buffer interface, and may be used wherever bytes-like objects are supported" [39]. The buffer protocol is also how bulk data crosses a sub-interpreter boundary without a pickle round trip, since objects implementing it "share underlying data directly" [9].
- **String building.** "str and bytes objects are immutable, therefore concatenating many strings together is inefficient as each concatenation creates a new object. In the general case, the total runtime cost is quadratic in the total string length." The recommended idioms are a list plus `str.join()` (or `io.StringIO`) for `str`, and a `bytearray` with `+=` for `bytes` [41].
- **`functools.cache`** for pure functions with hashable arguments. It is `lru_cache(maxsize=None)`, and "Because it never needs to evict old values, this is smaller and faster than `@lru_cache` with a size limit" [40]. Two traps: "The cache keeps references to the arguments and return values until they age out of the cache or until the cache is cleared", so an unbounded cache on a long-lived process is a memory leak with good intentions; and while the cache is threadsafe, "It is possible for the wrapped function to be called more than once if another thread makes an additional call before the initial call has been completed and cached" [40].
- **Serialisation.** orjson's README claims it is "something like 10x as fast as `json`" to serialise and "something like 2x as fast as `json`" to deserialise, outputs `bytes` rather than `str`, and natively handles dataclasses, datetimes, numpy arrays and UUIDs; these are the author's own benchmarks [42]. msgspec's own benchmarks put "msgspec structs" fastest and say "When used without schemas, msgspec is on-par with orjson (the next fastest JSON library)", with a validation benchmark reporting msgspec "~12x faster than pydantic V2"; the author states the bias plainly: "I wrote msgspec, naturally whatever benchmark I publish it's going to perform well in" [43]. Treat both as vendor benchmarks and re-run them on your payloads.
- **pydantic v2** has documented tips worth applying before you conclude validation is the bottleneck: prefer `model_validate_json()` over `model_validate(json.loads(...))`; instantiate a `TypeAdapter` once because "Each time a `TypeAdapter` is instantiated, it will construct a new validator and serializer"; use concrete `list`/`dict` rather than `Sequence`/`Mapping`; prefer tagged (discriminated) unions; and on a simple benchmark in the docs themselves, "TypedDict is about ~2.5x faster than nested models". The page also tells you to avoid wrap validators "if you really care about performance", since they are "generally slower than other validators" [44].

```python
from dataclasses import dataclass
from functools import cache


@dataclass(slots=True, frozen=True)
class Point:
    x: float
    y: float


@cache
def parse_rule(source: str) -> tuple[str, ...]:
    """Pure, hashable input, bounded key space: a safe use of @cache."""
    return tuple(part.strip() for part in source.split(","))


def render(points: list[Point]) -> str:
    chunks = [f"{p.x},{p.y}" for p in points]
    return ";".join(chunks)
```

## 7. Service level: worker models, pools, and GC around fork

**Worker model.** Gunicorn "uses a pre-fork worker model: an arbiter process manages worker processes, while the workers handle requests and responses" [30]. Its worker types are `sync` (the default, one request at a time, no keep-alive, and it "Requires a buffering proxy (nginx, HAProxy) for production"), `gthread` (a thread pool per worker, with keep-alive), ASGI workers for asyncio frameworks, `gevent` (greenlets, thousands of concurrent connections, "May require patches for some libraries"), and `tornado` [30]. Sizing guidance, verbatim in spirit: workers are not clients, "Gunicorn typically needs only 4-12 workers to handle heavy traffic", and the starting formula is `workers = (2 x CPU cores) + 1` [30]. Reach for async workers when you have "Long blocking calls (external APIs, slow databases)", "Direct internet traffic without a buffering proxy", streaming bodies, long polling, or WebSockets [30].

Uvicorn takes a different tack: "Unlike gunicorn, uvicorn does not use pre-fork, but uses spawn, which allows uvicorn's multiprocess manager to still work well on Windows" [29]. Its docs recommend running under a process manager so you can "perform server upgrades without dropping requests", note that `--reload` is for local development and that "--reload and --workers arguments are mutually exclusive", and describe proxy header handling for deployments behind one or more proxies [29].

**GC around fork.** This is the highest-leverage tuning knob in a pre-fork service, and it is documented in the stdlib. "If a process will `fork()` without `exec()`, avoiding unnecessary copy-on-write in child processes will maximize memory sharing and reduce overall memory usage." The prescribed sequence is: "call `gc.disable()` early in the parent process, `gc.freeze()` right before `fork()`, and `gc.enable()` early in child processes", where `gc.freeze()` moves tracked objects "to a permanent generation" that is ignored "in all the future collections"; all three were added in 3.7 [31].

```python
import gc


def post_fork_parent_ready() -> None:
    """Gunicorn: call from when_ready, after the app is imported."""
    gc.disable()
    gc.freeze()


def post_fork_child() -> None:
    """Gunicorn: call from post_fork, in each worker."""
    gc.enable()
```

**Connection pooling.** Nothing in the Python runtime changes the fact that a pool is per process. With a pre-fork model, total connections is workers times pool size, which is the usual way a Python service exhausts a database's connection limit; the gunicorn sizing note that 4-12 workers is normally enough is the relevant constraint [30]. The gunicorn docs frame the `gthread` trade as threads sharing memory for a lower footprint against workers isolating failures for better fault tolerance [30], and the same trade governs pooling: fewer, fatter workers means fewer pools to size.

## Agreed vs folklore (compressed)

**Agreed** (each with a primary source): tracing profilers change what they measure, not just how long it takes [11][13]; sampling from outside the process is the production-safe default [12][20]; free-threading is supported-but-optional with a 5-10% single-thread cost [1][6]; sub-interpreters give true multi-core parallelism but serialise with pickle [25]; blocking calls in an async loop stall every other task [27]; `TaskGroup` cancels siblings where `gather` does not [26]; a native extension is a packaging commitment, not just a compile step [2][36][37]; `gc.freeze()` before fork is the documented way to preserve copy-on-write sharing [31].

**Folklore, and what the sources actually say:**

- *"Python is slow, so rewrite it in Rust."* The documented ladder starts much lower. Vectorisation moves the loop to C with no packaging change [32]; mypyc reports "1.5x to 5x" on already-annotated code with no rewrite [34]. Rust is the rung that additionally costs a toolchain, a wheel matrix, an ABI decision, and free-threading auditing of your `unsafe` code [36][37][2].
- *"The GIL makes threads useless."* It never made them useless for I/O; the stdlib's `ThreadPoolExecutor` default is sized so that it "preserves at least 5 workers for I/O bound tasks" [25]. And since 3.14 there is a supported build where threads do run Python bytecode in parallel [1][6]. What is true is that on a GIL build, threads do not help CPU-bound Python.
- *"Async is always faster."* Async has no parallelism to offer CPU-bound code, and one CPU-bound second delays every concurrent task by that second [27]. Even within async, the eager task factory is a win specifically when coroutines "complete synchronously", such as cache hits, and it comes with an ordering change [26].
- *"The JIT makes everything faster now."* As of 2026-09 the JIT is not in the default build and PEP 744 expects it "to remain that way for the foreseeable future" [7]. 3.14's own docs give a range "from 10% slower to 20% faster" [1], 3.15's improved numbers are labelled "not yet final" and still include "roughly 15% slowdown" cases [2], and the optimizer's maintainer has publicly called the coverage of JIT speedups inaccurate [45].
- *"Free-threading means my code is now parallel."* Importing one unprepared C extension re-enables the GIL for the whole process [22][24], and `dict`/`list` internal locks do not make "sequences of operations" atomic [24].

## Synthesis (inferred)

**The decision rule: measure, then take the cheapest rung that clears the gap.**

1. Attach a sampler to the real workload first (`profiling.sampling attach` on 3.15, py-spy before that). Do not start with cProfile, because it perturbs the optimiser you are trying to characterise.
2. Read the profile for its *shape*, not its top line. Time in native frames means the Python layer is not the problem. Time spread thinly across thousands of small calls means an interpreter-level change (annotations plus mypyc, or a build with the tail-calling interpreter) may pay. Time in one hot loop means vectorise or compile that loop. Time waiting means it is a concurrency problem, not a compute problem.
3. Exhaust the free rungs before the expensive ones, in this order: algorithm, data structure (`slots`, `array`, generators), serialisation library, caching, concurrency model, interpreter build flags, native extension. Each step down this list costs strictly more to maintain and to ship.
4. Re-measure on a tuned system with median plus MAD, and keep the benchmark in the repository. A speedup you cannot reproduce next quarter is not a speedup.

**Applied to a typical Python service in this stack.** Assume a containerised API under gunicorn or uvicorn talking to Postgres, roughly the shape described in `docker-assembly-guide.md`:

- **Pin the version deliberately.** 3.14 is the right default today: free-threading is supported, the GC regression of 3.14.0 through 3.14.4 is fixed from 3.14.5, and it has bugfix support to 2030-10 [1][4]. Wait for 3.15's general release before adopting it for the sampling profiler and the Windows tail-calling interpreter, since it is still prerelease as of 2026-09 [2][4].
- **Leave the JIT off** and keep `PYTHON_JIT=1` as an A/B experiment in staging with a pyperf-tuned benchmark, because the documented range spans a slowdown [1][2].
- **Do not move to a free-threaded build for a request-per-worker web service.** A pre-fork model already uses every core; you would pay the 5-10% single-thread penalty for parallelism you are not using [1]. Free-threading earns its keep where one process must share large mutable state across CPU-bound threads, which is the batch/ETL/model-serving shape, not the CRUD shape.
- **Put `gc.disable()` plus `gc.freeze()` in the arbiter and `gc.enable()` in `post_fork`** [31]. On a service with a large import graph and many workers this is usually the single largest resident-memory win available, and it is four lines.
- **Replace `json` with orjson or msgspec at the serialisation boundary only**, after confirming from the profile that encoding is actually hot; both vendors' numbers are large but self-published [42][43].
- **Size the pool as workers times pool size**, and prefer fewer, threaded workers when the database connection limit binds before CPU does [30].

**Where this connects to the rest of the corpus.** The free-threading decision is the same decision `appendix-rust-concurrency.md` frames for Rust: shared-mutable-state parallelism buys you throughput and hands you a correctness obligation. The memory-visibility reasoning you need once the GIL stops serialising your threads is the model in `appendix-memory-models.md`. And the build-flag choices above (which interpreter, which ABI, which wheels) are container-image decisions, so they belong in the image layering discussed in `docker-assembly-guide.md` rather than in application code.
