use criterion::{criterion_group, criterion_main, Criterion};
use bunnies::attacks::{magic, manual, precomputed};
use bunnies::attacks::magic::relevant_mask::{PrecomputedMasksForSquares, BISHOP_RELEVANT_MASKS, ROOK_RELEVANT_MASKS};
use bunnies::bitboard::{get_bit_combinations_iter, Bitboard};
use bunnies::color::Color;
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

fn benchmark_king_attacks(c: &mut Criterion) {
    let mut group = c.benchmark_group("King Attacks");

    // Warm up static initialization
    let _ = precomputed::precomputed_single_king_attacks(Square::A6);

    // Test all squares for king attacks
    group.bench_function("Manual King Attacks", |b| {
        b.iter(|| {
            for square in Square::iter_all() {
                let _ = manual::multi_king_attacks(square.get_mask());
            }
        })
    });

    group.bench_function("Precomputed King Attacks", |b| {
        b.iter(|| {
            for square in Square::iter_all() {
                let _ = precomputed::precomputed_single_king_attacks(*square);
            }
        })
    });

    group.finish();
}

fn benchmark_knight_attacks(c: &mut Criterion) {
    let mut group = c.benchmark_group("Knight Attacks");

    // Warm up static initialization
    let _ = precomputed::precomputed_single_knight_attacks(Square::A6);

    // Test all squares for knight attacks
    group.bench_function("Manual Knight Attacks", |b| {
        b.iter(|| {
            for square in Square::iter_all() {
                let _ = manual::multi_knight_attacks(square.get_mask());
            }
        })
    });

    group.bench_function("Precomputed Knight Attacks", |b| {
        b.iter(|| {
            for square in Square::iter_all() {
                let _ = precomputed::precomputed_single_knight_attacks(*square);
            }
        })
    });

    group.finish();
}

fn benchmark_pawn_attacks(c: &mut Criterion) {
    let mut group = c.benchmark_group("Pawn Attacks");

    // Test all squares for white pawn attacks
    group.bench_function("Manual White Pawn Attacks", |b| {
        b.iter(|| {
            for square in Square::iter_all() {
                let _ = manual::multi_pawn_attacks(square.get_mask(), Color::White);
            }
        })
    });

    // Test all squares for black pawn attacks
    group.bench_function("Manual Black Pawn Attacks", |b| {
        b.iter(|| {
            for square in Square::iter_all() {
                let _ = manual::multi_pawn_attacks(square.get_mask(), Color::Black);
            }
        })
    });

    group.finish();
}

fn benchmark_pawn_pushes(c: &mut Criterion) {
    let mut group = c.benchmark_group("Pawn Pushes");

    // Test all squares for white pawn moves
    group.bench_function("Manual White Pawn Pushes", |b| {
        b.iter(|| {
            for square in Square::iter_all() {
                let _ = manual::multi_pawn_moves(square.get_mask(), Color::White);
            }
        })
    });

    // Test all squares for black pawn moves
    group.bench_function("Manual Black Pawn Pushes", |b| {
        b.iter(|| {
            for square in Square::iter_all() {
                let _ = manual::multi_pawn_moves(square.get_mask(), Color::Black);
            }
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_king_attacks,
    benchmark_knight_attacks,
    benchmark_pawn_attacks,
    benchmark_pawn_pushes,
);
criterion_main!(benches);