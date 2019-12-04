fn digits(number: u32) -> Vec<u32> {
    number.to_string()
        .chars()
        .map(|d| d.to_digit(10).unwrap())
        .collect()
}

fn validate_pass(pass: u32) -> bool {
    let mut adjanced = false;
    let mut adj_combo = 1;
    let mut increasing = true;
    let digits = digits(pass);
    for i in 1..digits.len() {
        if digits[i-1] > digits[i] {
            increasing = false;
            break;
        } else if digits[i-1] == digits[i] {
            adj_combo += 1;
        } else {
            if adj_combo == 2 {
                adjanced = true;
            }
            adj_combo = 1;
        }
    }
    if adj_combo == 2 { adjanced = true; }
    adjanced && increasing
}

fn main() {
    let mut count = 0;
    for i in 171309..643603 {
        if validate_pass(i) {
            count += 1;
        }
    }
    println!("{}", count);
}
