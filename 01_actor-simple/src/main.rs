// Inspired by https://ryhl.io/blog/actors-with-tokio/

use channel_actor::ChannelActorHandle;
use consumer::ChannelConsumer;
use log::*;
use mutex_actor::MutexActorHandle;
use traits::ActorHandle;

mod channel_actor;
mod common;
mod consumer;
mod mutex_actor;
mod traits;

fn init_logger() {
    let log_level =
        std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    std::env::set_var("RUST_LOG", log_level);
    env_logger::builder()
        .format_timestamp(None)
        .format_module_path(false)
        .format_target(false)
        .init();
}

#[tokio::main]
async fn main() {
    init_logger();
    console_subscriber::init();

    if std::env::args().any(|arg| arg == "--mutex") {
        run(MutexActorHandle::new()).await;
    } else {
        run(ChannelActorHandle::new()).await;
    }
}

async fn run<T: ActorHandle>(handle: T) {
    let consumer_uno = ChannelConsumer::new(handle.clone(), "uno");
    let consumer_dos = ChannelConsumer::new(handle.clone(), "dos");
    let consumer_tres = ChannelConsumer::new(handle.clone(), "tres");

    let hdl_tres = consumer_tres.get_id_periodically_in_separate_runtime(15);

    tokio::try_join!(
        consumer_uno.get_id_periodically(10),
        consumer_dos.get_id_periodically(20)
    )
    .map_err(|e| error!("Error: {:?}", e))
    .unwrap();

    hdl_tres.join().unwrap();
}
