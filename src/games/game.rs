use futures::future::join_all;

use crate::{models::*, prelude::*};

#[async_trait]
pub trait Game: Send + Sync {
    fn get_players(&mut self) -> Vec<&mut Player>;
    fn add_player(&mut self, _name: String, _team_id: TeamId, _connection: Stream) -> Result<()> {
        Ok(())
    }

    fn get_player_count(&self) -> usize {
        0
    }

    fn is_full(&self) -> bool {
        false
    }

    fn initialize_game(&mut self) -> Result<()> {
        Ok(())
    }

    async fn handle_user(&mut self, mut _connection: Stream, _name: String) -> Result<()> {
        Ok(())
    }

    fn generate_cards(&mut self) -> Result<()> {
        Ok(())
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

    async fn run_game(&mut self) -> Result<()> {
        Ok(())
    }
}

impl<T: Game + 'static> From<T> for BoxGame {
    fn from(module: T) -> Self {
        Box::new(module)
    }
}
