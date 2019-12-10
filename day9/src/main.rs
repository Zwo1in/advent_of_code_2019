pub mod intcode_pc;
use intcode_pc::IntcodePC;

use std::thread;

const INPUT: &str = include_str!("../input");
fn load_prog() -> Vec<i64> {
    INPUT.split(",").map(|s| {
        s.trim().parse::<i64>().unwrap()
    }).collect()
}

fn main() {
    let prog = load_prog();
    let (intcode_pc, input, output) = IntcodePC::new(prog);
    thread::spawn(move || intcode_pc.run());
    let _ = input.send(2);
    while let Ok(out) = output.recv() {
        println!("{}", out);
    }
}
