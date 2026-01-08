# Process Calculi: Formal Models of Concurrent Communication

**Purpose**: Mathematical frameworks for modeling, analyzing, and verifying concurrent systems through process composition and communication primitives.

**Core Insight**: Computation is interaction. Process calculi provide algebraic laws for reasoning about concurrent behavior, protocol correctness, and system composition.

---

## Table of Contents

1. [Foundational Concepts](#foundational-concepts)
2. [CSP: Communicating Sequential Processes](#csp-communicating-sequential-processes)
3. [π-Calculus: Mobile Processes](#π-calculus-mobile-processes)
4. [Session Types: Protocol Correctness](#session-types-protocol-correctness)
5. [Join Calculus: Multi-Way Synchronization](#join-calculus-multi-way-synchronization)
6. [Practical Applications](#practical-applications)
7. [Stack-Specific Implementations](#stack-specific-implementations)
8. [Integration Points](#integration-points)

---

## Foundational Concepts

### What Are Process Calculi?

Process calculi are algebraic systems for describing concurrent processes that interact through message passing or synchronization. They provide:

1. **Syntax**: Languages for describing processes and their composition
2. **Semantics**: Formal rules for how processes evolve over time
3. **Equivalences**: Notions of when two processes behave the same
4. **Analysis Tools**: Techniques for proving properties about systems

**Key Features**:
- Compositional reasoning (reason about parts, compose results)
- Formal semantics (unambiguous meaning)
- Equivalence relations (bisimulation, trace equivalence)
- Tool support (model checkers, theorem provers)

### Core Abstractions

```
Process ::= 0                    -- terminated/deadlock
         |  a.P                  -- action prefix (do a, then P)
         |  P | Q                -- parallel composition
         |  P + Q                -- choice
         |  (νx)P                -- restriction (new name/channel)
         |  !P                   -- replication (infinite copies)
```

### Behavioral Equivalences

**Trace Equivalence**: Two processes are trace equivalent if they have the same sets of observable action sequences.

**Bisimulation**: Stronger than trace equivalence. P ∼ Q if:
- Every action P can do, Q can match and reach a bisimilar state
- Every action Q can do, P can match and reach a bisimilar state
- Recursively for all derivatives

**Why Bisimulation Matters**: Captures observational equivalence - processes are interchangeable in any context.

---

## CSP: Communicating Sequential Processes

**Developed by**: Tony Hoare (1978)

**Key Idea**: Processes communicate through synchronous, named events. Composition through parallel operators with explicit synchronization sets.

### CSP Syntax and Semantics

```
Basic Processes:
STOP                     -- deadlock
SKIP                     -- successful termination
a → P                    -- event prefix
P □ Q                    -- external choice
P ⊓ Q                    -- internal choice
P ||| Q                  -- interleaving (no sync)
P || Q                   -- parallel with sync on shared events
P \ A                    -- hiding (internalize events in A)
```

### CSP Laws (Algebraic Properties)

**Choice Laws**:
```
P □ STOP = P                           (identity)
P □ Q = Q □ P                          (commutativity)
(P □ Q) □ R = P □ (Q □ R)             (associativity)
```

**Parallel Laws**:
```
P ||| STOP = P                         (identity)
P ||| Q = Q ||| P                      (commutativity)
(P ||| Q) ||| R = P ||| (Q ||| R)     (associativity)
```

**Hiding Laws**:
```
P \ ∅ = P                              (identity)
(P \ A) \ B = P \ (A ∪ B)             (composition)
```

### Deadlock Analysis

A process P is **deadlock-free** if every reachable state has at least one enabled event (or successfully terminates).

**Dining Philosophers Example**:
```
PHIL(i) = sitdown.i → pickup.i.left → pickup.i.right →
          eat.i → putdown.i.right → putdown.i.left →
          standup.i → PHIL(i)

FORK(i) = pickup.?.i → putdown.?.i → FORK(i)

SYSTEM = (PHIL(0) || ... || PHIL(4)) || (FORK(0) || ... || FORK(4))
```

**Deadlock occurs** when all philosophers pick up left fork simultaneously. **Solution**: Asymmetric philosopher (one picks right first).

### Refinement

Process Q **refines** P (written P ⊑ Q) if Q is more deterministic than P - every behavior of Q is a behavior of P.

**Refinement Orderings**:
- **Trace Refinement**: traces(Q) ⊆ traces(P)
- **Failures Refinement**: failures(Q) ⊆ failures(P)
- **Failures-Divergences Refinement**: Strongest practical notion

**Use Case**: Start with abstract specification P, refine to implementation Q, verify P ⊑ Q.

---

## π-Calculus: Mobile Processes

**Developed by**: Robin Milner, Joachim Parrow, David Walker (1992)

**Key Innovation**: Channel names can be sent as messages, enabling **mobile** process topologies.

### π-Calculus Syntax

```
P, Q ::= 0                    -- nil process
      |  x(y).P               -- input on x, bind y
      |  x̄⟨z⟩.P               -- output z on x
      |  P | Q                -- parallel composition
      |  (νx)P                -- restriction (fresh name)
      |  !P                   -- replication
      |  τ.P                  -- internal action
```

**Key Difference from CSP**: Channels are first-class values that can be transmitted.

### Scope Extrusion

When a restricted name is sent outside its scope, the restriction "moves" with it:

```
(νx)(x̄⟨y⟩.P | Q)  →  (νx)(P | Q{y/x})  if x fresh in Q
```

This models **capability passing** - sending a private channel grants access rights.

### Example: Secure Communication

```
-- Create private channel, send it to client
Server = (νk)( client⟨k⟩.k(msg).Process(msg) )

-- Receive private channel, use it to send message
Client = server(k).k̄⟨"secret"⟩.0
```

After channel k is transmitted, only Client and Server know it - providing secure communication.

### Bisimulation in π-Calculus

**Strong Bisimulation**: Relation R where P R Q implies:
- If P →^α P', then ∃Q': Q →^α Q' and P' R Q'
- Symmetric for Q

**Weak Bisimulation**: Abstract over internal (τ) actions:
- If P →^α P', then ∃Q': Q ⇒^α Q' and P' R Q'

Where ⇒ is the reflexive-transitive closure of →.

### π-Calculus Laws

**Structural Congruence**:
```
P | 0 ≡ P                          (identity)
P | Q ≡ Q | P                      (commutativity)
(P | Q) | R ≡ P | (Q | R)         (associativity)
(νx)0 ≡ 0                          (null restriction)
(νx)(νy)P ≡ (νy)(νx)P              (restriction swap)
(νx)(P | Q) ≡ P | (νx)Q  if x ∉ fn(P)  (scope extension)
```

**Communication Law**:
```
x̄⟨z⟩.P | x(y).Q  →  P | Q{z/y}
```

---

## Session Types: Protocol Correctness

**Developed by**: Kohei Honda, Vasco Vasconcelos, Nobuko Yoshida (1990s)

**Key Idea**: Types that describe communication protocols. Type checking ensures:
- **Communication Safety**: No type mismatches in messages
- **Session Fidelity**: Processes follow declared protocols
- **Deadlock Freedom**: Well-typed processes don't deadlock

### Session Type Syntax

```
S, T ::= !⟨U⟩.S              -- send value of type U, continue with S
      |  ?(U).S              -- receive value of type U, continue with S
      |  ⊕{l₁:S₁,...,lₙ:Sₙ}  -- select one of n labels
      |  &{l₁:S₁,...,lₙ:Sₙ}  -- offer branches
      |  μX.S                -- recursion
      |  end                 -- session termination
```

**Duality**: For every session type S, there's a dual S̄:
```
!⟨U⟩.S̄ = ?(U).S̄
?(U).S̄ = !⟨U⟩.S̄
⊕{lᵢ:Sᵢ}̄ = &{lᵢ:S̄ᵢ}
&{lᵢ:Sᵢ}̄ = ⊕{lᵢ:S̄ᵢ}
end̄ = end
```

### Example: Two-Buyer Protocol

```
-- Seller's session type
SellerType = ?(Title).!(Price).&{
  buy: ?(Contribution).!(Contribution).!⟨Address⟩.!⟨Date⟩.end,
  quit: end
}

-- Buyer1's type
Buyer1Type = !⟨Title⟩.?(Price).⊕{
  buy: !⟨Contribution⟩.?(Address).end,
  quit: end
}

-- Buyer2's type
Buyer2Type = ?(Price).&{
  buy: ?(Contribution).!⟨Contribution⟩.?(Date).end,
  quit: end
}
```

Type system ensures: Buyer1Type | Buyer2Type | SellerType is well-typed iff protocols match.

### Global Types and Projection

**Global Type** (choreography): Describes entire protocol from bird's eye view.

```
G = Buyer1 → Seller: Title.
    Seller → Buyer1: Price.
    Seller → Buyer2: Price.
    Buyer1 → Buyer2: {
      ok: Buyer2 → Seller: Contribution.
          Buyer1 → Seller: Contribution.
          Seller → Buyer1: Address.
          Seller → Buyer2: Date.
          end
      quit: end
    }
```

**Projection**: Extract local session type for each participant:
```
G ↾ Buyer1 = Buyer1Type
G ↾ Buyer2 = Buyer2Type
G ↾ Seller = SellerType
```

**Theorem (Projection Safety)**: If G is well-formed, then G ↾ p₁ | ... | G ↾ pₙ is deadlock-free.

### Multiparty Session Types (MPST)

Generalization to n parties. Provides:
- **Communication Safety**: Type-safe message passing
- **Progress**: Well-typed processes never get stuck
- **Deadlock Freedom**: No circular dependencies

**Tools**:
- **Scribble**: Protocol description language with endpoint projection
- **SessionJ**: Java implementation of session types
- **Links**: Functional language with native session types

---

## Join Calculus: Multi-Way Synchronization

**Developed by**: Cédric Fournet, Georges Gonthier (1996)

**Key Idea**: Replace channel-based communication with **join patterns** - synchronize on presence of messages across multiple channels.

### Join Calculus Syntax

```
P ::= 0                         -- nil
   |  x⟨v⟩                      -- asynchronous send
   |  def J in P                -- definition
   |  P | P                     -- parallel

J ::= x₁(y₁) ∧ ... ∧ xₙ(yₙ) ▷ P   -- join pattern
```

**Join Pattern**: Fires when all channels x₁,...,xₙ have messages. Consumes messages atomically.

### Example: Readers-Writers Lock

```
def
  -- Shared state
  idle() ∧ read_request(k) ▷ k⟨ok⟩ | readers(1)
  readers(n) ∧ read_request(k) ▷ k⟨ok⟩ | readers(n+1)
  readers(n) ∧ done_reading() ▷
    if n = 1 then idle() else readers(n-1)

  idle() ∧ write_request(k) ▷ k⟨ok⟩ | writing()
  writing() ∧ done_writing() ▷ idle()
in
  idle()  -- Initially idle
```

**Atomicity**: Join patterns fire atomically - no race conditions.

### Example: Barrier Synchronization

```
def barrier(n) =
  def
    wait() ∧ arrived(count) ▷
      if count + 1 = n then release(n)
      else arrived(count + 1)

    release(0) ▷ barrier(n)
    release(k) ▷ continue() | release(k-1)
  in
    arrived(0)
in
  barrier(4)

-- Usage: 4 processes call wait(), all proceed after 4th arrives
```

### Comparison with Other Calculi

| Feature | CSP | π-Calculus | Join Calculus |
|---------|-----|-----------|---------------|
| Communication | Synchronous | Async/Sync | Asynchronous |
| Mobility | No | Yes | Yes |
| Join Patterns | No | No | Yes |
| Syntax Complexity | Medium | Medium | Low |
| Tool Support | Excellent (FDR) | Good (ABC, MMC) | Limited |

**When to Use Join Calculus**:
- Multi-way synchronization patterns
- Implementing coordination abstractions (barriers, latches, semaphores)
- Chemical reaction metaphor (messages as molecules)

---

## Practical Applications

### 1. Protocol Verification with Session Types

**Problem**: Ensure client-server protocol correctness at compile time.

**Solution**:
```typescript
// Global protocol in Scribble
protocol HTTP {
  request(Method, Path) from Client to Server;
  choice at Server {
    response(200, Body) from Server to Client;
  } or {
    response(404, Error) from Server to Client;
  }
}

// TypeScript endpoint projections (conceptual)
type ClientSession =
  | { send: ["request", Method, Path], then: ClientReceive }

type ClientReceive =
  | { recv: ["response", 200, Body], then: End }
  | { recv: ["response", 404, Error], then: End }

type ServerSession =
  | { recv: ["request", Method, Path], then: ServerSend }

type ServerSend =
  | { send: ["response", 200, Body], then: End }
  | { send: ["response", 404, Error], then: End }
```

### 2. Deadlock Detection with CSP

**Problem**: Verify concurrent algorithm is deadlock-free.

**Solution**: Model in CSP, use FDR (Failures-Divergences Refinement) model checker:

```csp
-- Model each component as CSP process
Worker(id) = get_task?t → process.t → put_result!t → Worker(id)
Queue = put?x → (get!x → Queue [] put?y → put!x → put!y → Queue)

System = Worker(1) || Worker(2) || Queue

-- Check deadlock freedom
assert System :[deadlock free]
```

### 3. Mobile Code with π-Calculus

**Problem**: Model mobile agents that carry code to remote hosts.

**Solution**:
```
-- Agent carries continuation as channel
Agent(code) = (νreturn)(
  server⟨code, return⟩.
  return(result).
  processResult(result)
)

-- Server receives code channel, invokes it
Server = agent(code, return).
         (νx)(code⟨x⟩.0 | x(data).return̄⟨process(data)⟩.0)
```

**Real-World**: Basis for mobile code security models (ambient calculus, seal calculus).

### 4. Distributed Barriers with Join Calculus

**Problem**: Coordinate N distributed workers to synchronize at barrier points.

**Solution**:
```
def distributedBarrier(n) =
  def
    worker_arrive(id) ∧ count(k) ▷
      if k + 1 = n then releaseAll(n) | count(0)
      else count(k + 1)

    releaseAll(0) ▷ 0
    releaseAll(k) ▷ worker_release() | releaseAll(k - 1)
  in
    count(0)
```

---

## Stack-Specific Implementations

### Rust: CSP-Style Channels

Rust's channels are inspired by CSP with ownership semantics:

```rust
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;

/// CSP-style process: repeatedly receives, processes, sends
fn worker_process(
    input: Receiver<Task>,
    output: Sender<Result>
) {
    for task in input {
        let result = process_task(task);
        output.send(result).unwrap();
    }
}

/// Parallel composition with synchronization
fn parallel_pipeline() {
    let (tx1, rx1) = channel();
    let (tx2, rx2) = channel();
    let (tx3, rx3) = channel();

    // Stage 1
    thread::spawn(move || {
        worker_process(rx1, tx2);
    });

    // Stage 2
    thread::spawn(move || {
        worker_process(rx2, tx3);
    });

    // Producer
    thread::spawn(move || {
        for task in generate_tasks() {
            tx1.send(task).unwrap();
        }
    });

    // Consumer
    for result in rx3 {
        println!("Result: {:?}", result);
    }
}

/// CSP-style choice with select!
use crossbeam_channel::{select, Receiver};

fn choice_process(rx1: Receiver<A>, rx2: Receiver<B>) {
    loop {
        select! {
            recv(rx1) -> msg => {
                // Handle A
                if let Ok(a) = msg {
                    process_a(a);
                }
            }
            recv(rx2) -> msg => {
                // Handle B
                if let Ok(b) = msg {
                    process_b(b);
                }
            }
        }
    }
}
```

**Session Types in Rust**:
```rust
// Using session_types crate (conceptual)
use session_types::*;

// Protocol: !Int.?Bool.End
type ClientProto = Send<i32, Recv<bool, Eps>>;
type ServerProto = Recv<i32, Send<bool, Eps>>;

fn client(chan: Chan<(), ClientProto>) {
    let chan = chan.send(42);
    let (chan, response) = chan.recv();
    println!("Server responded: {}", response);
    chan.close();
}

fn server(chan: Chan<(), ServerProto>) {
    let (chan, value) = chan.recv();
    let result = value > 0;
    let chan = chan.send(result);
    chan.close();
}
```

### TypeScript: π-Calculus with Async Channels

```typescript
// Mobile channel implementation
class Channel<T> {
  private queue: T[] = [];
  private waiters: Array<(value: T) => void> = [];

  send(value: T): void {
    if (this.waiters.length > 0) {
      const waiter = this.waiters.shift()!;
      waiter(value);
    } else {
      this.queue.push(value);
    }
  }

  async receive(): Promise<T> {
    if (this.queue.length > 0) {
      return this.queue.shift()!;
    }
    return new Promise((resolve) => {
      this.waiters.push(resolve);
    });
  }
}

// π-calculus: sending channels as values
interface Message {
  replyTo: Channel<Response>;
  data: string;
}

async function server(inbox: Channel<Message>) {
  while (true) {
    const msg = await inbox.receive();
    const response = processData(msg.data);
    // Reply on the provided channel (mobility!)
    msg.replyTo.send(response);
  }
}

async function client(serverChan: Channel<Message>) {
  const replyChan = new Channel<Response>();
  serverChan.send({
    replyTo: replyChan,  // Send our own channel
    data: "request"
  });
  const response = await replyChan.receive();
  console.log("Got response:", response);
}

// Scope extrusion: private channel sent to server
async function secureClient(serverChan: Channel<Message>) {
  // (νk) - fresh channel
  const privateChan = new Channel<Response>();

  serverChan.send({
    replyTo: privateChan,
    data: "secret message"
  });

  // Only we can receive on privateChan
  const response = await privateChan.receive();
  return response;
}
```

**Session Types with TypeScript**:
```typescript
// Session type encoding
type Send<T, S> = { send(value: T): S };
type Recv<T, S> = { recv(): Promise<[T, S]> };
type End = { close(): void };

// Protocol: !String.?Int.End
type ClientSession = Send<string, Recv<number, End>>;

class SessionChannel<S> {
  constructor(private chan: Channel<any>) {}

  send<T, S2>(value: T): SessionChannel<S2> {
    this.chan.send(value);
    return new SessionChannel<S2>(this.chan);
  }

  async recv<T, S2>(): Promise<[T, SessionChannel<S2>]> {
    const value = await this.chan.recv();
    return [value, new SessionChannel<S2>(this.chan)];
  }

  close(): void {
    // Cleanup
  }
}

// Usage with type safety
async function typedClient(
  session: SessionChannel<Send<string, Recv<number, End>>>
) {
  const s1 = session.send("hello");
  const [response, s2] = await s1.recv();
  s2.close();
  return response;
}
```

### PHP: Actor-Based Process Patterns

PHP doesn't have native concurrency, but we can model process calculus patterns:

```php
<?php

namespace ProcessCalculi;

/**
 * CSP-style channel using Laravel queues
 */
class CSPChannel
{
    private string $channelName;

    public function __construct(string $name)
    {
        $this->channelName = $name;
    }

    public function send(mixed $message): void
    {
        // CSP synchronous send → enqueue and wait
        dispatch(new ChannelMessage(
            $this->channelName,
            $message
        ))->onQueue($this->channelName);
    }

    public function receive(): mixed
    {
        // Poll queue for message
        return Queue::pop($this->channelName);
    }
}

/**
 * π-calculus mobile channel pattern
 */
class MobileChannel
{
    private string $channelId;

    public function __construct()
    {
        $this->channelId = uniqid('chan_', true);
    }

    public function getId(): string
    {
        return $this->channelId;
    }

    public function send(mixed $message): void
    {
        Cache::put(
            "channel:{$this->channelId}",
            $message,
            now()->addMinutes(5)
        );

        // Notify via broadcast
        broadcast(new ChannelMessageEvent(
            $this->channelId,
            $message
        ));
    }

    public function receive(): mixed
    {
        // Wait for message on this channel
        return Cache::pull("channel:{$this->channelId}");
    }
}

/**
 * Join pattern implementation
 */
class JoinPattern
{
    private array $patterns = [];
    private array $pending = [];

    public function when(
        array $channels,
        callable $handler
    ): void {
        $patternId = md5(json_encode($channels));
        $this->patterns[$patternId] = [
            'channels' => $channels,
            'handler' => $handler,
        ];
    }

    public function send(string $channel, mixed $message): void
    {
        // Add to pending messages
        $this->pending[$channel][] = $message;

        // Try to fire patterns
        $this->tryFire();
    }

    private function tryFire(): void
    {
        foreach ($this->patterns as $pattern) {
            $channels = $pattern['channels'];

            // Check if all channels have messages
            $allPresent = true;
            $messages = [];

            foreach ($channels as $chan) {
                if (empty($this->pending[$chan])) {
                    $allPresent = false;
                    break;
                }
                $messages[$chan] = array_shift(
                    $this->pending[$chan]
                );
            }

            if ($allPresent) {
                // Fire the pattern
                ($pattern['handler'])(...array_values($messages));
            }
        }
    }
}

/**
 * Barrier synchronization with join patterns
 */
class Barrier
{
    private int $parties;
    private int $arrived = 0;
    private JoinPattern $join;
    private array $waiters = [];

    public function __construct(int $parties)
    {
        $this->parties = $parties;
        $this->join = new JoinPattern();

        // Join pattern: when N workers arrive, release all
        $this->join->when(
            ['arrive'],
            fn($workerId) => $this->handleArrival($workerId)
        );
    }

    public function await(string $workerId): void
    {
        $this->join->send('arrive', $workerId);
        $this->waiters[$workerId] = true;

        // Wait for release (polling in PHP)
        while ($this->waiters[$workerId] ?? false) {
            usleep(10000); // 10ms
        }
    }

    private function handleArrival(string $workerId): void
    {
        $this->arrived++;

        if ($this->arrived === $this->parties) {
            // Release all waiters
            foreach (array_keys($this->waiters) as $id) {
                unset($this->waiters[$id]);
            }
            $this->arrived = 0;
        }
    }
}

/**
 * Usage example: Parallel pipeline with CSP channels
 */
class ParallelPipeline
{
    public function run(): void
    {
        $stage1Output = new CSPChannel('stage1');
        $stage2Output = new CSPChannel('stage2');

        // Stage 1: Generate tasks
        dispatch(function () use ($stage1Output) {
            for ($i = 0; $i < 100; $i++) {
                $stage1Output->send(['id' => $i, 'data' => "task-$i"]);
            }
        });

        // Stage 2: Process tasks
        dispatch(function () use ($stage1Output, $stage2Output) {
            while ($task = $stage1Output->receive()) {
                $result = $this->processTask($task);
                $stage2Output->send($result);
            }
        });

        // Stage 3: Collect results
        dispatch(function () use ($stage2Output) {
            while ($result = $stage2Output->receive()) {
                Log::info("Result: " . json_encode($result));
            }
        });
    }
}
```

---

## Integration Points

### With Actor Model
- **Process calculi provide formal semantics** for actor message passing
- **π-calculus models** actor topology changes (dynamic supervision trees)
- **Session types ensure** actor protocol correctness

**Example**: Model Akka actor with session type for mailbox protocol.

### With FRP
- **CSP models** FRP networks as processes communicating on channels
- **Synchronous dataflow** is CSP with deterministic scheduling
- **Join patterns** model FRP merge and combine operators

**Example**: FRP switch operator is CSP external choice.

### With Type Theory
- **Session types** are linear types for communication channels
- **Dependent session types** allow protocol parameters
- **Refinement types** specify message invariants

**Example**: Session type `!{x: Int | x > 0}.End` ensures positive integers only.

### With Formal Verification
- **CSP model checking** (FDR) verifies deadlock freedom
- **π-calculus bisimulation** proves process equivalence
- **Session type checking** guarantees protocol adherence

**Example**: Verify distributed algorithm in CSP, implement in Go with channels.

### With Consistency Models
- **Process calculi model** distributed system message passing
- **Causal consistency** corresponds to happens-before in π-calculus
- **Eventual consistency** modeled by process convergence

**Example**: CRDT merge is join pattern on replica messages.

---

## Further Reading

### Foundational Papers
- Hoare (1978) - "Communicating Sequential Processes"
- Milner, Parrow, Walker (1992) - "A Calculus of Mobile Processes"
- Honda, Vasconcelos, Kubo (1998) - "Language Primitives and Type Discipline for Structured Communication-Based Programming"
- Fournet & Gonthier (1996) - "The Reflexive Chemical Abstract Machine and the Join-Calculus"

### Books
- Hoare - "Communicating Sequential Processes" (1985)
- Milner - "Communicating and Mobile Systems: the π-Calculus" (1999)
- Sangiorgi & Walker - "The π-Calculus: A Theory of Mobile Processes" (2001)
- Yoshida & Vasconcelos - "Language Primitives and Type Discipline for Structured Communication-Based Programming Revisited" (2007)

### Tools
- **FDR** (Failures-Divergences Refinement) - CSP model checker
- **Scribble** - Multiparty session type protocol language
- **APTE** - π-calculus tool for security protocol verification
- **ABC** (Abramsky's Bisimulation Checker) - π-calculus bisimulation

### Online Resources
- [CSP Tutorial](https://www.cs.ox.ac.uk/projects/concurrency-tools/)
- [π-calculus Primer](http://www.cs.unibo.it/~sangio/pi-calculus.html)
- [Session Types Tutorial](https://www.di.fc.ul.pt/~vv/papers/vasconcelos_session-types-tutorial.pdf)
- [Scribble Project](http://www.scribble.org/)

---

**End of Process Calculi Appendix**
