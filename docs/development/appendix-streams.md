# Appendix: Streams Deep Dive

Purpose: deep, language-agnostic reference for designing, reasoning about, and operating data streams from first principles to advanced patterns.

## Table of Contents
- Core Mental Model
- Lifecycle and Contracts
- Time Semantics
- Ordering and Delivery Semantics
- Backpressure and Flow Control
- Buffering and Admission Control
- State, Idempotency, and Consistency
- Topologies and Composition
- Reliability and Recovery
- Performance and Resource Management
- Observability and Governance
- Advanced Theory and Principles
- Control and Stability
- Traffic Shaping and Scheduling
- Tail Latency and Resilience
- Temporal Logic and Determinism
- Data Semantics and Consistency Models
- Formal Models and Algebra
- Queueing and Sizing Heuristics
- Watermarks and Windows Deep Dive
- CRDTs and Convergence Patterns
- Replay, Checkpointing, and WAL Strategies
- Anomaly Catalogue and Mitigations
- Migration and Evolution
- Operations Runbook Prompts
- Kafka Applied Patterns
- Formal Proof Sketches / Examples
- Formal Specification Snippets (TLA+, Coq/Lean)
- Runtime Mappings: Flink via TLA+
- Runtime Mappings: Flink Checkpointing EOS (TLA+)
- Lean/Coq Operator Proofs
- Runtime Mappings: Beam Model (PCollection/Window/Trigger)
- Runtime Mappings: Kafka Streams (Topology, EOS)
- Runtime Mappings: JavaScript (RxJS, Web Streams, Async Iterators)
- RxJS Patterns and Examples
- Node Streams Patterns
- RxJS Event-Time Patterns and Footguns
- Node Streams Error Handling Patterns
- RxJS Watermark Operator Example
- Node Streams Performance Tuning
- Production-Grade RxJS Windowing
- Node Streams Benchmarking Tips
- Benchmark Harness Templates (Node, Rust)
- Cross-Language Comparison Tips
- Future Directions and Frontier Ideas
- Testing and Verification
- Design Checklist
- Applied Pattern Example: Credit-Based Flow Control, Watermarking with Lateness, Replay-Safe Sink

## Core Mental Model
- Stream = potentially unbounded sequence over time; think of a function from a time index to values or value+timestamp pairs.
- Treat time as a first-class dimension; most errors come from ignoring time semantics.
- Differentiate stream values (events) from signals/behaviors (time-varying state).
- Prefer declarative dataflow graphs over callback webs; operators form an algebra (map/filter/scan/window/join).
- Purity in transforms; isolate side effects at the edges to preserve referential transparency and replayability.

## Lifecycle and Contracts
- Explicit lifecycle: creation → transformation → consumption → teardown; define cancellation, completion, and error propagation.
- Contract upfront: throughput targets, latency SLOs, allowed loss, ordering guarantees, retry policy, shutdown behavior.
- Cold vs hot: cold produces per-subscription; hot is shared and needs multicast discipline.
- Resource safety: every acquisition must have a bound and a release path; pair subscriptions with disposers/finalizers.

## Time Semantics
- Event time (when it happened), processing time (when observed), ingestion time (when admitted). Choose per operator and be explicit.
- Watermarks: lower bound on unseen event times; allow windows to declare completeness without waiting forever.
- Lateness: slack allowed after a watermark; late events can trigger corrections (upsert/retraction) or be dropped.
- Clocks and drift: assume skew; guard with synchronization tolerance and monotonic comparisons where possible.
- Scheduling: virtual time/test schedulers for determinism; real schedulers for production latency. Keep the two aligned conceptually.

## Ordering and Delivery Semantics
- Ordering scopes: total order (costly), per-key order (common), causal order (vector clocks/Lamport), or order-agnostic.
- Delivery modes: at-most-once (lossy, simple), at-least-once (requires dedup/idempotency), effectively-once (transactional or idempotent sinks).
- Duplicates and reordering are normal under retries; design operators to be order-robust unless strict order is required.
- Idempotent sinks and monotonic state updates make at-least-once acceptable in most systems.

## Backpressure and Flow Control
- Backpressure is a control loop: producer is the plant, consumer demand is the controller. Stability needs bounded gain and feedback.
- Pull-based demand (async iteration, credits) is the simplest: consumer asks for N; producer never exceeds demand.
- Push-based needs signaling: pause/resume hooks or explicit credit tokens; without it, you rely on buffers (hidden debt).
- Concurrency caps = backpressure: limit parallel in-flight work to keep latency bounded.
- Avoid unbounded buffers; they mask problems until they exhaust memory and destroy latency.

## Buffering and Admission Control
- Little’s Law (L = λW) connects arrival rate, queue length, and wait time; use it to size buffers and set expectations.
- Bounded queues force explicit policies when full:
  - Drop-tail: simple, can starve the head.
  - Drop-head: keeps recent items (freshness-first).
  - Random early drop: prevents thundering herd and synchronization.
  - Priority shedding: drop by key/priority class.
  - Spill: persist to disk/kv when memory is full; resume later.
- Coalescing: sample/latest, throttle, debounce to reduce volume when freshness > completeness.
- Batching: trades tail latency for throughput; tune to SLOs and downstream limits.

## State, Idempotency, and Consistency
- Stateful operators (aggregations, joins) must scope state: per-key, per-window, or global with strong bounds.
- Eviction: TTLs, count caps, LRU/LFU by key; avoid unbounded cardinality growth.
- Idempotency layers:
  - Deterministic transforms + pure state transitions.
  - Sequence numbers or version vectors to reject duplicates/out-of-order.
  - CRDTs when concurrency and partition tolerance are required with eventual convergence.
- Checkpointing + replay vs. changelog + materialized view; choose based on recovery time vs. write amplification.
- Side effects isolated: transactional outbox or idempotent writes to avoid double-application on replay.

## Topologies and Composition
- Linear pipelines: simplest latency story; fewer coordination points.
- DAGs with fan-out/fan-in: require explicit merge semantics (ordering, dedup).
- Feedback edges: treat as control systems; add delay/gain limits (debounce, min interval) to prevent oscillation.
- Partitioning: hash or range by key for parallelism; stable partitioning aids reproducibility and recovery.
- Fusion: combine adjacent operators to reduce overhead; balance against modularity and backpressure visibility.

## Reliability and Recovery
- Restart boundaries: define which stages restart independently; align with partitions for isolation.
- Checkpoints: periodic snapshots + offsets/watermarks; tradeoff between checkpoint cost and recovery time.
- WAL/change capture: log-before-apply for durability; pair with deterministic reprocessing for correctness.
- Exactly-once is “effectively-once”: combine at-least-once delivery with idempotent/transactional sinks.
- Poison-pill handling: retries with jitter/backoff and max attempts; quarantine or DLQ with metadata for inspection.

## Performance and Resource Management
- Avoid blocking in async loops; offload CPU/blocking I/O to dedicated pools.
- Preallocate/pool buffers; minimize per-event allocations (GC pressure in managed runtimes).
- Serialization costs dominate at high throughput; prefer zero-copy paths and columnar/batched formats when possible.
- Cache locality: co-locate compute and data; minimize cross-core chatter for partitioned workloads.
- Concurrency tuning: per-operator caps; work stealing for imbalance; avoid oversubscription that causes tail latency inflation.

## Observability and Governance
- Metrics to emit: ingress rate, egress rate, service time, queue depth, backlog age, watermark lag, drop count, retry count, failure rate.
- Tracing: propagate context across async boundaries to recover causality; mark spans per stage.
- Logging: structured, bounded, and sampled; avoid hot-loop logging.
- Introspection hooks: expose current watermarks, buffer utilization, and in-flight counts for operators.
- Governance: schemas and contracts versioned; compatibility tested before rollout; feature flags for new operators.

