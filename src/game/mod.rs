use crate::{dice::*, hero::*, phase::*, treasure::*};
use rand::prelude::*;

mod controls;
mod gameplay;
mod render;
mod utils;

#[derive(Debug)]
pub struct Game<R: Rng> {
    rng: R,
    blink: bool,
    delve: u64,
    level: u64,
    phase: Phase,
    hero: Hero,
    party: Vec<Ally>,
    graveyard: Vec<Ally>,
    dungeon: Vec<Monster>,
    treasure: Vec<Treasure>,
    inventory: Vec<Treasure>,
    dragon_lair: usize,
    ally_cursor: usize,
    reroll_cursor: usize,
    monster_cursor: usize,
}

impl<R: Rng> Game<R> {
    pub fn new(rng: R, hero: HeroType) -> Self {
        Self {
            rng,
            blink: true,
            delve: 0,
            level: 5,
            phase: Phase::Setup,
            hero: Hero::new(hero),
            party: Vec::new(),
            graveyard: Vec::new(),
            dungeon: Vec::new(),
            treasure: TREASURE.clone(),
            inventory: Vec::new(),
            dragon_lair: 0,
            ally_cursor: 0,
            reroll_cursor: 0,
            monster_cursor: 0,
        }
    }

    pub fn toggle_blink(&mut self) {
        self.blink = !self.blink;
    }
}
