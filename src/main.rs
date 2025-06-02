mod constants;
mod enums;
mod errors;
mod macros;
mod models;
mod prelude;
mod tcp_messenger;
mod types;

use itertools::Itertools;
use lazy_static::lazy_static;
use rand::{rngs::ThreadRng, seq::SliceRandom, Rng};
use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use {constants::*, enums::PlayerChoice, models::*, prelude::*};

const NUMBER_OF_PLAYERS: usize = 4;
const TARGET_SCORE: usize = 104;
const HIGHEST_BET: usize = 13;
const TEAM_SIZE: usize = 2;
const NUMBER_OF_TEAMS: usize = NUMBER_OF_PLAYERS / TEAM_SIZE;

lazy_static! {
    static ref TEAMS: Arc<RwLock<BTreeMap<TeamId, Team>>> = create_shared_state(BTreeMap::new());
    static ref FIELD: Arc<RwLock<Vec<PlayerId>>> = create_shared_state(Vec::new());
    static ref CARDS: Arc<RwLock<Vec<Card>>> = create_shared_state(Vec::new());
    static ref STARTER: Arc<RwLock<PlayerId>> = create_shared_state(PlayerId::nil());
    static ref HOKM: Arc<RwLock<Hokm>> = create_shared_state(Hokm::default());
    static ref NUMBER_OF_CLIENTS: Arc<RwLock<usize>> = create_shared_state(0);
    static ref PLAYERS: Arc<RwLock<BTreeMap<PlayerId, Player>>> =
        create_shared_state(BTreeMap::new());
}

fn create_shared_state<T>(initial: T) -> Arc<RwLock<T>> {
    Arc::new(RwLock::new(initial))
}