## Advanced Theory and Principles
- Algebraic thinking: operators should be lawful (associative folds, idempotent merges) to enable reordering and parallelization.
- Monotonicity: prefer monotone transformations so partial results can only grow in information; simplifies retractions and replay.
- Information theory: compress/encode near source; watch entropy vs. overhead; use framing to preserve message boundaries.
- Category/dataflow view: streams as morphisms; composition must preserve totality, error semantics, and cancellation semantics.
- Ergonomics vs. guarantees: prefer explicit types for time, key, and ordering; make defaults safe (bounded, cancellable, error-aware).

## Control and Stability
- Treat backpressure as feedback control; avoid integral windup by bounding queued work and clearing on cancellation.
- Stability requires negative feedback: credits/token windows, demand signals, or rate limiters; avoid positive feedback loops without damping.
- Hysteresis for flapping: add thresholds and cooldowns for pause/resume decisions to prevent oscillation.
- Graceful degradation: shed load by priority when nearing limits; fail fast rather than slow death by queue growth.

## Traffic Shaping and Scheduling
- Token bucket for average rate with burst tolerance; leaky bucket for smoother, constant outflow; combine per-tenant for fairness.
- Weighted fair queueing (WFQ) or deficit round robin (DRR) to prevent loud neighbors from starving others.
- Concurrency budgets per stage: separate limits for I/O vs. CPU-bound work; avoid sharing one global knob.
- Admission control at ingress: reject early when downstream is constrained; use backoff hints to callers.
- Co-location and pinning: partition-aware scheduling to keep hot keys near state; reduces cross-node chatter.

## Tail Latency and Resilience
- Hedged requests sparingly; cap concurrency to avoid amplifying load; cancel losers promptly.
- Bulkheading: isolate tenants/keys/pipelines with resource pools and queues to contain failure and contention.
- Retry discipline: bounded attempts with jitter; respect idempotency and non-retriable errors; surface retry budgets as metrics.
- Brownout modes: temporarily disable optional work (enrichment, heavy joins) under pressure to protect core latency SLOs.
- Chaos/fault drills: inject slowdowns, drops, reorderings to validate steady-state and degraded behavior.

## Temporal Logic and Determinism
- Deterministic processing: given the same input log and configuration, outputs are reproducible; avoids heisenbugs on replay.
- Temporal operators: window, interval, delay should be defined against explicit clocks; document how they interact with watermarks.
- Causality: preserve happens-before where required; otherwise design for eventual convergence with commutative updates.
- Virtual time for proofs: model operator semantics in virtual time to reason about reordering and lateness.

## Data Semantics and Consistency Models
- Encapsulate time and identity: events carry key, version/sequence, timestamp, and causal metadata where needed.
- Consistency choices: strong/linearizable (rare in streams), per-key order with eventual consistency (common), CRDTs for mergeable state.
- Idempotent effect pattern: make outputs a pure function of (key, version) to neutralize duplicates.
- Correct-by-construction sinks: append-only logs with compaction, or materialized views with deterministic upserts.
- Schema evolution: backward/forward compatible encodings; reject or quarantine incompatible payloads early.

## Formal Models and Algebra
- Streams as coalgebras: behavior described by next-state/next-output; encourages total, side-effect-free transforms.
- Process calculi: model interaction with CSP/π-calculus for channel safety and deadlock analysis.
- Denotational vs. operational semantics: specify what (function of input log to output log) before how (runtime mechanics).
- Algebraic laws: map/filter fusion; associativity of merge; commutativity/idempotence of reducers; distributivity of windowed folds when possible.
- Lattices: design state as join-semilattice to enable monotone, convergent updates (basis for CRDTs and mergeable state).

## Queueing and Sizing Heuristics
- Little’s Law: `L = λW`; given target wait `W` and arrival `λ`, size buffers/workers so `L` stays bounded.
- M/M/1 intuition: utilization `ρ = λ/μ`; as `ρ→1`, latency explodes; keep `ρ` < ~0.7 for stable tails.
- Parallel servers: split by key/partition to approximate M/M/k; watch skew—hot keys violate independence.
- Concurrency caps: set `k ≈ ceil(λ * SLO)` where `SLO` is max service time; adjust via observed p90/p99.
- Backoff budgets: jittered exponential with max attempts; expose remaining budget to avoid storms.

## Watermarks and Windows Deep Dive
- Watermark computation: `watermark = min(source_time_seen) - max_skew`; if multiple sources, take min across sources or use aligned barriers.
- Allowed lateness: choose `Δ` from empirical tail of arrival skew; track how often events arrive beyond `Δ`.
- Window types:
  - Tumbling: non-overlapping; simpler state.
  - Sliding: overlapping; more state, better resolution.
  - Session: gaps-based; require heuristics for gap length and merging.
- Completeness signals: watermark or count-based; prefer watermark + lateness for event time correctness.
- Corrections: retract/upsert downstream with versioned aggregates to handle late arrivals cleanly.

## CRDTs and Convergence Patterns
- Choose lattice per data shape: GCounter/PNCounter for counts; GSet/2PSet/OR-Set for membership; LWW-Register for last-writer-wins; MV-Register when conflicts must be preserved; GMap for nested structures.
- Design join as commutative, associative, idempotent; ensures convergence under reordering/duplication.
- Causal context: include dots/vector clocks when deletion or conflict resolution depends on causality.
- Bounded growth: use tombstone compaction or reset epochs for long-lived sets/maps.
- Hybrid: CRDT for availability + periodic compaction into strongly consistent store for size and queryability.

## Replay, Checkpointing, and WAL Strategies
- WAL-first: append input offsets and operations before applying; enables redrive after crash.
- Checkpoint interval: trade runtime overhead vs. recovery time; shorter intervals reduce catch-up time but add steady-state cost.
- Determinism: ensure pure transforms and partition-stable assignment so replay reproduces outputs.
- Idempotent sinks: sequence/version gating; transactional outbox to decouple side effects from commit.
- Split checkpoints: operator state separate from source offsets to allow partial rollbacks when safe.

## Anomaly Catalogue and Mitigations
- Unbounded queues → OOM/latency blowup: mitigate with bounds + shed/spill.
- Reordering breaking windows: mitigate with watermarks + lateness + corrections or CRDT aggregates.
- Duplicate side effects on replay: mitigate with idempotent sink/outbox and version checks.
- Hot keys causing imbalance: mitigate with key-splitting (subkeys), skew-aware sharding, or hotspot pooling.
- Feedback oscillation: mitigate with debounce, hysteresis, and bounded gain.
- Log divergence after replay: mitigate with deterministic processing, explicit versions, and compaction of old states.

## Migration and Evolution
- Schema evolution: add fields as optional; avoid breaking renames; use compatibility tests on sample payloads.
- Rolling upgrades: feature-flag new operators; dual-run (shadow) with diffing before cutover.
- State migrations: version state; include migration functions; checkpoint pre-migration and allow rollback.
- Replay after change: validate determinism; if semantics change, snapshot and cut new log lineage.

## Operations Runbook Prompts
- “What is backlog age and watermark lag?” → tells if time semantics are slipping.
- “Which queues are at cap and shedding?” → pinpoints backpressure failures.
- “Who are the hot keys/partitions?” → guides rebalancing.
- “What are p99 service times per operator?” → reveals bottlenecks and head-of-line blocking.
- “What is retry rate and DLQ volume?” → surfaces instability or upstream flakiness.

