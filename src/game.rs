use std::{
  collections::BTreeMap,
  fmt::{self, Display},
  usize,
};

use rand::{prelude::ThreadRng, seq::SliceRandom, Rng};
use uuid::Uuid;

pub const ROWS: usize = 10;
pub const COLUMNS: usize = 10;
pub const SHIP_SIZE: usize = 3;

pub struct Game {
  players: [Player; 2],
  winner: Option<Player>,
  turn: usize,
  rule: Rule,
}

impl Game {
  pub fn new() -> Self {
    Self {
      turn: 0,
      winner: None,
      players: [Player::new(), Player::default()],
      rule: Rule::Default,
    }
  }

  fn player_by_turn(&self, turn: usize) -> &Player {
    &self.players[turn]
  }
  pub fn player(&self) -> &Player {
    &self.players[0]
  }
  pub fn opponent(&self) -> &Player {
    &self.players[1]
  }
}

enum Rule {
  Default,     // single shots
  SuperCharge, // not more than total number of ships alive
  Desperation, // not more than number of killed ships + 1
}

#[derive(Ord, Eq, PartialEq, PartialOrd, Debug)]
pub enum Status {
  LIVE,
  MISS,
  HIT,
  KILL,
  SPACE,
}

impl Status {
  pub fn as_char(&self) -> char {
    match *self {
      Status::LIVE => '*',
      Status::MISS => '-',
      Status::HIT => 'X',
      Status::KILL => 'K',
      Status::SPACE => '.',
    }
  }
  pub fn as_emoji(&self) -> &str {
    match *self {
      Status::LIVE => "🚢",
      Status::MISS => "❌",
      Status::HIT => "💥",
      Status::KILL => "💀",
      Status::SPACE => "",
    }
  }
  pub fn from_char(c: char) -> Self {
    match c {
      '*' => Status::LIVE,
      '-' => Status::MISS,
      'X' => Status::HIT,
      'K' => Status::KILL,
      '.' => Status::SPACE,
      _ => panic!("char is not a valid Status"),
    }
  }
}

#[derive(Ord, Eq, PartialEq, PartialOrd)]
pub struct Player {
  is_bot: bool,
  boards: [Board; 2],
  extra_shots: bool,
}

impl Player {
  fn new() -> Self {
    Self {
      is_bot: false,
      boards: [Board::new(true), Board::new(false)],
      extra_shots: false,
    }
  }

  pub fn player_board(&self) -> &Board {
    &self.boards[0]
  }
  pub fn opponent_board(&self) -> &Board {
    &self.boards[1]
  }
}

impl Default for Player {
  fn default() -> Self {
    Self {
      is_bot: true,
      ..Self::new()
    }
  }
}

#[derive(Ord, Eq, PartialEq, PartialOrd)]
pub struct Board {
  pub positions: Vec<Vec<Position>>,
  ships: Vec<Ship>,
  firing_status: BTreeMap<String, String>,
}

impl Board {
  fn new(is_self: bool) -> Self {
    let mut rng = rand::thread_rng();
    // create empty positions
    let mut positions = (0..ROWS)
      .map(|r| {
        (0..COLUMNS)
          .map(|c| Position::new((r, c)))
          .collect::<Vec<_>>()
      })
      .collect::<Vec<_>>();

    let ships = if is_self {
      let ship_types = ShipType::get_initial_ships();
      ship_types
        .iter()
        .map(|s_type| {
          let mut ship_placed = false;
          let mut ship = Ship::new(s_type.clone());
          // place ships on the board without overlap
          // doing this in a while loop is sub optimal as this is causing
          // infinite loop if number of ships are more than 4 currently
          while !ship_placed {
            let start_cords = get_random_coordinate(&mut rng);
            if !ship.is_overlapping(&positions, start_cords) {
              // draw ship on to board
              if ship.draw(&mut positions, start_cords) {
                positions[start_cords.0][start_cords.1].ship_id = Some(ship.id.to_owned());
              }
              ship_placed = true
            } else {
              ship = Ship::new(s_type.clone());
            }
          }
          ship
        })
        .collect::<Vec<_>>()
    } else {
      vec![]
    };

    Self {
      ships,
      firing_status: BTreeMap::new(),
      positions,
    }
  }

