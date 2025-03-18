use std::cell::RefCell;
use std::rc::Rc;
use crate::color::Color;
use crate::pgn::move_data::PgnMoveData;
use crate::pgn::rendering_config::PgnRenderingConfig;
use crate::piece_type::PieceType;
use crate::r#move::{Move, MoveFlag};
use crate::state::{State, Termination};

pub struct MoveTreeNode {
    pub move_data: Option<PgnMoveData>, // None for the root node
    pub comment: Option<String>, // Root node may have a comment, so this is not part of MoveData
    pub continuations: Vec<Rc<RefCell<MoveTreeNode>>>,
}

impl MoveTreeNode {
    pub fn new_root(comment: Option<String>) -> MoveTreeNode {
        MoveTreeNode {
            move_data: None,
            comment,
            continuations: Vec::new(),
        }
    }
    pub fn new(move_data: PgnMoveData, comment: Option<String>) -> MoveTreeNode {
        MoveTreeNode {
            move_data: Some(move_data),
            comment,
            continuations: Vec::new(),
        }
    }

    pub fn add_continuation(&mut self, continuation: &Rc<RefCell<MoveTreeNode>>) {
        self.continuations.push(Rc::clone(continuation));
    }

    pub fn has_continuations(&self) -> bool {
        !self.continuations.is_empty()
    }

    pub fn get_main_continuation(&self) -> Option<Rc<RefCell<MoveTreeNode>>> {
        self.continuations.first().map(|c| Rc::clone(c))
    }

    pub fn has_multiple_continuations(&self) -> bool {
        self.continuations.len() > 1
    }

    pub fn get_alternative_continuations(&self) -> Vec<Rc<RefCell<MoveTreeNode>>> {
        self.continuations.iter().skip(1).map(|c| Rc::clone(c)).collect()
    }

