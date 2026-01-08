# **Business Logic Placement & Model-Level Architecture**

A visual \+ narrative explanation of where business logic belongs in the playbook.

---

## **Visual Diagram**

```
graph TD
  subgraph Domain Layer
    R[Reducers / Statecharts]
    V[Value Objects / Enums]
    E[Domain Events]
  end

  subgraph Application Layer
    C[Command Handlers]
    S[Sagas / Orchestrators]
  end

  subgraph Infrastructure Layer
    M[ORM Models / DB]
    O[Outbox / Queues]
    A[APIs / External Systems]
  end

  R -->|Pure transitions| V
  R -->|Emit| E
  C -->|Calls| R
  S -->|Coordinates| R
  C -->|Uses| M
  S -->|Publishes| O
  O -->|Delivers| A
```

---

## **Narrative**

### **Domain Layer**

* **Reducers / Statecharts**: The mathematical heart of the system. Pure, deterministic, replayable.

* **Value Objects / Enums**: Make illegal states unrepresentable.

* **Domain Events**: Immutable facts that describe what happened.

### **Application Layer**

* **Command Handlers**: Validate intent, load state from repos, run reducers, persist results, and publish events.

* **Sagas / Orchestrators**: Coordinate across multiple aggregates using domain events. No embedded rules.

### **Infrastructure Layer**

* **ORM Models / DB**: Persistence only — rows, tables, relations. Thin adapters between DB and domain.

* **Outbox / Queues**: Deliver events reliably.

* **External Systems**: APIs, messaging buses, and services.

---

## **Why not put business logic in models?**

* Breaks purity and determinism — can’t replay or test cleanly.

* Couples rules to IO and persistence concerns.

* Produces god models that are brittle and hard to evolve.

Models remain useful for:

* Enforcing DB integrity (constraints, casts).

* Read-optimized helpers (scopes, query building).

---

## **Migration Path from Fat Models**

1. Inventory logic in existing models.

2. Extract core rules into reducers and value objects.

3. Route commands through handlers that call reducers.

4. Keep models as thin persistence adapters.

5. Validate by replaying production logs against new reducers.

---

## **Why this fits the playbook**

* Aligns with reducer purity and property testing.

* Supports time travel debugging and event sourcing.

* Enforces clean hexagonal boundaries: **decisions in domain, effects in infra**.

---

*End of document — Business Logic Architecture Diagram & Narrative*

