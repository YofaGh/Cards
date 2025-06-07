use itertools::Itertools;
use rand::{rngs::ThreadRng, seq::SliceRandom, Rng};
use std::collections::BTreeMap;

use crate::{
    constants::*, enums::PlayerChoice, get_player, get_player_mut, get_team, get_team_mut,
    models::*, prelude::*,
};

const NUMBER_OF_PLAYERS: usize = 4;
const TARGET_SCORE: usize = 104;
const HIGHEST_BET: usize = 13;
const TEAM_SIZE: usize = 2;
const NUMBER_OF_TEAMS: usize = NUMBER_OF_PLAYERS / TEAM_SIZE;

pub struct Game {
    teams: BTreeMap<TeamId, Team>,
    players: BTreeMap<PlayerId, Player>,
    field: Vec<PlayerId>,
    cards: Vec<Card>,
    starter: PlayerId,
    hokm: Hokm,
    max_players: usize,
    target_score: usize,
    pub started: bool,
    pub finished: bool,
}

impl Game {
    pub fn new() -> Self {
        Self {
            teams: BTreeMap::new(),
            players: BTreeMap::new(),
            field: Vec::new(),
            cards: Vec::new(),
            starter: PlayerId::nil(),
            hokm: Hokm::default(),
            max_players: NUMBER_OF_PLAYERS,
            target_score: TARGET_SCORE,
            started: false,
            finished: false,
        }
    }

    pub fn add_player(
        &mut self,
        name: String,
        team_id: TeamId,
        connection: TcpStream,
    ) -> Result<()> {
        if self.get_player_count()? >= self.max_players {
            return Err(Error::Other("Game is Full".to_owned()));
        }
        let player: Player = Player::new(name, team_id, connection);
        get_team_mut!(self.teams, team_id).players.push(player.id);
        self.players.insert(player.id, player);
        Ok(())
    }

    pub fn get_player_count(&self) -> Result<usize> {
        Ok(self.players.len())
    }

    pub fn is_full(&self) -> Result<bool> {
        Ok(self.get_player_count()? >= self.max_players)
    }

    pub fn initialize_game(&mut self) -> Result<()> {
        if self.started {
            return Err(Error::Other("Game Already Started".to_owned()));
        }
        self.generate_teams()?;
        self.generate_cards()?;
        self.started = true;
        Ok(())
    }

