# Appendix: Emergence as a Design Practice

## Purpose

Ground emergent design in practical steps for building systems that behave like living organisms: simple rules, strong constraints, rich feedback, and the ability to adapt without losing identity.

## What “emergence” means in software

- Complex, coherent behavior arising from **simple, local rules** (no central conductor).
- Robustness from **redundancy and feedback**, not from perfect prediction.
- Structure that **adapts to environment signals** (traffic, errors, cost, user intent).
- Evolution through **variation and selection** (experiments, flags, migration ladders).

## Living-organism mental model

- **Genome = contracts and types.** Schema, reducers, invariants encode what cannot break.
- **Cells = modules/services.** Small, specialized units with clear membranes (interfaces) and message passing.
- **Metabolism = dataflow.** Streams/events/queues move energy; backpressure keeps metabolism healthy.
- **Nervous system = observability.** Telemetry, traces, and health signals trigger reflexes.
- **Immune system = policy + kill switches.** Rate limits, circuit breakers, safemode modes to contain damage.
- **Growth = safe replication.** Scaling via idempotent handlers and stateless workers.

## Non-negotiables for emergent systems

1) **Simple rules, explicit invariants.** Make illegal states unrepresentable; keep transitions pure and replayable.  
2) **Local autonomy, clear membranes.** Services/components decide locally with well-defined ports/adapters.  
3) **Feedback first.** Every loop emits signals; controllers act on them (autoscale, degrade, reroute).  
4) **Controlled variation.** Flags, canaries, and shadow traffic let you explore without risking the organism.  
5) **Repair over perfection.** Prefer fast detection, isolation, and rollback paths to upfront exhaustiveness.

## Designing with emergence (practices)

- **Start from invariants, not features.** Define what must always hold; let behavior emerge from transitions respecting those rules.
- **Favor message passing over shared state.** Local decisions accumulate into global behavior without tight coupling.
- **Use small deterministic cores.** Reducers/FSMs as the “DNA”; IO shells handle sensors/effectors.
- **Build feedback loops explicitly.** Control loops per subsystem: observe → compare to target → act (scale, shed load, slow producers).
- **Instrument first.** Ship observability with the feature so the organism can “feel” new limbs immediately.
- **Experiment continuously.** Treat every change as a hypothesis; deploy behind flags; measure survival fitness (latency, error budgets, conversion).
- **Let structure follow flow.** Event logs and projections let new behaviors emerge without rewiring writers.

## Architecture patterns that encourage emergence

- **Evented core + projections.** Facts are immutable; new read models emerge without disturbing writers.
- **Actor/queue topology.** Per-entity mailboxes prevent interference; topology changes shape behavior safely.
- **Plugins via stable contracts.** Capability injection (policies, calculators, adapters) allows new traits without rewriting hosts.
- **Policy + control planes.** Central intent, distributed enforcement (rate limits, budgets, routing).
- **Simulation & replay.** Run time-travel and randomized schedules to see global patterns before shipping.

## Canonical CS examples to steal from

- **Cellular automata (Game of Life).** Tiny local rules → gliders/spaceships; lesson: choose rules that are composable and conservative (mass/energy preserved).
- **Boids / flocking.** Alignment, separation, cohesion produce schooling; lesson: encode “steer towards average, avoid collision, respect distance” for distributed load balancers and caches.
- **Gossip/epidemic protocols.** Random peer sampling leads to fast, resilient spread; lesson: probabilistic fan-out beats centralized coordination for cache invalidation and health.
- **CRDTs.** Commutative/associative/ idempotent ops converge without coordination; lesson: pick merge laws that make conflicts impossible.
- **PID/control loops.** Continuous correction stabilizes noisy systems; lesson: tune controllers (P/I/D or discrete equivalents) for autoscaling and backpressure.

## Subtypes of emergence (useful distinctions)

