use jsonrpc_core::{futures, BoxFuture, MetaIoHandler, Params};
use log::*;
use serde_json::Value;
use std::{sync::Arc, time::Duration};

use jsonrpc_pubsub::{PubSubHandler, Session, Subscriber, SubscriptionId};
use jsonrpc_ws_server::{RequestContext, Server, ServerBuilder};

use crate::{common::TickSubscription, pubsub_handler::PubsubActor};

pub struct PubsubService {
    server: Option<Server>,
    actor: PubsubActor,
}

impl PubsubService {
    pub fn new(url: &str) -> Self {
        let mut io = PubSubHandler::new(MetaIoHandler::default());
        let mut service = Self {
            server: None,
            actor: PubsubActor::new_separate_thread(),
        };

        service.add_version_subscription(&mut io);
        service.add_ticker_subscription(&mut io);

        let server = ServerBuilder::with_meta_extractor(
            io,
            |context: &RequestContext| Arc::new(Session::new(context.sender())),
        )
        .start(&url.parse().unwrap())
        .expect("Unable to start RPC server");

        service.server.replace(server);

        info!("Created Pubsub server at {}", url);
        service
    }

    fn add_version_subscription(&self, io: &mut PubSubHandler<Arc<Session>>) {
        io.add_sync_method("version", |_params: Params| {
            Ok(Value::String("1.0.0".to_string()))
        });
    }

    fn add_ticker_subscription(&self, io: &mut PubSubHandler<Arc<Session>>) {
        let subscribe = {
            let actor = self.actor.clone();
            move |params: Params,
                  _session: Arc<Session>,
                  subscriber: Subscriber| {
                // All subscriptions come in on the same subscribe thread, while this is
                // different from the main thread it causes one subscription blocking
                // other subscriptions if it performs tasks synchronously.
                // Additionally if we don't put the receive calls of ticks on a separate
                // thread then this function never returns and the client never receives the
                // subscription confirmation nor any ticks.
                let thread = std::thread::current();
                debug!(
                    "tick sub thread: {:?} - {:?}",
                    thread.name(),
                    thread.id()
                );

                info!("params: {:#?}", params);

                let interval = params
                    .parse::<TickSubscription>()
                    .expect("Invalid params")
                    .interval;

                if let Err(err) = actor
                    .sub_ticker(subscriber, Duration::from_millis(interval))
                {
                    error!("Failed to subscribe to ticker: {:?}", err);
                    // reject_internal_error(
                    //     subscriber,
                    //     "Failed to subscribe",
                    //     Some(err),
                    // );
                };
            }
        };

        let unsubscribe = {
            let actor = self.actor.clone();
            move |id: SubscriptionId,
                  _session: Option<Arc<Session>>|
                  -> BoxFuture<jsonrpc_core::Result<Value>> {
                debug!("Closing tick subscription");
                match id {
                    SubscriptionId::Number(id) => {
                        actor.unsubscribe(id);
                    }
                    SubscriptionId::String(_) => {
                        warn!("subscription id should be a number")
                    }
                }
                Box::pin(futures::future::ready(Ok(Value::Bool(true))))
            }
        };

        io.add_subscription(
            "tickNotification",
            ("tickSubscribe", subscribe),
            ("tickUnsubscribe", unsubscribe),
        );
    }

    pub fn run(&mut self) {
        let server = self.server.take().expect("Call run only once.");
        // std::thread::spawn(move || {
        let thread = std::thread::current();
        debug!("Server thread: {:?} - {:?}", thread.name(), thread.id());
        let _ = server.wait();
        // });
    }
}
