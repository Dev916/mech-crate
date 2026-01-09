//! Supervisor Actor
//!
//! Implements supervision patterns for fault tolerance.
//! See: appendix-actor-model.md - Section 3: Supervision & Fault Tolerance

use actix::prelude::*;
use std::collections::HashMap;
use std::time::Duration;

// ─────────────────────────────────────────────────────────────────────────────
// Supervision Strategies
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum SupervisionStrategy {
    /// Restart only the failed child
    OneForOne,
    /// Restart all children on any failure
    AllForOne,
    /// Restart failed child and all started after it
    RestForOne,
    /// Pass failure to parent supervisor
    Escalate,
    /// Terminate failed child permanently
    Stop,
}

#[derive(Debug, Clone)]
pub struct RestartPolicy {
    pub max_restarts: u32,
    pub within_seconds: u64,
    pub backoff: BackoffStrategy,
}

#[derive(Debug, Clone)]
pub enum BackoffStrategy {
    Immediate,
    Fixed(Duration),
    Exponential { base: Duration, max: Duration },
}

impl Default for RestartPolicy {
    fn default() -> Self {
        Self {
            max_restarts: 3,
            within_seconds: 60,
            backoff: BackoffStrategy::Exponential {
                base: Duration::from_millis(100),
                max: Duration::from_secs(30),
            },
        }
    }
}

impl RestartPolicy {
    pub fn next_delay(&self, restart_count: u32) -> Duration {
        match &self.backoff {
            BackoffStrategy::Immediate => Duration::ZERO,
            BackoffStrategy::Fixed(d) => *d,
            BackoffStrategy::Exponential { base, max } => {
                let delay = base.saturating_mul(2u32.saturating_pow(restart_count));
                std::cmp::min(delay, *max)
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Child Info
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug)]
struct ChildInfo {
    name: String,
    restart_count: u32,
    last_restart: std::time::Instant,
}

// ─────────────────────────────────────────────────────────────────────────────
// Supervisor Messages
// ─────────────────────────────────────────────────────────────────────────────

/// Report child failure
#[derive(Message)]
#[rtype(result = "SupervisionDecision")]
pub struct ChildFailed {
    pub child_name: String,
    pub error: String,
}

/// Register a child with the supervisor
#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterChild {
    pub name: String,
}

/// Unregister a child
#[derive(Message)]
#[rtype(result = "()")]
pub struct UnregisterChild {
    pub name: String,
}

#[derive(Debug, Clone)]
pub enum SupervisionDecision {
    Restart { delay: Duration },
    Escalate,
    Stop,
}

// ─────────────────────────────────────────────────────────────────────────────
// Supervisor Actor
// ─────────────────────────────────────────────────────────────────────────────

pub struct SupervisorActor {
    strategy: SupervisionStrategy,
    policy: RestartPolicy,
    children: HashMap<String, ChildInfo>,
}

impl SupervisorActor {
    pub fn new(strategy: SupervisionStrategy) -> Self {
        Self {
            strategy,
            policy: RestartPolicy::default(),
            children: HashMap::new(),
        }
    }

    pub fn with_policy(mut self, policy: RestartPolicy) -> Self {
        self.policy = policy;
        self
    }

    fn decide(&self, child: &mut ChildInfo) -> SupervisionDecision {
        let now = std::time::Instant::now();
        let within = Duration::from_secs(self.policy.within_seconds);

        // Reset counter if outside window
        if now.duration_since(child.last_restart) > within {
            child.restart_count = 0;
        }

        child.restart_count += 1;
        child.last_restart = now;

        if child.restart_count > self.policy.max_restarts {
            match self.strategy {
                SupervisionStrategy::Escalate => SupervisionDecision::Escalate,
                _ => SupervisionDecision::Stop,
            }
        } else {
            let delay = self.policy.next_delay(child.restart_count - 1);
            SupervisionDecision::Restart { delay }
        }
    }
}

impl Actor for SupervisorActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        tracing::info!("SupervisorActor started with {:?} strategy", self.strategy);
    }
}

impl Handler<RegisterChild> for SupervisorActor {
    type Result = ();

    fn handle(&mut self, msg: RegisterChild, _ctx: &mut Self::Context) {
        self.children.insert(
            msg.name.clone(),
            ChildInfo {
                name: msg.name,
                restart_count: 0,
                last_restart: std::time::Instant::now(),
            },
        );
    }
}

impl Handler<UnregisterChild> for SupervisorActor {
    type Result = ();

    fn handle(&mut self, msg: UnregisterChild, _ctx: &mut Self::Context) {
        self.children.remove(&msg.name);
    }
}

impl Handler<ChildFailed> for SupervisorActor {
    type Result = SupervisionDecision;

    fn handle(&mut self, msg: ChildFailed, _ctx: &mut Self::Context) -> Self::Result {
        tracing::warn!("Child {} failed: {}", msg.child_name, msg.error);

        match self.children.get_mut(&msg.child_name) {
            Some(child) => {
                let decision = self.decide(child);
                
                match &decision {
                    SupervisionDecision::Restart { delay } => {
                        tracing::info!(
                            "Will restart {} after {:?} (attempt {})",
                            msg.child_name,
                            delay,
                            child.restart_count
                        );
                    }
                    SupervisionDecision::Escalate => {
                        tracing::error!("Escalating failure of {} to parent", msg.child_name);
                    }
                    SupervisionDecision::Stop => {
                        tracing::error!("Stopping {} permanently after too many failures", msg.child_name);
                        self.children.remove(&msg.child_name);
                    }
                }

                decision
            }
            None => {
                tracing::warn!("Unknown child {} reported failure", msg.child_name);
                SupervisionDecision::Stop
            }
        }
    }
}