## Kafka Applied Patterns
- Partitioning and ordering: only per-partition order is guaranteed; key by the dimension that requires order and affinity; avoid workflows needing global order.
- Producer discipline: enable idempotence, `acks=all`, set `min.insync.replicas` ≥ 2; cap `max.in.flight.requests.per.connection` (1 for strict order with retries, small >1 if you accept potential reorder).
- Exactly/effectively-once: use transactions (`transactional.id`) to atomically write outputs and commit input offsets; pair with idempotent sink semantics (upsert by key+version) to neutralize duplicates on replay or EOS gaps.
- Offset handling: commit offsets only after side effects are durable; with transactions, use “read-process-write” and send offsets to the transaction; without transactions, use outbox pattern or idempotent sink plus idempotent offset store.
- Backpressure: consumers pause partitions when downstream is saturated; bound in-flight per partition; tune fetch sizes and `max.poll.interval.ms` to avoid rebalance churn; producers throttle via batching and `linger.ms` tuned to latency budget.
- Rebalances: prefer cooperative sticky rebalancing to reduce duplicate processing; on crash recovery, design for replay with idempotent sinks.
- Storage policy: use compaction for command/event topics that can be reduced by key; retention for audit/replay; compaction + monotone updates yields deterministic recovery.
- Lag as signal: monitor consumer lag per partition; scale consumers or partitions based on sustained lag; investigate hot keys if lag is skewed.

## Formal Proof Sketches / Examples
- Theorem (CRDT convergence): Let `(S, ⊔)` be a join-semilattice, updates `u_i ∈ S`. Each replica applies updates via `state := state ⊔ u_i`. Because `⊔` is associative, commutative, and idempotent, for any permutation with duplicates of updates, the final state is `⊔{u_i}` (LUB). Thus reordering/duplication cannot change the limit; replicas converge if they eventually receive the same set of updates. Proof: follows from semilattice properties; LUB is unique.
- Theorem (effectively-once via versioned upsert): Assume total order on versions per key and sink rule “apply iff `v > stored_version(k)`.” For any multiset of deliveries with reordering/duplication, the final stored value for `k` corresponds to the payload at `max version`. Proof: invariant `stored_version` monotone increasing; any delivery with `v ≤ stored_version` is a no-op; therefore the only state transitions are strictly increasing in version and terminate at the maximum observed.
- Theorem (bounded in-flight with credits): Producer holds credit counter `c` initialized to `C`; send decrements if `c > 0`, ack increments. Invariant: `c + inflight = C` and `0 ≤ c ≤ C`. Proof by induction on transitions; send reduces `c` and raises `inflight` symmetrically, ack reverses; thus `inflight ≤ C` always. Liveness requires fairness on ack path.
- Theorem (watermark stability with lateness): For window `[a, b)`, watermark `W` is lower bound on unseen event times; allowed lateness `Δ`. If `W ≥ b + Δ`, no future admissible event has `event_time < b`; thus any operator with drop-or-correct policy produces stable output for that window. Proof: by definition of watermark/lateness bounds.
- Theorem (deterministic replay equivalence): Let pipeline be composition of pure operators over a total order of events with deterministic partitioning. Let `F` be the total function from input log to output log. Reprocessing the same ordered log yields `F(log)` again. Proof: functional determinism; absence of external nondeterminism implies identical result for identical input.
- Theorem (Kafka EOS safety with transactions + idempotent sink): Assume partition-stable assignment, transactional writes of outputs and offsets, and sink rule as above. State space: `{committed, aborted}` transactions. Safety: no committed output without committed offset and vice versa (atomicity); thus any message is reflected at most once in outputs. Liveness under crash: uncommitted work is replayed; idempotent sink absorbs duplicates. Proof: Kafka protocol guarantees atomic commit of batch; idempotent sink ensures convergence if producer fencing or transaction abort happens.
- Theorem (outbox idempotency for side effects): With outbox table `(id, payload, sent_flag)`, where producer inserts rows and effect dispatcher sends only rows with `sent_flag=false` then atomically sets `sent_flag=true`, duplicates (replay or retried dispatch) cannot produce duplicate side effects if sink is idempotent on `id`. Proof: monotone flag plus idempotent downstream keyed by `id` yields at-most-once externally.
- Lemma (commutative/associative fold robustness): If reducer `⊗` is associative and commutative with identity `e`, then folding any permutation of a multiset yields same result: `fold ⊗ e (permute xs) = fold ⊗ e xs`. Proof: by induction on list length and associativity/commutativity; important for unordered merges.
- Lemma (window correction boundedness): For lateness `Δ`, number of corrections per window is bounded by count of events with `event_time < end` arriving in `(end, end+Δ]`. Proof: after `end+Δ`, operator rejects or diverts all such events; thus correction stream is finite per window.

## Formal Specification Snippets (TLA+, Coq/Lean)
- TLA+ sketch for credit-based flow control (bounded in-flight):
  ```tla
  ---------------- MODULE Credits ----------------
  EXTENDS Naturals
  CONSTANT C \* capacity
  VARIABLES c, inflight
  Init == c = C /\ inflight = 0
  Send == /\ c > 0 /\ c' = c - 1 /\ inflight' = inflight + 1
  Ack  == /\ inflight > 0 /\ c' = c + 1 /\ inflight' = inflight - 1
  Next == Send \/ Ack
  Inv  == c >= 0 /\ inflight >= 0 /\ c + inflight = C
  THEOREM Safety == Inv /\ [] (Next => Inv')
  ```
  Safety shows `inflight ≤ C` always; add liveness with fairness on `Ack`.

- TLA+ sketch for watermark stability (window completeness):
  ```tla
  VARIABLES W, latestClosed
  \* Assume W is nondecreasing lower bound on unseen event_time.
  CloseWindow == /\ W >= windowEnd
                 /\ latestClosed' = TRUE
  Stable == [](W >= windowEnd + lateness => latestClosed)
  ```
  Stability asserts once watermark passes `end + lateness`, window is closed and remains so.

- Lean/Coq-style lemma for CRDT join convergence (Lean flavor):
  ```lean
  variables {S : Type} [semilattice_sup S]
  def apply_updates (s : S) (us : list S) : S :=
    us.foldl (· ⊔ ·) s

  lemma convergence (s : S) (xs ys : list S)
    (hperm : xs.perm ys) :
    apply_updates s xs = apply_updates s ys :=
  by
    unfold apply_updates
    simpa [list.perm.foldl_eq] using hperm
  ```
  Shows order/duplication (under multiset equivalence) does not change final state when using `⊔`.

- Lean/Coq-style lemma for versioned upsert fixpoint:
  ```lean
  structure Event := (v : nat) (payload : α)
  def apply (cur : Event) (e : Event) : Event := if e.v > cur.v then e else cur

  lemma max_version_fixed (cur : Event) (es : list Event) :
    let final := es.foldl apply cur
    final.v = (cur.v :: es.map Event.v).maximum ∧
    final = cur ∨ final ∈ es :=
  by
    -- proof relies on total order of nat and monotone select of max
  ```
  Captures that final state corresponds to max version irrespective of order.

