use crate::Color;
use crate::r#move::MoveList;
use crate::position::Position;

fn count_nodes<const N: usize, const STM: Color>(pos: &mut Position<N, STM>, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }
    let mut moves = MoveList::new();
    pos.generate_legal_moves(&mut moves);

    if depth == 1 {
        return moves.len() as u64;
    }

    let mut total = 0u64;
    for &mv in moves.as_slice() {
        pos.make_move(mv);
        match STM {
            Color::White => {
                // SAFETY: After `make_move_in_place`, board/context match `Position<N, { Color::Black }>`;
                // same memory as `pos` until `unmake_move_in_place`.
                let child = unsafe {
                    &mut *std::ptr::from_mut(pos).cast::<Position<N, { Color::Black }>>()
                };
                total += count_nodes(child, depth - 1);
                child.unmake_move(mv);
            }
            Color::Black => {
                let child = unsafe {
                    &mut *std::ptr::from_mut(pos).cast::<Position<N, { Color::White }>>()
                };
                total += count_nodes(child, depth - 1);
                child.unmake_move(mv);
            }
        }
    }
    total
}

impl<const N: usize, const STM: Color> Position<N, STM> {
    /// Counts leaf nodes to `depth` (divide-perft), mutating this position in place.
    /// Must be called on the search root; context stack must fit the traversal depth.
    #[inline]
    pub fn perft(&mut self, depth: u8) -> u64 {
        count_nodes(self, depth)
    }
}
