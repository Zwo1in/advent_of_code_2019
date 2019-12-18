use tracker::*;

fn main() {
    let mut tracker = prepare_tracker(INPUT);
    println!("{}", tracker);
    (0..1000).for_each(|_| tracker.step());
    println!("{}", tracker);
    println!("Total energy: {}", tracker.total_energy());

    let mut tracker = prepare_tracker(INPUT);
    let mut n = 0u128;
    let x_period = loop {
        n += 1;
        tracker.step_x();
        if is_initial_state_x(&tracker) {
            println!("{}", n);
            break n;
        }
    };
    let mut tracker = prepare_tracker(INPUT);
    let mut n = 0u128;
    let y_period = loop {
        n += 1;
        tracker.step_y();
        if is_initial_state_y(&tracker) {
            println!("{}", n);
            break n;
        }
    };
    let mut tracker = prepare_tracker(INPUT);
    let mut n = 0u128;
    let z_period = loop {
        n += 1;
        tracker.step_z();
        if is_initial_state_z(&tracker) {
            println!("{}", n);
            break n;
        }
    };
    println!("{}, {}, {}", x_period, y_period, z_period);
}

