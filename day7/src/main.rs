use itertools::Itertools;

use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use std::collections::VecDeque;

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

    fn execute(self, code: &mut Vec<i32>, pc: &mut usize, input: &Receiver<i32>) -> Option<i32> {
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
                if n == args_len-1 && self.ord !=4 && self.ord != 5 && self.ord != 6 {
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
            1 => {
                code[args[2] as usize] = args[0] + args[1];
                *pc += 4;
            },
            2 => {
                code[args[2] as usize] = args[0] * args[1];
                *pc += 4;
            },
            3 => {
                let inp = input.recv().unwrap();
                code[args[0] as usize] = inp;
                *pc += 2;
            },
            4 => {
                *pc += 2;
                return Some(args[0]);
            },
            5 => {
                if args[0] != 0 { *pc = args[1] as usize } else { *pc += 3; }
            },
            6 => {
                if args[0] == 0 { *pc = args[1] as usize } else { *pc += 3; }
            },
            7 => {
                code[args[2] as usize] = if args[0] < args[1] { 1 } else { 0 };
                *pc += 4;
            },
            8 => {
                code[args[2] as usize] = if args[0] == args[1] { 1 } else { 0 };
                *pc += 4;
            },
            _ => (),
        }
        None
    }
}

struct IntcodePC {
    program: Vec<i32>,
    input: Receiver<i32>,
    output: Sender<i32>,
}

impl IntcodePC {
    #[allow(unused)]
    fn new(program: Vec<i32>) -> (Self, Sender<i32>, Receiver<i32>) {
        let (in_sender, in_receiver) = channel();
        let (out_sender, out_receiver) = channel();
        (Self { program, input: in_receiver, output: out_sender },
         in_sender, out_receiver)
    } 

    fn piped(program: Vec<i32>, input: Receiver<i32>) -> (Self, Receiver<i32>) {
        let (out_sender, out_receiver) = channel();
        (Self { program, input, output: out_sender }, out_receiver)
    } 

    fn run(mut self) -> (Vec<i32>, Vec<i32>) {
        let mut pc = 0;
        let mut outputs = vec![];
        while self.program[pc] != 99 {
            let order = Order::new(self.program[pc]);
            if let Some(ret) = order.execute(&mut self.program, &mut pc, &self.input) {
                outputs.push(ret);
                self.output.send(ret).unwrap_or_else(|_| ());
            }
        }
        (self.program, outputs)
    }
}

fn max_thruster_signal(prog: Vec<i32>, feedback: bool) -> i32 {
    if !feedback { (0..5) } else { (5..10) }.permutations(5).map(|setup| {
        let (input, mut output) = channel();
        let mut amps = vec![];
        for _ in 0..5 {
            let amp = IntcodePC::piped(prog.clone(), output);
            amps.push(amp.0);
            output = amp.1;
        }
        let mut inputs: VecDeque<_> = amps.iter().map(|amp| amp.output.clone()).collect();
        inputs.rotate_right(1);
        if feedback {
            amps[4].output = input.clone();
        }
        inputs[0] = input;
        let mut handles = vec![];
        for amp in amps {
            handles.push(thread::spawn(move || amp.run()));
        }
        for (input, seq) in inputs.iter().zip(setup.iter()) {
            input.send(*seq).unwrap();
        }
        inputs[0].send(0).unwrap();
        handles.into_iter().map(|h| *h.join().unwrap().1.last().unwrap()).last().unwrap()
    })
    .max().unwrap()
}
        