## Runtime Mappings: Flink via TLA+
- Flink window operator with watermark/lateness (tumbling window `[a,b)`):
  ```tla
  ---------------- MODULE FlinkWindow ----------------
  EXTENDS Naturals, Sequences
  CONSTANT WindowEnd, Lateness
  VARIABLES wm, buf, out, closed
  \* wm: watermark (nondecreasing), buf: seq of events (ts, payload), out: emitted aggregates, closed: bool

  Init == /\ wm = 0
          /\ buf = << >>
          /\ out = << >>
          /\ closed = FALSE

  Ingest(ev) == /\ ev.ts < WindowEnd + Lateness
                /\ buf' = Append(buf, ev)
                /\ UNCHANGED << wm, out, closed >>

  AdvanceWm(w) == /\ w >= wm
                 /\ wm' = w
                 /\ UNCHANGED << buf, out, closed >>

  Close == /\ wm >= WindowEnd + Lateness
           /\ ~closed
           /\ let agg == Aggregate(buf) in
              /\ out' = Append(out, agg)
              /\ closed' = TRUE
           /\ UNCHANGED wm

  Next == \E ev \in Events: Ingest(ev)
          \/ \E w \in Nat: AdvanceWm(w)
          \/ Close

  InvOrderStable == \A ev \in buf: ev.ts < WindowEnd + Lateness
  InvClosure == closed => wm >= WindowEnd + Lateness

  THEOREM Safety == Init /\ [] (Next => InvOrderStable /\ InvClosure)
  \* Liveness (eventual close) requires wm to eventually exceed WindowEnd + Lateness.
  ```
  Notes:
  - `Aggregate` presumed associative/commutative to tolerate event reorder in `buf`.
  - To model sliding/session windows, extend state with multiple buffers keyed by window/session ID and eligibility predicates.
  - To model allowed corrections, replace `closed` with stage that emits retractions/upserts until `wm >= end+Δ`, then freezes.

## Runtime Mappings: Flink Checkpointing EOS (TLA+)
- Sketch of barrier-aligned checkpoints for exactly-once:
  ```tla
  ---------------- MODULE FlinkCheckpoint ----------------
  EXTENDS Naturals, Sequences
  VARIABLES state, offset, snapshotting, barrierId, inbox, out

  Init == /\ state = InitState
          /\ offset = 0
          /\ snapshotting = FALSE
          /\ barrierId = 0
          /\ inbox = << >> \* buffered events awaiting barrier alignment
          /\ out = << >>

  Event(e) == /\ ~snapshotting
              /\ offset' = offset + 1
              /\ state' = Step(state, e)
              /\ UNCHANGED << snapshotting, barrierId, inbox, out >>

  Barrier(id) == /\ id = barrierId + 1
                 /\ snapshotting' = TRUE
                 /\ barrierId' = id
                 /\ inbox' = << >>
                 /\ UNCHANGED << state, offset, out >>

  Align(e) == /\ snapshotting
              /\ inbox' = Append(inbox, e)
              /\ UNCHANGED << state, offset, snapshotting, barrierId, out >>

  Snapshot == /\ snapshotting
              /\ SnapshotState(state, offset) \* abstract write to durable store
              /\ snapshotting' = FALSE
              /\ \E evs \in Seq(Events): inbox' = << >> /\ state' = Fold(Step, state, evs) /\ offset' = offset + Len(evs)
                 \* replay buffered events after snapshot in order
              /\ UNCHANGED << barrierId, out >>

  Next == \E e \in Events: Event(e)
          \/ \E e \in Events: Align(e)
          \/ \E id \in Nat: Barrier(id)
          \/ Snapshot

  InvOrder == snapshotting => inbox \*holds events post-barrier; order preserved
  InvExactlyOnce == \* For any persisted snapshot (state, offset), replaying from offset+1 yields same out as if uninterrupted.
  ```
  Notes:
  - `SnapshotState` is assumed atomic/durable.
  - In real Flink, alignment per input channel; this abstraction collapses multiple channels into `inbox`.
  - Exactly-once arises from barrier alignment + replay from offset; idempotent sinks or two-phase commit needed at sinks.

## Runtime Mappings: Beam Model (PCollection/Window/Trigger)
- Beam primitives:
  - PCollection: multiset of elements with timestamps (event time) and window assignments.
  - WindowFn: assigns element to one or more windows; must be deterministic.
  - Trigger: determines when to emit panes; after firing, can continue (with accumulation or discarding) until final watermark + allowed lateness.
  - Accumulation modes: accumulating vs. discarding vs. accumulating-and-retracting.
  - Watermark: lower bound on unseen event times; allowed lateness per PCollection.
- EOS mapping:
  - If runner provides exactly-once, state + timers are checkpointed; retries replay inputs with the same windowing/timer semantics.
  - Idempotent sinks or de-dup on output keys still recommended when crossing system boundaries.
- Determinism requirements:
  - WindowFn and Trigger must be deterministic to guarantee consistent pane boundaries on replay.
  - Accumulators must be associative/commutative for parallelism; for accumulating-and-retracting, require invertibility.
- Practical patterns:
  - Use fixed/tumbling windows for simplicity; sessions for bursty workloads; avoid global windows unless input is bounded.
  - Triggers: `afterWatermark` + early firings (`processingTime` or `elementCount`) and late firings bounded by allowed lateness; combine with accumulating-and-retracting for correctness under lateness.
  - Set allowed lateness to empirical skew tail; set GC time (window expiration) accordingly to reclaim state.

## Runtime Mappings: Kafka Streams (Topology, EOS)
- Topology semantics:
  - KStreams are per-partition ordered logs; KTables are changelog-backed materialized views.
  - Operators execute per partition; joins require co-partitioned topics by key.
  - Repartition topics inserted when key changes; beware of added network/shuffle cost and state blowup.
- EOS model (exactly-once v2):
  - Uses idempotent producers + transactions to atomically write changelog/output and commit consumed offsets.
  - Processing guarantee configured to exactly-once-v2 ensures per-task transactions with fencing.
- State:
  - RocksDB state stores backed by changelog; recovery via restoring changelog then replaying recent segments.
  - Caching layer: write-back cache can delay flush; on crash, only flushed data persisted, but changelog replays missed updates.
- Windowing:
  - Hopping/tumbling/session windows supported; retention must cover window + grace period (`grace` = allowed lateness).
  - Suppression operator can emit only final results (after grace) or intermediate with buffering; must be bounded.
- Backpressure and flow:
  - Kafka clients handle pull-based flow; per-partition ordering and fetch window act as implicit backpressure.
  - Use `max.in.flight` and `linger` to tune producer; consumer `max.poll.interval` and `max.partition.fetch.bytes` to avoid rebalances.
- Idempotency at sinks:
  - When writing to external sinks, keep idempotent upsert keyed by logical key + seq/version; Kafka EOS covers Kafka topics, but not external systems.

## Runtime Mappings: JavaScript (RxJS, Web Streams, Async Iterators)
- RxJS essentials:
  - Prefer cold observables (`defer`, `from`, `Observable` ctor) and share at the edges (`share`, `shareReplay({bufferSize, refCount})` bounded).
  - Control concurrency: `mergeMap(fn, concurrency)`, `switchMap` for latest-wins, `exhaustMap` to block reentry, `concatMap` for ordering.
  - Teardown: always tie subscriptions to `takeUntil`, `finalize`, or `using`; avoid orphaned subscriptions in components.
  - Backpressure: use `throttleTime`, `auditTime`, `sample`, `bufferTime/count`, and bounded `mergeMap` concurrency; avoid unbounded `Subject` as a bus.
  - Error discipline: surface errors; use `retryWhen` with jitter/backoff; avoid `catchError` that swallows without metrics.
  - Scheduling: use `observeOn`/`subscribeOn` intentionally (e.g., `animationFrameScheduler` for UI, `queueScheduler` for deterministic recursion, `asyncScheduler` to yield).
  - Testing: marble tests with `TestScheduler`; model time-sensitive logic deterministically.
  - Multicast patterns: `share` for hot source; `shareReplay` bounded when late subscribers need last value; avoid infinite buffer `ReplaySubject`.
  - UI feedback: debounce inputs, cancel in-flight async via `switchMap` to promise/observable that respects abort signals.

