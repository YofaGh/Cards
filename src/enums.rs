pub enum PlayerChoice {
    Pass,
    Choice(usize),
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum MessageType {
    Broadcast,
    Handshake,
    Username,
    TeamChoice,
    Bet,
    Fold,
    HokmChoice,
    CardPlay,
    Unknown = 100,
}

impl From<&str> for MessageType {
    fn from(value: &str) -> Self {
        match value {
            "0" => MessageType::Broadcast,
            "1" => MessageType::Handshake,
            "2" => MessageType::Username,
            "3" => MessageType::TeamChoice,
            "4" => MessageType::Bet,
            "5" => MessageType::Fold,
            "6" => MessageType::HokmChoice,
            "7" => MessageType::CardPlay,
            _ => MessageType::Unknown,
        }
    }
}