use super::*;
use std::{io, iter::repeat};
use tui::layout::Rect;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    terminal::{CompletedFrame, Frame},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

impl<R: Rng> Game<R> {
    fn render_array<B: Backend, T: Render>(
        &self,
        f: &mut Frame<B>,
        area: Rect,
        data: &[T],
        style_fn: fn(&Game<R>, usize) -> Style,
    ) {
        let row = Layout::default()
            .direction(Direction::Horizontal)
            .horizontal_margin(1)
            .vertical_margin(2)
            .constraints(
                repeat(Constraint::Ratio(1, data.len() as u32))
                    .take(data.len())
                    .collect::<Vec<_>>(),
            )
            .split(area);

        row.iter().zip(data).enumerate().for_each(|(i, (c, t))| {
            let style = style_fn(self, i);
            let mut sprite = t.render();
            sprite.patch_style(style);
            f.render_widget(Paragraph::new(sprite).alignment(Alignment::Center), *c)
        });
    }

    fn render_monster_phase<B: Backend>(
        &self,
        f: &mut Frame<B>,
        area: Rect,
        subphase: &MonsterPhase,
    ) {
        f.render_widget(Block::default().borders(Borders::ALL), area);

        let middle = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)].as_ref())
            .split(area);

        fn monster_style<R: Rng>(game: &Game<R>, i: usize) -> Style {
            let equal_monsters = indexes_of(&game.dungeon, game.current_monster());
            let is_affected = |i: usize| game.affects_all() && equal_monsters.contains(&i);
            let is_selected = |i: usize| i == game.dungeon.cursor(DungeonCursor::Monster as usize);
            let is_reroll_selected =
                |i: usize| i == game.dungeon.cursor(DungeonCursor::Reroll as usize);

            let style = Style::default();
            match game.phase {
                Phase::Monster(ref mp) => match mp {
                    MonsterPhase::SelectMonster if game.blink && is_selected(i) => {
                        style.bg(Color::White)
                    }
                    MonsterPhase::SelectReroll(Reroll::Monster)
                        if game.blink && is_reroll_selected(i) =>
                    {
                        style.bg(Color::White)
                    }
                    MonsterPhase::SelectMonster if !is_selected(i) && is_affected(i) => {
                        style.bg(Color::DarkGray)
                    }
                    MonsterPhase::SelectReroll(_) if game.dungeon.is_selected(i) => {
                        style.bg(Color::DarkGray)
                    }
                    MonsterPhase::ConfirmReroll if game.dungeon.is_selected(i) => {
                        style.bg(Color::DarkGray).add_modifier(Modifier::DIM)
                    }
                    MonsterPhase::ConfirmCombat if is_affected(i) => {
                        style.bg(Color::DarkGray).add_modifier(Modifier::DIM)
                    }
                    _ => style.bg(Color::Black),
                },
                _ => unreachable!(),
            }
        }

        fn ally_style<R: Rng>(game: &Game<R>, i: usize) -> Style {
            let equal_monsters = indexes_of(&game.dungeon, game.current_monster());
            let is_selected = |i: usize| i == game.party.cursor(PartyCursor::Ally as usize);
            let is_reroll_selected =
                |i: usize| i == game.party.cursor(PartyCursor::Reroll as usize);

            let style = Style::default();
            match game.phase {
                Phase::Monster(ref mp) => match mp {
                    MonsterPhase::SelectAlly if game.blink && is_selected(i) => {
                        style.bg(Color::White)
                    }
                    MonsterPhase::SelectReroll(Reroll::Ally)
                        if game.blink && is_reroll_selected(i) =>
                    {
                        style.bg(Color::White)
                    }
                    MonsterPhase::SelectReroll(_) if game.party.is_selected(i) => {
                        style.bg(Color::DarkGray)
                    }
                    MonsterPhase::ConfirmReroll if game.party.is_selected(i) => {
                        style.bg(Color::DarkGray).add_modifier(Modifier::DIM)
                    }
                    _ if mp != &MonsterPhase::SelectAlly && is_selected(i) => {
                        style.bg(Color::DarkGray).add_modifier(Modifier::DIM)
                    }
                    _ => style.bg(Color::Black),
                },
                _ => unreachable!(),
            }
        }

        f.render_widget(Block::default().borders(Borders::ALL), middle[0]);
        f.render_widget(Block::default().borders(Borders::ALL), middle[1]);

        self.render_array(f, middle[0], &*self.dungeon, monster_style);
        self.render_array(f, middle[1], &*self.party, ally_style);
    }

    pub fn render<'a, B: Backend>(
        &self,
        terminal: &'a mut Terminal<B>,
    ) -> Result<CompletedFrame<'a>, io::Error> {
        let size = terminal.size()?;

        terminal.draw(|f| {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .horizontal_margin(1)
                .vertical_margin(size.height.checked_sub(24).unwrap_or(0) / 2)
                .constraints([Constraint::Length(12), Constraint::Length(9)])
                .split(f.size());
            let sublayout = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([
                    Constraint::Percentage(20),
                    Constraint::Percentage(60),
                    Constraint::Percentage(20),
                ])
                .split(layout[0]);

            let info = sublayout[0];
            let playfield = sublayout[1];
            let graveyard = sublayout[2];

            let controls = layout[1];

            f.render_widget(
                Block::default().title("layout 0").borders(Borders::ALL),
                layout[0],
            );
            f.render_widget(
                Block::default().title("controls").borders(Borders::ALL),
                controls,
            );

            #[allow(clippy::single_match)]
            match &self.phase {
                Phase::Monster(sp) => self.render_monster_phase(f, playfield, sp),
                _ => (),
            }
        })
    }
}