    pub fn get_available_team(&self) -> Result<Vec<(TeamId, String)>> {
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

    fn generate_cards(&mut self) -> Result<()> {
        TYPES.iter().for_each(|type_: &Hokm| {
            (0..NUMBERS.len()).for_each(|i: usize| {
                self.cards
                    .push(Card::new(type_.to_owned(), NUMBERS[i].to_owned(), i))
            })
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

    pub fn broadcast_message(&self, message: &str) -> Result<()> {
        self.players
            .values()
            .try_for_each(|player: &Player| player.send_message(message, 0))
    }

    pub fn shuffle_cards(&mut self, hard_shuffle: bool) -> Result<()> {
        let mut rng: ThreadRng = rand::rng();
        if hard_shuffle {
            return {
                self.cards.shuffle(&mut rng);
                Ok(())
            };
        }
        self.broadcast_message("Shuffling cards...")?;
        let random_time: usize = rng.random_range(1..=3);
        (0..random_time).for_each(|_| {
            let start: usize = rng.random_range(0..self.cards.len());
            let end: usize = rng.random_range(0..self.cards.len());
            let (start, end) = if end < start {
                (end, start)
            } else {
                (start, end)
            };
            let mut new_cards: Vec<Card> = Vec::with_capacity(self.cards.len());
            new_cards.extend_from_slice(&self.cards[start..end]);
            new_cards.extend_from_slice(&self.cards[..start]);
            new_cards.extend_from_slice(&self.cards[end..]);
            self.cards = new_cards;
        });
        Ok(())
    }

    pub fn hand_out_cards(&mut self) -> Result<()> {
        self.broadcast_message("Handing out cards...")?;
        let cards_per_player: usize = self.cards.len() / NUMBER_OF_PLAYERS;
        self.field
            .iter()
            .enumerate()
            .try_for_each(|(i, player_id)| -> Result<()> {
                get_player_mut!(self.players, *player_id).set_cards(
                    self.cards[i * cards_per_player..(i + 1) * cards_per_player].to_vec(),
                )
            })
    }

    pub fn set_starter(&mut self, bettor_id: PlayerId, bet: usize) -> Result<PlayerId> {
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
        self.broadcast_message(&format!(
            "Starter: {}",
            get_player!(self.players, self.starter).name
        ))?;
        Ok(self.starter)
    }

    pub fn fold_first(&mut self, player_id: PlayerId) -> Result<()> {
        let team_id: TeamId = get_player!(self.players, player_id).team_id;
        let mut folded_cards: Vec<Card> = Vec::new();
        loop {
            let (hand_len, prompt) = {
                let player: &Player = get_player!(self.players, player_id);
                let hand_len: usize = player.hand.len();
                if hand_len <= 12 {
                    break;
                }
                let prompt: String = format!("{}\nChoose a card to fold", player.get_hand());
                (hand_len, prompt)
            };
            if let PlayerChoice::Choice(player_choice) = self.get_player_choice(
                get_player!(self.players, player_id),
                &prompt,
                false,
                hand_len - 1,
            )? {
                let folded_card: Card = get_player_mut!(self.players, player_id)
                    .hand
                    .remove(player_choice);
                folded_cards.push(folded_card);
            }
        }
        get_team_mut!(self.teams, team_id)
            .collected_hands
            .push(folded_cards);
        Ok(())
    }

    pub fn set_hokm(&mut self, player_id: PlayerId, bet: usize) -> Result<()> {
        let hokms: &[Hokm] = if bet == HIGHEST_BET { &HOKMS } else { &TYPES };
        let hokms_str: String = hokms
            .iter()
            .enumerate()
            .map(|(index, hokm)| format!("{}:{index}", hokm))
            .join(", ");
        let player: &Player = get_player!(self.players, player_id);
        let mut pre: &str = "";
        loop {
            let prompt: String = format!("{pre}{} what is your hokm? {hokms_str}", player.name);
            if let PlayerChoice::Choice(player_choice) =
                self.get_player_choice(player, &prompt, false, hokms.len() - 1)?
            {
                if player_choice < hokms.len() {
                    self.hokm = hokms[player_choice].to_owned();
                    self.broadcast_message(&format!("Hokm: {}", hokms[player_choice]))?;
                    return Ok(());
                }
                pre = INVALID_RESPONSE;
            }
        }
    }

    pub fn get_hand_collector_id(&self, ground: &Ground) -> Result<PlayerId> {
        let winner_id: Option<&(PlayerId, Card)> = match self.hokm {
            NARAS => ground
                .cards
                .iter()
                .filter(|(_, card)| card.type_ == ground.type_)
                .min_by_key(|(_, card)| card.ord),
            SARAS => ground
                .cards
                .iter()
                .filter(|(_, card)| card.type_ == ground.type_)
                .max_by_key(|(_, card)| card.ord),
            TAK_NARAS => ground
                .cards
                .iter()
                .filter(|(_, card)| card.type_ == ground.type_)
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
                let hokm_winner: Option<&(PlayerId, Card)> = ground
                    .cards
                    .iter()
                    .filter(|(_, card)| card.type_ == self.hokm)
                    .max_by_key(|(_, card)| card.ord);
                match hokm_winner {
                    Some(_) => hokm_winner,
                    None => ground
                        .cards
                        .iter()
                        .filter(|(_, card)| card.type_ == ground.type_)
                        .max_by_key(|(_, card)| card.ord),
                }
            }
        };
        winner_id
            .map(|(player_id, _)| *player_id)
            .ok_or(Error::NoValidCard)
    }

    pub fn collect_hand(&mut self, player_to_collect_id: PlayerId, ground: Ground) -> Result<()> {
        let team_to_collect_id: TeamId = get_player!(self.players, player_to_collect_id).team_id;
        get_team_mut!(self.teams, team_to_collect_id)
            .collected_hands
            .push(ground.cards.into_iter().map(|(_, card)| card).collect());
        Ok(())
    }

    pub fn start_betting(&mut self, ground_cards: Vec<Card>) -> Result<(usize, PlayerId, TeamId)> {
        let mut highest_bet_option: Option<usize> = None;
        let mut highest_bettor_id: PlayerId = PlayerId::nil();
        let mut others_bets: Vec<String> = Vec::new();
        loop {
            for player_id in self.field.iter() {
                let player: &Player = get_player!(self.players, *player_id);
                let player_hand: String = player.hand.iter().map(ToString::to_string).join(", ");
                let prompt: String =
                    format!("These are your cards: {player_hand}\nWhat is your bet?");
                match self.get_player_choice(player, &prompt, true, HIGHEST_BET)? {
                    PlayerChoice::Choice(player_choice) => {
                        if highest_bet_option
                            .is_none_or(|highest_bet: usize| player_choice > highest_bet)
                        {
                            highest_bet_option = Some(player_choice);
                            highest_bettor_id = *player_id;
                            others_bets.push(format!("{}: {player_choice}", player.name));
                            if player_choice == HIGHEST_BET {
                                break;
                            }
                        }
                    }
                    PlayerChoice::Pass => others_bets.push(format!("{}: pass", player.name)),
                }
                self.broadcast_message(&others_bets.join(", "))?;
            }
            if let Some(highest_bet) = highest_bet_option {
                let (name, team_id) = {
                    let highest_bettor: &mut Player =
                        get_player_mut!(self.players, highest_bettor_id);
                    highest_bettor.add_cards(ground_cards)?;
                    (highest_bettor.name.to_owned(), highest_bettor.team_id)
                };
                self.broadcast_message(&format!("{name} wins with {highest_bet}!"))?;
                return Ok((highest_bet, highest_bettor_id, team_id));
            }
        }
    }

    pub fn start_round(&mut self, ground: &mut Ground, round_starter_id: &PlayerId) -> Result<()> {
        let (prompt, hand_len, player_id) = {
            let player = get_player!(self.players, *round_starter_id);
            let prompt = format!(
                "{}: {}\nChoose a card to play:",
                player.name,
                player.get_hand()
            );
            (prompt, player.hand.len(), player.id)
        };
        if let PlayerChoice::Choice(player_choice) = self.get_player_choice(
            get_player!(self.players, *round_starter_id),
            &prompt,
            false,
            hand_len - 1,
        )? {
            let card: Card = get_player_mut!(self.players, *round_starter_id)
                .hand
                .remove(player_choice);
            ground.add_card(player_id, card)?;
        }
        Ok(())
    }

    pub fn continue_round(&mut self, ground: &mut Ground, index: usize) -> Result<()> {
        let ground_cards: String = {
            ground
                .cards
                .iter()
                .map(|(player_id, card)| {
                    Ok(format!(
                        "{}:{}",
                        get_player!(self.players, *player_id).name,
                        card
                    ))
                })
                .collect::<Result<Vec<String>, Error>>()?
                .join(", ")
        };
        self.broadcast_message(&ground_cards)?;
        let player_to_play_id: PlayerId = self.field[index % NUMBER_OF_PLAYERS];
        let mut pre: String = String::new();
        loop {
            let (player_hand, hand_len, player_id) = {
                let player: &Player = get_player!(self.players, player_to_play_id);
                (player.get_hand(), player.hand.len(), player.id)
            };
            if let PlayerChoice::Choice(player_choice) = self.get_player_choice(
                get_player!(self.players, player_to_play_id),
                &format!("{pre}\n{player_hand}\nChoose a card to play:"),
                false,
                hand_len - 1,
            )? {
                let (has_matching_card, selected_card_type) = {
                    let player: &Player = get_player!(self.players, player_to_play_id);
                    let has_matching: bool = player
                        .hand
                        .iter()
                        .any(|player_card: &Card| player_card.type_ == ground.type_);
                    let selected_type: Hokm = player.hand[player_choice].type_.clone();
                    (has_matching, selected_type)
                };
                if has_matching_card && selected_card_type != ground.type_ {
                    pre = format!("You have {}!\n", ground.type_.name);
                    continue;
                }
                let card: Card = get_player_mut!(self.players, player_to_play_id)
                    .hand
                    .remove(player_choice);
                return ground.add_card(player_id, card);
            }
        }
    }

    pub fn finish_round(
        &mut self,
        off_team_id: TeamId,
        def_team_id: TeamId,
        bet: usize,
    ) -> Result<()> {
        let off_team: &mut Team = get_team_mut!(self.teams, off_team_id);
        let winner_team: String = if off_team.collected_hands.len() == bet {
            off_team.score += if bet == HIGHEST_BET { bet * 2 } else { bet };
            off_team
        } else {
            let def_team: &mut Team = get_team_mut!(self.teams, def_team_id);
            def_team.score += bet * 2;
            def_team
        }
        .to_string();
        self.broadcast_message(&format!("Winner of this round is: {}", winner_team))
    }

    pub fn prepare_next_round(&mut self) -> Result<()> {
        self.teams.values_mut().for_each(|team: &mut Team| {
            team.collected_hands
                .drain(..)
                .for_each(|hand: Vec<Card>| self.cards.extend(hand));
        });
        self.players.values_mut().for_each(|player: &mut Player| {
            self.cards.extend(player.hand.drain(..));
        });
        Ok(())
    }

    pub fn should_continue_round(
        &self,
        off_team_id: TeamId,
        def_team_id: TeamId,
        bet: usize,
    ) -> Result<bool> {
        let off_team: &Team = get_team!(self.teams, off_team_id);
        let def_team: &Team = get_team!(self.teams, def_team_id);
        Ok(off_team.collected_hands.len() < bet && def_team.collected_hands.len() < (14 - bet))
    }

    pub fn should_continue_game(&self) -> Result<bool> {
        Ok(self
            .teams
            .values()
            .all(|team: &Team| team.score < self.target_score))
    }

    pub fn finish_game(&mut self) -> Result<()> {
        let winner_team: &str = &self
            .teams
            .values()
            .find(|team: &&Team| team.score >= self.target_score)
            .map(ToString::to_string)
            .ok_or(Error::Other(
                "Team with required score was not found".to_string(),
            ))?;
        self.broadcast_message(&format!("Winner is {winner_team}"))?;
        self.finished = true;
        Ok(())
    }

    pub fn get_opposing_team_id(&self, team_id: TeamId) -> Result<TeamId> {
        Ok(*self
            .teams
            .keys()
            .find(|opposing_team_id: &&TeamId| **opposing_team_id != team_id)
            .ok_or(Error::Other("Opposing team ID not found".to_owned()))?)
    }

    fn get_player_choice(
        &self,
        player: &Player,
        prompt: &str,
        passable: bool,
        max_value: usize,
    ) -> Result<PlayerChoice> {
        let mut pre: String = String::new();
        loop {
            player.send_message(&format!("{pre}{prompt}"), 1)?;
            let response: String = player.receive_message()?;
            if response == "pass" {
                if passable {
                    return Ok(PlayerChoice::Pass);
                }
                pre = "You can't pass this one".to_owned();
            } else if let Ok(choice) = response.parse::<usize>() {
                if choice <= max_value {
                    return Ok(PlayerChoice::Choice(choice));
                }
                pre = format!("Choice can't be greater than {max_value}");
            } else {
                pre = INVALID_RESPONSE.to_owned();
            }
        }
    }

    pub fn run_game(&mut self) -> Result<()> {
        self.generate_field()?;
        self.shuffle_cards(true)?;
        while self.should_continue_game()? {
            self.teams
                .values()
                .sorted_by_key(ToString::to_string)
                .try_for_each(|team: &Team| {
                    self.broadcast_message(&format!("{}: {}", team.name, team.score))
                })?;
            self.shuffle_cards(false)?;
            let ground_cards: Vec<Card> = self.cards.drain(0..4).collect();
            self.hand_out_cards()?;
            let (highest_bet, highest_bettor_id, off_team_id) = self.start_betting(ground_cards)?;
            let mut round_starter_id: PlayerId =
                self.set_starter(highest_bettor_id, highest_bet)?;
            self.fold_first(highest_bettor_id)?;
            self.set_hokm(highest_bettor_id, highest_bet)?;
            let def_team_id: TeamId = self.get_opposing_team_id(off_team_id)?;
            while self.should_continue_round(off_team_id, def_team_id, highest_bet)? {
                self.teams
                    .values()
                    .sorted_by_key(ToString::to_string)
                    .try_for_each(|team: &Team| {
                        self.broadcast_message(&format!(
                            "{}: {}",
                            team.name,
                            team.collected_hands.len()
                        ))
                    })?;
                let round_starter_index: usize = self
                    .field
                    .iter()
                    .find_position(|player_id: &&PlayerId| **player_id == round_starter_id)
                    .map(|(index, _)| index)
                    .ok_or(Error::player_not_found(round_starter_id))?;
                let mut ground: Ground = Ground::new();
                self.start_round(&mut ground, &round_starter_id)?;
                (1..NUMBER_OF_PLAYERS).try_for_each(|index: usize| {
                    self.continue_round(&mut ground, round_starter_index + index)
                })?;
                round_starter_id = self.get_hand_collector_id(&ground)?;
                self.collect_hand(round_starter_id, ground)?;
            }
            self.finish_round(off_team_id, def_team_id, highest_bet)?;
            self.prepare_next_round()?;
        }
        self.finish_game()
    }
}
