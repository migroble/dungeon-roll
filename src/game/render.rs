use super::*;
use std::{io, iter::repeat, ops::ControlFlow};
use tui::layout::Rect;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    terminal::{CompletedFrame, Frame},
    widgets::{Block, BorderType, Borders, Paragraph},
    Terminal,
};

fn vertical_center(area: Rect) -> Rect {
    if area.height > 0 {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length((area.height - 1) / 2),
                Constraint::Length(1),
                Constraint::Percentage(100),
            ])
            .split(area)[1]
    } else {
        area
    }
}

fn draw_block<B: Backend>(f: &mut Frame<B>, block: Block, area: Rect) -> Rect {
    let inner = block.inner(area);
    f.render_widget(block, area);
    inner
}

impl<R: Rng> Game<R> {
    fn render_controls<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let mut controls = vec!["→: Next", "←: Previous"];
        match self.phase {
            Phase::Monster(ref mp) => match mp {
                MonsterPhase::SelectReroll(Reroll::Ally) => controls.append(&mut vec![
                    "↑: Dungeon row",
                    "Space: Select",
                    "Enter: Confirm",
                    "Esc: Back",
                ]),
                MonsterPhase::ConfirmCombat => controls = vec!["Enter: Confirm", "Esc: Back"],
                MonsterPhase::SelectReroll(Reroll::Monster) => controls.append(&mut vec![
                    "↓: Party row",
                    "Space: Select",
                    "Enter: Confirm",
                    "Esc: Back",
                ]),
                MonsterPhase::ConfirmReroll => controls = vec!["Enter: Confirm", "Esc: Back"],
                _ => controls.append(&mut vec!["Enter: Select", "Esc: Back"]),
            },
            Phase::Loot(ref lp) => controls.append(&mut vec![
                "Enter: Confirm",
                match lp {
                    LootPhase::SelectAlly => "Esc: Skip loot",
                    _ => "Esc: Back",
                },
            ]),
            Phase::Dragon => controls.append(&mut vec!["Space: Select", "Enter: Confirm"]),
            Phase::Regroup => (),
            _ => unreachable!(),
        }
        controls.push("Q: Exit");

        let text_area = draw_block(f, Block::default().borders(Borders::ALL), area);
        let rows = 2;
        let columns = (controls.len() + 1) / 2;
        let column = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                repeat(Constraint::Ratio(1, rows as u32))
                    .take(rows)
                    .collect::<Vec<_>>(),
            )
            .split(text_area);

        column.iter().enumerate().for_each(|(i, col)| {
            let ratio = if i == rows - 1 {
                controls.len() - i * columns
            } else {
                columns
            };

            let row = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    repeat(Constraint::Ratio(1, ratio as u32))
                        .take(columns)
                        .collect::<Vec<_>>(),
                )
                .split(*col);

            row.iter().enumerate().try_for_each(|(j, cell)| {
                let index = i * columns + j;
                if index >= controls.len() {
                    return ControlFlow::Break(());
                }

                let r = vertical_center(*cell);
                f.render_widget(
                    Paragraph::new(controls[index]).alignment(Alignment::Center),
                    r,
                );

                ControlFlow::Continue(())
            });
        })
    }

    fn render_array<B: Backend, T: Render>(
        &self,
        f: &mut Frame<B>,
        area: Rect,
        data: &[T],
        style_fn: fn(&Game<R>, usize) -> Style,
    ) {
        let row = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints(
                repeat(Constraint::Ratio(1, data.len() as u32))
                    .take(data.len())
                    .collect::<Vec<_>>(),
            )
            .split(area);

        row.iter().zip(data).enumerate().for_each(|(i, (col, t))| {
            let style = style_fn(self, i);
            let mut sprite = t.render();
            sprite.patch_style(style);
            let r = vertical_center(*col);
            f.render_widget(Paragraph::new(sprite).alignment(Alignment::Center), r)
        });
    }

    fn render_playfield<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let middle = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)].as_ref())
            .split(area);

        let (dungeon_style, party_style) = match &self.phase {
            Phase::Monster(_) => (
                monster_phase_style::dungeon_cursor_style,
                monster_phase_style::party_cursor_style,
            ),
            _ => unreachable!(),
        };

        let (dungeon_border, party_border) = match self.phase {
            Phase::Monster(ref mp) => match mp {
                MonsterPhase::SelectAlly | MonsterPhase::SelectReroll(Reroll::Ally) => {
                    (BorderType::Plain, BorderType::Thick)
                }
                MonsterPhase::SelectMonster | MonsterPhase::SelectReroll(Reroll::Monster) => {
                    (BorderType::Thick, BorderType::Plain)
                }
                _ => (BorderType::Plain, BorderType::Plain),
            },
            _ => unreachable!(),
        };

        let dungeon_area = draw_block(
            f,
            Block::default()
                .title(" Dungeon ")
                .border_type(dungeon_border)
                .borders(Borders::ALL),
            middle[0],
        );
        let party_area = draw_block(
            f,
            Block::default()
                .title(" Party ")
                .border_type(party_border)
                .borders(Borders::ALL),
            middle[1],
        );

        self.render_array(f, dungeon_area, &*self.dungeon, dungeon_style);
        self.render_array(f, party_area, &*self.party, party_style);
    }

    pub fn render<'a, B: Backend>(
        &self,
        terminal: &'a mut Terminal<B>,
    ) -> Result<CompletedFrame<'a>, io::Error> {
        let size = terminal.size()?;

        terminal.draw(|f| {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Ratio(3, 4), Constraint::Ratio(1, 4)])
                .split(f.size());

            let game = draw_block(f, Block::default().borders(Borders::ALL), layout[0]);

            let sublayout = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([
                    Constraint::Percentage(20),
                    Constraint::Percentage(60),
                    Constraint::Percentage(20),
                ])
                .split(game);

            let info = sublayout[0];
            let graveyard = sublayout[2];

            let playfield = sublayout[1];
            self.render_playfield(f, playfield);

            let controls = layout[1];
            self.render_controls(f, controls);
        })
    }
}

mod monster_phase_style {
    use super::*;

    pub fn dungeon_cursor_style<R: Rng>(game: &Game<R>, i: usize) -> Style {
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

    pub fn party_cursor_style<R: Rng>(game: &Game<R>, i: usize) -> Style {
        let equal_monsters = indexes_of(&game.dungeon, game.current_monster());
        let is_selected = |i: usize| i == game.party.cursor(PartyCursor::Ally as usize);
        let is_reroll_selected = |i: usize| i == game.party.cursor(PartyCursor::Reroll as usize);

        let style = Style::default();
        match game.phase {
            Phase::Monster(ref mp) => match mp {
                MonsterPhase::SelectAlly if game.blink && is_selected(i) => style.bg(Color::White),
                MonsterPhase::SelectReroll(Reroll::Ally) if game.blink && is_reroll_selected(i) => {
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
}
