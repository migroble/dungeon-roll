use super::*;
use crossterm::event::{Event, KeyCode};

impl<R: Rng> Game<R> {
    pub fn handle_event(&mut self, event: Event) -> bool {
        if let Event::Key(kc) = event {
            match (kc.code, &self.phase) {
                (KeyCode::Right, _) => self.select_next(),
                (KeyCode::Left, _) => self.select_prev(),
                (KeyCode::Up, Phase::Monster(MonsterPhase::SelectReroll)) => self.select_top(),
                (KeyCode::Down, Phase::Monster(MonsterPhase::SelectReroll)) => self.select_bottom(),
                (KeyCode::Char(' '), Phase::Monster(MonsterPhase::SelectReroll)) => self.select(),
                (KeyCode::Enter, _) => self.next_phase(),
                (KeyCode::Backspace, _) => self.prev_phase(),
                (KeyCode::Esc, _) => return true,
                _ => return false,
            }
        }

        self.blink = true;

        false
    }

    fn select_next(&mut self) {
        match self.phase {
            Phase::Monster(MonsterPhase::SelectAlly) | Phase::Loot(LootPhase::SelectAlly) => {
                self.party.next(PartyCursor::Ally as usize)
            }
            Phase::Monster(MonsterPhase::SelectReroll) => {
                self.party.next(PartyCursor::Reroll as usize)
            }
            Phase::Monster(MonsterPhase::SelectMonster) => {
                self.dungeon.next(DungeonCursor::Monster as usize)
            }
            _ => (),
        };
    }

    fn select_prev(&mut self) {
        match self.phase {
            Phase::Monster(MonsterPhase::SelectAlly) | Phase::Loot(LootPhase::SelectAlly) => {
                self.party.prev(PartyCursor::Ally as usize)
            }
            Phase::Monster(MonsterPhase::SelectReroll) => {
                self.party.prev(PartyCursor::Reroll as usize)
            }
            Phase::Monster(MonsterPhase::SelectMonster) => {
                self.dungeon.prev(DungeonCursor::Monster as usize)
            }
            _ => (),
        };
    }

    fn select(&mut self) {
        let cursor = if self.selection_row == 0 {
            self.party.cursor(PartyCursor::Reroll as usize)
        } else {
            self.dungeon.cursor(DungeonCursor::Reroll as usize)
        };

        let row = &mut self.selection[self.selection_row];
        if row.contains(&cursor) {
            row.remove(&cursor);
        } else {
            row.insert(cursor);
        }
    }

    fn select_top(&mut self) {
        self.selection_row = 1;
    }

    fn select_bottom(&mut self) {
        self.selection_row = 0;
    }
}