- **Weak emergence:** Macro behavior is novel but reducible to micro rules via simulation (Game of Life patterns, flocking). Most software falls here.  
- **Strong emergence:** Macro properties with “downward causation” influencing micro behavior (rare in software; sometimes invoked in socio-technical dynamics).  
- **Statistical vs structural:** Statistical emergence is aggregate patterns (traffic distributions); structural emergence is new stable structures/topologies (clusters, gliders, shards).  
- **Benign vs malignant:** Benign patterns improve fitness (autoscaling stability); malignant patterns harm it (retry storms, thundering herd).

## CS paradigms and models for emergent behavior

- **Complex adaptive systems (CAS):** Agents + adaptation + feedback; emphasizes fitness landscapes and phase changes.  
- **Self-organizing systems:** No central controller; order arises via local interactions and feedback (stigmergy, pheromone trails, gossip).  
- **Swarm intelligence:** Distributed search/optimization (ant colony, particle swarm) mapped to routing, caching, scheduling.  
- **Agent-based modeling (ABM):** Simulate micro rules to study macro outcomes; useful for load/topology experiments.  
- **Artificial life (ALife):** Evolving programs/organisms; informs mutation/selection loops in infrastructure (parameter sweeps, auto-tuning).  
- **Epidemic/gossip protocols:** Probabilistic fan-out for dissemination, membership, failure detection.  
- **CRDT/convergent replicas:** Emergence of consistency from algebraic merge laws instead of coordination.  
- **Self-stabilization:** Systems that converge to legality from any starting state; design reducers/policies to be convergent.  
- **Stigmergy:** Agents coordinate indirectly via shared environment (queues, logs) rather than direct messaging.  
- **Morphogenetic computing:** Shape/topology formation from local gradients; informs sharding/rebalancing strategies.

## Cyclomatic complexity and emergence

- Emergent behavior benefits from **simple, inspectable local rules**. High cyclomatic complexity in reducers/policies is a smell: it hides the local rule set and makes macro outcomes harder to reason about.  
- Keep complexity per “agent” low (reducers, handlers, policies); move variation into configuration/flags and composition of small functions.  
- Use property tests and replay to cover the remaining branches; if a policy/reducer’s complexity grows, split into composable rules or multiple agents to preserve clarity of the membranes.

## Designing local rules (recipe)

1) **Define the agent + environment.** What is the “cell” (worker, shard, reducer) and what signals can it sense (queue depth, error rate, deadline budget)?  
2) **Declare hard constraints.** Invariants and quotas: max in-flight per key, max latency budget, max outstanding writes.  
3) **Choose simple actions.** Slow down, shed work, reroute, buffer, spawn, merge; keep actions few and reversible.  
4) **Encode priorities.** Tie-break by urgency (deadlines), fairness (per-tenant caps), and health (error rate).  
5) **Add conservation laws.** Track credits/permits/tokens so production cannot exceed consumption capacity.  
6) **Expose state + intent.** Each agent publishes its local view (depth, health) to make coordination emergent instead of hard-wired.  
7) **Make rules cheap to simulate.** Keep them deterministic and small so they can run in property tests and offline replay.

## Layered control loops (time scales)

- **Reflex (ms–s):** Hot-path guards: per-aggregate serialization, deadlines, rate limits, circuit breakers, immediate degrade modes.  
- **Homeostasis (s–m):** Autoscaling, dynamic backpressure targets, adaptive timeouts based on observed latency distributions.  
- **Evolution (m–days):** Feature flags, parameter sweeps, canary policies; select winners by fitness metrics (SLO error, cost per unit, user conversion).

## Testing for emergence (before prod)

- **Property/simulation tests:** Randomized schedulers, varying message interleavings, and injected clock skew to ensure invariants survive.  
- **Phase-change hunts:** Sweep load until a bottleneck flips state (queue explosion, thrash); add detectors and guardrails at those thresholds.  
- **Replay with perturbation:** Take prod traces, shuffle order, and replay through reducers/FSMs to see if behavior holds.  
- **Differential runs:** Compare old vs new rulesets under identical event logs to ensure changes only affect intended metrics.  
- **Chaos in staging:** Kill nodes, drop messages, add latency; observe whether control loops restore targets.

