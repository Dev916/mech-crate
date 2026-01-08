# Theory Map: Complete Integration Guide

Purpose: Master index connecting all theoretical foundations, showing how concepts relate, when to use each paradigm, and stack-specific implementations.

## Complete Theory Catalog

### Your Foundation (Existing)
- **appendix-algebraic-effects-optics.md** - Pure effects & nested state
- **appendix-concurrency-time.md** - Time semantics & per-aggregate serialization
- **appendix-emergence.md** - Systems thinking & adaptation
- **appendix-frp-general.md** (+ variants) - Reactive programming
- **appendix-fsm.md** - State machine modeling
- **appendix-pattern-playbook.md** - Practical patterns
- **appendix-streams.md** - Streaming systems deep dive

### New Theoretical Extensions
- **appendix-actor-model.md** - Message-passing concurrency
- **appendix-process-calculi.md** - CSP, π-calculus, session types
- **appendix-coordination-models.md** - Tuple spaces, Petri nets
- **appendix-dataflow.md** - Kahn networks, SDF, FBP
- **appendix-consensus.md** - Paxos, Raft, Byzantine consensus
- **appendix-consistency-models.md** - Linearizability to eventual
- **appendix-memory-models.md** - Hardware/software memory semantics
- **appendix-type-theory.md** - Types as specifications
- **appendix-formal-verification.md** - Proving correctness
- **appendix-category-theory.md** - Mathematical composition

---

## Concept Dependency Graph

```
                    Category Theory
                   (composition laws)
                          │
        ┌─────────────────┼─────────────────┐
        │                 │                 │
   Type Theory      Functors/Monads    Adjunctions
        │                 │                 │
        ├─────────────────┴─────────────────┤
        │                                   │
   Dependent Types                    Free Structures
   Linear Types                       Effect Algebras
   Session Types                           │
        │                                   │
        └─────────────┬─────────────────────┘
                      │
          ┌───────────┼───────────┐
          │           │           │
    Process Calculi  Actor     Algebraic
    (CSP/π-calc)     Model     Effects
          │           │           │
          └───────────┼───────────┘
                      │
              ┌───────┼───────┐
              │       │       │
         Dataflow  Streams   FRP
              │       │       │
              └───────┼───────┘
                      │
          ┌───────────┼───────────┐
          │           │           │
    Coordination  Consensus  Consistency
    (Petri nets)  (Paxos)    Models
          │           │           │
          └───────────┼───────────┘
                      │
              Memory Models
              Formal Verification
```

---

## Integration Matrix

### By Architectural Concern

#### Correctness & Safety
```yaml
Type-Level Safety:
  - Type Theory → dependent/linear/refinement types
  - Session Types → protocol correctness
  - Category Theory → composition laws

Runtime Verification:
  - Formal Verification → TLA+, model checking, proof assistants
  - Process Calculi → bisimulation, refinement
  - Petri Nets → reachability, liveness analysis

Testing:
  - Property-based → algebraic laws
  - Model-based → FSM coverage
  - Simulation → deterministic replay
```

#### Concurrency & Distribution
```yaml
Message Passing:
  - Actor Model → isolated state, supervision
  - Process Calculi → formal semantics
  - Session Types → protocol types

Shared Memory:
  - Memory Models → happens-before, acquire-release
  - Consistency Models → linearizability spectrum
  - Formal Verification → concurrent correctness proofs

Distributed Coordination:
  - Consensus → Paxos/Raft for agreement
  - CRDTs → conflict-free eventual consistency
  - Coordination Models → tuple spaces, barriers
```

#### Composability & Abstraction
```yaml
Functional Composition:
  - Category Theory → functors, monads, natural transformations
  - Algebraic Effects → effect composition
  - Streams → operator algebra

Dataflow Composition:
  - Kahn Networks → deterministic parallelism
  - FRP → signal functions, behaviors
  - Reactive Streams → backpressure composition
```

---

## Decision Trees

### "How Should I Model Concurrency?"

```
START: What's the primary coordination mechanism?

MESSAGE PASSING
├─ Isolated stateful entities? → Actor Model
├─ Dynamic reconfiguration? → π-calculus concepts
├─ Protocol verification? → Session Types
└─ Pipeline/dataflow? → CSP/Channels

SHARED MEMORY
├─ Need formal memory semantics? → Memory Models appendix
├─ Lock-free data structures? → Progress guarantees (wait-free → lock-free)
├─ Distributed shared state? → Consistency Models
└─ Proof of correctness? → Separation Logic

COORDINATION PATTERNS
├─ Multi-way synchronization? → Join Calculus
├─ Workflow modeling? → Petri Nets
├─ Decoupled pub-sub? → Tuple Spaces
└─ Event-driven? → FRP/Streams

DETERMINISTIC DATAFLOW
├─ Static schedule possible? → Synchronous Dataflow
├─ Dynamic streaming? → Kahn Process Networks
└─ Component-based? → Flow-Based Programming
```

### "What Consistency Model Do I Need?"

