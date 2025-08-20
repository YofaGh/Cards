//! Card shuffling algorithms module
//!
//! This module provides various shuffling methods that simulate real-world card shuffling techniques.
//! Each method has different characteristics in terms of randomization and how it affects the order
//! of cards, making the shuffling behavior more realistic and varied.

#![allow(dead_code)]

use rand::{rngs::ThreadRng, seq::SliceRandom, Rng};
use std::clone::Clone;

/// Enumeration of different shuffling methods available
///
/// Each method simulates a different real-world card shuffling technique:
/// - `Hard`: Complete randomization using Fisher-Yates shuffle
/// - `Riffle`: Simulates the riffle shuffle where cards are split and interleaved
/// - `Cut`: Simple cut where the deck is split and bottom half moves to top
/// - `Overhand`: Simulates overhand shuffling by moving small packets from back to front
/// - `Hindu`: Simulates Hindu shuffle by taking packets from front and reversing them
pub enum ShuffleMethod {
    /// Complete randomization - perfectly random shuffle
    Hard,
    /// Riffle shuffle - splits deck and interleaves the halves
    Riffle,
    /// Cut shuffle - simple cut and stack operation
    Cut,
    /// Overhand shuffle - moves small packets from back to front
    Overhand,
    /// Hindu shuffle - takes packets from front and reverses them
    Hindu,
}

/// Shuffles a vector of items using the specified shuffling method
///
/// # Arguments
///
/// * `items` - A mutable reference to the vector to be shuffled
/// * `method` - The shuffling method to use
///
/// # Type Parameters
///
/// * `Item` - The type of items in the vector, must implement `Clone`
///
/// # Examples
///
/// ```
/// use crate::games::common::shuffler::{shuffle, ShuffleMethod};
///
/// let mut cards = vec![1, 2, 3, 4, 5];
/// shuffle(&mut cards, ShuffleMethod::Riffle);
/// ```
pub fn shuffle<Item: Clone>(items: &mut Vec<Item>, method: ShuffleMethod) {
    match method {
        ShuffleMethod::Hard => hard_shuffle(items),
        ShuffleMethod::Riffle => riffle_shuffle(items),
        ShuffleMethod::Cut => cut_shuffle(items),
        ShuffleMethod::Overhand => overhand_shuffle(items),
        ShuffleMethod::Hindu => hindu_shuffle(items),
    }
}

/// Performs a perfect random shuffle using the Fisher-Yates algorithm
///
/// This is the most thorough shuffling method, providing true randomization
/// where every possible permutation is equally likely.
///
/// # Arguments
///
/// * `items` - A mutable slice of items to shuffle
fn hard_shuffle<Item>(items: &mut [Item]) {
    items.shuffle(&mut rand::rng());
}

/// Simulates a riffle shuffle by splitting the deck and interleaving cards
///
/// The deck is split roughly in half (with some randomness), then cards are
/// dropped alternately from each half in small groups (1-3 cards). This
/// simulates the imperfect nature of human riffle shuffling.
///
/// Performs 1-2 iterations to simulate multiple riffle passes.
///
/// # Arguments
///
/// * `items` - A mutable reference to the vector to shuffle
fn riffle_shuffle<Item: Clone>(items: &mut Vec<Item>) {
    if items.len() < 2 {
        return;
    }
    let mut rng: ThreadRng = rand::rng();
    let iterations: i32 = rng.random_range(1..=2);
    for _ in 0..iterations {
        let split_point: usize =
            items.len() / 2 + rng.random_range(-2i32..=2).max(-(items.len() as i32 / 2)) as usize;
        let split_point: usize = split_point.clamp(1, items.len() - 1);
        let mut left_half: Vec<Item> = items.drain(..split_point).collect();
        let mut right_half: Vec<Item> = std::mem::take(items);
        items.clear();
        let mut left_idx: usize = 0;
        let mut right_idx: usize = 0;
        while left_idx < left_half.len() || right_idx < right_half.len() {
            let left_remaining: usize = left_half.len() - left_idx;
            let right_remaining: usize = right_half.len() - right_idx;
            if left_remaining == 0 {
                items.extend(right_half.drain(right_idx..));
                break;
            }
            if right_remaining == 0 {
                items.extend(left_half.drain(left_idx..));
                break;
            }
            let drop_count: usize = rng.random_range(1..=3);
            if rng.random_bool(left_remaining as f64 / (left_remaining + right_remaining) as f64) {
                let count: usize = drop_count.min(left_remaining);
                items.extend(left_half.drain(left_idx..left_idx + count));
                left_idx += count;
            } else {
                let count: usize = drop_count.min(right_remaining);
                items.extend(right_half.drain(right_idx..right_idx + count));
                right_idx += count;
            }
        }
    }
}

/// Simulates a simple cut shuffle
///
/// The deck is cut at a random point, and the bottom portion is moved
/// to the top. This is the simplest form of shuffling and provides
/// minimal randomization.
///
/// # Arguments
///
/// * `items` - A mutable reference to the vector to shuffle
fn cut_shuffle<Item: Clone>(items: &mut Vec<Item>) {
    if items.len() < 2 {
        return;
    }
    let cut_point: usize = rand::rng().random_range(1..items.len());
    let bottom_half: Vec<Item> = items.drain(cut_point..).collect();
    let top_half: Vec<Item> = std::mem::take(items);
    items.extend(bottom_half);
    items.extend(top_half);
}

/// Simulates an overhand shuffle
///
/// Small packets (1-5 cards) are repeatedly taken from the back of the deck
/// and placed at the front, reversing their order. This process is repeated
/// 3-5 times to simulate multiple overhand shuffle passes.
///
/// # Arguments
///
/// * `items` - A mutable reference to the vector to shuffle
fn overhand_shuffle<Item: Clone>(items: &mut Vec<Item>) {
    if items.len() < 3 {
        return;
    }
    let mut rng: ThreadRng = rand::rng();
    let iterations: i32 = rng.random_range(3..=5);
    for _ in 0..iterations {
        let mut shuffled: Vec<Item> = Vec::with_capacity(items.len());
        while !items.is_empty() {
            let packet_size: usize = rng.random_range(1..=5.min(items.len()));
            let packet: Vec<Item> = items.drain(items.len() - packet_size..).collect();
            for item in packet.into_iter().rev() {
                shuffled.insert(0, item);
            }
        }
        *items = shuffled;
    }
}

/// Simulates a Hindu shuffle
///
/// Small packets (2-6 cards) are taken from the front of the deck and
/// their order is reversed as they're added to the result. Occasionally,
/// the remaining cards are also reversed and added. This process is
/// repeated 3-6 times.
///
/// # Arguments
///
/// * `items` - A mutable reference to the vector to shuffle
fn hindu_shuffle<Item: Clone>(items: &mut Vec<Item>) {
    if items.len() < 3 {
        return;
    }
    let mut rng: ThreadRng = rand::rng();
    let iterations: i32 = rng.random_range(3..=6);
    for _ in 0..iterations {
        let mut result: Vec<Item> = Vec::with_capacity(items.len());
        while !items.is_empty() {
            let packet_size: usize = rng.random_range(2..=6.min(items.len()));
            let packet: Vec<Item> = items.drain(..packet_size).collect();
            for item in packet.into_iter().rev() {
                result.push(item);
            }
            if !items.is_empty() && rng.random_bool(0.3) {
                result.extend(items.drain(..).rev());
            }
        }
        *items = result;
    }
}
