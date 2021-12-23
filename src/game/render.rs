use super::{
    indexes_of, DragonPhase, DungeonCursor, Game, LootPhase, MonsterPhase, PartyCursor, Phase,
    Render, Reroll, Rng, Row,
};
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

static DRAGON: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/dragon.txt"));

fn vertical_center(area: Rect, height: u16) -> Rect {
    if area.height > 0 {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(area.height.saturating_sub(height) / 2),
                Constraint::Length(height),
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

fn render_dragon<B: Backend>(f: &mut Frame<B>, area: Rect) {
    let display_area = draw_block(
        f,
        Block::default().title(" Dungeon ").borders(Borders::ALL),
        area,
    );
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(100)])
        .split(display_area);
    let dragon_area = vertical_center(chunks[1], DRAGON.lines().count().try_into().unwrap_or(0));

    f.render_widget(Paragraph::new(DRAGON), dragon_area);
}

fn render_list<B: Backend>(
    f: &mut Frame<B>,
    block: Block,
    area: Rect,
    data: Vec<&str>,
    rows: usize,
) {
    let text_area = draw_block(f, block, area);
    let columns = (data.len() + 1) / rows;
    let column = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            repeat(Constraint::Ratio(1, rows.try_into().unwrap_or(1)))
                .take(rows)
                .collect::<Vec<_>>(),
        )
        .split(text_area);

    column.iter().enumerate().for_each(|(i, col)| {
        let ratio = if i == rows - 1 {
            data.len() - i * columns
        } else {
            columns
        };

        let row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                repeat(Constraint::Ratio(1, ratio.try_into().unwrap_or(1)))
                    .take(columns)
                    .collect::<Vec<_>>(),
            )
            .split(*col);

        row.iter().enumerate().try_for_each(|(j, cell)| {
            let index = i * columns + j;
            if index >= data.len() {
                return ControlFlow::Break(());
            }

            let r = vertical_center(*cell, 1);
            f.render_widget(Paragraph::new(data[index]).alignment(Alignment::Center), r);

            ControlFlow::Continue(())
        });
    });
}

impl<R: Rng> Game<R> {
    #[allow(clippy::non_ascii_literal)]
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
                MonsterPhase::ConfirmCombat | MonsterPhase::ConfirmReroll => {
                    controls = vec!["Enter: Confirm", "Esc: Back"];
                }
                MonsterPhase::SelectReroll(Reroll::Monster) => controls.append(&mut vec![
                    "↓: Party row",
                    "Space: Select",
                    "Enter: Confirm",
                    "Esc: Back",
                ]),
                _ => controls.append(&mut vec!["Enter: Select", "Esc: Back"]),
            },
            Phase::Loot(ref lp) => controls.append(&mut vec![
                "Enter: Confirm",
                match lp {
                    LootPhase::SelectAlly => "Esc: Skip loot",
                    _ => "Esc: Back",
                },
            ]),
            Phase::Dragon(DragonPhase::SelectAlly) => {
                controls.append(&mut vec!["Space: Select", "Enter: Confirm"]);
            }
            Phase::Dragon(DragonPhase::Confirm) => {
                controls.append(&mut vec!["Enter: Confirm", "Esc: Back"]);
            }
            Phase::Regroup => (),
            _ => unreachable!(),
        }
        controls.push("Q: Exit");

        render_list(f, Block::default().borders(Borders::ALL), area, controls, 2);
    }

    fn render_array<B: Backend, T: Render>(
        &self,
        f: &mut Frame<B>,
        area: Rect,
        row: Row,
        name: &'static str,
        data: &[T],
        style_fn: fn(&Game<R>, usize) -> Style,
    ) {
        let border = if Some(row) == self.selected_row() {
            BorderType::Thick
        } else {
            BorderType::Plain
        };

        let array_area = draw_block(
            f,
            Block::default()
                .title(name)
                .border_type(border)
                .borders(Borders::ALL),
            area,
        );

        let row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                repeat(Constraint::Ratio(1, data.len().try_into().unwrap_or(1)))
                    .take(data.len())
                    .collect::<Vec<_>>(),
            )
            .split(array_area);

        row.iter().zip(data).enumerate().for_each(|(i, (col, t))| {
            let style = style_fn(self, i);
            let mut sprite = t.render();
            sprite.patch_style(style);

            let r = vertical_center(*col, 1);
            f.render_widget(Paragraph::new(sprite).alignment(Alignment::Center), r);
        });
    }

    fn render_playfield<B: Backend, S: Styler>(&self, f: &mut Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Ratio(2, 5),
                    Constraint::Ratio(2, 5),
                    Constraint::Ratio(1, 5),
                ]
                .as_ref(),
            )
            .split(area);
        let subchunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)].as_ref())
            .split(chunks[2]);

        match self.phase {
            Phase::Dragon(_) => render_dragon(f, chunks[0]),
            Phase::Regroup => (),
            _ => self.render_array(
                f,
                chunks[0],
                Row::Dungeon,
                " Dungeon ",
                &*self.dungeon,
                S::dungeon_style,
            ),
        }
        self.render_array(
            f,
            chunks[1],
            Row::Party,
            " Party ",
            &*self.party,
            S::party_style,
        );
        self.render_array(
            f,
            subchunks[0],
            Row::Graveyard,
            " Graveyard ",
            &self.graveyard,
            S::graveyard_style,
        );

        render_list(
            f,
            Block::default().title(" Inventory ").borders(Borders::ALL),
            subchunks[1],
            vec![
                &format!("XP: {}", self.hero.xp()),
                &format!("Loot: {}", self.inventory.len()),
            ],
            1,
        );
    }

    pub fn render<'a, B: Backend>(
        &self,
        terminal: &'a mut Terminal<B>,
    ) -> Result<CompletedFrame<'a>, io::Error> {
        terminal.draw(|f| {
            if self.phase == Phase::Setup {
                return;
            }

            let game = draw_block(f, Block::default().borders(Borders::ALL), f.size());

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Ratio(3, 4), Constraint::Ratio(1, 4)])
                .split(game);

            let sublayout = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                .split(layout[0]);

            let info = sublayout[1];
            draw_block(
                f,
                Block::default().title(" Info ").borders(Borders::ALL),
                info,
            );

            let playfield = sublayout[0];
            match &self.phase {
                Phase::Monster(_) => self.render_playfield::<_, MPStyler>(f, playfield),
                Phase::Loot(_) => self.render_playfield::<_, LPStyler>(f, playfield),
                Phase::Dragon(_) => self.render_playfield::<_, DPStyler>(f, playfield),
                _ => (),
            };

            let controls = layout[1];
            self.render_controls(f, controls);
        })
    }
}

