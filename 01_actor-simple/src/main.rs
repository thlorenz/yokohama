// Inspired by https://ryhl.io/blog/actors-with-tokio/

use channel_actor::ChannelActorHandle;
use consumer::ChannelConsumer;
use log::*;
mod channel_actor;
mod common;
mod consumer;

fn init_logger() {
    let log_level = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
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

    let channel_actor_handle = ChannelActorHandle::new();
    let channel_consumer_uno = ChannelConsumer::new(channel_actor_handle.clone(), "uno");
    let channel_consumer_dos = ChannelConsumer::new(channel_actor_handle.clone(), "dos");
    let channel_consumer_tres = ChannelConsumer::new(channel_actor_handle.clone(), "tres");

    let hdl_tres = channel_consumer_tres.get_id_periodically_in_separate_runtime(15);

    tokio::try_join!(
        channel_consumer_uno.get_id_periodically(10),
        channel_consumer_dos.get_id_periodically(20)
    )
    .map_err(|e| error!("Error: {:?}", e))
    .unwrap();

    hdl_tres.join().unwrap();
}
