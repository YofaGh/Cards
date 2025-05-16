mod client;
mod constants;
mod enums;
mod errors;
mod macros;
mod models;
mod types;

use constants::*;
use enums::*;
use errors::Error;
use itertools::Itertools;
use lazy_static::lazy_static;
use models::*;
use rand::{rngs::ThreadRng, seq::SliceRandom, Rng};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard},
    thread,
    time::Duration,
    usize,
};
use types::*;
use uuid::Uuid;

const NUMBER_OF_PLAYERS: usize = 4;
const HIGHEST_BET: usize = 13;
const TEAM_SIZE: usize = 2;
const NUMBER_OF_TEAMS: usize = NUMBER_OF_PLAYERS / TEAM_SIZE;

lazy_static! {
    static ref TEAMS: Arc<RwLock<BTreeMap<TeamId, Team>>> = Arc::new(RwLock::new(BTreeMap::new()));
    static ref FIELD: Arc<RwLock<Vec<PlayerId>>> = Arc::new(RwLock::new(Vec::new()));
    static ref CARDS: Arc<RwLock<Vec<Card>>> = Arc::new(RwLock::new(Vec::new()));
    static ref STARTER: Arc<RwLock<PlayerId>> = Arc::new(RwLock::new(Uuid::nil()));
    static ref HOKM: Arc<RwLock<Hokm>> = Arc::new(RwLock::new(Hokm::default()));
    static ref NUMBER_OF_CLIENTS: Arc<RwLock<usize>> = Arc::new(RwLock::new(0));
    static ref PLAYERS: Arc<RwLock<BTreeMap<PlayerId, Player>>> =
        Arc::new(RwLock::new(BTreeMap::new()));
}

