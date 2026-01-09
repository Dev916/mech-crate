//! Supervisor Actor
//!
//! Monitors worker health and implements restart strategies.
//! Based on Erlang/OTP supervision patterns.
//!
//! See: appendix-actor-model.md - Section 3: Supervision & Fault Tolerance

use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;

use super::{Actor, ActorAddr, spawn_actor};

/// Supervision strategies
#[derive(Debug, Clone, Copy)]
pub enum Strategy {
    /// Restart only the failed worker
    OneForOne,
    /// Restart all workers on any failure
    AllForOne,
    /// Stop permanently after max failures
    Stop,
}

/// Restart policy configuration
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
            max_restarts: 5,
            within_seconds: 60,
            backoff: BackoffStrategy::Exponential {
                base: Duration::from_millis(100),
                max: Duration::from_secs(30),
            },
        }
    }
}

/// Supervisor messages
pub enum SupervisorMessage {
    /// Worker failed
    WorkerFailed { worker_id: Uuid, error: String },
    /// Worker recovered
    WorkerRecovered { worker_id: Uuid },
    /// Register worker
    RegisterWorker { worker_id: Uuid },
    /// Unregister worker
    UnregisterWorker { worker_id: Uuid },
    /// Get health status
    GetHealth { reply: tokio::sync::oneshot::Sender<HealthStatus> },
}

#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub healthy: bool,
    pub total_workers: usize,
    pub failed_workers: usize,
    pub total_restarts: u32,
}

/// Worker tracking info
struct WorkerInfo {
    id: Uuid,
    restarts: Vec<Instant>,
    healthy: bool,
}

/// Supervisor actor
pub struct Supervisor {
    strategy: Strategy,
    policy: RestartPolicy,
    workers: HashMap<Uuid, WorkerInfo>,
    total_restarts: u32,
}

impl Supervisor {
    pub fn new(strategy: Strategy, policy: RestartPolicy) -> Self {
        Self {
            strategy,
            policy,
            workers: HashMap::new(),
            total_restarts: 0,
        }
    }

    pub fn spawn(self) -> ActorAddr<SupervisorMessage> {
        spawn_actor(self, 100)
    }

    fn should_restart(&self, worker: &WorkerInfo) -> Option<Duration> {
        let now = Instant::now();
        let cutoff = now - Duration::from_secs(self.policy.within_seconds);
        
        let recent_restarts = worker.restarts
            .iter()
            .filter(|&t| *t > cutoff)
            .count() as u32;

        if recent_restarts >= self.policy.max_restarts {
            return None; // Too many restarts
        }

        // Calculate backoff delay
        let delay = match &self.policy.backoff {
            BackoffStrategy::Immediate => Duration::ZERO,
            BackoffStrategy::Fixed(d) => *d,
            BackoffStrategy::Exponential { base, max } => {
                let delay = base.saturating_mul(2u32.pow(recent_restarts));
                std::cmp::min(delay, *max)
            }
        };

        Some(delay)
    }
}

#[async_trait::async_trait]
impl Actor for Supervisor {
    type Message = SupervisorMessage;

    async fn handle(&mut self, msg: Self::Message) {
        match msg {
            SupervisorMessage::WorkerFailed { worker_id, error } => {
                tracing::warn!(worker_id = %worker_id, error, "Worker failed");
                
                if let Some(worker) = self.workers.get_mut(&worker_id) {
                    worker.healthy = false;
                    worker.restarts.push(Instant::now());
                    
                    match self.should_restart(worker) {
                        Some(delay) => {
                            self.total_restarts += 1;
                            tracing::info!(
                                worker_id = %worker_id,
                                delay_ms = delay.as_millis(),
                                "Scheduling worker restart"
                            );
                            metrics::counter!("worker_restarts").increment(1);
                            
                            // In real implementation, would trigger restart here
                        }
                        None => {
                            tracing::error!(
                                worker_id = %worker_id,
                                "Worker exceeded restart limit, stopping"
                            );
                            metrics::counter!("worker_permanent_failures").increment(1);
                        }
                    }

                    // Handle AllForOne strategy
                    if matches!(self.strategy, Strategy::AllForOne) {
                        tracing::warn!("AllForOne strategy: restarting all workers");
                        for (_, w) in self.workers.iter_mut() {
                            if w.healthy {
                                w.healthy = false;
                                // Would trigger restart here
                            }
                        }
                    }
                }
            }

            SupervisorMessage::WorkerRecovered { worker_id } => {
                if let Some(worker) = self.workers.get_mut(&worker_id) {
                    worker.healthy = true;
                    tracing::info!(worker_id = %worker_id, "Worker recovered");
                }
            }

            SupervisorMessage::RegisterWorker { worker_id } => {
                self.workers.insert(worker_id, WorkerInfo {
                    id: worker_id,
                    restarts: Vec::new(),
                    healthy: true,
                });
                tracing::debug!(worker_id = %worker_id, "Worker registered");
            }

            SupervisorMessage::UnregisterWorker { worker_id } => {
                self.workers.remove(&worker_id);
                tracing::debug!(worker_id = %worker_id, "Worker unregistered");
            }

            SupervisorMessage::GetHealth { reply } => {
                let failed = self.workers.values().filter(|w| !w.healthy).count();
                let _ = reply.send(HealthStatus {
                    healthy: failed == 0,
                    total_workers: self.workers.len(),
                    failed_workers: failed,
                    total_restarts: self.total_restarts,
                });
            }
        }
    }

    async fn started(&mut self) {
        tracing::info!(strategy = ?self.strategy, "Supervisor started");
    }

    async fn stopped(&mut self) {
        tracing::info!(
            total_restarts = self.total_restarts,
            "Supervisor stopped"
        );
    }
}
