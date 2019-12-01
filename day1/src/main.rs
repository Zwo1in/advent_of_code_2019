use math::round::floor;

const INPUT: &str = include_str!("../input");

fn calculate_fuel(mass: i64) -> i64 {
    let mass = floor(mass as f64 / 3.0, 0) as i64;
    mass - 2
}

fn fuel_with_backup(mass: i64) -> i64 {
    let mut sum = 0;
    let mut tmp = calculate_fuel(mass);
    while tmp > 0 {
        sum += tmp;
        tmp = calculate_fuel(tmp);
    }
    sum
}

fn main() {
    let mut sum = 0;
    for line in INPUT.lines() {
        let line: i64 = line.trim().parse().unwrap();
        sum += fuel_with_backup(line);
    }

    println!("{}", sum);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn _12() {
        assert_eq!(2, calculate_fuel(12));
    }

    #[test]
    fn _14() {
        assert_eq!(2, calculate_fuel(14));
    }

    #[test]
    fn _1969() {
        assert_eq!(654, calculate_fuel(1969));
    }

    #[test]
    fn _100756() {
        assert_eq!(33583, calculate_fuel(100756));
    }

    #[test]
    fn with_backup_12() {
        assert_eq!(2, fuel_with_backup(12));
    }

    #[test]
    fn with_backup_14() {
        assert_eq!(2, fuel_with_backup(14));
    }

    #[test]
    fn with_backup_1969() {
        assert_eq!(966, fuel_with_backup(1969));
    }

    #[test]
    fn with_backup_100756() {
        assert_eq!(50346, fuel_with_backup(100756));
    }
}
