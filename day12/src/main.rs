use tracker::{prepare_tracker, is_initial_state, INPUT};

fn main() {
    let mut tracker = prepare_tracker(INPUT);
    println!("{}", tracker);
    (0..1000).for_each(|_| tracker.step());
    println!("{}", tracker);
    println!("Total energy: {}", tracker.total_energy());

    let mut tracker = prepare_tracker(INPUT);
    let mut n = 0u128;
    loop {
        n += 1;
        tracker.step();
        if is_initial_state(&tracker) {
            println!("Found initial state after {} steps", n);
            break;
        }
    }
}

