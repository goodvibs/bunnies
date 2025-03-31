use crate::position::Position;

fn count_nodes(state: &mut Position, depth: u8) -> u64 {
    if !state.is_probably_valid() {
        0
    } else if depth == 0 {
        1
    } else {
        let mut total_nodes = 0;

        let mut attacks = 0;
        let pseudolegal_moves = state.calc_pseudolegal_moves(&mut attacks);

        for mv in pseudolegal_moves {
            state.make_move(mv, attacks);
            total_nodes += count_nodes(state, depth - 1);
            state.unmake_move(mv);
        }

        total_nodes
    }
}

impl Position {
    pub fn perft(&self, depth: u8) -> u64 {
        count_nodes(&mut self.clone(), depth)
    }
}