- Web Streams API:
  - Backpressure built-in via `ReadableStreamDefaultReader.read()` (pull) and controller `desiredSize`; use `pipeThrough`/`pipeTo` to compose.
  - Set `highWaterMark` and `size` functions thoughtfully; avoid default huge buffers in high-throughput paths.
  - TransformStream for map/filter; use `AbortController` to cancel pipelines; always `await stream.closed` or handle `cancel`.
  - In browsers, integrate with `fetch().body` and compression streams; in Node 18+, Web Streams interop with `stream.Readable.fromWeb`.

- Async iterators in Node:
  - Prefer `for await...of` over `data` events; combine with `Readable.from(async function*)` for pull-friendly producers.
  - Use `stream.promises.pipeline` to wire backpressure across transforms; set `highWaterMark` to bound buffers.
  - Avoid mixing `data` event mode and async iteration on the same stream.

  - Frontend patterns:
    - Event streams (UI): debounce inputs, `switchMap` to cancel stale requests, guard side effects with `takeUntil` on component teardown.
    - Animation and time: use `animationFrameScheduler` or `requestAnimationFrame` driven streams; avoid `setInterval` drift.
    - Resource cleanup: tie subscriptions to component lifecycle (e.g., React `useEffect` return cleanup) or signals/abort controllers.
    - Memory safety: prefer `fromEventPattern` with explicit unsubscribe to avoid leaking DOM listeners.

## RxJS Patterns and Examples
- Latest-wins async with cancellation (e.g., search box):
  ```ts
  const search$ = fromEvent(inputEl, 'input').pipe(
    map(e => (e.target as HTMLInputElement).value.trim()),
    debounceTime(150),
    distinctUntilChanged(),
    switchMap(term =>
      term === '' ? of([]) : from(fetch(`/api/search?q=${encodeURIComponent(term)}`)).pipe(
        switchMap(res => res.json()),
        catchError(err => {
          console.error(err);
          return of([]); // degrade gracefully
        })
      )
    )
  );
  const sub = search$.subscribe(renderResults);
  // teardown: sub.unsubscribe() or tie to takeUntil(destroy$)
  ```

- Bounded concurrency for work queue:
  ```ts
  const tasks$ = new Subject<Task>();
  const results$ = tasks$.pipe(
    mergeMap(task => defer(() => runTask(task)), 4), // cap in-flight at 4
    share()
  );
  results$.subscribe(onResult, onError);
  ```

- Backpressure via sampling/throttling:
  ```ts
  const fastClicks$ = fromEvent(btn, 'click');
  const sampled$ = fastClicks$.pipe(sampleTime(500)); // keep latest every 500ms
  const throttled$ = fastClicks$.pipe(throttleTime(200, asyncScheduler, { leading: true, trailing: true }));
  ```

- Windowed aggregation with late corrections (simulation):
  ```ts
  const events$ = ...; // emits {ts, value}
  const windowed$ = events$.pipe(
    groupBy(e => Math.floor(e.ts / WINDOW_MS)),
    mergeMap(group$ =>
      group$.pipe(
        bufferTime(WINDOW_MS), // simple tumbling; for lateness you’d need custom buffering + watermark
        map(events => ({
          window: group$.key,
          sum: events.reduce((s, e) => s + e.value, 0)
        }))
      )
    )
  );
  ```
  For true event-time with lateness, combine with explicit watermark logic using `timestamp`, custom operators, or a stream of watermarks driving `takeUntil`.

- Resource-safe subscription in React:
  ```ts
  useEffect(() => {
    const sub = source$.pipe(
      takeUntil(destroy$), // or use fromEventPattern with abort controllers
      finalize(() => console.log('cleanup'))
    ).subscribe(...);
    return () => sub.unsubscribe();
  }, [deps]);
  ```

## Node Streams Patterns
- Async iterator pipeline with backpressure:
  ```js
  import { createReadStream, createWriteStream } from 'fs';
  import { pipeline } from 'stream/promises';
  import { Transform } from 'stream';

  const upper = new Transform({
    readableObjectMode: true,
    writableObjectMode: true,
    transform(chunk, _enc, cb) {
      try {
        cb(null, chunk.toString().toUpperCase());
      } catch (err) { cb(err); }
    }
  });

  await pipeline(
    createReadStream('in.txt'),
    upper,
    createWriteStream('out.txt')
  ); // backpressure-aware, rejects on error
  ```

- `Readable.from` for pull-friendly producers:
  ```js
  import { Readable } from 'stream';

  const source = Readable.from(async function* () {
    for await (const item of getItems()) {
      yield JSON.stringify(item) + '\n';
    }
  }(), { highWaterMark: 16 }); // bound buffer
  ```

- Transform with concurrency using `parallel-transform`-style pattern:
  ```js
  import { Transform } from 'stream';

  function parallelTransform(concurrency, fn) {
    let active = 0, queue = [];
    return new Transform({
      objectMode: true,
      transform(chunk, _enc, cb) {
        const run = () => {
          active++;
          fn(chunk).then(
            res => { this.push(res); done(); },
            err => done(err)
          );
        };
        const done = (err) => {
          active--;
          cb(err);
          if (queue.length && active < concurrency) {
            const next = queue.shift();
            next();
          }
        };
        if (active < concurrency) run(); else queue.push(run);
      }
    });
  }
  ```
  Prefer battle-tested libs (`parallel-transform`, `streamx`) in production.

- Backpressure cues:
  - Check `stream.write()` return value; pause source when false, resume on `drain`.
  - Set `highWaterMark` to a sane bound; avoid default large buffers for high-rate streams.
  - Use `pipeline` to propagate errors and close all stages.

- Avoid pitfalls:
  - Don’t mix `.on('data')` flowing mode with async iteration on same stream.
  - Always handle `error` and `close` events; with `pipeline`, attach a single rejection handler.
  - For objectMode, set `objectMode: true` explicitly; tune `highWaterMark` (number of objects).

## RxJS Event-Time Patterns and Footguns
- Event-time approximation:
  - Use `timestamp()` to attach processing-time stamps; to approximate event time, rely on upstream event ts and a separate watermark source.
  - Custom watermark stream: derive from source timestamps (e.g., `scan` min/max with max skew) and emit monotone watermarks; feed into windowing logic.
  - Prefer bounded lateness policies: partition events into on-time vs. late side channel; emit corrections/upserts for late arrivals.
- Watermark-driven window close (sketch):
  ```ts
  const WATERMARK_LAG = 2000;
  const wm$ = events$.pipe(
    map(e => e.ts),
    scan((wm, ts) => Math.max(wm, ts - WATERMARK_LAG), 0),
    distinctUntilChanged()
  );

  const windows$ = events$.pipe(
    groupBy(e => Math.floor(e.ts / WINDOW_MS)),
    mergeMap(group$ =>
      combineLatest([group$.pipe(toArray()), wm$]).pipe(
        filter(([, wm]) => wm >= (group$.key + 1) * WINDOW_MS),
        map(([events]) => ({ window: group$.key, sum: events.reduce((s, e) => s + e.value, 0) })),
        take(1)
      )
    )
  );
  ```
- Footguns:
  - `shareReplay` without bounds → leaks memory and replays unbounded history; always set `bufferSize`/`windowTime`.
  - Hot sources without `refCount` → dangling upstream work after last subscriber; use `share({ resetOnError: true, resetOnComplete: true, resetOnRefCountZero: true })`.
  - `switchMap` teardown timing: inner teardown is synchronous; if teardown must complete before next inner starts, handle it explicitly.
  - `Subject` as bus → unbounded push with no backpressure; prefer `Observable` creation with explicit buffering or sampling.
  - Synchronous heavy producers → block event loop; yield with `observeOn(asyncScheduler)` or chunk work.

