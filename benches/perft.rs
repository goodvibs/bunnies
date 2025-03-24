use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use bunnies::state::State;

fn perform_perft(state: &State, depth: u8) -> u64 {
    state.perft(depth)
}

fn bench_initial_position(c: &mut Criterion) {
    let initial_state = State::initial();

    let mut group = c.benchmark_group("Initial Position");

    group.bench_function(BenchmarkId::new("perft", 4), |b| {
        b.iter(|| perform_perft(black_box(&initial_state), black_box(4)))
    });

    group.finish();
}

fn bench_kiwipete(c: &mut Criterion) {
    let state = State::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap();

    let mut group = c.benchmark_group("Kiwipete");

    group.bench_function(BenchmarkId::new("perft", 3), |b| {
        b.iter(|| perform_perft(black_box(&state), black_box(3)))
    });

    group.finish();
}

fn bench_position_3(c: &mut Criterion) {
    let state = State::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();

    let mut group = c.benchmark_group("Position 3");

    group.bench_function(BenchmarkId::new("perft", 5), |b| {
        b.iter(|| perform_perft(black_box(&state), black_box(5)))
    });

    group.finish();
}

fn bench_position_4(c: &mut Criterion) {
    let state = State::from_fen("r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1").unwrap();

    let mut group = c.benchmark_group("Position 4");

    group.bench_function(BenchmarkId::new("perft", 4), |b| {
        b.iter(|| perform_perft(black_box(&state), black_box(4)))
    });

    group.finish();
}

fn bench_position_5(c: &mut Criterion) {
    let state = State::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8").unwrap();

    let mut group = c.benchmark_group("Position 5");

    group.bench_function(BenchmarkId::new("perft", 3), |b| {
        b.iter(|| perform_perft(black_box(&state), black_box(3)))
    });

    group.finish();
}

// Additional benchmark comparing different depths for initial position
fn bench_initial_position_depths(c: &mut Criterion) {
    let initial_state = State::initial();

    let mut group = c.benchmark_group("Initial Position Depths");
    group.sample_size(10);

    // Benchmark at multiple depths to see scaling behavior
    for depth in 1..=5 {
        group.bench_function(BenchmarkId::new("depth", depth), |b| {
            b.iter(|| perform_perft(black_box(&initial_state), black_box(depth)))
        });
    }

    group.finish();
}

// Benchmark to compare nodes per second across different positions at a fixed, lower depth
fn bench_nodes_per_second(c: &mut Criterion) {
    let initial_state = State::initial();
    let kiwipete = State::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap();
    let position3 = State::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();
    let position4 = State::from_fen("r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1").unwrap();
    let position5 = State::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8").unwrap();

    let mut group = c.benchmark_group("Nodes Per Second Comparison");
    // Use the same lower depth (4) for all positions to make comparison fair
    const DEPTH: u8 = 4;

    group.bench_function("Initial", |b| {
        b.iter(|| perform_perft(black_box(&initial_state), black_box(DEPTH)))
    });

    group.bench_function("Kiwipete", |b| {
        b.iter(|| perform_perft(black_box(&kiwipete), black_box(DEPTH)))
    });

    group.bench_function("Position3", |b| {
        b.iter(|| perform_perft(black_box(&position3), black_box(DEPTH)))
    });

    group.bench_function("Position4", |b| {
        b.iter(|| perform_perft(black_box(&position4), black_box(DEPTH)))
    });

    group.bench_function("Position5", |b| {
        b.iter(|| perform_perft(black_box(&position5), black_box(DEPTH)))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_initial_position,
    bench_kiwipete,
    bench_position_3,
    bench_position_4,
    bench_position_5,
    bench_initial_position_depths,
    bench_nodes_per_second
);
criterion_main!(benches);