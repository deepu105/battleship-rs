use std::{collections::BTreeSet, fmt};

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
  pub message: String,
  pub clear_alert_countdown: u16,
  game: Game,
  active_column: usize,
  active_row: usize,
  selected_coordinates: BTreeSet<Coordinate>,
}

impl App {
  pub fn new(title: String, enhanced_graphics: bool) -> Self {
    App {
      title,
      should_quit: false,
      enhanced_graphics,
      active_column: 0,
      active_row: 0,
      selected_coordinates: BTreeSet::new(),
      game: Game::new(),
      message: String::default(),
      clear_alert_countdown: 0,
    }
  }

  fn on_up(&mut self) {
    if let Some(active_row) = self.active_row.checked_sub(1) {
      self.active_row = active_row;
    }
  }

  fn on_down(&mut self) {
    if self.active_row < ROWS - 1 {
      self.active_row += 1;
    }
  }

  fn on_right(&mut self) {
    if self.active_column < COLUMNS - 1 {
      self.active_column += 1;
    }
  }

  fn on_left(&mut self) {
    if let Some(active_column) = self.active_column.checked_sub(1) {
      self.active_column = active_column;
    }
  }

  fn on_select(&mut self) {
    if !self.game.is_won() {
      if self.is_selected((self.active_row, self.active_column)) {
        self
          .selected_coordinates
          .remove(&(self.active_row, self.active_column));
      } else if self.is_valid_rule() {
        self
          .selected_coordinates
          .insert((self.active_row, self.active_column));
      } else {
        self.message = "Maximum shots for rule selected".into()
      }
    }
  }

  fn on_fire(&mut self) {
    self.message = if self.selected_coordinates.len() == 0 {
      "Select opponent coordinates to hit".into()
    } else if !self.game.is_won() && self.game.is_user_turn() {
      self.game.fire(&self.selected_coordinates)
    } else {
      "Not your turn".into()
    };
  }

  fn is_valid_rule(&mut self) -> bool {
    self.game.is_valid_rule(self.selected_coordinates.len())
  }

  fn is_selected(&self, coordinate: Coordinate) -> bool {
    self.selected_coordinates.iter().any(|c| *c == coordinate)
  }

  fn active(&self) -> Coordinate {
    (self.active_row, self.active_column)
  }

  pub fn is_won(&self) -> bool {
    self.game.is_won()
  }

  pub fn cell(&self, c: Coordinate, read_only: bool) -> Cell {
    Cell::new(self, c, read_only)
  }

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
        self.on_select();
      }
      Key::Char('\n') => self.on_fire(),
      _ => { /* do nothing */ }
    }
  }

  pub fn on_tick(&mut self) {
    self.clear_alert_countdown += 1
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
      self.app.active() == self.coordinate
    }
  }

  fn is_selected(&self) -> bool {
    if self.read_only {
      false
    } else {
      self.app.is_selected(self.coordinate)
    }
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
            Color::Yellow
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
    } else if position.status == Status::LIVE {
      Color::Green
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
