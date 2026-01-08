# Project & File-Structure Theory for Scalable Applications

**Audience:**  
- Humans designing applications  
- LLMs generating or modifying codebases  

**Goal:**  
Define principles and patterns for structuring application folders/files so they scale with:
- Codebase size
- Team size
- Feature complexity
- Operational complexity (deployments, environments, etc.)

---

## 1. Why File Structure Matters (The “Theory” Bit)

File trees *encode architecture*. The way files are grouped influences:

- **Coupling:** How much changes in one area force changes in others  
- **Cohesion:** How related the things in the same folder/module are  
- **Cognitive load:** How hard it is to find, understand, and modify code  
- **Team boundaries:** Who owns what, and how teams interfere with each other (Conway’s Law)  
- **Change patterns:** Whether common changes touch one module or scatter across the repo

We’re not in “hard science” territory, but we do have decades of software engineering theory:

- **Modularization (Parnas):** Group by *reasons for change*, not by random technical detail.
- **Information Hiding:** Internal details stay behind clear interfaces.
- **High cohesion, low coupling:** The classic design heuristic.
- **Conway’s Law:** System structure tends to mirror org structure; structure should either reflect or intentionally resist this.

From this perspective, a "good" file structure is one where:

1. Most changes touch a **small number of modules**.
2. It’s obvious **where new code should live**.
3. You can **delete or replace a module** without surgery across the codebase.
4. Team boundaries and domain boundaries are **aligned with the folder/module layout**.

---

## 2. Design Goals for a Scalable Structure

When organizing files, aim to optimize for:

1. **Change Locality**  
   - When a feature changes, most edits happen in one folder/module.
   - Example: “User Profile” feature mostly lives in `features/profile/`.

2. **Domain-Centric Thinking**  
   - Structure reflects business/domain concepts (Orders, Billing, Auth) rather than only technical layers (Controllers, Services, Repos).

3. **Stable Boundaries & APIs**  
   - Boundaries (modules, packages, services) expose small, stable interfaces.
   - Internals can churn; boundaries stay predictable.

4. **Explicit Architecture**  
   - There’s a clear “rules of the game”: e.g. `api` layer can depend on `domain`, but `domain` must not depend on `api`.

5. **Scalability Along Multiple Axes**  
   - **More features:** Add new feature modules without rewriting existing ones.
   - **More devs:** Multiple teams can work with minimal merge conflicts.
   - **More services:** If needed, feature or bounded context can be split out into a separate service.

6. **Discoverability**  
   - Given a task, a newcomer (or an LLM) can reliably guess where a file *should* be.

---

## 3. Two Fundamental Strategies: By Layer vs By Feature

Most project structures are variations or hybrids of these two.

### 3.1 Package-by-Layer (Horizontal Slices)

Group files by **technical role**:

```text
src/
  controllers/
  services/
  repositories/
  models/
  utils/
```

**Pros:**
- Simple to understand.
- Works fine for small apps.
- Reflects typical textbook architectures (MVC, layered).

**Cons:**
- Scaling problem: a single feature often touches many layers across the tree.
- When app grows, `services/` and `utils/` become grab bags.
- Hard to isolate and extract features into separate modules/services.

**When to use:**
- Very small projects or prototypes.
- Educational / tutorial codebases.
- When team and domain are both tiny and stable.

---

### 3.2 Package-by-Feature / Domain (Vertical Slices)

Group by **domain concept / feature**, not by layer:

```text
src/
  users/
    api/
    domain/
    infra/
  billing/
    api/
    domain/
    infra/
  shared/
    auth/
    logging/
```

**Pros:**
- High change locality: changes to "Billing" mostly stay in `billing/`.
- Easier to scale teams: “Team Billing” owns `billing/`, “Team Users” owns `users/`.
- Closer to how the business thinks (DDD-friendly).
- Natural path to modular monolith / microservices.

**Cons:**
- Slightly more upfront thought required.
- Some cross-cutting concerns (logging, auth, etc.) need a clear shared home.

**When to use (recommended default):**
- Anything beyond a small toy project.
- Systems you expect to grow in features, teams, or lifespan.

---

## 4. Architecture Styles That Influence File Layout

File structures should reflect architecture, not the other way around. Some key patterns:

### 4.1 Layered Architecture (N-tier)

- Layers like `presentation -> application -> domain -> infrastructure`.
- Mapped to folders:

```text
src/
  presentation/
  application/
  domain/
  infrastructure/
```

**Guiding rule:** dependencies go downward only (e.g. `presentation` can depend on `application`, but not vice versa).

