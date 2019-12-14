use itertools::Itertools;
use std::collections::HashMap;
use std::fmt;
use std::thread;
use std::time::Duration;
use console::{Term, Key};

pub mod intcode_pc;
use intcode_pc::{load_prog, IntcodePC, Message,};
use Tile::*;

const INPUT: &'static str = include_str!("../input");

#[derive(PartialEq, Eq, Copy, Clone)]
enum Tile {
    Empty,
    Wall,
    Block,
    HorizontalPaddle,
    Ball,
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match *self {
                            Empty => ' ',
                            Wall  => '#',
                            Block => 'x',
                            HorizontalPaddle => '-',
                            Ball  => 'o',
                        })
    }
}

impl From<i64> for Tile {
    fn from(input: i64) -> Self {
        match input {
            0 => Empty,
            1 => Wall,
            2 => Block,
            3 => HorizontalPaddle,
            4 => Ball,
            _ => panic!("Cannot convert to a Tile"),
        }
    }
}

struct Board {
    width: usize,
    height: usize,
    area: HashMap<(usize, usize), Tile>,
}

impl Board {
    fn new() -> Self {
        let width = 1;
        let height = 1;
        let area = HashMap::new();
        Self { width, height, area }
    }
    
    fn set_tile(&mut self, x: usize, y: usize, tile: Tile) {
        if x >= self.width {
            self.width = x + 1;
        }
        if y >= self.height {
            self.height = y + 1;
        }
        self.area.insert((x, y), tile);
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\n", 
            (0..self.height).cartesian_product(0..self.width)
                .map(|(y, x)| {
                    format!("{}", self.area.get(&(x, y)).unwrap_or(&Tile::Empty))
                })
                .chunks(self.width).into_iter()
                .map(|chunk| {
                    chunk.collect::<String>()
                })
                .join("\n"))
    }
}

struct Game<In, Out>
where
    In: Fn() -> Message,
    Out: Fn(i64) -> Option<()>,
{
    board:  Board,
    score:  i64,
    input:  In,
    output: Out,
    term: Term,
}

enum GameState {
    Score,
    State,
    Finished,
    NeedInput,
}

impl<In, Out> Game<In, Out>
where
    In: Fn() -> Message,
    Out: Fn(i64) -> Option<()>,
{
    fn new(input: In, output: Out) -> Self {
        Self {
            board: Board::new(),
            score: 0,
            input,
            output,
            term: Term::stdout(),
        }
    }

    fn run(&mut self) {
        loop {
            let input_result = self.handle_input();
            if let GameState::State = input_result {
                continue
            } else if let GameState::Score = input_result {
                break
            } else {
                return
            }
        }
        self.draw();
        loop {
            let input = self.handle_input();
            match input {
                GameState::Finished => break,
                GameState::NeedInput => self.simulate_user(),
                _ => (),
            }
            self.draw();
        }
    }

    fn handle_input(&mut self) -> GameState {
        let msg = (self.input)();
        match msg {
            Message::Finished  => GameState::Finished,
            Message::NeedInput => {
                println!("\nNeedInput");
                GameState::NeedInput
            },
            Message::Value(v) => {
                if v == -1 {
                    self.get_score();
                    GameState::Score
                } else {
                    self.update_tile(v);
                    GameState::State
                }
            },
        }
    }

    fn get_score(&mut self) {
        assert_eq!((self.input)().unwrap_val(), 0);
        self.score = (self.input)().unwrap_val();
    }

    fn update_tile(&mut self, x: i64) {
        let y = (self.input)().unwrap_val();
        let tile = (self.input)().unwrap_val();
        self.board.set_tile(x as usize, y as usize, tile.into());
    }

    #[allow(unused)]
    fn simulate_user(&self) {
        use std::cmp::Ordering::*;
        thread::sleep(Duration::from_millis(20));
        let ball_x = self.board.area.iter().filter(|(_, &v)| v == Tile::Ball)
            .map(|(k, _)| k.0)
            .next().unwrap();
        let paddle_x = self.board.area.iter().filter(|(_, &v)| v == Tile::HorizontalPaddle)
            .map(|(k, _)| k.0)
            .next().unwrap();
        let _ = match paddle_x.cmp(&(ball_x)) {
            Less => (self.output)(1),
            Equal => (self.output)(0),
            Greater => (self.output)(-1),
        };
    }

    #[allow(unused)]
    fn handle_user_movement(&self) {
        let key = self.term.read_key().unwrap();
        if key == Key::ArrowLeft {
            let _ = (self.output)(-1);
        } else if key == Key::ArrowRight {
            let _ = (self.output)(1);
        } else {
            let _ = (self.output)(0);
        }
    }

    fn draw(&self) {
        let b = &format!("{}", self.board)[..];
        let s = &format!("Score: {}", self.score)[..];
        self.term.clear_screen().unwrap();
        self.term.write_str(b).unwrap();
        self.term.write_str(s).unwrap();
    }
}


fn main() {
    let mut prog = load_prog(INPUT);
    prog[0] = 2;
    let (pc, pc_in, pc_out) = IntcodePC::new(prog);

    let pc_handle = thread::spawn(move || pc.run());

    let input  = || pc_out.recv().unwrap_or(Message::Finished);
    let output = |num: i64| pc_in.send(Message::Value(num)).ok();

    let mut game = Game::new(input, output);
    game.run();
    pc_handle.join().unwrap();
}