  fn as_grid(&self) -> Vec<String> {
    self
      .positions
      .iter()
      .map(|row| {
        row
          .iter()
          .map(|c| c.to_string())
          .collect::<Vec<_>>()
          .join("")
      })
      .collect::<Vec<_>>()
  }
}

impl Display for Board {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let s = self.as_grid().join("\n");
    write!(f, "{}", s)
  }
}

#[derive(Ord, Eq, PartialEq, PartialOrd, Debug)]
pub struct Position {
  pub status: Status,
  coordinate: Coordinate,
  ship_id: Option<String>,
}

impl Position {
  fn new(coordinate: Coordinate) -> Self {
    Self {
      coordinate,
      status: Status::SPACE,
      ship_id: None,
    }
  }
}

impl Display for Position {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.status.as_char())
  }
}

pub type Coordinate = (usize, usize);

#[derive(Ord, Eq, PartialEq, PartialOrd)]
struct Ship {
  //   coordinate: Coordinate,
  //   positions: BTreeSet<Position>,
  id: String,
  rotation: u16,
  alive: bool,
  ship_type: ShipType,
}

const ROTATIONS: [u16; 4] = [90, 180, 270, 360];

impl Ship {
  fn new(ship_type: ShipType) -> Self {
    Self {
      //   coordinate: (0, 0),
      //   positions: (),
      id: Uuid::new_v4().to_string(),
      rotation: ROTATIONS.choose(&mut rand::thread_rng()).map_or(0, |r| *r),
      alive: true,
      ship_type,
    }
  }

  fn shape(&self) -> ShipShape {
    self.ship_type.get_shape(self.rotation)
  }

  fn is_overlapping(&self, positions: &Vec<Vec<Position>>, start_cord: Coordinate) -> bool {
    let mut ship_found = false;
    if !positions.is_empty() && !positions[0].is_empty() {
      let mut x = start_cord.0;
      for row in self.shape() {
        let mut y = start_cord.1;
        for _ in row {
          if positions[x][y].status == Status::LIVE {
            ship_found = true;
          }
          y += 1;
        }
        x += 1;
      }
    }
    ship_found
  }

  fn draw(&self, positions: &mut Vec<Vec<Position>>, start_cord: Coordinate) -> bool {
    let mut ship_drawn = false;
    if !positions.is_empty() && !positions[0].is_empty() {
      let shape = self.shape();

      let mut x = start_cord.0;
      for row in shape {
        let mut y = start_cord.1;
        for col in row {
          if Status::LIVE == Status::from_char(col) {
            positions[x][y].status = Status::LIVE;
            ship_drawn = true
          }
          y += 1;
        }
        x += 1;
      }
    }
    ship_drawn
  }
}

#[derive(Clone, Ord, Eq, PartialEq, PartialOrd)]
enum ShipType {
  X,
  V,
  H,
  I,
}

type ShipShape = [[char; SHIP_SIZE]; SHIP_SIZE];

