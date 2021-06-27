use std::{
  collections::{BTreeMap, BTreeSet},
  fmt::{self, Display},
  usize,
};

use rand::{prelude::ThreadRng, seq::SliceRandom, Rng};
use structopt::clap::arg_enum;
use uuid::Uuid;

pub const ROWS: usize = 10;
pub const COLUMNS: usize = 10;
pub const SHIP_SIZE: usize = 3;

pub struct Game {
  players: [Player; 2],
  winner: Option<usize>,
  turn: usize,
  rule: Rule,
  difficulty: Difficulty,
}

impl Game {
  pub fn new(rule: Rule, difficulty: Difficulty) -> Self {
    Self {
      turn: 0,
      winner: None,
      players: [Player::new(), Player::default()],
      rule,
      difficulty,
    }
  }

  fn player_by_turn(&mut self, turn: usize) -> &mut Player {
    &mut self.players[turn]
  }

  fn generate_firing_coordinates(&mut self) -> BTreeSet<Coordinate> {
    let mut rng = rand::thread_rng();

    let number_of_shots = match self.rule {
      Rule::Default => 1,
      Rule::SuperCharge => self
        .player_by_turn(self.turn)
        .player_board()
        .ships_alive()
        .len(),
      Rule::Desperation => {
        self.player_by_turn(self.turn).opponent_board().ships.len()
          - self
            .player_by_turn(self.turn)
            .opponent_board()
            .ships_alive()
            .len()
      }
    };

    let mut shots = BTreeSet::new();

    for _ in 0..number_of_shots {
      let random_coords = if self.difficulty == Difficulty::Easy {
        get_random_coordinate(&mut rng, 0)
      } else {
        // TODO generate cords based on previous hits, skip missed/hit slots and try slots near previous hits
        (0, 0)
      };
      shots.insert(random_coords);
    }

    shots
  }

  pub fn fire(&mut self, shots: &BTreeSet<Coordinate>, bot: bool) -> String {
    let player_index = self.turn;
    let opponent_index = 1 - player_index;
    let opponent = self.player_by_turn(opponent_index);
    let opponent_board = opponent.player_board_mut();
    let (response, lost) = opponent_board.take_fire(shots);

    let player = self.player_by_turn(player_index);
    let message = player.opponent_board_mut().update_status(response, bot);
    self.turn = opponent_index;
    if lost {
      self.winner = Some(player_index);
      if bot {
        "You lost 🙁".into()
      } else {
        "You won 🙌".into()
      }
    } else {
      message
    }
  }

  pub fn bot_fire(&mut self) -> String {
    let shots = self.generate_firing_coordinates();
    self.fire(&shots, true)
  }

  pub fn is_user_turn(&self) -> bool {
    self.turn == 0
  }

  pub fn is_won(&self) -> bool {
    self.winner.is_some()
  }

  pub fn is_valid_rule(&self, existing_shots: usize) -> bool {
    match self.rule {
      Rule::Default => existing_shots < 1,
      Rule::SuperCharge => existing_shots < self.player().player_board().ships_alive().len(),
      Rule::Desperation => {
        existing_shots
          <= (self.player().opponent_board().ships.len()
            - self.player().opponent_board().ships_alive().len())
      }
    }
  }

  pub fn player(&self) -> &Player {
    &self.players[0]
  }
}

arg_enum! {
    #[derive(Ord, Eq, PartialEq, PartialOrd, Debug)]
    pub enum Rule {
      Default,     // single shots
      SuperCharge, // not more than total number of ships alive
      Desperation, // not more than number of killed ships + 1
    }
}

arg_enum! {
    #[derive(Ord, Eq, PartialEq, PartialOrd, Debug)]
    pub enum Difficulty {
        Easy, // computer generates random shots
        Hard, // computer generates shots based on analysis of hit/miss data
    }
}

#[derive(Ord, Eq, PartialEq, PartialOrd, Debug, Clone)]
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
      Status::LIVE => "",
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

#[derive(Ord, Eq, PartialEq, PartialOrd, Clone)]
pub struct Player {
  is_bot: bool,
  boards: [Board; 2],
}

impl Player {
  fn new() -> Self {
    Self {
      is_bot: false,
      boards: [Board::new(true), Board::new(false)],
    }
  }

