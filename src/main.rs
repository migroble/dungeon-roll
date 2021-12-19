#![allow(dead_code)]
#![allow(unused_variables)]

use crossterm::{
    event::{Event, EventStream, KeyCode},
    terminal::enable_raw_mode,
};
use futures::StreamExt;
use rand::prelude::*;
use rand_pcg::Pcg64Mcg;
use std::{collections::HashMap, hash::Hash, io, iter::repeat, time::Duration};
use tokio::time::sleep;
use tui::layout::Rect;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    terminal::{CompletedFrame, Frame},
    text::Text,
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};

#[macro_use]
extern crate dice_derive;
#[macro_use]
extern crate lazy_static;

trait Dice {
    fn nth(n: u64) -> Self;
    fn faces() -> u64;
}

#[derive(Debug, Dice, PartialEq, Eq, Hash)]
enum Ally {
    Fighter,
    Cleric,
    Mage,
    Champion,
    Thief,
    Scroll,
}

impl Ally {
    fn render(&self) -> Text {
        match self {
            Ally::Fighter => Text::styled("F", Style::default().fg(Color::Green)),
            Ally::Cleric => Text::styled("C", Style::default().fg(Color::Gray)),
            Ally::Mage => Text::styled("M", Style::default().fg(Color::Blue)),
            Ally::Champion => Text::styled("C", Style::default().fg(Color::Yellow)),
            Ally::Thief => Text::styled("T", Style::default().fg(Color::Rgb(128, 0, 128))),
            Ally::Scroll => Text::styled("S", Style::default().fg(Color::Rgb(255, 165, 0))),
        }
    }
}

#[derive(Debug, Dice, PartialEq, Eq, Hash)]
enum Monster {
    Goblin,
    Skeleton,
    Ooze,
    Dragon,
    Chest,
    Potion,
}

impl Monster {
    fn render(&self) -> Text {
        match self {
            Monster::Goblin => Text::styled("G", Style::default().fg(Color::Green)),
            Monster::Skeleton => Text::styled("S", Style::default().fg(Color::Gray)),
            Monster::Ooze => Text::styled("O", Style::default().fg(Color::Blue)),
            Monster::Dragon => Text::styled("D", Style::default().fg(Color::Red)),
            Monster::Chest => Text::styled("C", Style::default().fg(Color::Rgb(128, 0, 128))),
            Monster::Potion => Text::styled("P", Style::default().fg(Color::Rgb(255, 165, 0))),
        }
    }
}

#[derive(Debug)]
enum HeroLevel {
    Novice,
    Expert,
}

