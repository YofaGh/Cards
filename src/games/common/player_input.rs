use tokio::time::timeout;

use crate::{
    games::INVALID_RESPONSE,
    models::{Card, Player},
    prelude::*,
};

pub async fn get_player_choice(
    player: &mut Player,
    message: &mut GameMessage,
    passable: bool,
    max_value: usize,
) -> Result<PlayerChoice> {
    let player_name: String = player.name.clone();
    let operation = async {
        loop {
            player.send_message(message).await?;
            match player.receive_message().await? {
                GameMessage::PlayerChoice { choice } => {
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
                        match Card::try_from(choice) {
                            Ok(card) => {
                                if player.hand.contains(&card) {
                                    return Ok(PlayerChoice::CardChoice(card));
                                }
                                message.set_demand_error("You don't have this card!".to_owned());
                            }
                            Err(_) => message.set_demand_error(INVALID_RESPONSE.to_owned()),
                        }
                    }
                }
                invalid => {
                    message.set_demand_error(format!(
                        "Expected message type PlayerChoice, but received {}",
                        invalid.message_type()
                    ));
                }
            }
        }
    };
    let config: &'static Config = get_config();
    if config.timeout.player_choice_enabled {
        return timeout(config.timeout.player_choice, operation)
            .await
            .timeout_context(format!("Player {player_name} took too long to make choice"));
    }
    operation.await
}

pub async fn get_player_team_choice(
    player: &mut Player,
    available_teams: Vec<(TeamId, String)>,
) -> Result<TeamId> {
    let player_name: String = player.name.clone();
    let mut message: GameMessage = GameMessage::team(
        available_teams
            .iter()
            .map(|(_, name)| name.clone())
            .collect(),
        String::new(),
    );
    let operation = async {
        loop {
            player.send_message(&message).await?;
            match player.receive_message().await? {
                GameMessage::PlayerChoice { choice } => {
                    if let Some((team_id, _)) =
                        available_teams.iter().find(|(_, name)| *name == choice)
                    {
                        return Ok(*team_id);
                    } else {
                        message.set_demand_error("Invalid team choice".to_owned());
                    }
                }
                invalid => {
                    message.set_demand_error(format!(
                        "Expected PlayerChoice, got {}",
                        invalid.message_type()
                    ));
                }
            }
        }
    };
    let config: &'static Config = get_config();
    if config.timeout.player_choice_enabled {
        return timeout(config.timeout.player_choice, operation)
            .await
            .timeout_context(format!("Player {player_name} took too long to choose team"));
    }
    operation.await
}
