use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, RwLock,
    },
    time::{Duration, Instant},
};

use log::*;
use tokio::task::JoinHandle;

use crate::traits::ActorHandle;

pub struct ChannelConsumer<T: ActorHandle> {
    handle: T,
    name: String,
    times: Arc<RwLock<Vec<Duration>>>,
}

impl<T: ActorHandle> ChannelConsumer<T> {
    pub fn new(handle: T, name: &str) -> Self {
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

async fn run_request_loop<T: ActorHandle>(
    times: &RwLock<Vec<Duration>>,
    handle: &mut T,
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
