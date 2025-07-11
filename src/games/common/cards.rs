use crate::models::Card;

pub fn code_cards(cards: &[Card]) -> Vec<String> {
    cards.iter().map(|card: &Card| card.code()).collect()
}
