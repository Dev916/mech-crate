# Coordination Models: Decoupled Concurrent System Composition

**Purpose**: Abstract coordination mechanisms that decouple computation from communication, enabling flexible concurrent system design through shared spaces and constraint-based synchronization.

**Core Insight**: Separate **what** entities compute from **how** they coordinate. Use declarative coordination media (tuple spaces, Petri nets) instead of explicit message passing.

---

## Table of Contents

1. [Foundational Concepts](#foundational-concepts)
2. [Linda and Tuple Spaces](#linda-and-tuple-spaces)
3. [Petri Nets](#petri-nets)
4. [Workflow Nets](#workflow-nets)
5. [Coordination Languages](#coordination-languages)
6. [Practical Applications](#practical-applications)
7. [Stack-Specific Implementations](#stack-specific-implementations)
8. [Integration Points](#integration-points)

---

## Foundational Concepts

### What Are Coordination Models?

Coordination models provide abstractions for managing interactions among concurrent entities **without** hardcoding communication patterns into computational components.

**Key Principle**: **Separation of Concerns**
- **Computation**: What each entity does (business logic)
- **Coordination**: How entities interact (communication patterns)

**Benefits**:
- **Modularity**: Components don't know about each other
- **Flexibility**: Change coordination without changing computation
- **Reasoning**: Analyze coordination independently of computation
- **Reusability**: Same components, different coordination strategies

### Coordination vs. Communication

| Aspect | Communication | Coordination |
|--------|---------------|--------------|
| Coupling | Tight (sender knows receiver) | Loose (via shared medium) |
| Abstraction | Low-level (send/receive) | High-level (patterns) |
| Synchronization | Explicit (channels, locks) | Implicit (constraints) |
| Composition | Hard to change | Declarative, flexible |

**Example**:
- **Communication**: `actor.send(message)` - explicit recipient
- **Coordination**: `space.out(task)` - anonymous, anyone can take it

### Coordination Media

Abstract spaces that mediate interactions:

1. **Tuple Spaces** (Linda): Associative memory for tuples
2. **Petri Nets**: Token-based constraint models
3. **Event Spaces**: Publish-subscribe event buses
4. **Constraint Systems**: Rule-based coordination
5. **Workflow Engines**: Process orchestration

---

## Linda and Tuple Spaces

**Developed by**: David Gelernter (1985) at Yale

**Key Idea**: **Generative communication** - processes communicate by creating, reading, and consuming tuples in a shared tuple space.

### Linda Operations

Four primitive operations on tuple space:

```
out(tuple)          -- Insert tuple into space (non-blocking)
in(template)        -- Remove matching tuple (blocking until match)
rd(template)        -- Read matching tuple without removal (blocking)
eval(tuple)         -- Create active tuple (spawns process)
```

**Tuple**: Ordered sequence of values, e.g., `("task", 42, "pending")`

**Template**: Pattern with actuals and formals, e.g., `("task", ?id, "pending")` matches any task with pending status.

**Template Matching**: Tuple matches template if:
- Same arity (number of fields)
- Actuals in template equal corresponding tuple fields
- Formals (variables) bind to corresponding tuple fields

### Example: Producer-Consumer

```
-- Producer
out("job", 1, "data1")
out("job", 2, "data2")

-- Consumer
in("job", ?id, ?data)  -- Blocks until tuple available
processJob(id, data)
in("job", ?id, ?data)  -- Get next job
processJob(id, data)
```

**Decoupling**: Producer and consumer don't know about each other. Tuple space mediates.

### Example: Distributed Map-Reduce

```
-- Master: distribute tasks
for i in 1..N:
  out("map-task", i, data[i])

-- Workers: process tasks
loop:
  in("map-task", ?id, ?data)
  result = map(data)
  out("reduce-task", id, result)

-- Reducer: collect results
for i in 1..N:
  in("reduce-task", i, ?result)
  accumulate(result)
```

**Fault Tolerance**: If worker dies, task tuple remains in space for another worker.

### Linda Semantics

**Atomicity**: Operations are atomic. No race conditions in matching.

**Blocking Semantics**:
- `in` and `rd` block until matching tuple exists
- `out` never blocks (space is unbounded)

**Associative Access**: Tuples retrieved by content, not address.

**Space-Uncoupling**: Entities don't need to know each other's names.

**Time-Uncoupling**: Tuples persist after creator terminates.

### Extensions and Variants

**JavaSpaces** (Jini/Apache River):
- Distributed tuple space for Java
- Adds lease-based tuple expiration
- Transactions for atomic multi-tuple operations

**TSpaces** (IBM):
- Persistent tuple spaces
- SQL-like queries for tuple retrieval
- Integration with message queuing

**LIME** (Linda in Mobile Environments):
- Tuple spaces for mobile computing
- Federated tuple spaces that merge/split as devices move

**Distributed Data Structures**:
```
-- Shared counter
out("counter", 0)

-- Increment atomically
in("counter", ?val)
out("counter", val + 1)

-- Barrier synchronization
-- N processes
out("barrier", 0)
in("barrier", ?count)
if count + 1 == N:
  out("release")
else:
  out("barrier", count + 1)
in("release")  -- All wait here
```

---

## Petri Nets

**Developed by**: Carl Adam Petri (1962)

**Key Idea**: Model concurrency as a bipartite graph of **places** and **transitions** with **tokens** flowing through the net.

### Petri Net Structure

**Components**:
- **Places** (circles): Represent states or conditions
- **Transitions** (bars): Represent events or actions
- **Arcs**: Connect places to transitions or transitions to places
- **Tokens** (dots): Reside in places, represent resources or state

**Notation**:
```
P = {p₁, p₂, ..., pₙ}     -- Places
T = {t₁, t₂, ..., tₘ}     -- Transitions
F ⊆ (P × T) ∪ (T × P)     -- Flow relation (arcs)
M₀: P → ℕ                 -- Initial marking (token distribution)
```

**Marking**: Assignment of tokens to places, represents system state.

### Firing Rules

Transition t is **enabled** in marking M if:
```
∀p ∈ •t: M(p) ≥ 1
```
Where •t = set of input places to t.

**Firing** enabled transition t produces new marking M':
```
∀p ∈ P:
  M'(p) = M(p) - 1  if p ∈ •t \ t•
          M(p) + 1  if p ∈ t• \ •t
          M(p)      otherwise
```

Where t• = set of output places from t.

### Example: Producer-Consumer

```
        [buffer]
         ○ ○ ○  (3 tokens = 3 slots)
        ↗     ↘
[produce]    [consume]
       |         |
      (P)       (C)

P enabled if buffer not full → adds item (token to buffer)
C enabled if buffer not empty → removes item (token from buffer)
```

**Concurrency**: Multiple transitions can fire simultaneously if enabled and don't conflict (share places).

### Petri Net Properties

**Behavioral Properties**:

1. **Reachability**: Is marking M reachable from M₀?
2. **Boundedness**: Is number of tokens in any place bounded?
3. **Safety**: Is each place bounded by 1 token? (binary net)
4. **Liveness**: Can every transition eventually fire?
5. **Deadlock-Freedom**: Is there always an enabled transition?

**Structural Properties**:

1. **Conservativeness**: Total number of tokens constant
2. **Consistency**: Every transition fires same number of times in cycle
3. **Free-Choice**: After choice, no conflict

**Analysis Techniques**:
- **Reachability Graph**: Enumerate all reachable markings (state space explosion)
- **Invariant Analysis**: Find place/transition invariants (linear algebra)
- **Reduction Rules**: Simplify net preserving properties
- **Model Checking**: Verify temporal logic properties (CTL, LTL)

### Example: Dining Philosophers

```
    [F1]        [F5]
      ○          ○
    ↗   ↘      ↗   ↘
[eat1]   [think1]   [eat5]
    ↖   ↗      ↖   ↗
      ○          ○
    [F2]        [F4]
      ○
    ↗   ↘
[eat2]   [think2]
    ↖   ↗
      ○
    [F3]

Initially: 1 token in each fork place, 1 token in each thinking place

eat_i enabled if both fork_i and fork_{i+1} have tokens
Firing eat_i consumes forks, philosopher eats
Completing eating returns forks, goes to thinking
```

**Deadlock**: If all philosophers grab left fork simultaneously. Model checking reveals this.

**Solution**: Add token limit ensuring at most 4 philosophers can pick up forks.

### Extended Petri Nets

**Colored Petri Nets (CPN)**:
- Tokens have data values (colors)
- Transitions have guards and data transformations
- More expressive, avoid state explosion

**Timed Petri Nets**:
- Transitions or places have time delays
- Model real-time systems

**Hierarchical Petri Nets**:
- Transitions can be subnets (refinement)
- Manage complexity through abstraction

---

## Workflow Nets

**Workflow Nets** (WF-nets): Subclass of Petri nets for modeling business processes.

### WF-Net Structure

**Requirements**:
1. **Source place** (i): No incoming arcs, represents workflow start
2. **Sink place** (o): No outgoing arcs, represents workflow end
3. **Connectivity**: Every place/transition on path from i to o

**Soundness**: WF-net is **sound** if:
1. **Option to complete**: From any reachable state, can reach state with token only in o
2. **Proper completion**: When token in o, no tokens elsewhere
3. **No dead transitions**: Every transition can fire in some execution

### Workflow Patterns

**Sequential Routing**: Transitions fire in sequence
```
[Start] → (T1) → [P1] → (T2) → [P2] → (T3) → [End]
```

**Parallel Split (AND-split)**:
```
        → [P1] → (T1) →
[P] → (split)            (join) → [End]
        → [P2] → (T2) →
```

**Exclusive Choice (XOR-split)**:
```
        → [P1] → (T1) →
[P] → ( ? )              [End]
        → [P2] → (T2) →
```

**Synchronization (AND-join)**:
```
[P1] →
       (join) → [End]
[P2] →
```

**Merge (XOR-join)**:
```
[P1] →
       → [End]
[P2] →
```

### Example: Order Fulfillment Workflow

```
[Order] → (Validate) → [Valid] → (AND-split) →
                                    ↓
                        [Check Stock]  [Authorize Payment]
                                    ↓
                              (AND-join) → [Ready] →
                              (Pack) → [Packed] →
                              (Ship) → [Shipped] → [End]

If validation fails:
[Order] → (Validate) → [Invalid] → (Cancel) → [End]
```

**Soundness Check**: Verify using reachability analysis or reduction rules.

### Process Mining

**Given**: Event logs from workflow execution

**Goal**: Discover Petri net model that fits logs

**Techniques**:
- **α-algorithm**: Infer causality from event ordering
- **Heuristic mining**: Handle noise and incomplete logs
- **Conformance checking**: Verify model matches reality

**Applications**:
- Business process optimization
- Compliance verification
- Bottleneck identification

---

## Coordination Languages

### Reo: Channel-Based Coordination

**Reo** (developed by Farhad Arbab): Coordination language based on composable channels.

**Primitives**:
- **Channels**: FIFO, sync, lossy, filter, etc.
- **Nodes**: Merge, replicate, route
- **Composition**: Build complex coordinators from primitives

**Example**:
```
-- Exclusive router
A --sync--> (X) --sync--> B
            (X) --sync--> C

Data from A goes to either B or C, not both
```

**Constraint Automata**: Formal semantics for Reo using automata on ports.

### BPEL: Business Process Execution Language

**BPEL**: XML-based language for orchestrating web services.

**Constructs**:
- **Invoke**: Call web service
- **Receive**: Wait for incoming message
- **Reply**: Send response
- **Flow**: Parallel execution
- **Sequence**: Sequential execution
- **If/Switch**: Conditional branching

**Example**:
```xml
<sequence>
  <receive partnerLink="customer" operation="order"/>
  <flow>
    <invoke partnerLink="inventory" operation="checkStock"/>
    <invoke partnerLink="payment" operation="authorize"/>
  </flow>
  <invoke partnerLink="shipping" operation="ship"/>
  <reply partnerLink="customer" operation="order"/>
</sequence>
```

**Tool Support**: Engines like Apache ODE, Oracle BPEL PM.

---

## Practical Applications

### 1. Job Scheduling with Tuple Spaces

**Problem**: Distribute computational jobs to worker pool.

**Solution**:
```python
# Master
for job in jobs:
    space.out(("job", job.id, job.data))

# Worker pool (N workers)
while True:
    job_tuple = space.in(("job", None, None))
    job_id, job_data = job_tuple[1], job_tuple[2]
    result = process(job_data)
    space.out(("result", job_id, result))

# Collector
for i in range(len(jobs)):
    result_tuple = space.in(("result", None, None))
    results.append(result_tuple[2])
```

**Fault Tolerance**: If worker crashes, job remains in space for retry.

### 2. Manufacturing Process with Petri Nets

**Problem**: Model assembly line with resource constraints.

**Solution**:
```
[Raw Material] → (Machine A) → [Processed A] →
                                               (Assembly) → [Product]
[Components]   → (Machine B) → [Processed B] →

Tokens = items
Places = stages/buffers
Transitions = machines/operations
```

**Analysis**: Check for deadlocks, bottlenecks (min cut in reachability graph).

### 3. Distributed Lock Manager

**Problem**: Implement distributed mutual exclusion using tuple spaces.

**Solution**:
```python
# Initialize lock
space.out(("lock", "available"))

# Acquire lock
def acquire_lock(client_id):
    space.in(("lock", "available"))
    space.out(("lock", "held", client_id))

# Release lock
def release_lock(client_id):
    space.in(("lock", "held", client_id))
    space.out(("lock", "available"))
```

**Fairness**: FIFO fairness if tuple space has FIFO matching policy.

### 4. Workflow Automation

**Problem**: Automate document approval process.

**Solution**: Model as WF-net
```
[Submit] → (Validate) → [Pending] →
           (Approve)  → [Approved] → (Notify) → [Done]
           (Reject)   → [Rejected] → (Notify) → [Done]
```

**Implementation**: Use workflow engine (Camunda, Activiti) that executes WF-net.

---

## Stack-Specific Implementations

### Rust: Tuple Space with Message Passing

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// Tuple representation
#[derive(Clone, Debug, PartialEq)]
enum TupleField {
    Int(i32),
    Str(String),
    Formal(String), // Variable for matching
}

type Tuple = Vec<TupleField>;

/// Template for matching tuples
type Template = Vec<TupleField>;

/// Tuple space implementation
struct TupleSpace {
    tuples: Arc<Mutex<Vec<Tuple>>>,
    waiters: Arc<Mutex<HashMap<String, mpsc::Sender<Tuple>>>>,
}

impl TupleSpace {
    fn new() -> Self {
        TupleSpace {
            tuples: Arc::new(Mutex::new(Vec::new())),
            waiters: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Insert tuple into space (non-blocking)
    fn out(&self, tuple: Tuple) {
        let mut tuples = self.tuples.lock().unwrap();
        tuples.push(tuple.clone());

        // Notify waiting readers
        let waiters = self.waiters.lock().unwrap();
        for (_, sender) in waiters.iter() {
            let _ = sender.try_send(tuple.clone());
        }
    }

    /// Check if tuple matches template
    fn matches(tuple: &Tuple, template: &Template) -> bool {
        if tuple.len() != template.len() {
            return false;
        }

        tuple.iter().zip(template.iter()).all(|(t, p)| {
            match (t, p) {
                (_, TupleField::Formal(_)) => true, // Formal matches any
                (TupleField::Int(a), TupleField::Int(b)) => a == b,
                (TupleField::Str(a), TupleField::Str(b)) => a == b,
                _ => false,
            }
        })
    }

    /// Remove matching tuple (blocking)
    async fn in_async(&self, template: Template) -> Tuple {
        loop {
            // Try to find matching tuple
            {
                let mut tuples = self.tuples.lock().unwrap();
                if let Some(idx) = tuples.iter()
                    .position(|t| Self::matches(t, &template))
                {
                    return tuples.remove(idx);
                }
            }

            // Wait for new tuple
            let (tx, mut rx) = mpsc::channel(10);
            let waiter_id = uuid::Uuid::new_v4().to_string();

            {
                let mut waiters = self.waiters.lock().unwrap();
                waiters.insert(waiter_id.clone(), tx);
            }

            // Wait for matching tuple
            while let Some(tuple) = rx.recv().await {
                if Self::matches(&tuple, &template) {
                    let mut waiters = self.waiters.lock().unwrap();
                    waiters.remove(&waiter_id);
                    return tuple;
                }
            }
        }
    }

    /// Read matching tuple without removal (blocking)
    async fn rd_async(&self, template: Template) -> Tuple {
        loop {
            let tuples = self.tuples.lock().unwrap();
            if let Some(tuple) = tuples.iter()
                .find(|t| Self::matches(t, &template))
            {
                return tuple.clone();
            }
            drop(tuples);

            // Wait for new tuple
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    }
}

/// Example: Producer-Consumer with tuple space
async fn producer_consumer_example() {
    let space = Arc::new(TupleSpace::new());

    // Producer
    let space_clone = space.clone();
    tokio::spawn(async move {
        for i in 0..10 {
            space_clone.out(vec![
                TupleField::Str("job".to_string()),
                TupleField::Int(i),
            ]);
            println!("Produced job {}", i);
        }
    });

    // Consumer
    let space_clone = space.clone();
    tokio::spawn(async move {
        for _ in 0..10 {
            let tuple = space_clone.in_async(vec![
                TupleField::Str("job".to_string()),
                TupleField::Formal("id".to_string()),
            ]).await;

            if let TupleField::Int(id) = tuple[1] {
                println!("Consumed job {}", id);
            }
        }
    });
}

/// Petri Net implementation
struct PetriNet {
    places: HashMap<String, usize>, // Place name -> token count
    transitions: Vec<Transition>,
}

struct Transition {
    name: String,
    inputs: Vec<(String, usize)>,  // (place, tokens_needed)
    outputs: Vec<(String, usize)>, // (place, tokens_produced)
}

impl PetriNet {
    fn new() -> Self {
        PetriNet {
            places: HashMap::new(),
            transitions: Vec::new(),
        }
    }

    fn add_place(&mut self, name: String, initial_tokens: usize) {
        self.places.insert(name, initial_tokens);
    }

    fn add_transition(&mut self, transition: Transition) {
        self.transitions.push(transition);
    }

    /// Check if transition is enabled
    fn is_enabled(&self, transition: &Transition) -> bool {
        transition.inputs.iter().all(|(place, needed)| {
            self.places.get(place).map_or(false, |&count| count >= *needed)
        })
    }

    /// Fire enabled transition
    fn fire(&mut self, transition_name: &str) -> Result<(), String> {
        let transition = self.transitions.iter()
            .find(|t| t.name == transition_name)
            .ok_or("Transition not found")?;

        if !self.is_enabled(transition) {
            return Err("Transition not enabled".to_string());
        }

        // Consume input tokens
        for (place, count) in &transition.inputs {
            *self.places.get_mut(place).unwrap() -= count;
        }

        // Produce output tokens
        for (place, count) in &transition.outputs {
            *self.places.entry(place.clone()).or_insert(0) += count;
        }

        Ok(())
    }

    /// Get all enabled transitions
    fn enabled_transitions(&self) -> Vec<&str> {
        self.transitions.iter()
            .filter(|t| self.is_enabled(t))
            .map(|t| t.name.as_str())
            .collect()
    }
}

/// Example: Producer-Consumer Petri Net
fn producer_consumer_petri_net() {
    let mut net = PetriNet::new();

    // Places
    net.add_place("buffer".to_string(), 3); // 3 empty slots
    net.add_place("producer_ready".to_string(), 1);
    net.add_place("consumer_ready".to_string(), 1);

    // Transitions
    net.add_transition(Transition {
        name: "produce".to_string(),
        inputs: vec![
            ("producer_ready".to_string(), 1),
            ("buffer".to_string(), 1),
        ],
        outputs: vec![
            ("producer_ready".to_string(), 1),
            ("item".to_string(), 1),
        ],
    });

    net.add_transition(Transition {
        name: "consume".to_string(),
        inputs: vec![
            ("consumer_ready".to_string(), 1),
            ("item".to_string(), 1),
        ],
        outputs: vec![
            ("consumer_ready".to_string(), 1),
            ("buffer".to_string(), 1),
        ],
    });

    // Simulate
    for _ in 0..5 {
        println!("Enabled: {:?}", net.enabled_transitions());
        if net.is_enabled(&net.transitions[0]) {
            net.fire("produce").unwrap();
            println!("Produced");
        }
    }
}
```

### TypeScript: Tuple Space with Async Patterns

```typescript
type TupleField = number | string | symbol; // symbol = formal
type Tuple = TupleField[];
type Template = TupleField[];

class TupleSpace {
  private tuples: Tuple[] = [];
  private waiters: Map<string, (tuple: Tuple) => void> = new Map();

  /** Insert tuple (non-blocking) */
  out(tuple: Tuple): void {
    this.tuples.push(tuple);
    this.notifyWaiters(tuple);
  }

  /** Remove matching tuple (blocking) */
  async in(template: Template): Promise<Tuple> {
    // Try immediate match
    const index = this.tuples.findIndex(t =>
      this.matches(t, template)
    );

    if (index !== -1) {
      return this.tuples.splice(index, 1)[0];
    }

    // Wait for matching tuple
    return new Promise((resolve) => {
      const waiterId = `waiter-${Math.random()}`;
      this.waiters.set(waiterId, (tuple) => {
        if (this.matches(tuple, template)) {
          this.waiters.delete(waiterId);
          const idx = this.tuples.findIndex(t =>
            this.matches(t, template)
          );
          if (idx !== -1) {
            resolve(this.tuples.splice(idx, 1)[0]);
          }
        }
      });
    });
  }

  /** Read matching tuple (blocking, non-consuming) */
  async rd(template: Template): Promise<Tuple> {
    const tuple = this.tuples.find(t =>
      this.matches(t, template)
    );

    if (tuple) {
      return tuple;
    }

    // Wait for matching tuple
    return new Promise((resolve) => {
      const waiterId = `waiter-${Math.random()}`;
      this.waiters.set(waiterId, (tuple) => {
        if (this.matches(tuple, template)) {
          this.waiters.delete(waiterId);
          resolve(tuple);
        }
      });
    });
  }

  private matches(tuple: Tuple, template: Template): boolean {
    if (tuple.length !== template.length) return false;

    return tuple.every((field, i) => {
      const pattern = template[i];
      // Symbol represents formal (wildcard)
      if (typeof pattern === 'symbol') return true;
      return field === pattern;
    });
  }

  private notifyWaiters(tuple: Tuple): void {
    this.waiters.forEach(waiter => waiter(tuple));
  }
}

/** Example: Distributed Map-Reduce */
async function mapReduceExample() {
  const space = new TupleSpace();
  const FORMAL = Symbol('formal');

  // Master: distribute tasks
  const data = [1, 2, 3, 4, 5];
  for (let i = 0; i < data.length; i++) {
    space.out(['map-task', i, data[i]]);
  }

  // Workers: process map tasks
  const workers = Array.from({ length: 3 }, (_, workerId) =>
    (async () => {
      while (true) {
        const [_, id, value] = await space.in([
          'map-task',
          FORMAL,
          FORMAL
        ]) as [string, number, number];

        const result = value * 2; // Map function
        space.out(['reduce-task', id, result]);
      }
    })()
  );

  // Reducer: collect results
  const results: number[] = [];
  for (let i = 0; i < data.length; i++) {
    const [_, id, result] = await space.in([
      'reduce-task',
      FORMAL,
      FORMAL
    ]) as [string, number, number];
    results.push(result);
  }

  console.log('Results:', results);
}

/** Petri Net implementation */
interface Place {
  name: string;
  tokens: number;
}

interface Transition {
  name: string;
  inputs: { place: string; count: number }[];
  outputs: { place: string; count: number }[];
}

class PetriNet {
  private places = new Map<string, number>();
  private transitions: Transition[] = [];

  addPlace(name: string, initialTokens: number = 0): void {
    this.places.set(name, initialTokens);
  }

  addTransition(transition: Transition): void {
    this.transitions.push(transition);
  }

  isEnabled(transitionName: string): boolean {
    const transition = this.transitions.find(
      t => t.name === transitionName
    );
    if (!transition) return false;

    return transition.inputs.every(({ place, count }) => {
      const tokens = this.places.get(place) ?? 0;
      return tokens >= count;
    });
  }

  fire(transitionName: string): boolean {
    const transition = this.transitions.find(
      t => t.name === transitionName
    );

    if (!transition || !this.isEnabled(transitionName)) {
      return false;
    }

    // Consume input tokens
    transition.inputs.forEach(({ place, count }) => {
      const current = this.places.get(place)!;
      this.places.set(place, current - count);
    });

    // Produce output tokens
    transition.outputs.forEach(({ place, count }) => {
      const current = this.places.get(place) ?? 0;
      this.places.set(place, current + count);
    });

    return true;
  }

  getMarking(): Map<string, number> {
    return new Map(this.places);
  }

  enabledTransitions(): string[] {
    return this.transitions
      .filter(t => this.isEnabled(t.name))
      .map(t => t.name);
  }
}

/** Example: Workflow net */
function orderFulfillmentWorkflow() {
  const net = new PetriNet();

  // Places
  net.addPlace('order', 1);
  net.addPlace('validated', 0);
  net.addPlace('stock_checked', 0);
  net.addPlace('payment_ok', 0);
  net.addPlace('ready', 0);
  net.addPlace('shipped', 0);

  // Transitions
  net.addTransition({
    name: 'validate',
    inputs: [{ place: 'order', count: 1 }],
    outputs: [{ place: 'validated', count: 1 }],
  });

  net.addTransition({
    name: 'check_stock',
    inputs: [{ place: 'validated', count: 1 }],
    outputs: [{ place: 'stock_checked', count: 1 }],
  });

  net.addTransition({
    name: 'authorize_payment',
    inputs: [{ place: 'validated', count: 1 }],
    outputs: [{ place: 'payment_ok', count: 1 }],
  });

  net.addTransition({
    name: 'prepare',
    inputs: [
      { place: 'stock_checked', count: 1 },
      { place: 'payment_ok', count: 1 },
    ],
    outputs: [{ place: 'ready', count: 1 }],
  });

  net.addTransition({
    name: 'ship',
    inputs: [{ place: 'ready', count: 1 }],
    outputs: [{ place: 'shipped', count: 1 }],
  });

  return net;
}
```

### PHP: Tuple Space with Redis

```php
<?php

namespace Coordination;

use Illuminate\Support\Facades\Redis;

/**
 * Tuple Space implementation using Redis
 */
class TupleSpace
{
    private string $namespace;

    public function __construct(string $namespace = 'tuplespace')
    {
        $this->namespace = $namespace;
    }

    /**
     * Insert tuple into space (non-blocking)
     */
    public function out(array $tuple): void
    {
        $tupleJson = json_encode($tuple);
        $key = $this->namespace . ':tuples';

        Redis::rpush($key, $tupleJson);

        // Notify waiters via pub/sub
        Redis::publish(
            $this->namespace . ':notify',
            $tupleJson
        );
    }

    /**
     * Remove matching tuple (blocking with timeout)
     */
    public function in(array $template, int $timeoutSeconds = 60): ?array
    {
        $startTime = time();

        while (time() - $startTime < $timeoutSeconds) {
            // Try to find matching tuple
            $key = $this->namespace . ':tuples';
            $tuples = Redis::lrange($key, 0, -1);

            foreach ($tuples as $index => $tupleJson) {
                $tuple = json_decode($tupleJson, true);

                if ($this->matches($tuple, $template)) {
                    // Remove tuple atomically using Lua script
                    $script = <<<'LUA'
                        local tuples = redis.call('LRANGE', KEYS[1], 0, -1)
                        local index = tonumber(ARGV[1])
                        if tuples[index + 1] then
                            redis.call('LSET', KEYS[1], index, '__DELETED__')
                            redis.call('LREM', KEYS[1], 1, '__DELETED__')
                            return tuples[index + 1]
                        end
                        return nil
LUA;

                    $result = Redis::eval(
                        $script,
                        1,
                        $key,
                        $index
                    );

                    if ($result) {
                        return json_decode($result, true);
                    }
                }
            }

            // Wait for new tuple
            usleep(100000); // 100ms
        }

        return null;
    }

    /**
     * Read matching tuple (non-consuming, blocking)
     */
    public function rd(array $template, int $timeoutSeconds = 60): ?array
    {
        $startTime = time();

        while (time() - $startTime < $timeoutSeconds) {
            $key = $this->namespace . ':tuples';
            $tuples = Redis::lrange($key, 0, -1);

            foreach ($tuples as $tupleJson) {
                $tuple = json_decode($tupleJson, true);

                if ($this->matches($tuple, $template)) {
                    return $tuple;
                }
            }

            usleep(100000); // 100ms
        }

        return null;
    }

    /**
     * Check if tuple matches template
     */
    private function matches(array $tuple, array $template): bool
    {
        if (count($tuple) !== count($template)) {
            return false;
        }

        foreach ($tuple as $i => $field) {
            $pattern = $template[$i];

            // null represents formal (wildcard)
            if ($pattern === null) {
                continue;
            }

            if ($field !== $pattern) {
                return false;
            }
        }

        return true;
    }
}

/**
 * Petri Net implementation
 */
class PetriNet
{
    private array $places = [];
    private array $transitions = [];

    public function addPlace(string $name, int $initialTokens = 0): void
    {
        $this->places[$name] = $initialTokens;
    }

    public function addTransition(
        string $name,
        array $inputs,
        array $outputs
    ): void {
        $this->transitions[$name] = [
            'inputs' => $inputs,   // ['place' => count]
            'outputs' => $outputs, // ['place' => count]
        ];
    }

    public function isEnabled(string $transitionName): bool
    {
        if (!isset($this->transitions[$transitionName])) {
            return false;
        }

        $transition = $this->transitions[$transitionName];

        foreach ($transition['inputs'] as $place => $count) {
            if (($this->places[$place] ?? 0) < $count) {
                return false;
            }
        }

        return true;
    }

    public function fire(string $transitionName): bool
    {
        if (!$this->isEnabled($transitionName)) {
            return false;
        }

        $transition = $this->transitions[$transitionName];

        // Consume input tokens
        foreach ($transition['inputs'] as $place => $count) {
            $this->places[$place] -= $count;
        }

        // Produce output tokens
        foreach ($transition['outputs'] as $place => $count) {
            $this->places[$place] =
                ($this->places[$place] ?? 0) + $count;
        }

        return true;
    }

    public function getMarking(): array
    {
        return $this->places;
    }

    public function enabledTransitions(): array
    {
        return array_filter(
            array_keys($this->transitions),
            fn($name) => $this->isEnabled($name)
        );
    }
}

/**
 * Example: Job queue with tuple space
 */
class JobQueue
{
    private TupleSpace $space;

    public function __construct()
    {
        $this->space = new TupleSpace('jobqueue');
    }

    public function submitJob(string $type, array $data): void
    {
        $jobId = uniqid('job_', true);
        $this->space->out(['job', $type, $jobId, $data]);
        Log::info("Submitted job: $jobId");
    }

    public function worker(string $workerType): void
    {
        while (true) {
            // Match any job of our type
            $tuple = $this->space->in(
                ['job', $workerType, null, null],
                timeout: 30
            );

            if ($tuple) {
                [$_, $type, $jobId, $data] = $tuple;
                Log::info("Processing job: $jobId");

                try {
                    $result = $this->processJob($type, $data);
                    $this->space->out([
                        'result',
                        $jobId,
                        'success',
                        $result
                    ]);
                } catch (\Exception $e) {
                    $this->space->out([
                        'result',
                        $jobId,
                        'error',
                        $e->getMessage()
                    ]);
                }
            }
        }
    }

    private function processJob(string $type, array $data): mixed
    {
        // Simulate work
        sleep(1);
        return ['processed' => $data];
    }
}

/**
 * Example: Order workflow with Petri net
 */
function createOrderWorkflow(): PetriNet
{
    $net = new PetriNet();

    // Places
    $net->addPlace('order_received', 1);
    $net->addPlace('validated', 0);
    $net->addPlace('inventory_checked', 0);
    $net->addPlace('payment_authorized', 0);
    $net->addPlace('ready_to_ship', 0);
    $net->addPlace('shipped', 0);

    // Transitions
    $net->addTransition(
        'validate_order',
        ['order_received' => 1],
        ['validated' => 1]
    );

    $net->addTransition(
        'check_inventory',
        ['validated' => 1],
        ['inventory_checked' => 1]
    );

    $net->addTransition(
        'authorize_payment',
        ['validated' => 1],
        ['payment_authorized' => 1]
    );

    $net->addTransition(
        'prepare_shipment',
        [
            'inventory_checked' => 1,
            'payment_authorized' => 1,
        ],
        ['ready_to_ship' => 1]
    );

    $net->addTransition(
        'ship_order',
        ['ready_to_ship' => 1],
        ['shipped' => 1]
    );

    return $net;
}
```

---

## Integration Points

### With Actor Model
- **Tuple spaces** provide shared coordination space for actors
- **Petri nets** model actor interaction patterns and supervision strategies
- **Workflow nets** specify multi-actor protocols

**Example**: Saga pattern using tuple spaces for compensation coordination.

### With Process Calculi
- **Linda** can be encoded in π-calculus (channels for tuple matching)
- **Petri nets** have process algebra semantics (Petri net algebra)
- **Reo** combines channel-based coordination with process algebra

**Example**: Prove Linda operation atomicity using process calculus.

### With Distributed Systems
- **Tuple spaces** provide coordination for distributed components
- **Petri nets** model distributed protocols and verify correctness
- **Workflow nets** orchestrate microservices

**Example**: Model two-phase commit as Petri net, verify liveness.

### With Type Theory
- **Session types** for typed coordination channels
- **Dependent types** for tuple space templates with constraints
- **Linear types** ensure tuple consumed exactly once

**Example**: `in` operation has linear type - tuple removed from space.

---

## Further Reading

### Foundational Papers
- Gelernter (1985) - "Generative Communication in Linda"
- Petri (1962) - "Kommunikation mit Automaten"
- van der Aalst (1998) - "The Application of Petri Nets to Workflow Management"
- Arbab (2004) - "Reo: A Channel-based Coordination Model for Component Composition"

### Books
- Carriero & Gelernter - "How to Write Parallel Programs: A Guide to the Perplexed"
- Reisig - "A Primer in Petri Net Design"
- van der Aalst & Stahl - "Modeling Business Processes: A Petri Net-Oriented Approach"
- Papadopoulos & Arbab - "Coordination Models and Languages"

### Tools
- **JavaSpaces** - Tuple space for Java
- **CPN Tools** - Colored Petri Net modeling and analysis
- **ProM** - Process mining framework
- **Reo** - Coordination language implementation
- **PIPE** - Platform Independent Petri Net Editor

---

**End of Coordination Models Appendix**