## RxJS Watermark Operator Example
- Simple watermark generator and window closer (event-time, allowed lateness):
  ```ts
  import { Observable, Subject, combineLatest } from 'rxjs';
  import { map, scan, distinctUntilChanged, groupBy, mergeMap, filter, take, toArray } from 'rxjs/operators';

  type Event = { ts: number; value: number };

  const WINDOW_MS = 5_000;
  const ALLOWED_LATENESS = 2_000;

  // Derive monotone watermark from observed timestamps with skew allowance
  const watermark = (events$: Observable<Event>) =>
    events$.pipe(
      map(e => e.ts),
      scan((wm, ts) => Math.max(wm, ts - ALLOWED_LATENESS), 0),
      distinctUntilChanged()
    );

  // Tumbling window close when watermark passes window end
  const windowedSum = (events$: Observable<Event>) => {
    const wm$ = watermark(events$);
    return events$.pipe(
      groupBy(e => Math.floor(e.ts / WINDOW_MS)),
      mergeMap(group$ =>
        combineLatest([group$.pipe(toArray()), wm$]).pipe(
          filter(([, wm]) => wm >= (group$.key + 1) * WINDOW_MS),
          map(([events]) => ({
            window: group$.key,
            sum: events.reduce((s, e) => s + e.value, 0)
          })),
          take(1) // emit once per window
        )
      )
    );
  };
  ```
  Notes:
  - This is a minimal illustration; for production, avoid buffering all events with `toArray`—instead maintain incremental aggregates per window.
  - Watermark here is based on observed timestamps minus slack; if sources are out-of-order beyond `ALLOWED_LATENESS`, you’ll drop late events.
  - For sliding/session windows, the same watermark stream can gate emits, but state management becomes more complex.

## Production-Grade RxJS Windowing
- Incremental aggregation per window key:
  ```ts
  import { Observable, merge, Subject } from 'rxjs';
  import { groupBy, mergeMap, map, tap, filter, takeUntil } from 'rxjs/operators';

  type Event = { ts: number; value: number };
  type Watermark = number;

  const WINDOW_MS = 5_000;
  const ALLOWED_LATENESS = 2_000;

  function windowedSum(events$: Observable<Event>, wm$: Observable<Watermark>) {
    const done$ = new Subject<void>();
    return events$.pipe(
      groupBy(e => Math.floor(e.ts / WINDOW_MS)),
      mergeMap(group$ => {
        let acc = 0;
        const windowEnd = (group$.key + 1) * WINDOW_MS;
        return merge(
          group$.pipe(
            tap(e => { acc += e.value; })
          ),
          wm$.pipe(
            filter(wm => wm >= windowEnd + ALLOWED_LATENESS),
            map(() => ({ window: group$.key, sum: acc })),
            takeUntil(done$),
            tap(() => done$.next()) // close this window stream
          )
        );
      })
    );
  }
  ```
Notes:
- Accumulates incrementally; no unbounded buffers.
- Emits once per window after watermark surpasses `end + lateness`; late events beyond that are ignored.
- For retractions/upserts on late events (within lateness), adjust `acc` on arrival and emit corrections instead of a single finalize.

- Session windows (gap-based) sketch:
  ```ts
  import { timer, merge, of, Subject } from 'rxjs';
  import { groupBy, mergeMap, takeUntil, tap, map, filter, switchMap } from 'rxjs/operators';

  type KeyedEvent = { key: string; ts: number; value: number };
  type Watermark = number;

  function sessionWindows(events$: Observable<KeyedEvent>, gapMs: number, wm$: Observable<Watermark>) {
    return events$.pipe(
      groupBy(e => e.key),
      mergeMap(group$ => {
        let current: { start: number; end: number; sum: number } | null = null;
        const idle$ = new Subject<void>();
        const close$ = new Subject<void>();

        const maybeCloseOnWatermark$ = wm$.pipe(
          filter(wm => !!current && wm >= current!.end + gapMs),
          tap(() => close$.next())
        );

        return merge(
          group$.pipe(
            tap(e => {
              if (!current || e.ts - current.end > gapMs) {
                if (current) close$.next();
                current = { start: e.ts, end: e.ts, sum: e.value };
              } else {
                current = { ...current, end: e.ts, sum: current.sum + e.value };
              }
              idle$.next();
            })
          ),
          idle$.pipe(
            switchMap(() => timer(gapMs)),
            takeUntil(close$),
            tap(() => close$.next())
          ),
          maybeCloseOnWatermark$,
          close$.pipe(
            map(() => current),
            filter((c): c is NonNullable<typeof current> => !!c),
            tap(() => { current = null; })
          )
        ).pipe(filter(Boolean));
      })
    );
  }
  ```
  Notes:
  - Closes on inactivity gap or watermark surpassing end+gap.
  - Uses subjects for idle/reset; ensure teardown to avoid leaks.
  - For production, cap number of active sessions and clean up idle subjects; add lateness handling (drop or correction).
  - Lateness corrections: if late events arrive within allowed lateness and belong to a closed session, emit correction records (upsert or delta) keyed by session id; ignore beyond lateness.

- Testing:
  - Use `TestScheduler` with virtual time; feed synthetic events with timestamps and watermarks; assert emissions via marble diagrams.

## Node Streams Performance Tuning

## Node Streams Performance Tuning
- Batching writes:
  - Buffer small writes and flush in larger chunks to reduce syscalls; but cap batch size to avoid latency spikes.
  - Use `cork`/`uncork` on writable streams to coalesce writes.
- `highWaterMark` tuning:
  - Lower for latency-sensitive pipelines; higher for throughput-heavy but memory-tolerant workloads.
  - Separate `highWaterMark` for objectMode vs. binary; default 16KB (binary) / 16 objects (objectMode).
- Object mode costs:
  - Prefer binary/Buffer streams for raw throughput; serialize/deserialize at boundaries sparingly.
  - If using objectMode, keep payloads small and avoid deep cloning per chunk.
- Avoid extra copies:
  - Reuse Buffer slices; avoid `Buffer.concat` in hot paths—preallocate or use ring buffers.
  - Use `Readable.from` with async generators to pull as needed rather than push storms.
- Concurrency:
  - In transform streams that do async work, bound concurrency (e.g., with a queue) to prevent unbounded in-flight promises.
  - For CPU-bound work, offload to worker threads; don’t block the event loop inside transforms.
- GC pressure:
  - Minimize per-chunk allocations; reuse objects where safe; avoid large closures in transform hot paths.
- Metrics:
  - Track throughput, backpressure (`write()` false counts), drain wait time, transform service time; surface p95/p99.

## Node Streams Benchmarking Tips
- Isolate components:
  - Benchmark transforms in-memory with `Readable.from` and `Writable` sink counting bytes/objects.
  - Avoid disk/network unless specifically measuring I/O.
- Measure:
  - Throughput (MB/s or objs/s), GC time, CPU usage, latency per chunk, backpressure events (drain waits).
  - Count `write()` false returns and average/percentile drain wait.
- Tools:
  - `perf_hooks.performance` for timing; `process.resourceUsage()` for CPU; `clinic flame/doctor` for profiling; `0x` or `node --prof` for detailed CPU.
  - `--trace_gc` or `--trace_gc_verbose` to inspect GC pauses; keep payloads representative.
- Warm-up:
  - Run a warm-up iteration to allow JIT optimization before measuring.
- Payload realism:
  - Use realistic chunk sizes and distributions; micro-benchmarks with tiny buffers can mislead.
- Avoid shared process noise:
  - Pin CPU (taskset on Linux), run with minimal background load; repeat runs and take medians/p95.