fn get_read_lock<T>(rwlock: &'static RwLock<T>) -> Result<RwLockReadGuard<'static, T>> {
    rwlock
        .read()
        .map_err(|err: PoisonError<RwLockReadGuard<T>>| {
            Error::Lock(format!("Read lock error {}", err.to_string()))
        })
}

fn get_write_lock<T>(rwlock: &'static RwLock<T>) -> Result<RwLockWriteGuard<'static, T>> {
    rwlock
        .write()
        .map_err(|err: PoisonError<RwLockWriteGuard<T>>| {
            Error::Lock(format!("Write lock error {}", err.to_string()))
        })
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
            continue;
        }
        if let Ok(choice) = response.parse::<usize>() {
            if choice > max_value {
                pre = format!("Choice can't be greater than {}", max_value);
                continue;
            }
            return Ok(PlayerChoice::Choice(choice));
        } else {
            pre = INVALID_RESPONSE.to_owned();
            continue;
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
        cards_guard.shuffle(&mut rng);
        return Ok(());
    }
    let random_time: i32 = rng.random_range(1..=3);
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

fn set_starter(highest_better_id: PlayerId, highest_bet: usize) -> Result<()> {
    let mut starter_guard: RwLockWriteGuard<PlayerId> = get_write_lock(&STARTER)?;
    if starter_guard.is_nil() || highest_bet == HIGHEST_BET {
        *starter_guard = highest_better_id;
        return Ok(());
    }
    let team_with_highest_score_id: TeamId = get_read_lock(&TEAMS)?
        .values()
        .max_by_key(|team: &&Team| team.score)
        .map(|team: &Team| team.id)
        .ok_or_else(|| Error::Other("team with highest score was found".to_owned()))?;
    let starter_team_id: PlayerId = get_player!(get_read_lock(&PLAYERS)?, *starter_guard).team_id;
    if starter_team_id == team_with_highest_score_id {
        return Ok(());
    }
    let field_guard: RwLockReadGuard<Vec<PlayerId>> = get_read_lock(&FIELD)?;
    let index: usize = field_guard
        .iter()
        .find_position(|player_id: &&PlayerId| **player_id == *starter_guard)
        .map(|(index, _)| index)
        .ok_or_else(|| Error::player_not_found(*starter_guard))?;
    *starter_guard = field_guard[(index + 1) % field_guard.len()];
    Ok(())
}

fn fold_first(player_id: PlayerId) -> Result<()> {
    let mut players_guard: RwLockWriteGuard<BTreeMap<PlayerId, Player>> = get_write_lock(&PLAYERS)?;
    let player: &mut Player = get_player_mut!(players_guard, player_id);
    let mut folded_cards: Vec<Card> = Vec::new();
    while player.hand.len() > 12 {
        let player_hand: String = player.get_hand();
        if let PlayerChoice::Choice(player_choice) = get_player_choice(
            player,
            &format!("{player_hand}\nChoose a card to fold"),
            false,
            player.hand.len() - 1,
        )? {
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
    let hokms_to_show: String = hokms
        .iter()
        .enumerate()
        .map(|(index, hokm)| format!("{}:{}", hokm.to_string(), index))
        .join(", ");
    let players_guard: RwLockReadGuard<BTreeMap<PlayerId, Player>> = get_read_lock(&PLAYERS)?;
    let player: &Player = get_player!(players_guard, player_id);
    let mut pre: &'static str = "";
    loop {
        let prompt: String = format!(
            "{}{} what is your hokm? {}",
            pre, player.name, hokms_to_show
        );
        if let PlayerChoice::Choice(player_choice) =
            get_player_choice(player, &prompt, false, hokms.len())?
        {
            if player_choice > 3 && bet != HIGHEST_BET {
                pre = INVALID_RESPONSE;
                continue;
            }
            *get_write_lock(&HOKM)? = hokms[player_choice].clone();
            return Ok(());
        }
    }
}

fn hand_collector(ground: &Ground) -> Result<PlayerId> {
    let hokm_guard: RwLockReadGuard<Hokm> = get_read_lock(&HOKM)?;
    if hokm_guard.eq(&NARAS) {
        ground
            .cards
            .iter()
            .filter(|(_, card)| card.type_ == ground.type_)
            .min_by_key(|(_, card)| card.ord)
            .map(|(player_id, _)| *player_id)
            .ok_or(Error::NoValidCard)
    } else if hokm_guard.eq(&SARAS) {
        ground
            .cards
            .iter()
            .filter(|(_, card)| card.type_ == ground.type_)
            .max_by_key(|(_, card)| card.ord)
            .map(|(player_id, _)| *player_id)
            .ok_or(Error::NoValidCard)
    } else if hokm_guard.eq(&TAK_NARAS) {
        ground
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
            })
            .map(|(player_id, _)| *player_id)
            .ok_or(Error::NoValidCard)
    } else {
        let hokm_winner: Option<PlayerId> = ground
            .cards
            .iter()
            .filter(|(_, card)| card.type_ == *hokm_guard)
            .max_by_key(|(_, card)| card.ord)
            .map(|(player_id, _)| *player_id);
        if let Some(hw) = hokm_winner {
            return Ok(hw);
        }
        ground
            .cards
            .iter()
            .filter(|(_, card)| card.type_ == ground.type_)
            .max_by_key(|(_, card)| card.ord)
            .map(|(player_id, _)| *player_id)
            .ok_or(Error::NoValidCard)
    }
}

fn start_betting() -> Result<(usize, PlayerId)> {
    let mut highest_bet_option: Option<usize> = None;
    let mut highest_better_id: PlayerId = Uuid::nil();
    let mut others_bets: Vec<String> = Vec::new();
    loop {
        for player_id in get_read_lock(&FIELD)?.iter() {
            let players_guard: RwLockReadGuard<BTreeMap<PlayerId, Player>> =
                get_read_lock(&PLAYERS)?;
            let player: &Player = get_player!(players_guard, *player_id);
            let player_hand: String = player.hand.iter().map(ToString::to_string).join(", ");
            let prompt: String = format!("These are your cards: {player_hand}\nWhat is your bet?");
            match get_player_choice(player, &prompt, true, HIGHEST_BET)? {
                PlayerChoice::Pass => continue,
                PlayerChoice::Choice(player_choice) => {
                    if highest_bet_option
                        .map_or(true, |highest_bet: usize| player_choice > highest_bet)
                    {
                        highest_bet_option = Some(player_choice);
                        highest_better_id = *player_id;
                        others_bets.push(format!("{}: {}", player.name.to_owned(), player_choice));
                        if player_choice == HIGHEST_BET {
                            break;
                        }
                    }
                }
            }
            broadcast_message(&others_bets.join(", "))?;
        }
        if let Some(highest_bet) = highest_bet_option {
            return Ok((highest_bet, highest_better_id));
        }
    }
}

fn start_round(ground: &mut Ground, round_starter_id: &PlayerId) -> Result<()> {
    let mut players_guard: RwLockWriteGuard<BTreeMap<PlayerId, Player>> = get_write_lock(&PLAYERS)?;
    let player_to_start: &mut Player = get_player_mut!(players_guard, *round_starter_id);
    let player_hand: String = player_to_start.get_hand();
    if let PlayerChoice::Choice(player_choice) = get_player_choice(
        player_to_start,
        &format!(
            "{}: {}\nChoose a card to play:",
            player_to_start.name, player_hand
        ),
        false,
        player_to_start.hand.len() - 1,
    )? {
        ground.add_card(
            player_to_start.id,
            player_to_start.hand.remove(player_choice),
        )?;
    }
    Ok(())
}

fn continue_round(ground: &mut Ground, round_starter_index: usize, index: usize) -> Result<()> {
    let players_guard: RwLockReadGuard<BTreeMap<PlayerId, Player>> = get_read_lock(&PLAYERS)?;
    let ground_cards: String = ground
        .cards
        .iter()
        .map(|(player_id, card)| {
            Ok(format!(
                "{}:{}",
                get_player!(players_guard, *player_id).name,
                card.to_string()
            ))
        })
        .collect::<Result<Vec<String>, Error>>()?
        .join(", ");
    broadcast_message(&ground_cards)?;
    let field_guard: RwLockReadGuard<Vec<PlayerId>> = get_read_lock(&FIELD)?;
    let player_to_play_id: PlayerId =
        field_guard[(round_starter_index + index) % field_guard.len()];
    let mut players_guard: RwLockWriteGuard<BTreeMap<PlayerId, Player>> = get_write_lock(&PLAYERS)?;
    let player_to_play: &mut Player = get_player_mut!(players_guard, player_to_play_id);
    let mut pre: String = String::new();
    loop {
        let player_hand: String = player_to_play.get_hand();
        if let PlayerChoice::Choice(player_choice) = get_player_choice(
            player_to_play,
            &format!("{pre}\n{player_hand}\nChoose a card to play:"),
            false,
            player_to_play.hand.len() - 1,
        )? {
            let has_matching_card: bool = player_to_play
                .hand
                .iter()
                .any(|player_card: &Card| player_card.type_ == ground.type_);
            if has_matching_card && player_to_play.hand[player_choice].type_ != ground.type_ {
                pre = format!("You have {}!\n", ground.type_.name);
                continue;
            }
            return ground.add_card(player_to_play.id, player_to_play.hand.remove(player_choice));
        }
    }
}

fn finish_round(better_team_id: TeamId, catcher_team_id: TeamId, highest_bet: usize) -> Result<()> {
    let team_string: String;
    let mut teams_guard: RwLockWriteGuard<BTreeMap<TeamId, Team>> = get_write_lock(&TEAMS)?;
    let better_team: &mut Team = get_team_mut!(teams_guard, better_team_id);
    if better_team.collected_hands.len() == highest_bet {
        better_team.score += if highest_bet == HIGHEST_BET {
            HIGHEST_BET * 2
        } else {
            highest_bet
        };
        team_string = better_team.to_string();
    } else {
        let catcher_team: &mut Team = get_team_mut!(teams_guard, catcher_team_id);
        catcher_team.score += highest_bet * 2;
        team_string = catcher_team.to_string();
    }
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

fn should_continue_round(
    better_team_id: TeamId,
    catcher_team_id: TeamId,
    highest_bet: usize,
) -> Result<bool> {
    let teams_guard: RwLockReadGuard<BTreeMap<TeamId, Team>> = get_read_lock(&TEAMS)?;
    let better_team: &Team = get_team!(teams_guard, better_team_id);
    let catcher_team: &Team = get_team!(teams_guard, catcher_team_id);
    Ok(better_team.collected_hands.len() < highest_bet
        && catcher_team.collected_hands.len() < (14 - highest_bet))
}

fn should_continue_game() -> Result<bool> {
    Ok(get_read_lock(&TEAMS)?
        .values()
        .all(|team: &Team| team.score < 104))
}

fn start_game() -> Result<()> {
    generate_cards()?;
    generate_field()?;
    shuffle_cards(true)?;
    while should_continue_game()? {
        get_read_lock(&TEAMS)?
            .values()
            .try_for_each(|team: &Team| {
                broadcast_message(format!("{}: {}", team.name, team.score).as_str())
            })?;
        broadcast_message("Shuffling cards...")?;
        shuffle_cards(false)?;
        broadcast_message("Handing out cards...")?;
        let ground_cards: Vec<Card> = get_write_lock(&CARDS)?.drain(0..4).collect();
        hand_out_cards()?;
        let (highest_bet, highest_better_id) = start_betting()?;
        let (highest_better_team_id, highest_better_name) = {
            let mut players_guard: RwLockWriteGuard<BTreeMap<PlayerId, Player>> =
                get_write_lock(&PLAYERS)?;
            let highest_better: &mut Player = get_player_mut!(players_guard, highest_better_id);
            highest_better.add_cards(ground_cards)?;
            (highest_better.team_id, highest_better.name.clone())
        };
        broadcast_message(&format!(
            "{} wins with {}!",
            highest_better_name, highest_bet
        ))?;
        set_starter(highest_better_id, highest_bet)?;
        let starter_guard: RwLockReadGuard<PlayerId> = get_read_lock(&STARTER)?;
        let (starter_name, starter_id) = {
            let players_guard: RwLockReadGuard<BTreeMap<PlayerId, Player>> =
                get_read_lock(&PLAYERS)?;
            let starter: &Player = get_player!(players_guard, *starter_guard);
            (starter.name.to_owned(), starter.id)
        };
        broadcast_message(&format!("Starter: {}", starter_name))?;
        fold_first(highest_better_id)?;
        set_hokm(highest_better_id, highest_bet)?;
        broadcast_message(&format!("Hokm: {}", get_read_lock(&HOKM)?.to_string()))?;
        let catcher_team_id: TeamId = *get_read_lock(&TEAMS)?
            .keys()
            .find(|team_id: &&TeamId| **team_id != highest_better_team_id)
            .ok_or_else(|| Error::Other("Catcher team ID not found".to_owned()))?;
        let mut round_starter_id: PlayerId = starter_id;
        while should_continue_round(highest_better_team_id, catcher_team_id, highest_bet)? {
            let round_starter_index: usize = get_read_lock(&FIELD)?
                .iter()
                .find_position(|player_id: &&PlayerId| **player_id == round_starter_id)
                .map(|(index, _)| index)
                .ok_or_else(|| Error::player_not_found(round_starter_id))?;
            broadcast_message(
                &get_read_lock(&TEAMS)?
                    .values()
                    .sorted_by_key(|team: &&Team| team.name.to_owned())
                    .map(|team: &Team| format!("{}: {}", team.name, team.collected_hands.len()))
                    .join("\n"),
            )?;
            let mut ground: Ground = Ground::new();
            start_round(&mut ground, &round_starter_id)?;
            (1..NUMBER_OF_PLAYERS).try_for_each(|index: usize| {
                continue_round(&mut ground, round_starter_index, index)
            })?;
            round_starter_id = hand_collector(&ground)?;
            let team_to_collect_id: TeamId =
                get_player!(get_write_lock(&PLAYERS)?, round_starter_id).team_id;
            get_team_mut!(get_write_lock(&TEAMS)?, team_to_collect_id)
                .collected_hands
                .push(ground.cards.into_iter().map(|(_, card)| card).collect());
        }
        finish_round(highest_better_team_id, catcher_team_id, highest_bet)?;
        prepare_next_round()?;
    }
    let team_winner_name: &str = &get_read_lock(&TEAMS)?
        .values()
        .find(|team: &&Team| team.score >= 104)
        .ok_or_else(|| Error::Other("Team with required score was not found".to_string()))?
        .name
        .to_owned();
    broadcast_message(&format!("Winner is {team_winner_name}"))
}

fn client_handler(mut connection: TcpStream) -> Result<()> {
    let message: &'static str = "1$_$_$Choose your name:";
    let message_bytes: &[u8] = message.as_bytes();
    connection
        .write_all(&message_bytes.len().to_be_bytes())
        .map_err(Error::connection)?;
    connection
        .write_all(message_bytes)
        .map_err(Error::connection)?;
    let mut buffer: [u8; 1024] = [0; 1024];
    let bytes_read: usize = connection.read(&mut buffer).map_err(Error::connection)?;
    let name: String = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
    let mut pre: &'static str = "";
    'outer: loop {
        let mut teams_guard: RwLockWriteGuard<BTreeMap<TeamId, Team>> = get_write_lock(&TEAMS)?;
        let available_teams: Vec<&Team> = teams_guard
            .values()
            .filter_map(|team: &Team| {
                if team.players.len() < TEAM_SIZE {
                    Some(team)
                } else {
                    None
                }
            })
            .sorted_by_key(|team: &&Team| team.name.to_owned())
            .collect();
        let available_teams_str: String = available_teams
            .iter()
            .enumerate()
            .map(|(i, team)| format!("{}:{}", team.name, i))
            .join(", ");
        let message: String = format!("1$_$_${}Choose your team: {}", pre, available_teams_str);
        let message_bytes: &[u8] = message.as_bytes();
        connection
            .write_all(&message_bytes.len().to_be_bytes())
            .map_err(Error::connection)?;
        connection
            .write_all(message_bytes)
            .map_err(Error::connection)?;
        let mut buffer: [u8; 1024] = [0; 1024];
        let bytes_read: usize = connection.read(&mut buffer).map_err(Error::connection)?;
        let response: String = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
        match response.parse::<usize>() {
            Ok(team) => {
                if team >= available_teams.len() || available_teams[team].players.len() >= TEAM_SIZE
                {
                    pre = INVALID_RESPONSE;
                    continue;
                }
                let team_id: TeamId = available_teams[team].id;
                connection.flush().map_err(Error::connection)?;
                let player: Player = Player::new(name, team_id, connection);
                get_team_mut!(teams_guard, team_id).players.push(player.id);
                get_write_lock(&PLAYERS)?.insert(player.id, player);
                *get_write_lock(&NUMBER_OF_CLIENTS)? += 1;
                break 'outer;
            }
            Err(_) => pre = INVALID_RESPONSE,
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    generate_teams()?;
    let listener: TcpListener =
        TcpListener::bind(format!("{}:{}", HOST, PORT)).map_err(Error::bind_port)?;
    // listener.set_nonblocking(true).unwrap();
    while *get_read_lock(&NUMBER_OF_CLIENTS)? != NUMBER_OF_PLAYERS {
        match listener.accept() {
            Ok((stream, _)) => client_handler(stream)?,
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(200));
            }
            Err(err) => println!("{}", err),
        }
    }
    broadcast_message("All players connected. Game starting...!")?;
    start_game()
}
