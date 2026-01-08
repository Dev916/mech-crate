# Appendix K: Algebraic Effects & Optics

Purpose  
Give reducers, FSMs, and services a **clear, uniform way** to express:
- fine-grained updates to nested immutable state (optics), and
- “things that must happen in the real world” (algebraic effects)

…without smuggling IO or messy mutation into the domain core. Pairs directly with:

- Reducers & FSM appendix   
- Business logic placement (domain vs infra)   
- Rust/Laravel idioms and ports & adapters   

---

## K1. Non‑negotiables

1. **No raw IO inside reducers or state machines.**  
   Domain logic returns new state + *descriptions* of side effects, not side effects themselves.   

2. **All nested state updates use optics**, not hand-rolled copying:
   - Lenses for product fields
   - Prisms for sum/enum variants
   - Traversals for collections

3. **Every effect belongs to a small effect algebra**:  
   A well‑named trait/interface/port set, not “just call the SDK/client inline.”

4. **Testing uses alternative interpreters**:
   - Fakes / in‑memory
   - Record & replay
   - Cost/simulation interpreters

---

## K2. Concepts at a glance

### K2.1 Optics

- **Lens\<S, A\>**
  - Focus: a field `A` inside a product `S`
  - Ops: `get(s) -> A`, `set(s, a) -> S`, `over(s, f: A -> A) -> S`
- **Prism\<S, A\>**
  - Focus: a variant of a sum/enum `S`
  - Ops: `preview(s) -> Option<A>`, `review(a) -> S`
- **Traversal\<S, A\>**
  - Focus: many `A` values inside `S` (e.g., list items)
  - Ops: `over_all(s, f: A -> A) -> S`

Think: *optics = composable paths into immutable data*.

### K2.2 Algebraic effects

- **Effect algebra**: a small interface/trait describing what the program *may* ask of the world:
  - `Clock`: `now() -> Instant`
  - `Logger`: `info(msg)`, `warn(msg)`, …
  - `Payments`: `authorize(amount, card) -> Result<Authorization, Error>`
- **Interpreter/handler**: an implementation of that algebra:
  - Real infra (HTTP, DB, queues)
  - In‑memory test double
  - “Dry run” simulator

Effects become **operations you can reason about**, not scattered ad‑hoc calls.

---

## K3. Optics + Reducers + FSMs

Reducers and FSMs are already your “mathematical heart.”   
Optics make nested updates **mechanical and composable**.

### K3.1 Minimal optics API (language‑agnostic)

Define a minimal set of building blocks:

```ts
// Conceptual; adjust syntax per language
type Lens<S, A> = {
  get: (s: S) => A;
  set: (s: S, a: A) => S;
  over: (s: S, f: (a: A) => A) => S;
};

type Prism<S, A> = {
  preview: (s: S) => A | null;
  review: (a: A) => S;
};

type Traversal<S, A> = {
  overAll: (s: S, f: (a: A) => A) => S;
};
```


Then provide a small “path builder” per stack:
	•	Rust: helper macros for struct fields and enum variants.
	•	Laravel/PHP: Lens::field('order', 'total'), Lens::index('items').
	•	Nuxt/TS: type‑safe path<'order.items[0].price'> builder or hand‑written lenses.

### K3.2 Usage in reducers

Instead of:

```ts
// Pseudocode: manual nested update
return {
  ...state,
  order: {
    ...state.order,
    items: state.order.items.map((item, i) =>
      i === idx ? { ...item, price: newPrice } : item
    )
  }
};
```

Use a traversal:


```ts
const itemAt = itemTraversal(index); // Traversal<Order, Item>
return itemAt.over(state, item =>
  item.id === targetId ? { ...item, price: newPrice } : item
);
```

Rules:
	•	Reducers never manually copy nested structures.
	•	FSM transition tables talk in terms of optics:
“order.items.*.status set to Cancelled on TIMEOUT.”


### K3.3 Optics + FSM appendix integration


Extend the FSM checklist:  ￼
	•	“Where context is nested, document and use a lens/traversal path for each transition.”
	•	Example column in transition table:

|___________________________________________________________________________________|
| From | Event    | Guard       | To       | Action                                 |
|______|__________|_____________|__________|________________________________________|
| Paid | REFUNDOK | amount <= ? | Refunded | items[*].status = Refunded (traversal) |
|______|__________|_____________|__________|________________________________________|



## K4. Effect algebras & interpreters

### K4.1 Shape of an effect algebra

Rust (core/ports)

```rust
pub trait Clock {
    fn now(&self) -> OffsetDateTime;
}

pub trait Payments {
    fn authorize(&self, cmd: AuthorizeCmd) -> Result<AuthOk, PaymentErr>;
}

pub trait Logger {
    fn info(&self, msg: &str);
    fn warn(&self, msg: &str);
}
```

