use crate::{
    core::Game,
    games::*,
    get_player, get_player_mut, get_team, get_team_mut,
    models::*,
    network::{receive_message, send_message},
    prelude::*,
};

const NUMBER_OF_PLAYERS: usize = 4;
const TARGET_SCORE: usize = 104;
const HIGHEST_BET: usize = 13;
const TEAM_SIZE: usize = 2;
const NUMBER_OF_TEAMS: usize = NUMBER_OF_PLAYERS / TEAM_SIZE;

#[async_trait]
impl Game for Qafoon {
    fn get_players(&mut self) -> Vec<&mut Player> {
        self.players.values_mut().collect()
    }

    fn add_player(&mut self, name: String, team_id: TeamId, connection: Stream) -> Result<()> {
        if self.is_full() {
            return Err(Error::Other("Game is Full".to_owned()));
        }
        let player: Player = Player::new(name, team_id, connection);
        get_team_mut!(self.teams, team_id).players.push(player.id);
        self.players.insert(player.id, player);
        Ok(())
    }

    fn get_player_count(&self) -> usize {
        self.players.len()
    }

    fn is_full(&self) -> bool {
        self.get_player_count() >= NUMBER_OF_PLAYERS
    }

    fn get_status(&self) -> &GameStatus {
        &self.status
    }

    fn initialize_game(&mut self) -> Result<()> {
        if self.get_status() == &GameStatus::Started {
            return Err(Error::Other("Game Already Started".to_owned()));
        }
        self.generate_teams()?;
        self.generate_cards()?;
        Ok(())
    }

    async fn handle_user(&mut self, mut connection: Stream, name: String) -> Result<()> {
        let mut error: String = String::new();
        loop {
            let available_teams: Vec<(TeamId, String)> = self.get_available_team()?;
            let message: GameMessage = GameMessage::team(
                available_teams
                    .iter()
                    .map(|(_, team_name)| team_name.clone())
                    .collect(),
                error.clone(),
            );
            send_message(&mut connection, &message).await?;
            let response: GameMessage = receive_message(&mut connection).await?;
            match response {
                GameMessage::PlayerChoice { choice } => {
                    let team_option: Option<&(TeamId, String)> = available_teams
                        .iter()
                        .find(|(_, team_name)| *team_name == choice);
                    if let Some((team_id, _)) = team_option {
                        self.add_player(name, *team_id, connection)?;
                        return Ok(());
                    } else {
                        error = INVALID_RESPONSE.to_owned()
                    }
                }
                _ => error = INVALID_RESPONSE.to_owned(),
            }
        }
    }

    fn generate_cards(&mut self) -> Result<()> {
        TYPES.iter().for_each(|type_: &Hokm| {
            (0..NUMBERS.len()).for_each(|i: usize| {
                self.cards
                    .push(Card::new(type_.to_owned(), NUMBERS[i].to_owned(), i))
            })
        });
        Ok(())
    }

    async fn run_game(&mut self) -> Result<()> {
        self.set_status(GameStatus::Started);
        self.generate_field()?;
        shuffle(&mut self.cards, ShuffleMethod::Hard);
        while self.should_continue_game()? {
            self.broadcast_message(BroadcastMessage::GameScore {
                teams_score: self.get_teams_game_score(),
            })
            .await?;
            self.broadcast_message(BroadcastMessage::ShufflingCards)
                .await?;
            shuffle(&mut self.cards, ShuffleMethod::Riffle);
            let ground_cards: Vec<Card> = self.cards.drain(0..4).collect();
            self.hand_out_cards().await?;
            let (highest_bet, highest_bettor_id, off_team_id) =
                self.start_betting(ground_cards).await?;
            self.set_starter(highest_bettor_id, highest_bet).await?;
            let mut round_starter_id: PlayerId = self.starter;
            self.fold_first(highest_bettor_id).await?;
            self.set_hokm(highest_bettor_id, highest_bet).await?;
            let def_team_id: TeamId = self.get_opposing_team_id(off_team_id)?;
            while self.should_continue_round(off_team_id, def_team_id, highest_bet)? {
                self.broadcast_message(BroadcastMessage::RoundScore {
                    teams_score: self.get_teams_round_score(),
                })
                .await?;
                let round_starter_index: usize = self
                    .field
                    .iter()
                    .find_position(|player_id: &&PlayerId| **player_id == round_starter_id)
                    .map(|(index, _)| index)
                    .ok_or(Error::player_not_found(round_starter_id))?;
                self.play_card(round_starter_id).await?;
                for index in 1..NUMBER_OF_PLAYERS {
                    self.broadcast_message(BroadcastMessage::GroundCards {
                        ground_cards: self.get_ground_cards()?,
                    })
                    .await?;
                    let player_to_play_id: PlayerId =
                        self.field[(round_starter_index + index) % NUMBER_OF_PLAYERS];
                    self.play_card(player_to_play_id).await?;
                }
                round_starter_id = self.get_hand_collector_id()?;
                self.collect_hand(round_starter_id)?;
            }
            self.finish_round(off_team_id, def_team_id, highest_bet)
                .await?;
            self.prepare_next_round()?;
        }
        self.finish_game().await
    }
}

