# **Living Pattern Playbook — Rust, Laravel, Nuxt** {#living-pattern-playbook-—-rust,-laravel,-nuxt}

A practical, evolving guide to design patterns, reliability tactics, and modeling techniques you can drop into real services. Code is organized in appendices so the core narrative stays clean.

---

## **Table of Contents** {#table-of-contents}

[Living Pattern Playbook — Rust, Laravel, Nuxt](#living-pattern-playbook-—-rust,-laravel,-nuxt)

[Table of Contents](#table-of-contents)

[1\. Purpose and scope](#1.-purpose-and-scope)

[2\. Principles that guide every service](#2.-principles-that-guide-every-service)

[3\. Pattern index at a glance](#3.-pattern-index-at-a-glance)

[4\. How to choose patterns](#4.-how-to-choose-patterns)

[5\. Modeling with statecharts and contracts](#5.-modeling-with-statecharts-and-contracts)

[6\. Reliability toolkit](#6.-reliability-toolkit)

[7\. Observability plan](#7.-observability-plan)

[8\. Delivery and rollout plan](#8.-delivery-and-rollout-plan)

[9\. Data and schema evolution](#9.-data-and-schema-evolution)

[10\. Security and trust as code](#10.-security-and-trust-as-code)

[11\. Team workflow and decision records](#11.-team-workflow-and-decision-records)

[12\. One month adoption plan](#12.-one-month-adoption-plan)

[13\. Appendices](#13.-appendices)

[Appendix A. Rust blueprints](#appendix-a.-rust-blueprints)

[Appendix B. Laravel blueprints](#appendix-b.-laravel-blueprints)

[Appendix C. Nuxt blueprints](#appendix-c.-nuxt-blueprints)

[Appendix D. TLA+ and Alloy starter specs](#appendix-d.-tla+-and-alloy-starter-specs)

[Appendix E. ADR template](#appendix-e.-adr-template)

[Appendix F. Property based testing recipes](#appendix-f.-property-based-testing-recipes)

[Appendix G. CRDT and real time collaboration](#appendix-g.-crdt-and-real-time-collaboration)

[Appendix H. Pure Reducer Patterns (Deep Dive)](#appendix-h.-pure-reducer-patterns-\(deep-dive\))

[1\) What is a pure reducer?](#1\)-what-is-a-pure-reducer?)

[2\) Laws & invariants](#2\)-laws-&-invariants)

[3\) Architecture: Functional core, imperative shell](#3\)-architecture:-functional-core,-imperative-shell)

[4\) Composition patterns](#4\)-composition-patterns)

[5\) Testing reducers (fast \+ powerful)](#5\)-testing-reducers-\(fast-+-powerful\))

[6\) Performance & ergonomics](#6\)-performance-&-ergonomics)

[7\) Concurrency & Distribution](#7\)-concurrency-&-distribution)

[Aggregate boundaries](#aggregate-boundaries)

[Serialization strategies](#serialization-strategies)

[Cross-aggregate coordination](#cross-aggregate-coordination)

[Distribution patterns](#distribution-patterns)

[Invariants under concurrency](#invariants-under-concurrency)

[8\) Anti-patterns](#8\)-anti-patterns)

[9\) Blueprints](#9\)-blueprints)

[Rust — Pure reducer \+ effects pattern](#rust-—-pure-reducer-+-effects-pattern)

[Laravel (PHP) — Reducer-as-service and fold](#laravel-\(php\)-—-reducer-as-service-and-fold)

[Nuxt/TypeScript — Slice reducers \+ combine](#nuxt/typescript-—-slice-reducers-+-combine)

[Appendix I. Testing Strategies for Reducers](#appendix-i.-testing-strategies-for-reducers)

[1\) Property-based testing](#1\)-property-based-testing)

[2\) Replay testing](#2\)-replay-testing)

[3\) Metamorphic testing](#3\)-metamorphic-testing)

[4\) Simulation testing](#4\)-simulation-testing)

[5\) Mutation testing](#5\)-mutation-testing)

[10\. Migration & Interoperability](#10\)-migration-&-interoperability)

[11\. Checklist for shipping reducer-based features](#11\)-checklist-for-shipping-reducer-based-features)

[12\. Key Takeaways](#12\)-key-takeaways)

[13\. Next Steps](#13\)-next-steps)

---

## **1\. Purpose and scope** {#1.-purpose-and-scope}

Give teams a single source of truth for modern patterns that trade complexity for leverage. The goal is to make correctness and evolution natural.

Out of scope: exhaustive academic treatment. We target implementable slices with references to deeper sources when useful.

---

## **2\. Principles that guide every service** {#2.-principles-that-guide-every-service}

* Information hiding and stable interfaces  
* High cohesion with low coupling  
* Explicit state and transitions  
* Idempotency by default  
* Time budget and deadlines propagate  
* Backpressure instead of buffers  
* Schemas are contracts with version rules  
* Security decisions are centralized and testable  
* Everything important is observable  
* Small controlled experiments behind a flag

---

## **3\. Pattern index at a glance** {#3.-pattern-index-at-a-glance}

**Architecture**

* Hexagonal ports and adapters  
* Modular monolith with bounded contexts  
* Event sourcing and projections when audit or time travel is required  
* Orchestrated sagas for multi step money or entitlement flows  
* Workflow engines for durable timers and compensation

**Concurrency and flow control**

* Actor model and CSP style channels  
* Reactive streams with backpressure  
* Bulkheads, circuit breakers, hedged requests

**Data and evolution**

* Append only event logs  
* Data contracts with compatibility gates  
* Change data capture driven projections

**Collaboration and offline**

* CRDT based sync for local first experiences  
* Causality aware UX

**Testing and assurance**

* Design by contract  
* Property based and model based testing  
* Deterministic simulation for workflows  
* Mutation testing to harden test suites

---

## **4\. How to choose patterns** {#4.-how-to-choose-patterns}

Use a quick decision table at design time. Pick the dominant force and apply the smallest pattern that resolves it.

| Force | Symptom | Minimal pattern | When to avoid |
| ----- | ----- | ----- | ----- |
| Temporal complexity | Many branches and retries | Statechart with guards and invariants | When flow is a single step |
| Multi service money flow | Needs compensation and audit | Saga with orchestrator plus outbox | When two steps and same database |
| Collaboration | Concurrent edits and offline | CRDT document plus event log | When strict linear history is required |
| Scale and latency | Queue storms and fan out | Backpressure with bounded queues | When throughput is tiny |
| Auditability | Who did what and when | Event sourcing with projections | When history is irrelevant |

---

## **5\. Modeling with statecharts and contracts** {#5.-modeling-with-statecharts-and-contracts}

Model work as a hierarchical statechart with explicit events. Then state invariants as contracts. Keep the model text based and close to the code. See the code in Appendix A and B for Rust and Laravel implementations, and Appendix C for Nuxt UI state.

Checklist

* All states listed with entry and exit actions  
* Events listed with guards and expected effects  
* Illegal states are unrepresentable in data types  
* Invariants captured as runtime contracts and property tests

---

## **6\. Reliability toolkit** {#6.-reliability-toolkit}

Use defaults that become shared code in every service.

* Idempotency  
  * Carry an idempotency key in every external call and command  
  * Store attempt ledger with status and checksum  
* Retries  
  * Exponential backoff with jitter and a hard cap  
  * Retry only on safe conditions and idempotent operations  
* Timeouts and deadlines  
  * One default time budget, propagate deadline through calls  
  * Log remaining time in spans  
* Outbox and inbox tables  
  * Write business change and message in one transaction  
  * A background relay publishes from outbox  
* Circuit breaker and bulkheads  
  * Trip on error rate and latency budget breach  
  * Reserve capacity per consumer pool  
* Sagas  
  * Explicit steps with compensation and a durable state store

---

## **7\. Observability plan** {#7.-observability-plan}

* Metrics  
  * RED for services and SLI per critical endpoint  
* Traces  
  * One span per request with business attributes and idempotency key  
* Logs  
  * Structured, sampled at the edge, correlated by trace id  
* SLOs and error budgets  
  * Publish targets and budget policy in repo

---

## **8\. Delivery and rollout plan** {#8.-delivery-and-rollout-plan}

* Feature flags  
* Dark launch and shadow traffic  
* Canary then linear ramp  
* Rollback plan in the pull request template  
* Post release verification playbook

---

## **9\. Data and schema evolution** {#9.-data-and-schema-evolution}

* Version schemas and review compatibility  
* Additive changes first, then migrations with dual write or backfill  
* CDC stream for projections and caches  
* Archive policy for append only stores

---

## **10\. Security and trust as code** {#10.-security-and-trust-as-code}

* Threat model per feature with STRIDE  
* Policy engine for authorization rules  
* Secrets and keys rotated with short TTL  
* Signed events for audit with key provenance

---

## **11\. Team workflow and decision records** {#11.-team-workflow-and-decision-records}

* One ADR per significant decision with alternatives and tradeoffs  
* Design review checklist in repo  
* Runbooks co located with services

---

## **12\. One month adoption plan** {#12.-one-month-adoption-plan}

Week 1

* Pick one flow and model it as a statechart  
* Add property based tests for invariants

Week 2

* Add outbox and idempotency middleware to one service  
* Add deadlines and a retry policy as a shared library

Week 3

* Move a real saga into a workflow engine  
* Add standard metrics and traces with exemplars

Week 4

* Pilot CRDT based sync or WASM plugin behind a flag  
* Shadow traffic and compare causally

---

## **13\. Appendices** {#13.-appendices}

### **Appendix A. Rust blueprints** {#appendix-a.-rust-blueprints}

* State machine with enums and pattern matching  
* Command handler with outbox pattern  
* Property based test with proptest  
* Tower layer for idempotency and deadlines  
* TLA+ sketch for a saga

See code in this appendix section.

### **Appendix B. Laravel blueprints** {#appendix-b.-laravel-blueprints}

* Module structure for bounded contexts  
* Domain events and outbox relay  
* Idempotency middleware  
* Horizon ready queue setup with retries and jitter  
* Saga coordinator options and Temporal interop  
* Pest tests with generators and mutation testing with Infection

### **Appendix C. Nuxt blueprints** {#appendix-c.-nuxt-blueprints}

* Statecharts with XState  
* Event modeled UI with projections  
* Offline first with Yjs and repair loops  
* Edge function adapter for ports and adapters style

### **Appendix D. TLA+ and Alloy starter specs** {#appendix-d.-tla+-and-alloy-starter-specs}

* Minimal specs to model order capture and payment saga

### **Appendix E. ADR template** {#appendix-e.-adr-template}

* Copy friendly template for quick decisions

### **Appendix F. Property based testing recipes** {#appendix-f.-property-based-testing-recipes}

* Patterns for generators and shrinking

### **Appendix G. CRDT and real time collaboration** {#appendix-g.-crdt-and-real-time-collaboration}

* Choose a document shape per feature: map for forms, array for ordered lists, text for editors  
* Store events in an append only log with causal metadata  
* Reconcile with a repair loop that compares projection to source of truth

---

### **Appendix H. Pure Reducer Patterns (Deep Dive)** {#appendix-h.-pure-reducer-patterns-(deep-dive)}

#### **1\) What is a pure reducer?** {#1)-what-is-a-pure-reducer?}

A **reducer** is a total, deterministic function of the form:

```
(State, Event) -> State
```

*   
  **Pure**: no IO, no time, no randomness, no hidden reads/writes.  
* **Total**: defined for all valid `(state, event)` pairs; if an event is illegal for a state, return the *same* state (no-op) or a well-defined error value (sum type).  
* **Deterministic**: same inputs, same output — this unlocks replay, time travel, and property-based testing.

Think: the reducer is the *mathematical heart* of your domain; everything else (IO, DB, queues) is the *shell*.

---

#### **2\) Laws & invariants** {#2)-laws-&-invariants}

If you log events `e1..en` and fold them:

```
fold(State0, [e1, e2, ..., en]) = reduce(...reduce(reduce(State0, e1), e2)..., en)
```

Useful laws (context-dependent):

* **Idempotency (per event type)**: applying the same event twice should either be impossible (guarded by command-layer) or a no-op.  
* **Monotonicity (for some domains)**: certain counters or sets only grow (append-only). Model as such.  
* **Compositionality**: global state can be a product of slices; reducers compose via `product` and `sum` types.  
* **Illegal states unrepresentable**: encode with ADTs/enums and typed fields.

---

#### **3\) Architecture: Functional core, imperative shell** {#3)-architecture:-functional-core,-imperative-shell}

* **Core**: reducers \+ pure decision logic (validation, pricing, transitions). No framework/IO.  
* **Shell**: adapters that translate *commands* to *events*, persist events, call reducers, publish side effects.  
* **Boundary**: the shell never mutates domain state directly — it *feeds events* to the reducer.

Pattern: `handle(command) -> [events, effects]` where the reducer stays `(state, event) -> state`. The command handler is allowed to do IO to validate and then emit events; the reducer folds them.

---

#### **4\) Composition patterns** {#4)-composition-patterns}

* **Slice reducers (product types)**: split state into slices, each with its reducer, then combine.  
* **Layered reducers**: base reducer \+ decorators that enforce cross-cutting invariants (e.g., quotas, limits).  
* **Event upcasting**: support schema evolution by mapping old events to new ones before reducing.  
* **Lens-based updates**: use lenses/paths to focus updates inside nested state without mutation.

---

#### **5\) Testing reducers (fast \+ powerful)** {#5)-testing-reducers-(fast-+-powerful)}

* **Example tests**: given state \+ event \=\> expected state.  
* **Property-based**: generate arbitrary event sequences; assert invariants always hold and no panics occur.  
* **Replay tests**: a production event log must reproduce production projections bit-for-bit.  
* **Metamorphic tests**: inserting a no-op event, or reordering commutative events, does not change final state.

---

#### **6\) Performance & ergonomics** {#6)-performance-&-ergonomics}

* Prefer **persistent data structures** or structural sharing (Rust: `Arc`, smallvec, custom; JS: Immer or immutable libs) to keep copying cheap.  
* Normalize collections (ids \-\> maps) to keep reducer operations O(1) or O(log n).  
* Derive views with **selectors** and memoization; reducers stay ignorant of derived data.

---

#### **7\) Concurrency & Distribution** {#7)-concurrency-&-distribution}

Reducers shine when scoped correctly — especially under concurrency and distributed conditions.

##### **Aggregate boundaries** {#aggregate-boundaries}

* Scope each reducer to a single aggregate key (e.g., `order_id`, `account_id`).  
* Within that boundary, events are processed serially, guaranteeing consistency.  
* Concurrency across aggregates is safe because each reducer instance is isolated.

##### **Serialization strategies** {#serialization-strategies}

* Actor mailbox: each aggregate reducer runs inside an actor, processing one event at a time.  
* DB row lock: lock per aggregate row while applying events, ensuring serial consistency.  
* Partition key: use Kafka/SQS/stream partitions keyed by aggregate ID to guarantee event ordering.

##### **Cross-aggregate coordination** {#cross-aggregate-coordination}

* Use a **saga** or workflow orchestrator to coordinate reducers across multiple aggregates.  
* Reducers themselves stay pure and local; the saga emits events to each aggregate’s reducer.

##### **Distribution patterns** {#distribution-patterns}

* Event sourcing with sharded logs: reducers replay from partitions assigned per aggregate.  
* Snapshotting: store snapshots alongside events to reduce replay cost.  
* Leader election: for reducers with global scope (rare), ensure only one active instance processes events.

##### **Invariants under concurrency** {#invariants-under-concurrency}

* Keep invariants scoped to the aggregate whenever possible.  
* For global invariants (e.g., unique usernames), enforce at command level with locks or consensus, not inside reducers.

---

#### **8\) Anti-patterns** {#8)-anti-patterns}

Things to avoid when designing reducers:

* Impure reducers: calling time, random, DB, or network APIs.  
* Hidden mutations: modifying global singletons, static caches, or shared state.  
* Transport leakage: mixing HTTP, headers, or gRPC details into domain events.  
* Side effects inside reducer: reducers must only return new state, not perform I/O.  
* Overloaded reducers: too many responsibilities; split into slices or layers.

---

#### **9\) Blueprints** {#9)-blueprints}

##### **Rust — Pure reducer \+ effects pattern** {#rust-—-pure-reducer-+-effects-pattern}

```rust
#[derive(Clone, Debug)]
pub enum OrderState { Draft, Pending { total: u64 }, Paid { txn: String, total: u64 }, Cancelled { reason: String } }

#[derive(Clone, Debug)]
pub enum OrderEvent { Submitted { total: u64 }, PaymentReceived { txn: String }, Cancelled { reason: String } }

pub fn reduce(state: &OrderState, ev: &OrderEvent) -> OrderState {
    use OrderEvent::*; use OrderState::*;
    match (state, ev) {
        (Draft, Submitted { total }) if *total > 0 => Pending { total: *total },
        (Pending { total }, PaymentReceived { txn }) => Paid { txn: txn.clone(), total: *total },
        (Draft, Cancelled { reason }) | (Pending { .. }, Cancelled { reason }) => Cancelled { reason: reason.clone() },
        (s, _) => s.clone(), // no-op for illegal transitions
    }
}

pub fn fold(mut s: OrderState, events: &[OrderEvent]) -> OrderState {
    for e in events { s = reduce(&s, e); }
    s
}
```

##### **Laravel (PHP) — Reducer-as-service and fold** {#laravel-(php)-—-reducer-as-service-and-fold}

```php
final class OrderReducer {
  public function reduce(OrderState $s, array $e): OrderState {
    return match([$s, $e['type'] ?? null]) {
      [OrderState::Draft, 'Submitted'] => $e['total'] > 0 ? OrderState::Pending : $s,
      [OrderState::Pending, 'PaymentReceived'] => OrderState::Paid,
      [OrderState::Draft, 'Cancelled'], [OrderState::Pending, 'Cancelled'] => OrderState::Cancelled,
      default => $s,
    };
  }

  public function fold(OrderState $s, array $events): OrderState {
    foreach ($events as $e) { $s = $this->reduce($s, $e); }
    return $s;
  }
}
```

##### **Nuxt/TypeScript — Slice reducers \+ combine** {#nuxt/typescript-—-slice-reducers-+-combine}

```ts
export type OrderState =
  | { tag: 'Draft' }
  | { tag: 'Pending'; total: number }
  | { tag: 'Paid'; total: number; txn: string }
  | { tag: 'Cancelled'; reason: string };

export type OrderEvent =
  | { type: 'Submitted'; total: number }
  | { type: 'PaymentReceived'; txn: string }
  | { type: 'Cancelled'; reason?: string };

export function reduce(s: OrderState, e: OrderEvent): OrderState {
  switch (s.tag) {
    case 'Draft':
      if (e.type === 'Submitted' && e.total > 0) return { tag: 'Pending', total: e.total };
      if (e.type === 'Cancelled') return { tag: 'Cancelled', reason: e.reason || 'unknown' };
      return s;
    case 'Pending':
      if (e.type === 'PaymentReceived') return { tag: 'Paid', total: s.total, txn: e.txn };
      if (e.type === 'Cancelled') return { tag: 'Cancelled', reason: e.reason || 'unknown' };
      return s;
    default:
      return s; // terminal states are idempotent
  }
}

export const fold = (s0: OrderState, events: OrderEvent[]) => events.reduce(reduce, s0);
```

---

#### **10\) Migration & Interoperability** {#10)-migration-&-interoperability}

* Wrapping legacy systems: translate DB mutations into domain events, fold them with reducers to cross-check state.  
* Event upcasting: evolve event schemas by mapping old events into new forms before reduction.  
* Time travel debugging: reducers \+ event logs allow replaying history step by step.  
* Golden log tests: keep canonical logs to ensure replay results remain consistent after refactors.

---

#### **11\) Checklist for shipping reducer-based features** {#11)-checklist-for-shipping-reducer-based-features}

---

#### **12\) Key Takeaways** {#12)-key-takeaways}

* Reducers are the simplest *truth-preserving core* for domain logic.  
* By keeping reducers pure, you enable replay, testing, and deterministic debugging.  
* Reducers compose naturally across slices and aggregates.  
* Paired with event sourcing, reducers provide infinite observability into past states.  
* Anti-patterns usually involve breaking purity or hiding side effects.

---

#### **13\) Next Steps** {#13)-next-steps}

* Apply reducers to a production flow (orders, payments, certificates).  
* Establish golden event logs for replay.  
* Extend reducer approach with property-based testing.  
* Pilot reducer \+ event sourcing in a small bounded context before rolling wider.

### 

### **Appendix I. Testing Strategies for Reducers** {#appendix-i.-testing-strategies-for-reducers}

#### **1\) Property-based testing** {#1)-property-based-testing}

* Generate random sequences of domain events.  
* Fold them through the reducer.  
* Assert invariants (e.g., totals never negative, illegal states unreachable).  
* Use shrinking to isolate minimal counterexamples.

#### **2\) Replay testing** {#2)-replay-testing}

* Record actual production event logs.  
* Replay them through the reducer to reconstruct state.  
* Compare against the persisted state (DB snapshot) for consistency.  
* Golden logs: preserve a canonical set of event logs to ensure deterministic replay across refactors.

#### **3\) Metamorphic testing** {#3)-metamorphic-testing}

* Insert no-op events and verify final state remains unchanged.  
* Reorder commutative events (e.g., adding items in a cart) and assert same final state.  
* Append redundant cancellations and ensure idempotency.

#### **4\) Simulation testing** {#4)-simulation-testing}

* Use deterministic schedulers to interleave events from concurrent aggregates.  
* Validate that per-aggregate invariants hold regardless of scheduling order.

#### **5\) Mutation testing** {#5)-mutation-testing}

* Apply mutations to reducer logic (invert conditions, alter comparisons).  
* Run property and replay tests to ensure they catch regressions.  
* Ensures your test suite is strong enough to detect subtle bugs.

