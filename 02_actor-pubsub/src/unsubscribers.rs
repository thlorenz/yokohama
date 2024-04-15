use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use tokio_util::sync::CancellationToken;

#[derive(Clone)]
pub struct Unsubscribers {
    unsubscribers: Arc<Mutex<HashMap<u64, CancellationToken>>>,
}

impl Unsubscribers {
    pub fn new() -> Self {
        Self {
            unsubscribers:
                Arc::<Mutex<HashMap<u64, CancellationToken>>>::default(),
        }
    }

    pub fn add(&self, id: u64) -> CancellationToken {
        let unsubscriber = CancellationToken::new();
        let mut unsubscribers = self.unsubscribers.lock().unwrap();
        unsubscribers.insert(id, unsubscriber.clone());
        unsubscriber
    }

    pub fn unsubscribe(&self, id: u64) {
        let mut unsubscribers = self.unsubscribers.lock().unwrap();
        if let Some(unsubscriber) = unsubscribers.remove(&id) {
            unsubscriber.cancel();
        }
    }
}
