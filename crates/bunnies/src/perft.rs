use crate::Color;
use crate::{MoveList, Position, ZobristPolicy};

fn count_nodes<const N: usize, const STM: Color, Z: ZobristPolicy>(
    position: &mut Position<N, STM, Z>,
    depth: u8,
) -> u64 {
    if depth == 0 {
        return 1;
    }
    if depth == 1 {
        return position.count_legal_moves() as u64;
    }
    let mut moves = MoveList::new();
    position.generate_moves(&mut moves);

    let mut total = 0u64;
    for &move_ in moves.as_slice() {
        position.make_move(move_);
        match STM {
            Color::White => {
                let child = unsafe { position.rebrand_stm_mut::<{ Color::Black }>() };
                total += count_nodes(child, depth - 1);
                child.unmake_move(move_);
            }
            Color::Black => {
                let child = unsafe { position.rebrand_stm_mut::<{ Color::White }>() };
                total += count_nodes(child, depth - 1);
                child.unmake_move(move_);
            }
        }
    }
    total
}

impl<const N: usize, const STM: Color, Z: ZobristPolicy> Position<N, STM, Z> {
    /// Counts leaf nodes to `depth` (divide-perft), mutating this position in place.
    /// Must be called on the search root; context stack must fit the traversal depth.
    #[inline]
    pub fn perft(&mut self, depth: u8) -> u64 {
        count_nodes(self, depth)
    }
}
