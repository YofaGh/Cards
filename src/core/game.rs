use tokio::{
    io::{AsyncWriteExt, ReadHalf, WriteHalf},
    sync::oneshot,
    task::JoinHandle,
    time::{timeout, Duration},
};

use super::{send_message_to_player, timed_choice};
use crate::{
    games::INVALID_RESPONSE,
    models::{CorrelatedMessage, Player, PlayerConnection},
    network::close_connection,
    prelude::*,
};

#[async_trait]
pub trait Game: Send + Sync {
    fn add_player(&mut self, player_id: PlayerId, name: String, connection: Stream) -> Result<()>;
    fn clean_up(&mut self);
    fn generate_cards(&mut self) -> Result<()>;
    fn get_available_teams(&self) -> Result<Vec<(TeamId, String)>>;
    fn get_field(&self) -> Vec<PlayerId>;
    fn get_id(&self) -> GameId;
    fn get_players(&mut self) -> Vec<&mut Player>;
    fn get_player_ids(&self) -> Vec<PlayerId>;
    fn get_player(&self, player_id: PlayerId) -> Result<&Player>;
    fn get_player_count(&self) -> usize;
    fn get_status(&self) -> &GameStatus;
    fn get_player_sender(&self, player_id: PlayerId) -> Result<&Sender<CorrelatedMessage>>;
    fn get_player_receiver(&mut self, player_id: PlayerId) -> Result<&mut Receiver<GameMessage>>;
    fn get_reconnection_receiver(&mut self) -> Result<&mut Receiver<(PlayerId, Stream)>>;
    fn initialize_game(&mut self) -> Result<()>;
    fn is_full(&self) -> bool;
    fn remove_player_channels(&mut self, player_id: PlayerId);
    fn remove_player_connection(&mut self, player_id: PlayerId) -> Option<PlayerConnection>;
    fn set_status(&mut self, status: GameStatus);
    fn setup_reconnection(&mut self) -> Result<Sender<(PlayerId, Stream)>>;
    fn setup_player_connection(&mut self, player_id: PlayerId, connection: Stream) -> Result<()>;
    fn setup_receiver(
        &self,
        player_id: PlayerId,
        reader: ReadHalf<Stream>,
        sender: Sender<GameMessage>,
        req_sender: Sender<CorrelatedMessage>,
        shutdown_rx: oneshot::Receiver<()>,
    ) -> Result<JoinHandle<ReadHalf<Stream>>>;
    async fn get_semi_state(&self) -> Result<Value>;
    async fn run_game(&mut self) -> Result<()>;
    async fn setup_teams(&mut self) -> Result<()>;
    async fn send_player_full_state(&mut self, player_id: PlayerId) -> Result<()>;
    async fn update_shared_state(&self) -> Result<()>;

    fn is_finished(&self) -> bool {
        self.get_status() == &GameStatus::Finished
    }

    fn is_started(&self) -> bool {
        self.get_status() == &GameStatus::Started
    }

    async fn receive_message_from_player(
        &mut self,
        player_id: PlayerId,
    ) -> Result<Option<GameMessage>> {
        Ok(self.get_player_receiver(player_id)?.recv().await)
    }

    async fn reconnect_disconnected_player(
        &mut self,
        player_id: PlayerId,
        connection: Stream,
    ) -> Result<()> {
        let _ = self.close_player_connection(player_id).await;
        self.setup_player_connection(player_id, connection)
    }

    fn get_player_reconnection_timeout(&self) -> std::time::Duration {
        get_config().timeout.player_reconnection
    }

    fn get_player_reconnection_max_retires(&self) -> usize {
        get_config().timeout.player_reconnection_max_retries
    }

