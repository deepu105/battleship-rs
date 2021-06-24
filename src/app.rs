use std::fmt;

use termion::event::Key;
use tui::{
  style::{Color, Modifier, Style},
  widgets::{Block, BorderType, Borders},
};

use crate::game::{Coordinate, Game, Position, Status, COLUMNS, ROWS};

pub struct App {
  pub title: String,
  pub should_quit: bool,
  pub enhanced_graphics: bool,
  game: Game,
  active_column: usize,
  active_row: usize,
}

impl App {
  pub fn new(title: String, enhanced_graphics: bool) -> Self {
    App {
      title,
      should_quit: false,
      enhanced_graphics,
      active_column: 0,
      active_row: 0,
      game: Game::new(),
    }
  }

  pub fn on_up(&mut self) {
    if let Some(active_row) = self.active_row.checked_sub(1) {
      self.active_row = active_row;
    }
  }

  pub fn on_down(&mut self) {
    if self.active_row < ROWS - 1 {
      self.active_row += 1;
    }
  }

  pub fn on_right(&mut self) {
    if self.active_column < COLUMNS - 1 {
      self.active_column += 1;
    }
  }

  pub fn on_left(&mut self) {
    if let Some(active_column) = self.active_column.checked_sub(1) {
      self.active_column = active_column;
    }
  }

  pub fn cell(&self, c: Coordinate, read_only: bool) -> Cell {
    Cell::new(self, c, read_only)
  }

  fn active(&self) -> Coordinate {
    (self.active_row, self.active_column)
  }

  //   fn active_cell(&self, read_only: bool) -> Cell {
  //     self.cell(self.active(), read_only)
  //   }

  //   fn expose_active_cell(&mut self) -> Result<bool, Error> {
  //     self.board.expose(self.active())
  //   }

  //   fn expose_all(&mut self) -> Result<(), Error> {
  //     self.board.expose_all()
  //   }

  //   fn won(&self) -> bool {
  //     self.board.won()
  //   }

  pub fn on_key(&mut self, key: Key) {
    match key {
      Key::Up | Key::Char('k') => {
        self.on_up();
      }
      Key::Down | Key::Char('j') => {
        self.on_down();
      }
      Key::Left | Key::Char('h') => {
        self.on_left();
      }
      Key::Right | Key::Char('l') => {
        self.on_right();
      }
      Key::Char(' ') => {
        // mark cell as selected
      }
      Key::Char('\n') => {
        // trigger fire
      }
      _ => { /* do nothing */ }
    }
  }

  pub fn on_tick(&mut self) {
    // Update progress
  }
}

pub struct Cell<'app> {
  app: &'app App,
  coordinate: Coordinate,
  read_only: bool,
}

impl<'app> Cell<'app> {
  fn new(app: &'app App, coordinate: Coordinate, read_only: bool) -> Self {
    Self {
      app,
      coordinate,
      read_only,
    }
  }

  fn get_position(&self) -> &Position {
    if self.read_only {
      &self.app.game.player().player_board().positions[self.coordinate.0][self.coordinate.1]
    } else {
      &self.app.game.opponent().opponent_board().positions[self.coordinate.0][self.coordinate.1]
    }
  }

  fn is_active(&self) -> bool {
    if self.read_only {
      false
    } else {
      self.app.active() == (self.coordinate)
    }
  }

  fn is_selected(&self) -> bool {
    // self.app.board.tile(self.row, self.column).unwrap().flagged
    false
  }

  pub fn block(&self) -> Block {
    Block::default()
      .borders(Borders::ALL)
      .style(
        Style::default()
          .bg(Color::Black)
          .fg(if self.is_active() {
            Color::Cyan
          } else if self.is_selected() {
            Color::LightCyan
          } else {
            let position = self.get_position();
            match position.status {
              Status::LIVE => Color::Green,
              Status::MISS => Color::Yellow,
              Status::HIT => Color::LightRed,
              Status::KILL => Color::Red,
              Status::SPACE => Color::White,
            }
          })
          .add_modifier(if self.is_active() {
            Modifier::BOLD
          } else {
            Modifier::empty()
          }),
      )
      .border_type(BorderType::Plain)
  }

  pub fn text_style(&self) -> Style {
    let position = self.get_position();
    Style::default().bg(if position.status == Status::SPACE {
      Color::White
    } else {
      Color::Black
    })
  }
}

impl fmt::Display for Cell<'_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let position = self.get_position();

    write!(f, "{}", position.status.as_emoji())
  }
}
