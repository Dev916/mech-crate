# Consistency Models: The Spectrum of Distributed Guarantees

**Purpose**: Formal specifications of correctness for concurrent and distributed systems, defining what behaviors are legal when multiple processes access shared state.

**Core Insight**: Stronger consistency = easier reasoning but lower performance. Trade consistency for availability (CAP), latency (PACELC), or scalability.

---

## Table of Contents

1. [Foundational Concepts](#foundational-concepts)
2. [Strong Consistency](#strong-consistency)
3. [Weak Consistency](#weak-consistency)
4. [Causal Consistency](#causal-consistency)
5. [Eventual Consistency](#eventual-consistency)
6. [Trade-offs and Theorems](#trade-offs-and-theorems)
7. [Stack-Specific Implementations](#stack-specific-implementations)
8. [Integration Points](#integration-points)

---

## Foundational Concepts

### What is a Consistency Model?

**Consistency Model**: Contract between system and programmer specifying:
- Which operation orderings are legal
- Which values reads can return
- Guarantees about visibility of writes

**Not the same as**:
- **ACID Consistency**: Application-level invariants
- **Consensus**: Agreement protocol for replicated state

### History-Based Definitions

**History**: Sequence of operations across all processes.

**Operation**: Tuple (process, operation_type, object, value, timestamp)
- Example: `(P1, write, x, 5, t1)`, `(P2, read, x, 5, t2)`

**Legal History**: History allowed by consistency model.

### Ordering Relations

**Program Order** (→_p):
- Operations by same process, in program order
- Example: If P1 writes x then reads y, `write(x) →_p read(y)`

**Happens-Before** (→):
- Transitive closure of program order + causal dependencies
- Foundation for causal consistency

**Real-Time Order**:
- If op1 completes before op2 starts (wall clock)
- Used in linearizability definition

### Replication and Consistency

**Why Replicate?**:
- **Fault Tolerance**: Survive failures
- **Scalability**: Distribute load
- **Latency**: Serve from nearby replica

**Replication Challenge**: Keep replicas consistent while:
- Handling concurrent updates
- Tolerating network delays/partitions
- Maintaining performance

---

## Strong Consistency

### Linearizability (Atomic Consistency)

**Strongest consistency model** for concurrent objects.

**Definition** (Herlihy & Wing, 1990):
System is linearizable if:
1. **Linearization Points**: Each operation appears to execute instantaneously at some point between invocation and response
2. **Real-Time Ordering**: If op1 completes before op2 starts, op1's linearization point precedes op2's
3. **Sequential Specification**: Linearized history respects object's sequential specification

**Intuition**: System behaves like single copy with atomic operations.

**Example (Linearizable)**:
```
P1: write(x, 1) ---------------|
P2:              |----- read(x) returns 1 -------|
P3:                          |---- read(x) returns 1 ---|

Linearization: write(x,1) → read(x)_P2 → read(x)_P3
```

**Example (Not Linearizable)**:
```
P1: write(x, 1) -----|
P2:              |---- read(x) returns 0 --|
P3:                        |--- read(x) returns 1 ---|

P2's read overlaps write but returns old value,
while P3's later read sees new value.
Not linearizable: no consistent ordering.
```

**Properties**:
- **Composable**: Linearizable objects compose into linearizable system
- **Non-Blocking**: Some operation can always complete (with infinite threads)
- **Local**: Can verify per-object independently

**Implementation**: Consensus-based replication (Paxos, Raft), single leader.

### Sequential Consistency

**Weaker than linearizability**: Relaxes real-time constraint.

**Definition** (Lamport, 1979):
System is sequentially consistent if:
1. **Program Order**: Operations of each process appear in program order
2. **Global Order**: All processes see operations in same total order
3. No requirement to respect real-time order between processes

**Intuition**: There exists some sequential interleaving consistent with each process's program order.

**Example (Sequentially Consistent but not Linearizable)**:
```
P1: write(x, 1) ---|
P2: write(y, 1) ---|
P3:                 |--- read(y) returns 1, read(x) returns 0 ---|
P4:                 |--- read(x) returns 1, read(y) returns 0 ---|

Sequential order: write(y,1) → write(x,1) (for P3's view)
                  write(x,1) → write(y,1) (for P4's view)
Contradiction! Not sequentially consistent.

But if:
P3: read(y)=1, read(x)=1
P4: read(x)=1, read(y)=1
Then sequential: write(x,1) → write(y,1) → reads
```

**Properties**:
- Not composable (unlike linearizability)
- Easier to implement than linearizability (no real-time constraint)

**Implementation**: Total Store Order (TSO), some cache coherence protocols.

### Strict Serializability

**Combines serializability (transactions) + linearizability (real-time).**

**Definition**:
Transactions are:
1. **Serializable**: Equivalent to some serial execution
2. **Linearizable**: Respect real-time order

**Used In**: Spanner (Google), FaunaDB (external consistency).

---

## Weak Consistency

### PRAM (Pipelined RAM)

**Guarantee**: Writes by single process seen in order by all others. No ordering between different processes' writes.

**Definition**:
- Writes from same process must be seen in order
- Writes from different processes can be seen in any order

**Example**:
```
P1: write(x, 1), write(x, 2)
P2: read(x) = 1, read(x) = 2   ✓ (ordered)
P3: read(x) = 2, read(x) = 1   ✗ (violates PRAM)

P1: write(x, 1)
P2: write(y, 1)
P3: read(x)=0, read(y)=1
P4: read(y)=0, read(x)=1       ✓ (different processes, any order OK)
```

**Use Case**: Processor memory models (see Memory Models appendix).

### Processor Consistency

**Stronger than PRAM**: Adds constraint that all processes agree on order of writes to same location.

**Definition**:
- PRAM + writes to same location are totally ordered
- Different locations can be reordered

**Implementation**: Many multiprocessor cache coherence protocols.

### Monotonic Reads

**Guarantee**: If process reads value v, subsequent reads won't return older values.

**Formally**: If `read(x) = v_i` precedes `read(x)` in process's program order, second read returns `v_j` where `v_j` at least as recent as `v_i`.

**Example**:
```
P1: read(x) = 5  (version 5)
P1: read(x) = 7  ✓ (version 7 ≥ 5)
P1: read(x) = 3  ✗ (violates monotonic reads)
```

**Use Case**: Session consistency in distributed databases.

### Monotonic Writes

**Guarantee**: Writes by same process executed in order.

**Formally**: If `write(x, v1)` precedes `write(x, v2)` in process's program order, all replicas apply writes in that order.

**Example**:
```
P1: write(x, 1)
P1: write(x, 2)

All replicas eventually have: x = 2
No replica sees: x = 2 then x = 1
```

**Use Case**: Social media feeds (posts appear in order user created them).

### Read Your Writes

**Guarantee**: Process's reads reflect its prior writes.

**Formally**: If process writes x, subsequent reads by that process return written value (or newer).

**Example**:
```
P1: write(x, 5)
P1: read(x) = 5  ✓ (sees own write)
P1: read(x) = 3  ✗ (doesn't see own write)
```

**Implementation**: Session stickiness (route to same replica), write-through cache.

### Writes Follow Reads

**Guarantee**: Writes after read are ordered after values read.

**Formally**: If process reads `x = v1` then writes `y = v2`, any process reading `y = v2` will see `x ≥ v1`.

**Use Case**: Comment threads (comment sees post it replies to).

---

## Causal Consistency

**Key Insight**: Preserve causally related operations, allow concurrent operations to be seen in different orders.

### Definition

**Causal Consistency**:
- Operations causally related must be seen in same order by all processes
- Concurrent operations can be seen in different orders

**Causality** (→):
- **Program Order**: op1 →_p op2 in same process → op1 → op2
- **Reads-From**: write(x, v) → read(x, v)
- **Transitivity**: op1 → op2 and op2 → op3 → op1 → op3

**Concurrent**: op1 || op2 if neither op1 → op2 nor op2 → op1.

### Examples

**Causal**:
```
P1: write(x, 1)
P2: read(x) = 1, write(y, 2)  // causally depends on write(x,1)
P3: read(y) = 2, read(x) = 1  ✓ (sees cause before effect)
P4: read(y) = 2, read(x) = 0  ✗ (sees effect without cause)
```

**Concurrent Writes (OK in Causal)**:
```
P1: write(x, 1)
P2: write(x, 2)  // concurrent with P1's write
P3: read(x) = 1, read(x) = 2
P4: read(x) = 2, read(x) = 1  // different order OK (concurrent)
```

### Vector Clocks

**Mechanism**: Track causality using vector clocks.

**Vector Clock**: Array V where V[i] = number of events process i has seen.

**Rules**:
1. Local event: V[i]++
2. Send message: Include V in message
3. Receive message with V_m: V[i] = max(V[i], V_m[i]) for all i, then V[i]++

**Comparison**:
- V1 < V2 if ∀i: V1[i] ≤ V2[i] and ∃j: V1[j] < V2[j] (V1 causally precedes V2)
- V1 || V2 if neither V1 < V2 nor V2 < V1 (concurrent)

**Example**:
```
P1: [1,0,0] write(x,1)
P2: [0,1,0] → receive from P1 → [1,2,0] write(y,2)
P3: [0,0,1] → receive from P2 → [1,2,2] read(y)=2

P3 knows write(y,2) causally depends on write(x,1)
Must see write(x,1) before applying write(y,2)
```

### Causal+ Consistency

**Extension**: Causal consistency + convergence for concurrent writes.

**Combines**:
- Causal consistency (ordering)
- CRDTs or conflict resolution for concurrent updates

**Example Systems**: COPS, Eiger, ChainReaction.

---

## Eventual Consistency

**Weakest useful model**: If no new updates, all replicas eventually converge.

### Definition

**Eventual Consistency**:
1. **Eventual Delivery**: Update delivered to all replicas eventually
2. **Convergence**: Replicas with same updates have same state
3. **Termination**: All methods terminate

**No guarantees about**:
- How long until convergence
- Intermediate states
- Order of updates seen

### Examples

**Allowed**:
```
P1: write(x, 1)
P2: read(x) = 0  (stale, but eventually...)
P2: read(x) = 1  (converged)
```

**Anomalies** (allowed temporarily):
```
P1: write(x, 1), write(y, 1)
P2: read(y) = 1, read(x) = 0  // sees y before x (causal violation)
```

### Strong Eventual Consistency

**Stronger variant**: CRDTs provide strong eventual consistency.

**Guarantees**:
1. **Eventual Delivery**: Same as eventual consistency
2. **Strong Convergence**: Replicas with same updates have same state (deterministic merge)
3. **No coordination**: Updates applied locally without synchronization

**Conflict-Free Replicated Data Types (CRDTs)**:
- **State-based** (CvRDTs): Merge states, must be semilattice
- **Operation-based** (CmRDTs): Commutative operations

**Examples**:
- **G-Counter**: Grow-only counter (increment only)
- **PN-Counter**: Positive-negative counter (increment/decrement)
- **LWW-Register**: Last-writer-wins register (timestamp-based)
- **OR-Set**: Observed-remove set (add/remove elements)

### CRDT Example: G-Counter

```
State: Map[ProcessID, Int]

increment(id):
  state[id]++

value():
  return sum(state.values)

merge(other):
  for each id:
    state[id] = max(state[id], other.state[id])
```

**Merge is**:
- Commutative: merge(A, B) = merge(B, A)
- Associative: merge(merge(A, B), C) = merge(A, merge(B, C))
- Idempotent: merge(A, A) = A

**Result**: Strong eventual consistency - all replicas converge to same count.

---

## Trade-offs and Theorems

### CAP Theorem

**Theorem** (Brewer, 2000; Gilbert & Lynch, 2002):

"A distributed system can provide at most 2 of 3 guarantees:
- **Consistency** (C): Linearizability
- **Availability** (A): Every request receives response
- **Partition Tolerance** (P): System continues despite network partitions"

**In Practice**: Networks partition → must choose C or A.

**CP Systems** (Sacrifice Availability):
- HBase, MongoDB (with majority writes)
- Consensus-based: Paxos, Raft

**AP Systems** (Sacrifice Consistency):
- Cassandra (tunable), DynamoDB
- Eventual consistency, CRDTs

**CA Systems** (Mythical):
- Only work in single-node or perfect network (unrealistic)

### PACELC

**Extension of CAP** (Abadi, 2012):

"If **P**artition, trade **A**vailability vs **C**onsistency;
**E**lse (no partition), trade **L**atency vs **C**onsistency."

**PA/EL**: Prioritize availability and latency (Cassandra, DynamoDB)
**PC/EC**: Prioritize consistency (HBase, VoltDB)
**PA/EC**: Prioritize availability during partition, consistency otherwise
**PC/EL**: Prioritize consistency during partition, latency otherwise

### Harvest and Yield

**Alternative to CAP** (Fox & Brewer, 1999):

- **Yield**: Probability of completing request
- **Harvest**: Fraction of data reflected in response

**Trade-off**: Can sacrifice harvest (incomplete data) to maintain yield (availability).

**Example**: Search engine returns partial results (reduced harvest) but always responds (high yield).

---

## Stack-Specific Implementations

### Rust: Causally Consistent Store

```rust
use std::collections::HashMap;
use std::cmp::Ordering;

/// Vector clock for causality tracking
#[derive(Clone, Debug, PartialEq, Eq)]
struct VectorClock {
    clocks: HashMap<String, u64>,
}

impl VectorClock {
    fn new() -> Self {
        VectorClock {
            clocks: HashMap::new(),
        }
    }

    fn increment(&mut self, process_id: &str) {
        *self.clocks.entry(process_id.to_string()).or_insert(0) += 1;
    }

    fn merge(&mut self, other: &VectorClock) {
        for (proc, &time) in &other.clocks {
            let entry = self.clocks.entry(proc.clone()).or_insert(0);
            *entry = (*entry).max(time);
        }
    }

    fn happens_before(&self, other: &VectorClock) -> bool {
        let mut less_than = false;
        let mut less_equal = true;

        for (proc, &time) in &self.clocks {
            let other_time = other.clocks.get(proc).copied().unwrap_or(0);
            if time > other_time {
                less_equal = false;
            } else if time < other_time {
                less_than = true;
            }
        }

        for (proc, _) in &other.clocks {
            if !self.clocks.contains_key(proc) {
                less_than = true;
            }
        }

        less_than && less_equal
    }

    fn concurrent(&self, other: &VectorClock) -> bool {
        !self.happens_before(other) && !other.happens_before(self)
    }
}

/// Versioned value with vector clock
#[derive(Clone, Debug)]
struct VersionedValue<T> {
    value: T,
    version: VectorClock,
}

/// Causally consistent key-value store
struct CausalStore<T: Clone> {
    process_id: String,
    data: HashMap<String, Vec<VersionedValue<T>>>,
    clock: VectorClock,
}

impl<T: Clone> CausalStore<T> {
    fn new(process_id: String) -> Self {
        CausalStore {
            process_id,
            data: HashMap::new(),
            clock: VectorClock::new(),
        }
    }

    fn write(&mut self, key: String, value: T) {
        // Increment local clock
        self.clock.increment(&self.process_id);

        let versioned = VersionedValue {
            value,
            version: self.clock.clone(),
        };

        // Add to version history
        self.data.entry(key).or_insert_with(Vec::new).push(versioned);
    }

    fn read(&mut self, key: &str) -> Option<Vec<T>> {
        // Return all concurrent versions
        let versions = self.data.get(key)?;

        // Filter to maximal elements (not dominated by others)
        let mut maximal = Vec::new();

        for v1 in versions {
            let mut is_maximal = true;
            for v2 in versions {
                if v1.version.happens_before(&v2.version) {
                    is_maximal = false;
                    break;
                }
            }
            if is_maximal {
                maximal.push(v1.value.clone());
            }
        }

        Some(maximal)
    }

    fn merge(&mut self, other: &CausalStore<T>) {
        // Merge vector clocks
        self.clock.merge(&other.clock);

        // Merge data
        for (key, other_versions) in &other.data {
            let local_versions = self.data.entry(key.clone()).or_insert_with(Vec::new);

            for other_version in other_versions {
                // Add if not dominated by existing version
                let mut should_add = true;
                for local_version in local_versions.iter() {
                    if other_version.version.happens_before(&local_version.version) {
                        should_add = false;
                        break;
                    }
                }

                if should_add {
                    local_versions.push(other_version.clone());
                }
            }
        }
    }
}

/// CRDT: G-Counter (grow-only counter)
#[derive(Clone, Debug)]
struct GCounter {
    counts: HashMap<String, u64>,
}

impl GCounter {
    fn new() -> Self {
        GCounter {
            counts: HashMap::new(),
        }
    }

    fn increment(&mut self, process_id: String) {
        *self.counts.entry(process_id).or_insert(0) += 1;
    }

    fn value(&self) -> u64 {
        self.counts.values().sum()
    }

    fn merge(&mut self, other: &GCounter) {
        for (proc, &count) in &other.counts {
            let entry = self.counts.entry(proc.clone()).or_insert(0);
            *entry = (*entry).max(count);
        }
    }
}

/// CRDT: PN-Counter (increment/decrement)
#[derive(Clone, Debug)]
struct PNCounter {
    increments: GCounter,
    decrements: GCounter,
}

impl PNCounter {
    fn new() -> Self {
        PNCounter {
            increments: GCounter::new(),
            decrements: GCounter::new(),
        }
    }

    fn increment(&mut self, process_id: String) {
        self.increments.increment(process_id);
    }

    fn decrement(&mut self, process_id: String) {
        self.decrements.increment(process_id);
    }

    fn value(&self) -> i64 {
        self.increments.value() as i64 - self.decrements.value() as i64
    }

    fn merge(&mut self, other: &PNCounter) {
        self.increments.merge(&other.increments);
        self.decrements.merge(&other.decrements);
    }
}
```

### TypeScript: Eventually Consistent Store with CRDTs

```typescript
/**
 * Last-Writer-Wins Register CRDT
 */
class LWWRegister<T> {
  private value: T | null = null;
  private timestamp: number = 0;
  private node_id: string;

  constructor(nodeId: string) {
    this.node_id = nodeId;
  }

  set(value: T): void {
    const now = Date.now();
    // Handle clock skew with node ID tiebreaker
    if (now > this.timestamp) {
      this.value = value;
      this.timestamp = now;
    }
  }

  get(): T | null {
    return this.value;
  }

  merge(other: LWWRegister<T>): void {
    if (
      other.timestamp > this.timestamp ||
      (other.timestamp === this.timestamp &&
        other.node_id > this.node_id)
    ) {
      this.value = other.value;
      this.timestamp = other.timestamp;
    }
  }

  getState(): { value: T | null; timestamp: number; nodeId: string } {
    return {
      value: this.value,
      timestamp: this.timestamp,
      nodeId: this.node_id,
    };
  }
}

/**
 * Observed-Remove Set CRDT
 */
class ORSet<T> {
  private elements = new Map<T, Set<string>>();
  private nodeId: string;
  private counter = 0;

  constructor(nodeId: string) {
    this.nodeId = nodeId;
  }

  add(element: T): void {
    const tag = `${this.nodeId}:${this.counter++}`;
    if (!this.elements.has(element)) {
      this.elements.set(element, new Set());
    }
    this.elements.get(element)!.add(tag);
  }

  remove(element: T): void {
    // Remove element by clearing its tags
    this.elements.delete(element);
  }

  has(element: T): boolean {
    const tags = this.elements.get(element);
    return tags !== undefined && tags.size > 0;
  }

  values(): T[] {
    return Array.from(this.elements.keys()).filter(
      elem => this.elements.get(elem)!.size > 0
    );
  }

  merge(other: ORSet<T>): void {
    // Union of all element-tag pairs
    for (const [element, otherTags] of other.elements.entries()) {
      if (!this.elements.has(element)) {
        this.elements.set(element, new Set());
      }
      const localTags = this.elements.get(element)!;
      for (const tag of otherTags) {
        localTags.add(tag);
      }
    }
  }
}

/**
 * Vector clock implementation
 */
class VectorClock {
  private clocks = new Map<string, number>();

  increment(processId: string): void {
    const current = this.clocks.get(processId) || 0;
    this.clocks.set(processId, current + 1);
  }

  merge(other: VectorClock): void {
    for (const [proc, time] of other.clocks.entries()) {
      const localTime = this.clocks.get(proc) || 0;
      this.clocks.set(proc, Math.max(localTime, time));
    }
  }

  happensBefore(other: VectorClock): boolean {
    let strictlyLess = false;
    let allLessOrEqual = true;

    const allProcs = new Set([
      ...this.clocks.keys(),
      ...other.clocks.keys(),
    ]);

    for (const proc of allProcs) {
      const thisTime = this.clocks.get(proc) || 0;
      const otherTime = other.clocks.get(proc) || 0;

      if (thisTime > otherTime) {
        allLessOrEqual = false;
      } else if (thisTime < otherTime) {
        strictlyLess = true;
      }
    }

    return strictlyLess && allLessOrEqual;
  }

  concurrent(other: VectorClock): boolean {
    return (
      !this.happensBefore(other) &&
      !other.happensBefore(this)
    );
  }

  clone(): VectorClock {
    const cloned = new VectorClock();
    cloned.clocks = new Map(this.clocks);
    return cloned;
  }
}

/**
 * Eventually consistent key-value store
 */
class EventualStore<T> {
  private data = new Map<string, LWWRegister<T>>();
  private nodeId: string;

  constructor(nodeId: string) {
    this.nodeId = nodeId;
  }

  set(key: string, value: T): void {
    if (!this.data.has(key)) {
      this.data.set(key, new LWWRegister<T>(this.nodeId));
    }
    this.data.get(key)!.set(value);
  }

  get(key: string): T | null {
    const register = this.data.get(key);
    return register ? register.get() : null;
  }

  merge(other: EventualStore<T>): void {
    for (const [key, otherRegister] of other.data.entries()) {
      if (!this.data.has(key)) {
        this.data.set(key, new LWWRegister<T>(this.nodeId));
      }
      this.data.get(key)!.merge(otherRegister);
    }
  }
}
```

### PHP: Session Consistency Implementation

```php
<?php

namespace Consistency;

use Illuminate\Support\Facades\Redis;
use Illuminate\Support\Facades\Cache;

/**
 * Session consistency: Read-your-writes guarantee
 */
class SessionConsistentStore
{
    private string $sessionId;
    private int $version = 0;

    public function __construct(string $sessionId)
    {
        $this->sessionId = $sessionId;
        $this->loadVersion();
    }

    /**
     * Write with version tracking
     */
    public function write(string $key, mixed $value): void
    {
        $this->version++;

        $data = [
            'value' => $value,
            'version' => $this->version,
            'timestamp' => microtime(true),
        ];

        // Write to all replicas
        foreach ($this->getReplicas() as $replica) {
            $this->writeToReplica($replica, $key, $data);
        }

        // Store session version
        $this->saveVersion();
    }

    /**
     * Read with version checking (read-your-writes)
     */
    public function read(string $key): mixed
    {
        $replicas = $this->getReplicas();

        foreach ($replicas as $replica) {
            $data = $this->readFromReplica($replica, $key);

            if ($data && $data['version'] >= $this->version) {
                // Replica has version at least as recent as session
                return $data['value'];
            }
        }

        // Fallback: read from any replica (eventual consistency)
        foreach ($replicas as $replica) {
            $data = $this->readFromReplica($replica, $key);
            if ($data) {
                return $data['value'];
            }
        }

        return null;
    }

    private function getReplicas(): array
    {
        return config('database.replicas', ['localhost']);
    }

    private function writeToReplica(
        string $replica,
        string $key,
        array $data
    ): void {
        Redis::connection($replica)->hset(
            "data:{$key}",
            'value',
            serialize($data)
        );
    }

    private function readFromReplica(
        string $replica,
        string $key
    ): ?array {
        $raw = Redis::connection($replica)->hget("data:{$key}", 'value');

        return $raw ? unserialize($raw) : null;
    }

    private function loadVersion(): void
    {
        $this->version = (int) Cache::get(
            "session_version:{$this->sessionId}",
            0
        );
    }

    private function saveVersion(): void
    {
        Cache::put(
            "session_version:{$this->sessionId}",
            $this->version,
            now()->addHours(24)
        );
    }
}

/**
 * CRDT: LWW-Element-Set
 */
class LWWSet
{
    private array $added = [];
    private array $removed = [];

    public function add(mixed $element): void
    {
        $timestamp = microtime(true);
        $this->added[$element] = $timestamp;

        // Remove from removed set if present
        unset($this->removed[$element]);
    }

    public function remove(mixed $element): void
    {
        $timestamp = microtime(true);
        $this->removed[$element] = $timestamp;
    }

    public function contains(mixed $element): bool
    {
        $addedTime = $this->added[$element] ?? 0;
        $removedTime = $this->removed[$element] ?? 0;

        // Element present if added more recently than removed
        return $addedTime > $removedTime;
    }

    public function elements(): array
    {
        return array_filter(
            array_keys($this->added),
            fn($elem) => $this->contains($elem)
        );
    }

    public function merge(LWWSet $other): void
    {
        // Merge added timestamps (keep maximum)
        foreach ($other->added as $elem => $time) {
            if (!isset($this->added[$elem]) || $time > $this->added[$elem]) {
                $this->added[$elem] = $time;
            }
        }

        // Merge removed timestamps (keep maximum)
        foreach ($other->removed as $elem => $time) {
            if (!isset($this->removed[$elem]) || $time > $this->removed[$elem]) {
                $this->removed[$elem] = $time;
            }
        }
    }
}

/**
 * Quorum-based consistency (tunable)
 */
class QuorumStore
{
    private int $replicationFactor;
    private int $readQuorum;
    private int $writeQuorum;

    public function __construct(
        int $replicationFactor = 3,
        int $readQuorum = 2,
        int $writeQuorum = 2
    ) {
        $this->replicationFactor = $replicationFactor;
        $this->readQuorum = $readQuorum;
        $this->writeQuorum = $writeQuorum;

        // Ensure R + W > N for strong consistency
        assert($readQuorum + $writeQuorum > $replicationFactor);
    }

    public function write(string $key, mixed $value): bool
    {
        $version = time();
        $replicas = $this->selectReplicas($key);
        $successes = 0;

        foreach ($replicas as $replica) {
            if ($this->writeToReplica($replica, $key, $value, $version)) {
                $successes++;
            }

            if ($successes >= $this->writeQuorum) {
                return true;
            }
        }

        return false; // Failed to reach write quorum
    }

    public function read(string $key): mixed
    {
        $replicas = $this->selectReplicas($key);
        $responses = [];

        foreach ($replicas as $replica) {
            $data = $this->readFromReplica($replica, $key);
            if ($data !== null) {
                $responses[] = $data;
            }

            if (count($responses) >= $this->readQuorum) {
                break;
            }
        }

        if (count($responses) < $this->readQuorum) {
            throw new \Exception('Failed to reach read quorum');
        }

        // Return value with highest version
        usort($responses, fn($a, $b) => $b['version'] <=> $a['version']);

        return $responses[0]['value'];
    }

    private function selectReplicas(string $key): array
    {
        // Consistent hashing to select replicas
        // Simplified: just return configured replicas
        return config('database.replicas', []);
    }

    private function writeToReplica(
        string $replica,
        string $key,
        mixed $value,
        int $version
    ): bool {
        try {
            Redis::connection($replica)->hmset("data:{$key}", [
                'value' => serialize($value),
                'version' => $version,
            ]);
            return true;
        } catch (\Exception $e) {
            return false;
        }
    }

    private function readFromReplica(string $replica, string $key): ?array
    {
        try {
            $data = Redis::connection($replica)->hgetall("data:{$key}");
            if (empty($data)) {
                return null;
            }

            return [
                'value' => unserialize($data['value']),
                'version' => (int) $data['version'],
            ];
        } catch (\Exception $e) {
            return null;
        }
    }
}
```

---

## Integration Points

### With Consensus
- **Consensus provides linearizability**: Replicated state machine → strongest consistency
- **Trade-off**: Consensus sacrifices availability during partitions

### With Actor Model
- **Eventual consistency**: Natural for actor systems (async messaging)
- **Causal consistency**: Preserve message causality in distributed actors

### With Distributed Systems
- **Replication strategy**: Consistency model determines replication protocol
- **CAP trade-offs**: Choose consistency level based on application needs

---

## Further Reading

### Papers
- Lamport (1979) - "How to Make a Multiprocessor Computer That Correctly Executes Multiprocess Programs"
- Herlihy & Wing (1990) - "Linearizability: A Correctness Condition for Concurrent Objects"
- Gilbert & Lynch (2002) - "Brewer's Conjecture and the Feasibility of Consistent, Available, Partition-Tolerant Web Services"
- Shapiro et al. (2011) - "Conflict-Free Replicated Data Types"

### Books
- Kleppmann - "Designing Data-Intensive Applications"
- Tanenbaum & Van Steen - "Distributed Systems: Principles and Paradigms"

---

**End of Consistency Models Appendix**
