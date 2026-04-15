use crate::r#move::MoveList;
use crate::position::{Position, SideState};

fn count_nodes<const N: usize, S: SideState>(pos: &mut Position<N, S>, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }
    let mut moves = MoveList::new();
    pos.generate_legal_moves(&mut moves);
    let mut total = 0u64;
    for &mv in moves.as_slice() {
        pos.make_move_in_place(mv)
            .expect("perft depth within context stack");
        // SAFETY: After `make_move_in_place`, board/context match `Position<N, S::Other>`; same
        // memory as `pos` but we must not use `pos` as `&mut Position<N, S>` until `unmake_move_in_place`.
        unsafe {
            let child = &mut *std::ptr::from_mut(pos).cast::<Position<N, S::Other>>();
            total += count_nodes(child, depth - 1);
            child.unmake_move_in_place(mv);
        }
    }
    total
}

impl<const N: usize, S: SideState> Position<N, S> {
    /// [`Self::perft`] without cloning `self` first; reuses this `Position` (must be at the search root).
    pub fn perft(&mut self, depth: u8) -> u64 {
        count_nodes(self, depth)
    }
}
