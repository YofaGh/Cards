#![allow(dead_code)]

use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
    time::SystemTime,
};
use tokio::sync::{Mutex, MutexGuard};

use crate::{games::*, prelude::*};

pub struct ActiveGame {
    pub id: GameId,
    pub game_type: String,
    pub game: Arc<Mutex<BoxGame>>,
    pub created_at: SystemTime,
    pub player_count: usize,
}

pub struct GameRegistry {
    factories: HashMap<String, GameFactory>,
    active_games: Arc<Mutex<HashMap<GameId, ActiveGame>>>,
}

impl GameRegistry {
    pub fn new() -> Self {
        let mut registry: GameRegistry = Self {
            factories: HashMap::new(),
            active_games: Arc::new(Mutex::new(HashMap::new())),
        };
        registry.register("Qafoon", || Box::new(Qafoon::new()));
        registry
    }

    pub fn register(&mut self, name: &str, factory: GameFactory) {
        self.factories.insert(name.to_string(), factory);
    }

    pub async fn create_game(&self, name: &str) -> Result<(GameId, Arc<Mutex<BoxGame>>)> {
        let factory: &GameFactory = self
            .factories
            .get(name)
            .ok_or_else(|| Error::Other(format!("Game {name} is not supported")))?;
        let game: Arc<Mutex<BoxGame>> = Arc::new(Mutex::new(factory()));
        let game_id: GameId = GameId::new_v4();
        let active_game: ActiveGame = ActiveGame {
            id: game_id,
            game_type: name.to_string(),
            game: game.clone(),
            created_at: SystemTime::now(),
            player_count: 0,
        };
        self.active_games.lock().await.insert(game_id, active_game);
        Ok((game_id, game))
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
                if game_guard.get_status() == &GameStatus::Finished {
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

pub async fn create_tracked_game(name: &str) -> Result<(GameId, Arc<Mutex<BoxGame>>)> {
    get_game_registry().create_game(name).await
}

pub async fn get_active_game(game_id: GameId) -> Option<Arc<Mutex<BoxGame>>> {
    get_game_registry().get_active_game(game_id).await
}

pub async fn list_all_active_games() -> Vec<(GameId, String, usize)> {
    get_game_registry().list_active_games().await
}
