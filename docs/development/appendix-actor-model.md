# Appendix: Actor Model - Deep Theory & Practice

Purpose: Provide comprehensive coverage of the Actor Model computational paradigm, from Hewitt's original axioms through modern implementations like Erlang/OTP, Akka, and Orleans.

## Table of Contents
- Foundations (Hewitt 1973)
- Actor Axioms and Laws
- Mailbox Semantics
- Supervision & Fault Tolerance
- Location Transparency
- Behavioral Patterns
- Virtual Actors (Orleans Model)
- Testing Actors
- Anti-Patterns
- Stack-Specific Implementations

---

## 1. Foundations (Hewitt 1973)

### The Actor Axioms

An **Actor** is the fundamental unit of computation. Each actor can:
1. **Send** a finite number of messages to other actors
2. **Create** a finite number of new actors
3. **Designate** the behavior for the next message it receives

**No shared state. No global time. Only messages.**

```
Actor = (Address, Behavior, Mailbox)

Behavior : Message → (Actions, NextBehavior)

Actions = {
  Send(address, message),
  Create(behavior) → address,
  Become(newBehavior)
}
```

### Actor Laws

**1. Encapsulation Law**
- An actor's state is completely private
- The only way to interact is via messages
- No actor can directly read/write another's state

**2. Locality Law**
- An actor can only send to addresses it knows:
  - Addresses it was created with
  - Addresses received in messages
  - Addresses of actors it created

**3. Message Delivery Guarantees**
- At-most-once: messages may be lost
- At-least-once: messages may duplicate
- Exactly-once: requires additional protocols

**4. Fairness**
- Every message sent to an actor will eventually be delivered
- Every actor with messages will eventually process one

**5. No Shared State Theorem**
- Actors share nothing; all coordination via messages
- This eliminates data races by construction

### Operational Semantics

```
Configuration = (Actors, Messages-in-Transit)

Transition Rules:
─────────────────────────────────────────────────────
         a has message m, behavior b
         b(m) = (actions, b')
─────────────────────────────────────────────────────
(Actors ∪ {(a, b, [m|rest])}, M)
    →
(Actors' ∪ {(a, b', rest)} ∪ created, M ∪ sent)
```

### Comparison with Other Models

| Aspect | Actor Model | CSP | Shared Memory |
|--------|-------------|-----|---------------|
| Communication | Async messages | Sync channels | Direct access |
| State | Private per actor | Private per process | Shared |
| Identity | Named addresses | Anonymous channels | Memory locations |
| Blocking | Never (async) | On channel ops | On locks |
| Failure isolation | Natural | Possible | Difficult |

---

## 2. Mailbox Semantics

### Ordering Guarantees

**FIFO per sender-receiver pair:**
- If actor A sends m1 then m2 to actor B
- B receives m1 before m2

**No global ordering:**
- Messages from different senders can interleave arbitrarily
- This is fundamental, not a limitation

### Mailbox Strategies

```yaml
Unbounded:
  behavior: Accept all messages
  risk: Memory exhaustion
  use: Low-volume, trusted senders

Bounded with backpressure:
  behavior: Signal sender when full
  requires: Bidirectional protocol
  use: Flow-controlled systems

Bounded with drop:
  policies:
    - drop-newest: Reject incoming when full
    - drop-oldest: Evict oldest to make room
  use: Real-time, freshness matters

Priority:
  behavior: Process by priority class
  risk: Starvation of low priority
  use: Multi-tenant, QoS requirements

Stash:
  behavior: Defer messages for later
  use: State machines waiting for specific events
```

### Mailbox Implementation

```rust
// Bounded mailbox with backpressure
enum MailboxResult<M> {
    Accepted,
    Backpressure { retry_after: Duration },
    Rejected { reason: RejectReason },
}

trait Mailbox<M>: Send {
    fn enqueue(&self, msg: M) -> MailboxResult<M>;
    fn dequeue(&self) -> Option<M>;
    fn len(&self) -> usize;
    fn capacity(&self) -> usize;
}

// Priority mailbox
struct PriorityMailbox<M> {
    high: VecDeque<M>,
    normal: VecDeque<M>,
    low: VecDeque<M>,
}

impl<M: HasPriority> Mailbox<M> for PriorityMailbox<M> {
    fn dequeue(&self) -> Option<M> {
        self.high.pop_front()
            .or_else(|| self.normal.pop_front())
            .or_else(|| self.low.pop_front())
    }
}
```

