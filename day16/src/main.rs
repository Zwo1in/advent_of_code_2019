const INPUT: &'static str = include_str!("../input");

fn parse_input(input: &'static str) -> Vec<i32> {
    input.trim().chars()
        .map(|ch| ch.to_digit(10).unwrap() as i32)
        .collect()
}

fn get_offset(vec: &Vec<i32>) -> usize {
    vec[..7].iter().map(|&d| std::char::from_digit(d as u32, 10).unwrap())
        .collect::<String>()
        .parse()
        .unwrap()
}

fn vec_times_n(vec: Vec<i32>, n: usize) -> Vec<i32> {
    (0..n).flat_map(|_| vec.clone()).collect()
}

fn apply_transform(fft: &Vec<i32>, position: u32) -> i32 {
    const FACTORS: [i32; 4] = [0, 1, 0, -1];
    let multipilers = FACTORS.iter().flat_map(|n| vec![n; position as usize])
        .cycle()
        .skip(1);
    multipilers.zip(fft.iter())
        .map(|(&a, &b)| a*b)
        .sum::<i32>()
        .abs()
        .rem_euclid(10)
}

fn next_phase(fft: Vec<i32>, offset: usize) -> Vec<i32> {
    (1..=fft.len()+1).map(|x| apply_transform(&fft, x as u32))
        .collect()
}

fn main() {
    let mut signal = parse_input(INPUT);
    for _ in 0..100 {
        signal = next_phase(signal, 0);
    }
    println!("result: {:?}", &signal[..8]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn after_100_phases1() {
        let input = "80871224585914546619083218645595";
        let mut signal = parse_input(input);
        for _ in 0..100 {
            signal = next_phase(signal, 0);
        }
        assert_eq!(&signal[..8], &[2, 4, 1, 7, 6, 1, 7, 6]);
    }

    #[test]
    fn after_100_phases2() {
        let input = "19617804207202209144916044189917";
        let mut signal = parse_input(input);
        for _ in 0..100 {
            signal = next_phase(signal, 0);
        }
        assert_eq!(&signal[..8], &[7, 3, 7, 4, 5, 4, 1, 8]);
    }

    #[test]
    fn after_100_phases3() {
        let input = "69317163492948606335995924319873";
        let mut signal = parse_input(input);
        for _ in 0..100 {
            signal = next_phase(signal, 0);
        }
        assert_eq!(&signal[..8], &[5, 2, 4, 3, 2, 1, 3, 3]);
    }

    #[test]
    fn real_signal_with_offset1() {
        let input = "03036732577212944063491565474664";
        let mut signal = parse_input(input);
        let offset = get_offset(&signal);
        signal = vec_times_n(signal, 10000);
        for _ in 0..100 {
            signal = next_phase(signal, 0);
        }
        assert_eq!(&signal[offset..offset+8], &[8, 4, 4, 6, 2, 0, 2, 6]);
    }
}
