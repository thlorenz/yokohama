use log::*;
use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};
use tokio::sync::{mpsc, oneshot};

use crate::errors::{PubsubError, PubsubResult};

pub enum Subscription {
    Ticker {
        ack_subid: oneshot::Sender<u64>,
        ticker: mpsc::Sender<u64>,
        interval: Duration,
    },
}

struct PubsubActorImpl {
    subscriptions: mpsc::Receiver<Subscription>,
    sub_id: AtomicU64,
}

impl PubsubActorImpl {
    pub fn new(subscriptions: mpsc::Receiver<Subscription>) -> Self {
        Self {
            subscriptions,
            sub_id: AtomicU64::default(),
        }
    }

    fn get_subid(&self) -> u64 {
        self.sub_id.fetch_add(1, Ordering::Relaxed)
    }

    async fn handle_subscription(&mut self, subscription: Subscription) {
        match subscription {
            Subscription::Ticker {
                ack_subid,
                ticker,
                interval,
            } => {
                let subid = self.get_subid();
                let _ = ack_subid.send(subid).map_err(|e| {
                    warn!("Failed to send response: {:?}", e);
                });
                let mut count = 0;
                loop {
                    // TODO: unsubscribe
                    tokio::time::sleep(interval).await;
                    count += 1;
                    let _ = ticker.send(count).await.map_err(|e| {
                        warn!("Failed to send response: {:#?}", e);
                    });
                }
            }
        }
    }
}

// -----------------
// PubsubActor
// -----------------
#[derive(Clone)]
pub struct PubsubActor {
    subscribe: mpsc::Sender<Subscription>,
}

impl PubsubActor {
    pub fn new_separate_thread() -> Self {
        let (subscribe, subscriptions) = mpsc::channel(100);
        let mut actor = PubsubActorImpl::new(subscriptions);

        std::thread::spawn(move || {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .thread_name("PubsubActorRuntime")
                .build()
                .unwrap()
                .block_on(async move {
                    while let Some(subscription) =
                        actor.subscriptions.recv().await
                    {
                        actor.handle_subscription(subscription).await;
                    }
                });
        });

        Self { subscribe }
    }

    pub fn sub_ticker(
        &self,
        interval: Duration,
    ) -> PubsubResult<(u64, mpsc::Receiver<u64>)> {
        let (subid_tx, subid_rx) = oneshot::channel();
        let (ticker_tx, ticker_rx) = mpsc::channel(100);
        self.subscribe
            .blocking_send(Subscription::Ticker {
                ack_subid: subid_tx,
                ticker: ticker_tx,
                interval,
            })
            .map_err(|err| {
                PubsubError::FailedToSendSubscription(Box::new(err))
            })?;

        let subid = subid_rx.blocking_recv().map_err(|err| {
            PubsubError::FailedToConfirmSubscription(Box::new(err))
        })?;

        Ok((subid, ticker_rx))
    }
}
