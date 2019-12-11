use std::sync::mpsc::{Sender, Receiver};
use std::collections::HashMap;
use std::thread;

use itertools::{Itertools, MinMaxResult};

mod intcode_pc;
use intcode_pc::IntcodePC;

const INPUT: &'static str = include_str!("../input");

fn load_prog() -> Vec<i64> {
    INPUT.split(",").map(|s| {
        s.trim().parse::<i64>().unwrap()
    }).collect()
}

enum Dir {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug)]
enum Turn {
    Left,
    Right,
}

impl From<i64> for Turn {
    fn from(val: i64) -> Self {
        match val {
            0 => Turn::Left,
            1 => Turn::Right,
            _ => panic!("Wrong turn value"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Color {
    Black,
    White,
}

impl Color {
    fn as_i64(self) -> i64 {
        match self {
            Color::White => 1,
            Color::Black => 0,
        }
    }
}

impl From<i64> for Color {
    fn from(val: i64) -> Self {
        match val {
            0 => Color::Black,
            1 => Color::White,
            _ => panic!("Wrong color value"),
        }
    }
}

struct Board(HashMap<Pos, Color>);

impl Board {
    fn print(self) {
        use MinMaxResult::MinMax;
        let (min_x, max_x) = if let MinMax(l, h) = self.0.keys().minmax_by_key(|k| k.x) {
            (l.x, h.x)
        } else {
            return;
        };
        let (min_y, max_y) = if let MinMax(l, h) = self.0.keys().minmax_by_key(|k| k.y) {
            (l.y, h.y)
        } else {
            return;
        };
        (min_y..=max_y).rev().cartesian_product(min_x..=max_x)
            .for_each(|(y, x)| {
                let color = if let Some(c) = self.0.get(&Pos { x, y }) {
                    *c
                } else {
                    Color::Black
                };
                print!("{}", if color == Color::Black { ' ' } else { '#' });
                if x == max_x {
                    println!("");
                }
            });
    }
}
        

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
struct Pos {
    x: i64,
    y: i64,
}

impl Dir {
    fn turn(&mut self, side: Turn) {
        use Dir::*;
        *self = match *self {
            Up => match side {
                Turn::Left => Left,
                Turn::Right => Right,
            },
            Down => match side {
                Turn::Left => Right,
                Turn::Right => Left,
            },
            Left => match side {
                Turn::Left => Down,
                Turn::Right => Up,
            },
            Right => match side {
                Turn::Left => Up,
                Turn::Right => Down,
            },
        };
    }
}

struct Picasso {
    dir: Dir,
    pos: Pos,
    pc_in: Sender<i64>,
    pc_out: Receiver<i64>,
}

impl Picasso {
    fn new(pc_in: Sender<i64>, pc_out: Receiver<i64>) -> Self {
        Self {
            pos: Pos { x: 0, y: 0 },
            dir: Dir::Up,
            pc_in,
            pc_out,
        }
    }
    
    fn move_forward(&mut self) {
        match self.dir {
            Dir::Up => self.pos.y += 1,
            Dir::Down => self.pos.y -= 1,
            Dir::Left => self.pos.x -= 1,
            Dir::Right => self.pos.x += 1,
        }
    }

    fn run(&mut self, board: &mut Board) {
        board.0.insert(self.pos, Color::White); // For part 2
        loop {
            let tile_color = if let Some(color) = board.0.get(&self.pos) {
                *color
            } else {
                Color::Black
            };
            if let None = self.pc_in.send(tile_color.as_i64()).ok() {
                break;
            }
            if let Some(desired_color) = self.pc_out.recv().ok() {
                board.0.insert(self.pos, desired_color.into());
            } else {
                break;
            }
            if let Some(turn) = self.pc_out.recv().ok() {
                self.dir.turn(turn.into());
                self.move_forward();
            } else {
                break;
            }
        }
    }
}

fn main() {
    let prog = load_prog();
    let mut board = Board(HashMap::new());
    let (pc, input, output) = IntcodePC::new(prog);
    let mut picasso = Picasso::new(input, output);
    let pc_handle = thread::spawn(move || pc.run());
    picasso.run(&mut board);
    board.print();
    pc_handle.join().unwrap();
}