```
START: What are the requirements?

STRICT ORDERING REQUIRED
├─ Single-object operations? → Linearizability
├─ Multi-object transactions? → Strict Serializability
└─ Total order across all ops? → Sequential Consistency

CAUSALITY MATTERS
├─ See-your-own-writes? → Causal Consistency + session guarantees
├─ Related operations ordered? → Causal+ / COPS
└─ Collaborative editing? → CRDTs + causal metadata

HIGH AVAILABILITY PRIORITY
├─ Can tolerate staleness? → Eventual Consistency
├─ Need convergence guarantees? → Strong Eventual (CRDTs)
└─ Application-specific resolution? → Last-write-wins + timestamps
```

### "Which Verification Technique?"

```
START: What are you verifying?

FINITE STATE SYSTEM
├─ Safety properties (bad states)? → Model Checking (SPIN/TLC)
├─ Liveness (progress)? → Model Checking + fairness
└─ Protocol conformance? → Session Types / Process calculi

DISTRIBUTED ALGORITHM
├─ Consensus/agreement? → TLA+ specification
├─ Consistency model? → Formal model + invariants
└─ Fault tolerance? → Model checking with failures

FUNCTIONAL CORRECTNESS
├─ Pure functions? → Property-based testing
├─ Stateful systems? → Hoare Logic / Separation Logic
└─ Full proof? → Proof Assistants (Coq/Lean)

PERFORMANCE BOUNDS
├─ Time complexity? → Complexity Theory appendix
├─ Concurrent progress? → Progress guarantees analysis
└─ Resource usage? → Amortized analysis
```

---

## Cross-Reference Guide

### Actor Model ↔ Other Paradigms

**Actor Model + Process Calculi**
```
Actors formalize:          π-calculus provides:
- Async message passing    - Formal semantics
- Location transparency    - Bisimulation equivalence
- Dynamic creation         - Scope extrusion rules

Use both when: Need formal verification of actor protocols
```

**Actor Model + Session Types**
```
Actors provide:            Sessions provide:
- Runtime implementation   - Compile-time protocol checking
- Fault tolerance          - Type-safe communication
- Distribution             - Deadlock freedom

Use both when: Building distributed protocols with correctness guarantees
```

**Actor Model + Consensus**
```
Actors provide:            Consensus provides:
- Per-entity isolation     - Cross-entity agreement
- Independent state        - Consistent views
- Message delivery         - Total order broadcast

Use both when: Coordinating distributed actors (e.g., cluster membership)
```

### Type Theory ↔ Other Paradigms

**Dependent Types + Verification**
```
Dependent types:           Formal verification:
- Specifications in types  - Specifications in logic
- Compile-time checking    - Proof obligations
- Constructive proofs      - Automated reasoning

Use both when: Want strongest guarantees (e.g., certified compilers)
```

**Linear Types + Actors**
```
Linear types:              Actors:
- Resource tracking        - Lifecycle management
- Use-once guarantees      - Message consumption
- Compile-time             - Runtime

Use both when: Building systems with resource constraints (files, connections)
```

**Session Types + Protocols**
```
Session types:             Process calculi:
- Protocol as type         - Protocol as process
- Type checking            - Bisimulation checking
- Static guarantees        - Dynamic semantics

Use both when: Implementing and verifying communication protocols
```

### Consistency Models ↔ Implementations

**Linearizability + Actor Model**
```
Challenge: Actors are inherently async/eventual
Solution: Add explicit synchronization points
- Use ask() for synchronous request-response
- Implement consensus for critical operations
- Partition by key for per-key linearizability
```

**Causal Consistency + CRDTs**
```
CRDTs provide:             Causality tracking:
- Conflict-free merge      - Vector clocks
- Commutativity            - Happens-before
- Convergence              - Causal ordering

Use both when: Offline-first, collaborative applications
```

**Eventual Consistency + Streams**
```
Streams handle:            Eventual consistency:
- Time-varying values      - Temporary divergence
- Propagation delays       - Convergence guarantees
- Backpressure             - Anti-entropy

Use both when: Real-time dashboards, monitoring systems
```

---

## Language-Specific Roadmaps

### Rust Journey

```
FOUNDATION
├─ Ownership/Borrowing → Affine Types (Type Theory)
├─ Traits → Type Classes (Category Theory)
└─ async/await → Effect Systems

CONCURRENCY
├─ Channels → CSP (Process Calculi)
├─ Actor frameworks → Actor Model + Supervision
├─ Atomic operations → Memory Models (Acquire-Release)
└─ Lock-free structures → Progress Guarantees

VERIFICATION
├─ Type system → Refinement Types (basic)
├─ Testing → Property-based (QuickCheck-style)
└─ Unsafe code → Formal verification (limited)

PATTERNS
├─ Iterators/Streams → FRP, Dataflow
├─ Result/Option → Category Theory (Monads)
└─ Builder pattern → Free structures
```

### TypeScript Journey