trait Styler {
    fn dungeon_style<R: Rng>(_game: &Game<R>, _i: usize) -> Style {
        Style::default()
    }

    fn party_style<R: Rng>(_game: &Game<R>, _i: usize) -> Style {
        Style::default()
    }

    fn graveyard_style<R: Rng>(_game: &Game<R>, _i: usize) -> Style {
        Style::default()
    }
}

struct MPStyler;
impl Styler for MPStyler {
    fn dungeon_style<R: Rng>(game: &Game<R>, i: usize) -> Style {
        let equal_monsters = indexes_of(&game.dungeon, game.current_monster());
        let is_selected = |i: usize| i == game.dungeon.cursor(DungeonCursor::Monster as usize);
        let is_affected =
            |i: usize| game.affects_all() && equal_monsters.contains(&i) || is_selected(i);
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

    fn party_style<R: Rng>(game: &Game<R>, i: usize) -> Style {
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

struct LPStyler;
impl Styler for LPStyler {
    fn dungeon_style<R: Rng>(game: &Game<R>, i: usize) -> Style {
        let cursor = game.dungeon.cursor(DungeonCursor::Monster as usize);
        let mut equal_monsters = indexes_of(&game.dungeon, game.current_monster());
        equal_monsters.retain(|i| *i != cursor);
        equal_monsters.truncate(game.graveyard.selection().len().saturating_sub(1));

        let is_selected = |i: usize| i == cursor;
        let is_affected =
            |i: usize| game.affects_all() && equal_monsters.contains(&i) || is_selected(i);

        let style = Style::default();
        match game.phase {
            Phase::Loot(ref mp) => match mp {
                LootPhase::SelectLoot if game.blink && is_selected(i) => style.bg(Color::White),
                LootPhase::SelectLoot if is_affected(i) => style.bg(Color::DarkGray),
                LootPhase::ConfirmLoot if is_affected(i) => {
                    style.bg(Color::DarkGray).add_modifier(Modifier::DIM)
                }
                LootPhase::SelectGraveyard if is_affected(i) => style.bg(Color::DarkGray),
                LootPhase::ConfirmGraveyard if is_affected(i) => {
                    style.bg(Color::DarkGray).add_modifier(Modifier::DIM)
                }
                _ => style.bg(Color::Black),
            },
            _ => unreachable!(),
        }
    }

    fn party_style<R: Rng>(game: &Game<R>, i: usize) -> Style {
        let is_selected = |i: usize| i == game.party.cursor(PartyCursor::Ally as usize);

        let style = Style::default();
        match game.phase {
            Phase::Loot(ref lp) => match lp {
                LootPhase::SelectAlly if game.blink && is_selected(i) => style.bg(Color::White),
                LootPhase::ConfirmLoot if game.party.is_selected(i) => {
                    style.bg(Color::DarkGray).add_modifier(Modifier::DIM)
                }
                LootPhase::SelectGraveyard if is_selected(i) => style.bg(Color::DarkGray),
                LootPhase::ConfirmGraveyard if is_selected(i) => {
                    style.bg(Color::DarkGray).add_modifier(Modifier::DIM)
                }
                _ => style.bg(Color::Black),
            },
            _ => unreachable!(),
        }
    }

    fn graveyard_style<R: Rng>(game: &Game<R>, i: usize) -> Style {
        let is_selected = |i: usize| i == game.graveyard.cursor(0);

        let style = Style::default();
        match game.phase {
            Phase::Loot(ref lp) => match lp {
                LootPhase::SelectGraveyard if game.blink && is_selected(i) => {
                    style.bg(Color::White)
                }
                LootPhase::SelectGraveyard if game.graveyard.is_selected(i) => {
                    style.bg(Color::DarkGray)
                }
                LootPhase::ConfirmGraveyard if game.graveyard.is_selected(i) => {
                    style.bg(Color::DarkGray).add_modifier(Modifier::DIM)
                }
                _ => style.bg(Color::Black),
            },
            _ => unreachable!(),
        }
    }
}

struct DPStyler;
impl Styler for DPStyler {
    fn party_style<R: Rng>(game: &Game<R>, i: usize) -> Style {
        let is_selected = |i: usize| i == game.party.cursor(PartyCursor::Ally as usize);

        let style = Style::default();
        match game.phase {
            Phase::Dragon(ref dp) => match dp {
                DragonPhase::SelectAlly if game.blink && is_selected(i) => style.bg(Color::White),
                DragonPhase::SelectAlly if game.party.is_selected(i) => style.bg(Color::DarkGray),
                DragonPhase::Confirm if game.party.is_selected(i) => {
                    style.bg(Color::DarkGray).add_modifier(Modifier::DIM)
                }
                _ => style.bg(Color::Black),
            },
            _ => unreachable!(),
        }
    }
}
