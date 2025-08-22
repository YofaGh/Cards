#![allow(dead_code)]

use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
    time::SystemTime,
};
use tokio::sync::{Mutex, MutexGuard};

use crate::{
    games::*,
    models::{UserSession, UserSessionStatus},
    network::{close_connection, send_message},
    prelude::*,
};

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
    pub started_at: SystemTime,
    pub timeout_at: Option<SystemTime>,
    pub reconnection_sender: Sender<(PlayerId, Stream)>,
    pub player_ids: Vec<PlayerId>,
}

#[derive(Clone)]
pub struct GameRegistry {
    factories: Arc<HashMap<String, GameFactory>>,
    state: Arc<Mutex<RegistryState>>,
}

#[derive(Default)]
struct RegistryState {
    active_games: HashMap<GameId, ActiveGame>,
    game_queues: HashMap<String, GameQueue>,
    user_sessions: HashMap<UserId, UserSession>,
}

impl Default for GameRegistry {
    fn default() -> Self {
        Self {
            factories: Arc::new(HashMap::new()),
            state: Arc::new(Mutex::new(RegistryState {
                active_games: HashMap::new(),
                game_queues: HashMap::new(),
                user_sessions: HashMap::new(),
            })),
        }
    }
}

impl GameRegistry {
    pub fn new() -> Self {
        let mut factories: HashMap<String, GameFactory> = HashMap::new();
        factories.insert("Qafoon".to_string(), Qafoon::boxed_new);
        let registry: GameRegistry = Self {
            factories: Arc::new(factories),
            state: Arc::new(Mutex::new(RegistryState::default())),
        };
        registry.start_cleanup_service();
        registry
    }

    pub fn get_available_games(&self) -> Vec<String> {
        self.factories.keys().cloned().collect()
    }

    pub async fn is_user_in_game(&self, user_id: UserId) -> bool {
        self.state.lock().await.user_sessions.contains_key(&user_id)
    }

    pub async fn get_user_session(&self, user_id: UserId) -> Option<UserSession> {
        self.state.lock().await.user_sessions.get(&user_id).cloned()
    }

    async fn remove_user_session(&self, user_id: UserId) {
        self.state.lock().await.user_sessions.remove(&user_id);
    }

    pub async fn add_player_to_queue(
        &self,
        user_id: UserId,
        username: String,
        game_choice: String,
        mut connection: Stream,
    ) -> Result<()> {
        {
            let state: MutexGuard<RegistryState> = self.state.lock().await;
            if let Some(existing_session) = state.user_sessions.get(&user_id) {
                let message: GameMessage = match existing_session.status {
                    UserSessionStatus::InQueue => GameMessage::AlreadyInQueueError {
                        game_type: existing_session.game_type.clone(),
                    },
                    UserSessionStatus::InGame => GameMessage::AlreadyInGameError {
                        game_type: existing_session.game_type.clone(),
                    },
                };
                let _ = send_message(&mut connection, &message).await;
                let _ = close_connection(&mut connection).await;
                return Err(Error::Registry("User already in game".to_string()));
            }
        }
        let game_arc: Arc<Mutex<BoxGame>> = self.get_or_create_queue(&game_choice).await?;
        let (game_id, player_added) = {
            let mut game: MutexGuard<BoxGame> = game_arc.lock().await;
            if game.get_player_count() == 0 {
                if let Err(err) = game.initialize_game() {
                    self.cleanup_failed_queue(&game_choice).await;
                    return Err(err);
                }
            }
            let game_id: GameId = game.get_id();
            let reconnection_token: crate::auth::TokenPair =
                match crate::auth::generate_reconnection_token(user_id, game_id) {
                    Ok(token) => token,
                    Err(e) => {
                        if game.get_player_count() == 0 {
                            drop(game);
                            self.cleanup_failed_queue(&game_choice).await;
                        }
                        return Err(Error::Registry(format!(
                            "Failed to generate reconnection token: {e}"
                        )));
                    }
                };
            if let Err(e) = send_message(
                &mut connection,
                &GameMessage::ReconnectionToken {
                    token: reconnection_token.access_token,
                },
            )
            .await
            {
                if game.get_player_count() == 0 {
                    drop(game);
                    self.cleanup_failed_queue(&game_choice).await;
                }
                return Err(Error::Registry(format!(
                    "Failed to send reconnection token: {e}"
                )));
            }
            match game.add_player(user_id, username.clone(), connection) {
                Ok(_) => Ok((game_id, game.is_full())),
                Err(err) => {
                    if game.get_player_count() == 0 {
                        drop(game);
                        self.cleanup_failed_queue(&game_choice).await;
                    }
                    Err(err)
                }
            }
        }?;
        {
            self.state.lock().await.user_sessions.insert(
                user_id,
                UserSession {
                    user_id,
                    username: username.clone(),
                    game_id,
                    game_type: game_choice.clone(),
                    status: UserSessionStatus::InQueue,
                    joined_at: SystemTime::now(),
                },
            );
        }
        println!("Added user {username} to queue for {game_choice}");
        if player_added {
            self.promote_full_game(&game_choice, game_arc).await?;
        }
        Ok(())
    }

