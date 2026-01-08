# Memory Models: Hardware-Software Concurrency Interface

**Purpose**: Formal specifications of how concurrent reads and writes to shared memory interact, defining legal reorderings and visibility guarantees at the hardware and programming language level.

**Core Insight**: Modern processors and compilers reorder operations for performance. Memory models define which reorderings are allowed, enabling both optimization and correct concurrent programming.

---

## Table of Contents

1. [Foundational Concepts](#foundational-concepts)
2. [Hardware Memory Models](#hardware-memory-models)
3. [Programming Language Memory Models](#programming-language-memory-models)
4. [Progress Guarantees](#progress-guarantees)
5. [Synchronization Primitives](#synchronization-primitives)
6. [Practical Patterns](#practical-patterns)
7. [Stack-Specific Implementations](#stack-specific-implementations)
8. [Integration Points](#integration-points)

---

## Foundational Concepts

### What is a Memory Model?

**Memory Model**: Contract specifying:
- Which operation reorderings are legal (loads, stores)
- When writes become visible to other threads
- Guarantees provided by synchronization operations

**Why Needed**:
- **Compiler Optimizations**: Reorder instructions, eliminate redundant loads
- **CPU Optimizations**: Out-of-order execution, store buffers, cache coherence
- **Performance**: Sequential consistency too restrictive, weak models enable speed

### Sequential Consistency (SC)

**Definition** (Lamport, 1979):
"Result of any execution is the same as if operations of all processors were executed in some sequential order, and operations of each processor appear in program order."

**Intuition**: Interleaving of threads as if single-threaded execution.

**Example (SC)**:
```
Thread 1:        Thread 2:
x = 1            y = 1
r1 = y           r2 = x

Possible outcomes with SC:
- r1=0, r2=1  (T1 then T2)
- r1=1, r2=0  (T2 then T1)
- r1=1, r2=1  (interleaving)
Impossible: r1=0, r2=0 (would require reordering)
```

**Cost**: Requires memory fences after every store → slow on modern hardware.

### Happens-Before Relation

**Happens-Before** (→):
Foundation for reasoning about concurrent execution.

**Rules**:
1. **Program Order**: If A before B in thread, A → B
2. **Synchronization Order**: Release → Acquire on same variable
3. **Transitivity**: A → B and B → C ⇒ A → C

**Data Race**: Concurrent accesses where at least one is write and not ordered by →.

**Data-Race-Free (DRF)**: Program with no data races behaves as if sequentially consistent.

### Reordering Categories

**LoadLoad**: Load followed by load
**LoadStore**: Load followed by store
**StoreStore**: Store followed by store
**StoreLoad**: Store followed by load (hardest to prevent)

**Example**:
```
// Original
x = 1        // Store
r = y        // Load

// Reordered (StoreLoad reordering)
r = y        // Load
x = 1        // Store
```

**Impact**: Other threads may observe loads before preceding stores.

---

## Hardware Memory Models

### x86-64: Total Store Order (TSO)

**Model**: Strong, close to sequential consistency.

**Guarantees**:
- LoadLoad: Not reordered
- LoadStore: Not reordered
- StoreStore: Not reordered
- **StoreLoad**: CAN be reordered (store buffer)

**Store Buffer**: Stores go to buffer before memory, loads bypass buffer if hit, otherwise read memory.

**Example (TSO allows)**:
```
Thread 1:        Thread 2:
x = 1            y = 1
r1 = y           r2 = x

Outcome r1=0, r2=0 POSSIBLE on TSO:
- Both stores in buffers
- Both loads read memory (old values)
- Stores later drain to memory
```

**Fence**: `mfence` instruction ensures all prior stores visible before subsequent loads.

### ARM/RISC-V: Weak Memory Model

**Model**: Very weak, aggressive reordering.

**Allows**:
- All types of reordering (LoadLoad, LoadStore, StoreStore, StoreLoad)
- Stores from same thread observed in different orders by different threads
- Dependent loads can be reordered (speculation)

**Example (ARM allows)**:
```
Thread 1:        Thread 2:
x = 1            y = 1
y = 1            r1 = y
                 r2 = x

Outcome r1=1, r2=0 POSSIBLE on ARM:
- T2 sees y=1 but not x=1 (stores reordered from T2's view)
```

**Barriers**:
- `dmb` (Data Memory Barrier): Order memory accesses
- `dsb` (Data Synchronization Barrier): Complete before subsequent
- `isb` (Instruction Synchronization Barrier): Flush pipeline

### Power: Even Weaker

**Model**: Allows more reorderings than ARM.

**Additional Permissions**:
- Load-Store reordering even with address dependency
- Requires `sync` or `lwsync` barriers

---

## Programming Language Memory Models

### C++11 Memory Model

**Atomics**: `std::atomic<T>` with memory ordering.

**Memory Orders**:
1. **memory_order_relaxed**: No ordering constraints
2. **memory_order_acquire**: Prevents reordering of subsequent loads/stores before acquire
3. **memory_order_release**: Prevents reordering of prior loads/stores after release
4. **memory_order_acq_rel**: Both acquire and release
5. **memory_order_seq_cst**: Sequential consistency (default)

**Acquire-Release Semantics**:
```cpp
std::atomic<int> x{0}, y{0};

// Thread 1
y.store(1, std::memory_order_relaxed);
x.store(1, std::memory_order_release);  // Release

// Thread 2
while (x.load(std::memory_order_acquire) == 0);  // Acquire
assert(y.load(std::memory_order_relaxed) == 1);  // Always true!
```

**Explanation**: Release-acquire pair creates happens-before edge. All writes before release visible after acquire.

**Relaxed Ordering**:
```cpp
std::atomic<int> counter{0};

// Multiple threads
counter.fetch_add(1, std::memory_order_relaxed);
```

**Use Case**: Counters, statistics (when only final value matters).

### Java Memory Model (JMM)

**Happens-Before Rules**:
1. **Program Order**: Within thread
2. **Monitor Lock**: Unlock → Lock on same object
3. **Volatile**: Write → Read of same volatile
4. **Thread Start**: `start()` → first action in new thread
5. **Thread Join**: Last action in thread → `join()` returns
6. **Transitivity**

**Volatile Variables**:
```java
class Example {
    private volatile boolean flag = false;
    private int data = 0;

    // Thread 1
    void writer() {
        data = 42;             // 1
        flag = true;           // 2 (volatile write)
    }

    // Thread 2
    void reader() {
        if (flag) {            // 3 (volatile read)
            assert data == 42; // 4 - Always true!
        }
    }
}
```

**Guarantee**: Volatile write (2) happens-before volatile read (3). By transitivity, (1) happens-before (4).

### Rust Memory Model

**Based on C++11**: Similar atomic operations and orderings.

**Ownership and Borrowing**: Prevents many data races at compile time.

**Atomics**:
```rust
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

static FLAG: AtomicBool = AtomicBool::new(false);
static DATA: AtomicUsize = AtomicUsize::new(0);

// Thread 1
DATA.store(42, Ordering::Relaxed);
FLAG.store(true, Ordering::Release);

// Thread 2
while !FLAG.load(Ordering::Acquire) {}
assert_eq!(DATA.load(Ordering::Relaxed), 42);
```

**Send and Sync Traits**:
- **Send**: Type can be transferred across threads
- **Sync**: Type can be shared across threads (`&T` is Send)

**Compiler Enforcement**: Prevents sharing non-Sync types without synchronization.

---

## Progress Guarantees

### Classifications

**Wait-Free**: Every operation completes in bounded steps.
- **Strongest guarantee**: No starvation possible
- **Example**: Atomic fetch-and-add
- **Hard to implement**: Complex algorithms

**Lock-Free**: Some operation completes in bounded steps (system-wide progress).
- **Guarantee**: System makes progress even if threads stall
- **Example**: Michael-Scott queue, Treiber stack
- **Weaker than wait-free**: Individual thread may starve

**Obstruction-Free**: Operation completes in bounded steps if runs in isolation.
- **Guarantee**: Progress when no contention
- **Example**: Some transactional memory implementations
- **Weakest non-blocking**

**Blocking**: No progress guarantee.
- **Example**: Locks, mutexes
- **Vulnerable**: To deadlock, priority inversion, convoying

### Wait-Free Example: Fetch-And-Add

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

fn wait_free_increment(counter: &AtomicUsize) -> usize {
    // Guaranteed to complete in one atomic operation
    counter.fetch_add(1, Ordering::SeqCst)
}
```

**Bounded**: One atomic operation, always completes.

### Lock-Free Example: Treiber Stack

```rust
use std::sync::atomic::{AtomicPtr, Ordering};
use std::ptr;

struct Node<T> {
    data: T,
    next: *mut Node<T>,
}

struct TreiberStack<T> {
    head: AtomicPtr<Node<T>>,
}

impl<T> TreiberStack<T> {
    fn new() -> Self {
        TreiberStack {
            head: AtomicPtr::new(ptr::null_mut()),
        }
    }

    fn push(&self, data: T) {
        let new_node = Box::into_raw(Box::new(Node {
            data,
            next: ptr::null_mut(),
        }));

        loop {
            let head = self.head.load(Ordering::Acquire);
            unsafe { (*new_node).next = head; }

            // CAS: if head unchanged, swap it with new_node
            if self.head.compare_exchange(
                head,
                new_node,
                Ordering::Release,
                Ordering::Acquire,
            ).is_ok() {
                return; // Success
            }
            // Retry on CAS failure
        }
    }

    fn pop(&self) -> Option<T> {
        loop {
            let head = self.head.load(Ordering::Acquire);
            if head.is_null() {
                return None; // Empty
            }

            let next = unsafe { (*head).next };

            // CAS: if head unchanged, swap with next
            if self.head.compare_exchange(
                head,
                next,
                Ordering::Release,
                Ordering::Acquire,
            ).is_ok() {
                // Success: extract data and free node
                unsafe {
                    let node = Box::from_raw(head);
                    return Some(node.data);
                }
            }
            // Retry on CAS failure
        }
    }
}
```

**Lock-Free**: At least one thread makes progress per iteration (CAS succeeds).

---

## Synchronization Primitives

### Compare-And-Swap (CAS)

**Atomic Operation**:
```
CAS(addr, expected, new):
  atomic {
    old = *addr
    if old == expected:
      *addr = new
      return (true, old)
    else:
      return (false, old)
  }
```

**Use**: Foundation for lock-free data structures.

**ABA Problem**:
```
Thread 1: CAS(ptr, A, B)  // Read A
Thread 2: CAS(ptr, A, C)  // Change to C
Thread 3: CAS(ptr, C, A)  // Change back to A
Thread 1: CAS succeeds!   // But missed C interlude

Solution: Tagged pointers, version numbers
```

### Load-Link/Store-Conditional (LL/SC)

**ARM/RISC-V**: Alternative to CAS.

```
LL(addr):  // Load-linked
  return *addr, reserve(addr)

SC(addr, value):  // Store-conditional
  if reservation valid:
    *addr = value
    return true
  else:
    return false
```

**Advantage**: No ABA problem (reservation invalidated by any write).

### Fetch-And-Add (FAA)

```
FAA(addr, delta):
  atomic {
    old = *addr
    *addr = old + delta
    return old
  }
```

**Use**: Counters, indices, wait-free operations.

---

## Practical Patterns

### Double-Checked Locking (Correct Version)

```rust
use std::sync::{Mutex, Once};
use std::sync::atomic::{AtomicPtr, Ordering};

static INSTANCE: AtomicPtr<Singleton> = AtomicPtr::new(std::ptr::null_mut());
static ONCE: Once = Once::new();

struct Singleton {
    data: i32,
}

fn get_instance() -> &'static Singleton {
    // Fast path: already initialized
    let ptr = INSTANCE.load(Ordering::Acquire);
    if !ptr.is_null() {
        return unsafe { &*ptr };
    }

    // Slow path: initialize
    ONCE.call_once(|| {
        let instance = Box::new(Singleton { data: 42 });
        INSTANCE.store(Box::into_raw(instance), Ordering::Release);
    });

    unsafe { &*INSTANCE.load(Ordering::Relaxed) }
}
```

**Key**: Acquire-release ensures initialization visible.

### Seqlock (Read-Side Scalable)

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

struct Seqlock<T> {
    seq: AtomicUsize,
    data: std::cell::UnsafeCell<T>,
}

impl<T: Copy> Seqlock<T> {
    fn read(&self) -> T {
        loop {
            let seq1 = self.seq.load(Ordering::Acquire);
            if seq1 & 1 != 0 {
                // Writer active, retry
                std::hint::spin_loop();
                continue;
            }

            let data = unsafe { *self.data.get() };
            let seq2 = self.seq.load(Ordering::Acquire);

            if seq1 == seq2 {
                return data; // Consistent read
            }
            // Retry if seq changed
        }
    }

    fn write(&self, value: T) {
        // Increment seq (make odd)
        let seq = self.seq.fetch_add(1, Ordering::Release);
        unsafe { *self.data.get() = value; }
        // Increment seq (make even)
        self.seq.store(seq + 2, Ordering::Release);
    }
}
```

**Use Case**: Rarely-written, frequently-read data (e.g., time-of-day).

### RCU (Read-Copy-Update)

**Concept**: Readers never block, writers create new version.

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicPtr, Ordering};

struct RCU<T> {
    data: AtomicPtr<T>,
}

impl<T> RCU<T> {
    fn read(&self) -> *mut T {
        self.data.load(Ordering::Acquire)
    }

    fn update(&self, new_data: T) {
        let new_ptr = Box::into_raw(Box::new(new_data));
        let old_ptr = self.data.swap(new_ptr, Ordering::Release);

        // Defer freeing old_ptr until readers done
        // (grace period, epoch-based reclamation, etc.)
        unsafe {
            // Simplified: immediate free (unsafe!)
            drop(Box::from_raw(old_ptr));
        }
    }
}
```

**Use Case**: Linux kernel (read-mostly data structures).

---

## Stack-Specific Implementations

### Rust: Atomic Operations

```rust
use std::sync::atomic::{AtomicI32, AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

/// Lock-free counter
struct Counter {
    value: AtomicI32,
}

impl Counter {
    fn new() -> Self {
        Counter {
            value: AtomicI32::new(0),
        }
    }

    fn increment(&self) -> i32 {
        self.value.fetch_add(1, Ordering::SeqCst)
    }

    fn get(&self) -> i32 {
        self.value.load(Ordering::SeqCst)
    }
}

/// Spinlock implementation
struct Spinlock {
    locked: AtomicBool,
}

impl Spinlock {
    fn new() -> Self {
        Spinlock {
            locked: AtomicBool::new(false),
        }
    }

    fn lock(&self) {
        while self.locked.swap(true, Ordering::Acquire) {
            // Spin with hint
            while self.locked.load(Ordering::Relaxed) {
                std::hint::spin_loop();
            }
        }
    }

    fn unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }
}

/// Example: Parallel increment
fn parallel_counter_example() {
    let counter = Arc::new(Counter::new());
    let mut handles = vec![];

    for _ in 0..10 {
        let counter_clone = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            for _ in 0..1000 {
                counter_clone.increment();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Final count: {}", counter.get());
    // Always 10000 (correct!)
}
```

### TypeScript: Atomics and SharedArrayBuffer

```typescript
/**
 * TypeScript/JavaScript atomics (for SharedArrayBuffer)
 */

// Create shared memory
const sab = new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 2);
const sharedArray = new Int32Array(sab);

/**
 * Atomic operations on shared memory
 */
function atomicIncrement(
  array: Int32Array,
  index: number
): number {
  // Atomic fetch-and-add
  return Atomics.add(array, index, 1);
}

function atomicCompareExchange(
  array: Int32Array,
  index: number,
  expected: number,
  replacement: number
): number {
  // Returns old value
  return Atomics.compareExchange(
    array,
    index,
    expected,
    replacement
  );
}

/**
 * Lock-free stack using CAS
 */
class LockFreeStack {
  private head: number = 0;
  private nodes: Array<{ value: any; next: number }> = [];

  push(value: any): void {
    const nodeIndex = this.nodes.length;
    this.nodes.push({ value, next: -1 });

    // CAS loop
    while (true) {
      const currentHead = this.head;
      this.nodes[nodeIndex].next = currentHead;

      // Attempt CAS (simplified, no actual atomic)
      const success = this.compareAndSwapHead(
        currentHead,
        nodeIndex
      );

      if (success) {
        break;
      }
    }
  }

  pop(): any | null {
    while (true) {
      const currentHead = this.head;

      if (currentHead === 0) {
        return null; // Empty
      }

      const headNode = this.nodes[currentHead];
      const nextHead = headNode.next;

      const success = this.compareAndSwapHead(
        currentHead,
        nextHead
      );

      if (success) {
        return headNode.value;
      }
    }
  }

  private compareAndSwapHead(
    expected: number,
    newValue: number
  ): boolean {
    // Simplified CAS (not truly atomic in JS)
    if (this.head === expected) {
      this.head = newValue;
      return true;
    }
    return false;
  }
}

/**
 * Futex-like wait/wake
 */
function waitExample() {
  const sab = new SharedArrayBuffer(4);
  const view = new Int32Array(sab);

  // Worker thread
  const worker = new Worker('worker.js');
  worker.postMessage(sab);

  // Wait for value to change
  Atomics.wait(view, 0, 0);  // Wait if view[0] == 0
  console.log('Woken up!');
}

// In worker.js:
// Atomics.store(view, 0, 1);
// Atomics.notify(view, 0, 1);  // Wake one waiter
```

### PHP: Mutex-Based Concurrency (No Native Atomics)

```php
<?php

namespace Memory;

use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Facades\Redis;

/**
 * Distributed lock using Redis
 */
class DistributedLock
{
    private string $key;
    private int $ttl;

    public function __construct(string $key, int $ttl = 10)
    {
        $this->key = "lock:{$key}";
        $this->ttl = $ttl;
    }

    public function acquire(): bool
    {
        // Redis SET with NX (only if not exists)
        return (bool) Redis::set(
            $this->key,
            getmypid(),
            'NX',
            'EX',
            $this->ttl
        );
    }

    public function release(): bool
    {
        // Only release if we own the lock
        $script = <<<'LUA'
            if redis.call("get", KEYS[1]) == ARGV[1] then
                return redis.call("del", KEYS[1])
            else
                return 0
            end
LUA;

        return (bool) Redis::eval(
            $script,
            1,
            $this->key,
            getmypid()
        );
    }

    public function withLock(callable $callback): mixed
    {
        $maxAttempts = 10;
        $attempt = 0;

        while (!$this->acquire()) {
            if (++$attempt >= $maxAttempts) {
                throw new \RuntimeException('Failed to acquire lock');
            }
            usleep(100000); // 100ms
        }

        try {
            return $callback();
        } finally {
            $this->release();
        }
    }
}

/**
 * Atomic counter using Redis
 */
class AtomicCounter
{
    private string $key;

    public function __construct(string $key)
    {
        $this->key = "counter:{$key}";
    }

    public function increment(): int
    {
        return Redis::incr($this->key);
    }

    public function decrement(): int
    {
        return Redis::decr($this->key);
    }

    public function add(int $delta): int
    {
        return Redis::incrby($this->key, $delta);
    }

    public function get(): int
    {
        return (int) Redis::get($this->key);
    }

    public function compareAndSet(int $expected, int $new): bool
    {
        $script = <<<'LUA'
            local current = tonumber(redis.call("get", KEYS[1]))
            if current == tonumber(ARGV[1]) then
                redis.call("set", KEYS[1], ARGV[2])
                return 1
            else
                return 0
            end
LUA;

        return (bool) Redis::eval(
            $script,
            1,
            $this->key,
            $expected,
            $new
        );
    }
}

/**
 * Read-write lock using Redis
 */
class ReadWriteLock
{
    private string $key;

    public function __construct(string $key)
    {
        $this->key = "rwlock:{$key}";
    }

    public function readLock(): bool
    {
        // Increment reader count
        return Redis::hincrby($this->key, 'readers', 1) > 0;
    }

    public function readUnlock(): void
    {
        Redis::hincrby($this->key, 'readers', -1);
    }

    public function writeLock(): bool
    {
        $maxAttempts = 100;

        for ($i = 0; $i < $maxAttempts; $i++) {
            $script = <<<'LUA'
                local readers = tonumber(redis.call("hget", KEYS[1], "readers") or "0")
                local writer = redis.call("hget", KEYS[1], "writer")

                if readers == 0 and not writer then
                    redis.call("hset", KEYS[1], "writer", ARGV[1])
                    return 1
                else
                    return 0
                end
LUA;

            $acquired = Redis::eval(
                $script,
                1,
                $this->key,
                getmypid()
            );

            if ($acquired) {
                return true;
            }

            usleep(10000); // 10ms
        }

        return false;
    }

    public function writeUnlock(): void
    {
        Redis::hdel($this->key, 'writer');
    }
}
```

---

## Integration Points

### With Consistency Models
- **Memory model ⊆ Consistency model**: Memory model for shared memory, consistency for distributed
- **SC memory model = Linearizability**: Both require total order

### With Concurrency Primitives
- **Locks use memory barriers**: Ensure critical section visibility
- **Lock-free uses atomics**: Build on memory model guarantees

### With Formal Verification
- **Model check memory models**: Verify absence of data races
- **Litmus tests**: Small programs testing memory model behaviors

---

## Further Reading

### Papers
- Adve & Gharachorloo (1996) - "Shared Memory Consistency Models: A Tutorial"
- Boehm & Adve (2008) - "Foundations of the C++ Concurrency Memory Model"
- Manson, Pugh, Adve (2005) - "The Java Memory Model"
- Herlihy (1991) - "Wait-Free Synchronization"

### Books
- Herlihy & Shavit - "The Art of Multiprocessor Programming"
- McKenney - "Is Parallel Programming Hard, And, If So, What Can You Do About It?"
- Goetz et al. - "Java Concurrency in Practice"

### Tools
- **CDSCHECKER**: Model checker for C++ memory model
- **Relacy Race Detector**: Test concurrent C++ programs
- **ThreadSanitizer**: Dynamic data race detector

---

**End of Memory Models Appendix**
