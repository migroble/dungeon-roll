use crate::{dice::*, hero::*, phase::*, treasure::*};
use rand::prelude::*;

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
    dragon_lair: usize,
    selection_ally: usize,
    selection_reroll: usize,
    selection_monster: usize,
}

impl<R: Rng> Game<R> {
    pub fn new(rng: R, hero: HeroType) -> Self {
        Self {
            rng,
            blink: true,
            delve: 0,
            level: 5,
            phase: Phase::Monster(MonsterPhase::SelectAlly),
            hero: Hero::new(hero),
            party: Vec::new(),
            graveyard: Vec::new(),
            dungeon: Vec::new(),
            treasure: TREASURE.clone(),
            dragon_lair: 0,
            selection_ally: 0,
            selection_reroll: 0,
            selection_monster: 0,
        }
    }

    pub fn set_blink(&mut self) {
        self.blink = true;
    }

    pub fn toggle_blink(&mut self) {
        self.blink = !self.blink;
    }
}
