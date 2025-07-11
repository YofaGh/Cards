use crate::{games::NUMBERS, models::Card};

pub fn code_cards(cards: &[Card]) -> Vec<String> {
    cards.iter().map(|card: &Card| card.code()).collect()
}

pub fn get_card_ord_by_number(card_number: &str) -> Option<usize> {
    NUMBERS.iter().position(|&x| x == card_number)
}