## Example blueprint: adaptive ingestion pipeline

- **Agent:** Worker shard keyed by tenant.  
- **Signals:** Queue depth per tenant, end-to-end latency percentile, error rate, cost per batch.  
- **Rules:** One writer per tenant; if depth grows, slow producers for that tenant; if latency budget tightens, shrink batch size; if error spikes, trip breaker to degrade to cached responses.  
- **Feedback:** Periodic gossip of per-tenant health so neighboring shards can rebalance; autoscaler listens to aggregate depth variance, not just mean.  
- **Selection:** Run two batching policies behind flags; select the one that lowers cost while keeping SLO error under target; codify as new default and retire the loser.

## Stack-specific emergent patterns

**Rust (tokio/ecosystem)**  
- Per-aggregate actors/mailboxes (mpsc keyed by ID) to isolate decisions; bounded channels enforce backpressure.  
- Supervisors around tasks; restart with jitter; map panics to kill switches.  
- `tokio::time::timeout` around IO; propagate deadline budgets through context structs.  
- Observability as a first-class trait: emit structured events (JSON/otlp) per transition; expose gauges for mailbox depth.  
- Property + simulation tests: deterministic clocks; injected scheduler to scramble interleavings.  
- Feature flags via static config + dynamic fetch; wrap reducers in “policy plugins” that can be swapped without recompiling core contracts.

**Rust snippet: keyed mailbox + deadline propagation**

```rust
use std::{collections::HashMap, time::Duration};
use tokio::{sync::mpsc, task, time};

type AggregateId = String;
type Command = String;

struct Envelope {
    cmd: Command,
    deadline: time::Instant,
}

async fn actor_loop(mut rx: mpsc::Receiver<Envelope>) {
    while let Some(env) = rx.recv().await {
        let start = time::Instant::now();
        // Reflex: enforce deadline; drop or emit a timeout event
        if time::Instant::now() >= env.deadline {
            tracing::warn!(cmd = ?env.cmd, "deadline-exceeded");
            metrics::counter!("cmd.dropped.deadline", 1, "cmd" => env.cmd.as_str());
            continue;
        }
        // Pure reducer lives here; IO happens around it
        // reducer.apply(event);
        metrics::histogram!("cmd.duration", start.elapsed().as_secs_f64(), "cmd" => env.cmd.as_str());
    }
}

pub async fn dispatch(
    routers: &mut HashMap<AggregateId, mpsc::Sender<Envelope>>,
    id: AggregateId,
    cmd: Command,
    budget: Duration,
) {
    let aggregate_label = id.as_str();
    let tx = routers.entry(id.clone()).or_insert_with(|| {
        let (tx, rx) = mpsc::channel::<Envelope>(128); // bounded = backpressure
        task::spawn(actor_loop(rx));
        tx
    });
    let _ = tx
        .try_send(Envelope { cmd, deadline: time::Instant::now() + budget })
        .map_err(|e| {
            tracing::warn!(aggregate = %id, err = ?e, "mailbox-backpressure");
            metrics::counter!("mailbox.backpressure", 1, "aggregate" => aggregate_label);
        });
}
```
Uses `tracing` + `metrics` macros; swap for your telemetry stack (OpenTelemetry, metrics-exporter-prometheus, etc.).

**Rust snippet: policy plugin around a reducer**

```rust
pub trait Policy {
    fn allow(&self, cmd: &CommandContext) -> bool;
}

pub struct PolicyWrapped<R, P> {
    reducer: R,
    policy: P,
}

impl<R, P> PolicyWrapped<R, P>
where
    R: Reducer,
    P: Policy,
{
    pub fn apply(&self, ctx: &CommandContext, cmd: Command) -> Result<State, Error> {
        if !self.policy.allow(ctx) {
            return Err(Error::Denied);
        }
        self.reducer.apply(cmd)
    }
}
```

