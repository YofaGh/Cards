#[macro_export]
macro_rules! get_player_mut {
    ($players:expr, $id:expr) => {
        $players.get_mut_or_error(&$id, || Error::player_not_found($id))?
    };
}

#[macro_export]
macro_rules! get_player {
    ($players:expr, $id:expr) => {
        $players.get_or_error(&$id, || Error::player_not_found($id))?
    };
}

#[macro_export]
macro_rules! get_team_mut {
    ($teams:expr, $id:expr) => {
        $teams.get_mut_or_error(&$id, || Error::team_not_found($id))?
    };
}

#[macro_export]
macro_rules! get_team {
    ($teams:expr, $id:expr) => {
        $teams.get_or_error(&$id, || Error::team_not_found($id))?
    };
}

#[macro_export]
macro_rules! get_player_communication {
    ($self:expr, $player_id:expr) => {{
        let receiver = $self
            .players_receiver
            .get_mut(&$player_id)
            .ok_or(Error::player_not_found($player_id))?;
        let sender = $self
            .players_sender
            .get(&$player_id)
            .ok_or(Error::player_not_found($player_id))?;
        (receiver, sender)
    }};
}