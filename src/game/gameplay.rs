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
            self.phase = Phase::Monster(MonsterPhase::ConfirmCombat);
            self.next_phase();
        }
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
        self.party
            .selection()
            .iter()
            .for_each(|s| self.party.set_value(*s, roll(&mut self.rng)));
        self.party.clear_selection();

        self.dungeon
            .selection()
            .iter()
            .for_each(|s| self.dungeon.set_value(*s, roll(&mut self.rng)));
        self.dungeon.clear_selection();
    }

    fn end_loot_phase(&mut self) -> Phase {
        if self.has_loot() {
            Phase::Loot(LootPhase::SelectAlly)
        } else {
            self.dragon_phase()
        }
    }

    fn dragon_phase(&mut self) -> Phase {
        self.party.set_invariants(DRAGON_ALLY_INV.to_vec());
        Phase::Dragon(DragonPhase::SelectAlly)
    }

    pub(super) fn next_phase(&mut self) {
        self.phase = match self.phase {
            Phase::Monster(ref mp) => match mp {
                MonsterPhase::SelectAlly => {
                    if self.current_ally() == &Ally::Scroll {
                        Phase::Monster(MonsterPhase::SelectReroll(Reroll::Ally))
                    } else {
                        Phase::Monster(MonsterPhase::SelectMonster)
                    }
                }
                MonsterPhase::SelectReroll(_) => Phase::Monster(MonsterPhase::ConfirmReroll),
                MonsterPhase::ConfirmReroll => {
                    self.execute_reroll();
                    if self.has_monsters() {
                        Phase::Monster(MonsterPhase::SelectAlly)
                    } else {
                        self.party.set_invariants(LOOT_ALLY_INV.to_vec());
                        Phase::Loot(LootPhase::SelectAlly)
                    }
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
                LootPhase::SelectAlly => {
                    if let Ally::Scroll = self.current_ally() {
                        self.dungeon
                            .set_invariants(LOOT_SCROLL_DUNGEON_INV.to_vec());
                    } else {
                        self.dungeon.set_invariants(LOOT_DUNGEON_INV.to_vec());
                    }
                    Phase::Loot(LootPhase::SelectLoot)
                }
                LootPhase::SelectLoot => match self.current_monster() {
                    Monster::Chest => Phase::Loot(LootPhase::ConfirmLoot),
                    Monster::Potion => Phase::Loot(LootPhase::SelectGraveyard),
                    _ => Phase::Loot(LootPhase::SelectLoot),
                },
                LootPhase::ConfirmLoot => {
                    self.inventory.push(
                        self.treasure
                            .remove(self.rng.gen_range(0..self.treasure.len())),
                    );
                    self.end_loot_phase()
                }
                LootPhase::SelectGraveyard => Phase::Loot(LootPhase::ConfirmGraveyard),
                LootPhase::ConfirmGraveyard => self.end_loot_phase(),
            },
            Phase::Regroup => {
                self.next_level();
                self.dungeon.set_invariants(MON_DUNGEON_INV.to_vec());
                self.party.set_invariants(MON_ALLY_INV.to_vec());
                Phase::Monster(MonsterPhase::SelectAlly)
            }
            _ => unreachable!(),
        };
    }

    pub(super) fn prev_phase(&mut self) {
        self.phase = match self.phase {
            Phase::Monster(ref mp) => match mp {
                MonsterPhase::SelectAlly => Phase::Monster(MonsterPhase::SelectAlly),
                MonsterPhase::SelectReroll(_) => Phase::Monster(MonsterPhase::SelectAlly),
                MonsterPhase::ConfirmReroll => {
                    Phase::Monster(MonsterPhase::SelectReroll(Reroll::Ally))
                }
                MonsterPhase::SelectMonster => Phase::Monster(MonsterPhase::SelectAlly),
                MonsterPhase::ConfirmCombat => Phase::Monster(MonsterPhase::SelectMonster),
            },
            Phase::Loot(ref lp) => match lp {
                LootPhase::SelectAlly => self.dragon_phase(),
                LootPhase::SelectLoot => Phase::Loot(LootPhase::SelectAlly),
                LootPhase::ConfirmLoot => Phase::Loot(LootPhase::SelectLoot),
                LootPhase::SelectGraveyard => Phase::Loot(LootPhase::SelectLoot),
                LootPhase::ConfirmGraveyard => Phase::Loot(LootPhase::SelectGraveyard),
            },
            Phase::Dragon(_) => Phase::Dragon(DragonPhase::SelectAlly),
            _ => unreachable!(),
        };
    }
}
