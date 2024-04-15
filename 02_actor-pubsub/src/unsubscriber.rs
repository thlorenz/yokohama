use std::{
    collections::HashMap,
    future::Future,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
};

#[derive(Clone)]
pub struct Unsubscriber {
    unsubscribed: Arc<AtomicBool>,
}

impl Unsubscriber {
    pub fn new() -> Self {
        Self {
            unsubscribed: Arc::<AtomicBool>::default(),
        }
    }

    pub fn unsubscribe(&self) {
        self.unsubscribed.store(true, Ordering::SeqCst);
    }
}

impl Future for Unsubscriber {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if self.unsubscribed.load(Ordering::SeqCst) {
            std::task::Poll::Ready(())
        } else {
            std::task::Poll::Pending
        }
    }
}

#[derive(Clone)]
pub struct Unsubscribers {
    unsubscribers: Arc<RwLock<HashMap<u64, Unsubscriber>>>,
}

impl Unsubscribers {
    pub fn new() -> Self {
        Self {
            unsubscribers: Arc::<RwLock<HashMap<u64, Unsubscriber>>>::default(),
        }
    }

    pub fn add(&self, id: u64) -> Unsubscriber {
        let unsubscriber = Unsubscriber::new();
        let mut unsubscribers = self.unsubscribers.write().unwrap();
        unsubscribers.insert(id, unsubscriber.clone());
        unsubscriber
    }

    pub fn unsubscribe(&self, id: u64) {
        let mut unsubscribers = self.unsubscribers.write().unwrap();
        if let Some(unsubscriber) = unsubscribers.remove(&id) {
            unsubscriber.unsubscribe()
        }
    }
}
