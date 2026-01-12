🚫 NEVER DELETE THIS FILE 🚫

# 📖 Codex Execution Rules

## ⚡ Use MechCrate RAG for Architectural Decisions

**Before implementing**: Query the MechCrate MCP server RAG tools for architectural guidance, design patterns, and code examples.

**7 RAG Tools Available**:
- `rag_search` - Semantic search across all documentation
- `rag_search_category` - Search specific categories (recipe, command, docker, codex, infrastructure)
- `rag_find_implementation` - Find code examples and configurations
- `rag_get_guidance` - Get architecture and design guidance
- `rag_compare_approaches` - Compare technologies, recipes, or strategies
- `rag_find_related` - Discover related documentation
- `rag_health` - Check RAG system availability

**When to query**: Before choosing patterns, selecting technologies, implementing complex algorithms, designing APIs, or making architectural decisions.

All codex documentation is indexed and semantically searchable. Use RAG first.

---

## Table of Contents
1. [The Journey Inward: Advanced Theory Index](#1-the-journey-inward-advanced-theory-index)
2. [Code References](#2-code-references)
3. [Procedure](#3-procedure)
4. [Functional Design Foundations](#4-functional-design-foundations)
   - [Core Principles](#41-core-principles)
   - [Types and Domain Modeling](#42-types-and-domain-modeling)
   - [Functional Patterns](#43-functional-patterns)
   - [Error Handling Strategy](#44-error-handling-strategy)
   - [Effects and Concurrency](#45-effects-and-concurrency)
   - [Data and API Boundaries](#46-data-and-api-boundaries)
   - [UI with Functional Style](#47-ui-with-functional-style)
   - [Finite State & State Machines](#48-finite-state--state-machines)
5. [Strong Software Design Strategies](#5-strong-software-design-strategies)
6. [Tests](#6-tests)
7. [Adoption Guide & Playbooks](#7-adoption-guide--playbooks)
8. [Appendices](#8-appendices)

---

## 1. The Journey Inward: Advanced Theory Index

**Start Here**: For advanced programming theory, patterns, and computational thinking, consult the [INDEX.md](INDEX.md).

**What It Contains**:
- **Category Theory**: Functors, Monads, Comonads, F-Algebras, Optics, Yoneda Lemma
- **Type Theory**: Dependent types, GADTs, Higher-Kinded Types, Effect Systems
- **Computation Models**: Lambda Calculus, Process Calculi, Automata Theory
- **Advanced Patterns**: Recursion Schemes, Free Monads, Tagless Final, Defunctionalization
- **Language-Specific Implementations**: Rust, TypeScript, PHP, Haskell
- **Infrastructure & Operations**: Docker Assembly Guide with production-grade patterns

**How to Use the Index**:
- **For Humans**: Navigate by use-case or scenario to find relevant documents
- **For LLMs**: Use document metadata (complexity, use cases, prerequisites) to determine which references to pull
- **For Architecture**: Reference appropriate documents during design reviews for rigorous foundations

**When to Dive Deeper**:
- Designing composable, type-safe APIs
- Implementing advanced abstractions or DSLs
- Need rigorous mathematical foundations
- Teaching or learning advanced functional programming
- Building sophisticated distributed systems
- Setting up production-grade Docker infrastructure

The INDEX provides comprehensive metadata for each document including:
- Size and complexity ratings (⭐ to ⭐⭐⭐⭐⭐)
- Best use cases and when to reference
- Prerequisites and related documents
- Language-specific routing for LLMs

**See**: [INDEX.md](INDEX.md) for the complete catalog and navigation guide.

---

## 2. Code References
1. use the context7 mcp tool to get the most up-to-date implementation details for frameworks and tools
2. If a `__reference__` (or similar) folder exists:
   - Analyze those projects first.
   - They often contain framework and tool sources at the latest version.
   - Always use them to avoid outdated patterns.

---

## 3. Procedure
0. Default to high-detail, structured reasoning. Provide thorough, rigorous explanations with explicit assumptions, options trade-offs, edge cases, and validation steps. Keep results scannable with short headers and concise bullets; avoid filler. Do not reveal hidden chain-of-thought or private scratch work—summarize reasoning at a high level. Ask clarifying questions only when necessary. Adjust verbosity on request: "brief" = minimal, "standard" = typical, "high" = this default, "max" = high with deeper validation. Offer a short plan and progress updates for multi-step tasks.
1. You have access to `/tmp` folders and the internet. Use them (curl, API exploration, etc).  
2. Test everything. Never assume. Install and run dependencies as required (composer, yarn, cargo, docker, etc).  
3. Work in small increments. After each change, build and confirm nothing broke.  
4. Code like a veteran with 20+ years of experience. Write small testable units with clean abstractions.  
5. For UI frameworks (React, Vue, Svelte, Leptos, dioxus):  
   - Split components if they approach 100 lines.  
   - Create shared modules/mixins/components instead of duplicating code.  
6. Always use Yarn. Use `/tmp` workspaces if permission issues occur.  
7. Aim for zero warnings. Builds must compile cleanly.  
8. Keep `README.md` updated with the latest implementation details.  
9. Use the `Makefile` as the central CLI:  
   - Store long commands in `scripts/` and reference them from the Makefile.  
10. Use `/tmp` for build and cargo operations due to sandbox constraints.  
11. Add ❤️ and 🌻🌹🪻 where appropriate.  
12. Commits:  
   - Must be atomic.  
   - Follow Conventional Commits format.  
13. If new tools or dependencies are needed:  
   - Stop and request the install command.  
   - Wait for confirmation before proceeding.  

---

## 4. Functional Design Foundations

### 4.1 Core Principles
- Prefer pure functions. One input → one output, no hidden effects.  
- Favor immutability. Never mutate in place, always return new values.  
- Total functions over partial functions. Handle every valid input.  
- Referential transparency. Calls can be replaced by values without changing behavior.  
- Composition over inheritance. Build from smaller modules.  
- Make effects explicit (IO, time, randomness, logging, network).  

### 4.2 Types and Domain Modeling
- Use algebraic data types (sum/product) to encode valid states.
- Replace null with Option/Maybe.
- Represent failures with Result/Either, not exceptions.
- Encode invariants in types (newtypes, branded, phantom).
- Model workflows as state machines/typestates. Only allow valid transitions.

### 4.3 Functional Patterns
- Function composition, pipelines, data-last style.
- Currying/partial application for reuse.
- Higher-order functions (`map`, `reduce`, `filter`, `traverse`) as first tools.
- Use lawful abstractions (monoid, functor, applicative, monad) with restraint.
- Interpreter pattern: define domain algebra, provide multiple interpreters.
- Ports & adapters: keep domain pure, push side effects to adapters.

### 4.4 Error Handling Strategy
- Use explicit return types (`Result`/`Either`) with rich error values.
- Centralize error classification (transient, fatal, user input).
- Collapse and lift errors at boundaries, never leak raw library errors.

### 4.5 Effects and Concurrency
- Isolate non-determinism. Pass time and randomness as dependencies.
- Concurrency via message passing (queues, channels, actors).
- Cancellation/timeouts modeled in types, never ignored.

### 4.6 Data and API Boundaries
- Always decode/encode at the edges.
- Keep domain models independent of transport.
- Use mappers for translation.

### 4.7 UI with Functional Style
- Derive state from props/data, never imperative mutation.
- Use reducer-style updates for complex state.
- Prefer pure components. Memoize only if measured benefit.
- Always keep it sexy in both implementation and presentation. SEXY.


### 4.8 Finite State & State Machines
- Model ALL non-trivial workflows as finite state machines (FSMs) or statecharts.  
- State = closed set (enum/sum type).  
- Transitions = named events only, with pure guards.  
- Side-effects = explicit actions, never direct mutations.  
- Invalid transitions = impossible by types or rejected with explicit error.  
- Document transition tables/diagrams alongside code.  
- Tests must cover all `(state, event)` pairs, invariants, and rejection cases.  
- Observability: log structured transition events, metrics for state occupancy.  

---

## 5. Strong Software Design Strategies

### 5.1 Architecture and Modularity
- High cohesion, low coupling.  
- Explicit boundaries. Public API is small/stable, internals private.  
- Depend on abstractions, never concrete details.  

### 5.2 Evolution and Change
- Design for extension via composition/configuration.
- Keep changelog + ADRs.
- Use feature flags for safe delivery.

### 5.3 Quality Gates
- Static analysis + formatters required (ESLint, Prettier, Rustfmt, Clippy, Pint, Larastan).
- Enforce type checks on every build.
- CI must run tests, lints, type checks.

### 5.4 Testing Strategy
- Unit tests for pure logic.
- Property-based tests for invariants.
- Contract tests at IO boundaries.
- Integration tests for workflows.
- Load/stress tests for performance budgets.

### 5.5 Performance and Reliability
- Define budgets for latency, memory, throughput.
- Cache/precompute only with explicit invalidation.
- Add observability: logs, metrics, traces.

### 5.6 Security and Compliance
- Least privilege for secrets/keys.
- Validate inputs, sanitize outputs.
- Track dependencies/licenses, update regularly.

### 5.7 Documentation and Communication
- Keep `README`, API docs, examples runnable.  
- Provide usage snippets for each public API.  
- Record tradeoffs & rejected options in ADRs.  
- Organize docs atomically in `/docs`.  

---

## 6. Tests
1. Every feature must include a unit or feature test.
2. Run all tests before delivery.
3. No untested code may be handed off.
4. Property-based tests for invariants where possible.
5. Contract tests for adapters at IO boundaries.
6. Use puppeteer MCP to observe UI/UX results and iterate to make things smooth, functional and exceptional.



## 7. Adoption Guide & Playbooks

### 7.1 Language-Specific Guides

#### PHP / Laravel (Munus) 

- Libraries: munusphp/munus for Option, Either, Try.

- Policies:

  - Repositories return Option<T> for lookups; controllers fold to 404/defaults.

  - Services return Either<DomainError, T>; controllers fold to HTTP.

  - No null in domain code. Replace with Option.

  - Avoid throwing; use Either unless truly exceptional.

- Patterns:

  - Option::fromNullable($x)->map(...)->getOrElse($d)

  - Either::right($x)->flatMap(...)->mapLeft(...)

  - Controller boundary: fold(err => response..., ok => response...).

- Example:

```php
use Munus\Control\Option;
use Munus\Control\Either;

final class UserRepo {
    /** @return Option<User> */
    public function byId(int $id): Option { return Option::fromNullable(User::find($id)); }
}

final class IssueCouponService {
    /** @return Either<CouponError, string> */
    public function __invoke(string $email, string $code): Either { /* ... */ }
}

// Controller
$res = ($service)($req->string('email'), $req->string('code'));
return $res->fold(
    fn($err) => response()->json(['error' => (string)$err], 422),
    fn($id)  => response()->json(['id' => $id], 201)
);
```

#### TypeScript / Node (fp-ts)


- Libraries: fp-ts, io-ts for decoders, fp-ts/lib/TaskEither for async.

- Policies:

  - Domain functions return Either<E, A>; async returns TaskEither<E, A>.

  - Decode all external input with io-ts; convert to Either.

- Example:

```typescript
import * as TE from 'fp-ts/TaskEither';
import { pipe } from 'fp-ts/function';

const fetchUser = (id: string): TE.TaskEither<'not_found'|'io', User> => /* ... */;

const handler = (id: string) => pipe(
  fetchUser(id),
  TE.match(
    err => ({ status: err === 'not_found' ? 404 : 500 }),
    user => ({ status: 200, body: user })
  )
);
```

#### Rust

- Use Option<T> and Result<T, E> (native).

- Map/and_then for pipelines; never unwrap in library code.

- Convert error types at boundaries via From/thiserror.

#### Python (optional)

- returns library (pip install returns) for Maybe, Result, IOResult.

### 7.2 Incremental Adoption Plan

- Boundary First: Make controllers/handlers fold Either and Option.

- Repositories: Return Option for lookups; delete null from signatures.

- Services: Bubble domain failures via Either with rich errors.

- Validation: Centralize decoding/validation to return Either.

- IO Isolation: Wrap external calls in Try/TaskEither (or language equivalent).

- Refactor Loops: Replace nested if/try with map/flatMap/fold chains; extract helpers once chains exceed 5 steps.

- FSMs: For non‑trivial workflows, refactor to state machines with explicit transitions and actions.


✨ **Mindset**: Test. Document. Build clean. Build sexy. Commit smart.

---

## 8. Appendices
- [Appendix A: Rust Idioms and Patterns](appendix-rust.md)
- [Appendix B: Laravel Idioms and Patterns](appendix-laravel.md)
- [Appendix C: FSM Code Examples](appendix-fsm.md)
- [Appendix D: Software Design Pattern Playbook](appendix-pattern-playbook.md)
- [Appendix E: Business Logic Placement](appendix-business-logic-placement.md)
