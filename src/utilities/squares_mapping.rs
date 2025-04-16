use crate::{Bitboard, Square};

pub struct SquaresMapping<const NUM_SQUARES_PER_KEY: usize, OutputType: Copy> {
    masks: Vec<OutputType>,
}

impl<const NUM_SQUARES_PER_KEY: usize, OutputType: Copy> SquaresMapping<NUM_SQUARES_PER_KEY, OutputType> {
    pub fn init_(calc_mask: impl Fn([Square; NUM_SQUARES_PER_KEY]) -> OutputType) -> Self {
        let total_combos = 64usize.pow(NUM_SQUARES_PER_KEY as u32);
        let mut masks = vec![unsafe { std::mem::zeroed() }; total_combos];

        // Generate indices for all combinations
        Self::for_each_combo(|combo, index| {
            masks[index] = calc_mask(combo);
        });

        Self { masks }
    }

    pub fn get_(&self, squares: [Square; NUM_SQUARES_PER_KEY]) -> OutputType {
        let index = Self::combo_to_index(squares);
        unsafe { *self.masks.get_unchecked(index) }
    }

    fn combo_to_index(squares: [Square; NUM_SQUARES_PER_KEY]) -> usize {
        let mut index = 0;
        for i in 0..NUM_SQUARES_PER_KEY {
            index = index * 64 + squares[i] as usize;
        }
        index
    }

    fn for_each_combo(mut f: impl FnMut([Square; NUM_SQUARES_PER_KEY], usize)) {
        fn recursive_combos<const M: usize>(
            current: &mut [Square; M],
            depth: usize,
            cb: &mut dyn FnMut([Square; M], usize),
        ) {
            if depth == M {
                let mut index = 0;
                for i in 0..M {
                    index = index * 64 + current[i] as usize;
                }
                cb(*current, index);
                return;
            }

            for square in Square::ALL {
                current[depth] = square;
                recursive_combos(current, depth + 1, cb);
            }
        }

        let mut current = [Square::A1; NUM_SQUARES_PER_KEY];
        recursive_combos(&mut current, 0, &mut f);
    }
}

// Convenience type aliases
pub type SquaresOneToOneMapping<OutputType> = SquaresMapping<1, OutputType>;
pub type SquaresTwoToOneMapping<OutputType> = SquaresMapping<2, OutputType>;
pub type SquaresThreeToOneMapping<OutputType> = SquaresMapping<3, OutputType>;

pub type SquaresToMasks = SquaresMapping<1, Bitboard>;
pub type SquarePairsToMasks = SquaresMapping<2, Bitboard>;
pub type SquareTriplesToMasks = SquaresMapping<3, Bitboard>;

// Extension methods for more ergonomic access
impl<OutputType: Copy> SquaresOneToOneMapping<OutputType> {
    pub fn init(calc_mask: impl Fn(Square) -> OutputType) -> Self {
        Self::init_(|squares| calc_mask(squares[0]))
    }

    #[inline]
    pub fn get(&self, square: Square) -> OutputType {
        self.get_([square])
    }
}

impl<OutputType: Copy> SquaresTwoToOneMapping<OutputType> {
    pub fn init(calc_mask: impl Fn(Square, Square) -> OutputType) -> Self {
        Self::init_(|squares| calc_mask(squares[0], squares[1]))
    }

    #[inline]
    pub fn get(&self, sq1: Square, sq2: Square) -> OutputType {
        self.get_([sq1, sq2])
    }
}

impl<OutputType: Copy> SquaresThreeToOneMapping<OutputType> {
    pub fn init(calc_mask: impl Fn(Square, Square, Square) -> OutputType) -> Self {
        Self::init_(|squares| calc_mask(squares[0], squares[1], squares[2]))
    }

    #[inline]
    pub fn get(&self, sq1: Square, sq2: Square, sq3: Square) -> OutputType {
        self.get_([sq1, sq2, sq3])
    }
}