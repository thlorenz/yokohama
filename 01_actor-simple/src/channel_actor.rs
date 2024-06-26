use crate::{common::ActorMessage, traits::ActorHandle};
use async_trait::async_trait;
use log::*;
use tokio::sync::{mpsc, oneshot};

// Seeing ~95µs request round trip time for same-thread consumer and ~150µs for separate-thread consumer

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
}

#[async_trait]
impl ActorHandle for ChannelActorHandle {
    async fn get_id(&mut self) -> u64 {
        let (send, recv) = oneshot::channel();

        let message = ActorMessage::GetId { respond_to: send };
        let _ = self.sender.send(message).await.map_err(|e| {
            warn!("Failed to send message: {:?}", e);
        });

        recv.await.expect("Actor task has been killed")
    }
}