```typescript
// TypeScript/Node.js actor mailbox
class ActorMailbox<M> {
    private queue: M[] = [];
    private maxSize: number;
    private processing = false;

    constructor(maxSize: number = Infinity) {
        this.maxSize = maxSize;
    }

    async enqueue(msg: M): Promise<MailboxResult> {
        if (this.queue.length >= this.maxSize) {
            return { status: 'backpressure', retryAfter: 100 };
        }
        this.queue.push(msg);
        this.process();
        return { status: 'accepted' };
    }

    private async process(): Promise<void> {
        if (this.processing) return;
        this.processing = true;

        while (this.queue.length > 0) {
            const msg = this.queue.shift()!;
            await this.actor.receive(msg);
        }

        this.processing = false;
    }
}
```

```php
// PHP actor mailbox (Laravel Queue-based)
class ActorMailbox
{
    private string $actorId;
    private int $maxSize;

    public function __construct(string $actorId, int $maxSize = 1000)
    {
        $this->actorId = $actorId;
        $this->maxSize = $maxSize;
    }

    public function enqueue(ActorMessage $message): MailboxResult
    {
        $queueSize = Cache::get("mailbox:{$this->actorId}:size", 0);

        if ($queueSize >= $this->maxSize) {
            return MailboxResult::backpressure(retryAfter: 1000);
        }

        dispatch(new ProcessActorMessage($this->actorId, $message))
            ->onQueue("actor:{$this->actorId}");

        Cache::increment("mailbox:{$this->actorId}:size");

        return MailboxResult::accepted();
    }

    public function dequeue(): ?ActorMessage
    {
        Cache::decrement("mailbox:{$this->actorId}:size");
        // Message retrieved by Laravel's queue worker
    }
}
```

---

## 3. Supervision & Fault Tolerance

### The "Let It Crash" Philosophy

**Principle:** Don't try to handle every error locally. Let actors crash and let supervisors decide recovery strategy.

**Why this works:**
- Errors are isolated to single actors
- No corrupted shared state to clean up
- Supervisor has broader context for decisions
- Simpler actor code (happy path only)

### Supervision Strategies

```yaml
One-for-One:
  behavior: Restart only the failed child
  use: Independent children

All-for-One:
  behavior: Restart all children on any failure
  use: Tightly coupled children, shared assumptions

Rest-for-One:
  behavior: Restart failed child and all started after it
  use: Sequential dependencies

Escalate:
  behavior: Pass failure to parent supervisor
  use: Cannot handle locally

Stop:
  behavior: Terminate failed child permanently
  use: Fatal errors, graceful degradation
```

### Supervision Tree Structure

```
                    ┌─────────────┐
                    │   Guardian  │ (root supervisor)
                    └──────┬──────┘
                           │
          ┌────────────────┼────────────────┐
          │                │                │
    ┌─────▼─────┐    ┌─────▼─────┐    ┌─────▼─────┐
    │ DB Super  │    │ HTTP Super│    │ Worker    │
    │(all-for-1)│    │(one-for-1)│    │  Pool     │
    └─────┬─────┘    └─────┬─────┘    └─────┬─────┘
          │                │                │
     ┌────┴────┐      ┌────┴────┐      ┌────┴────┐
     │         │      │         │      │         │
   Writer   Reader  Handler  Handler  Worker  Worker
```

### Restart Policies

```rust
// Rust
struct RestartPolicy {
    max_restarts: u32,
    within: Duration,
    backoff: BackoffStrategy,
}

enum BackoffStrategy {
    Immediate,
    Fixed(Duration),
    Exponential { base: Duration, max: Duration, jitter: bool },
}

fn decide(failure: &Failure, policy: &RestartPolicy, history: &[Restart]) -> Decision {
    let recent = history.iter()
        .filter(|r| r.timestamp > Instant::now() - policy.within)
        .count();

    if recent >= policy.max_restarts as usize {
        Decision::Escalate
    } else {
        Decision::Restart {
            delay: policy.backoff.next_delay(recent)
        }
    }
}
```

```typescript
// TypeScript
interface RestartPolicy {
    maxRestarts: number;
    within: number; // milliseconds
    backoff: BackoffStrategy;
}

type BackoffStrategy =
    | { type: 'immediate' }
    | { type: 'fixed'; delay: number }
    | { type: 'exponential'; base: number; max: number; jitter: boolean };

class Supervisor {
    decide(failure: Failure, policy: RestartPolicy, history: Restart[]): Decision {
        const cutoff = Date.now() - policy.within;
        const recent = history.filter(r => r.timestamp > cutoff).length;

        if (recent >= policy.maxRestarts) {
            return { type: 'escalate' };
        }

        return {
            type: 'restart',
            delay: this.calculateBackoff(policy.backoff, recent)
        };
    }
}
```

