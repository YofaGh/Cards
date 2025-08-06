#![allow(dead_code)]

use crate::{models::Player, prelude::*};

#[async_trait]
pub trait Game: Send + Sync {
    fn get_players(&mut self) -> Vec<&mut Player>;
    fn add_player(&mut self, name: String, connection: Stream) -> Result<()>;
    fn get_player_count(&self) -> usize;
    fn is_full(&self) -> bool;
    fn get_status(&self) -> &GameStatus;
    fn initialize_game(&mut self) -> Result<()>;
    fn generate_cards(&mut self) -> Result<()>;
    fn set_status(&mut self, status: GameStatus);
    async fn start(&mut self) -> Result<()>;
    async fn setup_teams(&mut self) -> Result<()>;

    fn is_finished(&self) -> bool {
        self.get_status() == &GameStatus::Finished
    }
    fn is_started(&self) -> bool {
        self.get_status() == &GameStatus::Started
    }
    fn is_not_started(&self) -> bool {
        self.get_status() == &GameStatus::NotStarted
    }
    async fn _broadcast_message(&mut self, message: BroadcastMessage) -> Result<Vec<String>> {
        let game_message: GameMessage = GameMessage::Broadcast { message };
        let send_futures: Vec<_> = self
            .get_players()
            .into_iter()
            .map(|player: &mut Player| (player.name.clone(), player))
            .map(|(player_name, player)| {
                let game_message: GameMessage = game_message.clone();
                async move {
                    match player.send_message(&game_message).await {
                        Err(Error::Tcp(_)) => Some(player_name),
                        _ => None,
                    }
                }
            })
            .collect();
        let results: Vec<Option<String>> = futures::future::join_all(send_futures).await;
        let failed_players: Vec<String> = results
            .into_iter()
            .filter_map(|x: Option<String>| x)
            .collect();
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
    async fn end_game(&mut self, reason: String) -> Result<()> {
        self._broadcast_message(BroadcastMessage::GameCancelled { reason })
            .await
            .ok();
        for player in self.get_players() {
            player.close_connection().await.ok();
        }
        self.set_status(GameStatus::Ended);
        Err(Error::Other("Game ended".to_string()))
    }
}
