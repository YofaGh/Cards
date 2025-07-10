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
    let mut rng: ThreadRng = rand::rng();
    let random_iterations: i32 = rng.random_range(1..=3);
    for _ in 0..random_iterations {
        let start: usize = rng.random_range(0..items.len());
        let end: usize = rng.random_range(0..items.len());
        let (start, end) = if end < start {
            (end, start)
        } else {
            (start, end)
        };
        let mut new_items: Vec<Item> = Vec::with_capacity(items.len());
        new_items.extend_from_slice(&items[start..end]);
        new_items.extend_from_slice(&items[..start]);
        new_items.extend_from_slice(&items[end..]);
        *items = new_items;
    }
}

fn cut_shuffle<Item: Clone>(items: &mut Vec<Item>) {
    if items.len() < 2 {
        return;
    }
    let cut_point: usize = rand::rng().random_range(1..items.len());
    let mut new_items: Vec<Item> = Vec::with_capacity(items.len());
    new_items.extend_from_slice(&items[cut_point..]);
    new_items.extend_from_slice(&items[..cut_point]);
    *items = new_items;
}

fn overhand_shuffle<Item>(items: &mut Vec<Item>) {
    if items.len() < 3 {
        return;
    }
    let mut rng: ThreadRng = rand::rng();
    let iterations: i32 = rng.random_range(3..=7);
    for _ in 0..iterations {
        let packet_size: usize = rng.random_range(1..=items.len().min(10));
        let start_pos: usize = rng.random_range(0..=(items.len() - packet_size));
        let packet: Vec<Item> = items.drain(start_pos..start_pos + packet_size).collect();
        let insert_pos: usize = rng.random_range(0..=items.len().min(3));
        for (i, item) in packet.into_iter().enumerate() {
            items.insert(insert_pos + i, item);
        }
    }
}

fn hindu_shuffle<Item>(items: &mut Vec<Item>) {
    if items.len() < 3 {
        return;
    }
    let mut rng: ThreadRng = rand::rng();
    let iterations: i32 = rng.random_range(4..=8);
    for _ in 0..iterations {
        let packet_size: usize = rng.random_range(2..=items.len().min(8));
        let middle_start: usize = items.len() / 3;
        let middle_end: usize = (items.len() * 2) / 3;
        let start_pos: usize =
            rng.random_range(middle_start..=(middle_end - packet_size).max(middle_start));
        let packet: Vec<Item> = items.drain(start_pos..start_pos + packet_size).collect();
        for (i, item) in packet.into_iter().enumerate() {
            items.insert(i, item);
        }
    }
}
