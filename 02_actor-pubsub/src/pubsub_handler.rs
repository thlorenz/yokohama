use jsonrpc_pubsub::{Subscriber, SubscriptionId};
use log::*;
use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};
use tokio::sync::mpsc;

use crate::{
    common::ResultWithSubscriptionId,
    errors::{PubsubError, PubsubResult},
};

pub enum Subscription {
    Ticker {
        subscriber: Subscriber,
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
                interval,
                subscriber,
            } => {
                let subid = self.get_subid();
                let sink = subscriber
                    .assign_id(SubscriptionId::Number(subid))
                    .map_err(|e| {
                        error!("Failed to assign subscription id: {:?}", e);
                    })
                    .unwrap();
                let mut tick = 0;
                loop {
                    // TODO: unsubscribe
                    tokio::time::sleep(interval).await;
                    tick += 1;
                    let res = ResultWithSubscriptionId::new(tick, subid);
                    if sink.notify(res.into_params_map()).is_err() {
                        debug!("Subscripion has ended");
                        break;
                    }
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
        subscriber: Subscriber,
        interval: Duration,
    ) -> PubsubResult<()> {
        self.subscribe
            .blocking_send(Subscription::Ticker {
                interval,
                subscriber,
            })
            .map_err(|err| {
                PubsubError::FailedToSendSubscription(Box::new(err))
            })?;

        Ok(())
    }
}