    async fn get_player_choice(
        &mut self,
        player_id: PlayerId,
        message: &mut GameMessage,
        passable: bool,
        max_value: usize,
    ) -> Result<PlayerChoice> {
        let player_name: String = self.get_player(player_id)?.name.clone();
        let operation = async {
            loop {
                self.send_message_to_player(player_id, player_name.clone(), message.clone())
                    .await?;
                match self.receive_message_from_player(player_id).await? {
                    Some(GameMessage::PlayerChoice { choice }) => {
                        if choice == "pass" {
                            if passable {
                                return Ok(PlayerChoice::Pass);
                            }
                            message.set_demand_error("You can't pass this one".to_owned());
                        } else if message.message_type() == "Hokm" {
                            return Ok(PlayerChoice::HokmChoice(Hokm::from(choice)));
                        } else if message.message_type() == "Bet" {
                            if let Ok(choice) = choice.parse::<usize>() {
                                if choice <= max_value {
                                    return Ok(PlayerChoice::NumberChoice(choice));
                                }
                                message.set_demand_error(format!(
                                    "Choice can't be greater than {max_value}"
                                ));
                            } else {
                                message.set_demand_error(INVALID_RESPONSE.to_owned());
                            }
                        } else {
                            match crate::models::Card::try_from(choice) {
                                Ok(card) => {
                                    if self.get_player(player_id)?.cards.contains(&card) {
                                        return Ok(PlayerChoice::CardChoice(card));
                                    }
                                    message
                                        .set_demand_error("You don't have this card!".to_owned());
                                }
                                Err(_) => message.set_demand_error(INVALID_RESPONSE.to_owned()),
                            }
                        }
                    }
                    Some(invalid) => {
                        message.set_demand_error(format!(
                            "Expected message type PlayerChoice, but received {}",
                            invalid.message_type()
                        ));
                    }
                    None => {
                        return Err(Error::Tcp("Receiver was closed".to_string()));
                    }
                }
            }
        };
        match timed_choice(operation, player_name.clone()).await {
            Err(Error::Tcp(_)) => {
                self.end_game(format!("Player {player_name} left")).await?;
                Err(Error::Tcp("Player {player_name} left".to_string()))
            }
            other => other,
        }
    }

    async fn get_player_team_choice(&mut self, player_id: PlayerId) -> Result<TeamId> {
        let player_name: String = self.get_player(player_id)?.name.clone();
        let available_teams: Vec<(TeamId, String)> = self.get_available_teams()?;
        let mut message: GameMessage = GameMessage::team(
            available_teams
                .iter()
                .map(|(_, name)| name.clone())
                .collect(),
            String::new(),
        );
        let operation = async {
            loop {
                self.send_message_to_player(player_id, player_name.clone(), message.clone())
                    .await?;
                match self.receive_message_from_player(player_id).await? {
                    Some(GameMessage::PlayerChoice { choice }) => {
                        if let Some((team_id, _)) =
                            available_teams.iter().find(|(_, name)| *name == choice)
                        {
                            return Ok(*team_id);
                        } else {
                            message.set_demand_error("Invalid team choice".to_owned());
                        }
                    }
                    Some(invalid) => {
                        message.set_demand_error(format!(
                            "Expected PlayerChoice, got {}",
                            invalid.message_type()
                        ));
                    }
                    None => {
                        return Err(Error::Tcp("Receiver was closed".to_string()));
                    }
                }
            }
        };
        match timed_choice(operation, player_name.clone()).await {
            Err(Error::Tcp(_)) => {
                self.end_game(format!("Player {player_name} left")).await?;
                Err(Error::Tcp("Player {player_name} left".to_string()))
            }
            other => other,
        }
    }

    async fn send_message_to_player(
        &mut self,
        player_id: PlayerId,
        player_name: String,
        message: GameMessage,
    ) -> Result<()> {
        let max_retries: usize = self.get_player_reconnection_max_retires();
        let mut attempt: usize = 0;
        loop {
            let result: Result<()> = send_message_to_player(
                self.get_player_sender(player_id)?,
                message.clone(),
                player_id,
            )
            .await;
            if result.is_ok() {
                return Ok(());
            }
            attempt += 1;
            if attempt > max_retries {
                return self
                    .end_game(format!(
                        "Player {player_name} failed after {attempt} attempts"
                    ))
                    .await;
            }
            tokio::time::sleep(Duration::from_millis(100 * (1 << attempt))).await;
            let mut rec: Vec<(PlayerId, String)> = vec![(player_id, player_name.clone())];
            let _ = self.handle_player_reconnection(&mut rec).await;
            if !rec.is_empty() {
                return self
                    .end_game(format!("Player {player_name} was disconnected"))
                    .await;
            }
        }
    }