  pub fn player_board_mut(&mut self) -> &mut Board {
    &mut self.boards[0]
  }
  pub fn opponent_board_mut(&mut self) -> &mut Board {
    &mut self.boards[1]
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

#[derive(Ord, Eq, PartialEq, PartialOrd, Clone)]
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
            let start_cords = get_random_coordinate(&mut rng, SHIP_SIZE);
            if !ship.is_overlapping(&positions, start_cords) {
              // draw ship on to board
              if ship.draw(&mut positions, start_cords) {
                ship_placed = true
              }
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

  fn ships_alive(&self) -> Vec<&Ship> {
    self.ships.iter().filter(|s| s.alive).collect::<Vec<_>>()
  }

  fn find_ship_mut(&mut self, id: String) -> Option<&mut Ship> {
    self.ships.iter_mut().find(|s| s.id == id)
  }

  fn alive_pos_by_ship(&self, id: String) -> Vec<&Position> {
    self
      .positions
      .iter()
      .flat_map(|pr| pr.iter())
      .filter(|pc| pc.ship_id.is_some() && pc.ship_id.clone().unwrap() == id)
      .filter(|pc| pc.status == Status::LIVE)
      .collect::<Vec<_>>()
  }

  fn take_fire(&mut self, shots: &BTreeSet<Coordinate>) -> (BTreeMap<Coordinate, Status>, bool) {
    let mut response = BTreeMap::new();
    for shot in shots {
      let pos = self.positions[shot.0][shot.1].clone();
      let mut status = Status::MISS;
      if pos.status == Status::LIVE {
        status = Status::HIT;
        if let Some(id) = &pos.ship_id {
          if self.alive_pos_by_ship(id.clone()).len() <= 1 {
            let ship = self.find_ship_mut(id.clone());
            if let Some(ship) = ship {
              status = Status::KILL;
              ship.alive = false;
            }
          }
        }
      }
      if pos.status != Status::HIT && pos.status != Status::KILL {
        self.positions[shot.0][shot.1].status = status.clone();
      }
      response.insert(*shot, status);
    }
    (response, self.ships_alive().is_empty())
  }

  fn update_status(&mut self, response: BTreeMap<Coordinate, Status>, bot: bool) -> String {
    let mut kill_count = 0;
    let mut hit_count = 0;
    let mut miss_count = 0;
    for (shot, status) in response {
      let mut pos = &mut self.positions[shot.0][shot.1];
      if pos.status != Status::HIT && pos.status != Status::KILL {
        pos.status = status.clone();
      }
      match status {
        Status::MISS => miss_count += 1,
        Status::HIT => hit_count += 1,
        Status::KILL => kill_count += 1,
        _ => {}
      }
    }
    let mut msg: Vec<String> = if bot {
      vec!["Computer have ".into()]
    } else {
      vec!["You have ".into()]
    };
    if kill_count > 0 {
      msg.push(format!("sunk {} ship.", kill_count));
    } else {
      msg.push(format!("{} hit.", hit_count));
    }
    if miss_count > 0 {
      msg.push(format!(
        " {} missed {}.",
        if bot { "Computer" } else { "You" },
        miss_count
      ));
    }
    msg.join("")
  }
}

impl Display for Board {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let s = self.as_grid().join("\n");
    write!(f, "{}", s)
  }
}

#[derive(Ord, Eq, PartialEq, PartialOrd, Debug, Clone)]
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

#[derive(Ord, Eq, PartialEq, PartialOrd, Clone)]
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

