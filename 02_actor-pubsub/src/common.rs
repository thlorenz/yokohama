use jsonrpc_core::Params;
use serde::{Deserialize, Serialize};

// -----------------
// ResultWithSubscriptionId
// -----------------
#[derive(Serialize, Debug)]
pub struct ResultWithSubscriptionId<T: Serialize> {
    pub result: T,
    pub subid: u64,
}

impl<T: Serialize> ResultWithSubscriptionId<T> {
    pub fn new(result: T, subscription: u64) -> Self {
        Self {
            result,
            subid: subscription,
        }
    }

    fn into_value_map(self) -> serde_json::Map<String, serde_json::Value> {
        let mut map = serde_json::Map::new();
        map.insert(
            "result".to_string(),
            serde_json::to_value(self.result).unwrap(),
        );
        map.insert(
            "subid".to_string(),
            serde_json::to_value(self.subid).unwrap(),
        );
        map
    }

    pub fn into_params_map(self) -> Params {
        Params::Map(self.into_value_map())
    }
}

// -----------------
// TickSubscription
// -----------------
#[derive(Serialize, Deserialize, Debug)]
pub struct TickSubscription {
    pub interval: u64,
}
