// Inspired by https://ryhl.io/blog/actors-with-tokio/

use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, RwLock,
    },
    time::{Duration, Instant},
};

use crate::common::ActorMessage;
use log::*;
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};

// -----------------
// ChannelActor
// -----------------
struct ChannelActor {
    receiver: mpsc::Receiver<ActorMessage>,
    next_id: u64,
}

impl ChannelActor {
    pub fn new(receiver: mpsc::Receiver<ActorMessage>) -> Self {
        Self {
            receiver,
            next_id: 0,
        }
    }

    fn handle_message(&mut self, message: ActorMessage) {
        match message {
            ActorMessage::GetId { respond_to } => {
                self.next_id += 1;
                let _ = respond_to.send(self.next_id).map_err(|e| {
                    warn!("Failed to send response: {:?}", e);
                });
            }
        }
    }
}

// -----------------
// ChannelActorHandle
// -----------------
#[derive(Clone)]
pub struct ChannelActorHandle {
    sender: mpsc::Sender<ActorMessage>,
}

impl ChannelActorHandle {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(100);
        let mut actor = ChannelActor::new(receiver);

        tokio::spawn(async move {
            while let Some(message) = actor.receiver.recv().await {
                actor.handle_message(message);
            }
        });

        Self { sender }
    }

    pub async fn get_id(&mut self) -> u64 {
        let (send, recv) = oneshot::channel();

        let message = ActorMessage::GetId { respond_to: send };
        let _ = self.sender.send(message).await.map_err(|e| {
            warn!("Failed to send message: {:?}", e);
        });

        recv.await.expect("Actor task has been killed")
    }
}

// -----------------
// ChannelConsumer
// -----------------
pub struct ChannelConsumer {
    handle: ChannelActorHandle,
    name: String,
    times: Arc<RwLock<Vec<Duration>>>,
}

impl ChannelConsumer {
    pub fn new(handle: ChannelActorHandle, name: &str) -> Self {
        Self {
            handle,
            name: name.to_string(),
            times: Arc::<RwLock<Vec<Duration>>>::default(),
        }
    }

    pub fn get_id_periodically(&self, interval_millis: u64) -> JoinHandle<()> {
        let mut handle = self.handle.clone();
        let name = self.name.clone();
        let times = self.times.clone();
        tokio::task::spawn(async move {
            run_request_loop(&times, &mut handle, &name, interval_millis).await;
        })
    }

    pub fn get_id_periodically_in_separate_runtime(
        &self,
        interval_millis: u64,
    ) -> std::thread::JoinHandle<()> {
        let mut handle = self.handle.clone();
        let name = self.name.clone();
        let times = self.times.clone();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(1)
                .enable_all()
                .thread_name(name.clone())
                .build()
                .unwrap();

            rt.block_on(async move {
                run_request_loop(&times, &mut handle, &name, interval_millis).await;
            });
        })
    }
}

async fn run_request_loop(
    times: &RwLock<Vec<Duration>>,
    handle: &mut ChannelActorHandle,
    name: &str,
    interval_millis: u64,
) {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    loop {
        let start = Instant::now();
        let id = handle.get_id().await;
        let elapsed = start.elapsed();
        times.write().unwrap().push(elapsed);

        let count = COUNTER.fetch_add(1, Ordering::Relaxed);
        if count % 100 == 0 {
            let total = times.read().unwrap().iter().sum::<Duration>();
            let avg = total / times.read().unwrap().len() as u32;
            info!(
                "Consumer '{}' got id: {} (avg: {}µs)",
                name,
                id,
                avg.as_nanos() as f64 / 1000.0
            );
        }
        tokio::time::sleep(Duration::from_millis(interval_millis)).await;
    }
}
