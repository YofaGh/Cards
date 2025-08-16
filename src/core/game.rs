#![allow(dead_code)]

use tokio::{
    io::{AsyncWriteExt, ReadHalf, WriteHalf},
    sync::oneshot,
    task::{JoinError, JoinHandle},
};

use crate::{
    core::send_message_to_player,
    models::{CorrelatedMessage, Player, PlayerConnection},
    prelude::*,
};

#[async_trait]
pub trait Game: Send + Sync {
    fn get_players(&mut self) -> Vec<&mut Player>;
    fn get_player_ids(&self) -> Vec<PlayerId>;
    fn add_player(&mut self, name: String, connection: Stream) -> Result<()>;
    fn get_player_count(&self) -> usize;
    fn is_full(&self) -> bool;
    fn get_status(&self) -> &GameStatus;
    fn initialize_game(&mut self) -> Result<()>;
    fn generate_cards(&mut self) -> Result<()>;
    fn set_status(&mut self, status: GameStatus);
    fn get_field(&self) -> Vec<PlayerId>;
    async fn run_game(&mut self) -> Result<()>;
    async fn setup_teams(&mut self) -> Result<()>;
    async fn update_shared_state(&self) -> Result<()>;
    fn get_player_sender(&self, player_id: PlayerId) -> Result<&Sender<CorrelatedMessage>>;
    fn remove_player_connection(&mut self, player_id: PlayerId) -> Option<PlayerConnection>;
    fn remove_player(&mut self, player_id: PlayerId);
    fn setup_receiver(
        &self,
        player_id: PlayerId,
        reader: ReadHalf<Stream>,
        sender: Sender<GameMessage>,
        req_sender: Sender<CorrelatedMessage>,
        shutdown_rx: oneshot::Receiver<()>,
    ) -> Result<JoinHandle<ReadHalf<Stream>>>;

    fn is_finished(&self) -> bool {
        self.get_status() == &GameStatus::Finished
    }
    fn is_started(&self) -> bool {
        self.get_status() == &GameStatus::Started
    }
    fn is_not_started(&self) -> bool {
        self.get_status() == &GameStatus::NotStarted
    }
    async fn send_message_to_player(
        &self,
        player_id: PlayerId,
        message: GameMessage,
    ) -> Result<()> {
        send_message_to_player(self.get_player_sender(player_id)?, message, player_id).await
    }
    fn setup_sender(
        &self,
        writer: WriteHalf<Stream>,
        mut receiver: Receiver<CorrelatedMessage>,
        mut shutdown_rx: oneshot::Receiver<()>,
    ) -> Result<JoinHandle<WriteHalf<Stream>>> {
        let handle: JoinHandle<WriteHalf<Stream>> = tokio::spawn(async move {
            let mut writer: WriteHalf<Stream> = writer;
            loop {
                tokio::select! {
                    _ = &mut shutdown_rx => {
                        println!("Sender shutting down");
                        break;
                    }
                    correlated_msg = receiver.recv() => {
                        match correlated_msg {
                            Some(CorrelatedMessage { message, response_tx }) => {
                                let success = crate::network::protocol::send_message_halved(&mut writer, &message)
                                    .await;
                                let _ = response_tx.send(success);
                            }
                            None => break,
                        }
                    }
                }
            }
            writer
        });
        Ok(handle)
    }
    async fn _broadcast_message(&mut self, message: BroadcastMessage) -> Result<Vec<String>> {
        let game_message: GameMessage = GameMessage::Broadcast { message };
        let infos: Vec<(PlayerId, String)> = self
            .get_players()
            .iter()
            .map(|player: &&mut Player| (player.id, player.name.clone()))
            .collect();
        let player_info: Vec<(PlayerId, String, Sender<CorrelatedMessage>)> = infos
            .into_iter()
            .filter_map(|(player_id, player_name)| {
                if let Ok(sender) = self.get_player_sender(player_id) {
                    return Some((player_id, player_name, sender.clone()));
                }
                None
            })
            .collect();
        let send_futures: Vec<_> = player_info
            .into_iter()
            .map(|(player_id, player_name, sender)| {
                let game_message: GameMessage = game_message.clone();
                async move {
                    if send_message_to_player(&sender, game_message, player_id)
                        .await
                        .is_err()
                    {
                        return Some(player_name);
                    }
                    None
                }
            })
            .collect();
        let results: Vec<Option<String>> = futures::future::join_all(send_futures).await;
        let failed_players: Vec<String> = results.into_iter().flatten().collect();
        Ok(failed_players)
    }
    async fn broadcast_message(&mut self, message: BroadcastMessage) -> Result<()> {
        let failed_players: Vec<String> = self._broadcast_message(message).await?;
        if failed_players.is_empty() {
            return Ok(());
        }
        let reason: String = format!("Failed to send message to {}", failed_players.join(", "));
        self.end_game(reason).await
    }
    async fn close_player_connection(&mut self, player_id: PlayerId) -> Result<()> {
        if let Some(connection) = self.remove_player_connection(player_id) {
            let _ = connection.reader_shutdown_tx.send(());
            let _ = connection.writer_shutdown_tx.send(());
            let reader_result: Result<ReadHalf<Stream>, JoinError> = connection.reader_handle.await;
            let writer_result: Result<WriteHalf<Stream>, JoinError> =
                connection.writer_handle.await;
            match (reader_result, writer_result) {
                (Ok(reader), Ok(writer)) => {
                    let mut stream: Stream = reader.unsplit(writer);
                    if let Err(err) = stream.shutdown().await {
                        println!("Error shutting down stream for player {player_id:?}: {err:?}");
                    }
                    println!("Successfully closed connection for player {player_id:?}");
                }
                (Err(err1), Err(err2)) => {
                    println!(
                        "Both tasks failed for player {player_id:?}: reader={err1:?}, writer={err2:?}"
                    );
                }
                (Err(err), _) => {
                    println!("Reader task failed for player {player_id:?}: {err:?}");
                }
                (_, Err(err)) => {
                    println!("Writer task failed for player {player_id:?}: {err:?}");
                }
            }
        }
        self.remove_player(player_id);
        Ok(())
    }
    async fn start_game(&mut self) -> Result<()> {
        self.setup_teams().await?;
        self.broadcast_message(BroadcastMessage::GameStarting)
            .await?;
        self.run_game().await
    }
    async fn end_game(&mut self, reason: String) -> Result<()> {
        self._broadcast_message(BroadcastMessage::GameCancelled { reason })
            .await
            .ok();
        for player_id in self.get_player_ids() {
            self.close_player_connection(player_id).await.ok();
        }
        self.set_status(GameStatus::Ended);
        Err(Error::Other("Game ended".to_string()))
    }
}
