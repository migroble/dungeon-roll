use super::{
    roll, roll_n, Ally, DragonPhase, DungeonCursor, Game, LootPhase, Monster, MonsterPhase,
    PartyCursor, Phase, Reroll, Rng, DRAGON_ALLY_INV, LOOT_ALLY_INV, LOOT_DUNGEON_INV,
    LOOT_SCROLL_DUNGEON_INV, MON_ALLY_INV, MON_DUNGEON_INV,
};

impl<R: Rng> Game<R> {
    pub(super) fn next_delve(&mut self) {
        self.delve += 1;
        self.party.set_data(roll_n(&mut self.rng, 7));
        self.next_phase();
    }

    fn next_level(&mut self) {
        self.level += 1;
        self.dungeon.set_data(roll_n(&mut self.rng, self.level));
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

        let idx = self.party.cursor(PartyCursor::Ally as usize);
        let ally = self.party.remove(idx);
        self.graveyard.push(ally);
    }

    fn execute_loot(&mut self) {
        self.inventory.push(
            self.treasure
                .remove(self.rng.gen_range(0..self.treasure.len())),
        );
    }

    fn execute_graveyard(&mut self) {
        let sel = self.graveyard.selection();
        let revives = sel.len();

        sel.iter()
            .for_each(|s| self.party.push(self.graveyard.remove(*s)));
        self.graveyard.clear_selection();

        let mut i = 0;
        self.dungeon.retain(|m| {
            if m == &Monster::Potion && i < revives {
                i += 1;
                false
            } else {
                true
            }
        });

        let idx = self.party.cursor(PartyCursor::Ally as usize);
        let ally = self.party.remove(idx);
        self.graveyard.push(ally);
    }

    fn execute_dragon(&mut self) {
        self.dungeon.retain(|m| m != &Monster::Dragon);
    }

    fn enter_phase_trigger(&mut self) -> bool {
        match self.phase {
            Phase::Setup => {
                self.next_level();
                self.phase = Phase::Monster(MonsterPhase::SelectAlly);
                return true;
            }
            Phase::Monster(MonsterPhase::SelectAlly) => {
                if !self.has_monsters() {
                    self.phase = Phase::Loot(LootPhase::SelectAlly);
                    return true;
                }

                self.party.set_invariants(MON_ALLY_INV.to_vec());
                self.dungeon.set_invariants(MON_DUNGEON_INV.to_vec());
            }
            Phase::Loot(ref lp) => match lp {
                LootPhase::SelectAlly => {
                    if !self.has_loot() {
                        self.phase = Phase::Dragon(DragonPhase::SelectAlly);
                        return true;
                    }

                    self.party.set_invariants(LOOT_ALLY_INV.to_vec());
                }
                LootPhase::SelectLoot => {
                    if let Ally::Scroll = self.current_ally() {
                        self.dungeon
                            .set_invariants(LOOT_SCROLL_DUNGEON_INV.to_vec());
                    } else {
                        self.dungeon.set_invariants(LOOT_DUNGEON_INV.to_vec());
                    }
                }
                LootPhase::SelectGraveyard => {
                    self.graveyard.set_selection_limit(self.potion_count());
                }
                _ => (),
            },
            Phase::Dragon(DragonPhase::SelectAlly) => {
                if self.dragon_dice() >= 3 {
                    self.party.set_invariants(DRAGON_ALLY_INV.to_vec());
                    self.party.set_selection_limit(3);
                } else {
                    self.phase = Phase::Regroup;
                    return true;
                }
            }
            _ => (),
        }

        false
    }

    fn exit_phase_trigger(&mut self) {
        match self.phase {
            Phase::Monster(MonsterPhase::ConfirmReroll) => self.execute_reroll(),
            Phase::Monster(MonsterPhase::ConfirmCombat) => self.execute_combat(),
            Phase::Loot(LootPhase::ConfirmLoot) => self.execute_loot(),
            Phase::Loot(LootPhase::ConfirmGraveyard) => {
                self.execute_graveyard();
                self.party.set_selection_limit(0);
            }
            Phase::Dragon(DragonPhase::Confirm) => {
                self.execute_dragon();
                self.party.set_selection_limit(0);
            }
            _ => (),
        }
    }

    pub(super) fn next_phase(&mut self) {
        self.exit_phase_trigger();
        self.phase = match self.phase {
            Phase::Setup | Phase::Regroup => Phase::Setup,
            Phase::Monster(ref mp) => match mp {
                MonsterPhase::SelectAlly => {
                    if self.current_ally() == &Ally::Scroll {
                        Phase::Monster(MonsterPhase::SelectReroll(Reroll::Ally))
                    } else {
                        Phase::Monster(MonsterPhase::SelectMonster)
                    }
                }
                MonsterPhase::SelectReroll(_) => Phase::Monster(MonsterPhase::ConfirmReroll),
                MonsterPhase::ConfirmReroll | MonsterPhase::ConfirmCombat => {
                    Phase::Monster(MonsterPhase::SelectAlly)
                }
                MonsterPhase::SelectMonster => Phase::Monster(MonsterPhase::ConfirmCombat),
            },
            Phase::Loot(ref lp) => match lp {
                LootPhase::SelectAlly => {
                    if self.current_ally() == &Ally::Scroll && self.potion_count() == 0 {
                        Phase::Loot(LootPhase::SelectAlly)
                    } else {
                        Phase::Loot(LootPhase::SelectLoot)
                    }
                }
                LootPhase::SelectLoot => match self.current_monster() {
                    Monster::Chest => Phase::Loot(LootPhase::ConfirmLoot),
                    Monster::Potion => Phase::Loot(LootPhase::SelectGraveyard),
                    _ => Phase::Loot(LootPhase::SelectLoot),
                },
                LootPhase::ConfirmLoot | LootPhase::ConfirmGraveyard => {
                    Phase::Loot(LootPhase::SelectAlly)
                }
                LootPhase::SelectGraveyard => Phase::Loot(LootPhase::ConfirmGraveyard),
            },
            Phase::Dragon(DragonPhase::SelectAlly) => Phase::Dragon(DragonPhase::Confirm),
            Phase::Dragon(DragonPhase::Confirm) => Phase::Regroup,
        };
        while self.enter_phase_trigger() {}
    }

    pub(super) fn prev_phase(&mut self) {
        self.phase = match self.phase {
            Phase::Monster(ref mp) => match mp {
                MonsterPhase::SelectAlly
                | MonsterPhase::SelectReroll(_)
                | MonsterPhase::SelectMonster => Phase::Monster(MonsterPhase::SelectAlly),
                MonsterPhase::ConfirmReroll => {
                    Phase::Monster(MonsterPhase::SelectReroll(Reroll::Ally))
                }
                MonsterPhase::ConfirmCombat => Phase::Monster(MonsterPhase::SelectMonster),
            },
            Phase::Loot(ref lp) => match lp {
                LootPhase::SelectAlly => Phase::Dragon(DragonPhase::SelectAlly),
                LootPhase::SelectLoot => Phase::Loot(LootPhase::SelectAlly),
                LootPhase::ConfirmLoot | LootPhase::SelectGraveyard => {
                    Phase::Loot(LootPhase::SelectLoot)
                }
                LootPhase::ConfirmGraveyard => Phase::Loot(LootPhase::SelectGraveyard),
            },
            Phase::Dragon(_) => Phase::Dragon(DragonPhase::SelectAlly),
            _ => unreachable!(),
        };
        while self.enter_phase_trigger() {}
    }
}
