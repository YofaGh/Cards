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
}

#[derive(Clone)]
pub struct GameRegistry {
    factories: Arc<HashMap<String, GameFactory>>,
    active_games: Arc<Mutex<HashMap<GameId, ActiveGame>>>,
    game_queues: Arc<Mutex<HashMap<String, GameQueue>>>,
}

impl Default for GameRegistry {
    fn default() -> Self {
        Self {
            factories: Arc::new(HashMap::new()),
            active_games: Arc::new(Mutex::new(HashMap::new())),
            game_queues: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl GameRegistry {
    pub fn new() -> Self {
        let mut factories: HashMap<String, GameFactory> = HashMap::new();
        factories.insert("Qafoon".to_string(), Qafoon::boxed_new);
        let registry: GameRegistry = Self {
            factories: Arc::new(factories),
            active_games: Arc::new(Mutex::new(HashMap::new())),
            game_queues: Arc::new(Mutex::new(HashMap::new())),
        };
        registry.start_cleanup_service();
        registry
    }

    pub fn get_available_games(&self) -> Vec<String> {
        self.factories.keys().cloned().collect()
    }

    pub async fn add_player_to_queue(
        &self,
        username: String,
        game_choice: String,
        connection: Stream,
    ) -> Result<()> {
        let game_arc: Arc<Mutex<BoxGame>> = {
            let mut queues: MutexGuard<HashMap<String, GameQueue>> = self.game_queues.lock().await;
            if let Some(existing_queue) = queues.get(&game_choice) {
                existing_queue.game.clone()
            } else {
                let factory: &GameFactory = self
                    .factories
                    .get(&game_choice)
                    .ok_or_else(|| Error::Other(format!("Game {game_choice} is not supported")))?;
                let game: Arc<Mutex<BoxGame>> = Arc::new(Mutex::new(factory()));
                let new_queue: GameQueue = GameQueue {
                    game_type: game_choice.clone(),
                    game: game.clone(),
                    created_at: SystemTime::now(),
                    is_waiting: true,
                };
                queues.insert(game_choice.clone(), new_queue);
                game
            }
        };

        let mut game: MutexGuard<BoxGame> = game_arc.lock().await;
        if game.get_player_count() == 0 {
            game.initialize_game()?;
        }
        game.add_player(username, connection)?;
        if game.is_full() {
            drop(game);
            let mut queues: MutexGuard<HashMap<String, GameQueue>> = self.game_queues.lock().await;
            if let Some(queue) = queues.get_mut(&game_choice) {
                queue.is_waiting = false;
                let game_id: GameId = GameId::new_v4();
                let active_game: ActiveGame = ActiveGame {
                    id: game_id,
                    game_type: game_choice.clone(),
                    game: game_arc.clone(),
                    created_at: queue.created_at,
                };
                self.active_games.lock().await.insert(game_id, active_game);
                queues.remove(&game_choice);
                let registry: GameRegistry = self.clone();
                tokio::spawn(async move {
                    if let Err(err) = Self::run_full_game(game_arc, game_id, registry).await {
                        eprintln!("Game {game_id} failed: {err}");
                    }
                });
            }
        }
        Ok(())
    }

    async fn run_full_game(
        game_arc: Arc<Mutex<BoxGame>>,
        game_id: GameId,
        registry: GameRegistry,
    ) -> Result<()> {
        {
            let mut game: MutexGuard<BoxGame> = game_arc.lock().await;
            if let Err(err) = game.setup_teams().await {
                eprintln!("Team selection failed for game {game_id}: {err}");
                game.broadcast_message(BroadcastMessage::GameCancelled {
                    reason: "Team selection failed".to_string(),
                })
                .await?;
                return Err(err);
            }
            if let Err(err) = game.start().await {
                eprintln!("Game {game_id} failed to start: {err}");
                return Err(err);
            }
        }
        registry.remove_game(game_id).await?;
        Ok(())
    }

    fn start_cleanup_service(&self) {
        let queues: Arc<Mutex<HashMap<String, GameQueue>>> = self.game_queues.clone();
        let active_games: Arc<Mutex<HashMap<GameId, ActiveGame>>> = self.active_games.clone();
        let config: &'static Config = get_config();
        tokio::spawn(async move {
            let mut interval: tokio::time::Interval =
                tokio::time::interval(config.game_server.queue_clean_up_interval);
            loop {
                interval.tick().await;
                {
                    let mut queues_guard: MutexGuard<HashMap<String, GameQueue>> =
                        queues.lock().await;
                    let cutoff: SystemTime = SystemTime::now() - config.timeout.queue_cutoff;
                    let mut queues_to_remove: Vec<String> = Vec::new();
                    for (game_type, queue) in queues_guard.iter_mut() {
                        if queue.created_at <= cutoff {
                            if let Ok(mut game_guard) = queue.game.try_lock() {
                                if let Err(err) = game_guard
                                    .broadcast_message(BroadcastMessage::QueueTimeout)
                                    .await
                                {
                                    eprintln!(
                                        "Failed to notify players in queue {game_type}: {err}"
                                    );
                                }
                                for player in game_guard.get_players() {
                                    if let Err(err) = player.close_connection().await {
                                        eprintln!(
                                            "Failed to close connection for player {}: {err}",
                                            player.name
                                        );
                                    }
                                }
                            }
                            queues_to_remove.push(game_type.clone());
                        }
                    }
                    for game_type in queues_to_remove {
                        queues_guard.remove(&game_type);
                        println!("Cleaned up abandoned queue for {game_type}");
                    }
                }
                {
                    let mut active_guard: MutexGuard<HashMap<GameId, ActiveGame>> =
                        active_games.lock().await;
                    let initial_count: usize = active_guard.len();
                    active_guard.retain(|game_id: &GameId, active_game: &mut ActiveGame| {
                        if let Ok(game_guard) = active_game.game.try_lock() {
                            let should_keep: bool = !game_guard.is_finished();
                            if !should_keep {
                                println!("Cleaning up finished game {game_id}");
                            }
                            should_keep
                        } else {
                            true
                        }
                    });
                    let cleaned: usize = initial_count - active_guard.len();
                    if cleaned > 0 {
                        println!("Cleaned up {cleaned} finished games");
                    }
                }
            }
        });
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

    pub async fn list_active_games(&self) -> Vec<(GameId, String)> {
        self.active_games
            .lock()
            .await
            .values()
            .map(|game: &ActiveGame| (game.id, game.game_type.clone()))
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

pub async fn list_all_active_games() -> Vec<(GameId, String)> {
    get_game_registry().list_active_games().await
}
