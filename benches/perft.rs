include!(concat!(env!("CARGO_MANIFEST_DIR"), "/perft_case.rs"));

use std::time::Duration;

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};

macro_rules! define_perft_benches {
    ($($name:ident => ($case:expr, $depth:literal);)+) => {
        fn bench_perft_positions(c: &mut Criterion) {
            $(
            {
                const CONTEXTS_CAPACITY: usize = $depth + 1;
                let pos = ($case).position::<CONTEXTS_CAPACITY>();
                let mut group = c.benchmark_group(($case).name());
                let nodes = ($case).nodes_at_depth($depth);
                group.throughput(Throughput::Elements(nodes));
                group.bench_function(BenchmarkId::new(stringify!($name), $depth), |b| {
                    b.iter(|| {
                        let mut p = pos.clone();
                        black_box(p.perft(black_box($depth)))
                    })
                });
                group.finish();
            }
            )+
        }
    };
}

define_perft_benches! {
    bench_perft_initial_position => (PerftCase::Initial, 5);
    bench_perft_kiwipete => (PerftCase::Kiwipete, 4);
    bench_perft_position_3 => (PerftCase::Position3, 6);
    bench_perft_position_4 => (PerftCase::Position4, 5);
    bench_perft_position_5 => (PerftCase::Position5, 4);
}

criterion_group! {
    name = benches;
    config = Criterion::default().warm_up_time(Duration::from_secs(1));
    targets = bench_perft_positions
}

criterion_main!(benches);
