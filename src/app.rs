use std::fmt;

use termion::event::Key;
use tui::{
  style::{Color, Modifier, Style},
  widgets::{Block, BorderType, Borders},
};

pub const ROWS: u16 = 10;
pub const COLUMNS: u16 = 10;
const BOMB: &str = "💣";
const FLAG: &str = "⛳";
pub struct Cell<'app> {
  app: &'app App,
  row: u16,
  column: u16,
  read_only: bool,
}

impl<'app> Cell<'app> {
  fn new(app: &'app App, row: u16, column: u16, read_only: bool) -> Self {
    Self {
      app,
      row,
      column,
      read_only,
    }
  }

  fn is_active(&self) -> bool {
    if self.read_only {
      false
    } else {
      self.app.active() == (self.row, self.column)
    }
  }

  fn is_exposed(&self) -> bool {
    // self.app.board.tile(self.row, self.column).unwrap().exposed
    false
  }

  fn is_flagged(&self) -> bool {
    // self.app.board.tile(self.row, self.column).unwrap().flagged
    false
  }

  fn is_mine(&self) -> bool {
    // self.app.board.tile(self.row, self.column).unwrap().mine
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
          } else if self.is_mine() {
            Color::LightRed
          } else {
            Color::White
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
    Style::default()
      .fg(if self.is_exposed() && self.is_mine() {
        Color::LightYellow
      } else if self.is_exposed() {
        Color::White
      } else {
        Color::Black
      })
      .bg(if self.is_exposed() {
        Color::Black
      } else if self.is_active() {
        Color::Cyan
      } else {
        Color::White
      })
  }
}

impl fmt::Display for Cell<'_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{}",
      if self.is_flagged() {
        FLAG.to_owned()
      } else if self.is_mine() && self.is_exposed() {
        BOMB.to_owned()
      //   } else if self.is_exposed() {
      //     let num_adjacent_mines = self
      //       .app
      //       .board
      //       .tile(self.row, self.column)
      //       .unwrap()
      //       .adjacent_mines;
      //     if num_adjacent_mines == 0 {
      //       " ".to_owned()
      //     } else {
      //       format!("{}", num_adjacent_mines)
      //     }
      } else {
        "x".to_owned()
      }
    )
  }
}

pub(crate) type Coordinate = (u16, u16);
pub struct App {
  pub title: String,
  pub should_quit: bool,
  pub enhanced_graphics: bool,
  active_column: u16,
  active_row: u16,
}

impl App {
  pub fn new(title: String, enhanced_graphics: bool) -> App {
    App {
      title,
      should_quit: false,
      enhanced_graphics,
      active_column: 0,
      active_row: 0,
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

  pub fn cell(&self, (r, c): Coordinate, read_only: bool) -> Cell {
    Cell::new(self, r, c, read_only)
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
