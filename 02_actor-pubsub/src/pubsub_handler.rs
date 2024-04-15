use jsonrpc_pubsub::{Subscriber, SubscriptionId};
use log::*;
use std::time::{Duration, Instant};
use tokio::{sync::mpsc, task::JoinSet};
use tokio_util::sync::CancellationToken;

use crate::{
    common::ResultWithSubscriptionId,
    errors::{reject_internal_error, PubsubError, PubsubResult},
    unsubscribers::Unsubscribers,
};

pub enum Subscription {
    Ticker {
        subscriber: Subscriber,
        interval: Duration,
    },
}

impl Subscription {
    pub fn into_subscriber(self) -> Subscriber {
        match self {
            Subscription::Ticker { subscriber, .. } => subscriber,
        }
    }
}

struct SubscriptionsReceiver {
    subscriptions: mpsc::Receiver<Subscription>,
}

impl SubscriptionsReceiver {
    pub fn new(subscriptions: mpsc::Receiver<Subscription>) -> Self {
        Self { subscriptions }
    }
}

async fn handle_subscription(
    subscription: Subscription,
    subid: u64,
    unsubscriber: CancellationToken,
) {
    match subscription {
        Subscription::Ticker {
            interval,
            subscriber,
        } => {
            let sink = subscriber
                .assign_id(SubscriptionId::Number(subid))
                .map_err(|e| {
                    error!("Failed to assign subscription id: {:?}", e);
                })
                .unwrap();
            debug!("Subscribing to ticker: {}", subid);
            let mut tick = 0;
            let start = Instant::now();
            loop {
                tokio::select! {
                    _ = unsubscriber.cancelled() => {
                        debug!("Unsubscribing from ticker: {}", subid);
                        break;
                    },
                    _ = tokio::time::sleep(interval) => {
                        tick += 1;
                        let res = ResultWithSubscriptionId::new(tick, subid);
                        if sink.notify(res.into_params_map()).is_err() {
                            debug!("Subscripion has ended without proper unsubscribe");
                            break;
                        }
                    }
                };
            }
            let elapsed = start.elapsed();
            debug!("Ticker subscription {} lasted for {:?}", subid, elapsed);
        }
    }
}

// -----------------
// PubsubActor
// -----------------
#[derive(Clone)]
pub struct PubsubActor {
    subscribe: mpsc::Sender<Subscription>,
    unsubscribers: Unsubscribers,
}

impl PubsubActor {
    pub fn new() -> Self {
        let (subscribe_tx, subscribe_rx) = mpsc::channel(100);
        let unsubscribers = Unsubscribers::new();
        {
            let unsubscribers = unsubscribers.clone();
            std::thread::spawn(move || {
                tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .thread_name("PubsubActorRuntime")
                .build()
                .unwrap()
                .block_on(async move {
                    let mut subid: u64 = 0;
                    let mut pending_subs = JoinSet::new();
                    let mut actor = SubscriptionsReceiver::new(subscribe_rx);

                    // Waiting for either of the two:
                    // a) a new subscriptions comes in and we add it to pending subscriptions
                    // b) polling subs, once done they are auto-removed from pending subscriptions
                    loop {
                        tokio::select! {
                            subscription = actor.subscriptions.recv() => {
                                match subscription {
                                    Some(subscription) => {
                                        subid += 1;
                                        let unsubscriber = unsubscribers.add(subid);
                                        pending_subs
                                            .spawn(handle_subscription(
                                                subscription,
                                                subid,
                                                unsubscriber
                                            ));
                                        debug!("Added subscription to a total of {}",
                                            pending_subs.len());
                                    },
                                    None => break,
                                };
                            },
                            next = pending_subs.join_next() => {
                                if let Some(Err(err)) = next {
                                    error!("Failed to join task: {:?}", err)
                                }
                            }
                        }
                    }
                });
            });
        }

        Self {
            subscribe: subscribe_tx,
            unsubscribers,
        }
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
                let err_msg = format!("{:?}", err);
                let subscription = err.0;
                let subscriber = subscription.into_subscriber();
                reject_internal_error(
                    subscriber,
                    "Failed to subscribe",
                    Some(err_msg.clone()),
                );

                PubsubError::FailedToSendSubscription(err_msg)
            })?;

        Ok(())
    }

    pub fn unsubscribe(&self, id: u64) {
        self.unsubscribers.unsubscribe(id);
    }
}
