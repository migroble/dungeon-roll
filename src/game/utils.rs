use crate::{dice::*, game::Game};
use rand::prelude::*;
use std::{
    collections::HashSet,
    ops::{Deref, DerefMut},
};

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

impl<R: Rng> Game<R> {
    pub(super) fn current_monster(&self) -> &Monster {
        self.dungeon.value(0)
    }

    pub(super) fn current_ally(&self) -> &Ally {
        self.party.value(1)
    }

    pub(super) fn current_ally_reroll(&self) -> &Ally {
        self.party.value(0)
    }
}

pub type Invariant<T> = fn(&Cursor<T>, usize, &T) -> bool;

pub struct Cursor<T> {
    curr_cursor: usize,
    cursors: Vec<usize>,
    invariants: Vec<Invariant<T>>,
    data: Vec<T>,
}

impl<T> Cursor<T> {
    pub fn new(data: Vec<T>, invariants: Vec<Invariant<T>>) -> Self {
        let mut cursor = Self {
            curr_cursor: 0,
            cursors: vec![0, invariants.len()],
            invariants,
            data,
        };

        cursor.canonicalize();

        cursor
    }

    pub fn canonicalize(&mut self) {
        if self.data.len() > 0 {
            for i in 0..self.invariants.len() {
                let cursor = self.cursor(i);
                if cursor >= self.data.len() {
                    self.cursors[i] = self
                        .prev_valid(i, self.data.len())
                        .unwrap_or(self.data.len() - 1);
                } else if !self.invariants[i](&self, cursor, &self.value(i)) {
                    self.cursors[i] = self.next_valid(i, 0).unwrap_or(0);
                }
            }
        }
    }

    fn next_valid(&self, index: usize, skip: usize) -> Option<usize> {
        assert!(index < self.invariants.len());
        self.data
            .iter()
            .enumerate()
            .skip(skip)
            .find(|(i, t)| self.invariants[index](&self, *i, t))
            .map(|(i, _)| i)
    }

    fn prev_valid(&self, index: usize, skip: usize) -> Option<usize> {
        assert!(index < self.invariants.len());
        self.data
            .iter()
            .enumerate()
            .rev()
            .skip(self.data.len() - skip)
            .find(|(i, t)| self.invariants[index](&self, *i, t))
            .map(|(i, _)| i)
    }

    pub fn next(&mut self) {
        if let Some(idx) = self.next_valid(self.curr_cursor, self.curr_index() + 1) {
            *self.curr_index_mut() = idx;
        }
    }

    pub fn prev(&mut self) {
        if let Some(idx) = self.prev_valid(self.curr_cursor, self.curr_index()) {
            *self.curr_index_mut() = idx;
        }
    }

    pub fn set_cursor(&mut self, i: usize) {
        assert!(i < self.cursors.len());
        self.curr_cursor = i;
    }

    pub fn cursor(&self, c: usize) -> usize {
        assert!(c < self.cursors.len());
        self.cursors[c]
    }

    pub fn value(&self, c: usize) -> &T {
        &self.data[self.cursors[c]]
    }

    fn curr_index(&self) -> usize {
        self.cursors[self.curr_cursor]
    }

    fn curr_index_mut(&mut self) -> &mut usize {
        &mut self.cursors[self.curr_cursor]
    }
}

impl<T> Deref for Cursor<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for Cursor<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

enum SelectedRow {
    Top,
    Bottom,
}

impl Default for SelectedRow {
    fn default() -> Self {
        Self::Bottom
    }
}

#[derive(Default)]
pub struct Selection {
    selected: SelectedRow,
    top: HashSet<usize>,
    bottom: HashSet<usize>,
    top_cursor: usize,
    bottom_cursor: usize,
}

impl Selection {
    pub fn is_selected_top(&self, i: usize) -> bool {
        self.top.contains(&i)
    }

    pub fn is_selected_bottom(&self, i: usize) -> bool {
        self.bottom.contains(&i)
    }

    pub fn select_top(&mut self, i: usize) {
        self.selected = SelectedRow::Top;
    }

    pub fn select_bottom(&mut self, i: usize) {
        self.selected = SelectedRow::Bottom;
    }

    pub fn move_cursor(&mut self, i: usize) {
        match self.selected {
            SelectedRow::Top => self.top_cursor = i,
            SelectedRow::Bottom => self.bottom_cursor = i,
        }
    }
}
