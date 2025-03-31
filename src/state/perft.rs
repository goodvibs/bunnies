use crate::state::GameState;

fn count_nodes(state: &mut GameState, depth: u8) -> u64 {
    if !state.is_probably_valid() {
        0
    } else if depth == 0 {
        1
    } else {
        let mut total_nodes = 0;

        let pseudolegal_moves = state.calc_pseudolegal_moves();

        for mv in pseudolegal_moves {
            state.make_move(mv);
            total_nodes += count_nodes(state, depth - 1);
            state.unmake_move(mv);
        }

        total_nodes
    }
}

impl GameState {
    pub fn perft(&self, depth: u8) -> u64 {
        count_nodes(&mut self.clone(), depth)
    }
}
