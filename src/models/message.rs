#[derive(Debug)]
pub struct CorrelatedMessage {
    pub message: super::GameMessage,
    pub response_tx: tokio::sync::oneshot::Sender<crate::core::Result<()>>,
}
