//! This module provides functionality for parsing and rendering PGN (Portable Game Notation).

mod buffered_position_brancher;
mod buffered_position_context;
mod lexing_error;
mod move_data;
mod move_tree_node;
mod object;
mod parser;
mod parsing_error;
mod parsing_state;
mod position_context;
mod rendering_config;
mod token;
mod token_types;

pub use lexing_error::*;
use move_tree_node::*;
pub use object::*;
pub use parser::*;
pub use parsing_error::*;
pub use parsing_state::*;
pub use rendering_config::*;

#[cfg(test)]
mod tests {
    use crate::pgn::PgnParser;

    #[test]
    fn test_pgn_parsing_and_rendering() {
        // The PGN data with comments removed
        let pgn_input = r"1. e4 e5 2. Nf3 Nf6!!!! 3. Bc4 Nxe4 4. Nc3 Nc6 (4... Nxc3 5. dxc3??!! $20 { [%csl Gf6][%cal Gf7f6] } 5... f6 6. Nh4 $21 g6 7. f4 Qe7 8. f5 ) 5. O-O (5. Nxe4 d5 { [%cal Gd5e4,Gd5c4] } ) 5... Nxc3 6. dxc3 f6 7. Re1 d6 8. Nh4 g6 9. f4 Qe7 10. f5 Qg7 11. Qf3 Bd7 (11... g5 { [%csl Ge8] } 12. Qh5+ Kd8 { [%cal Gg5h4] } 13. Nf3 Bxf5 ) 12. b4 Be7 { [%csl Ge7][%cal Gf8e7] } (12... O-O-O 13. Bd5 b6 (13... g5 ) ) 13. Qe4 { [%csl Gg6][%cal Gf5g6] } 13... g5 (13... Nd8 ) 14. Nf3 O-O-O (14... Nd8 ) 15. a4 g4 16. Nh4 g3 17. h3 Rdf8 18. a5 Nd8 19. a6 Bc6 20. axb7+ Bxb7 21. Bd5 c6 22. Qc4 a6 23. Be3 Kd7 24. Be6+ Ke8 25. Rxa6 Bxa6 26. Qxa6 Rf7 27. Qc8 Bf8 28. Ra1 Rd7 29. Ra8 Qe7 30. Bb6 Bh6 31. Bxd7+ Kf8 32. Bxd8 Be3+ 33. Kf1 Kg7 34. Bxe7 Rxc8 35. Rxc8 d5 36. Nf3 d4 37. Bf8+ Kf7 38. Be6# { White wins by checkmate. } 1-0";

        // Create a parser and parse the PGN
        let mut parser = PgnParser::new(pgn_input);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse PGN: {:?}", result.err());

        // Render the parsed PGN
        let rendered_pgn = parser.constructed_object.to_string();

        // Expected PGN after parsing and rendering
        // This will need to be adjusted based on your actual expected output format
        // Especially with respect to move numbering and spacing
        let expected_pgn = r"1. e4 e5 2. Nf3 Nf6!!!! 3. Bc4 Nxe4 4. Nc3 Nc6 (4... Nxc3 5. dxc3??!! $20 f6 6. Nh4 $21 g6 7. f4 Qe7 8. f5) 5. O-O (5. Nxe4 d5) 5... Nxc3 6. dxc3 f6 7. Re1 d6 8. Nh4 g6 9. f4 Qe7 10. f5 Qg7 11. Qf3 Bd7 (11... g5 12. Qh5+ Kd8 13. Nf3 Bxf5) 12. b4 Be7 (12... O-O-O 13. Bd5 b6 (13... g5)) 13. Qe4 g5 (13... Nd8) 14. Nf3 O-O-O (14... Nd8) 15. a4 g4 16. Nh4 g3 17. h3 Rdf8 18. a5 Nd8 19. a6 Bc6 20. axb7+ Bxb7 21. Bd5 c6 22. Qc4 a6 23. Be3 Kd7 24. Be6+ Ke8 25. Rxa6 Bxa6 26. Qxa6 Rf7 27. Qc8 Bf8 28. Ra1 Rd7 29. Ra8 Qe7 30. Bb6 Bh6 31. Bxd7+ Kf8 32. Bxd8 Be3+ 33. Kf1 Kg7 34. Bxe7 Rxc8 35. Rxc8 d5 36. Nf3 d4 37. Bf8+ Kf7 38. Be6#";

        // Compare the rendered PGN with the expected PGN
        // This assertion might need to be adjusted depending on how your rendering handles
        // whitespace, since we're comparing strings directly
        assert_eq!(
            rendered_pgn.replace(" ", "").replace("\n", ""),
            expected_pgn.replace(" ", "").replace("\n", ""),
            "Rendered PGN doesn't match expected PGN"
        );

        // Optional: Print the rendered PGN for manual inspection
        println!("Rendered PGN:\n{}", rendered_pgn);
    }
}
