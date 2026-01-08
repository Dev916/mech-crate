# Rust: Threading and Concurrency Deep Dive

Comprehensive guide to Rust's concurrency primitives with focus on atomics vs synchronization primitives.

## Table of Contents

1. [Concurrency Fundamentals](#concurrency-fundamentals)
2. [Atomics: Lock-Free Synchronization](#atomics-lock-free-synchronization)
3. [Mutex: Exclusive Access](#mutex-exclusive-access)
4. [RwLock: Shared vs Exclusive Access](#rwlock-shared-vs-exclusive-access)
5. [Performance Comparison](#performance-comparison)
6. [When to Use What](#when-to-use-what)
7. [Advanced Patterns](#advanced-patterns)

---

## Concurrency Fundamentals

### Rust's Memory Model

Rust follows the **C++11 memory model** with strong guarantees:

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

// Ordering determines memory visibility guarantees
// Relaxed: No ordering constraints
// Acquire: Loads can't be reordered before this
// Release: Stores can't be reordered after this
// AcqRel: Combination of Acquire + Release
// SeqCst: Sequentially consistent (strongest)
```

**Key Concepts**:
- **Data Races**: Prevented by Rust's ownership system
- **Memory Ordering**: Controls visibility of operations across threads
- **Happens-Before**: Relationship ensuring one operation is visible to another

### Thread Creation

```rust
use std::thread;
use std::time::Duration;

fn thread_basics() {
    // Spawn a thread
    let handle = thread::spawn(|| {
        println!("Hello from thread!");
        42
    });

    // Wait for completion and get result
    let result = handle.join().unwrap();
    println!("Thread returned: {}", result);  // 42

    // Spawn with move semantics
    let data = vec![1, 2, 3];
    let handle = thread::spawn(move || {
        println!("Data: {:?}", data);
    });
    handle.join().unwrap();
    // `data` is no longer accessible here
}

fn scoped_threads() {
    let mut data = vec![1, 2, 3];

    // Scoped threads can borrow non-'static data
    thread::scope(|s| {
        s.spawn(|| {
            println!("Length: {}", data.len());
        });

        s.spawn(|| {
            data.push(4);
        });
    }); // All threads guaranteed joined here

    println!("Data: {:?}", data);  // [1, 2, 3, 4]
}
```

---

## Atomics: Lock-Free Synchronization

### Atomic Types

```rust
use std::sync::atomic::{
    AtomicBool, AtomicI32, AtomicU64, AtomicUsize, AtomicPtr,
    Ordering,
};
use std::sync::Arc;
use std::thread;

/// Basic atomic operations
fn atomic_basics() {
    let counter = AtomicUsize::new(0);

    // Load with different orderings
    let value = counter.load(Ordering::SeqCst);
    let relaxed = counter.load(Ordering::Relaxed);
    let acquire = counter.load(Ordering::Acquire);

    // Store
    counter.store(10, Ordering::SeqCst);
    counter.store(20, Ordering::Release);

    // Fetch and modify atomically
    let prev = counter.fetch_add(5, Ordering::SeqCst);  // Returns old value
    let prev = counter.fetch_sub(3, Ordering::SeqCst);
    let prev = counter.fetch_max(15, Ordering::SeqCst);
    let prev = counter.fetch_min(10, Ordering::SeqCst);

    // Compare and swap
    let current = 10;
    match counter.compare_exchange(
        current,
        20,
        Ordering::SeqCst,  // Success ordering
        Ordering::SeqCst,  // Failure ordering
    ) {
        Ok(prev) => println!("Swapped: {} -> 20", prev),
        Err(actual) => println!("Failed, actual: {}", actual),
    }

    // Weak compare_exchange (can spuriously fail, but faster)
    let _ = counter.compare_exchange_weak(20, 30, Ordering::SeqCst, Ordering::SeqCst);
}
```

### Memory Orderings Explained

```rust
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;

/// Relaxed ordering - no synchronization, only atomicity
fn relaxed_ordering() {
    let data = AtomicUsize::new(0);
    let ready = AtomicBool::new(false);

    thread::scope(|s| {
        // Writer
        s.spawn(|| {
            data.store(42, Ordering::Relaxed);  // Can be reordered!
            ready.store(true, Ordering::Relaxed);
        });

        // Reader
        s.spawn(|| {
            while !ready.load(Ordering::Relaxed) {
                thread::yield_now();
            }
            // DANGER: data might still be 0!
            // Relaxed provides no happens-before guarantee
            println!("Data: {}", data.load(Ordering::Relaxed));
        });
    });
}

/// Acquire-Release ordering - establishes happens-before
fn acquire_release_ordering() {
    let data = AtomicUsize::new(0);
    let ready = AtomicBool::new(false);

    thread::scope(|s| {
        // Writer
        s.spawn(|| {
            data.store(42, Ordering::Relaxed);
            ready.store(true, Ordering::Release);  // Release: all previous stores visible
        });

        // Reader
        s.spawn(|| {
            while !ready.load(Ordering::Acquire) {  // Acquire: see all previous stores
                thread::yield_now();
            }
            // SAFE: data is guaranteed to be 42
            println!("Data: {}", data.load(Ordering::Relaxed));
        });
    });
}

/// SeqCst ordering - strongest guarantee (total order)
fn seqcst_ordering() {
    let x = AtomicBool::new(false);
    let y = AtomicBool::new(false);
    let z = AtomicUsize::new(0);

    thread::scope(|s| {
        s.spawn(|| {
            x.store(true, Ordering::SeqCst);
        });

        s.spawn(|| {
            y.store(true, Ordering::SeqCst);
        });

        s.spawn(|| {
            while !x.load(Ordering::SeqCst) {}
            if y.load(Ordering::SeqCst) {
                z.fetch_add(1, Ordering::SeqCst);
            }
        });

        s.spawn(|| {
            while !y.load(Ordering::SeqCst) {}
            if x.load(Ordering::SeqCst) {
                z.fetch_add(1, Ordering::SeqCst);
            }
        });
    });

    // With SeqCst: z will be 1 or 2 (never 0)
    // With Relaxed: z could be 0!
    println!("z = {}", z.load(Ordering::SeqCst));
}
```

### Practical Atomic Examples

```rust
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Atomic counter - multiple threads incrementing
fn atomic_counter_example() {
    let counter = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            for _ in 0..1000 {
                counter.fetch_add(1, Ordering::Relaxed);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Final count: {}", counter.load(Ordering::Relaxed));  // 10000
}

/// Spinlock using atomics
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
        // Try to acquire lock
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            // Spin (busy wait)
            while self.locked.load(Ordering::Relaxed) {
                std::hint::spin_loop();  // Optimization hint
            }
        }
    }

    fn unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }
}

fn spinlock_example() {
    let lock = Arc::new(Spinlock::new());
    let mut handles = vec![];

    for i in 0..5 {
        let lock = Arc::clone(&lock);
        let handle = thread::spawn(move || {
            lock.lock();
            println!("Thread {} acquired lock", i);
            thread::sleep(Duration::from_millis(100));
            lock.unlock();
            println!("Thread {} released lock", i);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

/// Lock-free stack using atomics
struct LockFreeStack<T> {
    head: AtomicPtr<Node<T>>,
}

struct Node<T> {
    data: T,
    next: *mut Node<T>,
}

impl<T> LockFreeStack<T> {
    fn new() -> Self {
        LockFreeStack {
            head: AtomicPtr::new(std::ptr::null_mut()),
        }
    }

    fn push(&self, data: T) {
        let new_node = Box::into_raw(Box::new(Node {
            data,
            next: std::ptr::null_mut(),
        }));

        let mut head = self.head.load(Ordering::Relaxed);
        loop {
            unsafe {
                (*new_node).next = head;
            }

            match self.head.compare_exchange_weak(
                head,
                new_node,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => head = actual,
            }
        }
    }

    fn pop(&self) -> Option<T> {
        let mut head = self.head.load(Ordering::Acquire);
        loop {
            if head.is_null() {
                return None;
            }

            let next = unsafe { (*head).next };
            match self.head.compare_exchange_weak(
                head,
                next,
                Ordering::Acquire,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    let data = unsafe { Box::from_raw(head).data };
                    return Some(data);
                }
                Err(actual) => head = actual,
            }
        }
    }
}
```

---

## Mutex: Exclusive Access

### Mutex Basics

```rust
use std::sync::{Arc, Mutex};
use std::thread;

/// Basic mutex usage
fn mutex_basics() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            // Lock automatically releases when guard drops
            let mut num = counter.lock().unwrap();
            *num += 1;
        }); // Lock released here
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Result: {}", *counter.lock().unwrap());  // 10
}

/// Mutex with complex data
#[derive(Debug)]
struct BankAccount {
    balance: i64,
    transactions: Vec<i64>,
}

impl BankAccount {
    fn new(initial: i64) -> Self {
        BankAccount {
            balance: initial,
            transactions: vec![initial],
        }
    }

    fn deposit(&mut self, amount: i64) {
        self.balance += amount;
        self.transactions.push(amount);
    }

    fn withdraw(&mut self, amount: i64) -> Result<(), String> {
        if self.balance >= amount {
            self.balance -= amount;
            self.transactions.push(-amount);
            Ok(())
        } else {
            Err("Insufficient funds".to_string())
        }
    }
}

fn bank_account_example() {
    let account = Arc::new(Mutex::new(BankAccount::new(1000)));

    let mut handles = vec![];

    // Multiple deposits
    for i in 0..5 {
        let account = Arc::clone(&account);
        let handle = thread::spawn(move || {
            let mut acc = account.lock().unwrap();
            acc.deposit(100);
            println!("Thread {} deposited 100", i);
        });
        handles.push(handle);
    }

    // Multiple withdrawals
    for i in 0..3 {
        let account = Arc::clone(&account);
        let handle = thread::spawn(move || {
            let mut acc = account.lock().unwrap();
            match acc.withdraw(200) {
                Ok(_) => println!("Thread {} withdrew 200", i),
                Err(e) => println!("Thread {} failed: {}", i, e),
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let acc = account.lock().unwrap();
    println!("Final balance: {}", acc.balance);
    println!("Transactions: {:?}", acc.transactions);
}
```

### Avoiding Deadlocks

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// DEADLOCK: Two mutexes locked in different order
fn deadlock_example() {
    let resource1 = Arc::new(Mutex::new(0));
    let resource2 = Arc::new(Mutex::new(0));

    let r1 = Arc::clone(&resource1);
    let r2 = Arc::clone(&resource2);

    // Thread 1: locks resource1 then resource2
    let t1 = thread::spawn(move || {
        let _g1 = r1.lock().unwrap();
        thread::sleep(Duration::from_millis(10));
        let _g2 = r2.lock().unwrap();  // Waits for thread 2
        println!("Thread 1 acquired both locks");
    });

    let r1 = Arc::clone(&resource1);
    let r2 = Arc::clone(&resource2);

    // Thread 2: locks resource2 then resource1
    let t2 = thread::spawn(move || {
        let _g2 = r2.lock().unwrap();
        thread::sleep(Duration::from_millis(10));
        let _g1 = r1.lock().unwrap();  // Waits for thread 1
        println!("Thread 2 acquired both locks");
    });

    // DEADLOCK: Both threads wait forever
    t1.join().unwrap();
    t2.join().unwrap();
}

/// SOLUTION: Always lock in same order
fn no_deadlock_example() {
    let resource1 = Arc::new(Mutex::new(0));
    let resource2 = Arc::new(Mutex::new(0));

    let mut handles = vec![];

    for i in 0..2 {
        let r1 = Arc::clone(&resource1);
        let r2 = Arc::clone(&resource2);

        let handle = thread::spawn(move || {
            // Always lock resource1 first, then resource2
            let _g1 = r1.lock().unwrap();
            thread::sleep(Duration::from_millis(10));
            let _g2 = r2.lock().unwrap();
            println!("Thread {} acquired both locks", i);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

/// try_lock to avoid blocking
fn trylock_example() {
    let resource = Arc::new(Mutex::new(0));

    let r = Arc::clone(&resource);
    let t1 = thread::spawn(move || {
        let _g = r.lock().unwrap();
        thread::sleep(Duration::from_millis(100));
    });

    thread::sleep(Duration::from_millis(10));

    // Try to acquire without blocking
    match resource.try_lock() {
        Ok(_guard) => println!("Acquired lock"),
        Err(_) => println!("Could not acquire lock"),
    }

    t1.join().unwrap();
}
```

### Mutex with Condvar

```rust
use std::sync::{Arc, Mutex, Condvar};
use std::thread;
use std::time::Duration;

/// Producer-Consumer with Condvar
struct Queue<T> {
    items: Mutex<Vec<T>>,
    condvar: Condvar,
}

impl<T> Queue<T> {
    fn new() -> Self {
        Queue {
            items: Mutex::new(Vec::new()),
            condvar: Condvar::new(),
        }
    }

    fn push(&self, item: T) {
        let mut items = self.items.lock().unwrap();
        items.push(item);
        self.condvar.notify_one();  // Wake up one waiting thread
    }

    fn pop(&self) -> T {
        let mut items = self.items.lock().unwrap();
        while items.is_empty() {
            items = self.condvar.wait(items).unwrap();  // Wait for item
        }
        items.remove(0)
    }
}

fn producer_consumer_example() {
    let queue = Arc::new(Queue::new());

    // Producer
    let q = Arc::clone(&queue);
    let producer = thread::spawn(move || {
        for i in 0..10 {
            q.push(i);
            println!("Produced: {}", i);
            thread::sleep(Duration::from_millis(100));
        }
    });

    // Consumers
    let mut consumers = vec![];
    for id in 0..3 {
        let q = Arc::clone(&queue);
        let consumer = thread::spawn(move || {
            loop {
                let item = q.pop();
                println!("Consumer {} got: {}", id, item);
                if item == 9 {
                    break;
                }
            }
        });
        consumers.push(consumer);
    }

    producer.join().unwrap();
    for consumer in consumers {
        consumer.join().unwrap();
    }
}
```

---

## RwLock: Shared vs Exclusive Access

### RwLock Basics

```rust
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

/// RwLock allows multiple readers OR one writer
fn rwlock_basics() {
    let data = Arc::new(RwLock::new(vec![1, 2, 3]));

    // Multiple readers can acquire simultaneously
    let mut readers = vec![];
    for i in 0..5 {
        let data = Arc::clone(&data);
        let reader = thread::spawn(move || {
            let read_guard = data.read().unwrap();
            println!("Reader {} sees: {:?}", i, *read_guard);
            thread::sleep(Duration::from_millis(50));
        });
        readers.push(reader);
    }

    thread::sleep(Duration::from_millis(10));

    // Writer must wait for all readers
    let data_clone = Arc::clone(&data);
    let writer = thread::spawn(move || {
        let mut write_guard = data_clone.write().unwrap();
        write_guard.push(4);
        println!("Writer added 4");
    });

    for reader in readers {
        reader.join().unwrap();
    }
    writer.join().unwrap();

    println!("Final: {:?}", *data.read().unwrap());
}
```

### RwLock Read-Heavy Workload

```rust
use std::sync::{Arc, RwLock};
use std::thread;
use std::collections::HashMap;

/// Cache with RwLock - many reads, few writes
struct Cache {
    data: RwLock<HashMap<String, String>>,
}

impl Cache {
    fn new() -> Self {
        Cache {
            data: RwLock::new(HashMap::new()),
        }
    }

    fn get(&self, key: &str) -> Option<String> {
        // Acquire read lock (shared)
        let data = self.data.read().unwrap();
        data.get(key).cloned()
    }

    fn insert(&self, key: String, value: String) {
        // Acquire write lock (exclusive)
        let mut data = self.data.write().unwrap();
        data.insert(key, value);
    }

    fn update_if_exists(&self, key: &str, value: String) -> bool {
        // Upgrade from read to write lock pattern
        {
            let data = self.data.read().unwrap();
            if !data.contains_key(key) {
                return false;  // Release read lock
            }
        }

        // Acquire write lock
        let mut data = self.data.write().unwrap();
        // Recheck (another thread might have removed it)
        if data.contains_key(key) {
            data.insert(key.to_string(), value);
            true
        } else {
            false
        }
    }
}

fn cache_example() {
    let cache = Arc::new(Cache::new());

    // Pre-populate
    cache.insert("key1".to_string(), "value1".to_string());
    cache.insert("key2".to_string(), "value2".to_string());

    let mut handles = vec![];

    // Many readers
    for i in 0..20 {
        let cache = Arc::clone(&cache);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                let key = format!("key{}", (i % 2) + 1);
                if let Some(value) = cache.get(&key) {
                    assert!(value.starts_with("value"));
                }
            }
        });
        handles.push(handle);
    }

    // Few writers
    for i in 0..2 {
        let cache = Arc::clone(&cache);
        let handle = thread::spawn(move || {
            for j in 0..10 {
                let key = format!("key{}", i + 1);
                let value = format!("value{}_{}", i + 1, j);
                cache.insert(key, value);
                thread::sleep(Duration::from_millis(10));
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
```

### RwLock Pitfalls

```rust
use std::sync::{Arc, RwLock};
use std::thread;

/// Writer starvation - readers can starve writers
fn writer_starvation_example() {
    let data = Arc::new(RwLock::new(0));

    // Constant stream of readers
    for i in 0..100 {
        let data = Arc::clone(&data);
        thread::spawn(move || {
            loop {
                let _guard = data.read().unwrap();
                // Reader holds lock briefly
                thread::sleep(Duration::from_micros(100));
            }
        });
    }

    // Writer might wait indefinitely
    let data_clone = Arc::clone(&data);
    let writer = thread::spawn(move || {
        println!("Writer waiting...");
        let mut guard = data_clone.write().unwrap();
        *guard += 1;
        println!("Writer succeeded!");
    });

    thread::sleep(Duration::from_secs(1));
    // Writer might still be waiting...
}
```

---

## Performance Comparison

### Benchmark: Atomics vs Mutex vs RwLock

```rust
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Instant;

fn benchmark_atomics(iterations: usize) -> Duration {
    let counter = Arc::new(AtomicUsize::new(0));
    let start = Instant::now();

    let mut handles = vec![];
    for _ in 0..8 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            for _ in 0..iterations {
                counter.fetch_add(1, Ordering::Relaxed);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    start.elapsed()
}

fn benchmark_mutex(iterations: usize) -> Duration {
    let counter = Arc::new(Mutex::new(0));
    let start = Instant::now();

    let mut handles = vec![];
    for _ in 0..8 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            for _ in 0..iterations {
                let mut num = counter.lock().unwrap();
                *num += 1;
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    start.elapsed()
}

fn benchmark_rwlock_write(iterations: usize) -> Duration {
    let counter = Arc::new(RwLock::new(0));
    let start = Instant::now();

    let mut handles = vec![];
    for _ in 0..8 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            for _ in 0..iterations {
                let mut num = counter.write().unwrap();
                *num += 1;
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    start.elapsed()
}

fn benchmark_rwlock_read(iterations: usize) -> Duration {
    let data = Arc::new(RwLock::new(vec![1, 2, 3, 4, 5]));
    let start = Instant::now();

    let mut handles = vec![];
    for _ in 0..8 {
        let data = Arc::clone(&data);
        let handle = thread::spawn(move || {
            for _ in 0..iterations {
                let guard = data.read().unwrap();
                let _sum: i32 = guard.iter().sum();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    start.elapsed()
}

fn run_benchmarks() {
    let iterations = 100_000;

    println!("Benchmarking {} iterations per thread (8 threads):", iterations);
    println!();

    let atomic_time = benchmark_atomics(iterations);
    println!("Atomics:        {:?}", atomic_time);

    let mutex_time = benchmark_mutex(iterations);
    println!("Mutex:          {:?} ({:.2}x slower)",
             mutex_time,
             mutex_time.as_secs_f64() / atomic_time.as_secs_f64());

    let rwlock_write_time = benchmark_rwlock_write(iterations);
    println!("RwLock (write): {:?} ({:.2}x slower)",
             rwlock_write_time,
             rwlock_write_time.as_secs_f64() / atomic_time.as_secs_f64());

    let rwlock_read_time = benchmark_rwlock_read(iterations);
    println!("RwLock (read):  {:?} ({:.2}x slower)",
             rwlock_read_time,
             rwlock_read_time.as_secs_f64() / atomic_time.as_secs_f64());

    /*
    Typical results:
    Atomics:        15ms
    Mutex:          180ms (12x slower)
    RwLock (write): 210ms (14x slower)
    RwLock (read):  45ms (3x slower)
    */
}
```

### Performance Characteristics

| Primitive | Read Cost | Write Cost | Contention | Cache Coherence |
|-----------|-----------|------------|------------|-----------------|
| **Atomics** | Very Fast | Very Fast | Lock-free | High traffic |
| **Mutex** | Slow | Slow | Blocks | Low traffic |
| **RwLock** | Fast | Slow | Readers parallel | Medium traffic |
| **Spinlock** | Fast | Fast | Burns CPU | High traffic |

**Key Insights**:
- **Atomics**: Best for simple counters, flags, and lock-free data structures
- **Mutex**: Best for complex critical sections with rare contention
- **RwLock**: Best for read-heavy workloads with occasional writes
- **Spinlock**: Best for very short critical sections (microseconds)

---

## When to Use What

### Decision Tree

```
Do you need to protect complex data?
├─ NO → Use Atomics
│   ├─ Simple counter/flag? → AtomicUsize/AtomicBool
│   ├─ Need pointer updates? → AtomicPtr
│   └─ Building lock-free structure? → Atomics + careful ordering
│
└─ YES → Use locks
    ├─ Many readers, few writers? → RwLock
    │   └─ Writer starvation concern? → Consider parking_lot::RwLock
    │
    ├─ Balanced read/write? → Mutex
    │   └─ Very short critical section? → Consider Spinlock
    │
    └─ Need condition variables? → Mutex + Condvar
```

### Use Cases

**Use Atomics When**:
```rust
// ✅ Simple counters
let counter = AtomicUsize::new(0);
counter.fetch_add(1, Ordering::Relaxed);

// ✅ Flags
let shutdown = AtomicBool::new(false);
shutdown.store(true, Ordering::Release);

// ✅ Lock-free algorithms
// Complex: ABA problem, memory reclamation, etc.
```

**Use Mutex When**:
```rust
// ✅ Complex data structures
let map = Mutex::new(HashMap::new());

// ✅ Multiple fields need consistency
let account = Mutex::new(BankAccount { balance: 1000, history: vec![] });

// ✅ Rare contention
let cache = Mutex::new(compute_expensive_result());
```

**Use RwLock When**:
```rust
// ✅ Configuration (read-heavy)
let config = RwLock::new(Config::load());
config.read().unwrap().get_setting("key");

// ✅ Caches (read-heavy)
let cache = RwLock::new(HashMap::new());
cache.read().unwrap().get(&key);

// ✅ Many readers, rare writers
let users = RwLock::new(UserDatabase::new());
```

---

## Advanced Patterns

### Double-Checked Locking

```rust
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

/// Lazy initialization with double-checked locking
struct LazyInit<T> {
    initialized: AtomicBool,
    data: Mutex<Option<T>>,
}

impl<T> LazyInit<T> {
    fn new() -> Self {
        LazyInit {
            initialized: AtomicBool::new(false),
            data: Mutex::new(None),
        }
    }

    fn get_or_init<F>(&self, init: F) -> &T
    where
        F: FnOnce() -> T,
    {
        // Fast path: already initialized
        if self.initialized.load(Ordering::Acquire) {
            unsafe {
                // SAFETY: initialized flag guarantees data is Some
                return (*self.data.lock().unwrap()).as_ref().unwrap();
            }
        }

        // Slow path: need to initialize
        let mut data = self.data.lock().unwrap();

        // Double-check (another thread might have initialized)
        if !self.initialized.load(Ordering::Acquire) {
            *data = Some(init());
            self.initialized.store(true, Ordering::Release);
        }

        data.as_ref().unwrap()
    }
}
```

### Lock-Free Queue (MPSC)

```rust
use std::sync::atomic::{AtomicPtr, Ordering};
use std::ptr;

/// Multi-Producer Single-Consumer lock-free queue
struct MpscQueue<T> {
    head: AtomicPtr<Node<T>>,
    tail: AtomicPtr<Node<T>>,
}

struct Node<T> {
    data: Option<T>,
    next: AtomicPtr<Node<T>>,
}

impl<T> MpscQueue<T> {
    fn new() -> Self {
        let dummy = Box::into_raw(Box::new(Node {
            data: None,
            next: AtomicPtr::new(ptr::null_mut()),
        }));

        MpscQueue {
            head: AtomicPtr::new(dummy),
            tail: AtomicPtr::new(dummy),
        }
    }

    /// Push from any thread (lock-free)
    fn push(&self, data: T) {
        let new_node = Box::into_raw(Box::new(Node {
            data: Some(data),
            next: AtomicPtr::new(ptr::null_mut()),
        }));

        let prev_head = self.head.swap(new_node, Ordering::AcqRel);
        unsafe {
            (*prev_head).next.store(new_node, Ordering::Release);
        }
    }

    /// Pop from single consumer thread
    fn pop(&self) -> Option<T> {
        let tail = self.tail.load(Ordering::Acquire);
        let next = unsafe { (*tail).next.load(Ordering::Acquire) };

        if next.is_null() {
            return None;
        }

        self.tail.store(next, Ordering::Release);
        unsafe {
            let data = (*next).data.take();
            // Free old tail (dummy node)
            let _ = Box::from_raw(tail);
            data
        }
    }
}
```

### Seqlock (Optimistic Concurrency)

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

/// Seqlock - optimistic read, exclusive write
struct Seqlock<T> {
    seq: AtomicUsize,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for Seqlock<T> {}
unsafe impl<T: Send> Sync for Seqlock<T> {}

impl<T: Copy> Seqlock<T> {
    fn new(data: T) -> Self {
        Seqlock {
            seq: AtomicUsize::new(0),
            data: UnsafeCell::new(data),
        }
    }

    /// Read (optimistic, lock-free)
    fn read(&self) -> T {
        loop {
            let seq1 = self.seq.load(Ordering::Acquire);

            // Odd sequence number means write in progress
            if seq1 & 1 != 0 {
                std::hint::spin_loop();
                continue;
            }

            // Read data
            let value = unsafe { *self.data.get() };

            // Verify no write occurred
            let seq2 = self.seq.load(Ordering::Acquire);
            if seq1 == seq2 {
                return value;
            }

            // Write occurred, retry
        }
    }

    /// Write (exclusive)
    fn write(&self, new_data: T) {
        // Increment sequence (make it odd)
        let seq = self.seq.fetch_add(1, Ordering::Acquire);
        assert_eq!(seq & 1, 0, "Concurrent writes detected!");

        // Write data
        unsafe {
            *self.data.get() = new_data;
        }

        // Increment sequence again (make it even)
        self.seq.fetch_add(1, Ordering::Release);
    }
}

use std::cell::UnsafeCell;

fn seqlock_example() {
    let seqlock = Arc::new(Seqlock::new((0, 0)));

    // Writer
    let s = Arc::clone(&seqlock);
    thread::spawn(move || {
        for i in 0..1000 {
            s.write((i, i * 2));
        }
    });

    // Readers (lock-free)
    let mut readers = vec![];
    for _ in 0..4 {
        let s = Arc::clone(&seqlock);
        let r = thread::spawn(move || {
            for _ in 0..1000 {
                let (a, b) = s.read();
                assert_eq!(a * 2, b, "Inconsistent read!");
            }
        });
        readers.push(r);
    }

    for r in readers {
        r.join().unwrap();
    }
}
```

---

## Summary

### Quick Reference

| Scenario | Primitive | Reason |
|----------|-----------|--------|
| Simple counter | `AtomicUsize` | Lock-free, fast |
| Boolean flag | `AtomicBool` | Lock-free, minimal overhead |
| Complex data | `Mutex<T>` | Protects arbitrary state |
| Config/Cache | `RwLock<T>` | Many readers, rare writes |
| Channel | `crossbeam::channel` | Producer-consumer pattern |
| Wait condition | `Mutex + Condvar` | Block until condition |
| Lock-free queue | Atomics + unsafe | Maximum throughput |

### Memory Ordering Guide

- **Relaxed**: Only atomicity, no ordering (use for independent counters)
- **Acquire/Release**: Synchronize with matching Release/Acquire (most common)
- **SeqCst**: Total order across all threads (use when in doubt, but slower)

### Best Practices

1. **Prefer higher-level abstractions** (channels) over raw locks
2. **Use atomics for simple primitives** (counters, flags)
3. **Use Mutex for complex state** that must be consistent
4. **Use RwLock for read-heavy workloads** (>80% reads)
5. **Always lock in the same order** to prevent deadlocks
6. **Keep critical sections short** - do work outside locks
7. **Benchmark** - don't assume, measure!

---

**End of Rust Concurrency Deep Dive**