impl Qafoon {
    pub fn new() -> Self {
        Qafoon::default()
    }

    fn set_status(&mut self, status: GameStatus) {
        self.status = status;
    }

    fn get_available_team(&self) -> Result<Vec<(TeamId, String)>> {
        self.teams
            .values()
            .filter(|team: &&Team| team.players.len() < TEAM_SIZE)
            .sorted_by_key(ToString::to_string)
            .map(|team: &Team| Ok((team.id, team.name.to_owned())))
            .collect()
    }

    fn generate_teams(&mut self) -> Result<()> {
        (0..NUMBER_OF_TEAMS).for_each(|i: usize| {
            let team: Team = Team::new(format!("Team {}", i + 1));
            self.teams.insert(team.id, team);
        });
        Ok(())
    }

    fn generate_field(&mut self) -> Result<()> {
        let teams: Vec<&Team> = self.teams.values().collect();
        (0..TEAM_SIZE).for_each(|j: usize| {
            (0..NUMBER_OF_TEAMS).for_each(|i: usize| self.field.push(teams[i].players[j]))
        });
        Ok(())
    }

    fn get_ground_cards(&self) -> Result<Vec<(String, String)>> {
        self.ground
            .cards
            .iter()
            .map(|(player_id, card)| {
                Ok((
                    get_player!(self.players, *player_id).name.clone(),
                    card.code(),
                ))
            })
            .collect()
    }

    async fn hand_out_cards(&mut self) -> Result<()> {
        self.broadcast_message(BroadcastMessage::HandingOutCards)
            .await?;
        let cards_per_player: usize = self.cards.len() / NUMBER_OF_PLAYERS;
        for (i, player_id) in self.field.iter().enumerate() {
            let player: &mut Player = get_player_mut!(self.players, *player_id);
            let player_cards: Vec<Card> =
                self.cards[i * cards_per_player..(i + 1) * cards_per_player].to_vec();
            player.set_cards(player_cards)?;
            player
                .send_message(&GameMessage::Cards {
                    player_cards: code_cards(&player.hand),
                })
                .await?;
        }
        Ok(())
    }

    async fn set_starter(&mut self, bettor_id: PlayerId, bet: usize) -> Result<()> {
        if self.starter.is_nil() || bet == HIGHEST_BET {
            self.starter = bettor_id;
        } else {
            let team_with_highest_score_id: TeamId = self
                .teams
                .values()
                .max_by_key(|team: &&Team| team.score)
                .map(|team: &Team| team.id)
                .ok_or(Error::Other(
                    "team with highest score was not found".to_owned(),
                ))?;
            let starter_team_id: PlayerId = get_player!(self.players, self.starter).team_id;
            if starter_team_id != team_with_highest_score_id {
                let index: usize = self
                    .field
                    .iter()
                    .find_position(|player_id: &&PlayerId| **player_id == self.starter)
                    .map(|(index, _)| index)
                    .ok_or(Error::player_not_found(self.starter))?;
                self.starter = self.field[(index + 1) % self.field.len()];
            }
        }
        self.broadcast_message(BroadcastMessage::Starter {
            name: get_player!(self.players, self.starter).name.clone(),
        })
        .await?;
        Ok(())
    }

    async fn fold_first(&mut self, player_id: PlayerId) -> Result<()> {
        let team_id: TeamId = get_player!(self.players, player_id).team_id;
        let mut folded_cards: Vec<Card> = Vec::new();
        let mut message: GameMessage = GameMessage::demand(DemandMessage::Fold);
        loop {
            let player: &mut Player = get_player_mut!(self.players, player_id);
            if player.hand.len() <= 12 {
                break;
            }
            if let PlayerChoice::CardChoice(player_choice) =
                get_player_choice(player, &mut message, false, 0).await?
            {
                player.remove_card(&player_choice).ok();
                let card_code: String = player_choice.code();
                folded_cards.push(player_choice);
                player
                    .send_message(&GameMessage::RemoveCard { card: card_code })
                    .await?;
            }
        }
        get_team_mut!(self.teams, team_id)
            .collected_hands
            .push(folded_cards);
        Ok(())
    }

