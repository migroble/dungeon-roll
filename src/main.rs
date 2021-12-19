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
    style::{Color, Modifier, Style},
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

#[derive(Debug, Dice, PartialEq)]
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

#[derive(Debug, Dice, PartialEq, Clone)]
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

    fn is_monster(&self) -> bool {
        matches!(self, Monster::Goblin | Monster::Skeleton | Monster::Ooze)
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
enum MonsterSelect {
    Ally,
    Reroll,
    Monster,
}

#[derive(Debug, PartialEq, Eq)]
enum LootSelect {
    Ally,
    Loot,
}

#[derive(Debug, PartialEq, Eq)]
enum Phase {
    Monster(MonsterSelect),
    Loot(LootSelect),
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
    dragon_lair: usize,
    selection_ally: usize,
    selection_reroll: usize,
    selection_monster: usize,
}

impl<R: Rng> Game<R> {
    fn new(rng: R, hero: HeroType) -> Self {
        Self {
            rng,
            blink: true,
            delve: 0,
            level: 5,
            phase: Phase::Monster(MonsterSelect::Ally),
            hero: Hero::new(hero),
            party: Vec::new(),
            graveyard: Vec::new(),
            dungeon: Vec::new(),
            treasure: TREASURE.clone(),
            dragon_lair: 0,
            selection_ally: 0,
            selection_reroll: 0,
            selection_monster: 0,
        }
    }

    fn roll<T: Dice>(&mut self) -> T {
        T::nth(self.rng.gen_range(0..T::faces()))
    }

    fn roll_n<T: Dice>(&mut self, n: u64) -> Vec<T> {
        (0..n).map(|_| self.roll()).collect()
    }

    fn next_delve(&mut self) {
        self.delve += 1;
        self.party = self.roll_n(7);
        self.next_level();
    }

    fn next_level(&mut self) {
        self.level += 1;
        self.dungeon = self.roll_n(self.level);
        self.selection_monster = 0;
    }

    fn select_next(&mut self) {
        match self.phase {
            Phase::Monster(MonsterSelect::Ally) | Phase::Loot(LootSelect::Ally) => {
                if self.selection_ally < self.party.len() - 1 {
                    self.selection_ally += 1;
                }
            }
            Phase::Monster(MonsterSelect::Reroll) => {
                if self.selection_reroll + 1 == self.selection_ally
                    && self.selection_reroll + 2 < self.party.len()
                {
                    self.selection_reroll += 2;
                } else if self.selection_reroll + 1 < self.party.len() {
                    self.selection_reroll += 1;
                }
            }
            Phase::Monster(MonsterSelect::Monster) => {
                if self.selection_monster + 1 < self.dungeon.len() {
                    self.selection_monster += 1;
                }
            }
            _ => (),
        };
    }

    fn select_prev(&mut self) {
        match self.phase {
            Phase::Monster(MonsterSelect::Ally) | Phase::Loot(LootSelect::Ally) => {
                if self.selection_ally > 0 {
                    self.selection_ally -= 1;
                }
            }
            Phase::Monster(MonsterSelect::Reroll) => {
                if self.selection_reroll - 1 == self.selection_ally && self.selection_reroll - 1 > 0
                {
                    self.selection_reroll -= 2;
                } else if self.selection_reroll > 0 {
                    self.selection_reroll -= 1;
                }
            }
            Phase::Monster(MonsterSelect::Monster) => {
                if self.selection_monster > 0 {
                    self.selection_monster -= 1;
                }
            }
            _ => (),
        };
    }

    fn has_monsters(&self) -> bool {
        self.dungeon.iter().any(|m| m.is_monster())
    }

    fn execute_combat(&mut self) {
        let kill_all = matches!(
            (
                &self.party[self.selection_ally],
                &self.dungeon[self.selection_monster],
            ),
            (&Ally::Fighter, &Monster::Goblin)
                | (&Ally::Cleric, &Monster::Skeleton)
                | (&Ally::Mage, &Monster::Ooze)
                | (&Ally::Champion, _)
        );
        if kill_all {
            let monster = self.dungeon[self.selection_monster].clone();
            self.dungeon.retain(|m| m != &monster);
        } else {
            self.dungeon.remove(self.selection_monster);
        }

        let ally = self.party.remove(self.selection_ally);
        self.graveyard.push(ally);
    }

    fn execute_reroll(&mut self) {
        self.party[self.selection_reroll] = self.roll();
        self.party.remove(self.selection_ally);
    }

    fn next_phase(&mut self) {
        self.phase = match self.phase {
            Phase::Monster(MonsterSelect::Ally) => {
                if self.party[self.selection_ally] == Ally::Scroll {
                    if self.selection_reroll == self.selection_ally {
                        if self.selection_ally == 0 {
                            self.selection_reroll = 1;
                        } else {
                            self.selection_reroll -= 1;
                        }
                    }
                    Phase::Monster(MonsterSelect::Reroll)
                } else {
                    Phase::Monster(MonsterSelect::Monster)
                }
            }
            Phase::Monster(MonsterSelect::Reroll) => {
                self.execute_reroll();
                Phase::Monster(MonsterSelect::Ally)
            }
            Phase::Monster(MonsterSelect::Monster) => {
                self.execute_combat();
                if self.has_monsters() {
                    Phase::Monster(MonsterSelect::Ally)
                } else {
                    Phase::Loot(LootSelect::Ally)
                }
            }
            Phase::Loot(LootSelect::Ally) => Phase::Monster(MonsterSelect::Ally),
            _ => unreachable!(),
        };

        self.selection_monster = self.selection_monster.min(self.dungeon.len() - 1);
        self.selection_ally = self.selection_ally.min(self.party.len() - 1);
        self.selection_reroll = self.selection_reroll.min(self.party.len() - 1);
    }

    fn prev_phase(&mut self) {
        self.phase = match self.phase {
            Phase::Monster(MonsterSelect::Ally) => Phase::Monster(MonsterSelect::Ally),
            Phase::Monster(MonsterSelect::Reroll) => Phase::Monster(MonsterSelect::Ally),
            Phase::Monster(MonsterSelect::Monster) => Phase::Monster(MonsterSelect::Ally),
            Phase::Loot(LootSelect::Ally) => Phase::Loot(LootSelect::Ally),
            _ => unreachable!(),
        };
    }

    fn render_monster_phase<B: Backend>(
        &self,
        f: &mut Frame<B>,
        window: Rect,
        subphase: &MonsterSelect,
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
                let style = if i == self.selection_monster
                    && self.blink
                    && subphase == &MonsterSelect::Monster
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
                let style = if (i == self.selection_ally
                    && (self.blink && subphase == &MonsterSelect::Ally))
                    || (i == self.selection_reroll
                        && self.blink
                        && subphase == &MonsterSelect::Reroll)
                {
                    style.bg(Color::White)
                } else if i == self.selection_ally && subphase != &MonsterSelect::Ally {
                    style.bg(Color::DarkGray).add_modifier(Modifier::DIM)
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
                                KeyCode::Enter => game.next_phase(),
                                KeyCode::Backspace => game.prev_phase(),
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
