---

## 2) Appendix: Concurrency & Time

```md
# Appendix L: Concurrency & Time

Purpose  
Standardize how services **use time, parallelism, and scheduling**, so reducers, FSMs, and reliability patterns stay predictable under load. Complements:

- Reducer concurrency section  [oai_citation:3‡appendix-pattern-playbook.md](sediment://file_00000000246c71f599f201a49d965bab)  
- FSM/time modeling rules (TIMEOUT/EXPIRE as events)   
- Reliability toolkit (timeouts, retries, backpressure, circuit breakers)  [oai_citation:4‡appendix-pattern-playbook.md](sediment://file_00000000246c71f599f201a49d965bab)  

---

## L1. Non‑negotiables

1. **No hidden concurrency.**  
   If something can run in parallel, it is:
   - Explicit in the code (distinct tasks/jobs/queues),
   - Named in the design doc / ADR, and
   - Logged/observable.   

2. **One writer per aggregate / entity.**  
   All events for a single domain aggregate are processed **sequentially** (actor/mailbox, queue partition, or DB row lock).  [oai_citation:5‡appendix-pattern-playbook.md](sediment://file_00000000246c71f599f201a49d965bab)  

3. **Deadlines propagate.**  
   Time budgets flow across calls; no “fire and forget” without an explicit contract.   

4. **Time is modeled as data.**  
   Timers, expiries, and SLAs appear:
   - As fields on state/context, and
   - As explicit events (`EXPIRE`, `TIMEOUT`, `RENEW`).  [oai_citation:6‡appendix-fsm.md](sediment://file_0000000077a471f5bd470ba007e8e5bb)  

---

## L2. Concurrency models we actually use

### L2.1 Per‑aggregate serialization (default)

For any entity with a clear ID (`order_id`, `account_id`):

- All its events go through **one reducer instance at a time**.   
- Strategies:
  - Actor mailbox keyed by ID
  - Queue partition / Kafka topic keyed by ID
  - DB row lock around “load + reduce + persist”

This preserves:
- Determinism per aggregate
- Simple invariants (no lost updates, no interleaving)

### L2.2 Actor / CSP model for flows

For multi‑component flows:

- Use **actors** (services, workers) passing messages over queues/channels instead of shared mutable state.   
- Each actor:
  - Owns its state
  - Processes messages one at a time
  - Uses FSM + reducers internally for behaviour

### L2.3 Shared state + transactions (limited)

We only allow “multiple writers on shared data” when:

- Protected by:
  - DB transactions with clear isolation rules
  - “STM‑style” transactional updates on in‑memory state (rare)
- Carefully documented invariants and contention expectations.

Default: do *not* share mutable in‑memory state between tasks; use messages.

---

## L3. Deterministic concurrency

Goal: **same inputs → same outputs** even if scheduling changes.

Patterns:

1. **Pure core, effectful shell**  
   - All concurrency sits in the shell.
   - Shell pulls events, feeds them into pure reducers, persists results.   

2. **Per‑aggregate log**  
   - Append events in arrival order (or legal order),
   - Replay with reducers for debugging/time‑travel.   

3. **Deterministic schedulers in tests**  
   - Simulation tests interleave operations from multiple aggregates in different orders,
   - Assert that per‑aggregate invariants always hold.   

---

## L4. Time semantics

### L4.1 Wall‑clock vs logical time

- **Wall‑clock time**: `now()` from a Clock port (real world).  
- **Logical time**: counters/versions/sequence numbers derived from events.

Rules:

- Never infer causality from wall‑clock alone (clocks skew).
- For ordering, rely on:
  - Log position / sequence number
  - Aggregate version
  - Logical clocks (see below)

### L4.2 Deadlines & timeouts (service level)

Reuse reliability toolkit rules but make them explicit:   

- Every “top‑level” operation carries a **deadline**:
  - `expires_at` in context (HTTP header, message metadata)
  - Sub‑calls compute `remaining = deadline - now()`
- Defaults:
  - 1 global service default
  - Stricter budgets for hot paths
- If `remaining <= 0`:
  - Abort work and return a deadline‑exceeded error
  - Log the budget and path

### L4.3 Timers as events

FSM rule: **timers are not magic**; they are events:   

- Domain holds `expires_at` in its state/context.
- A scheduler/worker:
  - Scans for due items,
  - Emits `TIMEOUT`/`EXPIRE` events into the same event pipeline,
  - Reducer handles transitions for those events.

Advantages:

- Unified observability (timeouts show up as domain events)
- Replayable behaviour (same `EXPIRE` events → same outcomes)

---

## L5. Logical clocks & causality (advanced)

When you have **replication / CRDTs / multi‑region**:   

- Use **logical clocks** for “happens‑before” relationships:
  - Per‑replica counters
  - Vector clocks where necessary
- Store in event metadata:
  - `origin_region`, `origin_replica`
  - `clock` (e.g., `{replica: N, ...}`)

Guidelines:

- For **conflict resolution**, prefer:
  - CRDT merges for collaborative docs / sets
  - “Last writer wins” only when you can justify the semantics
- For debugging, expose clocks in logs/inspection tools for out‑of‑order weirdness.

---

## L6. Choosing a concurrency strategy

Decision table (simplified):

| Force / Symptom                        | Preferred pattern                                | Notes |
| ------------------------------------- | ----------------------------------------------- | ----- |
| Single entity with heavy invariants   | Per‑aggregate serialization (actor/queue/lock)  | Keep reducers local and pure. |
| Multi‑entity saga with compensation   | Orchestrated saga + per‑aggregate mailboxes     | No cross‑aggregate locks.  [oai_citation:7‡appendix-pattern-playbook.md](sediment://file_00000000246c71f599f201a49d965bab) |
| High read/write throughput on shared data | Append‑only log + projections                     | Projections rebuilt via incremental updates. |
| Many concurrent readers, few writers  | Reader/Writer split with caching and explicit invalidation | Cache writes go through single pipeline. |
| In‑memory correlation across tasks    | Actor with internal state, messages for requests | Avoid ad‑hoc shared maps. |

If in doubt, prefer:
- One writer per thing
- Messages over shared mutable state
- Logs + reducers for derived state

---

## L7. Stack‑specific guidelines

### Rust

- Use **tokio tasks + channels** for concurrency; avoid shared `Arc<Mutex<_>>` unless truly necessary.   
- Group timeouts via `tokio::time::timeout` around external calls; map to domain errors instead of panicking.
- For per‑aggregate processing:
  - Use channels keyed by ID, or
  - Use stream partitions (Kafka/NATS) mapped to reducer workers.
- Tests:
  - Use deterministic clocks by injecting a `Clock` port.
  - Use manual advancement (`advance_time`) in tests to trigger expiries.

### Laravel / PHP

- Concurrency = **queues + Horizon workers**, not threads.   
- One aggregate instance per job:
  - Job loads state, calls reducer, persists, emits events/outbox.
- Use:
  - Job‑level timeouts + max retries
  - Idempotency keys on jobs that may be re‑queued.   
- Cron/schedulers emit timer events into the same pipeline as other events.

### Nuxt / TypeScript

- UI: use **abortable fetch** (AbortController) and keep in-flight requests explicit in UI state.
- Node services: use async/await + queues, not fire‑and‑forget promises.
- For RAG calls, treat them as **effects with deadlines**:
  - If context retrieval or embedding is slow, respect upstream deadline, degrade gracefully.   

---

## L8. Testing & observability

### L8.1 Testing patterns

- **Simulation tests**:
  - Deterministic scheduler that:
    - Chooses which queue/job to process next,
    - Advances a fake clock,
    - Asserts invariants hold regardless of interleaving.   
- **Load tests**:
  - Validate that backpressure kicks in before meltdown (queue limits, circuit breakers).  [oai_citation:8‡appendix-pattern-playbook.md](sediment://file_00000000246c71f599f201a49d965bab)  

### L8.2 Observability requirements

Extend the playbook’s observability plan:   

- Metrics:
  - Per‑queue depth and age
  - Per‑state occupancy for FSMs
  - Deadline overrun counts
- Logs:
  - Every timeout/expiry as structured event (`{aggregate_id, from, to, event: "TIMEOUT"}`)
  - Concurrency decisions (e.g., shard id, actor id)
- Traces:
  - One span per significant async boundary (queue publish/consume, saga step)

---

## L9. LLM guidance hooks

When wiring MCP/RAG:   

- Pin a “**Concurrency & Time Ground Rules**” snippet:
  - “One writer per aggregate; timers as events; deadlines propagated.”
- Review questions for AI‑generated diffs:
  - ❓ “Does this introduce shared mutable state between tasks?”
  - ❓ “Is this new timer represented as an event and state field?”

This keeps emergent concurrency issues from sneaking in via generated code.