- Profiling examples:
  - CPU: `node --prof app.js` then `node --prof-process isolate-*-v8.log`; or `npx 0x app.js` for flamegraphs.
  - Async bottlenecks: `clinic bubbleprof -- node app.js`.
  - GC: run with `--trace_gc`/`--trace_gc_verbose`; parse pause times; correlate with allocation sites via sampling profiler.
  - Metrics in code: wrap transforms to record per-chunk service time and drain wait with `performance.now()`; emit percentiles.
  - GC profiling workflow: run representative load with `--trace_gc` and capture logs; run CPU profiler concurrently; identify allocations in hot transforms; reduce object churn or switch to Buffer pipelines; retest and compare pause percentiles.

## Benchmark Harness Templates (Node, Rust)
- Node stream microbench harness:
  ```js
  // bench.js
  import { Readable, Transform, Writable } from 'stream';
  import { performance } from 'perf_hooks';

  const N = 1e6;
  const payload = Buffer.from('x'.repeat(128));

  const source = Readable.from((function* () {
    for (let i = 0; i < N; i++) yield payload;
  })(), { highWaterMark: 64 });

  const tx = new Transform({
    transform(chunk, _enc, cb) {
      // no-op transform; replace with real work
      cb(null, chunk);
    }
  });

  let bytes = 0;
  const sink = new Writable({
    write(chunk, _enc, cb) { bytes += chunk.length; cb(); }
  });

  (async () => {
    const t0 = performance.now();
    await import('stream/promises').then(({ pipeline }) => pipeline(source, tx, sink));
    const t1 = performance.now();
    const mb = bytes / (1024 * 1024);
    const sec = (t1 - t0) / 1000;
    console.log(`Processed ${mb.toFixed(2)} MB in ${sec.toFixed(3)}s (${(mb/sec).toFixed(2)} MB/s)`);
  })();
  ```
  Run with profiling:
  - CPU: `node --prof bench.js` → `node --prof-process isolate-*-v8.log`
  - Flame: `npx 0x bench.js`
  - GC: `node --trace_gc bench.js` (or `--trace_gc_verbose`)

- Rust async stream microbench (tokio):
  ```rust
  // benches/stream_bench.rs
  use tokio::io::{AsyncReadExt, AsyncWriteExt};
  use tokio::time::Instant;

  const N: usize = 1_000_000;
  const PAYLOAD: &[u8] = &[b'x'; 128];

  #[tokio::main(flavor = "current_thread")]
  async fn main() {
    let mut bytes = 0usize;
    let t0 = Instant::now();

    let mut reader = tokio_stream::iter(0..N).map(|_| PAYLOAD);
    while let Some(chunk) = reader.next().await {
      // replace with real transform; here just count
      bytes += chunk.len();
    }

    let dur = t0.elapsed();
    let mb = bytes as f64 / (1024.0 * 1024.0);
    let sec = dur.as_secs_f64();
    println!("Processed {:.2} MB in {:.3}s ({:.2} MB/s)", mb, sec, mb/sec);
  }
  ```
  Run with profiling:
  - Perf (Linux): `perf record -g --call-graph=dwarf target/release/bench && perf report`
  - `cargo flamegraph -- target/release/bench` (with `inferno` installed)
  - Heap/alloc: `MALLOC_CONF=prof:true` with `jeprof` (jemalloc), or `DHAT`/`heaptrack` for deeper memory analysis.
  Notes:
  - Use `cargo bench`/`criterion` for more robust stats; include warm-up and multiple samples.
  - For realistic workloads, replace no-op with actual parsing/serialization and keep payload sizes representative.

## Cross-Language Comparison Tips
- Normalize workload:
  - Same payload sizes and distributions; same transform logic (e.g., checksum, parse JSON, compress).
  - Match concurrency model: async stream vs. blocking IO vs. multi-thread—state assumptions explicitly.
  - Warm-up JIT (Node/JS) vs. no JIT (Rust); discard first runs.
- Metrics to capture:
  - Throughput (MB/s, events/s), p95/p99 latency per chunk, CPU utilization, GC time (managed runtimes), RSS/heap usage.
  - Backpressure signals: queue depth, write stall counts, drain wait time.
- Env parity:
  - Pin CPU cores; disable turbo for stability; use same kernel settings (NAPI, TCP params) if networking involved.
  - Use release builds (`NODE_OPTIONS=--max-old-space-size=...` as needed, `cargo build --release`).
  - Ensure comparable buffering (`highWaterMark` vs. channel bounds).
- Profiling:
  - JS/Node: `0x`, `--prof`, `clinic` for async; GC with `--trace_gc`.
  - Rust: `perf`, `cargo flamegraph`, `heaptrack`/`DHAT` for allocs; `tokio-console` for async scheduling.
- Avoid micro-benchmark traps:
  - Tiny payloads favor interpreter overhead; large payloads may be bandwidth-bound—test multiple sizes.
  - Measure both CPU-bound and IO-bound scenarios; report both.
  - Report configuration clearly (hardware, OS, versions, flags).

## Node Streams Error Handling Patterns
- Default to `pipeline`/`finished`:
  ```js
  import { pipeline, finished } from 'stream/promises';
  await pipeline(src, tx, dst); // closes all on error
  await finished(dst); // waits for completion/error
  ```
- Centralized error propagation when manual piping:
  ```js
  src.pipe(tx).pipe(dst);
  const bail = (err) => {
    src.destroy(err);
    tx.destroy(err);
    dst.destroy(err);
  };
  for (const s of [src, tx, dst]) s.on('error', bail);
  ```
- Abortable pipelines:
  ```js
  const ac = new AbortController();
  pipeline(src, tx, dst, { signal: ac.signal }).catch(err => {
    if (err.code !== 'ABORT_ERR') throw err;
  });
  // ac.abort() to cancel
  ```
- Validation/parsing transforms:
  ```js
  const parser = new Transform({
    readableObjectMode: true,
    transform(chunk, _enc, cb) {
      try { cb(null, JSON.parse(chunk)); }
      catch (err) { cb(err); }
    }
  });
  ```
- Footguns:
  - Throwing inside transform without catching can terminate process; always `cb(err)` or emit `error`.
  - Not destroying upstream on downstream failure leaves hung producers; `pipeline` handles this.
  - Listening to `error` without removing listeners on reuse can leak; prefer one-shot pipelines or manage listeners lifecycle.
  - Mixing flowing mode (`data` events) with paused/pipe leads to dropped data; pick one mode.
## Lean/Coq Operator Proofs
- Associative/commutative reducer correctness under partitioned parallelism:
  ```lean
  variables {α : Type} [comm_monoid α]

  def fold_chunk (xs : list α) : α := xs.foldl (*) 1

  lemma fold_chunked_equiv :
    ∀ (chunks : list (list α)) (xs : list α),
      chunks.join = xs →
      (chunks.map fold_chunk).foldl (*) 1 = xs.foldl (*) 1
  | [], xs, h => by
      -- join [] = [] so xs = [], folds are identity
      subst h; simp [fold_chunk]
  | chunk :: rest, xs, h => by
      -- join (chunk :: rest) = chunk ++ rest.join
      have hx : chunk ++ rest.join = xs := h
      subst hx
      -- fold over map head + tail equals fold over concatenated chunk ++ rest.join
      simp [fold_chunk, list.map_cons, list.foldl_append, fold_chunked_equiv rest (rest.join) rfl, mul_comm, mul_left_comm, mul_assoc]
  ```
  Shows map-reduce with associative/commutative op is equivalent to single-fold—basis for parallel aggregation correctness.