    async fn set_hokm(&mut self, player_id: PlayerId, bet: usize) -> Result<()> {
        let hokms: &[Hokm] = if bet == HIGHEST_BET { &HOKMS } else { &TYPES };
        let player: &mut Player = get_player_mut!(self.players, player_id);
        let mut message: GameMessage = GameMessage::demand(DemandMessage::Hokm);
        loop {
            if let PlayerChoice::HokmChoice(player_choice) =
                get_player_choice(player, &mut message, false, hokms.len() - 1).await?
            {
                if hokms.contains(&player_choice) {
                    self.hokm = player_choice;
                    self.broadcast_message(BroadcastMessage::Hokm {
                        hokm: self.hokm.code(),
                    })
                    .await?;
                    return Ok(());
                }
                message.set_demand_error(INVALID_RESPONSE.to_owned());
            }
        }
    }

    fn get_hand_collector_id(&self) -> Result<PlayerId> {
        let winner_id: Option<&(PlayerId, Card)> = match self.hokm {
            Hokm::Naras => self
                .ground
                .cards
                .iter()
                .filter(|(_, card)| card.type_ == self.ground.type_)
                .min_by_key(|(_, card)| card.ord),
            Hokm::Saras => self
                .ground
                .cards
                .iter()
                .filter(|(_, card)| card.type_ == self.ground.type_)
                .max_by_key(|(_, card)| card.ord),
            Hokm::TakNaras => self
                .ground
                .cards
                .iter()
                .filter(|(_, card)| card.type_ == self.ground.type_)
                .min_by(|(_, card1), (_, card2)| {
                    if card1.ord == 12 {
                        std::cmp::Ordering::Less
                    } else if card2.ord == 12 {
                        std::cmp::Ordering::Greater
                    } else {
                        card1.ord.cmp(&card2.ord)
                    }
                }),
            _ => {
                let hokm_winner: Option<&(PlayerId, Card)> = self
                    .ground
                    .cards
                    .iter()
                    .filter(|(_, card)| card.type_ == self.hokm)
                    .max_by_key(|(_, card)| card.ord);
                match hokm_winner {
                    Some(_) => hokm_winner,
                    None => self
                        .ground
                        .cards
                        .iter()
                        .filter(|(_, card)| card.type_ == self.ground.type_)
                        .max_by_key(|(_, card)| card.ord),
                }
            }
        };
        winner_id
            .map(|(player_id, _)| *player_id)
            .ok_or(Error::NoValidCard)
    }

    fn collect_hand(&mut self, player_to_collect_id: PlayerId) -> Result<()> {
        let team_to_collect_id: TeamId = get_player!(self.players, player_to_collect_id).team_id;
        let ground_cards: Vec<Card> = self.ground.cards.drain(..).map(|(_, card)| card).collect();
        get_team_mut!(self.teams, team_to_collect_id)
            .collected_hands
            .push(ground_cards);
        Ok(())
    }

    async fn start_betting(
        &mut self,
        ground_cards: Vec<Card>,
    ) -> Result<(usize, PlayerId, TeamId)> {
        let mut highest_bet_option: Option<usize> = None;
        let mut highest_bettor_id: PlayerId = PlayerId::nil();
        let mut bets: Vec<(String, PlayerChoice)> = Vec::new();
        loop {
            for player_id in self.field.clone().into_iter() {
                let mut message: GameMessage = GameMessage::demand(DemandMessage::Bet);
                let player: &mut Player = get_player_mut!(self.players, player_id);
                let player_choice: PlayerChoice =
                    get_player_choice(player, &mut message, true, HIGHEST_BET).await?;
                bets.push((player.name.clone(), player_choice.clone()));
                if let PlayerChoice::NumberChoice(choice) = player_choice {
                    if highest_bet_option.is_none_or(|highest_bet: usize| choice > highest_bet) {
                        highest_bet_option = Some(choice);
                        highest_bettor_id = player_id;
                        if choice == HIGHEST_BET {
                            break;
                        }
                    }
                }
                self.broadcast_message(BroadcastMessage::Bets { bets: bets.clone() })
                    .await?;
            }
            if let Some(highest_bet) = highest_bet_option {
                let (name, team_id) = {
                    let highest_bettor: &mut Player =
                        get_player_mut!(self.players, highest_bettor_id);
                    highest_bettor.add_cards(ground_cards.clone())?;
                    highest_bettor
                        .send_message(&GameMessage::AddGroundCards {
                            ground_cards: code_cards(&ground_cards),
                        })
                        .await?;
                    (highest_bettor.name.to_owned(), highest_bettor.team_id)
                };
                self.broadcast_message(BroadcastMessage::BetWinner {
                    bet_winner: (name, highest_bet),
                })
                .await?;
                return Ok((highest_bet, highest_bettor_id, team_id));
            }
        }
    }

