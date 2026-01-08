# Formal Verification: Proving Software Correctness

**Purpose**: Mathematical techniques for proving that software systems satisfy their specifications, eliminating entire classes of bugs through rigorous analysis.

**Core Insight**: **Testing shows presence of bugs, verification shows absence**. Formal methods provide mathematical certainty about system behavior.

---

## Table of Contents

1. [Foundational Concepts](#foundational-concepts)
2. [Hoare Logic](#hoare-logic)
3. [Temporal Logic](#temporal-logic)
4. [Model Checking](#model-checking)
5. [TLA+ Specification](#tla-specification)
6. [SMT Solvers](#smt-solvers)
7. [Proof Assistants](#proof-assistants)
8. [Stack-Specific Verification](#stack-specific-verification)
9. [Integration Points](#integration-points)

---

## Foundational Concepts

### What is Formal Verification?

**Formal Verification**: Proving systems satisfy specifications using mathematical logic and automated reasoning.

**Approaches**:
1. **Deductive (Theorem Proving)**: Manually prove correctness
2. **Model Checking**: Automatically explore state space
3. **Static Analysis**: Analyze code without execution
4. **Runtime Verification**: Monitor execution against spec

**Benefits**:
- **Absolute Guarantees**: No counterexamples exist
- **Early Detection**: Find bugs before testing
- **Documentation**: Specs are precise, executable

**Challenges**:
- **Complexity**: State explosion, undecidability
- **Expertise**: Requires mathematical sophistication
- **Cost**: Time-intensive, tool limitations

### Specification Languages

**Properties**:
- **Safety**: "Bad things never happen" (e.g., no deadlock)
- **Liveness**: "Good things eventually happen" (e.g., progress)
- **Invariants**: Always true throughout execution

**Logics**:
- **First-Order Logic (FOL)**: Quantifiers, predicates
- **Temporal Logic**: Reasoning about time (LTL, CTL)
- **Hoare Logic**: Pre/post conditions for programs

---

## Hoare Logic

**Developed by**: C.A.R. Hoare (1969)

**Key Idea**: Specify program correctness using **Hoare triples**: `{P} C {Q}`
- **P**: Precondition (assumption before C)
- **C**: Command (program statement)
- **Q**: Postcondition (guarantee after C)

**Meaning**: If P holds before C and C terminates, then Q holds after C.

### Hoare Logic Rules

**Assignment**:
```
{Q[E/x]} x := E {Q}
```

**Example**:
```
{y + 1 > 0} x := y + 1 {x > 0}
```

**Sequence**:
```
{P} C₁ {R}    {R} C₂ {Q}
──────────────────────────
{P} C₁; C₂ {Q}
```

**If-Then-Else**:
```
{P ∧ B} C₁ {Q}    {P ∧ ¬B} C₂ {Q}
───────────────────────────────────
{P} if B then C₁ else C₂ {Q}
```

**While Loop**:
```
{I ∧ B} C {I}
─────────────────────────────────
{I} while B do C {I ∧ ¬B}
```

**I** is the **loop invariant** (must be maintained by loop body).

### Example: Array Summation

**Code**:
```c
// Precondition: n ≥ 0, a is array of n integers
sum = 0;
i = 0;
while (i < n) {
    sum = sum + a[i];
    i = i + 1;
}
// Postcondition: sum = Σ(a[0..n-1])
```

**Proof**:

**Invariant**: `I = (sum = Σ(a[0..i-1]) ∧ 0 ≤ i ≤ n)`

**Initial**: `{n ≥ 0} sum := 0; i := 0 {I}`
- After assignments: sum = 0, i = 0
- I holds: sum = Σ(a[0..-1]) = 0 ✓

**Maintenance**: `{I ∧ i < n} sum := sum + a[i]; i := i + 1 {I}`
- Before: sum = Σ(a[0..i-1]), i < n
- After: sum' = sum + a[i] = Σ(a[0..i]), i' = i + 1
- I holds: sum' = Σ(a[0..i'-1]) ✓

**Termination**: Loop terminates when i = n
- Final: I ∧ ¬(i < n) = sum = Σ(a[0..n-1]) ∧ i = n ✓

### Separation Logic

**Extension for heap manipulation**. Handles aliasing, memory safety.

**Key Operators**:
- `P * Q`: Separating conjunction (disjoint heaps)
- `P —* Q`: Magic wand (separating implication)
- `emp`: Empty heap
- `x ↦ v`: Heap cell at x contains v

**Frame Rule**:
```
{P} C {Q}
──────────────────  (if C doesn't modify vars in R)
{P * R} C {Q * R}
```

**Example**: Linked list append

```
{list(x, A) * list(y, B)}
  append(x, y)
{list(x, A @ B)}

Where list(x, A) = (x = null ∧ A = []) ∨
                   (∃v, next. x ↦ v, next * list(next, A'))
                               ∧ A = v :: A')
```

---

## Temporal Logic

**Key Idea**: Specify properties over time (sequences of states).

### Linear Temporal Logic (LTL)

**Operators**:
- `G φ` (Globally): φ holds at all future states
- `F φ` (Finally): φ holds at some future state
- `X φ` (Next): φ holds in next state
- `φ U ψ` (Until): φ holds until ψ becomes true

**Examples**:
- **Safety**: `G ¬deadlock` (never deadlock)
- **Liveness**: `G (request → F grant)` (every request eventually granted)
- **Fairness**: `G F enabled → G F executed` (infinitely often enabled → infinitely often executed)

### Computation Tree Logic (CTL)

**Path Quantifiers**:
- `A` (All paths): Property holds on all paths from state
- `E` (Exists path): Property holds on some path from state

**Combines with temporal operators**: AG, AF, EG, EF, etc.

**Examples**:
- `AG ¬deadlock`: On all paths, globally no deadlock (stronger than LTL G)
- `EF goal`: There exists path reaching goal
- `AG (request → AF grant)`: Every request eventually granted on all paths

### LTL vs CTL

| Property | LTL | CTL |
|----------|-----|-----|
| "Always P" | G P | AG P |
| "Eventually P" | F P | AF P (or EF P) |
| "P until Q" | P U Q | A[P U Q] (or E[P U Q]) |

**Expressiveness**: LTL and CTL incomparable (neither subsumes other).

---

## Model Checking

**Approach**: Automatically explore state space, verify temporal properties.

### Model Checking Algorithm (CTL)

**Input**: Model M (Kripke structure), Formula φ (CTL)

**Output**: True if M ⊨ φ, else counterexample

**Algorithm** (simplified):
1. Label each state with atomic propositions
2. Recursively label states satisfying subformulas
3. Check if initial state labeled with φ

**Example**:
```
M = states {s₀, s₁, s₂}
    transitions: s₀ → s₁, s₁ → s₂, s₂ → s₁
    labels: s₀: {p}, s₁: {q}, s₂: {p, q}

Check: M ⊨ AG (p → AF q)

1. Label states with p: {s₀, s₂}
2. Label states with q: {s₁, s₂}
3. Label states with AF q: {s₀, s₁, s₂} (all reach q)
4. Label states with p → AF q: {s₀, s₁, s₂} (implication)
5. Label states with AG (p → AF q): ...
```

### SPIN Model Checker

**Language**: Promela (Process Meta Language)

**Features**:
- Concurrent processes (like CSP)
- LTL property specifications
- Partial order reduction (state space reduction)
- Counterexample generation

**Example**: Mutual Exclusion

```promela
bool flag[2];
byte turn;

active [2] proctype P() {
    byte me = _pid;
    byte other = 1 - me;

    flag[me] = true;
    turn = other;

    (flag[other] == false || turn == me);  // Wait

    /* Critical Section */
    flag[me] = false;
}

// Property: Mutual exclusion
ltl mutex { [] !(flag[0] && flag[1]) }

// Property: Starvation freedom
ltl progress { []<> (flag[0] -> <> flag[0] == false) }
```

**Verification**: `spin -a mutex.pml && gcc -o pan pan.c && ./pan`

### TLC (TLA+ Model Checker)

**For TLA+ specifications** (see next section).

**Features**:
- Explicit state model checking
- Breadth-first or depth-first search
- Deadlock detection, invariant checking
- Simulation mode for large state spaces

---

## TLA+ Specification

**Developed by**: Leslie Lamport

**Key Idea**: Specify systems using temporal logic of actions.

### TLA+ Basics

**State**: Assignment of values to variables

**Action**: Relation between states (before/after)
- Example: `x' = x + 1` (x incremented)

**Specification**: Initial state + Next-state relation

**Example**: Simple Counter

```tla
-------------------------- MODULE Counter --------------------------
EXTENDS Integers

VARIABLE x

Init == x = 0

Increment == x' = x + 1

Next == Increment

Spec == Init /\ [][Next]_x

TypeInvariant == x \in Nat

====================================================================
```

**Check**: Invariant `x ∈ Nat` holds for all reachable states.

### TLA+ Example: Two-Phase Commit

```tla
------------------------ MODULE TwoPhaseCommit ----------------------
EXTENDS Integers, FiniteSets

CONSTANTS RM  \* Resource managers

VARIABLES
  rmState,      \* State of each RM: "working", "prepared", "committed", "aborted"
  tmState,      \* Transaction manager state: "init", "committed", "aborted"
  tmPrepared,   \* Set of RMs that prepared
  msgs          \* Messages in flight

Init ==
  /\ rmState = [r \in RM |-> "working"]
  /\ tmState = "init"
  /\ tmPrepared = {}
  /\ msgs = {}

RMPrepare(r) ==
  /\ rmState[r] = "working"
  /\ rmState' = [rmState EXCEPT ![r] = "prepared"]
  /\ msgs' = msgs \union {"Prepared"}
  /\ UNCHANGED <<tmState, tmPrepared>>

TMCommit ==
  /\ tmState = "init"
  /\ tmPrepared = RM  \* All prepared
  /\ tmState' = "committed"
  /\ msgs' = msgs \union {"Commit"}
  /\ UNCHANGED <<rmState, tmPrepared>>

RMCommit(r) ==
  /\ rmState[r] = "prepared"
  /\ "Commit" \in msgs
  /\ rmState' = [rmState EXCEPT ![r] = "committed"]
  /\ UNCHANGED <<tmState, tmPrepared, msgs>>

...

Next == \/ \E r \in RM : RMPrepare(r)
        \/ TMCommit
        \/ \E r \in RM : RMCommit(r)
        ...

Spec == Init /\ [][Next]_vars

\* Invariant: If any committed, all committed or aborted
Consistency ==
  \A r1, r2 \in RM :
    /\ rmState[r1] = "committed"
    /\ rmState[r2] # "aborted"
    => rmState[r2] \in {"committed", "prepared"}

====================================================================
```

**Verification**: TLC checks all reachable states satisfy `Consistency`.

---

## SMT Solvers

**Satisfiability Modulo Theories**: SAT solving + theories (arithmetic, arrays, etc.).

### Theories

**Common Theories**:
- **QF_LIA**: Quantifier-free linear integer arithmetic
- **QF_NIA**: Nonlinear integer arithmetic
- **QF_BV**: Bitvectors (fixed-width integers)
- **Arrays**: Array theory (read/write)
- **Uninterpreted Functions**: Function symbols without definition

### Example: Z3 (Microsoft)

**Problem**: Verify array bounds check elimination

```smt2
(declare-const n Int)
(declare-const i Int)

(assert (>= n 0))
(assert (>= i 0))
(assert (< i n))

; Check if i < n (should be valid given assertions)
(assert (not (< i n)))

(check-sat)  ; Should return unsat (no counterexample)
```

### Using SMT in Verification

**Symbolic Execution**:
1. Execute program symbolically (variables = symbols)
2. Build path condition (constraints on path)
3. Query SMT solver to check feasibility

**Example**:
```c
int foo(int x, int y) {
    if (x > y) {
        if (x < 10) {
            assert(y < 10);  // Is this always true?
        }
    }
}

Path: x > y and x < 10
Query SMT: (x > y) ∧ (x < 10) ∧ ¬(y < 10) satisfiable?
Result: SAT with y = 10, x = 9 (counterexample!)
```

---

## Proof Assistants

**Interactive theorem provers**: User guides proof, tool checks validity.

### Coq

**Based on**: Calculus of Inductive Constructions (dependent types)

**Example**: Prove list append associativity

```coq
Require Import List.
Import ListNotations.

Theorem app_assoc : forall (A : Type) (l m n : list A),
  l ++ m ++ n = (l ++ m) ++ n.
Proof.
  intros A l m n.
  induction l as [| h t IH].
  - (* l = [] *)
    simpl. reflexivity.
  - (* l = h :: t *)
    simpl. rewrite IH. reflexivity.
Qed.
```

**Use Cases**: CompCert (verified C compiler), Feit-Thompson theorem.

### Lean

**Theorem Prover + Programming Language**

**Example**: Fibonacci correctness

```lean
def fib : Nat → Nat
  | 0 => 0
  | 1 => 1
  | n + 2 => fib n + fib (n + 1)

theorem fib_pos : ∀ n : Nat, n > 0 → fib n > 0 := by
  intro n
  cases n with
  | zero => intro h; contradiction
  | succ n' =>
    intro _
    cases n' with
    | zero => simp [fib]
    | succ n'' => simp [fib]; omega
```

### Isabelle/HOL

**Higher-Order Logic** theorem prover.

**Example**: Prove De Morgan's law

```isabelle
lemma de_morgan: "¬(P ∧ Q) = (¬P ∨ ¬Q)"
  by auto
```

**Use Cases**: seL4 (verified microkernel), Flyspeck (Kepler conjecture).

---

## Stack-Specific Verification

### Rust: Verification with Prusti

```rust
// Prusti: Verification tool for Rust using Viper

use prusti_contracts::*;

#[requires(n >= 0)]
#[ensures(result >= 0)]
#[ensures(result == old(n) + 1)]
fn increment(n: i32) -> i32 {
    n + 1
}

#[requires(arr.len() > 0)]
#[ensures(result.is_some())]
#[ensures(result.unwrap() >= 0)]
#[ensures(forall(|i: usize| i < arr.len() ==>
            result.unwrap() >= arr[i]))]
fn max(arr: &[i32]) -> Option<i32> {
    let mut max_val = arr[0];
    let mut i = 1;

    while i < arr.len() {
        body_invariant!(i > 0);
        body_invariant!(i <= arr.len());
        body_invariant!(forall(|j: usize| j < i ==> max_val >= arr[j]));

        if arr[i] > max_val {
            max_val = arr[i];
        }
        i += 1;
    }

    Some(max_val)
}
```

**Verification**: `cargo prusti` checks contracts.

### TypeScript: Runtime Assertions

```typescript
/**
 * Design by Contract with runtime checks
 */

function requires(condition: boolean, message: string): asserts condition {
  if (!condition) {
    throw new Error(`Precondition violated: ${message}`);
  }
}

function ensures(condition: boolean, message: string): asserts condition {
  if (!condition) {
    throw new Error(`Postcondition violated: ${message}`);
  }
}

/**
 * Binary search with contracts
 */
function binarySearch(arr: number[], target: number): number {
  // Precondition: array is sorted
  requires(
    arr.every((v, i) => i === 0 || arr[i - 1] <= v),
    "Array must be sorted"
  );

  let left = 0;
  let right = arr.length - 1;

  while (left <= right) {
    // Loop invariant: if target exists, it's in arr[left..right]
    const mid = Math.floor((left + right) / 2);

    if (arr[mid] === target) {
      // Postcondition: found target at valid index
      ensures(arr[mid] === target, "Found element matches target");
      ensures(mid >= 0 && mid < arr.length, "Index in bounds");
      return mid;
    } else if (arr[mid] < target) {
      left = mid + 1;
    } else {
      right = mid - 1;
    }
  }

  // Postcondition: target not found
  ensures(!arr.includes(target), "Target not in array");
  return -1;
}

/**
 * Symbolic execution (conceptual)
 */
type SymbolicInt = { type: 'concrete', value: number } | { type: 'symbolic', name: string };

class SymbolicExecutor {
  pathCondition: string[] = [];

  symbolicIf(condition: boolean, condStr: string): boolean {
    if (condition) {
      this.pathCondition.push(condStr);
    } else {
      this.pathCondition.push(`!(${condStr})`);
    }
    return condition;
  }

  verify(assertion: boolean): boolean {
    if (!assertion) {
      console.log('Counterexample found!');
      console.log('Path condition:', this.pathCondition.join(' && '));
      return false;
    }
    return true;
  }
}
```

### PHP: Property-Based Testing (Approximation)

```php
<?php

namespace Verification;

/**
 * Property-based testing with QuickCheck-style generators
 */
class Property
{
    public static function forAll(
        callable $generator,
        callable $property,
        int $iterations = 100
    ): bool {
        for ($i = 0; $i < $iterations; $i++) {
            $input = $generator();

            if (!$property($input)) {
                echo "Property violated with input: ";
                var_dump($input);
                return false;
            }
        }

        return true;
    }
}

/**
 * Generators
 */
class Gen
{
    public static function int(int $min = PHP_INT_MIN, int $max = PHP_INT_MAX): callable
    {
        return fn() => random_int($min, $max);
    }

    public static function array(callable $elemGen, int $maxSize = 100): callable
    {
        return function() use ($elemGen, $maxSize) {
            $size = random_int(0, $maxSize);
            $arr = [];
            for ($i = 0; $i < $size; $i++) {
                $arr[] = $elemGen();
            }
            return $arr;
        };
    }

    public static function sortedArray(int $maxSize = 100): callable
    {
        return function() use ($maxSize) {
            $arr = Gen::array(Gen::int(0, 1000), $maxSize)();
            sort($arr);
            return $arr;
        };
    }
}

/**
 * Example: Verify list properties
 */
function verify_list_properties(): void
{
    // Property: reverse(reverse(list)) = list
    $prop1 = Property::forAll(
        Gen::array(Gen::int()),
        fn($arr) => array_reverse(array_reverse($arr)) === $arr
    );

    echo "reverse(reverse(x)) = x: " . ($prop1 ? "✓" : "✗") . "\n";

    // Property: sort preserves length
    $prop2 = Property::forAll(
        Gen::array(Gen::int()),
        function($arr) {
            $sorted = $arr;
            sort($sorted);
            return count($sorted) === count($arr);
        }
    );

    echo "sort preserves length: " . ($prop2 ? "✓" : "✗") . "\n";

    // Property: sorted array is monotonic
    $prop3 = Property::forAll(
        Gen::sortedArray(),
        function($arr) {
            for ($i = 1; $i < count($arr); $i++) {
                if ($arr[$i] < $arr[$i - 1]) {
                    return false;
                }
            }
            return true;
        }
    );

    echo "sorted array is monotonic: " . ($prop3 ? "✓" : "✗") . "\n";
}
```

---

## Integration Points

### With Type Systems
- **Refinement types**: SMT-based verification of type predicates
- **Dependent types**: Proofs encoded as programs

### With Testing
- **Model-based testing**: Generate tests from specifications
- **Property-based testing**: Approximate verification with random testing

### With Concurrency
- **SPIN**: Verify concurrent protocols
- **TLA+**: Specify distributed algorithms

---

## Further Reading

### Papers
- Hoare (1969) - "An Axiomatic Basis for Computer Programming"
- Pnueli (1977) - "The Temporal Logic of Programs"
- Clarke & Emerson (1981) - "Design and Synthesis of Synchronization Skeletons Using Branching Time Temporal Logic"
- Lamport (2002) - "Specifying Systems: The TLA+ Language and Tools"

### Books
- Huth & Ryan - "Logic in Computer Science: Modelling and Reasoning about Systems"
- Baier & Katoen - "Principles of Model Checking"
- Lamport - "Specifying Systems"
- Pierce et al. - "Software Foundations" (Coq-based)

### Tools
- **SPIN**: Model checker for concurrent systems
- **TLA+/TLC**: Specification and model checking
- **Z3**: SMT solver (Microsoft)
- **Coq/Lean/Isabelle**: Proof assistants
- **Dafny**: Verification-aware programming language
- **F***: Dependent types with SMT

---

**End of Formal Verification Appendix**
