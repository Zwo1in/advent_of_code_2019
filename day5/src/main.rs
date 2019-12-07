const INPUT: &str = include_str!("../input");
fn load_prog() -> Vec<i32> {
    INPUT.split(",").map(|s| {
        s.trim().parse::<i32>().unwrap()
    }).collect()
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Mode {
    Immediate,
    Position,
}

trait AsMode {
    fn as_mode(self) -> Mode;
}

impl AsMode for u32 {
    fn as_mode(self) -> Mode {
        match self {
            0 => Mode::Position,
            1 => Mode::Immediate,
            _ => panic!("Wrong mode"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Order {
    ord: i32,
    params: [Mode; 3],
}

impl Order {
    fn new(raw: i32) -> Self {
        let ord = raw%100;
        let raw = raw.to_string();
        let mut params = raw.chars().rev().skip(2).map(|c| c.to_digit(10).unwrap());
        let params = [params.next().unwrap_or(0).as_mode(),
                      params.next().unwrap_or(0).as_mode(),
                      params.next().unwrap_or(0).as_mode()];
        Self { ord, params }
    }

    fn execute(self, code: &mut Vec<i32>, pc: &mut usize, input: i32) -> Option<i32> {
        let args_len = match self.ord {
            1..=2 => 3,
            3..=4 => 1,
            5..=6 => 2,
            7..=8 => 3,
            _ => panic!("Wrong order"),
        };
        let args: Vec<_> = code[*pc+1..=*pc+args_len]
            .iter().enumerate()
            .map(|(n, arg)| {
                if n == args_len-1 && self.ord != 4 && self.ord != 5 && self.ord != 6 {
                    *arg
                } else {
                    match self.params[n] {
                        Mode::Position => code[*arg as usize],
                        Mode::Immediate => *arg,
                    }
                }
            })
            .collect();
        match self.ord {
            1 => { code[args[2] as usize] = args[0] + args[1]; *pc += 4; },
            2 => { code[args[2] as usize] = args[0] * args[1]; *pc += 4; },
            3 => { code[args[0] as usize] = input; *pc += 2; },
            4 => { *pc += 2; return Some(args[0]); },
            5 => { if args[0] != 0 { *pc = args[1] as usize } else { *pc += 3; } },
            6 => { if args[0] == 0 { *pc = args[1] as usize } else { *pc += 3; } },
            7 => { code[args[2] as usize] = if args[0] < args[1] { 1 } else { 0 }; *pc += 4; },
            8 => { code[args[2] as usize] = if args[0] == args[1] { 1 } else { 0 }; *pc += 4; },
            _ => (),
        }
        None
    }
}

fn intcode(mut code: Vec<i32>, input: i32) -> (Vec<i32>, Vec<i32>) {
    let mut pc = 0;
    let mut output = vec![];
    while code[pc] != 99 {
        let order = Order::new(code[pc]);
        if let Some(ret) = order.execute(&mut code, &mut pc, input) {
            output.push(ret);
        }
    }
    (code, output)
}

fn main() {
    let prog = load_prog();
    let (_, output) = intcode(prog, 5);
    println!("{:?}", output);
}

#[cfg(test)]
mod tests {
    use super::{*, Mode::*};

    #[test]
    fn order_parse() {
        let code = 11102;
        assert_eq!(Order::new(code),
             Order { ord: 2, params: [Immediate, Immediate, Immediate] });
    }

    #[test]
    fn order_parse_default_params() {
        let code = 1001;
        assert_eq!(Order::new(code),
             Order { ord: 1, params: [Position, Immediate, Position] });
    }

    #[test]
    fn input_output() {
        let prog = vec![3,0,4,0,99];
        assert_eq!(intcode(prog, 15).1, [15]);
    }

    #[test]
    fn first_prog() {
        let prog = vec![1,0,0,0,99];
        assert_eq!(intcode(prog, 0).0, [2,0,0,0,99]);
    }

    #[test]
    fn second_prog() {
        let prog = vec![2,3,0,3,99];
        assert_eq!(intcode(prog, 0).0, [2,3,0,6,99]);
    }

    #[test]
    fn third_prog() {
        let prog = vec![2,4,4,5,99,0];
        assert_eq!(intcode(prog, 0).0, [2,4,4,5,99,9801]);
    }

    #[test]
    fn fourth_prog() {
        let prog = vec![1,1,1,4,99,5,6,0,99];
        assert_eq!(intcode(prog, 0).0, [30,1,1,4,2,5,6,0,99]);
    }

    #[test]
    fn position_mode_78_1() {
        let prog = vec![3,9,8,9,10,9,4,9,99,-1,8];
        assert_eq!(intcode(prog.clone(), 8).1, [1]);
        assert_eq!(intcode(prog, 6).1, [0]);
    }

    #[test]
    fn position_mode_78_2() {
        let prog = vec![3,9,7,9,10,9,4,9,99,-1,8];
        assert_eq!(intcode(prog.clone(), 6).1, [1]);
        assert_eq!(intcode(prog, 9).1, [0]);
    }

    #[test]
    fn immediate_mode_78_1() {
        let prog = vec![3,3,1108,-1,8,3,4,3,99];
        assert_eq!(intcode(prog.clone(), 8).1, [1]);
        assert_eq!(intcode(prog, 6).1, [0]);
    }

    #[test]
    fn immediate_mode_78_2() {
        let prog = vec![3,3,1107,-1,8,3,4,3,99];
        assert_eq!(intcode(prog.clone(), 5).1, [1]);
        assert_eq!(intcode(prog, 9).1, [0]);
    }

    #[test]
    fn jump_test_position_mode() {
        let prog = vec![3,12,6,12,15,1,13,14,13,4,13,99,-1,0,1,9];
        assert_eq!(intcode(prog.clone(), 0).1, [0]);
        assert_eq!(intcode(prog, 1).1, [1]);
    }

    #[test]
    fn jump_test_immediate_mode() {
        let prog = vec![3,3,1105,-1,9,1101,0,0,12,4,12,99,1];
        assert_eq!(intcode(prog.clone(), 0).1, [0]);
        assert_eq!(intcode(prog, 1).1, [1]);
    }
}
