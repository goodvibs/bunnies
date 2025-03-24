use crate::state::State;

pub fn count_nodes(state: &mut State, depth: u8) -> u64 {
    if !state.is_probably_valid() {
        return 0;
    }
    if depth == 0 {
        return 1;
    }

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

impl State {
    pub fn perft(&self, depth: u8) -> u64 {
        count_nodes(&mut self.clone(), depth)
    }
}