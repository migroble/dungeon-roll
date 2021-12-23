#![allow(dead_code)]
#![allow(unused_variables)]

use crossterm::{
    event::EventStream,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
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

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut reader = EventStream::new();

    terminal.clear()?;
    loop {
        game.render(&mut terminal)?;

        tokio::select! {
            _ = sleep(Duration::from_millis(500)) => game.toggle_blink(),
            maybe_event = reader.next() => match maybe_event {
                Some(Ok(event)) => if game.handle_event(event) {break;},
                Some(Err(e)) => println!("Error: {:?}\r", e),
                None => break,
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