```
FOUNDATION
├─ Type system → Structural types, Generics
├─ Promises/async → Effect types, Monads
└─ Union types → Sum types (Type Theory)

CONCURRENCY
├─ Event loop → Actor-like patterns
├─ Worker threads → Message passing
├─ RxJS → FRP, Streams, Reactive
└─ Channels (libraries) → CSP patterns

VERIFICATION
├─ Type guards → Refinement types (limited)
├─ Branded types → Session types (partial)
└─ Property testing → QuickCheck-style

PATTERNS
├─ fp-ts → Category Theory (functors, monads)
├─ State machines → FSM, XState
└─ Effect-TS → Algebraic effects
```

### PHP Journey

```
FOUNDATION
├─ Types (8.x+) → Basic type safety
├─ Enums (8.1+) → Sum types
└─ Attributes → Metadata, aspects

CONCURRENCY
├─ Laravel Queues → Actor-like job processing
├─ Amp/ReactPHP → Async, event-driven
├─ Swoole → Long-running processes
└─ Message buses → Event sourcing patterns

VERIFICATION
├─ PHPStan/Psalm → Static analysis
├─ Pest property tests → Property-based testing
└─ Integration tests → Behavior verification

PATTERNS
├─ Event Sourcing → FSM + Events
├─ Pipeline pattern → Dataflow
├─ Repository → Ports & adapters
└─ Collections → Functor/Monad patterns (limited)
```

---

## Practical Application Patterns

### Pattern: Distributed Saga with Actors + Consensus

```
Problem: Multi-service transaction across actors

Solution Stack:
1. Actor Model → Per-service coordinator
2. Session Types → Protocol between coordinators
3. Consensus → Decide commit/abort
4. Algebraic Effects → Pure saga logic

Implementation:
- Each service = actor with local state
- Saga coordinator = actor orchestrating protocol
- Consensus (Raft) for final decision
- Effects separate decision from execution
```

### Pattern: Real-Time Collaboration

```
Problem: Multiple users editing same document

Solution Stack:
1. CRDTs → Conflict-free data structure
2. Causal Consistency → Track happens-before
3. Streams → Propagate updates
4. FRP → UI reactivity

Implementation:
- CRDT (text, JSON, etc.) as data model
- Vector clocks for causality
- WebSocket stream for updates
- FRP signals for UI updates
```

### Pattern: Type-Safe Microservices

```
Problem: Inter-service communication correctness

Solution Stack:
1. Session Types → Protocol specification
2. Process Calculi → Formal semantics
3. Code generation → From global types
4. Actor Model → Service implementation

Implementation:
- Scribble global protocol
- Generate stubs for each service
- Implement services as actors
- Session types ensure correctness
```

---

## Learning Paths

### Path 1: From Imperative to Functional

```
1. Category Theory basics → Functors, Monads
2. Algebraic Effects → Separate effects from logic
3. FRP → Reactive, declarative style
4. Type Theory → Dependent types for specs
5. Formal Verification → Proof of correctness
```

### Path 2: From OOP to Actor-Based

```
1. Actor Model → Message passing, isolation
2. Supervision → Fault tolerance patterns
3. Process Calculi → Formal understanding
4. Consensus → Distributed coordination
5. Consistency Models → Trade-offs
```

### Path 3: From Monolith to Distributed

```
1. Consistency Models → Understand spectrum
2. Consensus → Agreement algorithms
3. Actor Model → Natural distribution model
4. Coordination Models → Decoupled communication
5. Formal Verification → Distributed correctness
```

---

## Quick Reference: When to Use What

```yaml
Stateful Entities:
  primary: Actor Model
  complement: FSM, Session Types
  verify: TLA+, Model Checking

Pure Transformations:
  primary: Category Theory (functors)
  complement: Algebraic Effects
  verify: Property-based testing

Protocols:
  primary: Session Types
  complement: Process Calculi
  verify: Refinement checking

Streaming Data:
  primary: Streams, FRP
  complement: Dataflow models
  verify: Temporal logic

Distributed State:
  primary: Consistency Models, CRDTs
  complement: Consensus
  verify: TLA+, Formal verification

Workflows:
  primary: Petri Nets, FSM
  complement: Coordination Models
  verify: Reachability analysis
```

---

## Further Resources

### Books
- "Types and Programming Languages" (Pierce) → Type Theory foundation
- "Purely Functional Data Structures" (Okasaki) → Persistent structures
- "Specifying Systems" (Lamport) → TLA+ and distributed systems
- "Category Theory for Programmers" (Milewski) → Categorical thinking
- "The Art of Multiprocessor Programming" (Herlihy & Shavit) → Concurrency

### Papers
- Hewitt (1973) - Actor Model
- Hoare (1978) - CSP
- Milner (1992) - π-calculus
- Lamport (1998) - Part-Time Parliament (Paxos)
- Herlihy & Wing (1990) - Linearizability

### Tools
- TLA+ Toolbox → Distributed system verification
- SPIN → Protocol verification
- Coq/Lean → Proof assistants
- QuickCheck variants → Property-based testing
- Alloy → Lightweight formal methods

---

*End of Theory Map*