You can combine with feature slicing:

```text
src/
  users/
    application/
    domain/
    infrastructure/
  billing/
    application/
    domain/
    infrastructure/
```

---

### 4.2 Ports & Adapters / Hexagonal / Clean Architecture

Key ideas:
- Domain core is pure and framework-agnostic.
- External systems (databases, APIs, UI) connect through ports (interfaces) and adapters (implementations).

Possible layout:

```text
src/
  users/
    domain/        # entities, value objects, domain services
    application/   # use cases, orchestration
    ports/         # interfaces for persistence, messaging, etc.
    adapters/      # db adapters, API adapters
```

This shines when:
- You want testable business logic.
- You anticipate swapping infrastructure (databases, messaging, etc.).

---

### 4.3 Domain-Driven Design (DDD) & Bounded Contexts

- **Bounded context**: a self-contained area of the domain with its own model and language (e.g. `Billing`, `Catalog`, `Support`).
- Excellent basis for both file structure and team structure.

Common mapping:

```text
src/
  billing/           # bounded context
    domain/
    application/
    infrastructure/
  catalog/
    domain/
    application/
    infrastructure/
  shared-kernel/
    domain/
```

Bounded contexts can later be split into separate services without massive rewrites.

---

### 4.4 Modular Monolith & Microservices

- **Modular monolith:** a single deployable unit with strong internal module boundaries.
- **Microservices:** each module becomes its own deployable service, each with its own internal structure.

Monolith repo organized modularly:

```text
src/
  contexts/
    billing/
    catalog/
    users/
  shared/
```

Microservices from that monolith:

```text
services/
  billing-service/
    src/
      billing/
  catalog-service/
    src/
      catalog/
  users-service/
    src/
      users/
shared-libs/
  logging/
  auth/
```

**Key principle:**  
Design your monolith as if it were a set of services with clear contracts, even if it’s a single binary/run-time.

---

## 5. Practical Templates

### 5.1 Backend Service (Feature-Oriented, Modular Monolith Friendly)

```text
.
├─ src/
│  ├─ core/                 # cross-cutting, framework-agnostic logic
│  │  ├─ shared/            # shared types, utilities, cross-cutting domain helpers
│  │  └─ common/            # common infrastructure helpers (e.g. error types)
│  │
│  ├─ features/
│  │  ├─ users/
│  │  │  ├─ domain/         # entities, value objects, domain services
│  │  │  ├─ application/    # use cases, orchestrations
│  │  │  ├─ api/            # controllers/handlers/routes
│  │  │  └─ infra/          # db access, external API integration
│  │  ├─ billing/
│  │  │  ├─ domain/
│  │  │  ├─ application/
│  │  │  ├─ api/
│  │  │  └─ infra/
│  │  └─ ...
│  │
│  ├─ config/               # app configuration, DI wiring
│  └─ main.*                # app entrypoint
│
├─ tests/
│  ├─ unit/
│  ├─ integration/
│  └─ e2e/
│
├─ docs/
└─ scripts/
```

### 5.2 Frontend SPA (Feature-First)

```text
src/
  app/
    routes/
    layout/
    providers/
  features/
    auth/
      components/
      hooks/
      api/
      state/
    profile/
      components/
      hooks/
      api/
      state/
  shared/
    components/
    hooks/
    lib/
    styles/
```

Principle: each feature folder contains everything that feature needs; `shared/` contains reusable pieces used by multiple features.

---

## 6. Choosing a Structure: A Simple Decision Guide

When designing or evolving structure (for humans or LLMs), follow this:

1. **Identify domain areas or features.**
   - Example: Auth, Users, Billing, Catalog.

2. **Treat each as a module / bounded context.**
   - Give each its own top-level folder.

3. **Inside each feature/context, apply a simple, consistent internal pattern.**
   - `domain / application / api / infra` is a solid default.
   - Alternatively: `components / hooks / api / state` for frontend.

4. **Define dependency rules.**
   - Within a backend feature:
     - `domain` depends on nothing internal.
     - `application` depends on `domain`.
     - `api` depends on `application` and `domain`.
     - `infra` depends on `domain` and external libs.
   - Across features:
     - Prefer depending on **interfaces or contracts**, not concrete internals.

5. **Create a shared area intentionally.**
   - `shared/` for truly shared stuff.
   - Avoid dumping everything there; require a good reason before something becomes “shared”.

6. **Evolve, don’t freeze.**
   - Start simple, reorganize when:
     - A folder grows too large.
     - A feature becomes clearly distinct enough for a module/context.
     - You see recurring change patterns that suggest moving things closer.

