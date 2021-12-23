use super::*;

impl<R: Rng> Game<R> {
    pub(super) fn next_delve(&mut self) {
        self.delve += 1;
        self.party.set_data(roll_n(&mut self.rng, 7));
        self.next_level();
    }

    fn next_level(&mut self) {
        self.level += 1;
        self.dungeon.set_data(roll_n(&mut self.rng, self.level));
        if !self.has_monsters() {
            self.phase = Phase::Loot(LootPhase::SelectAlly)
        }
    }

    fn has_monsters(&self) -> bool {
        self.dungeon.iter().any(|m| m.is_monster())
    }

    pub(super) fn affects_all(&self) -> bool {
        matches!(
            (self.current_ally(), self.current_monster()),
            (Ally::Fighter, Monster::Goblin)
                | (Ally::Cleric, Monster::Skeleton)
                | (Ally::Mage, Monster::Ooze)
                | (Ally::Champion, _)
                | (Ally::Thief, Monster::Chest)
                | (_, Monster::Potion)
        )
    }

    fn execute_combat(&mut self) {
        if self.affects_all() {
            let monster = self.current_monster().clone();
            self.dungeon.retain(|m| m != &monster);
        } else {
            let idx = self.dungeon.cursor(DungeonCursor::Monster as usize);
            self.dungeon.remove(idx);
        }

        let idx = self.party.cursor(PartyCursor::Ally as usize);
        let ally = self.party.remove(idx);
        self.graveyard.push(ally);
    }

    fn execute_reroll(&mut self) {
        // TODO: re-do this
        // self.party[self.reroll_ally_cursor] = roll(&mut self.rng);
        // self.party.remove(self.ally_cursor);
    }

    pub(super) fn next_phase(&mut self) {
        self.phase = match self.phase {
            Phase::Monster(ref mp) => match mp {
                MonsterPhase::SelectAlly => {
                    if self.current_ally() == &Ally::Scroll {
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
                LootPhase::SelectAlly => Phase::Loot(LootPhase::SelectLoot),
                LootPhase::SelectLoot => match self.current_monster() {
                    Monster::Chest => Phase::Loot(LootPhase::ConfirmLoot),
                    Monster::Potion => Phase::Loot(LootPhase::SelectGraveyard),
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
    }

    pub(super) fn prev_phase(&mut self) {
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
