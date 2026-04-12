use crate::position::Position;

fn count_nodes<const N: usize>(state: &mut Position<N>, depth: u8) -> u64 {
    if depth == 0 {
        1
    } else {
        let mut total_nodes = 0;

        let pseudolegal_moves = state.moves();

        for mv in pseudolegal_moves {
            state
                .make_move(mv)
                .expect("perft depth within context stack");
            total_nodes += count_nodes(state, depth - 1);
            state.unmake_move(mv);
        }

        total_nodes
    }
}

impl<const N: usize> Position<N> {
    pub fn perft(&self, depth: u8) -> u64 {
        count_nodes(&mut self.clone(), depth)
    }
}
