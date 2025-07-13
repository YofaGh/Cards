use futures::future::join_all;

use crate::{models::*, prelude::*};

#[async_trait]
pub trait Game: Send + Sync {
    fn get_players(&mut self) -> Vec<&mut Player>;
    fn add_player(&mut self, _name: String, _team_id: TeamId, _connection: Stream) -> Result<()>;
    fn get_player_count(&self) -> usize;
    fn is_full(&self) -> bool;
    fn get_status(&self) -> &GameStatus;
    fn initialize_game(&mut self) -> Result<()>;
    fn generate_cards(&mut self) -> Result<()>;
    async fn start(&mut self) -> Result<()>;
    async fn handle_user(&mut self, mut _connection: Stream, _name: String) -> Result<()>;

    fn is_finished(&self) -> bool {
        self.get_status() == &GameStatus::Finished
    }
    fn is_started(&self) -> bool {
        self.get_status() == &GameStatus::Started
    }
    fn is_not_started(&self) -> bool {
        self.get_status() == &GameStatus::NotStarted
    }
    async fn broadcast_message(&mut self, message: BroadcastMessage) -> Result<()> {
        let game_message: GameMessage = GameMessage::Broadcast { message };
        let send_futures: Vec<_> = self
            .get_players()
            .into_iter()
            .map(|player: &mut Player| async {
                player.send_message(&game_message).await.ok();
            })
            .collect();
        join_all(send_futures).await;
        Ok(())
    }
}

impl<T: Game + 'static> From<T> for BoxGame {
    fn from(game: T) -> Self {
        Box::new(game)
    }
}
