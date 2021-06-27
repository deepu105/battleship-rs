use tui::{
  backend::Backend,
  layout::{Alignment, Constraint, Direction, Layout, Rect},
  style::{Color, Modifier, Style},
  widgets::{Block, BorderType, Borders, Clear, Paragraph},
  Frame,
};

use super::{
  game::{COLS, ROWS},
  App,
};

const CELL_WIDTH: u16 = 5;
const CELL_HEIGHT: u16 = 3;
const PADDING: u16 = 1;
const GRID_WIDTH: u16 = CELL_WIDTH * (COLS as u16) + 2 * PADDING;
const GRID_HEIGHT: u16 = CELL_HEIGHT * (ROWS as u16) + 2 * PADDING;

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
  let main_block = Block::default()
    .borders(Borders::ALL)
    .style(Style::default().bg(Color::Black).fg(Color::Cyan))
    .title(format!(
      "{} | Rule: {} ({}s)",
      app.title,
      app.rule(),
      app.elapsed_duration(),
    ));

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

  let header = Paragraph::new(
    "move: 🠔 🠗 🠕 🠖 (or) hjkl | select/unselect: <space> | fire: <enter> | quit: <q>",
  )
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

  // show alerts
  if app.frame_count % 8 != 0 || app.is_won() {
    draw_alert(f, app.message.clone(), v_chunks[1]);
  } else {
    // reset messages
    app.message = String::default();
  }
}

fn draw_board<B: Backend>(
  f: &mut Frame<B>,
  player_chunk: Rect,
  title: &str,
  app: &mut App,
  is_self: bool,
) {
  let row_constraints = std::iter::repeat(Constraint::Length(CELL_HEIGHT))
    .take(ROWS)
    .collect::<Vec<_>>();
  let col_constraints = std::iter::repeat(Constraint::Length(CELL_WIDTH))
    .take(COLS)
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
    .constraints(row_constraints)
    .split(board_rect);

  for (r, row_rect) in row_rects.into_iter().enumerate() {
    let col_rects = Layout::default()
      .direction(Direction::Horizontal)
      .vertical_margin(0)
      .horizontal_margin(1)
      .constraints(col_constraints.clone())
      .split(row_rect);

    for (c, cell_rect) in col_rects.into_iter().enumerate() {
      let cell = app.cell((r, c), is_self);
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

fn draw_alert<B: Backend>(f: &mut Frame<B>, message: String, area: Rect) {
  if !message.is_empty() {
    let area = centered_rect(50, 4, area);
    f.render_widget(Clear, area); //this clears out the background
    f.render_widget(
      Paragraph::new(message)
        .block(
          Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(
              Style::default()
                .fg(if true {
                  Color::Magenta
                } else {
                  Color::LightGreen
                })
                .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().add_modifier(Modifier::BOLD)),
        )
        .alignment(Alignment::Center)
        .style(Style::default()),
      area,
    );
  }
}

fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
  let Rect {
    width: grid_width,
    height: grid_height,
    ..
  } = r;

  let outer_height = (grid_height / 2)
    .checked_sub(height / 2)
    .unwrap_or_default();
  let popup_layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints(
      [
        Constraint::Length(outer_height),
        Constraint::Length(height),
        Constraint::Length(outer_height),
      ]
      .as_ref(),
    )
    .split(r);

  let outer_width = (grid_width / 2).checked_sub(width / 2).unwrap_or_default();

  Layout::default()
    .direction(Direction::Horizontal)
    .constraints(
      [
        Constraint::Length(outer_width),
        Constraint::Length(width),
        Constraint::Length(outer_width),
      ]
      .as_ref(),
    )
    .split(popup_layout[1])[1]
}
