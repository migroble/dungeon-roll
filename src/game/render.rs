use crate::{
    game::{utils::*, Game},
    phase::*,
};
use rand::prelude::*;
use std::{io, iter::repeat};
use tui::layout::Rect;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    terminal::{CompletedFrame, Frame},
    text::Text,
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};

impl<R: Rng> Game<R> {
    fn render_monster_phase<B: Backend>(
        &self,
        f: &mut Frame<B>,
        window: Rect,
        subphase: &MonsterPhase,
    ) {
        let middle = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(window);

        let monster_row = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints(
                repeat(Constraint::Ratio(1, self.dungeon.len() as u32))
                    .take(self.dungeon.len())
                    .collect::<Vec<_>>(),
            )
            .split(middle[0]);

        let equal_monsters = indexes_of(&self.dungeon, &self.current_monster());
        let is_affected = |i: usize| self.affects_all() && equal_monsters.contains(&i);
        let is_selected = |i: usize| i == self.monster_cursor;
        monster_row
            .iter()
            .zip(&self.dungeon)
            .enumerate()
            .for_each(|(i, (c, m))| {
                let style = Style::default();
                let style = match subphase {
                    MonsterPhase::SelectMonster if self.blink && is_selected(i) => {
                        style.bg(Color::White)
                    }
                    MonsterPhase::SelectMonster if !is_selected(i) && is_affected(i) => {
                        style.bg(Color::DarkGray)
                    }
                    MonsterPhase::ConfirmCombat if is_affected(i) => {
                        style.bg(Color::DarkGray).add_modifier(Modifier::DIM)
                    }
                    _ => style.bg(Color::Black),
                };

                let mut sprite = m.render();
                sprite.patch_style(style);
                f.render_widget(
                    Paragraph::new(sprite)
                        .block(Block::default().borders(Borders::ALL))
                        .alignment(Alignment::Center),
                    *c,
                )
            });

        let party_row = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints(
                repeat(Constraint::Ratio(1, self.party.len() as u32))
                    .take(self.party.len())
                    .collect::<Vec<_>>(),
            )
            .split(middle[1]);

        let is_selected = |i: usize| i == self.ally_cursor;
        let is_reroll_selected = |i: usize| i == self.reroll_cursor;
        party_row
            .iter()
            .zip(&self.party)
            .enumerate()
            .for_each(|(i, (c, p))| {
                let style = Style::default();
                let style = match subphase {
                    MonsterPhase::SelectAlly if self.blink && is_selected(i) => {
                        style.bg(Color::White)
                    }
                    MonsterPhase::SelectReroll if self.blink && is_reroll_selected(i) => {
                        style.bg(Color::White)
                    }
                    MonsterPhase::ConfirmReroll if is_reroll_selected(i) => {
                        style.bg(Color::DarkGray).add_modifier(Modifier::DIM)
                    }
                    _ if subphase != &MonsterPhase::SelectAlly && is_selected(i) => {
                        style.bg(Color::DarkGray).add_modifier(Modifier::DIM)
                    }
                    _ => style.bg(Color::Black),
                };

                let mut sprite = p.render();
                sprite.patch_style(style);
                f.render_widget(
                    Paragraph::new(sprite)
                        .block(Block::default().borders(Borders::ALL))
                        .alignment(Alignment::Center),
                    *c,
                )
            });
    }

    pub fn render<'a, B: Backend>(
        &self,
        terminal: &'a mut Terminal<B>,
    ) -> Result<CompletedFrame<'a>, io::Error> {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Length(5), Constraint::Percentage(100)].as_ref())
                .split(f.size());

            #[allow(clippy::single_match)]
            match &self.phase {
                Phase::Monster(sp) => self.render_monster_phase(f, chunks[1], sp),
                _ => (),
            }

            let paragraph = Paragraph::new(Text::from("\nWelcome to the dungeon!"))
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(Color::White).bg(Color::Black))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });

            f.render_widget(paragraph, chunks[0]);
            f.render_widget(Block::default().borders(Borders::ALL), chunks[1]);
        })
    }
}
