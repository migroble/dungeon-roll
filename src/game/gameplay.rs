use super::{
    roll, roll_n, Ally, DragonPhase, DungeonCursor, Game, LootPhase, Monster, MonsterPhase,
    PartyCursor, Phase, RegroupPhase, Reroll, Rng, DRAGON_ALLY_INV, LOOT_ALLY_INV,
    LOOT_DUNGEON_INV, LOOT_SCROLL_DUNGEON_INV, MON_ALLY_INV, MON_DUNGEON_INV, TREASURE,
};

impl<R: Rng> Game<R> {
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_precision_loss)]
    #[allow(clippy::cast_possible_truncation)]
    pub(super) fn next_delve(&mut self) {
        self.delve += 1;
        self.level = 0;
        self.run_xp = 0;
        self.no_monsters = false;
        self.party_size = (1. + 1.5 * (self.hero.xp() as f64 + 4.).sqrt()) as u64;
        self.phase = Phase::Setup;
        self.party.set_data(roll_n(&mut self.rng, self.party_size));
        self.graveyard.set_data(Vec::new());
        self.treasure = TREASURE.clone();
        self.inventory.clear();
        self.next_phase();
    }

    fn next_level(&mut self) {
        self.level += 1;
        self.run_xp += self.level;
        self.dungeon.retain(|m| m == &Monster::Dragon);
        self.dungeon.append(roll_n(&mut self.rng, self.level));
        self.no_monsters = !self.has_monsters();
    }

    fn kill_monster(&mut self) {
        let idx = self.dungeon.cursor(DungeonCursor::Monster as usize);
        self.dungeon.remove(idx);
    }

    fn kill_ally(&mut self) {
        let idx = self.party.cursor(PartyCursor::Ally as usize);
        let ally = self.party.remove(idx);
        self.graveyard.push(ally);
    }

    fn execute_combat(&mut self) {
        if self.affects_all() {
            let monster = self.current_monster().clone();
            self.dungeon.retain(|m| m != &monster);
        } else {
            self.kill_monster();
        }

        self.kill_ally();
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

        self.kill_ally();
    }

    fn execute_loot(&mut self) {
        self.inventory.push(
            self.treasure
                .remove(self.rng.gen_range(0..self.treasure.len())),
        );

        self.kill_monster();
        self.kill_ally();

        self.run_xp += 1;
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

        self.kill_ally();
    }

    fn execute_dragon(&mut self) {
        self.party.selection().iter().for_each(|s| {
            self.party.remove(*s);
        });
        self.party.clear_selection();
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
                    if !self.has_loot() || self.graveyard.is_empty() && !self.has_chest() {
                        if self.no_monsters {
                            self.phase = Phase::EmptyDungeon;
                        } else {
                            self.phase = Phase::Dragon(DragonPhase::SelectAlly);
                            return true;
                        }
                    }

                    self.party.set_invariants(LOOT_ALLY_INV.to_vec());
                }
                LootPhase::SelectLoot => {
                    self.dungeon.clear_selection();
                    self.party.clear_selection();
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
                    self.phase = Phase::Regroup(RegroupPhase::Continue);
                    return true;
                }
            }
            Phase::Regroup(RegroupPhase::ContinueSetup) => {
                self.phase = Phase::Setup;
                return true;
            }
            Phase::Regroup(RegroupPhase::EndSetup) => {
                self.hero.add_xp(self.run_xp);
                self.next_delve();
                return true;
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
            Phase::Setup => Phase::Setup,
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
            Phase::Dragon(DragonPhase::SelectAlly) => {
                if self.party.selection().len() == 3 {
                    Phase::Dragon(DragonPhase::Confirm)
                } else {
                    Phase::Dragon(DragonPhase::SelectAlly)
                }
            }
            Phase::Dragon(DragonPhase::Confirm) | Phase::EmptyDungeon => {
                Phase::Regroup(RegroupPhase::Continue)
            }
            Phase::Regroup(RegroupPhase::Continue | RegroupPhase::ContinueSetup) => {
                Phase::Regroup(RegroupPhase::ContinueSetup)
            }
            Phase::Regroup(RegroupPhase::End | RegroupPhase::EndSetup) => {
                Phase::Regroup(RegroupPhase::EndSetup)
            }
            Phase::Defeat => Phase::Defeat,
            Phase::Victory => Phase::Victory,
        };
        while self.enter_phase_trigger() {}
    }

    pub(super) fn prev_phase(&mut self) {
        if let Some(p) = match self.phase {
            Phase::Monster(ref mp) => match mp {
                MonsterPhase::SelectAlly => None,
                MonsterPhase::SelectReroll(_) | MonsterPhase::SelectMonster => {
                    Some(Phase::Monster(MonsterPhase::SelectAlly))
                }
                MonsterPhase::ConfirmReroll => {
                    Some(Phase::Monster(MonsterPhase::SelectReroll(Reroll::Ally)))
                }
                MonsterPhase::ConfirmCombat => Some(Phase::Monster(MonsterPhase::SelectMonster)),
            },
            Phase::Loot(ref lp) => match lp {
                LootPhase::SelectAlly => Some(Phase::Dragon(DragonPhase::SelectAlly)),
                LootPhase::SelectLoot => Some(Phase::Loot(LootPhase::SelectAlly)),
                LootPhase::ConfirmLoot | LootPhase::SelectGraveyard => {
                    Some(Phase::Loot(LootPhase::SelectLoot))
                }
                LootPhase::ConfirmGraveyard => Some(Phase::Loot(LootPhase::SelectGraveyard)),
            },
            Phase::Dragon(_) => Some(Phase::Dragon(DragonPhase::SelectAlly)),
            _ => None,
        } {
            self.phase = p;
            while self.enter_phase_trigger() {}
        }
    }
}