Laravel (app/Domain/Ports)


```php
interface Clock {
    public function now(): DateTimeImmutable;
}

interface Payments {
    public function authorize(AuthorizeCmd $cmd): Either/*<PaymentError, AuthOk>*/;
}

interface Logger {
    public function info(string $msg): void;
}
```

TypeScript / Node (fp-ts style)


```ts
interface Clock {
  now: () => Date;
}

interface Payments {
  authorize: (cmd: AuthorizeCmd) => TE.TaskEither<PaymentError, AuthOk>;
}

interface Logger {
  info: (msg: string) => T.Task<void>;
}interface Clock {
  now: () => Date;
}

interface Payments {
  authorize: (cmd: AuthorizeCmd) => TE.TaskEither<PaymentError, AuthOk>;
}

interface Logger {
  info: (msg: string) => T.Task<void>;
}
```

### K4.2 Program shape: capabilities in, pure-ish logic out


A service or saga receives capabilities (instances of those algebras) instead of concrete clients:

```rust
pub struct Env<'a> {
    pub clock: &'a dyn Clock,
    pub payments: &'a dyn Payments,
    pub logger: &'a dyn Logger,
}

pub async fn capture_payment(env: &Env, cmd: CaptureCmd) -> Result<CaptureResult, CaptureErr> {
    env.logger.info("capture_payment.start");
    let auth = env.payments.authorize(cmd.to_authorize_cmd()).await?;
    // pure reducer + optics for state update
    Ok(CaptureResult::from(auth))
}
```
Application/infra layers assemble Env with real implementations; tests assemble it with fakes.

## K5. Effect programs as data (optional but powerful)

For more control (and better LLM guidance), represent workflows as data instead of “just run the effect”:


```ts
type EffectOp =
  | { tag: 'Log'; level: 'info' | 'warn'; msg: string }
  | { tag: 'Now' }
  | { tag: 'Authorize'; cmd: AuthorizeCmd };

type Program<A> =
  | { tag: 'Pure'; value: A }
  | { tag: 'Bind'; op: EffectOp; k: (result: any) => Program<A> };
```

Then you write domain workflows that build a Program, and you provide interpreters:
	•	runReal(program) – touches DB/HTTP/etc.
	•	runTest(program) – uses in‑memory structures and captures logs.
	•	runCost(program) – estimates cost, time, risk.

This is a generalization of your existing interpreter pattern / workflow engines, but as a reusable primitive.

LLM angle: ask models to produce programs in this DSL, never raw imperative steps.


## K6. Testing & simulation with effects

Patterns:
	1.	Record–replay interpreter
	•	Interpreter records EffectOps instead of executing them.
	•	Use for:
	•	“What would happen if we run this saga?”
	•	Documenting expected sequences as golden test fixtures.
	2.	Fault injection interpreter
	•	Same API, but randomly injects failures on effect boundaries:
	•	Payments time out
	•	Clock jumps ahead
	•	Ideal for property‑based tests around sagas/retries.
	3.	Tracing interpreter
	•	Wraps a real interpreter and pushes events into logs/metrics (op_count, “per‑effect latency”).


## K7. Language‑specific checklists


### Rust
	•	All non‑deterministic concerns (time, UUID, rand, network) live in traits under core/ports.
	•	Reducers and FSMs accept plain values and never own a reference to an effect trait.
	•	Env structs group ports/capabilities for specific workflows (not one mega‑env).
	•	Tests define fake structs implementing the same traits; use them to assert sequence/order.

### Laravel / PHP
	•	Domain never sees Eloquent models; ports are defined as interfaces under Domain/Ports.
	•	Controllers/jobs build “effect envs” from Laravel services/container.
	•	Wrap external libraries (Stripe SDK, HTTP clients) behind effect ports.

### Nuxt / TypeScript
	•	For Node services, define small capability interfaces and pass them down (or use ReaderTaskEither).
	•	For UI stores (Pinia/composables), reducers + optics handle state; effects live in composables that inject e.g. http, clock.
	•	Where you already have RAG utilities, treat the RAG client as an effect algebra (search, pin, upsertDoc)


## K8. LLM guidance hooks


When wiring MCP/RAG for this repo:
	•	Pin a short “Effects & Optics Ground Rules” doc:
	•	“Use the Clock, Logger, Payments ports, not raw SDKs.”
	•	“Use Lens/Traversal helpers for nested state.”
	•	Add to review prompts:
	•	❓ “Does this code introduce new side effects instead of reusing ports?”
	•	❓ “Are nested updates expressed via optics?”

This turns the appendix into enforcement, not just philosophy.