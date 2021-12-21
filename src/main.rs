#![allow(dead_code)]
#![allow(unused_variables)]

use crossterm::{
    event::{Event, EventStream, KeyCode},
    terminal::enable_raw_mode,
};
use futures::StreamExt;
use rand::prelude::*;
use rand_pcg::Pcg64Mcg;
use std::{io, time::Duration};
use tokio::time::sleep;
use tui::{backend::CrosstermBackend, Terminal};

#[macro_use]
extern crate dice_derive;
#[macro_use]
extern crate lazy_static;

mod dice;
mod game;
mod hero;
mod phase;
mod treasure;

use game::*;
use hero::HeroType;

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
            _ = sleep(Duration::from_millis(500)) => game.toggle_blink(),
            maybe_event = reader.next() => match maybe_event {
                Some(Ok(event)) => {
                    game.set_blink();

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
