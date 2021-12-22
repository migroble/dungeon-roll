use crate::{dice::*, game::Game};
use rand::prelude::*;

pub fn roll<T: Dice, R: Rng>(rng: &mut R) -> T {
    T::nth(rng.gen_range(0..T::faces()))
}

pub fn roll_n<T: Dice, R: Rng>(rng: &mut R, n: u64) -> Vec<T> {
    (0..n).map(|_| roll(rng)).collect()
}

pub fn find_first_from<T, F: Fn(&T) -> bool>(items: &[T], n: usize, f: F) -> Option<usize> {
    items
        .iter()
        .enumerate()
        .skip(n)
        .find(|(_, item)| f(item))
        .map(|(i, _)| i)
}

pub fn find_first_before<T, F: Fn(&T) -> bool>(items: &[T], n: usize, f: F) -> Option<usize> {
    items
        .iter()
        .enumerate()
        .rev()
        .skip(items.len() - n)
        .find(|(_, item)| f(item))
        .map(|(i, _)| i)
}

pub fn indexes_of<T: PartialEq>(items: &[T], key: &T) -> Vec<usize> {
    items
        .iter()
        .enumerate()
        .filter_map(|(i, m)| if m == key { Some(i) } else { None })
        .collect()
}

impl<R: Rng> Game<R> {
    pub(super) fn current_monster(&self) -> &Monster {
        &self.dungeon[self.monster_cursor]
    }

    pub(super) fn current_ally(&self) -> &Ally {
        &self.party[self.ally_cursor]
    }

    pub(super) fn current_reroll(&self) -> &Ally {
        &self.party[self.reroll_cursor]
    }
}
