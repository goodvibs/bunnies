use bunnies::Position;

fn run_perft_test<const N: usize>(state: Position<N>, depth: u8, expected_nodes: u64) {
    let nodes = state.perft(depth);
    assert_eq!(
        nodes, expected_nodes,
        "Expected {} nodes at depth {}, but got {}",
        expected_nodes, depth, nodes
    );
}

#[test]
fn test_initial_position() {
    // Stack must hold root + 6 nested make_move frames (perft depth 6).
    let initial_state = Position::<7>::initial();
    run_perft_test(initial_state, 6, 119060324); // ~ 5 seconds on M1 Pro
}

#[test]
fn test_kiwipete() {
    let state = Position::<6>::from_fen(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    )
    .unwrap();
    run_perft_test(state, 5, 193690690); // ~ 8 seconds on M1 Pro
}

#[test]
fn test_position_3() {
    let state = Position::<8>::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();
    run_perft_test(state, 7, 178633661); // ~ 8 seconds on M1 Pro
}

#[test]
fn test_position_4() {
    let state =
        Position::<6>::from_fen("r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1")
            .unwrap();
    run_perft_test(state, 5, 15833292); // < 1 second on M1 Pro
}

#[test]
fn test_position_5() {
    let state =
        Position::<6>::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8")
            .unwrap();
    run_perft_test(state, 5, 89941194); // ~ 4 seconds on M1 Pro
}