```php
// PHP/Laravel
class RestartPolicy
{
    public function __construct(
        public int $maxRestarts,
        public int $withinSeconds,
        public BackoffStrategy $backoff
    ) {}
}

enum BackoffStrategy
{
    case Immediate;
    case Fixed;
    case Exponential;
}

class Supervisor
{
    public function decide(
        Failure $failure,
        RestartPolicy $policy,
        array $history
    ): Decision {
        $cutoff = now()->subSeconds($policy->withinSeconds);
        $recent = collect($history)
            ->filter(fn($r) => $r->timestamp->isAfter($cutoff))
            ->count();

        if ($recent >= $policy->maxRestarts) {
            return Decision::escalate();
        }

        return Decision::restart(
            delay: $this->calculateBackoff($policy->backoff, $recent)
        );
    }
}
```

### Error Kernel Pattern

```
Principle: Keep critical state in supervisors, not workers

┌─────────────────────────────────────┐
│ Supervisor (holds critical state)   │
│ - Account balances                  │
│ - Configuration                     │
│ - Recovery checkpoints              │
└──────────────┬──────────────────────┘
               │
    ┌──────────┼──────────┐
    │          │          │
┌───▼───┐  ┌───▼───┐  ┌───▼───┐
│Worker │  │Worker │  │Worker │
│(stateless or     │  │       │
│ recoverable)     │  │       │
└───────┘  └───────┘  └───────┘

Workers can crash freely; supervisor rebuilds them
with last known good state.
```

---

## 4. Location Transparency

### The Address Abstraction

```
ActorRef = opaque handle to an actor

Properties:
- ActorRef is serializable (can send over network)
- ActorRef hides location (local or remote)
- ActorRef is stable (survives restarts via supervision)
- ActorRef comparisons work across locations
```

### Distribution Strategies

```yaml
Cluster Sharding:
  concept: Partition actors by key across nodes
  routing: Hash(entity_id) → node → actor
  migration: Actors can move between nodes
  use: Stateful entities (users, orders, sessions)

Cluster Singleton:
  concept: Exactly one instance in cluster
  election: Oldest node runs singleton
  failover: Migrates on node failure
  use: Global coordinators, schedulers

Cluster-Aware Routers:
  strategies:
    - round-robin: Distribute evenly
    - consistent-hashing: By message key
    - adaptive: Based on load metrics
    - broadcast: To all nodes
```

---

## 5. Behavioral Patterns

### Finite State Machine Actor

```rust
// Rust
enum OrderState {
    Draft,
    Submitted { total: Money },
    Paid { txn_id: String },
    Shipped { tracking: String },
    Completed,
    Cancelled { reason: String },
}

impl Actor for OrderActor {
    fn receive(&mut self, msg: OrderCommand, ctx: &mut Context) {
        let (new_state, effects) = match (&self.state, msg) {
            (Draft, Submit { items }) => {
                let total = calculate_total(&items);
                (Submitted { total }, vec![EmitEvent(OrderSubmitted)])
            }
            (Submitted { total }, Pay { payment }) => {
                (Paid { txn_id: payment.txn_id }, vec![
                    EmitEvent(OrderPaid),
                    Send(inventory_actor, Reserve { items })
                ])
            }
            (state, cmd) => {
                ctx.log.warn("Invalid transition");
                (state.clone(), vec![])
            }
        };

        self.state = new_state;
        for effect in effects {
            effect.execute(ctx);
        }
    }
}
```

```typescript
// TypeScript
type OrderState =
    | { type: 'draft' }
    | { type: 'submitted'; total: Money }
    | { type: 'paid'; txnId: string }
    | { type: 'shipped'; tracking: string }
    | { type: 'completed' }
    | { type: 'cancelled'; reason: string };

class OrderActor implements Actor<OrderCommand> {
    private state: OrderState = { type: 'draft' };

    async receive(msg: OrderCommand, ctx: Context): Promise<void> {
        const [newState, effects] = this.transition(this.state, msg, ctx);
        this.state = newState;
        for (const effect of effects) {
            await effect.execute(ctx);
        }
    }

    private transition(
        state: OrderState,
        msg: OrderCommand,
        ctx: Context
    ): [OrderState, Effect[]] {
        if (state.type === 'draft' && msg.type === 'submit') {
            const total = calculateTotal(msg.items);
            return [
                { type: 'submitted', total },
                [emitEvent({ type: 'orderSubmitted', total })]
            ];
        }
        // ... other transitions
        return [state, []];
    }
}
```

```php
// PHP/Laravel
enum OrderState
{
    case Draft;
    case Submitted;
    case Paid;
    case Shipped;
    case Completed;
    case Cancelled;
}

class OrderActor
{
    private OrderState $state = OrderState::Draft;
    private array $context = [];

    public function receive(OrderCommand $msg, Context $ctx): void
    {
        [$newState, $effects] = match ([$this->state, $msg::class]) {
            [OrderState::Draft, SubmitCommand::class] => [
                OrderState::Submitted,
                [
                    new EmitEvent(new OrderSubmitted($msg->items)),
                    new UpdateContext(['total' => $this->calculateTotal($msg->items)])
                ]
            ],
            [OrderState::Submitted, PayCommand::class] => [
                OrderState::Paid,
                [
                    new EmitEvent(new OrderPaid($msg->payment)),
                    new SendMessage(
                        $this->inventoryActor,
                        new ReserveCommand($this->context['items'])
                    )
                ]
            ],
            default => [
                $this->state,
                [new LogWarning('Invalid transition')]
            ]
        };

        $this->state = $newState;
        foreach ($effects as $effect) {
            $effect->execute($ctx);
        }
    }
}
```

