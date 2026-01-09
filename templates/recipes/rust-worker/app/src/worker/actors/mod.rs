//! Actor Model Implementation
//!
//! Lightweight actors for job coordination and supervision.
//! Based on message-passing and isolated state.
//!
//! See: appendix-actor-model.md

mod job_coordinator;
mod supervisor;

pub use job_coordinator::*;
pub use supervisor::*;

use async_channel::{Receiver, Sender, bounded};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Actor address for sending messages
#[derive(Clone)]
pub struct ActorAddr<M> {
    sender: Sender<M>,
}

impl<M: Send + 'static> ActorAddr<M> {
    pub async fn send(&self, msg: M) -> Result<(), async_channel::SendError<M>> {
        self.sender.send(msg).await
    }

    pub fn try_send(&self, msg: M) -> Result<(), async_channel::TrySendError<M>> {
        self.sender.try_send(msg)
    }
}

/// Actor context for receiving messages
pub struct ActorContext<M> {
    pub receiver: Receiver<M>,
    pub id: Uuid,
}

/// Create a new actor channel
pub fn actor_channel<M>(buffer: usize) -> (ActorAddr<M>, ActorContext<M>) {
    let (sender, receiver) = bounded(buffer);
    let addr = ActorAddr { sender };
    let ctx = ActorContext {
        receiver,
        id: Uuid::new_v4(),
    };
    (addr, ctx)
}

/// Actor trait for implementing actors
#[async_trait::async_trait]
pub trait Actor: Send + Sync + 'static {
    type Message: Send + 'static;
    
    /// Handle a single message
    async fn handle(&mut self, msg: Self::Message);
    
    /// Called when actor starts
    async fn started(&mut self) {}
    
    /// Called when actor stops
    async fn stopped(&mut self) {}
}

/// Spawn an actor and return its address
pub fn spawn_actor<A: Actor>(mut actor: A, buffer: usize) -> ActorAddr<A::Message> {
    let (addr, ctx) = actor_channel(buffer);
    
    tokio::spawn(async move {
        actor.started().await;
        
        while let Ok(msg) = ctx.receiver.recv().await {
            actor.handle(msg).await;
        }
        
        actor.stopped().await;
    });
    
    addr
}
