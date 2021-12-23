use super::*;
use std::{collections::HashSet, ops::Deref};

pub fn roll<T: Dice, R: Rng>(rng: &mut R) -> T {
    T::nth(rng.gen_range(0..T::faces()))
}

pub fn roll_n<T: Dice, R: Rng>(rng: &mut R, n: u64) -> Vec<T> {
    (0..n).map(|_| roll(rng)).collect()
}

pub fn indexes_of<T: PartialEq>(items: &[T], key: &T) -> Vec<usize> {
    items
        .iter()
        .enumerate()
        .filter_map(|(i, m)| if m == key { Some(i) } else { None })
        .collect()
}

pub enum Row {
    Party,
    Dungeon,
}

impl<R: Rng> Game<R> {
    pub(super) fn current_monster(&self) -> &Monster {
        self.dungeon.value(DungeonCursor::Monster as usize)
    }

    pub(super) fn current_ally(&self) -> &Ally {
        self.party.value(PartyCursor::Ally as usize)
    }

    pub(super) fn selected_row(&self) -> Option<Row> {
        match self.phase {
            Phase::Monster(
                MonsterPhase::SelectMonster | MonsterPhase::SelectReroll(Reroll::Monster),
            )
            | Phase::Loot(LootPhase::SelectLoot) => Some(Row::Dungeon),
            Phase::Monster(MonsterPhase::SelectAlly | MonsterPhase::SelectReroll(Reroll::Ally))
            | Phase::Loot(LootPhase::SelectAlly)
            | Phase::Dragon(DragonPhase::SelectAlly) => Some(Row::Party),
            _ => None,
        }
    }

    pub(super) fn has_monsters(&self) -> bool {
        self.dungeon.iter().any(|m| m.is_monster())
    }

    pub(super) fn has_loot(&self) -> bool {
        self.dungeon
            .iter()
            .any(|m| !m.is_monster() && m != &Monster::Dragon)
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
}

pub type Invariant<T> = fn(&Cursor<T>, usize, &T) -> bool;

pub struct Cursor<T> {
    cursors: Vec<usize>,
    selection: HashSet<usize>,
    invariants: Vec<Invariant<T>>,
    data: Vec<T>,
}

impl<T> Cursor<T> {
    pub fn new(data: Vec<T>, invariants: Vec<Invariant<T>>) -> Self {
        let mut cursor = Self {
            cursors: vec![0, invariants.len()],
            selection: HashSet::new(),
            invariants,
            data,
        };

        cursor.canonicalize();

        cursor
    }

    fn canonicalize(&mut self) {
        if !self.data.is_empty() {
            for i in 0..self.invariants.len() {
                let cursor = self.cursor(i);
                if cursor >= self.data.len() {
                    self.cursors[i] = self
                        .prev_valid(i, self.data.len())
                        .unwrap_or(self.data.len() - 1);
                } else if !self.invariants[i](self, cursor, self.value(i)) {
                    self.cursors[i] = self
                        .next_valid(i, 0)
                        .unwrap_or_else(|| self.prev_valid(i, self.data.len()).unwrap_or(0));
                }
            }
        }
    }

    fn next_valid(&self, index: usize, skip: usize) -> Option<usize> {
        if index >= self.invariants.len() {
            return None;
        }

        self.data
            .iter()
            .enumerate()
            .skip(skip)
            .find(|(i, t)| self.invariants[index](self, *i, t))
            .map(|(i, _)| i)
    }

    fn prev_valid(&self, index: usize, skip: usize) -> Option<usize> {
        if index >= self.invariants.len() {
            return None;
        }

        self.data
            .iter()
            .enumerate()
            .rev()
            .skip(self.data.len() - skip)
            .find(|(i, t)| self.invariants[index](self, *i, t))
            .map(|(i, _)| i)
    }

    pub fn next(&mut self, c: usize) {
        if let Some(idx) = self.next_valid(c, self.cursor(c) + 1) {
            self.cursors[c] = idx;
        }
    }

    pub fn prev(&mut self, c: usize) {
        if let Some(idx) = self.prev_valid(c, self.cursor(c)) {
            self.cursors[c] = idx;
        }
    }

    pub fn cursor(&self, c: usize) -> usize {
        assert!(c < self.cursors.len());
        self.cursors[c]
    }

    pub fn value(&self, c: usize) -> &T {
        &self.data[self.cursors[c]]
    }

    pub fn set_invariants(&mut self, invariants: Vec<Invariant<T>>) {
        self.invariants = invariants;
        self.canonicalize();
    }

    pub fn set_data(&mut self, data: Vec<T>) {
        self.data = data;
        self.canonicalize();
    }

    pub fn set_value(&mut self, index: usize, value: T) {
        assert!(index < self.data.len());
        self.data[index] = value;
    }

    pub fn retain<F: FnMut(&T) -> bool>(&mut self, f: F) {
        self.data.retain(f);
        self.canonicalize();
    }

    pub fn remove(&mut self, i: usize) -> T {
        let ret = self.data.remove(i);
        self.canonicalize();
        ret
    }

    pub fn selection(&self) -> Vec<usize> {
        self.selection.iter().copied().collect()
    }

    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    pub fn toggle_select(&mut self, c: usize) {
        assert!(c < self.cursors.len());
        let index = self.cursor(c);
        if self.selection.contains(&index) {
            self.selection.remove(&index);
        } else {
            self.selection.insert(index);
        }
    }

    pub fn is_selected(&self, i: usize) -> bool {
        self.selection.contains(&i)
    }
}

impl<T> Deref for Cursor<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