    async fn handle_player_reconnection(
        &mut self,
        players_to_reconnect: &mut Vec<(PlayerId, String)>,
    ) -> Result<()> {
        if players_to_reconnect.is_empty() {
            return Ok(());
        }
        let reconnection_result: Result<Result<()>, tokio::time::error::Elapsed> =
            tokio::time::timeout(self.get_player_reconnection_timeout(), async {
                while !players_to_reconnect.is_empty() {
                    match self.get_reconnection_receiver()?.recv().await {
                        Some((reconnecting_player_id, mut stream)) => {
                            if let Some(pos) = players_to_reconnect
                                .iter()
                                .position(|(id, _)| *id == reconnecting_player_id)
                            {
                                match self
                                    .reconnect_disconnected_player(reconnecting_player_id, stream)
                                    .await
                                {
                                    Ok(_) => {
                                        players_to_reconnect.remove(pos);
                                        self.send_player_full_state(reconnecting_player_id).await?;
                                    }
                                    Err(e) => {
                                        eprintln!(
                                            "Failed to reconnect player {reconnecting_player_id}: {e}"
                                        );
                                    }
                                }
                            } else if self.get_player(reconnecting_player_id).is_ok() {
                                match self.reconnect_disconnected_player(reconnecting_player_id, stream).await {
                                    Ok(()) => {
                                        self.send_player_full_state(reconnecting_player_id).await?;
                                    }
                                    Err(err) => {
                                        eprintln!(
                                            "Failed to reconnect existing player {reconnecting_player_id}: {err}"
                                        );
                                    }
                                };
                            } else {
                                let _ = close_connection(&mut stream).await;
                            }
                        }
                        None => break,
                    }
                }
                Ok::<(), Error>(())
            })
            .await;
        match reconnection_result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(e),
            Err(_) => {
                let remaining_players: Vec<String> = players_to_reconnect
                    .iter()
                    .map(|(_, name)| name.clone())
                    .collect();
                self.end_game(format!(
                    "Players [{}] failed to reconnect within {:?}",
                    remaining_players.join(", "),
                    self.get_player_reconnection_timeout()
                ))
                .await
            }
        }
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
                                let success = crate::network::send_message(&mut writer, &message)
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

    async fn _broadcast_message(
        &mut self,
        message: BroadcastMessage,
    ) -> Result<Vec<(PlayerId, String)>> {
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
                        return Some((player_id, player_name));
                    }
                    None
                }
            })
            .collect();
        let results: Vec<Option<(PlayerId, String)>> =
            futures::future::join_all(send_futures).await;
        let failed_players: Vec<(PlayerId, String)> = results.into_iter().flatten().collect();
        Ok(failed_players)
    }

    async fn broadcast_message(&mut self, message: BroadcastMessage) -> Result<()> {
        let max_retries: usize = self.get_player_reconnection_max_retires();
        let mut attempt: usize = 0;
        loop {
            let mut failed_players: Vec<(PlayerId, String)> =
                self._broadcast_message(message.clone()).await?;
            if failed_players.is_empty() {
                return Ok(());
            }
            attempt += 1;
            if attempt > max_retries {
                return self
                    .end_game(format!(
                        "Players [{}] failed to reconnect after {} attempts",
                        failed_players
                            .iter()
                            .map(|(_, name)| name.clone())
                            .join(", "),
                        attempt
                    ))
                    .await;
            }
            let _ = self.handle_player_reconnection(&mut failed_players).await;
            if failed_players.is_empty() {
                continue;
            } else {
                return self
                    .end_game(format!(
                        "Players [{}] failed to reconnect within 1 minute",
                        failed_players
                            .iter()
                            .map(|(_, name)| name.clone())
                            .join(", ")
                    ))
                    .await;
            }
        }
    }

    async fn close_player_connection(&mut self, player_id: PlayerId) -> Result<()> {
        if let Some(connection) = self.remove_player_connection(player_id) {
            let _ = connection.reader_shutdown_tx.send(());
            let _ = connection.writer_shutdown_tx.send(());
            match (
                timeout(Duration::from_secs(5), connection.reader_handle).await,
                timeout(Duration::from_secs(5), connection.writer_handle).await,
            ) {
                (Ok(Ok(reader)), Ok(Ok(writer))) => {
                    if let Err(err) = reader.unsplit(writer).shutdown().await {
                        println!("Error shutting down stream for player {player_id:?}: {err:?}");
                    }
                }
                _ => {
                    println!("Timeout or error closing connection for player {player_id:?}");
                }
            }
        }
        self.remove_player_channels(player_id);
        Ok(())
    }

    async fn start_game(&mut self) -> Result<()> {
        self.setup_teams().await?;
        self.broadcast_message(BroadcastMessage::GameStarting)
            .await?;
        self.run_game().await
    }

    async fn end_game(&mut self, reason: String) -> Result<()> {
        let _ = self
            ._broadcast_message(BroadcastMessage::GameCancelled { reason })
            .await;
        for player_id in self.get_player_ids() {
            let _ = self.close_player_connection(player_id).await;
        }
        self.clean_up();
        self.set_status(GameStatus::Ended);
        Err(Error::Other("Game ended".to_string()))
    }
}
