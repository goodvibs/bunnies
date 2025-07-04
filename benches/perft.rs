use bunnies::Position;
use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};

fn bench_initial_position(c: &mut Criterion) {
    let initial_state = Position::initial();
    let mut group = c.benchmark_group("Initial Position");

    let expected_nodes = 197281; // perft(4) for initial position
    group.throughput(Throughput::Elements(expected_nodes));

    group.bench_function(BenchmarkId::new("perft", 4), |b| {
        b.iter(|| black_box(&initial_state).perft(black_box(4)))
    });
    group.finish();
}

fn bench_kiwipete(c: &mut Criterion) {
    let state =
        Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
            .unwrap();
    let mut group = c.benchmark_group("Kiwipete");

    let expected_nodes = 97862; // perft(3) for Kiwipete
    group.throughput(Throughput::Elements(expected_nodes));

    group.bench_function(BenchmarkId::new("perft", 3), |b| {
        b.iter(|| black_box(&state).perft(black_box(3)))
    });
    group.finish();
}

fn bench_position_3(c: &mut Criterion) {
    let state = Position::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();
    let mut group = c.benchmark_group("Position 3");

    let expected_nodes = 674624; // perft(5) for Position 3
    group.throughput(Throughput::Elements(expected_nodes));

    group.bench_function(BenchmarkId::new("perft", 5), |b| {
        b.iter(|| black_box(&state).perft(black_box(5)))
    });
    group.finish();
}

fn bench_position_4(c: &mut Criterion) {
    let state =
        Position::from_fen("r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1")
            .unwrap();
    let mut group = c.benchmark_group("Position 4");

    let expected_nodes = 422333; // perft(4) for Position 4
    group.throughput(Throughput::Elements(expected_nodes));

    group.bench_function(BenchmarkId::new("perft", 4), |b| {
        b.iter(|| black_box(&state).perft(black_box(4)))
    });
    group.finish();
}

fn bench_position_5(c: &mut Criterion) {
    let state =
        Position::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8").unwrap();
    let mut group = c.benchmark_group("Position 5");

    let expected_nodes = 62379; // perft(3) for Position 5
    group.throughput(Throughput::Elements(expected_nodes));

    group.bench_function(BenchmarkId::new("perft", 3), |b| {
        b.iter(|| black_box(&state).perft(black_box(3)))
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_initial_position,
    bench_kiwipete,
    bench_position_3,
    bench_position_4,
    bench_position_5
);
criterion_main!(benches);