    pub async fn reconnect_player(
        &self,
        player_id: PlayerId,
        game_id: GameId,
        connection: Stream,
    ) -> Result<()> {
        if let Some(sender) = self.get_active_game_sender(game_id).await {
            if let Err(err) = sender.send((player_id, connection)).await {
                return Err(Error::Registry(format!(
                    "Failed to reconnect player: {err}"
                )));
            }
        }
        Ok(())
    }

    async fn get_or_create_queue(&self, game_choice: &str) -> Result<Arc<Mutex<BoxGame>>> {
        let mut state: MutexGuard<RegistryState> = self.state.lock().await;
        if let Some(existing_queue) = state.game_queues.get(game_choice) {
            if existing_queue.is_waiting {
                if let Ok(game) = existing_queue.game.try_lock() {
                    if !game.is_full() {
                        return Ok(existing_queue.game.clone());
                    }
                }
            }
        }
        self.create_new_queue_locked(&mut state, game_choice).await
    }

    async fn create_new_queue_locked(
        &self,
        state: &mut RegistryState,
        game_choice: &str,
    ) -> Result<Arc<Mutex<BoxGame>>> {
        let factory: &GameFactory = self
            .factories
            .get(game_choice)
            .ok_or_else(|| Error::Registry(format!("Game {game_choice} is not supported")))?;
        let game: Arc<Mutex<BoxGame>> = Arc::new(Mutex::new(factory()));
        let new_queue: GameQueue = GameQueue {
            game_type: game_choice.to_string(),
            game: game.clone(),
            created_at: SystemTime::now(),
            is_waiting: true,
        };
        state.game_queues.insert(game_choice.to_string(), new_queue);
        Ok(game)
    }

    async fn promote_full_game(
        &self,
        game_choice: &str,
        game_arc: Arc<Mutex<BoxGame>>,
    ) -> Result<()> {
        let (game_id, reconnection_sender, player_ids) = {
            let mut game: MutexGuard<BoxGame> = game_arc.lock().await;
            let player_ids: Vec<PlayerId> = game.get_player_ids();
            (game.get_id(), game.setup_reconnection()?, player_ids)
        };
        {
            let mut state: MutexGuard<RegistryState> = self.state.lock().await;
            for player_id in &player_ids {
                if let Some(session) = state.user_sessions.get_mut(player_id) {
                    session.status = UserSessionStatus::InGame;
                }
            }
        }
        let registry_clone: GameRegistry = self.clone();
        let game_arc_clone: Arc<Mutex<BoxGame>> = game_arc.clone();
        {
            let mut state: MutexGuard<RegistryState> = self.state.lock().await;
            if let Some(queue) = state.game_queues.remove(game_choice) {
                let timeout_at: SystemTime = SystemTime::now() + get_config().timeout.game_duration;
                let active_game: ActiveGame = ActiveGame {
                    id: game_id,
                    game_type: game_choice.to_string(),
                    game: game_arc.clone(),
                    created_at: queue.created_at,
                    started_at: SystemTime::now(),
                    timeout_at: Some(timeout_at),
                    reconnection_sender,
                    player_ids: player_ids.clone(),
                };
                state.active_games.insert(game_id, active_game);
            } else {
                return Err(Error::Registry(format!(
                    "Queue for {game_choice} was removed before promotion"
                )));
            }
        }
        tokio::spawn(async move {
            let result: Result<()> = Self::run_full_game_with_timeout(
                game_arc_clone,
                game_id,
                registry_clone,
                player_ids,
            )
            .await;
            if let Err(err) = result {
                eprintln!("Game {game_id} failed: {err}");
            }
        });
        Ok(())
    }

