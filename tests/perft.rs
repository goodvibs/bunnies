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
    // perft(initial_state, 6, 119060324); // highest tested
}

#[test]
fn test_kiwipete() {
    let state = State::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap();
    // perft(state, 1, 48);
    // perft(state, 2, 2039);
    // perft(state, 3, 97862);
    perft(state, 4, 4085603);
    // perft(state, 5, 193690690); // highest tested
    // perft(state, 6, 8031647685);
}

#[test]
fn test_position_3() {
    let state = State::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();
    // perft(state, 1, 14);
    // perft(state, 2, 191);
    // perft(state, 3, 2812);
    // perft(state, 4, 43238);
    // perft(state, 5, 674624);
    perft(state, 6, 11030083); // highest tested
    // perft(state, 7, 178633661);
}

#[test]
fn test_position_4() {
    let state = State::from_fen("r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1").unwrap();
    // perft(state, 1, 6);
    // perft(state, 2, 264);
    // perft(state, 3, 9467);
    perft(state, 4, 422333);
    // perft(state, 5, 15833292); // highest tested
    // perft(state, 6, 706045033);
}

#[test]
fn test_position_5() {
    let state = State::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8").unwrap();
    // perft(state, 1, 44);
    // perft(state, 2, 1486);
    // perft(state, 3, 62379);
    perft(state, 4, 2103487);
    // perft(state, 5, 89941194); // highest tested
}