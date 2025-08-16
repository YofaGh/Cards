#[macro_export]
macro_rules! get_player_mut {
    ($players:expr, $id:expr) => {
        $players.get_mut_or_error(&$id, || Error::player_not_found($id))
    };
}

#[macro_export]
macro_rules! get_player {
    ($players:expr, $id:expr) => {
        $players.get_or_error(&$id, || Error::player_not_found($id))
    };
}

#[macro_export]
macro_rules! get_player_field_index {
    ($field:expr, $id:expr) => {
        $field
            .iter()
            .find_position(|player_id: &&PlayerId| **player_id == $id)
            .map(|(index, _)| index)
            .ok_or(Error::player_not_found($id))
    };
}

#[macro_export]
macro_rules! get_team_mut {
    ($teams:expr, $id:expr) => {
        $teams.get_mut_or_error(&$id, || Error::team_not_found($id))
    };
}

#[macro_export]
macro_rules! get_team {
    ($teams:expr, $id:expr) => {
        $teams.get_or_error(&$id, || Error::team_not_found($id))
    };
}

#[macro_export]
macro_rules! get_player_communication {
    ($players_receiver:expr, $players_sender:expr, $player_id:expr) => {{
        let receiver = $players_receiver
            .get_mut(&$player_id)
            .ok_or(Error::player_not_found($player_id));
        let sender = $players_sender
            .get(&$player_id)
            .ok_or(Error::player_not_found($player_id));
        match (receiver, sender) {
            (Ok(r), Ok(s)) => Ok((r, s)),
            (Err(e), _) | (_, Err(e)) => Err(e),
        }
    }};
}