---

## 7. Anti-Patterns to Avoid

When designing or generating file structures, avoid these:

1. **“Junk Drawer” Folders**
   - Names like `misc/`, `others/`, `tmp/`, `old/` in main src tree.
   - `utils/` that ends up containing unrelated stuff.

2. **God Modules / God Folders**
   - One mega-folder like `services/` with dozens of unrelated services.
   - Single module doing orchestration for many unrelated features.

3. **Circular Dependencies in Structure**
   - Feature A imports deep internals of Feature B, and vice versa.
   - Any structural pattern that encourages dependency cycles.

4. **Framework-Driven Structure Only**
   - Structure mirroring framework internals instead of domain.
   - Example: `controllers/`, `models/`, `views/` at root for a large, complex domain.

5. **Hidden Architecture**
   - No clear rules about what can depend on what.
   - Random imports across the tree.

---

## 8. Instructions for LLMs Using This Document

When you (LLM) are asked to create or modify a codebase:

1. **Detect / Choose an Architecture Style**
   - Small toy / script → simple layout is fine.
   - Growing app → prefer feature/domain-first.
   - Complex business domain → bounded contexts with DDD-ish layout.
   - Testable, long-lived core → ports & adapters / hexagonal style.

2. **Propose a File Tree Before Code**
   - Always output a proposed file tree first (as text).
   - Explain briefly why it’s structured that way (feature-first, layered, etc.).

3. **Default to Feature/Domain-First**
   - Top-level under `src/` should usually be `features/` or `contexts/` with domain names (e.g. `users`, `billing`, `catalog`).
   - Inside each feature, use consistent subfolders:
     - Backend: `domain/`, `application/`, `api/`, `infra/`.
     - Frontend: `components/`, `hooks/`, `api/`, `state/`, etc.

4. **Enforce Dependency Rules in Your Code**
   - Avoid imports that violate the intended direction (e.g. `domain` importing `infra`).
   - If necessary, introduce interfaces/ports to keep dependencies clean.

5. **Keep Shared Code Minimal and Intentional**
   - Only put things in `shared/` when they are used by multiple features.
   - If a helper is only used by `users/`, keep it inside `users/`.

6. **Maintain Change Locality**
   - When adding a feature or change, keep most of the new/changed files inside a single feature/context folder.
   - If changes span multiple features, consider whether this hints at:
     - A missing shared abstraction, or
     - Poorly drawn boundaries.

7. **Refactor Structures Gradually**
   - If the user’s current project is unstructured:
     - Identify natural clusters of files (by domain or responsibility).
     - Propose a reorganization plan (incremental steps).
     - Maintain backward compatibility where necessary (e.g., keep public APIs stable while moving internals).

8. **Document the Structure**
   - Always add or update a `ARCHITECTURE.md` or `README.md` explaining:
     - Chosen structure.
     - Dependency rules.
     - Conventions for adding new modules/features.

---

## 9. Example: Minimal `ARCHITECTURE.md` Template

You can reuse this in projects:

```markdown
# Architecture Overview

## High-Level Style

- Architecture: Feature-first, layered, modular monolith
- Top-level organization:
  - `src/features/*` — domain features (users, billing, etc.)
  - `src/shared/*` — shared utilities and cross-cutting concerns
  - `src/config/*` — configuration and wiring
  - `src/main.*` — application entrypoint

## Inside a Feature

Each feature directory follows:

- `domain/` — domain model and domain services
- `application/` — use cases, application services
- `api/` — HTTP handlers, controllers, route definitions
- `infra/` — database repositories, external API clients, messaging

Example:
```text
src/features/users/
  domain/
  application/
  api/
  infra/
```

## Dependency Rules

- `domain` depends only on language standard libs and `shared/domain` utilities.
- `application` can depend on `domain` and define ports/interfaces.
- `api` can depend on `application` and `domain`.
- `infra` can depend on `domain`, external libraries, and implements the ports from `application`.

Across features:

- Features may depend on `shared/`.
- Avoid direct deep imports into another feature’s internals; if needed, create explicit interfaces in `shared/` or a common module.

## Adding a New Feature

1. Create `src/features/<feature-name>/`.
2. Add `domain/`, `application/`, `api/`, `infra/` subfolders as needed.
3. Expose any externally-used API via:
   - HTTP routes in `api/`, or
   - Public application service in `application/`.
4. Update this document if the structure changes in a significant way.
```

---

End of document.