fn main() {
    let prog = load_prog();
    println!("{}", max_thruster_signal(prog, true));
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
        let (intcode_pc, input, output) = IntcodePC::new(prog); 
        thread::spawn(move || intcode_pc.run());
        input.send(15).unwrap();
        assert_eq!(output.recv().unwrap(), 15);
    }

    #[test]
    fn first_prog() {
        let prog = vec![1,0,0,0,99];
        let (intcode_pc, _, _) = IntcodePC::new(prog); 
        assert_eq!(intcode_pc.run().0, [2,0,0,0,99]);
    }

    #[test]
    fn second_prog() {
        let prog = vec![2,3,0,3,99];
        let (intcode_pc, _, _) = IntcodePC::new(prog); 
        assert_eq!(intcode_pc.run().0, [2,3,0,6,99]);
    }

    #[test]
    fn third_prog() {
        let prog = vec![2,4,4,5,99,0];
        let (intcode_pc, _, _) = IntcodePC::new(prog); 
        assert_eq!(intcode_pc.run().0, [2,4,4,5,99,9801]);
    }

    #[test]
    fn fourth_prog() {
        let prog = vec![1,1,1,4,99,5,6,0,99];
        let (intcode_pc, _, _) = IntcodePC::new(prog); 
        assert_eq!(intcode_pc.run().0, [30,1,1,4,2,5,6,0,99]);
    }

    #[test]
    fn position_mode_78_1() {
        let prog = vec![3,9,8,9,10,9,4,9,99,-1,8];
        let (intcode_pc, input, output) = IntcodePC::new(prog.clone()); 
        thread::spawn(move || intcode_pc.run());
        input.send(8).unwrap();
        assert_eq!(output.recv().unwrap(), 1);

        let (intcode_pc, input, output) = IntcodePC::new(prog); 
        thread::spawn(move || intcode_pc.run());
        input.send(6).unwrap();
        assert_eq!(output.recv().unwrap(), 0);
    }

    #[test]
    fn position_mode_78_2() {
        let prog = vec![3,9,7,9,10,9,4,9,99,-1,8];
        let (intcode_pc, input, output) = IntcodePC::new(prog.clone()); 
        thread::spawn(move || intcode_pc.run());
        input.send(6).unwrap();
        assert_eq!(output.recv().unwrap(), 1);

        let (intcode_pc, input, output) = IntcodePC::new(prog); 
        thread::spawn(move || intcode_pc.run());
        input.send(9).unwrap();
        assert_eq!(output.recv().unwrap(), 0);
    }

    #[test]
    fn immediate_mode_78_1() {
        let prog = vec![3,3,1108,-1,8,3,4,3,99];
        let (intcode_pc, input, output) = IntcodePC::new(prog.clone()); 
        thread::spawn(move || intcode_pc.run());
        input.send(8).unwrap();
        assert_eq!(output.recv().unwrap(), 1);

        let (intcode_pc, input, output) = IntcodePC::new(prog); 
        thread::spawn(move || intcode_pc.run());
        input.send(6).unwrap();
        assert_eq!(output.recv().unwrap(), 0);
    }

    #[test]
    fn immediate_mode_78_2() {
        let prog = vec![3,3,1107,-1,8,3,4,3,99];
        let (intcode_pc, input, output) = IntcodePC::new(prog.clone()); 
        thread::spawn(move || intcode_pc.run());
        input.send(5).unwrap();
        assert_eq!(output.recv().unwrap(), 1);

        let (intcode_pc, input, output) = IntcodePC::new(prog); 
        thread::spawn(move || intcode_pc.run());
        input.send(9).unwrap();
        assert_eq!(output.recv().unwrap(), 0);
    }

    #[test]
    fn jump_test_position_mode() {
        let prog = vec![3,12,6,12,15,1,13,14,13,4,13,99,-1,0,1,9];
        let (intcode_pc, input, output) = IntcodePC::new(prog.clone()); 
        thread::spawn(move || intcode_pc.run());
        input.send(0).unwrap();
        assert_eq!(output.recv().unwrap(), 0);

        let (intcode_pc, input, output) = IntcodePC::new(prog); 
        thread::spawn(move || intcode_pc.run());
        input.send(1).unwrap();
        assert_eq!(output.recv().unwrap(), 1);
    }

    #[test]
    fn jump_test_immediate_mode() {
        let prog = vec![3,3,1105,-1,9,1101,0,0,12,4,12,99,1];
        let (intcode_pc, input, output) = IntcodePC::new(prog.clone()); 
        thread::spawn(move || intcode_pc.run());
        input.send(0).unwrap();
        assert_eq!(output.recv().unwrap(), 0);

        let (intcode_pc, input, output) = IntcodePC::new(prog); 
        thread::spawn(move || intcode_pc.run());
        input.send(1).unwrap();
        assert_eq!(output.recv().unwrap(), 1);
    }

    #[test]
    fn max_thruster_signal_test1() {
        let prog = vec![3,15,3,16,1002,16,10,16,1,16,15,15,4,15,99,0,0];
        assert_eq!(max_thruster_signal(prog, false), 43210);
    }

    #[test]
    fn max_thruster_signal_test2() {
        let prog = vec![3,23,3,24,1002,24,10,24,1002,23,-1,23,101,5,23,23,1,24,23,23,4,23,99,0,0];
        assert_eq!(max_thruster_signal(prog, false), 54321);
    }

    #[test]
    fn max_thruster_signal_test3() {
        let prog = vec![3,31,3,32,1002,32,10,32,1001,31,-2,31,1007,31,0,33,1002,33,7,33,1,33,31,31,1,32,31,31,4,31,99,0,0,0];
        assert_eq!(max_thruster_signal(prog, false), 65210);
    }

    #[test]
    fn max_thruster_signal_with_feedback_test1() {
        let prog = vec![3,26,1001,26,-4,26,3,27,1002,27,2,27,1,27,26,27,4,27,1001,28,-1,28,1005,28,6,99,0,0,5];
        assert_eq!(max_thruster_signal(prog, true), 139629729);
    }

    #[test]
    fn max_thruster_signal_with_feedback_test2() {
        let prog = vec![3,52,1001,52,-5,52,3,53,1,52,56,54,1007,54,5,55,1005,55,26,1001,54,-5,54,1105,1,12,1,53,54,53,1008,54,0,55,1001,55,1,55,2,53,55,53,4,53,1001,56,-1,56,1005,56,6,99,0,0,0,0,10];
        assert_eq!(max_thruster_signal(prog, true), 18216);
    }
}
