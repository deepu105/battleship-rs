use std::{
  collections::BTreeSet,
  fmt,
  time::{Duration, Instant},
};

use termion::event::Key;
use tui::{
  style::{Color, Style},
  widgets::{Block, BorderType, Borders},
};

use super::game::{Coordinate, Difficulty, Game, Rule, Status, COLS, ROWS};

pub struct App {
  pub title: String,
  pub should_quit: bool,
  pub enhanced_graphics: bool,
  pub message: String,
  pub frame_count: u16,
  pub start_time: Instant,
  game: Game,
  active_column: usize,
  active_row: usize,
  selected_coordinates: BTreeSet<Coordinate>,
  duration: Option<Duration>,
}

impl App {
  pub fn new(title: String, rule: Rule, difficulty: Difficulty) -> Self {
    App {
      title,
      should_quit: false,
      enhanced_graphics: true,
      active_column: 0,
      active_row: 0,
      selected_coordinates: BTreeSet::new(),
      game: Game::new(rule, difficulty),
      message: String::default(),
      frame_count: 0,
      start_time: Instant::now(),
      duration: None,
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
    if self.active_column < COLS - 1 {
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
    let msg = if self.selected_coordinates.is_empty() {
      "Select opponent coordinates to hit".into()
    } else if !self.game.is_won() && self.game.is_user_turn() {
      let msg = self.game.fire(&self.selected_coordinates, false);
      self.selected_coordinates = BTreeSet::new();
      msg
    } else {
      "Not your turn".into()
    };
    // append to previous msg
    self.message = format!(
      "{}{}{}",
      self.message,
      if self.message.is_empty() { "" } else { "\n" },
      msg
    );
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

  pub fn rule(&self) -> &Rule {
    &self.game.rule
  }

  pub fn elapsed_duration(&self) -> u64 {
    if let Some(duration) = self.duration {
      duration.as_secs()
    } else {
      self.start_time.elapsed().as_secs()
    }
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
    if self.is_won() && self.duration.is_none() {
      let duration = self.start_time.elapsed();
      self.duration = Some(duration);
      self.message = format!("{} (In {} seconds)", self.message, duration.as_secs());
    }
    // computer delays firing by 2 seconds to make the game feel more natural
    if !self.game.is_user_turn() && !self.is_won() && self.frame_count % 8 == 0 {
      self.message = self.game.bot_fire();
    }
    self.frame_count += 1;
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

  fn get_position_status(&self) -> Status {
    let (pos, ship) = if self.read_only {
      self
        .app
        .game
        .player()
        .player_board()
        .find_position_and_ship(self.coordinate)
    } else {
      self
        .app
        .game
        .player()
        .opponent_board()
        .find_position_and_ship(self.coordinate)
    };

    pos.get_status(ship)
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
        // cell background and border color
        Style::default().bg(Color::Black).fg(if self.is_selected() {
          Color::Yellow
        } else if self.is_active() {
          Color::Cyan
        } else {
          let status = self.get_position_status();
          match status {
            Status::Live => Color::Yellow,
            Status::Hit | Status::Kill => Color::Red,
            Status::Miss | Status::Space => Color::White,
          }
        }),
      )
      .border_type(BorderType::Rounded)
  }

  pub fn text_style(&self) -> Style {
    // cell background color
    Style::default().bg(Color::Black)
  }
}

impl fmt::Display for Cell<'_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let status = self.get_position_status();

    write!(f, "{}", status)
  }
}