- Window aggregate with retractions convergence:
  ```lean
  variables {β : Type} [add_comm_group β]

  structure Ev := (t : nat) (v : β) (kind : bool) -- kind=false add, true retract

  def apply_ev (acc : β) (e : Ev) : β :=
    if e.kind then acc - e.v else acc + e.v

  lemma retract_converges :
    ∀ (es : list Ev) (acc : β),
      es.foldl apply_ev acc =
        acc
        + (es.filter (λ e, ¬ e.kind)).foldl (λ a e, a + e.v) 0
        - (es.filter (λ e, e.kind)).foldl (λ a e, a + e.v) 0
  | [], acc => by simp [apply_ev]
  | e :: es, acc =>
      by
        -- inductive step: unfold one event, rewrite filters
        cases hkind : e.kind
        · -- add case
          simp [apply_ev, hkind, retract_converges es, list.filter_cons, hkind, add_assoc, add_comm, add_left_comm]
        · -- retract case
          simp [apply_ev, hkind, retract_converges es, list.filter_cons, hkind, sub_eq_add_neg, add_assoc, add_comm, add_left_comm, add_right_comm]
  ```
  Basis for proving correctness of upsert/retraction-based window outputs under reordering.

- Deterministic state machine with inputs as list (replay safety):
  ```lean
  variables {σ α : Type} (step : σ → α → σ)

  lemma replay_deterministic (s0 : σ) (xs ys : list α)
    (hperm : xs = ys) :
    xs.foldl step s0 = ys.foldl step s0 :=
  by simpa [hperm]
  ```
  Trivial but formal: replaying same ordered inputs yields same state; extensions add stutter-insensitivity for duplicated inputs if `step` idempotent.

- Coq counterparts (Coq stdlib):
  ```coq
  Require Import List.
  Require Import Coq.micromega.Lia.
  Require Import Coq.Classes.Morphisms.
  Require Import Coq.Classes.RelationClasses.
  Require Import Coq.Setoids.Setoid.
  Import ListNotations.

  Section FoldChunked.
    Context {A : Type} `{Monoid : Monoid A}. (* assume a monoid with op dot and e *)

    Definition fold_chunk (xs : list A) : A := fold_left dot xs e.

    Lemma fold_chunked_equiv :
      forall (chunks : list (list A)) (xs : list A),
        concat chunks = xs ->
        fold_left dot (map fold_chunk chunks) e = fold_left dot xs e.
    Proof.
      induction chunks as [|c rest IH]; intros xs Hc.
      - simpl in Hc; subst; simpl; reflexivity.
      - simpl in Hc. apply app_inv_head in Hc.
        subst xs. simpl.
        rewrite IH by reflexivity.
        rewrite fold_left_app.
        (* associativity/commutativity needed; assume dot is commutative monoid *)
        admit.
    Admitted.
  End FoldChunked.

  Section Retract.
    Context {B : Type} `{AbGroup : AbelianGroup B}. (* additive commutative group *)

    Inductive kind := Add | Retract.
    Record Ev := { t : nat; v : B; k : kind }.

    Definition apply_ev (acc : B) (e : Ev) : B :=
      match k e with
      | Add => acc + v e
      | Retract => acc - v e
      end.

    Lemma retract_converges :
      forall (es : list Ev) (acc : B),
        fold_left apply_ev es acc =
          acc
          + fold_left (fun a e => a + v e) (filter (fun e => match k e with Add => true | _ => false end) es) 0
          - fold_left (fun a e => a + v e) (filter (fun e => match k e with Retract => true | _ => false end) es) 0.
    Proof.
      induction es as [|e es IH]; intro acc; simpl; auto.
      destruct (k e); simpl;
        try rewrite IH; ring.
    Qed.
  End Retract.
  ```
  Notes:
  - `Monoid`/`AbelianGroup` need explicit instances; `admit` placeholder would be discharged with associativity/commutativity lemmas or a commutative monoid typeclass.
  - Coq’s `ring` tactic solves the additive group algebra in `retract_converges`.

## Future Directions and Frontier Ideas
- Unified time semantics: tighter integration of event-time, processing-time, and logical clocks, with automatic skew estimation and dynamic lateness adjustment driven by observed distributions.
- Adaptive flow control: controllers that adjust credits, batch sizes, and concurrency using online control theory (PID with anti-windup, model-predictive control) to hit latency/throughput SLOs automatically.
- Semantic compression: structure-aware compression of streams (schematized, columnar, or learned codecs) to reduce bandwidth while preserving low-latency decode and selective access.
- Deterministic distributed execution: pervasive virtual time + deterministic schedulers across clusters to make replay/debug bit-for-bit reproducible, even with parallelism and failure.
- Verified operators: use lightweight formal methods (TLA+, Alloy, Coq/Lean snippets) to prove properties of custom window/joins, backpressure controllers, and idempotent sinks before deployment.
- CRDT-first streaming: default to lattice-backed state so that reordering and partition tolerance are safe; combine with compaction and bounded tombstones for long-lived deployments.
- Edge/near-data fusion: push compute to data sources (edge, storage nodes) with end-to-end lineage and replayability; use code mobility with sandboxing and determinism guarantees.
- Adaptive windowing: operators that shift window types/lengths based on observed skew and workload phase (e.g., switch from sliding to session when bursty).
- Streaming + ML copilot: online drift detectors and policy learners that tune operator parameters (batching, concurrency, thresholds) under guardrails; require safety proofs to avoid instability.

## Testing and Verification
- Virtual time and deterministic schedulers to test timing-dependent logic.
- Property-based tests for ordering, idempotency, associativity of reducers, and replay safety.
- Fault injection: dropped/duplicated/reordered events; slow consumers; partial failures.
- Model-based tests for stateful operators (windows, joins) against reference implementations.
- Workload simulation using recorded traces to validate latency/backpressure policies.

## Design Checklist
- Time: Which clock do operators use? Are watermarks defined? How is lateness handled?
- Ordering: Do you need total, per-key, or causal order? What happens on reordering?
- Delivery: Which mode (at-most/at-least/effectively-once)? How are side effects made idempotent?
- Backpressure: How is demand signaled? What are the bounds? What happens on overflow (drop, shed, spill)?
- State: How is it partitioned, bounded, evicted, checkpointed, and recovered?
- Topology: Where are restart boundaries and partition boundaries? How do merges handle duplicates?
- Observability: Which metrics/traces/logs prove the system is healthy? What are alert thresholds?
- Performance: What are throughput/latency SLOs? How are buffers, batches, and concurrency tuned?

## Applied Pattern Example: Credit-Based Flow Control, Watermarking with Lateness, Replay-Safe Sink
- Context: stream of events with event-time timestamps; need bounded latency, no unbounded buffering, tolerates duplicates, requires correctness under replay.
- Flow control:
  - Consumer maintains a credit counter (N max in-flight). Producer sends up to available credits; credits replenished on acknowledgment.
  - If consumer slows, credits deplete → producer naturally slows without unbounded buffer growth.
  - Queue is bounded; overflow policy = drop-head to preserve freshness, or spill to disk if loss is unacceptable.
- Watermarking with lateness:
  - Watermark emitted as `min(event_time_seen_so_far) - allowed_skew`; defines completeness frontier.
  - Windows close when watermark passes window end; allow lateness `Δ` to accept stragglers.
  - Late arrivals within `Δ` trigger upsert/retraction to downstream; beyond `Δ` are dropped or routed to a late lane for auditing.
- Replay-safe sink:
  - At-least-once delivery tolerated; sink is idempotent via primary key + version (seqno or event-time) per record.
  - Writes are `upsert(key, version, value)`; discard if version ≤ stored version to neutralize duplicates/reordered arrivals.
  - Side effects (e.g., notifications) gated by outbox table or idempotent token to avoid duplication on replay.
- End-to-end behavior:
  - Backpressure is enforced by credits; buffering is bounded and visible.
  - Correctness holds under reordering: watermark + lateness window handle stragglers; sink upsert neutralizes duplicates.
  - Recovery: on restart, replay from last checkpoint/WAL; deterministic partitioning and idempotent sink ensure convergence.
