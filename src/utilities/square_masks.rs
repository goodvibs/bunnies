use crate::{Bitboard, Square};

pub struct SquaresToMaskMapping<const N: usize> {
    masks: Vec<Bitboard>,
}

impl<const N: usize> SquaresToMaskMapping<N> {
    fn _init(calc_mask: impl Fn([Square; N]) -> Bitboard) -> Self {
        let total_combos = 64usize.pow(N as u32);
        let mut masks = vec![0; total_combos];

        // Generate indices for all combinations
        Self::for_each_combo(|combo, index| {
            masks[index] = calc_mask(combo);
        });

        Self { masks }
    }

    #[inline]
    fn _get(&self, squares: [Square; N]) -> Bitboard {
        let index = Self::combo_to_index(squares);
        // Using unsafe here for performance in a hot path
        // is a common optimization in chess engines
        unsafe { *self.masks.get_unchecked(index) }
    }

    #[inline]
    fn combo_to_index(squares: [Square; N]) -> usize {
        let mut index = 0;
        for i in 0..N {
            index = index * 64 + squares[i] as usize;
        }
        index
    }

    fn for_each_combo(mut f: impl FnMut([Square; N], usize)) {
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

        let mut current = [Square::A1; N];
        recursive_combos(&mut current, 0, &mut f);
    }
}

// Convenience type aliases
pub type SquareMasks = SquaresToMaskMapping<1>;
pub type SquarePairMasks = SquaresToMaskMapping<2>;
pub type SquareTripleMasks = SquaresToMaskMapping<3>;

// Extension methods for more ergonomic access
impl SquareMasks {
    #[inline]
    pub fn init(calc_mask: impl Fn(Square) -> Bitboard) -> Self {
        Self::_init(|squares| calc_mask(squares[0]))
    }
    
    #[inline]
    pub fn get(&self, square: Square) -> Bitboard {
        self._get([square])
    }
}

impl SquarePairMasks {
    #[inline]
    pub fn init(calc_mask: impl Fn(Square, Square) -> Bitboard) -> Self {
        Self::_init(|squares| calc_mask(squares[0], squares[1]))
    }
    
    #[inline]
    pub fn get(&self, sq1: Square, sq2: Square) -> Bitboard {
        self._get([sq1, sq2])
    }
}

impl SquareTripleMasks {
    #[inline]
    pub fn init(calc_mask: impl Fn(Square, Square, Square) -> Bitboard) -> Self {
        Self::_init(|squares| calc_mask(squares[0], squares[1], squares[2]))
    }
    
    #[inline]
    pub fn get(&self, sq1: Square, sq2: Square, sq3: Square) -> Bitboard {
        self._get([sq1, sq2, sq3])
    }
}