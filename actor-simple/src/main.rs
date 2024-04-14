use channel_actor::{ChannelActorHandle, ChannelConsumer};
use log::*;
mod channel_actor;
mod common;

fn init_logger() {
    let log_level = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    std::env::set_var("RUST_LOG", log_level);
    env_logger::init();
}

#[tokio::main]
async fn main() {
    init_logger();
    let channel_actor_handle = ChannelActorHandle::new();
    let channel_consumer_uno = ChannelConsumer::new(channel_actor_handle.clone(), "uno");
    let channel_consumer_dos = ChannelConsumer::new(channel_actor_handle.clone(), "dos");

    tokio::try_join!(
        channel_consumer_uno.get_id_periodically(500),
        channel_consumer_dos.get_id_periodically(700)
    )
    .map_err(|e| error!("Error: {:?}", e))
    .unwrap();
}
