use pubsub_service::PubsubService;

mod common;
mod errors;
mod pubsub_handler;
mod pubsub_service;
mod unsubscribers;

fn init_logger() {
    let log_level =
        std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    std::env::set_var("RUST_LOG", log_level);
    env_logger::builder().format_timestamp_millis().init();
}

fn main() {
    init_logger();
    console_subscriber::init();

    let mut service = PubsubService::new("127.0.0.1:6969");
    service.run();
}