---

## 6. Virtual Actors (Orleans Model)

### Automatic Lifecycle Management

```
Traditional Actors:
  - Explicit creation: system.actorOf(Props(...))
  - Explicit destruction: actor.stop()
  - Manual clustering/sharding

Virtual Actors (Grains):
  - Implicit activation: first message creates
  - Automatic deactivation: idle timeout
  - Transparent distribution
  - Single activation guarantee (per ID)
```

### Turn-Based Concurrency

```
Grain processing message M1:
  → Starts processing
  → await database.Read()  ← TURN ENDS
  → Database returns       ← TURN RESUMES
  → Continues M1

Interleaving is possible at await points!
Need to re-validate state after awaits.
```

---

## 7. Testing Actors

### Deterministic Testing

```rust
// Rust
#[test]
fn order_actor_processes_submission() {
    let mut kit = TestKit::new();
    let order = kit.spawn(OrderActor::new());

    order.tell(OrderCommand::Submit { items: vec![item1, item2] });

    kit.expect_state(&order, |state| {
        matches!(state, OrderState::Submitted { total } if total == Money::new(100))
    });
}
```

```typescript
// TypeScript
describe('OrderActor', () => {
    it('processes submission', async () => {
        const kit = new TestKit();
        const order = kit.spawn(new OrderActor());

        order.tell({ type: 'submit', items: [item1, item2] });

        await kit.expectState(order, (state) =>
            state.type === 'submitted' && state.total === 100
        );
    });
});
```

```php
// PHP/Laravel
class OrderActorTest extends TestCase
{
    public function test_processes_submission(): void
    {
        $kit = new TestKit();
        $order = $kit->spawn(new OrderActor());

        $order->tell(new SubmitCommand([$item1, $item2]));

        $kit->expectState($order, fn($state) =>
            $state === OrderState::Submitted
        );
    }
}
```

---

## 8. Anti-Patterns

### Blocking in Actor

```rust
// BAD
fn receive(&mut self, msg: Message, ctx: &mut Context) {
    std::thread::sleep(Duration::from_secs(10));  // BLOCKS!
}

// GOOD
fn receive(&mut self, msg: Message, ctx: &mut Context) {
    ctx.spawn_blocking(move || {
        let data = expensive_computation();
        self_ref.tell(ComputationResult(data));
    });
}
```

### Shared Mutable State

```rust
// BAD
let shared = Arc::new(RwLock::new(HashMap::new()));
let actor1 = spawn(Actor1 { cache: shared.clone() });

// GOOD
let cache_actor = spawn(CacheActor::new());
let actor1 = spawn(Actor1 { cache: cache_actor.clone() });
```

---

## 9. When to Use Actors

```yaml
Good fit:
  - Stateful entities with identity
  - Natural concurrency boundaries
  - Fault isolation requirements
  - Location-transparent distribution
  - Long-lived message-driven processes

Poor fit:
  - Pure data transformation (use streams)
  - Request-response with no state (use functions)
  - Tight latency requirements
  - Simple CRUD (use database directly)
  - Synchronous workflows
```

---

## Integration Points

- **→ Process Calculi**: Formal semantics (CSP, π-calculus)
- **→ Consensus**: Distributed actor coordination
- **→ Streams**: Reactive actors, backpressure
- **→ FSM**: Actor state machines

---

## References

### Rust
- **Actix**: High-performance actor framework
- **Bastion**: Fault-tolerant runtime inspired by Erlang
- **Tokio Actors**: Lightweight actor patterns with tokio

### TypeScript/JavaScript
- **Comedy**: Actor framework for Node.js
- **nact**: Lightweight actors inspired by Akka
- **Actor-js**: Simple actor implementation

### PHP
- **Laravel Queues**: Actor-like job processing
- **Amp**: Async concurrency primitives
- **ReactPHP**: Event-driven, non-blocking I/O

### Academic
- Hewitt et al. (1973): "A Universal Modular ACTOR Formalism for Artificial Intelligence"
- Agha (1986): "Actors: A Model of Concurrent Computation in Distributed Systems"
- Armstrong (2003): "Making reliable distributed systems in the presence of software errors" (Erlang/OTP)

*End of Appendix: Actor Model*
