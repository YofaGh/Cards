use crate::enums::Hokm;

pub const SERVER_HOST: &str = "127.0.0.1";
pub const SERVER_PORT: &str = "0";
pub const INVALID_RESPONSE: &str = "Invalid. try again\n";
pub const TYPES: [Hokm; 4] = [Hokm::Spades, Hokm::Hearts, Hokm::Diamonds, Hokm::Clubs];
pub const HOKMS: [Hokm; 7] = [
    Hokm::Spades,
    Hokm::Hearts,
    Hokm::Diamonds,
    Hokm::Clubs,
    Hokm::Naras,
    Hokm::Saras,
    Hokm::TakNaras,
];
pub const NUMBERS: [&str; 13] = [
    "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K", "A",
];
