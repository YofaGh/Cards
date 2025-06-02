use crate::models::Hokm;

pub const INVALID_RESPONSE: &str = "Invalid. try again\n";
pub const SPADES: Hokm = Hokm {
    name: "Spades",
    unicode_char: "\u{2660}",
};
pub const HEARTS: Hokm = Hokm {
    name: "Hearts",
    unicode_char: "\u{2665}",
};
pub const DIAMONDS: Hokm = Hokm {
    name: "Diamonds",
    unicode_char: "\u{2666}",
};
pub const CLUBS: Hokm = Hokm {
    name: "Clubs",
    unicode_char: "\u{2663}",
};
pub const NARAS: Hokm = Hokm {
    name: "Naras",
    unicode_char: "\u{2193}",
};
pub const SARAS: Hokm = Hokm {
    name: "Saras",
    unicode_char: "\u{2191}",
};
pub const TAK_NARAS: Hokm = Hokm {
    name: "Tak Naras",
    unicode_char: "\u{21a7}",
};
pub const TYPES: [Hokm; 4] = [SPADES, HEARTS, DIAMONDS, CLUBS];
pub const HOKMS: [Hokm; 7] = [SPADES, HEARTS, DIAMONDS, CLUBS, NARAS, SARAS, TAK_NARAS];
pub const NUMBERS: [&str; 13] = [
    "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K", "A",
];
