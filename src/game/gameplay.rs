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
    }

    pub fn select_next(&mut self) {
        match self.phase {
            Phase::Monster(MonsterPhase::SelectAlly) | Phase::Loot(LootPhase::SelectAlly) => {
                if self.selection_ally < self.party.len() - 1 {
                    self.selection_ally += 1;
                }
            }
            Phase::Monster(MonsterPhase::SelectReroll) => {
                if self.selection_reroll + 1 == self.selection_ally
                    && self.selection_reroll + 2 < self.party.len()
                {
                    self.selection_reroll += 2;
                } else if self.selection_reroll + 1 < self.party.len() {
                    self.selection_reroll += 1;
                }
            }
            Phase::Monster(MonsterPhase::SelectMonster) => {
                if let Some(pos) = find_first_from(&self.dungeon, self.selection_monster + 1, |m| {
                    m.is_monster()
                }) {
                    self.selection_monster = pos;
                }
            }
            _ => (),
        };
    }

    pub fn select_prev(&mut self) {
        match self.phase {
            Phase::Monster(MonsterPhase::SelectAlly) | Phase::Loot(LootPhase::SelectAlly) => {
                if self.selection_ally > 0 {
                    self.selection_ally -= 1;
                }
            }
            Phase::Monster(MonsterPhase::SelectReroll) => {
                if self.selection_reroll > 0 {
                    if self.selection_reroll - 1 == self.selection_ally
                        && self.selection_reroll - 1 > 0
                    {
                        self.selection_reroll -= 2;
                    } else {
                        self.selection_reroll -= 1;
                    }
                }
            }
            Phase::Monster(MonsterPhase::SelectMonster) => {
                if let Some(pos) =
                    find_first_before(&self.dungeon, self.selection_monster, |m| m.is_monster())
                {
                    self.selection_monster = pos;
                }
            }
            _ => (),
        };
    }

    fn next_level(&mut self) {
        self.level += 1;
        self.dungeon = roll_n(&mut self.rng, self.level);
        if let Some(n) = find_first_from(&self.dungeon, 0, |m| m.is_monster()) {
            self.selection_monster = n;
        } else {
            self.phase = Phase::Loot(LootPhase::SelectAlly)
        }
    }

    fn has_monsters(&self) -> bool {
        self.dungeon.iter().any(|m| m.is_monster())
    }

    pub(super) fn affects_all(&self) -> bool {
        matches!(
            (
                &self.party[self.selection_ally],
                &self.dungeon[self.selection_monster],
            ),
            (&Ally::Fighter, &Monster::Goblin)
                | (&Ally::Cleric, &Monster::Skeleton)
                | (&Ally::Mage, &Monster::Ooze)
                | (&Ally::Champion, _)
                | (&Ally::Thief, &Monster::Chest)
                | (_, &Monster::Potion)
        )
    }

    fn execute_combat(&mut self) {
        let kill_all = self.affects_all();
        if kill_all {
            let monster = self.dungeon[self.selection_monster].clone();
            self.dungeon.retain(|m| m != &monster);
        } else {
            self.dungeon.remove(self.selection_monster);
        }

        let ally = self.party.remove(self.selection_ally);
        self.graveyard.push(ally);
    }

    fn execute_reroll(&mut self) {
        self.party[self.selection_reroll] = roll(&mut self.rng);
        self.party.remove(self.selection_ally);
    }

    pub fn next_phase(&mut self) {
        self.phase = match self.phase {
            Phase::Monster(ref mp) => match mp {
                MonsterPhase::SelectAlly => {
                    if self.party[self.selection_ally] == Ally::Scroll {
                        if self.selection_reroll == self.selection_ally {
                            if self.selection_ally == 0 {
                                self.selection_reroll = 1;
                            } else {
                                self.selection_reroll -= 1;
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
            Phase::Loot(LootPhase::SelectAlly) => Phase::Monster(MonsterPhase::SelectAlly),
            _ => unreachable!(),
        };

        self.selection_monster = self.selection_monster.min(self.dungeon.len() - 1);
        self.selection_ally = self.selection_ally.min(self.party.len() - 1);
        self.selection_reroll = self.selection_reroll.min(self.party.len() - 1);
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
            Phase::Loot(LootPhase::SelectAlly) => Phase::Loot(LootPhase::SelectAlly),
            _ => unreachable!(),
        };
    }
}