**Laravel**  
- Emergence via queues/Horizon: one job = one aggregate; use tags to route to per-tenant queues when noisy neighbors appear.  
- Policies/middleware as membranes: rate limits, tenancy checks, feature flags (e.g., Laravel Pennant) gate behavior per request.  
- Observability: domain events logged to structured channels; Horizon metrics as signals for autoscale rules.  
- Rollbacks and healing: outbox pattern + idempotent jobs; replay dead-lettered jobs after patching; toggles to degrade controllers to cached reads.  
- Experiments: percentage flags + canaries on specific queues or routes; compare conversion/error/latency before promoting.

**Nuxt / Node edges**  
- Edge middleware as reflex layer: fast reject/shape traffic (geo, device, auth), attach deadlines/budgets.  
- Server routes as deterministic cores; adapters for IO; use message buses for heavier flows instead of coupling to requests.  
- Client-state emergence: use composables/stores as reducers; hydrate from events; keep optimistic updates reversible.  
- Progressive delivery: split traffic by segment; shadow requests to new handlers; measure error/latency before shifting weight.  
- Observability: edge logs + browser perf beacons feed backpressure signals (throttle features for slow clients); feature flags cached at edge with fast TTL.  
- Healing: global kill switches in middleware; degrade assets/features based on error rate or CLS/LCP thresholds.

## Homeostasis playbook (holding the setpoint)

- **Set the target:** Latency p95, error budget, freshness, cost per request; publish as SLO + budget.  
- **Sense:** Metrics + traces + domain events; per-tenant/segment where possible.  
- **Compare:** Error vs target; use small hysteresis bands to avoid flapping.  
- **Act:** Rate limit, shed (serve cached/partial), scale (tasks/pods), resize batches, reroute to cheaper paths.  
- **Control template (discrete PID-ish):**  
  - `error = target - observed`  
  - `integral = clamp(integral + error, floor, ceil)` to avoid windup  
  - `derivative = error - last_error`  
  - `output = kp*error + ki*integral + kd*derivative` → drives knob (concurrency, batch size, token bucket rate)  
- **Stack hooks:**  
  - Rust: adjust channel bounds, semaphore permits; expose metrics for depth and success latency.  
  - Laravel: Horizon queue concurrency per tag; middleware rate limits per tenant; cached fallbacks in controllers.  
  - Nuxt/edge: throttle in middleware; downgrade assets/features for slow clients; cache at edge with short TTLs.

## Metabolism & energy budget

- Treat CPU/IO/$ as calories; assign per-tenant/feature budgets and enforce with tokens/credits.  
- Cost-aware routing: prefer cheaper regions/paths when latency budget allows; fall back to local for hot paths.  
- Throttle “hungry” features (expensive joins, heavy embeds) when budget breached; swap to approximate/cached results.  
- Emit cost-to-serve metrics; feed product with “expensive tenant/feature” reports.  
- Run “energy audits”: identify work that can be deferred, batched, or dropped with minimal value loss.

## Memory, forgetting, and aging

- **Short-term:** Caches with explicit TTL and jitter; expose hit/miss and eviction reasons.  
- **Long-term:** Snapshots + logs → replay to regenerate; periodic compaction and anti-entropy to keep projections aligned.  
- **Forgetting:** Decay counters and stale flags; expire permissions/locks; clean abandoned feature flags/config.  
- **Aging detectors:** Drift checks between source-of-truth and projections; alarms on divergence growth.  
- **Rust hook:** schedule replay/regeneration jobs with deterministic reducers; guard with versioned schemas.  
- **Laravel hook:** nightly Horizon jobs for compaction/rebuild; migrations carry backfill + rollback.  
- **Nuxt hook:** client cache bust via versioned manifests; stale-while-revalidate for data.

## Morphogenesis (shaping topology)

- Split when: per-aggregate contention, conflicting latency/cost goals, or slow deploy cadence coupling.  
- Merge when: chatty boundaries, duplicated invariants, or coordination overhead dominates.  
- Fitness tests for new topology: soak with prod-like load, replay logs, measure SLO, cost, error budget burn.  
- Use sharding rules that preserve one-writer-per-key; avoid cross-shard transactions.  
- Keep adapters thin so moving boundaries changes wiring, not core contracts.

