//! Standard Algebraic Notation (SAN) rendering for moves.

use crate::types::{File, Move, MoveFlag, Piece};

impl Move {
    /// Renders this move in SAN format with full disambiguation and check/mate indicators.
    ///
    /// `moved_piece` should be the piece from the origin square before move execution.
    /// `disambiguation_str` is the already-computed SAN disambiguator (e.g. `"b"` / `"3"` / `"b3"`).
    /// `is_check` and `is_checkmate` refer to the resulting position.
    /// `is_capture` should reflect board semantics (including en-passant).
    pub fn san(
        &self,
        moved_piece: Piece,
        disambiguation_str: &str,
        is_check: bool,
        is_checkmate: bool,
        is_capture: bool,
    ) -> String {
        let to = self.to();
        let flag = self.flag();

        let move_str = if flag == MoveFlag::Castling {
            match to.file() {
                File::G => "O-O".to_string(),
                File::C => "O-O-O".to_string(),
                _ => panic!("Invalid castling move"),
            }
        } else {
            let from = self.from();
            let promotion = self.promotion();

            let piece_str = match moved_piece {
                Piece::Pawn => {
                    if is_capture {
                        from.file_char().to_string()
                    } else {
                        "".to_string()
                    }
                }
                Piece::Knight => "N".to_string(),
                Piece::Bishop => "B".to_string(),
                Piece::Rook => "R".to_string(),
                Piece::Queen => "Q".to_string(),
                Piece::King => "K".to_string(),
                _ => panic!("Invalid piece type"),
            };

            let capture_str = if is_capture { "x" } else { "" };

            let promotion_str = if flag == MoveFlag::Promotion {
                format!("={}", promotion.uppercase_ascii())
            } else {
                "".to_string()
            };

            format!(
                "{}{}{}{}{}",
                piece_str, disambiguation_str, capture_str, to, promotion_str
            )
        };

        let check_or_checkmate_str = if is_checkmate {
            "#"
        } else if is_check {
            "+"
        } else {
            ""
        };

        format!("{}{}", move_str, check_or_checkmate_str)
    }
}
