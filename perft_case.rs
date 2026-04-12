use bunnies::Position;

#[derive(Clone, Copy, Debug)]
pub enum PerftCase {
    Initial,
    Kiwipete,
    Position3,
    Position4,
    Position5,
}

impl PerftCase {
    pub const fn name(self) -> &'static str {
        match self {
            PerftCase::Initial => "initial",
            PerftCase::Kiwipete => "kiwipete",
            PerftCase::Position3 => "position3",
            PerftCase::Position4 => "position4",
            PerftCase::Position5 => "position5",
        }
    }

    pub fn position<const N: usize>(self) -> Position<N> {
        match self {
            PerftCase::Initial => Position::<N>::initial(),
            PerftCase::Kiwipete => Position::<N>::from_fen(
                "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            )
            .unwrap(),
            PerftCase::Position3 => {
                Position::<N>::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap()
            }
            PerftCase::Position4 => Position::<N>::from_fen(
                "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1",
            )
            .unwrap(),
            PerftCase::Position5 => {
                Position::<N>::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8")
                    .unwrap()
            }
        }
    }

    const fn nodes_depth_map(self) -> &'static [u64; 8] {
        match self {
            PerftCase::Initial => &[
                1,
                20,
                400,
                8_902,
                197_281,
                4_865_609,
                119_060_324,
                3_195_901_860,
            ],
            PerftCase::Kiwipete => &[
                1,
                48,
                2_039,
                97_862,
                4_085_603,
                193_690_690,
                8_031_647_685,
                0,
            ],
            PerftCase::Position3 => &[1, 14, 191, 2_812, 43_238, 674_624, 11_030_083, 178_633_661],
            PerftCase::Position4 => &[1, 6, 264, 9_467, 422_333, 15_833_292, 706_045_033, 0],
            PerftCase::Position5 => &[1, 44, 1_486, 62_379, 2_103_487, 89_941_194, 0, 0],
        }
    }

    pub const fn nodes_at_depth(self, depth: u8) -> u64 {
        self.nodes_depth_map()[depth as usize]
    }

    pub fn verify_perft(self, depth: u8, nodes_observed: u64) {
        let expected = self.nodes_at_depth(depth);

        assert_eq!(
            nodes_observed,
            expected,
            "perft mismatch for {} at depth {} (expected {}, got {})",
            self.name(),
            depth,
            expected,
            nodes_observed
        );
    }
}
