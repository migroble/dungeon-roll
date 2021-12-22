use crate::{
    dice::*,
    game::{utils::*, Game},
    phase::*,
};
use rand::prelude::*;

impl<R: Rng> Game<R> {
    pub fn next_delve(&mut self) {
        self.delve += 1;
        self.party = roll_n(&mut self.rng, 7);
        self.next_level();
        self.phase = Phase::Monster(MonsterPhase::SelectAlly);
    }

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

    fn next_level(&mut self) {
        self.level += 1;
        self.dungeon = roll_n(&mut self.rng, self.level);
        if let Some(n) = find_first_from(&self.dungeon, 0, |m| m.is_monster()) {
            self.monster_cursor = n;
        } else {
            self.phase = Phase::Loot(LootPhase::SelectAlly)
        }
    }

    fn has_monsters(&self) -> bool {
        self.dungeon.iter().any(|m| m.is_monster())
    }

    pub(super) fn affects_all(&self) -> bool {
        matches!(
            (self.current_ally(), self.current_monster(),),
            (Ally::Fighter, Monster::Goblin)
                | (Ally::Cleric, Monster::Skeleton)
                | (Ally::Mage, Monster::Ooze)
                | (Ally::Champion, _)
                | (Ally::Thief, Monster::Chest)
                | (_, Monster::Potion)
        )
    }

    fn execute_combat(&mut self) {
        let kill_all = self.affects_all();
        if kill_all {
            let monster = self.current_monster().clone();
            self.dungeon.retain(|m| m != &monster);
        } else {
            self.dungeon.remove(self.monster_cursor);
        }

        let ally = self.party.remove(self.ally_cursor);
        self.graveyard.push(ally);
    }

    fn execute_reroll(&mut self) {
        self.party[self.reroll_cursor] = roll(&mut self.rng);
        self.party.remove(self.ally_cursor);
    }

    pub fn next_phase(&mut self) {
        self.phase = match self.phase {
            Phase::Monster(ref mp) => match mp {
                MonsterPhase::SelectAlly => {
                    if self.current_ally() == &Ally::Scroll {
                        if self.reroll_cursor == self.ally_cursor {
                            if self.ally_cursor == 0 {
                                self.reroll_cursor = 1;
                            } else {
                                self.reroll_cursor -= 1;
                            }
                        }
                        Phase::Monster(MonsterPhase::SelectReroll)
                    } else {
                        Phase::Monster(MonsterPhase::SelectMonster)
                    }
                }
                MonsterPhase::SelectReroll => Phase::Monster(MonsterPhase::ConfirmReroll),
                MonsterPhase::ConfirmReroll => {
                    self.execute_reroll();
                    Phase::Monster(MonsterPhase::SelectAlly)
                }
                MonsterPhase::SelectMonster => Phase::Monster(MonsterPhase::ConfirmCombat),
                MonsterPhase::ConfirmCombat => {
                    self.execute_combat();
                    if self.has_monsters() {
                        Phase::Monster(MonsterPhase::SelectAlly)
                    } else {
                        Phase::Loot(LootPhase::SelectAlly)
                    }
                }
            },
            Phase::Loot(ref lp) => match lp {
                LootPhase::SelectAlly => Phase::Monster(MonsterPhase::SelectAlly),
                LootPhase::SelectLoot => match self.current_monster() {
                    &Monster::Chest => Phase::Loot(LootPhase::ConfirmLoot),
                    &Monster::Potion => Phase::Loot(LootPhase::SelectGraveyard),
                    _ => unreachable!(),
                },
                LootPhase::ConfirmLoot => {
                    self.inventory.push(
                        self.treasure
                            .remove(self.rng.gen_range(0..self.treasure.len())),
                    );
                    Phase::Dragon
                }
                LootPhase::SelectGraveyard => Phase::Loot(LootPhase::ConfirmGraveyard),
                LootPhase::ConfirmGraveyard => Phase::Dragon,
            },
            Phase::Regroup => {
                self.next_level();
                Phase::Monster(MonsterPhase::SelectAlly)
            }
            _ => unreachable!(),
        };

        self.monster_cursor = self.monster_cursor.min(self.dungeon.len() - 1);
        self.ally_cursor = self.ally_cursor.min(self.party.len() - 1);
        self.reroll_cursor = self.reroll_cursor.min(self.party.len() - 1);
    }

    pub fn prev_phase(&mut self) {
        self.phase = match self.phase {
            Phase::Monster(ref mp) => match mp {
                MonsterPhase::SelectAlly => Phase::Monster(MonsterPhase::SelectAlly),
                MonsterPhase::SelectReroll => Phase::Monster(MonsterPhase::SelectAlly),
                MonsterPhase::ConfirmReroll => Phase::Monster(MonsterPhase::SelectReroll),
                MonsterPhase::SelectMonster => Phase::Monster(MonsterPhase::SelectAlly),
                MonsterPhase::ConfirmCombat => Phase::Monster(MonsterPhase::SelectMonster),
            },
            Phase::Loot(ref lp) => match lp {
                LootPhase::SelectAlly => Phase::Loot(LootPhase::SelectAlly),
                LootPhase::SelectLoot => Phase::Loot(LootPhase::SelectAlly),
                LootPhase::ConfirmLoot => Phase::Loot(LootPhase::SelectLoot),
                LootPhase::SelectGraveyard => Phase::Loot(LootPhase::SelectLoot),
                LootPhase::ConfirmGraveyard => Phase::Loot(LootPhase::SelectGraveyard),
            },
            _ => unreachable!(),
        };
    }
}