#[derive(Debug)]
enum HeroType {
    Bard,
    Battlemage,
    Beguiler,
    Chieftain,
    Commander,
    DragonSlayer,
    Necromancer,
    Paladin,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum Treasure {
    VorpalSword,
    Talisman,
    ScepterOfPower,
    ThievesTools,
    Scroll,
    RingOfInvisiblity,
    DragonScales,
    Potion,
    DragonBait,
    TownPortal,
}

lazy_static! {
    static ref TREASURE: Vec<Treasure> = {
        let mut amounts = HashMap::new();
        amounts.insert(Treasure::VorpalSword, 3);
        amounts.insert(Treasure::Talisman, 3);
        amounts.insert(Treasure::ScepterOfPower, 3);
        amounts.insert(Treasure::ThievesTools, 3);
        amounts.insert(Treasure::Scroll, 3);
        amounts.insert(Treasure::RingOfInvisiblity, 4);
        amounts.insert(Treasure::DragonScales, 6);
        amounts.insert(Treasure::Potion, 3);
        amounts.insert(Treasure::DragonBait, 4);
        amounts.insert(Treasure::TownPortal, 4);

        let mut treasure = Vec::with_capacity(36);
        amounts.iter().for_each(|(item, amt)| {
            (0..*amt).for_each(|_| {
                treasure.push(item.clone());
            })
        });

        treasure
    };
}

#[derive(Debug)]
struct Hero {
    hero: HeroType,
    level: HeroLevel,
    xp: u64,
    ult_used: bool,
}

impl Hero {
    fn new(hero: HeroType) -> Self {
        Self {
            hero,
            level: HeroLevel::Novice,
            xp: 0,
            ult_used: false,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum MonsterPhase {
    SelectAlly,
    SelectMonster,
}

#[derive(Debug, PartialEq, Eq)]
enum LootPhase {
    SelectAlly,
    SelectLoot,
}

#[derive(Debug, PartialEq, Eq)]
enum Phase {
    Monster(MonsterPhase),
    Loot(LootPhase),
    Dragon,
    Regroup,
}

#[derive(Debug)]
struct Game<R: Rng> {
    rng: R,
    blink: bool,
    delve: u64,
    level: u64,
    phase: Phase,
    hero: Hero,
    party: Vec<Ally>,
    graveyard: Vec<Ally>,
    dungeon: Vec<Monster>,
    treasure: Vec<Treasure>,
    dragon_lair: u64,
    selection: u64,
}

impl<R: Rng> Game<R> {
    fn new(rng: R, hero: HeroType) -> Self {
        Self {
            rng,
            blink: true,
            delve: 0,
            level: 5,
            phase: Phase::Monster(MonsterPhase::SelectAlly),
            hero: Hero::new(hero),
            party: Vec::new(),
            graveyard: Vec::new(),
            dungeon: Vec::new(),
            treasure: TREASURE.clone(),
            dragon_lair: 0,
            selection: 0,
        }
    }

    fn roll<T: Dice + Eq + Hash>(&mut self, count: u64) -> Vec<T> {
        (0..count)
            .map(|_| self.rng.gen_range(0..T::faces()))
            .map(|n| T::nth(n))
            .collect()
    }

    fn next_delve(&mut self) {
        self.delve += 1;
        self.party = self.roll(7);
        self.next_level();
    }

    fn next_level(&mut self) {
        self.level += 1;
        self.dungeon = self.roll(self.level);
    }

    fn monster_indexes(&self, monster: Monster) -> Vec<usize> {
        self.dungeon
            .iter()
            .enumerate()
            .filter_map(|(n, m)| if m == &monster { Some(n) } else { None })
            .collect()
    }

    fn kill(&mut self, monster: Monster) {
        let indexes = self.monster_indexes(monster);
        if let Some(i) = indexes.first() {
            self.dungeon.remove(*i);
        }
    }

    fn kill_all(&mut self, monster: Monster) {
        self.monster_indexes(monster).iter().for_each(|i| {
            self.dungeon.remove(*i);
        });
    }

    fn select_next(&mut self) {
        let limit = match self.phase {
            Phase::Monster(MonsterPhase::SelectAlly) | Phase::Loot(LootPhase::SelectAlly) => {
                self.party.len()
            }
            Phase::Monster(MonsterPhase::SelectMonster) => self.dungeon.len(),
            _ => 0,
        } as u64;

        if self.selection < limit - 1 {
            self.selection += 1;
        }
    }

    fn select_prev(&mut self) {
        if self.selection > 0 {
            self.selection -= 1;
        }
    }

    fn next_phase(&mut self) {
        self.phase = match self.phase {
            Phase::Monster(MonsterPhase::SelectAlly) => Phase::Monster(MonsterPhase::SelectMonster),
            Phase::Monster(MonsterPhase::SelectMonster) => Phase::Loot(LootPhase::SelectAlly),
            Phase::Loot(LootPhase::SelectAlly) => Phase::Monster(MonsterPhase::SelectAlly),
            _ => unreachable!(),
        };
        self.selection = 0;
    }

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

        monster_row
            .iter()
            .zip(&self.dungeon)
            .enumerate()
            .for_each(|(i, (c, m))| {
                let style = Style::default();
                let style = if i as u64 == self.selection
                    && self.blink
                    && subphase == &MonsterPhase::SelectMonster
                {
                    style.bg(Color::White)
                } else {
                    style.bg(Color::Black)
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

        party_row
            .iter()
            .zip(&self.party)
            .enumerate()
            .for_each(|(i, (c, p))| {
                let style = Style::default();
                let style = if i as u64 == self.selection
                    && self.blink
                    && subphase == &MonsterPhase::SelectAlly
                {
                    style.bg(Color::White)
                } else {
                    style.bg(Color::Black)
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

    fn render<'a, B: Backend>(
        &self,
        terminal: &'a mut Terminal<B>,
    ) -> Result<CompletedFrame<'a>, io::Error> {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Length(5), Constraint::Percentage(100)].as_ref())
                .split(f.size());

            match &self.phase {
                Phase::Monster(sp) => self.render_monster_phase(f, chunks[1], sp),
                _ => (),
            }

            let paragraph = Paragraph::new(Text::from("\nWelcome to the dungeon!"))
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(Color::White).bg(Color::Black))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });
            /*
                        f.render_widget(
                            Paragraph::new(Text::from("Scary and gooey"))
                                .block(Block::default().title("Ooze").borders(Borders::ALL))
                                .style(Style::default().fg(Color::White).bg(Color::Black))
                                // .alignment(Alignment::Center)
                                .wrap(Wrap { trim: true }),
                            Rect::new(15, 7, 19, 3),
                        );
            */
            f.render_widget(paragraph, chunks[0]);
            f.render_widget(Block::default().borders(Borders::ALL), chunks[1]);
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    let mut seed: <Pcg64Mcg as SeedableRng>::Seed = Default::default();
    thread_rng().fill(&mut seed);
    let rng = Pcg64Mcg::from_seed(seed);
    let mut game = Game::new(rng, HeroType::Bard);
    game.next_delve();

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut reader = EventStream::new();
    enable_raw_mode()?;

    terminal.clear()?;
    loop {
        game.render(&mut terminal)?;

        tokio::select! {
            _ = sleep(Duration::from_millis(500)) => { game.blink = !game.blink; }
            maybe_event = reader.next() => match maybe_event {
                Some(Ok(event)) => {
                    game.blink = true;

                    if let Event::Key(kc) = event {
                            match kc.code {
                                KeyCode::Right => game.select_next(),
                                KeyCode::Left => game.select_prev(),
                                KeyCode::Esc => break,
                                _ => (),
                            }
                    }
                }
                Some(Err(e)) => println!("Error: {:?}\r", e),
                None => break,
            }
        }
    }

    Ok(())
}
