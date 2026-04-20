include!(concat!(env!("CARGO_MANIFEST_DIR"), "/perft_case.rs"));

macro_rules! define_perft_tests {
    ($($name:ident => ($case:expr, $depth:literal);)+) => {
        $(
            #[test]
            fn $name() {
                const CONTEXTS_CAPACITY: usize = $depth + 1;
                let mut pos = ($case).position::<CONTEXTS_CAPACITY>();
                let nodes_observed = pos.perft($depth);
                ($case).verify_perft($depth, nodes_observed);
            }
        )+
    };
}

define_perft_tests! {
    test_perft_initial_position => (PerftCase::Initial, 6);
    test_perft_kiwipete => (PerftCase::Kiwipete, 5);
    test_perft_position_3 => (PerftCase::Position3, 7);
    test_perft_position_4 => (PerftCase::Position4, 6);
    test_perft_position_5 => (PerftCase::Position5, 5);
}
