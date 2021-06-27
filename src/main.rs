mod app;
mod event;
mod game;
mod ui;

use std::{
  error::Error,
  io::{self, stdout, Write},
  time::Duration,
};

use app::App;
use event::{Event, Events};
use game::{Difficulty, Rule};
use structopt::StructOpt;
use termion::{
  event::Key,
  input::MouseTerminal,
  raw::IntoRawMode,
  screen::{AlternateScreen, ToMainScreen},
};
use tui::{backend::TermionBackend, Terminal};

#[derive(Debug, StructOpt)]
#[structopt(name = "battleship-rs", about = "A Battleship game in Rust")]
struct Opt {
  /// Game rule
  #[structopt(short, long, possible_values = &Rule::variants(), case_insensitive = true, default_value = "Default")]
  pub rule: Rule,
  /// Game rule
  #[structopt(short, long, possible_values = &Difficulty::variants(), case_insensitive = true, default_value = "Easy")]
  pub difficulty: Difficulty,
}

fn main() -> Result<(), Box<dyn Error>> {
  std::panic::set_hook(Box::new(move |x| {
    stdout()
      .into_raw_mode()
      .unwrap()
      .suspend_raw_mode()
      .unwrap();
    write!(stdout().into_raw_mode().unwrap(), "{}", ToMainScreen).unwrap();
    print!("{:?}", x);
  }));

  let opt = Opt::from_args();

  // time in ms between two ticks is 250ms.
  let events = Events::new(Duration::from_millis(250));

  let stdout = io::stdout().into_raw_mode()?;
  let stdout = MouseTerminal::from(stdout);
  let stdout = AlternateScreen::from(stdout);
  let backend = TermionBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;

  let mut app = App::new(" 🚀 Battleship.rs 🚀 ".into(), opt.rule, opt.difficulty);
  loop {
    terminal.draw(|f| ui::draw(f, &mut app))?;

    match events.next()? {
      Event::Input(key) => match key {
        Key::Ctrl('c') | Key::Char('q') => {
          app.should_quit = true;
        }
        _ => app.on_key(key),
      },
      Event::Tick => {
        app.on_tick();
      }
    }
    if app.should_quit {
      break;
    }
  }

  Ok(())
}
