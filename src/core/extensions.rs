use crate::{
    core::PlayerId,
    models::{CorrelatedMessage, GameMessage},
    prelude::{BTreeMap, Error, Result},
};

pub trait GetOrError<K, V> {
    fn get_or_error(&self, key: &K, error_fn: impl FnOnce() -> Error) -> Result<&V>;
    fn get_mut_or_error(&mut self, key: &K, error_fn: impl FnOnce() -> Error) -> Result<&mut V>;
}

impl<K: Ord, V> GetOrError<K, V> for BTreeMap<K, V> {
    fn get_or_error(&self, key: &K, error_fn: impl FnOnce() -> Error) -> Result<&V> {
        self.get(key).ok_or_else(error_fn)
    }
    fn get_mut_or_error(&mut self, key: &K, error_fn: impl FnOnce() -> Error) -> Result<&mut V> {
        self.get_mut(key).ok_or_else(error_fn)
    }
}

pub trait TimeoutExt<T> {
    fn timeout_context(self, context: impl Into<String>) -> Result<T>;
}

impl<T> TimeoutExt<T> for Result<Result<T>, tokio::time::error::Elapsed> {
    fn timeout_context(self, context: impl Into<String>) -> Result<T> {
        match self {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(err)) => Err(err),
            Err(_elapsed) => Err(Error::Timeout(context.into())),
        }
    }
}

pub fn read_file(path: impl AsRef<std::path::Path>) -> Result<Vec<u8>> {
    std::fs::read(path).map_err(Error::read_file)
}

pub async fn send_message_to_player(
    sender: &tokio::sync::mpsc::Sender<CorrelatedMessage>,
    message: GameMessage,
    player_id: &PlayerId,
) -> Result<()> {
    let (response_tx, response_rx) = tokio::sync::oneshot::channel();
    let correlated_message: CorrelatedMessage = CorrelatedMessage {
        message,
        response_tx,
    };
    sender
        .send(correlated_message)
        .await
        .map_err(|_| Error::Tcp(format!("Failed to send message to player: {player_id}")))?;
    match response_rx.await {
        Ok(result) => result,
        _ => Err(Error::Tcp(format!(
            "Response channel closed for player: {player_id}"
        ))),
    }
}
