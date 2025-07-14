use std::cmp::Ordering;

use crate::prelude::*;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Card {
    pub type_: Hokm,
    pub number: String,
    pub ord: usize,
}

impl Card {
    pub fn new(type_: Hokm, number: String, ord: usize) -> Self {
        Card { type_, number, ord }
    }
    pub fn code(&self) -> String {
        format!("{}-{}", self.type_.code(), self.number)
    }
}

impl TryFrom<String> for Card {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        if let Some((hokm_code, card_number)) = value.split_once("-") {
            if let Some(ord) = crate::games::get_card_ord_by_number(card_number) {
                return Ok(Card::new(
                    Hokm::from(hokm_code.to_string()),
                    card_number.to_string(),
                    ord,
                ));
            }
        }
        Err(Error::NoValidCard)
    }
}

impl std::fmt::Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} {}", self.type_.unicode_char(), self.number)
    }
}

impl PartialEq for Card {
    fn eq(&self, other: &Self) -> bool {
        self.type_ == other.type_ && self.ord == other.ord
    }
}

impl Eq for Card {}

impl PartialOrd for Card {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.ord.cmp(&other.ord))
    }
}

impl Ord for Card {
    fn cmp(&self, other: &Self) -> Ordering {
        self.ord.cmp(&other.ord)
    }
}
