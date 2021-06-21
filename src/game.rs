use std::{
  collections::{BTreeMap, BTreeSet},
  usize,
};

pub struct Game {
  next_player: PlayerType,
  winner: Option<Player>,
  game_over: bool,
  player_one: Player,
  player_two: Player,
  rule: Rule,
}

impl Game {
  pub fn new() -> Self {
    Self {
      next_player: PlayerType::Me,
      winner: None,
      game_over: false,
      player_one: Player::new("me".into()),
      player_two: Player::default(),
      rule: Rule::Default,
    }
  }
}

enum Rule {
  Default,
}

#[derive(Ord, Eq, PartialEq, PartialOrd)]
enum Status {
  LIVE,  //'*'
  MISS,  //'-'
  HIT,   //'X'
  KILL,  //'K'
  SPACE, //'.'
}
#[derive(Ord, Eq, PartialEq, PartialOrd)]
enum PlayerType {
  Me,
  Opponent,
}

#[derive(Ord, Eq, PartialEq, PartialOrd)]
struct Player {
  name: String,
  is_bot: bool,
  board: Board,
  opponent_board: Board,
  extra_shots: bool,
}

impl Player {
  fn new(name: String) -> Self {
    Self {
      name,
      is_bot: false,
      board: Board::new(),
      opponent_board: Board::new(),
      extra_shots: false,
    }
  }
}

impl Default for Player {
  fn default() -> Self {
    Self {
      name: "computer".into(),
      is_bot: true,
      board: Board::new(),
      opponent_board: Board::new(),
      extra_shots: false,
    }
  }
}

#[derive(Ord, Eq, PartialEq, PartialOrd)]
struct Ship {
  x: usize,
  y: usize,
  rotation: usize,
  alive: bool,
  positions: BTreeSet<Position>,
  shape: Vec<Vec<char>>,
  ship_type: ShipType,
  rotated: bool,
}

#[derive(Ord, Eq, PartialEq, PartialOrd)]
enum ShipType {}

#[derive(Ord, Eq, PartialEq, PartialOrd)]
struct Position {
  x: usize,
  y: usize,
  status: Status,
  ship: Ship,
}

#[derive(Ord, Eq, PartialEq, PartialOrd)]
struct Board {
  ships: BTreeSet<Ship>,
  grid: Vec<char>,
  firing_status: BTreeMap<String, String>,
  positions: Vec<Vec<Position>>,
}

impl Board {
  fn new() -> Self {
    Self {
      ships: BTreeSet::new(),
      grid: vec![],
      firing_status: BTreeMap::new(),
      positions: vec![],
    }
  }
}
