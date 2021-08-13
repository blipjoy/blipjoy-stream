use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use eater::{EaterSim, EaterVm};

fn bench_vm(c: &mut Criterion) {
    let mut group = c.benchmark_group("vm");

    let mut interp = EaterVm::new();
    let mut sim = EaterSim::new();

    let program = [
        0x1e, 0x2f, 0xe0, 0x75, 0x61, 0xf0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x03,
    ];

    group.bench_function(BenchmarkId::new("Interpreter", "print 3's"), |b| {
        b.iter(|| {
            interp.load(&program);
            interp.run();
        })
    });
    group.bench_function(BenchmarkId::new("Simulator", "print 3's"), |b| {
        b.iter(|| {
            sim.load(&program);
            sim.run();
        })
    });
}

criterion_group!(benches, bench_vm);
criterion_main!(benches);