fn get_read_lock<T>(rwlock: &'static RwLock<T>) -> Result<RwLockReadGuard<'static, T>> {
    rwlock.read().map_err(Error::rw_read)
}

fn get_write_lock<T>(rwlock: &'static RwLock<T>) -> Result<RwLockWriteGuard<'static, T>> {
    rwlock.write().map_err(Error::rw_write)
}

fn get_player_choice(
    player: &Player,
    prompt: &str,
    passable: bool,
    max_value: usize,
) -> Result<PlayerChoice> {
    let mut pre: String = String::new();
    loop {
        player.send_message(&format!("{}{}", pre, prompt), 1)?;
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
            pre = format!("Choice can't be greater than {}", max_value);
        } else {
            pre = INVALID_RESPONSE.to_owned();
        }
    }
}

fn generate_teams() -> Result<()> {
    let mut teams_guard: RwLockWriteGuard<BTreeMap<TeamId, Team>> = get_write_lock(&TEAMS)?;
    (0..NUMBER_OF_TEAMS).for_each(|i: usize| {
        let team: Team = Team::new(format!("Team {}", i + 1));
        teams_guard.insert(team.id, team);
    });
    Ok(())
}

fn generate_cards() -> Result<()> {
    let mut cards_guard: RwLockWriteGuard<Vec<Card>> = get_write_lock(&CARDS)?;
    TYPES.iter().for_each(|type_: &Hokm| {
        (0..NUMBERS.len()).for_each(|i: usize| {
            cards_guard.push(Card::new(type_.to_owned(), NUMBERS[i].to_owned(), i))
        })
    });
    Ok(())
}

fn generate_field() -> Result<()> {
    let mut field_guard: RwLockWriteGuard<Vec<PlayerId>> = get_write_lock(&FIELD)?;
    let teams_guard: RwLockReadGuard<BTreeMap<TeamId, Team>> = get_read_lock(&TEAMS)?;
    let teams: Vec<&Team> = teams_guard.values().collect();
    (0..TEAM_SIZE).for_each(|j: usize| {
        (0..NUMBER_OF_TEAMS).for_each(|i: usize| field_guard.push(teams[i].players[j]))
    });
    Ok(())
}

fn broadcast_message(message: &str) -> Result<()> {
    get_read_lock(&PLAYERS)?
        .values()
        .try_for_each(|player: &Player| player.send_message(message, 0))
}

fn shuffle_cards(hard_shuffle: bool) -> Result<()> {
    let mut rng: ThreadRng = rand::rng();
    let mut cards_guard: RwLockWriteGuard<Vec<Card>> = get_write_lock(&CARDS)?;
    if hard_shuffle {
        return {
            cards_guard.shuffle(&mut rng);
            Ok(())
        };
    }
    broadcast_message("Shuffling cards...")?;
    let random_time: usize = rng.random_range(1..=3);
    (0..random_time).for_each(|_| {
        let start: usize = rng.random_range(0..cards_guard.len());
        let end: usize = rng.random_range(0..cards_guard.len());
        let (start, end) = if end < start {
            (end, start)
        } else {
            (start, end)
        };
        let mut new_cards: Vec<Card> = Vec::with_capacity(cards_guard.len());
        new_cards.extend_from_slice(&cards_guard[start..end]);
        new_cards.extend_from_slice(&cards_guard[..start]);
        new_cards.extend_from_slice(&cards_guard[end..]);
        *cards_guard = new_cards;
    });
    Ok(())
}

fn hand_out_cards() -> Result<()> {
    broadcast_message("Handing out cards...")?;
    let cards_guard: RwLockReadGuard<Vec<Card>> = get_read_lock(&CARDS)?;
    let cards_per_player: usize = cards_guard.len() / NUMBER_OF_PLAYERS;
    let mut players_guard: RwLockWriteGuard<BTreeMap<PlayerId, Player>> = get_write_lock(&PLAYERS)?;
    get_read_lock(&FIELD)?
        .iter()
        .enumerate()
        .try_for_each(|(i, player_id)| -> Result<()> {
            get_player_mut!(players_guard, *player_id)
                .set_cards(cards_guard[i * cards_per_player..(i + 1) * cards_per_player].to_vec())
        })
}

fn set_starter(bettor_id: PlayerId, bet: usize) -> Result<PlayerId> {
    let mut starter_guard: RwLockWriteGuard<PlayerId> = get_write_lock(&STARTER)?;
    if starter_guard.is_nil() || bet == HIGHEST_BET {
        *starter_guard = bettor_id;
    } else {
        let team_with_highest_score_id: TeamId = get_read_lock(&TEAMS)?
            .values()
            .max_by_key(|team: &&Team| team.score)
            .map(|team: &Team| team.id)
            .ok_or_else(|| Error::Other("team with highest score was not found".to_owned()))?;
        let starter_team_id: PlayerId =
            get_player!(get_read_lock(&PLAYERS)?, *starter_guard).team_id;
        if starter_team_id != team_with_highest_score_id {
            let field_guard: RwLockReadGuard<Vec<PlayerId>> = get_read_lock(&FIELD)?;
            let index: usize = field_guard
                .iter()
                .find_position(|player_id: &&PlayerId| **player_id == *starter_guard)
                .map(|(index, _)| index)
                .ok_or_else(|| Error::player_not_found(*starter_guard))?;
            *starter_guard = field_guard[(index + 1) % field_guard.len()];
        }
    }
    broadcast_message(&format!(
        "Starter: {}",
        get_player!(get_read_lock(&PLAYERS)?, *starter_guard).name
    ))?;
    Ok(*starter_guard)
}

fn fold_first(player_id: PlayerId) -> Result<()> {
    let mut players_guard: RwLockWriteGuard<BTreeMap<PlayerId, Player>> = get_write_lock(&PLAYERS)?;
    let player: &mut Player = get_player_mut!(players_guard, player_id);
    let mut folded_cards: Vec<Card> = Vec::new();
    while player.hand.len() > 12 {
        let prompt: String = format!("{}\nChoose a card to fold", player.get_hand());
        if let PlayerChoice::Choice(player_choice) =
            get_player_choice(player, &prompt, false, player.hand.len() - 1)?
        {
            folded_cards.push(player.hand.remove(player_choice));
        }
    }
    get_team_mut!(get_write_lock(&TEAMS)?, player.team_id)
        .collected_hands
        .push(folded_cards);
    Ok(())
}

fn set_hokm(player_id: PlayerId, bet: usize) -> Result<()> {
    let hokms: &[Hokm] = if bet == HIGHEST_BET { &HOKMS } else { &TYPES };
    let hokms_str: String = hokms
        .iter()
        .enumerate()
        .map(|(index, hokm)| format!("{}:{}", hokm, index))
        .join(", ");
    let players_guard: RwLockReadGuard<BTreeMap<PlayerId, Player>> = get_read_lock(&PLAYERS)?;
    let player: &Player = get_player!(players_guard, player_id);
    let mut pre: &'static str = "";
    loop {
        let prompt: String = format!("{}{} what is your hokm? {}", pre, player.name, hokms_str);
        if let PlayerChoice::Choice(player_choice) =
            get_player_choice(player, &prompt, false, hokms.len() - 1)?
        {
            if player_choice < hokms.len() {
                *get_write_lock(&HOKM)? = hokms[player_choice].to_owned();
                broadcast_message(&format!("Hokm: {}", hokms[player_choice]))?;
                return Ok(());
            }
            pre = INVALID_RESPONSE;
        }
    }
}

fn get_hand_collector_id(ground: &Ground) -> Result<PlayerId> {
    let hokm_guard: RwLockReadGuard<Hokm> = get_read_lock(&HOKM)?;
    let winner_id: Option<&(PlayerId, Card)> = match *hokm_guard {
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
                .filter(|(_, card)| card.type_ == *hokm_guard)
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

fn collect_hand(player_to_collect_id: PlayerId, ground: Ground) -> Result<()> {
    let team_to_collect_id: TeamId =
        get_player!(get_read_lock(&PLAYERS)?, player_to_collect_id).team_id;
    get_team_mut!(get_write_lock(&TEAMS)?, team_to_collect_id)
        .collected_hands
        .push(ground.cards.into_iter().map(|(_, card)| card).collect());
    Ok(())
}

fn start_betting(ground_cards: Vec<Card>) -> Result<(usize, PlayerId, TeamId)> {
    let mut highest_bet_option: Option<usize> = None;
    let mut highest_bettor_id: PlayerId = PlayerId::nil();
    let mut others_bets: Vec<String> = Vec::new();
    loop {
        for player_id in get_read_lock(&FIELD)?.iter() {
            let players_guard: RwLockReadGuard<BTreeMap<PlayerId, Player>> =
                get_read_lock(&PLAYERS)?;
            let player: &Player = get_player!(players_guard, *player_id);
            let player_hand: String = player.hand.iter().map(ToString::to_string).join(", ");
            let prompt: String = format!("These are your cards: {player_hand}\nWhat is your bet?");
            match get_player_choice(player, &prompt, true, HIGHEST_BET)? {
                PlayerChoice::Choice(player_choice) => {
                    if highest_bet_option
                        .is_none_or(|highest_bet: usize| player_choice > highest_bet)
                    {
                        highest_bet_option = Some(player_choice);
                        highest_bettor_id = *player_id;
                        others_bets.push(format!("{}: {}", player.name, player_choice));
                        if player_choice == HIGHEST_BET {
                            break;
                        }
                    }
                }
                _ => continue,
            }
            broadcast_message(&others_bets.join(", "))?;
        }
        if let Some(highest_bet) = highest_bet_option {
            let (name, team_id) = {
                let mut players_guard: RwLockWriteGuard<BTreeMap<PlayerId, Player>> =
                    get_write_lock(&PLAYERS)?;
                let highest_bettor: &mut Player = get_player_mut!(players_guard, highest_bettor_id);
                highest_bettor.add_cards(ground_cards)?;
                (highest_bettor.name.to_owned(), highest_bettor.team_id)
            };
            broadcast_message(&format!("{} wins with {}!", name, highest_bet))?;
            return Ok((highest_bet, highest_bettor_id, team_id));
        }
    }
}

fn start_round(ground: &mut Ground, round_starter_id: &PlayerId) -> Result<()> {
    let mut players_guard: RwLockWriteGuard<BTreeMap<PlayerId, Player>> = get_write_lock(&PLAYERS)?;
    let player: &mut Player = get_player_mut!(players_guard, *round_starter_id);
    let prompt: String = format!(
        "{}: {}\nChoose a card to play:",
        player.name,
        player.get_hand()
    );
    if let PlayerChoice::Choice(player_choice) =
        get_player_choice(player, &prompt, false, player.hand.len() - 1)?
    {
        ground.add_card(player.id, player.hand.remove(player_choice))?;
    }
    Ok(())
}

fn continue_round(ground: &mut Ground, index: usize) -> Result<()> {
    let ground_cards: String = {
        let players_guard: RwLockReadGuard<BTreeMap<PlayerId, Player>> = get_read_lock(&PLAYERS)?;
        ground
            .cards
            .iter()
            .map(|(player_id, card)| {
                Ok(format!(
                    "{}:{}",
                    get_player!(players_guard, *player_id).name,
                    card
                ))
            })
            .collect::<Result<Vec<String>, Error>>()?
            .join(", ")
    };
    broadcast_message(&ground_cards)?;
    let player_to_play_id: PlayerId = get_read_lock(&FIELD)?[index % NUMBER_OF_PLAYERS];
    let mut players_guard: RwLockWriteGuard<BTreeMap<PlayerId, Player>> = get_write_lock(&PLAYERS)?;
    let player: &mut Player = get_player_mut!(players_guard, player_to_play_id);
    let mut pre: String = String::new();
    loop {
        let player_hand: String = player.get_hand();
        if let PlayerChoice::Choice(player_choice) = get_player_choice(
            player,
            &format!("{pre}\n{player_hand}\nChoose a card to play:"),
            false,
            player.hand.len() - 1,
        )? {
            let has_matching_card: bool = player
                .hand
                .iter()
                .any(|player_card: &Card| player_card.type_ == ground.type_);
            if has_matching_card && player.hand[player_choice].type_ != ground.type_ {
                pre = format!("You have {}!\n", ground.type_.name);
                continue;
            }
            return ground.add_card(player.id, player.hand.remove(player_choice));
        }
    }
}

fn finish_round(off_team_id: TeamId, def_team_id: TeamId, bet: usize) -> Result<()> {
    let mut teams_guard: RwLockWriteGuard<BTreeMap<TeamId, Team>> = get_write_lock(&TEAMS)?;
    let off_team: &mut Team = get_team_mut!(teams_guard, off_team_id);
    let team_string: String = if off_team.collected_hands.len() == bet {
        off_team.score += if bet == HIGHEST_BET { bet * 2 } else { bet };
        off_team
    } else {
        let def_team: &mut Team = get_team_mut!(teams_guard, def_team_id);
        def_team.score += bet * 2;
        def_team
    }
    .to_string();
    broadcast_message(&format!("Winner of this round is: {team_string}"))
}

fn prepare_next_round() -> Result<()> {
    let mut cards_guard: RwLockWriteGuard<Vec<Card>> = get_write_lock(&CARDS)?;
    get_write_lock(&TEAMS)?
        .values_mut()
        .for_each(|team: &mut Team| {
            team.collected_hands
                .drain(..)
                .for_each(|hand: Vec<Card>| cards_guard.extend(hand));
        });
    get_write_lock(&PLAYERS)?
        .values_mut()
        .for_each(|player: &mut Player| {
            cards_guard.extend(player.hand.drain(..));
        });
    Ok(())
}

fn should_continue_round(off_team_id: TeamId, def_team_id: TeamId, bet: usize) -> Result<bool> {
    let teams_guard: RwLockReadGuard<BTreeMap<TeamId, Team>> = get_read_lock(&TEAMS)?;
    let off_team: &Team = get_team!(teams_guard, off_team_id);
    let def_team: &Team = get_team!(teams_guard, def_team_id);
    Ok(off_team.collected_hands.len() < bet && def_team.collected_hands.len() < (14 - bet))
}

fn should_continue_game() -> Result<bool> {
    Ok(get_read_lock(&TEAMS)?
        .values()
        .all(|team: &Team| team.score < TARGET_SCORE))
}

fn finish_game() -> Result<()> {
    let winner_team: &str = &get_read_lock(&TEAMS)?
        .values()
        .find(|team: &&Team| team.score >= TARGET_SCORE)
        .map(ToString::to_string)
        .ok_or_else(|| Error::Other("Team with required score was not found".to_string()))?;
    broadcast_message(&format!("Winner is {winner_team}"))
}

fn get_opposing_team_id(team_id: TeamId) -> Result<TeamId> {
    Ok(*get_read_lock(&TEAMS)?
        .keys()
        .find(|opposing_team_id: &&TeamId| **opposing_team_id != team_id)
        .ok_or_else(|| Error::Other("Opposing team ID not found".to_owned()))?)
}

fn start_game() -> Result<()> {
    generate_cards()?;
    generate_field()?;
    shuffle_cards(true)?;
    while should_continue_game()? {
        get_read_lock(&TEAMS)?
            .values()
            .sorted_by_key(ToString::to_string)
            .try_for_each(|team: &Team| {
                broadcast_message(&format!("{}: {}", team.name, team.score))
            })?;
        shuffle_cards(false)?;
        let ground_cards: Vec<Card> = get_write_lock(&CARDS)?.drain(0..4).collect();
        hand_out_cards()?;
        let (highest_bet, highest_bettor_id, off_team_id) = start_betting(ground_cards)?;
        let mut round_starter_id: PlayerId = set_starter(highest_bettor_id, highest_bet)?;
        fold_first(highest_bettor_id)?;
        set_hokm(highest_bettor_id, highest_bet)?;
        let def_team_id: TeamId = get_opposing_team_id(off_team_id)?;
        while should_continue_round(off_team_id, def_team_id, highest_bet)? {
            get_read_lock(&TEAMS)?
                .values()
                .sorted_by_key(ToString::to_string)
                .try_for_each(|team: &Team| {
                    broadcast_message(&format!("{}: {}", team.name, team.collected_hands.len()))
                })?;
            let round_starter_index: usize = get_read_lock(&FIELD)?
                .iter()
                .find_position(|player_id: &&PlayerId| **player_id == round_starter_id)
                .map(|(index, _)| index)
                .ok_or_else(|| Error::player_not_found(round_starter_id))?;
            let mut ground: Ground = Ground::new();
            start_round(&mut ground, &round_starter_id)?;
            (1..NUMBER_OF_PLAYERS).try_for_each(|index: usize| {
                continue_round(&mut ground, round_starter_index + index)
            })?;
            round_starter_id = get_hand_collector_id(&ground)?;
            collect_hand(round_starter_id, ground)?;
        }
        finish_round(off_team_id, def_team_id, highest_bet)?;
        prepare_next_round()?;
    }
    finish_game()
}

fn client_handler(connection: TcpStream) -> Result<()> {
    let message: &'static str = "1$_$_$Choose your name:";
    send_message(&connection, message)?;
    let name: String = receive_message(&connection)?;
    let mut pre: &'static str = "";
    let mut teams_guard: RwLockWriteGuard<BTreeMap<TeamId, Team>> = get_write_lock(&TEAMS)?;
    loop {
        let available_teams: Vec<&Team> = teams_guard
            .values()
            .filter(|team: &&Team| team.players.len() < TEAM_SIZE)
            .sorted_by_key(ToString::to_string)
            .collect();
        let available_teams_str: String = available_teams
            .iter()
            .enumerate()
            .map(|(i, team)| format!("{}:{}", team.name, i))
            .join(", ");
        let message: String = format!("1$_$_${}Choose your team: {}", pre, available_teams_str);
        send_message(&connection, &message)?;
        match receive_message(&connection)?.parse::<usize>() {
            Ok(team) if team < available_teams.len() => {
                let team_id: TeamId = available_teams[team].id;
                let player: Player = Player::new(name, team_id, connection);
                get_team_mut!(teams_guard, team_id).players.push(player.id);
                get_write_lock(&PLAYERS)?.insert(player.id, player);
                *get_write_lock(&NUMBER_OF_CLIENTS)? += 1;
                return Ok(());
            }
            _ => pre = INVALID_RESPONSE,
        }
    }
}

fn main() -> Result<()> {
    generate_teams()?;
    let listener: std::net::TcpListener = get_listener()?;
    // listener.set_nonblocking(true).unwrap();
    while *get_read_lock(&NUMBER_OF_CLIENTS)? != NUMBER_OF_PLAYERS {
        match listener.accept() {
            Ok((stream, _)) => client_handler(stream)?,
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
            Err(err) => println!("{}", err),
        }
    }
    broadcast_message("All players connected. Game starting...!")?;
    start_game()
}
