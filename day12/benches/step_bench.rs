use tracker::{prepare_tracker, INPUT};
use criterion::{black_box, criterion_group, criterion_main, Criterion};


fn step_benchmark(c: &mut Criterion) {
    let track_bench = |input: &'static str| {
        let mut tracker = prepare_tracker(input);
        (0..1000).for_each(|_| tracker.step());
    };

    c.bench_function("step_benchmark", |b| b.iter(|| track_bench(black_box(INPUT))));
}

criterion_group!(benches, step_benchmark);
criterion_main!(benches);
