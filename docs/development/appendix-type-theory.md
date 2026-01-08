# Type Theory: Types as Specifications and Proofs

**Purpose**: Mathematical framework for typing systems in programming languages, connecting types to logic, enabling compile-time verification and safer software construction.

**Core Insight**: **Types are propositions, programs are proofs** (Curry-Howard correspondence). Rich type systems enable expressing and verifying properties at compile time.

---

## Table of Contents

1. [Foundational Concepts](#foundational-concepts)
2. [Curry-Howard Correspondence](#curry-howard-correspondence)
3. [Dependent Types](#dependent-types)
4. [Linear Types](#linear-types)
5. [Session Types](#session-types)
6. [Refinement Types](#refinement-types)
7. [Effect Systems](#effect-systems)
8. [Stack-Specific Implementations](#stack-specific-implementations)
9. [Integration Points](#integration-points)

---

## Foundational Concepts

### What is Type Theory?

**Type Theory**: Formal system where every term has a type, providing foundation for:
- Programming language semantics
- Proof assistants (Coq, Lean, Agda)
- Verified software development

**Key Ideas**:
- **Types classify terms**: `5 : Int`, `"hello" : String`
- **Functions have types**: `f : A → B`
- **Types can depend on values**: `Vec(n) : Type` (vector of length n)

### Simply Typed Lambda Calculus (STLC)

**Syntax**:
```
Types:    τ ::= Base | τ₁ → τ₂
Terms:    e ::= x | λx:τ. e | e₁ e₂
```

**Typing Rules**:
```
Γ ⊢ x : τ  if (x:τ) ∈ Γ

Γ, x:τ₁ ⊢ e : τ₂
─────────────────────  (λ)
Γ ⊢ (λx:τ₁. e) : τ₁ → τ₂

Γ ⊢ e₁ : τ₁ → τ₂    Γ ⊢ e₂ : τ₁
──────────────────────────────────  (App)
Γ ⊢ e₁ e₂ : τ₂
```

**Example**:
```
id = λx:Int. x      :  Int → Int
const = λx:Int. λy:Bool. x  :  Int → Bool → Int
```

### Polymorphism

**Parametric Polymorphism** (Generics):
```
id : ∀α. α → α
id = Λα. λx:α. x

id[Int] 5 = 5
id[String] "hello" = "hello"
```

**System F**: STLC + ∀ quantification.

---

## Curry-Howard Correspondence

**Fundamental Insight**: Deep correspondence between logic and programming.

### The Correspondence

| Logic | Type Theory | Programming |
|-------|-------------|-------------|
| Proposition | Type | Specification |
| Proof | Term | Program |
| Axiom | Constant | Primitive |
| Implication (A ⇒ B) | Function type (A → B) | Function |
| Conjunction (A ∧ B) | Product type (A × B) | Pair/Tuple |
| Disjunction (A ∨ B) | Sum type (A + B) | Either/Enum |
| True | Unit type (⊤) | Unit/void |
| False | Empty type (⊥) | Never/absurd |
| ∀x. P(x) | Dependent product (Π) | Polymorphism |
| ∃x. P(x) | Dependent sum (Σ) | Existential type |

### Example: De Morgan's Law

**Logic**: ¬(A ∧ B) ⇒ ¬A ∨ ¬B

**Type Theory**: `(A × B → ⊥) → (A → ⊥) + (B → ⊥)`

**Program** (in Coq-like syntax):
```coq
Definition de_morgan :
  forall A B : Type,
  (A * B -> False) -> (A -> False) + (B -> False).
Proof.
  intros A B H.
  left.  (* Choose ¬A *)
  intro a.
  apply H.
  (* Need to construct A * B from just A - stuck! *)
  (* Actually, this direction of De Morgan doesn't hold constructively *)
Abort.
```

**Constructive Logic**: Not all classical tautologies have computational content.

### Programs as Proofs

**Theorem**: For all n, if n > 0, then there exists m such that n = m + 1.

**Type**:
```
Π(n : Nat). (n > 0) → Σ(m : Nat). (n = m + 1)
```

**Proof/Program**:
```agda
theorem : (n : Nat) → (n > 0) → Σ Nat (λ m → n ≡ m + 1)
theorem (suc m) _ = m , refl
```

---

## Dependent Types

**Key Idea**: Types can depend on values, enabling precise specifications.

### Dependent Function Types (Π Types)

**Syntax**: `Π(x : A). B(x)`

**Meaning**: Function from `x : A` to `B(x)`, where B's type depends on x's value.

**Example**:
```agda
-- Vector: length-indexed list
data Vec (A : Type) : Nat → Type where
  []  : Vec A 0
  _::_ : {n : Nat} → A → Vec A n → Vec A (n + 1)

-- Type of append depends on input lengths
append : {A : Type} → {m n : Nat} →
         Vec A m → Vec A n → Vec A (m + n)
append [] ys = ys
append (x :: xs) ys = x :: append xs ys
```

**Benefit**: `append` guaranteed to return vector of correct length (checked by type system).

### Dependent Pair Types (Σ Types)

**Syntax**: `Σ(x : A). B(x)`

**Meaning**: Pair where second component's type depends on first component's value.

**Example**:
```agda
-- Σ type: existential package
data Σ (A : Type) (B : A → Type) : Type where
  _,_ : (x : A) → B x → Σ A B

-- Type-indexed data with its type
DynamicValue : Type
DynamicValue = Σ Type (λ T → T)

-- Examples
five : DynamicValue
five = (Nat , 5)

hello : DynamicValue
hello = (String , "hello")
```

### Equality Types

**Propositional Equality**:
```agda
data _≡_ {A : Type} (x : A) : A → Type where
  refl : x ≡ x

-- Theorem: append is associative
append-assoc : {A : Type} {l m n : Nat}
               (xs : Vec A l) (ys : Vec A m) (zs : Vec A n) →
               append xs (append ys zs) ≡ append (append xs ys) zs
append-assoc [] ys zs = refl
append-assoc (x :: xs) ys zs = cong (x ::_) (append-assoc xs ys zs)
```

**Proof by Induction**: Structural recursion on xs.

---

## Linear Types

**Key Idea**: Track resource usage. Linear value must be used exactly once.

### Linear Type System

**Types**:
- **Linear**: `A` (must use exactly once)
- **Unrestricted**: `!A` (can duplicate/discard)

**Typing Rules**:
```
Γ, x:A ⊢ x : A       (use exactly once)

Γ, x:!A ⊢ x : !A     (use multiple times)
Γ, x:!A ⊢ e : B
─────────────────     (discard)
Γ ⊢ e : B

Γ, x:!A, y:!A ⊢ e : B
──────────────────────  (duplicate)
Γ, x:!A ⊢ e[x/y] : B
```

### Example: File Handles

```
FileHandle : Linear Type

open : String → FileHandle
read : FileHandle → (String × FileHandle)
close : FileHandle → ()

-- Correct usage:
let h = open("file.txt") in
let (contents, h') = read(h) in
close(h')

-- ERROR: h used twice
let h = open("file.txt") in
let (c1, h') = read(h) in
let (c2, h'') = read(h) in  -- h already consumed!
close(h'')

-- ERROR: h not used (resource leak)
let h = open("file.txt") in
()  -- h never closed!
```

**Benefit**: Resource leaks and use-after-free prevented at compile time.

### Rust Ownership

Rust's ownership system approximates linear types:

```rust
fn process_file(path: &str) -> io::Result<String> {
    let file = File::open(path)?;  // Owns file handle
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    // file dropped here (RAII), handle closed
    Ok(contents)
}

// ERROR: use after move
let file = File::open("test.txt")?;
drop(file);
let _ = file.read_to_string(&mut buf)?;  // file moved!
```

**Affine Types**: Rust uses affine (≤1 use) instead of linear (=1 use). Can discard without using.

---

## Session Types

**Key Idea**: Types for communication protocols. Ensure protocol adherence at compile time.

### Session Type Syntax

```
S ::= !T. S         -- Send T, continue with S
   |  ?T. S         -- Receive T, continue with S
   |  &{l₁:S₁, ..., lₙ:Sₙ}  -- Offer branches
   |  ⊕{l₁:S₁, ..., lₙ:Sₙ}  -- Select branch
   |  end           -- Session termination
```

### Example: ATM Protocol

```
ATMClient :=
  ⊕{
    balance: !AccountID. ?Amount. end,
    withdraw: !AccountID. !Amount. &{
      ok: ?Cash. end,
      insufficient: end
    }
  }

ATMServer := dual of ATMClient
  = &{
      balance: ?AccountID. !Amount. end,
      withdraw: ?AccountID. ?Amount. ⊕{
        ok: !Cash. end,
        insufficient: end
      }
    }
```

**Type Safety**: Client and server have dual types → protocol followed correctly.

### Implementation (Rust-like)

```rust
// Session types library (conceptual)
enum Send<T, S> { Send(T, S) }
enum Recv<T, S> { Recv(Box<dyn FnOnce(T) -> S>) }
enum Choose<S1, S2> { Left(S1), Right(S2) }
enum Offer<S1, S2> { Offer(Box<dyn FnOnce(bool) -> Result<S1, S2>>) }
struct End;

// ATM client implementation
type ATMClient =
  Choose<
    Send<AccountID, Recv<Amount, End>>,  // balance
    Send<AccountID, Send<Amount,          // withdraw
      Offer<Recv<Cash, End>,              // ok
            End>>>                        // insufficient
  >;

fn atm_client(session: Chan<ATMClient>) {
    let session = session.select_left();  // Choose balance
    let session = session.send(account_id);
    let (amount, session) = session.recv();
    session.close();
}
```

**Benefit**: Protocol violations caught at compile time.

---

## Refinement Types

**Key Idea**: Types with predicates. Values satisfy both type and logical predicate.

### Refinement Type Syntax

```
{x : T | P(x)}
```

**Meaning**: Values of type T satisfying predicate P.

**Examples**:
```
Nat = {x : Int | x ≥ 0}
NonZero = {x : Int | x ≠ 0}
Sorted = {xs : List[Int] | isSorted(xs)}
```

### Example: Safe Division

```
divide : (x : Int) → (y : {y : Int | y ≠ 0}) → Int
divide x y = x / y

-- Usage:
divide 10 5   ✓  (5 ≠ 0 statically verified)
divide 10 0   ✗  (0 ≠ 0 fails, rejected at compile time)

-- With dynamic check:
safeDivide : Int → Int → Option Int
safeDivide x y =
  if y ≠ 0 then
    Some (divide x y)  -- y refined to NonZero in branch
  else
    None
```

### Liquid Types (F* / Dafny)

**Inference**: Automatically infer refinements using SMT solvers.

```fsharp
// F#-like syntax
let rec length (xs: list<'a>) : {n: int | n >= 0} =
  match xs with
  | [] -> 0
  | _::xs' -> 1 + length xs'

let head (xs: {xs: list<'a> | length xs > 0}) : 'a =
  match xs with
  | x::_ -> x
  // No need for [] case - type ensures non-empty!
```

**Verification**: SMT solver (Z3) verifies refinements automatically.

---

## Effect Systems

**Key Idea**: Track side effects in type system. Distinguish pure from impure computations.

### Effect Type Syntax

```
T ! E
```

**Meaning**: Computation returning T with effects E.

**Examples**:
```
pure : Int              -- Pure computation
read : Int ! {Read}     -- May read from environment
write : Unit ! {Write}  -- May write to environment
both : Int ! {Read, Write}  -- Both effects
```

### Koka Effect System

```koka
// Pure function
fun double(x : int) : int
  x * 2

// Effectful functions
fun print_hello() : console ()
  println("Hello")

fun read_file(path : string) : <exn,read> string
  // May throw exceptions and read files
  ...

// Effect handlers
fun test()
  with handler
    return(x) -> Just(x)
    ctl raise(msg) -> Nothing  // Handle exceptions
  do_something()
```

**Benefit**: Pure functions can be optimized aggressively (memoization, parallelization).

### Algebraic Effects (see Algebraic Effects appendix)

**Separate effect definition from handling**:

```ocaml
(* Define effect *)
effect Ask : string

(* Use effect *)
let greet () =
  let name = perform Ask in
  "Hello, " ^ name

(* Handle effect *)
let result =
  try greet () with
  | effect Ask k -> continue k "World"

(* result = "Hello, World" *)
```

---

## Stack-Specific Implementations

### Rust: Ownership as Affine Types

```rust
// Affine types: move semantics
struct Resource {
    data: String,
}

impl Resource {
    fn new(data: String) -> Self {
        Resource { data }
    }

    // Consumes self (affine)
    fn consume(self) -> String {
        self.data
    }
}

fn example() {
    let r = Resource::new("data".to_string());
    let data = r.consume();
    // r moved, can't use again
    // let _ = r.consume();  // ERROR: use after move
}

// Phantom types for state machines
use std::marker::PhantomData;

struct Locked;
struct Unlocked;

struct Door<State> {
    _state: PhantomData<State>,
}

impl Door<Locked> {
    fn unlock(self) -> Door<Unlocked> {
        println!("Unlocking door");
        Door { _state: PhantomData }
    }
}

impl Door<Unlocked> {
    fn lock(self) -> Door<Locked> {
        println!("Locking door");
        Door { _state: PhantomData }
    }

    fn open(&self) {
        println!("Opening door");
    }
}

fn door_example() {
    let door = Door::<Locked> { _state: PhantomData };
    // door.open();  // ERROR: can't open locked door
    let door = door.unlock();
    door.open();  // OK
    let door = door.lock();
}
```

### TypeScript: Refinement with Branded Types

```typescript
/**
 * Branded types for refinements
 */
type Brand<K, T> = K & { __brand: T };

type NonEmptyString = Brand<string, "NonEmpty">;
type PositiveNumber = Brand<number, "Positive">;
type Email = Brand<string, "Email">;

function nonEmptyString(s: string): NonEmptyString {
  if (s.length === 0) {
    throw new Error("String cannot be empty");
  }
  return s as NonEmptyString;
}

function positiveNumber(n: number): PositiveNumber {
  if (n <= 0) {
    throw new Error("Number must be positive");
  }
  return n as PositiveNumber;
}

function isEmail(s: string): s is Email {
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(s);
}

function sendEmail(to: Email, subject: NonEmptyString): void {
  console.log(`Sending email to ${to}: ${subject}`);
}

// Usage with guards
function processUser(email: string, age: number) {
  if (isEmail(email)) {
    // email refined to Email here
    sendEmail(email, nonEmptyString("Welcome!"));
  }

  if (age > 0) {
    const validAge = positiveNumber(age);
    // validAge has type PositiveNumber
  }
}

/**
 * Session types (encoding)
 */
type Send<T, Next> = {
  send(value: T): Next;
};

type Recv<T, Next> = {
  recv(): Promise<[T, Next]>;
};

type End = {
  close(): void;
};

// Protocol: !String.?Int.End
type ClientProtocol =
  Send<string, Recv<number, End>>;

class SessionChannel<Protocol> {
  constructor(private socket: any) {}

  send<T, Next>(this: SessionChannel<Send<T, Next>>, value: T): SessionChannel<Next> {
    this.socket.send(value);
    return this as any;
  }

  async recv<T, Next>(this: SessionChannel<Recv<T, Next>>): Promise<[T, SessionChannel<Next>]> {
    const value = await this.socket.recv();
    return [value, this as any];
  }

  close(this: SessionChannel<End>): void {
    this.socket.close();
  }
}

// Usage
async function client(session: SessionChannel<ClientProtocol>) {
  const s1 = session.send("hello");
  const [response, s2] = await s1.recv();
  s2.close();
}
```

### PHP: Runtime Validation (No Advanced Types)

```php
<?php

namespace TypeTheory;

/**
 * Refinement types via validation
 */
class NonEmptyString
{
    private string $value;

    private function __construct(string $value)
    {
        $this->value = $value;
    }

    public static function create(string $value): self
    {
        if (empty($value)) {
            throw new \InvalidArgumentException('String cannot be empty');
        }

        return new self($value);
    }

    public function getValue(): string
    {
        return $this->value;
    }
}

class PositiveInt
{
    private int $value;

    private function __construct(int $value)
    {
        $this->value = $value;
    }

    public static function create(int $value): self
    {
        if ($value <= 0) {
            throw new \InvalidArgumentException('Must be positive');
        }

        return new self($value);
    }

    public function getValue(): int
    {
        return $this->value;
    }
}

/**
 * Phantom types for state machines
 */
abstract class DoorState {}
class Locked extends DoorState {}
class Unlocked extends DoorState {}

class Door
{
    private string $state;

    private function __construct(string $state)
    {
        $this->state = $state;
    }

    public static function createLocked(): self
    {
        return new self(Locked::class);
    }

    public function unlock(): self
    {
        if ($this->state !== Locked::class) {
            throw new \RuntimeException('Door not locked');
        }

        return new self(Unlocked::class);
    }

    public function lock(): self
    {
        if ($this->state !== Unlocked::class) {
            throw new \RuntimeException('Door not unlocked');
        }

        return new self(Locked::class);
    }

    public function open(): void
    {
        if ($this->state !== Unlocked::class) {
            throw new \RuntimeException('Cannot open locked door');
        }

        echo "Door opened\n";
    }
}

/**
 * Session types (runtime)
 */
interface SessionType {}

class SendType implements SessionType
{
    public function __construct(
        public string $dataType,
        public SessionType $next
    ) {}
}

class RecvType implements SessionType
{
    public function __construct(
        public string $dataType,
        public SessionType $next
    ) {}
}

class EndType implements SessionType {}

class Session
{
    private SessionType $protocol;
    private $socket;

    public function __construct(SessionType $protocol, $socket)
    {
        $this->protocol = $protocol;
        $this->socket = $socket;
    }

    public function send($value): self
    {
        if (!$this->protocol instanceof SendType) {
            throw new \RuntimeException('Protocol violation: expected send');
        }

        // Send value
        fwrite($this->socket, serialize($value));

        // Advance protocol
        return new self($this->protocol->next, $this->socket);
    }

    public function recv()
    {
        if (!$this->protocol instanceof RecvType) {
            throw new \RuntimeException('Protocol violation: expected recv');
        }

        // Receive value
        $value = unserialize(fgets($this->socket));

        // Advance protocol
        return [$value, new self($this->protocol->next, $this->socket)];
    }

    public function close(): void
    {
        if (!$this->protocol instanceof EndType) {
            throw new \RuntimeException('Protocol not finished');
        }

        fclose($this->socket);
    }
}
```

---

## Integration Points

### With Formal Verification
- **Types as specifications**: Type checking = lightweight verification
- **Proof assistants**: Coq, Lean use dependent types for full verification

### With Concurrency
- **Session types**: Verify communication protocols
- **Linear types**: Prevent race conditions on resources

### With Functional Programming
- **Curry-Howard**: FP naturally aligns with constructive logic
- **Effect systems**: Track side effects in pure FP

---

## Further Reading

### Papers
- Curry & Feys (1958) - "Combinatory Logic"
- Howard (1980) - "The formulae-as-types notion of construction"
- Wadler (2015) - "Propositions as Types"
- Pierce & Sangiorgi (1996) - "Typing and Subtyping for Mobile Processes"

### Books
- Pierce - "Types and Programming Languages"
- Sørensen & Urzyczyn - "Lectures on the Curry-Howard Isomorphism"
- Pierce (ed.) - "Advanced Topics in Types and Programming Languages"

### Proof Assistants
- **Coq**: Dependently typed proof assistant
- **Agda**: Dependently typed programming language
- **Lean**: Theorem prover with dependent types
- **Idris**: Dependent types for practical programming

---

**End of Type Theory Appendix**
