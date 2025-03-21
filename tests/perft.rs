use bunnies::state::State;

fn count_nodes(state: &mut State, depth: u8) -> u64 {
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

fn perft(mut state: State, depth: u8, expected_nodes: u64) {
    let nodes = count_nodes(&mut state, depth);
    assert_eq!(nodes, expected_nodes, "Expected {} nodes at depth {}, but got {}", expected_nodes, depth, nodes);
}

#[test]
fn test_initial_position() {
    let initial_state = State::initial();
    // perft(initial_state, 1, 20);
    // perft(initial_state, 2, 400);
    // perft(initial_state, 3, 8902);
    // perft(initial_state, 4, 197281);
    perft(initial_state, 5, 4865609);
    // perft(initial_state, 6, 119060324);
}

#[test]
fn test_kiwipete() {
    let mut state = State::from_fen("rnbqkb1r/pppppppp/8/8/8/8/PPPPPPPP/RNBQKB1R w KQkq - 0 1").unwrap();
    perft(state, 1, 48);
    // perft(state, 2, 2039);
    // perft(state, 3, 97862);
    // perft(state, 4, 4085603);
    // perft(state, 5, 193690690);
    // perft(state, 6, 8031647685);
}