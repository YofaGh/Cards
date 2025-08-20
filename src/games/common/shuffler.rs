#![allow(dead_code)]

use rand::{rngs::ThreadRng, seq::SliceRandom, Rng};
use std::clone::Clone;

pub enum ShuffleMethod {
    Hard,
    Riffle,
    Cut,
    Overhand,
    Hindu,
}

pub fn shuffle<Item: Clone>(items: &mut Vec<Item>, method: ShuffleMethod) {
    match method {
        ShuffleMethod::Hard => hard_shuffle(items),
        ShuffleMethod::Riffle => riffle_shuffle(items),
        ShuffleMethod::Cut => cut_shuffle(items),
        ShuffleMethod::Overhand => overhand_shuffle(items),
        ShuffleMethod::Hindu => hindu_shuffle(items),
    }
}

fn hard_shuffle<Item>(items: &mut [Item]) {
    items.shuffle(&mut rand::rng());
}

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
        let mut right_half: Vec<Item> = items.drain(..).collect();
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

fn cut_shuffle<Item: Clone>(items: &mut Vec<Item>) {
    if items.len() < 2 {
        return;
    }
    let cut_point: usize = rand::rng().random_range(1..items.len());
    let bottom_half: Vec<Item> = items.drain(cut_point..).collect();
    let top_half: Vec<Item> = items.drain(..).collect();
    items.extend(bottom_half);
    items.extend(top_half);
}

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
            if items.len() > 0 && rng.random_bool(0.3) {
                result.extend(items.drain(..).rev());
            }
        }
        *items = result;
    }
}