impl ShipType {
  fn get_shape(&self, rotation: u16) -> ShipShape {
    let shape = match *self {
      ShipType::X => [
        [
          Status::LIVE.as_char(),
          Status::SPACE.as_char(),
          Status::LIVE.as_char(),
        ],
        [
          Status::SPACE.as_char(),
          Status::LIVE.as_char(),
          Status::SPACE.as_char(),
        ],
        [
          Status::LIVE.as_char(),
          Status::SPACE.as_char(),
          Status::LIVE.as_char(),
        ],
      ],
      ShipType::V => [
        [
          Status::LIVE.as_char(),
          Status::SPACE.as_char(),
          Status::LIVE.as_char(),
        ],
        [
          Status::LIVE.as_char(),
          Status::SPACE.as_char(),
          Status::LIVE.as_char(),
        ],
        [
          Status::SPACE.as_char(),
          Status::LIVE.as_char(),
          Status::SPACE.as_char(),
        ],
      ],
      ShipType::H => [
        [
          Status::LIVE.as_char(),
          Status::SPACE.as_char(),
          Status::LIVE.as_char(),
        ],
        [
          Status::LIVE.as_char(),
          Status::LIVE.as_char(),
          Status::LIVE.as_char(),
        ],
        [
          Status::LIVE.as_char(),
          Status::SPACE.as_char(),
          Status::LIVE.as_char(),
        ],
      ],
      ShipType::I => [
        [
          Status::SPACE.as_char(),
          Status::LIVE.as_char(),
          Status::SPACE.as_char(),
        ],
        [
          Status::SPACE.as_char(),
          Status::LIVE.as_char(),
          Status::SPACE.as_char(),
        ],
        [
          Status::SPACE.as_char(),
          Status::LIVE.as_char(),
          Status::SPACE.as_char(),
        ],
      ],
    };

    match rotation {
      180 => reverse_cols_of_rows(transpose(shape)),
      270 => reverse_rows_of_cols(reverse_cols_of_rows(shape)),
      360 => reverse_rows_of_cols(transpose(shape)),
      _ => shape,
    }
  }

  fn get_initial_ships() -> [ShipType; 4] {
    [Self::X, Self::V, Self::H, Self::I]
  }
}

fn get_random_coordinate(rng: &mut ThreadRng) -> Coordinate {
  (
    rng.gen_range(0..(ROWS - SHIP_SIZE)),
    rng.gen_range(0..(COLUMNS - SHIP_SIZE)),
  )
}
/**
 * transpose a 2D char array.
 */
fn transpose(inp: ShipShape) -> ShipShape {
  if inp.is_empty() {
    //empty or unset array, nothing do to here
    return inp;
  }

  let rows = inp.len();
  let cols = inp[0].len();

  let mut out = inp.clone();

  for x in 0..rows {
    for y in 0..cols {
      out[y][x] = inp[x][y];
    }
  }
  return out;
}

/**
 * reverse columns of each rows in a 2d array.
 */
fn reverse_cols_of_rows(inp: ShipShape) -> ShipShape {
  if inp.is_empty() {
    //empty or unset array, nothing do to here
    return inp;
  }
  let rows = inp.len();
  let cols = inp[0].len();
  let mut out = inp.clone();

  for x in 0..rows {
    for y in 0..cols {
      out[x][cols - y - 1] = inp[x][y];
    }
  }
  return out;
}

/**
 * reverse rows of each column in a 2d array.
 */
