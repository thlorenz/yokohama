use async_trait::async_trait;

#[async_trait]
pub trait ActorHandle: Clone + Send + 'static {
    async fn get_id(&mut self) -> u64;
}
