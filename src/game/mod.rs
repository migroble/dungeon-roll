use crate::{dice::*, hero::*, phase::*, treasure::*};
use rand::prelude::*;

mod controls;
mod gameplay;
mod render;
mod utils;

use utils::*;

pub struct Game<R: Rng> {
    rng: R,
    blink: bool,
    delve: u64,
    level: u64,
    phase: Phase,
    hero: Hero,
    party: Cursor<Ally>,
    dungeon: Cursor<Monster>,
    graveyard: Vec<Ally>,
    treasure: Vec<Treasure>,
    inventory: Vec<Treasure>,
}

#[derive(Copy, Clone)]
enum PartyCursor {
    Ally = 0,
    Reroll = 1,
}

#[derive(Copy, Clone)]
enum DungeonCursor {
    Monster = 0,
    Reroll = 1,
}

lazy_static! {
    static ref MON_ALLY_INV: Vec<Invariant<Ally>> = vec![
        |_, _, _| true,                                      // ally cursor
        |c, i, _| i != c.cursor(PartyCursor::Ally as usize), // reroll ally cursor
    ];
    static ref MON_DUNGEON_INV: Vec<Invariant<Monster>> = vec![
        |_, _, t| t.is_monster(),        // monster cursor
        |_, _, t| t != &Monster::Dragon, // reroll monster cursor
    ];

    static ref LOOT_ALLY_INV: Vec<Invariant<Ally>> = vec![
        |_, _, _| true,                                      // ally cursor
        |c, i, _| i != c.cursor(PartyCursor::Ally as usize), // reroll ally cursor
    ];
    static ref LOOT_DUNGEON_INV: Vec<Invariant<Monster>> = vec![
        |_, _, t| t != &Monster::Dragon,
    ];
    // scrolls cant open chests
    static ref LOOT_SCROLL_DUNGEON_INV: Vec<Invariant<Monster>> = vec![
        |_, _, t| t != &Monster::Chest && t != &Monster::Dragon,
    ];
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
            party: Cursor::new(Vec::new(), MON_ALLY_INV.to_vec()),
            graveyard: Vec::new(),
            dungeon: Cursor::new(Vec::new(), MON_DUNGEON_INV.to_vec()),
            treasure: TREASURE.clone(),
            inventory: Vec::new(),
        }
    }

    pub fn start(&mut self) {
        self.phase = Phase::Monster(MonsterPhase::SelectAlly);
        self.next_delve();
    }

    pub fn toggle_blink(&mut self) {
        self.blink = !self.blink;
    }
}