## Immune system deep dive

- **Passive armor:** Types, invariants, per-aggregate serialization, schema/version gates.  
- **Reflexes:** Deadlines, timeouts, rate limits, circuit breakers, backpressure, retries with jitter + caps.  
- **Detection:** Anomaly detectors on error mixes, retry storms, queue depth variance, SLO burn rates.  
- **Containment:** Kill switches, safemode (read-only/limited features), traffic shifting away from unhealthy cells.  
- **Drills:** Chaos (kill tasks/nodes, inject latency), failover tests, replay with perturbation, dead-letter rehearse.  
- **Runbooks:** Single-page triggers/actions/owners; automate if possible.

## Coevolution and niches

- Treat services/clients as species sharing resources (bandwidth, write capacity).  
- Contract tests = mutualism; backpressure + budgets manage competition; avoid parasitic consumers by auth + quotas.  
- Observe coevolution metrics: cross-service latency correlations, cache hit erosion from aggressive consumers.  
- Provide “refuges”: low-priority queues or delayed pathways for best-effort traffic.

## Behavioral plasticity (policy without redeploy)

- Policy tables/config drive decisions; rules evaluated at runtime with cache + TTL.  
- Guarded scripting/DSL for reflex updates; sandboxed and audited; can be rolled back via flags.  
- Safe defaults: deny-unknown, bounded loops, timeouts on policy eval; emit metrics per decision path.  
- Example knobs: per-tenant concurrency caps, burst multipliers, cache bypass lists, experiment weights.

## Alignment and ethics (guardrails)

- “Do-not-cross” invariants: never exceed cost caps, never retry after idempotency breaks, never drop compliance events.  
- Shape incentives: penalize retries after certain depth; prioritize fairness across tenants.  
- Observe for misalignment: runaway cost, spam amplification, starvation of low-volume tenants.  
- Human-in-the-loop for irreversible actions; require dual control for policy that widens blast radius.

## Lifecycle & regeneration

- **Birth:** Bootstrap with seeds/pins; ship observability first.  
- **Growth:** Scale by replication; verify one-writer-per-key holds; monitor fitness.  
- **Mutation:** Experiments/flags; differential tests against recorded logs; promote winners.  
- **Aging:** Config drift detection; retire stale flags/config; compaction of data.  
- **Death:** Sunset plan with dual writes, read switch, then write-off; archive or export.  
- **Regeneration:** Rebuild projections from logs; disaster drill to prove RPO/RTO; verify invariants post-rebuild.

## Examples (short case studies)

- **Cache invalidation outbreak:** New feature doubled cache churn, tanking hit rate. Signals: rising origin QPS, hit rate collapse, p95 up. Actions: edge TTL jitter + request coalescing; rate limit stampedes per key; replay showed a small fraction of keys were “hotspots” → added per-key breaker. Outcome: restored hit rate, reduced origin load 40%.  
- **Adaptive batching win:** Ingestion pipeline auto-shrank batches when p95 approached 2× target, then regrew under calm load. Signals: p95, queue depth variance. Actions: PID-ish controller adjusted batch size and concurrency permits. Outcome: 25% lower error budget burn, 15% cost drop via calmer scaling.  
- **Projection divergence:** Drift detected between source events and read model. Signals: periodic parity checks failed. Actions: halted writes to projection, replayed from log with new schema guard, added invariant check in reducer. Outcome: consistency restored; new drift alarm stays green.

## Metrics map (signals → detectors → actions)

