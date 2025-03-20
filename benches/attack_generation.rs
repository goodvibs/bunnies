use criterion::{criterion_group, criterion_main, Criterion};
use bunnies::attacks::{magic, manual};
use bunnies::attacks::magic::relevant_mask::{PrecomputedMasksForSquares, BISHOP_RELEVANT_MASKS, ROOK_RELEVANT_MASKS};
use bunnies::bitboard::{get_bit_combinations_iter, Bitboard};
use bunnies::square::Square;

fn sliding_piece_attacks_test(relevant_masks: &PrecomputedMasksForSquares, get_attacks: impl Fn(Square, Bitboard) -> Bitboard) {
    for square in Square::iter_all() {
        let relevant_mask = relevant_masks.get(*square);
        let occupied_masks_iter = get_bit_combinations_iter(relevant_mask);
        for occupied in occupied_masks_iter {
            let _ = get_attacks(*square, occupied);
        }
    }
}

fn benchmark_rook_attacks(c: &mut Criterion) {
    let mut group = c.benchmark_group("Rook Attacks");

    // Warm up static initialization
    let _ = magic::magic_single_rook_attacks(Square::A6, 71);

    group.bench_function("Manual Rook Attacks", |b| {
        b.iter(|| sliding_piece_attacks_test(&ROOK_RELEVANT_MASKS, manual::manual_single_rook_attacks))
    });

    group.bench_function("Magic Rook Attacks", |b| {
        b.iter(|| sliding_piece_attacks_test(&ROOK_RELEVANT_MASKS, magic::magic_single_rook_attacks))
    });

    group.finish();
}

fn benchmark_bishop_attacks(c: &mut Criterion) {
    let mut group = c.benchmark_group("Bishop Attacks");

    // Warm up static initialization
    let _ = magic::magic_single_bishop_attacks(Square::C4, 255);

    group.bench_function("Manual Bishop Attacks", |b| {
        b.iter(|| sliding_piece_attacks_test(&BISHOP_RELEVANT_MASKS, manual::manual_single_bishop_attacks))
    });

    group.bench_function("Magic Bishop Attacks", |b| {
        b.iter(|| sliding_piece_attacks_test(&BISHOP_RELEVANT_MASKS, magic::magic_single_bishop_attacks))
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_rook_attacks,
    benchmark_bishop_attacks
);
criterion_main!(benches);