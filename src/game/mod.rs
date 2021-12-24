use crate::{
    dice::{Ally, Dice, Monster, Render},
    hero::{Hero, Type},
    phase::{
        Dragon as DragonPhase, Loot as LootPhase, Monster as MonsterPhase, Phase,
        Regroup as RegroupPhase, Reroll,
    },
    treasure::{Treasure, TREASURE},
};
use rand::prelude::*;

mod controls;
mod gameplay;
mod render;
mod utils;

use utils::{indexes_of, roll, roll_n, Cursor, Invariant, Row};

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
        |_, _, t| t != &Monster::Dragon, // monster cursor
    ];
    // scrolls cant open chests
    static ref LOOT_SCROLL_DUNGEON_INV: Vec<Invariant<Monster>> = vec![
        |_, _, t| t != &Monster::Chest && t != &Monster::Dragon, // monster cursor
    ];
    static ref DRAGON_ALLY_INV: Vec<Invariant<Ally>> = vec![
        |_, _, t| t.is_companion(), // ally cursor
    ];
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

pub struct Game<R: Rng> {
    rng: R,
    blink: bool,
    delve: u64,
    level: u64,
    run_xp: u64,
    party_size: u64,
    no_monsters: bool,
    no_chests: bool,
    no_loot: bool,
    phase: Phase,
    hero: Hero,
    party: Cursor<Ally>,
    dungeon: Cursor<Monster>,
    graveyard: Cursor<Ally>,
    treasure: Vec<Treasure>,
    inventory: Vec<Treasure>,
}

impl<R: Rng> Game<R> {
    pub fn new(rng: R, hero: Type) -> Self {
        let mut game = Self {
            rng,
            blink: true,
            delve: 0,
            level: 0,
            run_xp: 0,
            no_monsters: false,
            no_chests: false,
            no_loot: false,
            party_size: 0,
            phase: Phase::Setup,
            hero: Hero::new(hero),
            party: Cursor::new(Vec::new(), MON_ALLY_INV.to_vec()),
            graveyard: Cursor::new(Vec::new(), vec![|_, _, _| true]),
            dungeon: Cursor::new(Vec::new(), MON_DUNGEON_INV.to_vec()),
            treasure: TREASURE.clone(),
            inventory: Vec::new(),
        };

        game.next_delve();

        game
    }

    pub fn toggle_blink(&mut self) {
        self.blink = !self.blink;
    }
}
