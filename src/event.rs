use std::{io, sync::mpsc, thread, time::Duration};

use termion::{event::Key, input::TermRead};

pub enum Event<I> {
  Input(I),
  Tick,
}

/// A small event handler that wrap termion input and tick events. Each event
/// type is handled in its own thread and returned to a common `Receiver`
pub struct Events {
  rx: mpsc::Receiver<Event<Key>>,
}

impl Events {
  pub fn new(tick_rate: Duration) -> Events {
    let (tx, rx) = mpsc::channel();

    let tx_clone = tx.clone();

    thread::spawn(move || {
      let stdin = io::stdin();
      for key in stdin.keys().flatten() {
        if let Err(err) = tx_clone.send(Event::Input(key)) {
          eprintln!("{}", err);
          return;
        }
      }
    });

    thread::spawn(move || loop {
      if let Err(err) = tx.send(Event::Tick) {
        eprintln!("{}", err);
        break;
      }
      thread::sleep(tick_rate);
    });

    Events { rx }
  }

  pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
    self.rx.recv()
  }
}