fn reverse_rows_of_cols(inp: ShipShape) -> ShipShape {
  if inp.is_empty() {
    //empty or unset array, nothing do to here
    return inp;
  }

  let rows = inp.len();
  let cols = inp[0].len();
  let mut out = inp.clone();

  for x in 0..rows {
    for y in 0..cols {
      out[rows - x - 1][y] = inp[x][y];
    }
  }
  return out;
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn test_get_random_coordinate() {
    let mut rng = rand::thread_rng();
    assert!(get_random_coordinate(&mut rng) < (ROWS, COLUMNS));
  }
  #[test]
  fn test_reverse_rows_of_cols() {
    #[rustfmt::skip]
    let ship = [
        ['*', '*', '-'], 
        ['-', '*', '-'], 
        ['-', '-', '*']
    ];
    #[rustfmt::skip]
    let expected = [
        ['-', '-', '*'], 
        ['-', '*', '-'], 
        ['*', '*', '-']
    ];
    assert_eq!(reverse_rows_of_cols(ship), expected);
  }

  #[test]
  fn test_reverse_cols_of_rows() {
    #[rustfmt::skip]
    let ship = [
        ['*', '*', '-'], 
        ['-', '*', '-'], 
        ['-', '-', '-']
    ];
    #[rustfmt::skip]
    let expected = [
        ['-', '*', '*'], 
        ['-', '*', '-'], 
        ['-', '-', '-']
    ];
    assert_eq!(reverse_cols_of_rows(ship), expected);
  }

  #[test]
  fn test_transpose() {
    #[rustfmt::skip]
    let ship = [
        ['*', '*', '-'], 
        ['-', '*', '-'], 
        ['-', '-', '-']
    ];
    #[rustfmt::skip]
    let expected = [
        ['*', '-', '-'], 
        ['*', '*', '-'], 
        ['-', '-', '-']
    ];
    assert_eq!(transpose(ship), expected);
  }

  #[test]
  fn test_ship_type_get_shape() {
    let ship = ShipType::H;
    #[rustfmt::skip]
    assert_eq!(ship.get_shape(90), [
        ['*', '.', '*'], 
        ['*', '*', '*'], 
        ['*', '.', '*']
    ]);
    #[rustfmt::skip]
    assert_eq!(ship.get_shape(180), [
        ['*', '*', '*'], 
        ['.', '*', '.'], 
        ['*', '*', '*']
    ]);
    let ship = ShipType::V;
    #[rustfmt::skip]
    assert_eq!(ship.get_shape(270), [
        ['.', '*', '.'], 
        ['*', '.', '*'], 
        ['*', '.', '*']
    ]);
    #[rustfmt::skip]
    assert_eq!(ship.get_shape(360), [
        ['*', '*', '.'], 
        ['.', '.', '*'], 
        ['*', '*', '.']
    ]);
  }

  #[test]
  fn test_ship_is_overlapping() {
    let ship = Ship::new(ShipType::H);

    assert!(!ship.is_overlapping(&vec![], (0, 0)));
    assert!(!ship.is_overlapping(&vec![vec![]], (0, 0)));

    let mut positions = (0..ROWS)
      .map(|r| {
        (0..COLUMNS)
          .map(|c| Position::new((r, c)))
          .collect::<Vec<_>>()
      })
      .collect::<Vec<_>>();
    // should pass as there is no overlap in default
    assert!(!ship.is_overlapping(&positions, (0, 0)));

    positions[1][5] = Position {
      coordinate: (1, 5),
      ship_id: Some("123".into()),
      status: Status::LIVE,
    };
    // should fail when there is overlap
    assert!(ship.is_overlapping(&positions, (1, 5)));
  }

  #[test]
  fn test_ship_draw() {
    let ship = Ship {
      id: "123".into(),
      rotation: 90,
      alive: true,
      ship_type: ShipType::H,
    };
    let mut positions = (0..ROWS)
      .map(|r| {
        (0..COLUMNS)
          .map(|c| Position::new((r, c)))
          .collect::<Vec<_>>()
      })
      .collect::<Vec<_>>();
    assert!(ship.draw(&mut positions, (5, 5)));
    let p = positions
      .iter()
      .map(|row| {
        row
          .iter()
          .map(|c| c.to_string())
          .collect::<Vec<_>>()
          .join("")
      })
      .collect::<Vec<_>>()
      .join("\n");
    assert_eq!(p, "..........\n..........\n..........\n..........\n..........\n.....*.*..\n.....***..\n.....*.*..\n..........\n..........  ");
    assert!(ship.is_overlapping(&positions, (5, 5)));
  }

  #[test]
  fn test_board_new() {
    let opponent_board = Board::new(false);

    // should be empty board initially
    assert_eq!(opponent_board.to_string(), "..........\n..........\n..........\n..........\n..........\n..........\n..........\n..........\n..........\n..........");

    let my_board = Board::new(true);

    // should be empty board initially
    assert_eq!(my_board.ships.len(), 4);
    assert_eq!(my_board.positions.len(), ROWS);
    // check if all ships are placed on the board
    my_board.ships.iter().for_each(|it| {
      let found = my_board
        .positions
        .iter()
        .find(|pr| {
          pr.iter()
            .find(|pc| pc.ship_id.is_some() && pc.ship_id.clone().unwrap() == it.id)
            .is_some()
        })
        .and_then(|o| {
          o.iter()
            .find(|pc| pc.ship_id.is_some() && pc.ship_id.clone().unwrap() == it.id)
        });
      assert!(found.is_some(), "ship not placed!");
    })
  }
}
