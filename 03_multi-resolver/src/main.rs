use std::time::Duration;

use tokio::{task::JoinSet, time::sleep};

mod oneshot;

#[tokio::main]
async fn main() {
    let resolver = oneshot::OneshotMultiResolver::new();
    let mut handles = JoinSet::new();
    for i in 0..10 {
        let resolver = resolver.clone();
        let start_time = std::time::Instant::now();
        handles.spawn(async move {
            let id = 42;
            let result = resolver.request(id).await;
            let elapsed = start_time.elapsed();
            println!(
                "Result {} for id {}: {} after {:?}",
                i, id, result, elapsed
            );
        });
        sleep(Duration::from_millis(50)).await;
    }
    while (handles.join_next().await).is_some() {}
}
