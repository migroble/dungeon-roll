use crate::{
    game::{utils::*, Game},
    phase::*,
};
use rand::prelude::*;

impl<R: Rng> Game<R> {
    pub fn select_next(&mut self) {
        match self.phase {
            Phase::Monster(MonsterPhase::SelectAlly) | Phase::Loot(LootPhase::SelectAlly) => {
                if self.ally_cursor < self.party.len() - 1 {
                    self.ally_cursor += 1;
                }
            }
            Phase::Monster(MonsterPhase::SelectReroll) => {
                if self.reroll_cursor + 1 == self.ally_cursor
                    && self.reroll_cursor + 2 < self.party.len()
                {
                    self.reroll_cursor += 2;
                } else if self.reroll_cursor + 1 < self.party.len() {
                    self.reroll_cursor += 1;
                }
            }
            Phase::Monster(MonsterPhase::SelectMonster) => {
                if let Some(pos) =
                    find_first_from(&self.dungeon, self.monster_cursor + 1, |m| m.is_monster())
                {
                    self.monster_cursor = pos;
                }
            }
            _ => (),
        };
    }

    pub fn select_prev(&mut self) {
        match self.phase {
            Phase::Monster(MonsterPhase::SelectAlly) | Phase::Loot(LootPhase::SelectAlly) => {
                if self.ally_cursor > 0 {
                    self.ally_cursor -= 1;
                }
            }
            Phase::Monster(MonsterPhase::SelectReroll) => {
                if self.reroll_cursor > 0 {
                    if self.reroll_cursor - 1 == self.ally_cursor && self.reroll_cursor - 1 > 0 {
                        self.reroll_cursor -= 2;
                    } else {
                        self.reroll_cursor -= 1;
                    }
                }
            }
            Phase::Monster(MonsterPhase::SelectMonster) => {
                if let Some(pos) =
                    find_first_before(&self.dungeon, self.monster_cursor, |m| m.is_monster())
                {
                    self.monster_cursor = pos;
                }
            }
            _ => (),
        };
    }
}
