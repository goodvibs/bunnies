use std::cell::RefCell;
use std::rc::Rc;
use crate::color::Color;
use crate::piece_type::PieceType;
use crate::r#move::{Move, MoveFlag};
use crate::state::{State, Termination};

#[derive(Debug, Clone)]
pub struct MoveData {
    pub mv: Move,
    pub annotation: Option<String>,
    pub nag: Option<u8>,
}

impl MoveData {
    fn render(&self, moved_piece: PieceType, disambiguation_str: &str, is_check: bool, is_checkmate: bool, is_capture: bool, include_annotations: bool, include_nags: bool) -> String {
        let mut result = self.mv.san(moved_piece, disambiguation_str, is_check, is_checkmate, is_capture);

        if include_annotations {
            if let Some(annotation) = &self.annotation {
                result.push_str(&format!(" {}", annotation));
            }
        }

        if include_nags {
            if let Some(nag) = self.nag {
                result.push_str(&format!(" ${}", nag));
            }
        }

        result
    }
}

pub struct MoveTreeNode {
    pub move_data: Option<MoveData>, // None for the root node
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
    pub fn new(move_data: MoveData, comment: Option<String>) -> MoveTreeNode {
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

    pub fn render(&self, mut state: State, last_continuations: &[Rc<RefCell<MoveTreeNode>>], include_variations: bool, include_annotations: bool, include_nags: bool, include_comments: bool, depth: u16) -> String {
        let rendered_last_continuations = {
            let mut result = String::new();
            for continuation in last_continuations {
                let rendered_continuation = &continuation.borrow().render(
                    state.clone(),
                    &last_continuations,
                    include_variations,
                    include_annotations,
                    include_nags,
                    include_comments,
                    depth + 1
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
            } else {
                // For black's moves in main line, only add "..." if it's the first move shown
                if depth == 0 && self.move_data.is_some() && last_continuations.is_empty() {
                    format!("{}... ", state.get_fullmove())
                } else {
                    "".to_string()
                }
            };

            let all_moves = state.calc_legal_moves();
            let all_other_moves: Vec<Move> = all_moves.iter().filter(|m| **m != mv).cloned().collect();
            let disambiguation_moves: Vec<Move> = all_other_moves.iter().filter(|m| m.get_destination() == mv_dest && state.board.get_piece_type_at(m.get_source()) == moved_piece).cloned().collect::<Vec<Move>>();
            let disambiguation_str = match disambiguation_moves.len() {
                0 => "".to_string(),
                _ => {
                    let file = mv_source.get_file();
                    let rank = mv_source.get_rank();
                    let is_file_ambiguous = disambiguation_moves.iter().any(|m| m.get_source().get_file() == file);
                    let is_rank_ambiguous = disambiguation_moves.iter().any(|m| m.get_source().get_rank() == rank);
                    match (is_file_ambiguous, is_rank_ambiguous) {
                        (true, true) => mv_source.to_string(),
                        (true, false) => mv_source.get_file_char().to_string(),
                        (false, true) => mv_source.get_rank_char().to_string(),
                        (false, false) => "".to_string()
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
            move_number_str + &move_data.render(moved_piece, disambiguation_str.as_str(), is_check, is_checkmate, is_capture, include_annotations, include_nags)
        } else {
            "".to_string()
        };

        let rendered_comment = if include_comments {
            if let Some(comment) = &self.comment {
                format!(" {{ {} }}", comment)
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        };

        let up_till_now = format!("{}{}{}", rendered_move, rendered_comment, rendered_last_continuations);

        if include_variations && self.has_continuations() {
            let main_continuation = self.get_main_continuation().unwrap();
            let alternative_continuations = self.get_alternative_continuations();
            let rendered_main_continuation = main_continuation.borrow().render(
                state,
                &alternative_continuations,
                include_variations,
                include_annotations,
                include_nags,
                include_comments,
                depth + 1
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
    use super::*;
    use crate::state::State;

    #[test]
    fn test_pgn_parsing_and_rendering() {
        // The PGN data with comments removed
        let pgn_data = r"1. e4 e5 2. Nf3 Nf6 3. Bc4 Nxe4 4. Nc3 Nc6 (4... Nxc3 5. dxc3 5... f6 6. Nh4 g6 7. f4 Qe7 8. f5) 5. O-O (5. Nxe4 d5) 5... Nxc3 6. dxc3 f6 7. Re1 d6 8. Nh4 g6 9. f4 Qe7 10. f5 Qg7 11. Qf3 Bd7 (11... g5 12. Qh5+ Kd8 13. Nf3 Bxf5) 12. b4 Be7 (12... O-O-O 13. Bd5 b6 (13... g5)) 13. Qe4 13... g5 (13... Nd8) 14. Nf3 O-O-O (14... Nd8) 15. a4 g4 16. Nh4 g3 17. h3 Rdf8 18. a5 Nd8 19. a6 Bc6 20. axb7+ Bxb7 21. Bd5 c6 22. Qc4 a6 23. Be3 Kd7 24. Be6+ Ke8 25. Rxa6 Bxa6 26. Qxa6 Rf7 27. Qc8 Bf8 28. Ra1 Rd7 29. Ra8 Qe7 30. Bb6 Bh6 31. Bxd7+ Kf8 32. Bxd8 Be3+ 33. Kf1 Kg7 34. Bxe7 Rxc8 35. Rxc8 d5 36. Nf3 d4 37. Bf8+ Kf7 38. Be6# 1-0";

        // Create a parser and parse the PGN
        let mut parser = crate::pgn::PgnParser::new(pgn_data);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse PGN: {:?}", result.err());

        // Render the parsed PGN
        let rendered_pgn = parser.constructed_object.tree_root.borrow().render(
            State::initial(),
            &[],
            true,  // include_variations
            true,  // include_annotations
            true,  // include_nags
            true, // include_comments
            0      // depth
        );

        // Expected PGN after parsing and rendering
        // This will need to be adjusted based on your actual expected output format
        // Especially with respect to move numbering and spacing
        let expected_pgn = r"1. e4 e5 2. Nf3 Nf6 3. Bc4 Nxe4 4. Nc3 Nc6 (4... Nxc3 5. dxc3 5... f6 6. Nh4 g6 7. f4 Qe7 8. f5) 5. O-O (5. Nxe4 d5) 5... Nxc3 6. dxc3 f6 7. Re1 d6 8. Nh4 g6 9. f4 Qe7 10. f5 Qg7 11. Qf3 Bd7 (11... g5 12. Qh5+ Kd8 13. Nf3 Bxf5) 12. b4 Be7 (12... O-O-O 13. Bd5 b6 (13... g5)) 13. Qe4 13... g5 (13... Nd8) 14. Nf3 O-O-O (14... Nd8) 15. a4 g4 16. Nh4 g3 17. h3 Rdf8 18. a5 Nd8 19. a6 Bc6 20. axb7+ Bxb7 21. Bd5 c6 22. Qc4 a6 23. Be3 Kd7 24. Be6+ Ke8 25. Rxa6 Bxa6 26. Qxa6 Rf7 27. Qc8 Bf8 28. Ra1 Rd7 29. Ra8 Qe7 30. Bb6 Bh6 31. Bxd7+ Kf8 32. Bxd8 Be3+ 33. Kf1 Kg7 34. Bxe7 Rxc8 35. Rxc8 d5 36. Nf3 d4 37. Bf8+ Kf7 38. Be6#";

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