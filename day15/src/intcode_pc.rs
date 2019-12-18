use std::sync::mpsc::{channel, Sender, Receiver};
use std::convert::TryInto;

pub fn load_prog(input: &'static str) -> Vec<i64> {
    input.split(",").map(|s| {
        s.trim().parse::<i64>().unwrap()
    }).collect()
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Mode {
    Position,
    Immediate,
    Relative,
    ReturnAddr,
    ReturnAddrRelative,
}

impl Mode {
    fn to_return_mode(self) -> Self {
        match self {
            Mode::Position => Mode::ReturnAddr,
            Mode::Relative => Mode::ReturnAddrRelative,
            Mode::Immediate => panic!("Cannot return from immediate mode"),
            _ => self,
        }
    }
}

trait AsMode {
    fn as_mode(self) -> Mode;
}

impl AsMode for u32 {
    fn as_mode(self) -> Mode {
        match self {
            0 => Mode::Position,
            1 => Mode::Immediate,
            2 => Mode::Relative,
            _ => panic!("Wrong mode"),
        }
    }
}

#[derive(Debug)]
pub enum Message {
    Value(i64),
    NeedInput,
    Finished,
}

impl Message {
    pub fn unwrap_val(self) -> i64 {
        if let Message::Value(v) = self {
            v
        } else {
            panic!("Failed on value unwrapping")
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Order {
    Add { a: Mode, b: Mode, res_addr: Mode },
    Mul { a: Mode, b: Mode, res_addr: Mode },
    In  { res_addr: Mode },
    Out { val: Mode },
    Jit { test: Mode, addr: Mode },
    Jif { test: Mode, addr: Mode },
    Lt  { a: Mode, b: Mode, res_addr: Mode },
    Eq  { a: Mode, b: Mode, res_addr: Mode },
    Rbo { offset: Mode },
}

trait AsOrder {
    fn as_order(self, params: [Mode; 3]) -> Order;
}

impl AsOrder for i64 {
    fn as_order(self, params: [Mode; 3]) -> Order {
        use Order::*;
        match self {
            1 => Add { a: params[0], b: params[1], res_addr: params[2].to_return_mode() },
            2 => Mul { a: params[0], b: params[1], res_addr: params[2].to_return_mode() },
            3 => In  { res_addr: params[0].to_return_mode()},
            4 => Out { val: params[0] },
            5 => Jit { test: params[0], addr: params[1] },
            6 => Jif { test: params[0], addr: params[1] },
            7 => Lt  { a: params[0], b: params[1], res_addr: params[2].to_return_mode() },
            8 => Eq  { a: params[0], b: params[1], res_addr: params[2].to_return_mode() },
            9 => Rbo { offset: params[0] },
            _ => panic!("Wrong order"),
        }
    }
}

impl Order {
    fn new(raw: i64) -> Self {
        let ord = raw%100;
        let raw = raw.to_string();
        let mut params = raw.chars().rev().skip(2).map(|c| c.to_digit(10).unwrap());
        let params = [params.next().unwrap_or(0).as_mode(),
                      params.next().unwrap_or(0).as_mode(),
                      params.next().unwrap_or(0).as_mode()];
        ord.as_order(params)
    }

    fn parse_args(&self, code: &mut Vec<i64>, pc: &usize, rel_base: &i64) -> Vec<i64> {
        use Order::*;
        let modes: Vec<Mode>;
        match *self {
            Add { a, b, res_addr } => modes = vec![a, b, res_addr],
            Mul { a, b, res_addr } => modes = vec![a, b, res_addr],
            In  { res_addr }       => modes = vec![res_addr],
            Out { val }            => modes = vec![val],
            Jit { test, addr }     => modes = vec![test, addr],
            Jif { test, addr }     => modes = vec![test, addr],
            Lt  { a, b, res_addr } => modes = vec![a, b, res_addr],
            Eq  { a, b, res_addr } => modes = vec![a, b, res_addr],
            Rbo { offset }         => modes = vec![offset],
        };
        modes.into_iter()
            .enumerate()
            .map(|(n, mode)| {
                let arg = Self::read(code, *pc+n+1);
                match mode {
                    Mode::Position           => Self::read(code, arg),
                    Mode::Relative           => Self::read(code, arg + rel_base),
                    Mode::Immediate          => arg,
                    Mode::ReturnAddr         => arg,
                    Mode::ReturnAddrRelative => arg + rel_base,
                }
            })
            .collect()
    }

    fn read(code: &mut Vec<i64>, addr: impl TryInto<usize>) -> i64 {
        let addr = addr.try_into().unwrap_or_else(|_| panic!("Couldn't cast an address"));
        if code.len() < addr {
            code.resize(2*addr, 0);
        }
        code[addr]
    }

    fn write(code: &mut Vec<i64>, addr: impl TryInto<usize>, value: i64) {
        let addr = addr.try_into().unwrap_or_else(|_| panic!("Couldn't cast an address"));
        if code.len() < addr {
            code.resize(2*addr, 0);
        }
        code[addr] = value;
    }

    fn execute(
        self,
        code: &mut Vec<i64>,
        pc: &mut usize,
        input: &Receiver<Message>,
        output: &Sender<Message>,
        rel_base: &mut i64
    ) -> Option<Message>
    {
        use Order::*;
        let args = self.parse_args(code, pc, rel_base);
        match self {
            Add {..} => {
                Self::write(code, args[2], args[0] + args[1]);
                *pc += 4;
            },
            Mul {..} => {
                Self::write(code, args[2], args[0] * args[1]);
                *pc += 4;
            },
            In  {..} => {
                output.send(Message::NeedInput).unwrap();
                if let Message::Value(val) = input.recv().unwrap() {
                    Self::write(code, args[0], val);
                    *pc += 2;
                } else {
                    panic!("Received non value");
                }
            },
            Out {..} => {
                *pc += 2;
                output.send(Message::Value(args[0])).unwrap();
                return Some(Message::Value(args[0]));
            },
            Jit {..} => {
                if args[0] != 0 { *pc = args[1] as usize } else { *pc += 3; }
            },
            Jif {..} => {
                if args[0] == 0 { *pc = args[1] as usize } else { *pc += 3; }
            },
            Lt  {..} => {
                Self::write(code, args[2], if args[0] < args[1] { 1 } else { 0 });
                *pc += 4;
            },
            Eq  {..} => {
                Self::write(code, args[2], if args[0] == args[1] { 1 } else { 0 });
                *pc += 4;
            },
            Rbo {..} => {
                *rel_base += args[0];
                *pc += 2;
            },
        };
        None
    }
}

pub struct IntcodePC {
    program: Vec<i64>,
    pub input: Receiver<Message>,
    pub output: Sender<Message>,
}

impl IntcodePC {
    pub fn new(program: Vec<i64>) -> (Self, Sender<Message>, Receiver<Message>) {
        let (in_sender, in_receiver) = channel();
        let (out_sender, out_receiver) = channel();
        (Self { program, input: in_receiver, output: out_sender },
            in_sender, out_receiver)
    }

    pub fn piped(program: Vec<i64>, input: Receiver<Message>) -> (Self, Receiver<Message>) {
        let (out_sender, out_receiver) = channel();
        (Self { program, input, output: out_sender }, out_receiver)
    } 

    pub fn run(mut self) -> (Vec<i64>, Vec<i64>) {
        let mut pc = 0;
        let mut rel_base = 0;
        let mut outputs = vec![];
        while self.program[pc] != 99 {
            let order = Order::new(self.program[pc]);
            if let Some(ret) = order.execute(
                &mut self.program,
                &mut pc,
                &self.input,
                &self.output,
                &mut rel_base
            ) {
                if let Message::Value(v) = ret {
                    outputs.push(v);
                }
            }
        }
        let _ = self.output.send(Message::Finished);
        (self.program, outputs)
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use super::{*, Mode::*};

    #[test]
    fn order_parse() {
        let code = 1102;
        assert_eq!(Order::new(code),
             Order::Mul { a: Immediate, b: Immediate, res_addr: ReturnAddr });
    }

    #[test]
    fn order_parse_default_params() {
        let code = 1001;
        assert_eq!(Order::new(code),
             Order::Add { a: Position, b: Immediate, res_addr: ReturnAddr });
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
    fn jump_position_mode() {
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
    fn jump_immediate_mode() {
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
    fn relative_base_offset1() {
        let prog = vec![109,1,204,-1,1001,100,1,100,1008,100,16,101,1006,101,0,99];
        let (intcode_pc, _, _) = IntcodePC::new(prog.clone()); 
        let handle = thread::spawn(move || intcode_pc.run());
        assert_eq!(prog, handle.join().unwrap().1);
    }

    #[test]
    fn big_number1() {
        let prog = vec![1102,34915192,34915192,7,4,7,99,0];
        let (intcode_pc, _, output) = IntcodePC::new(prog);
        thread::spawn(move || intcode_pc.run());
        assert_eq!(16, output.recv().unwrap().to_string().len());
    }

    #[test]
    fn big_number2() {
        let prog = vec![104,1125899906842624,99];
        let (intcode_pc, _, output) = IntcodePC::new(prog);
        thread::spawn(move || intcode_pc.run());
        assert_eq!(output.recv().unwrap(), 1125899906842624);
    }
}
