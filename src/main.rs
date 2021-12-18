use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use futures::StreamExt;
use rand::prelude::*;
use rand_pcg::Pcg64Mcg;
use std::{collections::HashMap, hash::Hash, io, iter::repeat, time::Duration};
use tokio::time::sleep;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
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
    Thief,
    Champion,
    Scroll,
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
    fn render(&self) -> &'static str {
        match self {
            Monster::Goblin => "G",
            Monster::Skeleton => "S",
            Monster::Ooze => "O",
            Monster::Dragon => "D",
            Monster::Chest => "C",
            Monster::Potion => "P",
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

#[derive(Debug)]
struct Game<R: Rng> {
    rng: R,
    delve: u64,
    level: u64,
    hero: Hero,
    party: Vec<Ally>,
    graveyard: Vec<Ally>,
    pub dungeon: Vec<Monster>,
    treasure: Vec<Treasure>,
    selection: u64,
}

impl<R: Rng> Game<R> {
    fn new(rng: R, hero: HeroType) -> Self {
        Self {
            rng,
            delve: 0,
            level: 5,
            hero: Hero::new(hero),
            party: Vec::new(),
            graveyard: Vec::new(),
            dungeon: Vec::new(),
            treasure: TREASURE.clone(),
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
        // println!("{:#?}\n{:#?}", self.party, self.dungeon);
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

    terminal.clear()?;
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Length(5), Constraint::Percentage(100)].as_ref())
                .split(f.size());

            let monster_row = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints(
                    repeat(Constraint::Percentage(20))
                        .take(5)
                        .collect::<Vec<_>>(),
                )
                .split(chunks[1]);

            monster_row.iter().zip(&game.dungeon).for_each(|(c, m)| {
                f.render_widget(
                    Paragraph::new(Text::from(m.render()))
                        // .block(Block::default().borders(Borders::ALL))
                        .style(Style::default().fg(Color::White).bg(Color::Black))
                        .alignment(Alignment::Center),
                    *c,
                )
            });

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
        })?;

        match reader.next().await {
            Some(Ok(event)) => {
                if event == Event::Key(KeyCode::Esc.into()) {
                    break;
                }
            }
            Some(Err(e)) => println!("Error: {:?}\r", e),
            None => break,
        };
    }

    Ok(())
}