    async fn run_full_game_with_timeout(
        game_arc: Arc<Mutex<BoxGame>>,
        game_id: GameId,
        registry: GameRegistry,
        player_ids: Vec<PlayerId>,
    ) -> Result<()> {
        let game_result: Result<Result<()>, tokio::time::error::Elapsed> =
            tokio::time::timeout(get_config().timeout.game_duration, async {
                game_arc.lock().await.start_game().await
            })
            .await;
        match game_result {
            Ok(Ok(_)) => {
                println!("Game {game_id} completed successfully");
            }
            Ok(Err(err)) => {
                eprintln!("Game {game_id} failed: {err}");
                if let Ok(mut game) = game_arc.try_lock() {
                    let _ = game
                        .broadcast_message(BroadcastMessage::GameError {
                            error: err.to_string(),
                        })
                        .await;
                }
            }
            Err(_) => {
                eprintln!("Game {game_id} timed out");
                if let Ok(mut game) = game_arc.try_lock() {
                    let _ = game.broadcast_message(BroadcastMessage::GameTimeout).await;
                    for player_id in &player_ids {
                        let _ = game.close_player_connection(*player_id).await;
                    }
                }
            }
        }
        for player_id in player_ids {
            registry.remove_user_session(player_id).await;
        }
        registry.remove_game(game_id).await?;
        Ok(())
    }

    async fn cleanup_failed_queue(&self, game_choice: &str) {
        let (should_remove, user_ids_to_cleanup) = {
            let state: MutexGuard<RegistryState> = self.state.lock().await;
            if let Some(queue) = state.game_queues.get(game_choice) {
                if let Ok(game) = queue.game.try_lock() {
                    let player_count: usize = game.get_player_count();
                    let user_ids: Vec<PlayerId> = if player_count == 0 {
                        game.get_player_ids()
                    } else {
                        vec![]
                    };
                    (player_count == 0, user_ids)
                } else {
                    (false, vec![])
                }
            } else {
                (false, vec![])
            }
        };
        if should_remove {
            let mut state: MutexGuard<RegistryState> = self.state.lock().await;
            state.game_queues.remove(game_choice);
            for user_id in user_ids_to_cleanup {
                state.user_sessions.remove(&user_id);
            }
        }
    }

    fn start_cleanup_service(&self) {
        let state_arc: Arc<Mutex<RegistryState>> = self.state.clone();
        let config: &'static Config = get_config();
        tokio::spawn(async move {
            let mut interval: tokio::time::Interval =
                tokio::time::interval(config.game_server.queue_clean_up_interval);
            loop {
                interval.tick().await;
                if let Err(err) = Self::cleanup_tick(&state_arc, config).await {
                    eprintln!("Cleanup service error: {err}");
                }
            }
        });
    }