  fn is_overlapping(&self, positions: &[Vec<Position>], start_cord: Coordinate) -> bool {
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
            positions[x][y].ship_id = Some(self.id.to_owned());
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

fn get_random_coordinate(rng: &mut ThreadRng, threshold: usize) -> Coordinate {
  (
    rng.gen_range(0..(ROWS - threshold)),
    rng.gen_range(0..(COLUMNS - threshold)),
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

  let mut out = inp;

  for (x, cols) in inp.iter().enumerate() {
    for (y, _) in cols.iter().enumerate() {
      out[y][x] = inp[x][y];
    }
  }
  out
}

/**
 * reverse columns of each rows in a 2d array.
 */
fn reverse_cols_of_rows(inp: ShipShape) -> ShipShape {
  if inp.is_empty() {
    //empty or unset array, nothing do to here
    return inp;
  }
  let mut out = inp;

  for (x, cols) in inp.iter().enumerate() {
    for (y, _) in cols.iter().enumerate() {
      out[x][cols.len() - y - 1] = inp[x][y];
    }
  }
  out
}

/**
 * reverse rows of each column in a 2d array.
 */
fn reverse_rows_of_cols(inp: ShipShape) -> ShipShape {
  if inp.is_empty() {
    //empty or unset array, nothing do to here
    return inp;
  }

  let mut out = inp;

  for (x, cols) in inp.iter().enumerate() {
    for (y, _) in cols.iter().enumerate() {
      out[inp.len() - x - 1][y] = inp[x][y];
    }
  }
  out
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn test_game_is_valid_rule() {
    let mut game = Game::new(Rule::Default, Difficulty::Easy);
    assert!(game.is_valid_rule(0));
    assert!(!game.is_valid_rule(1));

    game.rule = Rule::SuperCharge;

    assert!(game.is_valid_rule(0));
    assert!(game.is_valid_rule(3));
    assert!(!game.is_valid_rule(4));

    game.rule = Rule::Desperation;

    assert!(game.is_valid_rule(0));
    assert!(!game.is_valid_rule(1));
  }
  #[test]
  fn test_game_fire() {
    let mut game = Game::new(Rule::Default, Difficulty::Easy);

    let mut shots = BTreeSet::new();
    shots.insert((1, 1));
    shots.insert((3, 3));

    let msg = game.fire(&shots, false);

    assert!(!msg.is_empty());
    assert!(!game.is_user_turn());
    assert!(!game.winner.is_some());
  }

  #[test]
  fn test_get_random_coordinate() {
    let mut rng = rand::thread_rng();
    assert!(get_random_coordinate(&mut rng, SHIP_SIZE) < (ROWS, COLUMNS));
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

    assert!(!ship.is_overlapping(&[], (0, 0)));
    assert!(!ship.is_overlapping(&[vec![]], (0, 0)));

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
    assert_eq!(p, "..........\n..........\n..........\n..........\n..........\n.....*.*..\n.....***..\n.....*.*..\n..........\n..........");
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
        .flat_map(|pr| pr.iter())
        .filter(|pc| pc.ship_id.is_some() && pc.ship_id.clone().unwrap() == it.id)
        .collect::<Vec<_>>();
      match it.ship_type {
        ShipType::X => assert!(found.len() == 5, "ship X not placed!"),
        ShipType::V => assert!(found.len() == 5, "ship V not placed!"),
        ShipType::H => assert!(found.len() == 7, "ship H not placed!"),
        ShipType::I => assert!(found.len() == 3, "ship I not placed!"),
      }
    })
  }

  #[test]
  fn test_board_take_fire() {
    let mut board = Board::new(true);

    board.positions[1][1].status = Status::SPACE;
    board.positions[3][3].status = Status::LIVE;

    // set a ship as hit except for one position
    let ship_id = board.ships[0].id.clone();
    let mut pos = board
      .positions
      .iter_mut()
      .flat_map(|pr| pr.iter_mut())
      .filter(|pc| pc.ship_id.is_some() && pc.ship_id.clone().unwrap() == ship_id)
      .collect::<Vec<_>>();

    pos.iter_mut().skip(1).for_each(|p| p.status = Status::HIT);

    let c = pos.iter().take(1).map(|p| p.coordinate).collect::<Vec<_>>();

    let mut shots = BTreeSet::new();
    shots.insert((1, 1));
    shots.insert((3, 3));
    shots.insert(c[0]);

    let (res, lost) = board.take_fire(&shots);
    assert_eq!(res.get(&(1, 1)).unwrap(), &Status::MISS);
    assert_eq!(res.get(&(3, 3)).unwrap(), &Status::HIT);
    assert_eq!(res.get(&c[0]).unwrap(), &Status::KILL);
    assert!(!lost);
  }

  #[test]
  fn test_board_update_status() {
    let mut board = Board::new(false);

    let mut res = BTreeMap::new();
    res.insert((1, 1), Status::MISS);
    res.insert((3, 3), Status::HIT);
    res.insert((0, 2), Status::KILL);

    let message = board.update_status(res, false);
    assert_eq!(message, "You have sunk 1 ship. You missed 1.");

    let mut res = BTreeMap::new();
    res.insert((3, 3), Status::HIT);
    res.insert((0, 2), Status::HIT);

    let message = board.update_status(res.clone(), false);
    assert_eq!(message, "You have 2 hit.");
    let message = board.update_status(res, true);
    assert_eq!(message, "Computer have 2 hit.");
  }
}
