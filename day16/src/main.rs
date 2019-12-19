use itertools::Itertools;

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
    [0].iter().chain(fft.iter())
        .chunks(position as usize).into_iter()
        .skip(1)
        .step_by(2)
        .batching(|it| {
            if let Some(positive) = it.next() {
                let pos_sum = positive.fold(0, |acc, val| acc + val);
                if let Some(negative) = it.next() {
                    let neg_sum = negative.fold(0, |acc, val| acc + val);
                    Some(pos_sum - neg_sum)
                } else {
                    Some(pos_sum)
                }
            } else {
                None
            }
        })
        .fold(0, |acc, val| acc + val)
        .abs()
        .rem_euclid(10)
}

fn next_phase(fft: Vec<i32>) -> Vec<i32> {
    (1..=fft.len()).map(|x| apply_transform(&fft, x as u32))
        .collect()
}

fn next_phase_with_offset(fft: Vec<i32>) -> Vec<i32> {
    let mut sum = fft.iter().sum::<i32>();
    (0..fft.len()).map(|n| {
            let tmp = sum;
            sum -= fft[n];
            tmp.abs().rem_euclid(10)
        })
        .collect()
}


fn main() {
    let mut signal = parse_input(INPUT);
    let offset = get_offset(&signal);
    signal = vec_times_n(signal, 10000).split_off(offset);
    for _ in 0..100 {
        signal = next_phase_with_offset(signal);
    }
    println!("result: {:?}", &signal[..8].iter().filter_map(|&d| {
            std::char::from_digit(d as u32, 10)
        })
        .collect::<String>());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_next_phase() {
        let input = "12345678";
        let mut signal = parse_input(input);
        signal = next_phase(signal);
        assert_eq!(signal, [4, 8, 2, 2, 6, 1, 5, 8]);
    }

    #[test]
    fn after_100_phases1() {
        let input = "80871224585914546619083218645595";
        let mut signal = parse_input(input);
        for _ in 0..100 {
            println!("{:?}", signal);
            signal = next_phase(signal);
        }
        assert_eq!(&signal[..8], &[2, 4, 1, 7, 6, 1, 7, 6]);
    }

    #[test]
    fn after_100_phases2() {
        let input = "19617804207202209144916044189917";
        let mut signal = parse_input(input);
        for _ in 0..100 {
            signal = next_phase(signal);
        }
        assert_eq!(&signal[..8], &[7, 3, 7, 4, 5, 4, 1, 8]);
    }

    #[test]
    fn after_100_phases3() {
        let input = "69317163492948606335995924319873";
        let mut signal = parse_input(input);
        for _ in 0..100 {
            signal = next_phase(signal);
        }
        assert_eq!(&signal[..8], &[5, 2, 4, 3, 2, 1, 3, 3]);
    }

    #[test]
    fn real_signal_with_offset1() {
        let input = "03036732577212944063491565474664";
        let mut signal = parse_input(input);
        let offset = get_offset(&signal);
        signal = vec_times_n(signal, 10000).split_off(offset);
        for _ in 0..100 {
            signal = next_phase_with_offset(signal);
        }
        assert_eq!(&signal[..8], &[8, 4, 4, 6, 2, 0, 2, 6]);
    }
}