- **Latency p95/p99:** Detector: SLO burn-rate; Action: tighten rate limit, shrink batch size, autoscale workers.  
- **Error mix (5xx/timeouts):** Detector: spike over baseline + derivative; Action: trip circuit breaker, degrade to cache, page on-call.  
- **Queue depth/variance:** Detector: depth > N or variance > threshold; Action: slow producers per key/tenant, add workers, rebalance shard.  
- **Retry volume:** Detector: retries > X% of traffic; Action: cap retries, increase backoff, surface to caller to stop floods.  
- **Cost per request/job:** Detector: cost drift > budget; Action: reroute to cheaper path, disable expensive features, batch more aggressively.  
- **Drift/invariant breaches:** Detector: parity checks, property test alarms; Action: pause derived writes, replay from log, hotfix reducer.

## Runbooks (recipes)

- **Backpressure storm:**  
  - Detect: queue depth + latency rising, success steady.  
  - Act: reduce producer rate per key, raise consumer count to ceiling, enable request coalescing, drop optional work.  
  - Verify: depth falling, p95 normalizing, no retry spike.  
- **Retry flood:**  
  - Detect: retry ratio climbing, circuit trips.  
  - Act: cap retries, add jitter, mark failing downstream as unhealthy, return fast errors with Retry-After.  
  - Verify: retry ratio falls, downstream recovers, error mix stabilizes.  
- **Projection divergence:**  
  - Detect: parity check alarms.  
  - Act: freeze projection writes, snapshot log offset, replay into fresh projection, swap reads after consistency check.  
- **Flag rollback:**  
  - Detect: fitness metric down or error up after flag.  
  - Act: flip flag off globally, clear caches, rollback config; if data touched, run compensating reducer on affected keys.  
- **Shard hotspot:**  
  - Detect: outlier shard depth/latency.  
  - Act: split shard or rebalance keys; temporarily prioritize cold shards; add per-key breakers.

## Verification kit (CI/staging)

- **Replay tests:** Use recorded logs to assert invariants and deterministic outputs across reducer versions.  
- **Chaos drills:** Inject latency/drop/kill tasks in staging; assert control loops restore setpoints.  
- **Differential tests:** Run old vs new rule sets on same event stream; compare state/metrics deltas.  
- **Simulation/property tests:** Randomized schedules, clock skew, adversarial inputs for reducers/FSMs.  
- **Load sweeps:** Phase-change hunts to identify thresholds and tune detectors/guards.

## Visual sketches (inline)

**Control loop (discrete PID-ish)**

```
   +-----------+      +----------+      +----------+
   |  Sensors  | ---> | Compare  | ---> | Actuator |
   +-----------+      | to setpt |      +----------+
         ^            +----------+             |
         |                  ^                 |
         |                  |                 v
   +-----------+      +----------+      +----------+
   | Process / | <--- |  Model   | <--- |  Setpoint|
   | Workload  |      | (predic) |      +----------+
   +-----------+      +----------+
```

- Sensors: latency, errors, depth, cost.  
- Compare: target vs observed (with hysteresis).  
- Actuator: rate limit, batch resize, scale, degrade.  
- Model: optional predictor or simple bounds.  
- Setpoint: SLO target or budget.

**Immune stack layers**

```
[Types/Invariants]  // illegal states unrepresentable
        |
[Serialization per aggregate]
        |
[Reflexes: deadlines, timeouts, rate limits, breakers]
        |
[Containment: kill switches, safemode, traffic shift]
        |
[Drills: chaos, replay, parity checks]
```

**Ecology/topology sketch**

```
 Clients --> Edge (membrane: auth, flags, cache) --> Services (cells)
                      ^                      \
                      |                       \--> Workers (actors/mailboxes)
               Control plane (policies, budgets)

 Resources flow: queues/logs as shared environment (stigmergy)
```

## Runbook wiring

- Add the **Emergence preflight checklist** as a template in runbooks; require it for new features.  
- Include the **Metrics map** as a table in dashboards so detectors/actions are obvious during incidents.  
- Link the **Runbooks (recipes)** to on-call pages with copy/pastable commands/flags.  
- Keep diagrams close: embed the control-loop and immune stack sketches in the ops docs for quick recall.

## Quick-start glossary

