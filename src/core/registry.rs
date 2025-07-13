#![allow(dead_code)]

use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
    time::SystemTime,
};
use tokio::sync::{Mutex, MutexGuard};

use crate::{games::*, prelude::*};

pub struct GameQueue {
    pub game_type: String,
    pub game: Arc<Mutex<BoxGame>>,
    pub created_at: SystemTime,
    pub is_waiting: bool,
}

pub struct ActiveGame {
    pub id: GameId,
    pub game_type: String,
    pub game: Arc<Mutex<BoxGame>>,
    pub created_at: SystemTime,
    pub player_count: usize,
}

#[derive(Default)]
pub struct GameRegistry {
    factories: HashMap<String, GameFactory>,
    active_games: Arc<Mutex<HashMap<GameId, ActiveGame>>>,
    game_queues: Arc<Mutex<HashMap<String, GameQueue>>>,
}

impl GameRegistry {
    pub fn new() -> Self {
        let mut registry: GameRegistry = Self::default();
        registry.register("Qafoon", || Box::new(Qafoon::new()));
        registry
    }

    pub fn register(&mut self, name: &str, factory: GameFactory) {
        self.factories.insert(name.to_string(), factory);
    }

    pub fn get_available_games(&self) -> Vec<String> {
        self.factories.keys().cloned().collect()
    }

    pub async fn find_or_create_queue(&self, game_type: &str) -> Result<Arc<Mutex<BoxGame>>> {
        let mut queues: MutexGuard<HashMap<String, GameQueue>> = self.game_queues.lock().await;
        if let Some(queue) = queues.get(game_type) {
            if queue.is_waiting {
                return Ok(queue.game.clone());
            }
        }
        let factory: &GameFactory = self
            .factories
            .get(game_type)
            .ok_or_else(|| Error::Other(format!("Game {game_type} is not supported")))?;
        let game: Arc<Mutex<BoxGame>> = Arc::new(Mutex::new(factory()));
        game.lock().await.initialize_game()?;
        let queue: GameQueue = GameQueue {
            game_type: game_type.to_string(),
            game: game.clone(),
            created_at: SystemTime::now(),
            is_waiting: true,
        };
        queues.insert(game_type.to_string(), queue);
        Ok(game)
    }

    pub async fn promote_queue_to_active(&self, game_type: &str) -> Result<GameId> {
        let mut queues: MutexGuard<HashMap<String, GameQueue>> = self.game_queues.lock().await;
        let mut active_games: MutexGuard<HashMap<GameId, ActiveGame>> =
            self.active_games.lock().await;
        if let Some(queue) = queues.remove(game_type) {
            let game_id: GameId = GameId::new_v4();
            let active_game: ActiveGame = ActiveGame {
                id: game_id,
                game_type: game_type.to_string(),
                game: queue.game,
                created_at: queue.created_at,
                player_count: 0,
            };
            active_games.insert(game_id, active_game);
            Ok(game_id)
        } else {
            Err(Error::Other(format!(
                "No queue found for game type: {game_type}"
            )))
        }
    }

    pub async fn get_active_game(&self, game_id: GameId) -> Option<Arc<Mutex<BoxGame>>> {
        self.active_games
            .lock()
            .await
            .get(&game_id)
            .map(|active: &ActiveGame| active.game.clone())
    }

    pub async fn remove_game(&self, game_id: GameId) -> Result<()> {
        self.active_games.lock().await.remove(&game_id);
        Ok(())
    }

    pub async fn list_active_games(&self) -> Vec<(GameId, String, usize)> {
        self.active_games
            .lock()
            .await
            .values()
            .map(|game: &ActiveGame| (game.id, game.game_type.clone(), game.player_count))
            .collect()
    }

    pub async fn get_games_by_type(&self, game_type: &str) -> Vec<GameId> {
        self.active_games
            .lock()
            .await
            .values()
            .filter(|game: &&ActiveGame| game.game_type == game_type)
            .map(|game: &ActiveGame| game.id)
            .collect()
    }

    pub async fn get_active_games_count(&self) -> usize {
        self.active_games.lock().await.len()
    }

    pub async fn cleanup_finished_games(&self) -> Result<usize> {
        let mut active_games: MutexGuard<HashMap<GameId, ActiveGame>> =
            self.active_games.lock().await;
        let mut to_remove: Vec<GameId> = Vec::new();
        for (id, active_game) in active_games.iter() {
            if let Ok(game_guard) = active_game.game.try_lock() {
                if game_guard.is_finished() {
                    to_remove.push(*id);
                }
            }
        }
        let removed_count: usize = to_remove.len();
        for id in to_remove {
            active_games.remove(&id);
        }
        Ok(removed_count)
    }
}

static GAME_REGISTRY: OnceLock<GameRegistry> = OnceLock::new();

pub fn get_game_registry() -> &'static GameRegistry {
    GAME_REGISTRY.get_or_init(GameRegistry::new)
}

pub async fn get_active_game(game_id: GameId) -> Option<Arc<Mutex<BoxGame>>> {
    get_game_registry().get_active_game(game_id).await
}

pub async fn list_all_active_games() -> Vec<(GameId, String, usize)> {
    get_game_registry().list_active_games().await
}
