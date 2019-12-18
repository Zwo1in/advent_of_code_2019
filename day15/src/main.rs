use itertools::{Itertools, MinMaxResult};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::thread;
use std::time::Duration;
use console::Term;
use rand::prelude::*;
use lazy_static::lazy_static;

pub mod intcode_pc;
use intcode_pc::{load_prog, IntcodePC, Message,};
use Tile::*;

const INPUT: &'static str = include_str!("../input");
lazy_static! {
    static ref TERM: Term = Term::stdout();
}

#[derive(PartialEq, Eq, Copy, Clone)]
enum Tile {
    Empty,
    Drone,
    Wall,
    OxygenStation,
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match *self {
                            Empty => '.',
                            Wall  => '#',
                            Drone => 'o',
                            OxygenStation => '@',
                        })
    }
}

impl From<i64> for Tile {
    fn from(input: i64) -> Self {
        match input {
            0 => Wall,
            1 => Empty,
            2 => OxygenStation,
            _ => panic!("Cannot convert to a Tile"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct Vec2 {
    x: i32,
    y: i32,
}

impl Vec2 {
    fn zero() -> Self {
        Self { x: 0, y: 0 }
    }

    fn apply_move(&mut self, m: Move) {
        match m {
            Move::North => self.y += 1,
            Move::South => self.y -= 1,
            Move::West  => self.x -= 1,
            Move::East  => self.x += 1,
        }
    }

    fn inspect_move(&self, m: Move) -> Vec2 {
        let (mut x, mut y) = (self.x, self.y);
        match m {
            Move::North => y += 1,
            Move::South => y -= 1,
            Move::West  => x -= 1,
            Move::East  => x += 1,
        }
        (x, y).into()
    }
}
            

impl From<(i32, i32)> for Vec2 {
    fn from((x, y): (i32, i32)) -> Self {
        Self { x, y }
    }
}

struct Board(HashMap<Vec2, Tile>);

impl Board {
    fn new() -> Self {
        Self(HashMap::new())
    }
    
    fn set_tile(&mut self, pos: Vec2, tile: Tile) {
        self.0.insert(pos, tile);
    }

    fn get(&self, x: i32, y: i32) -> Option<&Tile> {
        self.0.get(&(x, y).into())
    }

    fn get_pos(&self, pos: Vec2) -> Option<&Tile> {
        self.0.get(&pos)
    }

    fn adjanced(&self, pos: Vec2) -> Vec<Vec2> {
        (1..=4).filter_map(|num| {
                if let Some(&tile) = self.get_pos(pos.inspect_move(num.into())) {
                    if tile != Tile::Wall {
                        return Some(pos.inspect_move(num.into()))
                    }
                }
                None
            })
            .collect()
    }

    fn bounds(&self) -> (Vec2, Vec2) {
        use MinMaxResult::MinMax;
        let x_bounds = self.0.keys().minmax_by_key(|pos| pos.x);
        let y_bounds = self.0.keys().minmax_by_key(|pos| pos.y);
        let (min_x, max_x) = if let MinMax(min, max) = x_bounds {
            (min.x, max.x)
        } else {
            (-1, 1)
        };
        let (min_y, max_y) = if let MinMax(min, max) = y_bounds {
            (min.y, max.y)
        } else {
            (-1, 1)
        };
        (Vec2 { x: min_x, y: min_y }, Vec2 { x: max_x, y: max_y })
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (min, max) = self.bounds();
        let width = if max.x-min.x == 0 {
            1
        } else {
            (max.x-min.x).abs() as usize
        };
        write!(f, "{}\n", 
            (min.y..max.y).cartesian_product(min.x..max.x)
                .map(|(y, x)| {
                    format!("{}", self.get(x, y).unwrap_or(&Tile::Empty))
                })
                .chunks(width).into_iter()
                .map(|chunk| {
                    chunk.collect::<String>()
                })
                .join("\n"))
    }
}

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq)]
enum Move {
    North,
    East,
    West,
    South,
}

impl Move {
    fn opposite(self) -> Self {
        match self {
            Move::North => Move::South,
            Move::South => Move::North,
            Move::West  => Move::East,
            Move::East  => Move::West,
        }
    }

    fn as_u64(&self) -> u64 {
        match *self {
            Move::North => 1,
            Move::South => 2,
            Move::West  => 3,
            Move::East  => 4,
        }
    }
}

impl From<u64> for Move {
    fn from(input: u64) -> Self {
        match input {
            1 => Move::North,
            2 => Move::South,
            3 => Move::West,
            4 => Move::East,
            _ => unreachable!(),
        }
    }
}

struct BoardBuilder<I, O>
where
    I: Fn() -> Message,
    O: Fn(u64) -> Option<()>,
{
    drone: Vec2,
    board: Board,
    stack: Vec<Move>,
    input: I,
    output: O,
    rng: ThreadRng,
}

impl<I, O> BoardBuilder<I, O>
where
    I: Fn() -> Message,
    O: Fn(u64) -> Option<()>,
{
    fn new(input: I, output: O) -> Self {
        let rng = thread_rng();
        let stack: Vec<Move> = vec![];
        let drone = Vec2::zero();
        let mut board = Board::new();
        board.set_tile(drone, Tile::Drone);
        Self {
            drone,
            board,
            stack,
            input,
            output,
            rng,
        }
    }

    fn not_visited(&self) -> HashSet<Move> {
        (1..=4).filter_map(|mov| {
            let mov = mov.into();
            if let None = self.board.get_pos(self.drone.inspect_move(mov)) {
                Some(mov)
            } else {
                None
            }
        })
        .collect()
    }

    fn next_move(&mut self) -> (Move, bool) {
        let not_visited = self.not_visited();
        if !not_visited.is_empty() {
            loop {
                let next: u64 = self.rng.gen_range(1, 5);
                if not_visited.contains(&next.into()) {
                    break (next.into(), false)
                }
            }
        } else {
            (self.stack.pop().unwrap().opposite(), true)
        }
    }

    fn draw(&mut self) {
        let tmp = self.board.get_pos(self.drone).cloned().unwrap();
        self.board.set_tile(self.drone, Tile::Drone);
        TERM.clear_screen().unwrap();
        TERM.write_str(format!("{}", self.board).as_str()).unwrap();
        self.board.set_tile(self.drone, tmp);
    }

    fn build(mut self) -> Board {
        while let Message::NeedInput = (self.input)() {
            let (next_move, stack_popped) = self.next_move();
            if stack_popped && self.stack.is_empty() {
                break
            }
            (self.output)(next_move.as_u64());
            match (self.input)() {
                Message::Value(val) => {
                    match val {
                        0 => {
                            self.board.set_tile(
                                self.drone.inspect_move(next_move),
                                Tile::Wall
                            )
                        },
                        tile @ 1..=2 => {
                            self.drone.apply_move(next_move);
                            self.board.set_tile(
                                self.drone,
                                tile.into()
                            );
                            if !stack_popped {
                                self.stack.push(next_move);
                            }
                        },
                        _ => unreachable!(),
                    }
                },
                Message::Finished => break,
                Message::NeedInput => unreachable!(),
            }
            //thread::sleep(Duration::from_millis(5));
            //self.draw();
        }
        self.board
    }
}

fn bfs_oxygen(board: &Board) -> u32 {
    let mut seen: HashSet<Vec2> = HashSet::new();
    let mut next: HashSet<Vec2> = HashSet::new();
    let target = board.0.iter()
        .find(|(_, &v)| v == Tile::OxygenStation)
        .map(|(k, _)| k)
        .unwrap();
    let mut iteration = 0;
    next.insert(Vec2::zero());
    loop {
        iteration += 1;
        next = next.iter()
            .flat_map(|&pos| board.adjanced(pos))
            .filter(|pos| !seen.contains(pos))
            .collect();
        if let Some(_) = next.iter().find(|&pos| pos == target) {
            break
        }
        seen = seen.union(&next).cloned().collect();
    }
    iteration
}

fn bfs_whole_place(board: &Board) -> u32 {
    let mut seen: HashSet<Vec2> = HashSet::new();
    let mut next: HashSet<Vec2> = HashSet::new();
    let start = board.0.iter()
        .find(|(_, &v)| v == Tile::OxygenStation)
        .map(|(k, _)| k)
        .cloned()
        .unwrap();
    let mut iteration = 0;
    next.insert(start);
    loop {
        next = next.iter()
            .flat_map(|&pos| board.adjanced(pos))
            .filter(|pos| !seen.contains(pos))
            .collect();
        if next.is_empty() {
            break
        }
        iteration += 1;
        seen = seen.union(&next).cloned().collect();
    }
    iteration
}

fn main() {
    let prog = load_prog(INPUT);
    let (pc, pc_in, pc_out) = IntcodePC::new(prog);

    let pc_handle = thread::spawn(move || pc.run());

    let input  = || pc_out.recv().unwrap_or(Message::Finished);
    let output = |num: u64| pc_in.send(Message::Value(num as i64)).ok();
    let board = BoardBuilder::new(input, output).build();
    println!("{}", board);
    println!("{}", bfs_oxygen(&board));
    println!("{}", bfs_whole_place(&board));
    //pc_handle.join().unwrap();
}