    async fn finish_round(
        &mut self,
        off_team_id: TeamId,
        def_team_id: TeamId,
        bet: usize,
    ) -> Result<()> {
        let off_team: &mut Team = get_team_mut!(self.teams, off_team_id);
        let round_winner: String = if off_team.collected_hands.len() == bet {
            off_team.score += if bet == HIGHEST_BET { bet * 2 } else { bet };
            off_team
        } else {
            let def_team: &mut Team = get_team_mut!(self.teams, def_team_id);
            def_team.score += bet * 2;
            def_team
        }
        .to_string();
        self.broadcast_message(BroadcastMessage::RoundWinner { round_winner })
            .await
    }

    fn prepare_next_round(&mut self) -> Result<()> {
        self.teams.values_mut().for_each(|team: &mut Team| {
            team.collected_hands
                .drain(..)
                .for_each(|hand: Vec<Card>| self.cards.extend(hand));
        });
        self.players
            .values_mut()
            .for_each(|player: &mut Player| self.cards.append(&mut player.hand));
        Ok(())
    }

    fn should_continue_round(
        &self,
        off_team_id: TeamId,
        def_team_id: TeamId,
        bet: usize,
    ) -> Result<bool> {
        let off_team: &Team = get_team!(self.teams, off_team_id);
        let def_team: &Team = get_team!(self.teams, def_team_id);
        Ok(off_team.collected_hands.len() < bet && def_team.collected_hands.len() < (14 - bet))
    }

    fn should_continue_game(&self) -> Result<bool> {
        Ok(self
            .teams
            .values()
            .all(|team: &Team| team.score < TARGET_SCORE))
    }

    async fn finish_game(&mut self) -> Result<()> {
        let game_winner: &String = &self
            .teams
            .values()
            .find(|team: &&Team| team.score >= TARGET_SCORE)
            .map(ToString::to_string)
            .ok_or(Error::Other(
                "Team with required score was not found".to_string(),
            ))?;
        self.broadcast_message(BroadcastMessage::GameWinner {
            game_winner: game_winner.to_string(),
        })
        .await?;
        for player in self.players.values_mut() {
            player.close_connection().await?;
        }
        self.set_status(GameStatus::Finished);
        Ok(())
    }

    fn get_opposing_team_id(&self, team_id: TeamId) -> Result<TeamId> {
        Ok(*self
            .teams
            .keys()
            .find(|opposing_team_id: &&TeamId| **opposing_team_id != team_id)
            .ok_or(Error::Other("Opposing team ID not found".to_owned()))?)
    }

    fn get_teams_game_score(&self) -> Vec<(String, usize)> {
        self.teams
            .values()
            .sorted_by_key(ToString::to_string)
            .map(|team: &Team| (team.name.clone(), team.score))
            .collect()
    }

    fn get_teams_round_score(&self) -> Vec<(String, usize)> {
        self.teams
            .values()
            .sorted_by_key(ToString::to_string)
            .map(|team: &Team| (team.name.clone(), team.collected_hands.len()))
            .collect()
    }

    async fn play_card(&mut self, player_id: PlayerId) -> Result<()> {
        let is_round_starter: bool = self.ground.cards.is_empty();
        let mut message: GameMessage = GameMessage::demand(DemandMessage::PlayCard);
        loop {
            if let PlayerChoice::CardChoice(player_choice) = get_player_choice(
                get_player_mut!(self.players, player_id),
                &mut message,
                false,
                0,
            )
            .await?
            {
                if !is_round_starter {
                    let has_matching_card: bool = get_player!(self.players, player_id)
                        .hand
                        .iter()
                        .any(|player_card: &Card| player_card.type_ == self.ground.type_);
                    if has_matching_card && player_choice.type_ != self.ground.type_ {
                        message
                            .set_demand_error(format!("You have {}!\n", self.ground.type_.name()));
                        continue;
                    }
                }
                let player: &mut Player = get_player_mut!(self.players, player_id);
                player.remove_card(&player_choice).ok();
                let card_code: String = player_choice.code();
                self.ground.add_card(player_id, player_choice)?;
                return player
                    .send_message(&GameMessage::RemoveCard { card: card_code })
                    .await;
            }
        }
    }
}
