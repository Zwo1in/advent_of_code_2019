const INPUT: &str = include_str!("../input");
fn load_prog() -> Vec<i32> {
    INPUT.split(",").map(|s| {
        s.trim().parse::<i32>().unwrap()
    }).collect()
}

fn intcode(mut code: Vec<i32>) -> Vec<i32> {
    let mut pc = 0;
    while code[pc] != 99 {
        let (id1, id2, id3) = (code[pc+1] as usize, code[pc+2] as usize, code[pc+3] as usize);
        match code[pc] {
            1 => {
                code[id3] = code[id1] + code[id2];
            },
            2 => {
                code[id3] = code[id1] * code[id2];
            },
            _ => panic!("An error in intcode!"),
        }
        pc += 4;
    }
    code
}

fn main() {
    for i in 0..=99 {
        for j in 0..=99 {
            let mut prog = load_prog();
            prog[1] = i;
            prog[2] = j;
            if intcode(prog)[0] == 19690720 {
                println!("{}", 100*i+j);
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn first_prog() {
        let prog = vec![1,0,0,0,99];
        assert_eq!(intcode(prog), [2,0,0,0,99]);
    }

    #[test]
    fn second_prog() {
        let prog = vec![2,3,0,3,99];
        assert_eq!(intcode(prog), [2,3,0,6,99]);
    }

    #[test]
    fn third_prog() {
        let prog = vec![2,4,4,5,99,0];
        assert_eq!(intcode(prog), [2,4,4,5,99,9801]);
    }

    #[test]
    fn fourth_prog() {
        let prog = vec![1,1,1,4,99,5,6,0,99];
        assert_eq!(intcode(prog), [30,1,1,4,2,5,6,0,99]);
    }
}
