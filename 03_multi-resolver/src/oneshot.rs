use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use tokio::{
    sync::oneshot,
    task::{self},
    time::sleep,
};

type NumberResult = u128;
type NumberRequests = Vec<oneshot::Sender<NumberResult>>;
type NumberRequestsMap =
    Mutex<HashMap<u64, (Option<NumberResult>, NumberRequests)>>;

#[derive(Debug, Clone)]
pub struct OneshotMultiResolver {
    requests: Arc<NumberRequestsMap>,
}

impl OneshotMultiResolver {
    pub fn new() -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn resolve(
        requests: &NumberRequestsMap,
        id: u64,
        result: NumberResult,
    ) {
        let requests = {
            let mut requests = requests.lock().unwrap();
            let requests_for_id = requests.remove(&id);
            requests.insert(id, (Some(result), Vec::new()));
            requests_for_id
        };
        if let Some((_, resolvers)) = requests {
            for resolver in resolvers.into_iter() {
                let _ = resolver.send(result);
            }
        }
    }

    pub async fn request(&self, id: u64) -> NumberResult {
        let requests_map = self.requests.clone();
        {
            let requests = requests_map.lock().unwrap();

            // A) The same result has already been requested
            if let Some((result, _)) = requests.get(&id) {
                // We already have the result and can just return it
                if let Some(result) = result {
                    let result = *result;
                    return result;
                }
            }
            // B)  This is the first request for this id
            else {
                let requests_map = requests_map.clone();
                task::spawn(async move {
                    let result = fetch_number(id).await;
                    Self::resolve(&requests_map, id, result).await;
                });
            }
        }

        // We didn't have the result yet, but it has been requested
        // Let's tell the resolver to tell us when it is ready
        let (tx, rx) = oneshot::channel();
        requests_map
            .lock()
            .unwrap()
            .entry(id)
            .or_insert((None, Vec::new()))
            .1
            .push(tx);
        rx.await.unwrap()
    }
}

async fn fetch_number(id: u64) -> NumberResult {
    sleep(Duration::from_millis(200)).await;
    id as u128 * id as u128
}