    async fn cleanup_tick(
        state_arc: &Arc<Mutex<RegistryState>>,
        config: &'static Config,
    ) -> Result<()> {
        let now: SystemTime = SystemTime::now();
        let queue_cutoff: SystemTime = now - config.timeout.queue_cutoff;
        let (expired_queues, finished_games, initial_game_count) = {
            let state: MutexGuard<RegistryState> = state_arc.lock().await;
            let expired_queues: Vec<(String, Arc<Mutex<BoxGame>>)> = state
                .game_queues
                .iter()
                .filter(|(_, queue)| queue.created_at <= queue_cutoff)
                .map(|(game_type, queue)| (game_type.clone(), queue.game.clone()))
                .collect();
            let finished_games: Vec<(GameId, Arc<Mutex<BoxGame>>, Vec<PlayerId>)> = state
                .active_games
                .iter()
                .filter_map(|(game_id, active_game)| {
                    let should_remove_timeout: bool = active_game
                        .timeout_at
                        .map(|timeout_at: SystemTime| now >= timeout_at)
                        .unwrap_or(false);
                    let should_remove_finished: bool = active_game
                        .game
                        .try_lock()
                        .map(|game: MutexGuard<BoxGame>| game.is_finished())
                        .unwrap_or(false);
                    if should_remove_timeout || should_remove_finished {
                        Some((
                            *game_id,
                            active_game.game.clone(),
                            active_game.player_ids.clone(),
                        ))
                    } else {
                        None
                    }
                })
                .collect();
            let initial_game_count: usize = state.active_games.len();
            (expired_queues, finished_games, initial_game_count)
        };
        {
            let mut state: MutexGuard<RegistryState> = state_arc.lock().await;
            for (game_type, _) in &expired_queues {
                if let Some(queue) = state.game_queues.remove(game_type) {
                    if let Ok(game) = queue.game.try_lock() {
                        for player_id in game.get_player_ids() {
                            state.user_sessions.remove(&player_id);
                        }
                    }
                }
                println!("Cleaned up expired queue for {game_type}");
            }
            for (game_id, _, player_ids) in &finished_games {
                state.active_games.remove(game_id);
                for player_id in player_ids {
                    state.user_sessions.remove(player_id);
                }
                println!("Cleaned up finished game {game_id}");
            }
        }
        for (_, game_arc) in expired_queues {
            if let Ok(mut game) = game_arc.try_lock() {
                let _ = game.broadcast_message(BroadcastMessage::QueueTimeout).await;
                let player_ids: Vec<PlayerId> = game.get_player_ids();
                for player_id in player_ids {
                    let _ = game.close_player_connection(player_id).await;
                }
            }
        }
        for (_, game_arc, _) in finished_games {
            if let Ok(mut game) = game_arc.try_lock() {
                let player_ids: Vec<PlayerId> = game.get_player_ids();
                for player_id in player_ids {
                    let _ = game.close_player_connection(player_id).await;
                }
            }
        }
        let final_count: usize = state_arc.lock().await.active_games.len();
        let cleaned_games: usize = initial_game_count.saturating_sub(final_count);
        if cleaned_games > 0 {
            println!("Cleaned up {cleaned_games} finished games");
        }
        Ok(())
    }

    pub async fn get_active_game(&self, game_id: GameId) -> Option<Arc<Mutex<BoxGame>>> {
        self.state
            .lock()
            .await
            .active_games
            .get(&game_id)
            .map(|active: &ActiveGame| active.game.clone())
    }

    pub async fn get_active_game_sender(
        &self,
        game_id: GameId,
    ) -> Option<Sender<(PlayerId, Stream)>> {
        self.state
            .lock()
            .await
            .active_games
            .get(&game_id)
            .map(|active: &ActiveGame| active.reconnection_sender.clone())
    }

    pub async fn remove_game(&self, game_id: GameId) -> Result<()> {
        if let Some(active_game) = self.state.lock().await.active_games.remove(&game_id) {
            drop(active_game.reconnection_sender);
            if let Ok(mut game) = active_game.game.try_lock() {
                for player_id in game.get_player_ids() {
                    let _ = game.close_player_connection(player_id).await;
                }
            }
            println!("Cleaned up game {game_id} with all its resources");
        }
        Ok(())
    }

    pub async fn list_active_games(&self) -> Vec<(GameId, String)> {
        self.state
            .lock()
            .await
            .active_games
            .values()
            .map(|game: &ActiveGame| (game.id, game.game_type.clone()))
            .collect()
    }

    pub async fn get_games_by_type(&self, game_type: &str) -> Vec<GameId> {
        self.state
            .lock()
            .await
            .active_games
            .values()
            .filter(|game: &&ActiveGame| game.game_type == game_type)
            .map(|game: &ActiveGame| game.id)
            .collect()
    }

    pub async fn get_active_games_count(&self) -> usize {
        self.state.lock().await.active_games.len()
    }

    pub async fn force_remove_user_session(&self, user_id: UserId) -> Result<()> {
        if let Some(session) = self.state.lock().await.user_sessions.remove(&user_id) {
            println!(
                "Force removed user {user_id} from {} session in {}",
                session.status.as_str(),
                session.game_type
            );
        }
        Ok(())
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
