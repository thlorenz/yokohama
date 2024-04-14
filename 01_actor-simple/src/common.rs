use tokio::sync::oneshot;

pub enum ActorMessage {
    GetId { respond_to: oneshot::Sender<u64> },
}
