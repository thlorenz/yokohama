use std::sync::{Arc, Mutex};

use crate::traits::ActorHandle;
use async_trait::async_trait;

// Not _really_ an actor, but provides same ActorHandle interface so consumers are unaware.
//
// Seeing ~0.9µs request round trip time for same-thread and separate-thread consumers
//
// It's about 100x faster than the channel-based actor and
// could be further improved via a RwLock.

// However this only addresses the use case of accessing shared state, but not
// consumer -> worker scenarios

#[derive(Clone)]
pub struct MutexActorHandle {
    state: Arc<Mutex<u64>>,
}

impl MutexActorHandle {
    pub fn new() -> Self {
        Self {
            state: Arc::<Mutex<u64>>::default(),
        }
    }
}

#[async_trait]
impl ActorHandle for MutexActorHandle {
    async fn get_id(&mut self) -> u64 {
        let mut state = self.state.lock().unwrap();
        *state += 1;
        *state
    }
}
