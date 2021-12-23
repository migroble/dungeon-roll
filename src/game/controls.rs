use super::*;
use crossterm::event::{Event, KeyCode};

impl<R: Rng> Game<R> {
    pub fn handle_event(&mut self, event: Event) -> bool {
        if let Event::Key(kc) = event {
            match (kc.code, &self.phase) {
                (KeyCode::Right, _) => self.select_next(),
                (KeyCode::Left, _) => self.select_prev(),
                (KeyCode::Enter, _) => self.next_phase(),
                (KeyCode::Esc, _) => self.prev_phase(),
                (KeyCode::Char('q'), _) => return true,
                (KeyCode::Up, Phase::Monster(MonsterPhase::SelectReroll(Reroll::Ally))) => {
                    self.select_top()
                }
                (KeyCode::Down, Phase::Monster(MonsterPhase::SelectReroll(Reroll::Monster))) => {
                    self.select_bottom()
                }
                (
                    KeyCode::Char(' '),
                    Phase::Monster(MonsterPhase::SelectReroll(_))
                    | Phase::Dragon(DragonPhase::SelectAlly),
                ) => self.toggle_select(),
                _ => return false,
            }
        }

        self.blink = true;

        false
    }

    fn select_next(&mut self) {
        match self.phase {
            Phase::Monster(MonsterPhase::SelectAlly)
            | Phase::Loot(LootPhase::SelectAlly)
            | Phase::Dragon(DragonPhase::SelectAlly) => self.party.next(PartyCursor::Ally as usize),
            Phase::Monster(MonsterPhase::SelectReroll(Reroll::Ally)) => {
                self.party.next(PartyCursor::Reroll as usize)
            }

            Phase::Monster(MonsterPhase::SelectMonster) | Phase::Loot(LootPhase::SelectLoot) => {
                self.dungeon.next(DungeonCursor::Monster as usize)
            }
            Phase::Monster(MonsterPhase::SelectReroll(Reroll::Monster)) => {
                self.dungeon.next(DungeonCursor::Reroll as usize)
            }
            _ => (),
        };
    }

    fn select_prev(&mut self) {
        match self.phase {
            Phase::Monster(MonsterPhase::SelectAlly)
            | Phase::Loot(LootPhase::SelectAlly)
            | Phase::Dragon(DragonPhase::SelectAlly) => self.party.prev(PartyCursor::Ally as usize),
            Phase::Monster(MonsterPhase::SelectReroll(Reroll::Ally)) => {
                self.party.prev(PartyCursor::Reroll as usize)
            }
            Phase::Monster(MonsterPhase::SelectMonster) | Phase::Loot(LootPhase::SelectLoot) => {
                self.dungeon.prev(DungeonCursor::Monster as usize)
            }
            Phase::Monster(MonsterPhase::SelectReroll(Reroll::Monster)) => {
                self.dungeon.prev(DungeonCursor::Reroll as usize)
            }
            _ => (),
        };
    }

    fn toggle_select(&mut self) {
        match self.phase {
            Phase::Monster(MonsterPhase::SelectReroll(ref r)) => match r {
                Reroll::Monster => self.dungeon.toggle_select(DungeonCursor::Reroll as usize),
                Reroll::Ally => self.party.toggle_select(PartyCursor::Reroll as usize),
            },
            Phase::Dragon(DragonPhase::SelectAlly) => {
                self.party.toggle_select(PartyCursor::Ally as usize)
            }
            _ => (),
        }
    }

    fn select_top(&mut self) {
        self.phase = Phase::Monster(MonsterPhase::SelectReroll(Reroll::Monster));
    }

    fn select_bottom(&mut self) {
        self.phase = Phase::Monster(MonsterPhase::SelectReroll(Reroll::Ally));
    }
}