    pub fn render(&self, mut state: State, last_continuations: &[Rc<RefCell<MoveTreeNode>>], include_variations: bool, config: PgnRenderingConfig, depth: u16, remind_fullmove: bool) -> String {
        let rendered_last_continuations = {
            let mut result = String::new();
            for continuation in last_continuations {
                let rendered_continuation = &continuation.borrow().render(
                    state.clone(),
                    &[],
                    include_variations,
                    config,
                    depth + 1,
                    true
                );
                result += &format!(" ({})", rendered_continuation);
            }
            result
        };

        let rendered_move = if let Some(move_data) = &self.move_data {
            let mv = move_data.mv;
            let mv_source = mv.get_source();
            let mv_dest = mv.get_destination();
            let moved_piece = state.board.get_piece_type_at(mv_source);
            let side_to_move = state.side_to_move;

            // Add move number for white's move or at the start of a variation
            let move_number_str = if side_to_move == Color::White {
                format!("{}. ", state.get_fullmove())
            } else if remind_fullmove {
                format!("{}... ", state.get_fullmove())
            } else {
                "".to_string()
            };

            let disambiguation_str = match moved_piece {
                PieceType::Pawn | PieceType::King => "".to_string(),
                PieceType::NoPieceType => panic!("Invalid piece type"),
                _ => {
                    let all_moves = state.calc_legal_moves();
                    let all_other_moves: Vec<Move> = all_moves.iter().filter(|m| **m != mv).cloned().collect();
                    let disambiguation_moves: Vec<Move> = all_other_moves.iter().filter(|m| m.get_destination() == mv_dest && state.board.get_piece_type_at(m.get_source()) == moved_piece).cloned().collect::<Vec<Move>>();
                    match disambiguation_moves.len() {
                        0 => "".to_string(),
                        _ => {
                            let file = mv_source.get_file();
                            let rank = mv_source.get_rank();
                            let is_file_ambiguous = disambiguation_moves.iter().any(|m| m.get_source().get_file() == file);
                            let is_rank_ambiguous = disambiguation_moves.iter().any(|m| m.get_source().get_rank() == rank);
                            match (is_file_ambiguous, is_rank_ambiguous) {
                                (true, true) => mv_source.to_string(),
                                (true, false) => mv_source.get_rank_char().to_string(),
                                (false, true) => mv_source.get_file_char().to_string(),
                                (false, false) => "".to_string()
                            }
                        }
                    }
                }
            };

            let is_capture = match mv.get_flag() {
                MoveFlag::EnPassant => true,
                MoveFlag::Castling => false,
                MoveFlag::NormalMove | MoveFlag::Promotion => state.board.get_piece_type_at(mv_dest) != PieceType::NoPieceType,
            };

            state.make_move(mv);
            state.check_and_update_termination();
            let is_checkmate = state.termination == Some(Termination::Checkmate);
            let is_check = state.board.is_color_in_check(state.side_to_move);

            // Combine move number and move
            move_number_str + &move_data.render(moved_piece, disambiguation_str.as_str(), is_check, is_checkmate, is_capture, config.include_annotations, config.include_nags)
        } else {
            "".to_string()
        };

        let rendered_comment = if config.include_comments {
            if let Some(comment) = &self.comment {
                format!(" {{ {} }}", comment)
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        };

        let up_till_now = format!("{}{}{}", rendered_move, rendered_comment, rendered_last_continuations);

        if self.has_continuations() {
            let main_continuation = self.get_main_continuation().unwrap();
            let alternative_continuations = match include_variations {
                true => self.get_alternative_continuations(),
                false => Vec::with_capacity(0)
            };
            let rendered_main_continuation = main_continuation.borrow().render(
                state,
                &alternative_continuations,
                include_variations,
                config,
                depth + 1,
                !last_continuations.is_empty()
            );

            // Add appropriate spacing before the next move
            if up_till_now.is_empty() {
                rendered_main_continuation
            } else {
                format!("{} {}", up_till_now, rendered_main_continuation)
            }
        } else {
            up_till_now
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::pgn::PgnParser;
    use super::*;
    use crate::state::State;

    #[test]
    fn test_pgn_parsing_and_rendering() {
        // The PGN data with comments removed
        let pgn_data = r"1. e4 e5 2. Nf3 Nf6!!!! 3. Bc4 Nxe4 4. Nc3 Nc6 (4... Nxc3 5. dxc3??!! $20 { [%csl Gf6][%cal Gf7f6] } 5... f6 6. Nh4 $21 g6 7. f4 Qe7 8. f5 ) 5. O-O (5. Nxe4 d5 { [%cal Gd5e4,Gd5c4] } ) 5... Nxc3 6. dxc3 f6 7. Re1 d6 8. Nh4 g6 9. f4 Qe7 10. f5 Qg7 11. Qf3 Bd7 (11... g5 { [%csl Ge8] } 12. Qh5+ Kd8 { [%cal Gg5h4] } 13. Nf3 Bxf5 ) 12. b4 Be7 { [%csl Ge7][%cal Gf8e7] } (12... O-O-O 13. Bd5 b6 (13... g5 ) ) 13. Qe4 { [%csl Gg6][%cal Gf5g6] } 13... g5 (13... Nd8 ) 14. Nf3 O-O-O (14... Nd8 ) 15. a4 g4 16. Nh4 g3 17. h3 Rdf8 18. a5 Nd8 19. a6 Bc6 20. axb7+ Bxb7 21. Bd5 c6 22. Qc4 a6 23. Be3 Kd7 24. Be6+ Ke8 25. Rxa6 Bxa6 26. Qxa6 Rf7 27. Qc8 Bf8 28. Ra1 Rd7 29. Ra8 Qe7 30. Bb6 Bh6 31. Bxd7+ Kf8 32. Bxd8 Be3+ 33. Kf1 Kg7 34. Bxe7 Rxc8 35. Rxc8 d5 36. Nf3 d4 37. Bf8+ Kf7 38. Be6# { White wins by checkmate. } 1-0";

        // Create a parser and parse the PGN
        let mut parser = PgnParser::new(pgn_data);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse PGN: {:?}", result.err());

        // Render the parsed PGN
        let rendered_pgn = parser.constructed_object.tree_root.borrow().render(
            State::initial(),
            &[],
            true,   // include_variations
            PgnRenderingConfig::default(),
            0,      // depth
            false   // remind_fullmove
        );

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