- **Emergence:** Coherent global behavior from simple local rules.  
- **Membrane:** Interface that controls what crosses (ports/adapters, middleware).  
- **Setpoint:** Target for a control loop (latency, cost, freshness).  
- **Fitness:** Metric to judge mutations/experiments (SLO burn, cost, conversion).  
- **Drift:** Divergence between intended state (schema/contracts) and observed state (projections/config).  
- **Safemode:** Degraded but safe operating mode with narrowed surface area.

## Emergence preflight checklist (ship gate)

- Invariants written and testable; illegal states unrepresentable.  
- Senses attached: metrics/traces/events for the new behavior; alarms defined.  
- Actuators ready: flag, kill switch, degrade path, backpressure knob, circuit breaker.  
- Control loop tuned: target/setpoint defined; hysteresis/bounds set; watchdog in place.  
- Tests run: replay/diff tests on recorded logs; property/simulation where applicable; chaos in staging if risky.  
- Rollback rehearsed: flag flip path, cache clear, compensating reducer/job if state touched.  
- Observability verified: dashboards show the new signals; runbook updated with triggers/actions/owners.

## Fitness function template (default goals)

- **SLO burn ↓:** error budget burn rate stays within policy (e.g., <1%/hour steady state).  
- **Latency steady:** p95 within target band; p99 not thrashing; tail protected.  
- **Cost efficient:** cost per request/job within budget; no runaway spend from retries or hotspots.  
- **Fairness:** per-tenant or per-key variance bounded; no starvation of low-volume tenants.  
- **Error mix healthy:** retries bounded; few timeouts; circuit trips rare and recover quickly.

## Visuals to add

- Control-loop diagram tied to the PID-ish template (sense → compare → act).  
- Ecology/topology map showing services as species, resources as flows, and membranes/interfaces.  
- Immune stack sketch layering types/invariants → reflexes → containment → drills.

## Building an emergent feature: a repeatable loop

1) **Define the genome.** Contracts, events, reducer/FSM transitions, invariants.  
2) **Attach senses.** Metrics, traces, domain events; declare what “healthy” looks like.  
3) **Add actuators.** Feature flags, circuit breakers, backpressure knobs, autoscaling rules.  
4) **Run small experiments.** Canaries + shadow traffic; observe global effects on throughput, error budget, and cost.  
5) **Select and stabilize.** Keep changes that improve fitness; codify as default policy; document learned thresholds.  
6) **Regenerate.** Periodically replay logs and chaos tests to ensure traits persist under stress.

## Anti-patterns (anti-evolution traits)

- **Centralized omniscient coordinator.** Removes local autonomy; single point of failure and friction.
- **Magic side effects.** Hidden concurrency/time/IO prevents observability and replay.
- **Hard-coded topology.** Tight wiring blocks new paths (no place for behavior to emerge).
- **One-way migrations with no rollback.** Organisms without a healing mechanism die easily.
- **Feature without senses.** Shipping code without telemetry is like adding a limb with no nerves.

## Signals to watch (vital signs)

- **Resilience:** MTTR, circuit break counts, retry storm detection, deadline budget adherence.  
- **Adaptation:** Time to ship a flagged experiment; % of traffic under progressive delivery.  
- **Coordination:** Queue lag per aggregate, mailbox depth variance, contention hotspots.  
- **Integrity:** Invariant breaches per replay, divergence between projections and source of truth.  
- **Energy use:** Cost per request/job, unnecessary work detected by sampling.

## How to teach the codebase to “heal”

- Default every risky path with a **kill switch** and **graceful degradation** mode.
- Make **state reconstruction** cheap (logs → reducers → projections) so recovery is replay, not surgery.
- Automate **rollback and cache invalidation** as first-class runbooks.
- Keep **chaos drills** in CI or staging to exercise the immune system before production.

## Culture notes

- Treat designs as **ecology maps**: who interacts, what resources flow, how feedback loops close.
- Reward teams for **clean experiments and fast recovery**, not just uptime or feature count.
- Write ADRs as **evolution notes**: which trait we tried, what metric moved, why we kept or reverted it.

*End of document — Appendix: Emergence as a Design Practice*
