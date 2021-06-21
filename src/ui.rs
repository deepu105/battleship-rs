use std::convert::TryFrom;

use tui::{
  backend::Backend,
  layout::{Alignment, Constraint, Direction, Layout, Rect},
  style::{Color, Modifier, Style},
  widgets::{Block, BorderType, Borders, Paragraph},
  Frame,
};

use super::{
  app::{COLUMNS, ROWS},
  App,
};

const CELL_WIDTH: u16 = 5;
const CELL_HEIGHT: u16 = 3;
const PADDING: u16 = 1;
const GRID_WIDTH: u16 = CELL_WIDTH * COLUMNS + 2 * PADDING;
const GRID_HEIGHT: u16 = CELL_HEIGHT * ROWS + 2 * PADDING;

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
  let main_block = Block::default()
    .borders(Borders::ALL)
    .style(Style::default().bg(Color::Black).fg(Color::Cyan))
    .title(app.title.as_ref());

  f.render_widget(main_block, f.size());

  let vertical_pad_block_height = f.size().height.checked_sub(GRID_HEIGHT).unwrap_or_default() / 2;
  let v_chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints(vec![
      Constraint::Min(vertical_pad_block_height),
      Constraint::Length(GRID_HEIGHT + 1),
      Constraint::Min(vertical_pad_block_height),
    ])
    .split(f.size());

  let header =
    Paragraph::new("move: <🠔 🠗 🠕 🠖 | hjkl> | select: <space> | fire: <enter> | quit: <q>")
      .style(Style::default().fg(Color::Gray))
      .block(Block::default().borders(Borders::NONE))
      .alignment(Alignment::Center);

  f.render_widget(header, v_chunks[2]);

  let board_chunks = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
    .split(v_chunks[1]);

  let player_chunk = board_chunks[0];
  let opponent_chunk = board_chunks[1];

  draw_board(f, player_chunk, "You", app, true);
  draw_board(f, opponent_chunk, "Computer", app, false);
}

fn draw_board<B: Backend>(
  f: &mut Frame<B>,
  player_chunk: Rect,
  title: &str,
  app: &mut App,
  read_only: bool,
) {
  let row_constraints = std::iter::repeat(Constraint::Length(CELL_HEIGHT))
    .take(ROWS.into())
    .collect::<Vec<_>>();
  let col_constraints = std::iter::repeat(Constraint::Length(CELL_WIDTH))
    .take(COLUMNS.into())
    .collect::<Vec<_>>();

  let horizontal_pad_block_width = (player_chunk.width - GRID_WIDTH) / 2;
  let h_main_rects = Layout::default()
    .direction(Direction::Horizontal)
    .constraints(vec![
      Constraint::Min(horizontal_pad_block_width),
      Constraint::Length(GRID_WIDTH),
      Constraint::Min(horizontal_pad_block_width),
    ])
    .split(player_chunk);

  let v_main_rects = Layout::default()
    .direction(Direction::Vertical)
    .constraints(vec![Constraint::Min(1), Constraint::Length(GRID_HEIGHT)])
    .split(h_main_rects[1]);

  let title = Paragraph::new(title)
    .style(
      Style::default()
        .fg(Color::Green)
        .add_modifier(Modifier::BOLD),
    )
    .block(Block::default().borders(Borders::NONE))
    .alignment(Alignment::Center);

  f.render_widget(title, v_main_rects[0]);

  let board_block = Block::default()
    .borders(Borders::ALL)
    .border_type(BorderType::Plain);

  let board_rect = v_main_rects[1];
  f.render_widget(board_block, board_rect);

  let row_rects = Layout::default()
    .direction(Direction::Vertical)
    .vertical_margin(1)
    .horizontal_margin(0)
    .constraints(row_constraints.clone())
    .split(board_rect);

  for (r, row_rect) in row_rects.into_iter().enumerate() {
    let col_rects = Layout::default()
      .direction(Direction::Horizontal)
      .vertical_margin(0)
      .horizontal_margin(1)
      .constraints(col_constraints.clone())
      .split(row_rect);

    let r = u16::try_from(r).unwrap();

    for (c, cell_rect) in col_rects.into_iter().enumerate() {
      let c = u16::try_from(c).unwrap();
      let cell = app.cell((r, c), read_only);
      let single_row_text = format!(
        "{:^length$}",
        cell.to_string(),
        length = usize::from(CELL_WIDTH - 2)
      );
      let pad_line = " ".repeat(usize::from(CELL_WIDTH));

      // 1 line for the text, 1 line each for the top and bottom of the cell == 3 lines
      // that are not eligible for padding
      let num_pad_lines = usize::from(CELL_HEIGHT.checked_sub(3).unwrap_or_default());

      // text is:
      //   pad with half the pad lines budget
      //   the interesting text
      //   pad with half the pad lines budget
      //   join with newlines
      let text = std::iter::repeat(pad_line.clone())
        .take(num_pad_lines / 2)
        .chain(std::iter::once(single_row_text.clone()))
        .chain(std::iter::repeat(pad_line).take(num_pad_lines / 2))
        .collect::<Vec<_>>()
        .join("\n");

      let cell_text = Paragraph::new(text)
        .block(cell.block())
        .style(cell.text_style());
      f.render_widget(cell_text, cell_rect);
    }
  }
}
