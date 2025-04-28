use crate::PieceType;
use crate::r#move::Move;
use crate::r#move::flag::MoveFlag;

impl Move {
    /// Returns the SAN (Standard Algebraic Notation) representation of the move.
    pub fn san(
        &self,
        moved_piece: PieceType,
        disambiguation_str: &str,
        is_check: bool,
        is_checkmate: bool,
        is_capture: bool,
    ) -> String {
        let dst_square = self.destination();
        let flag = self.flag();

        let move_str = if flag == MoveFlag::Castling {
            match dst_square.file() {
                6 => "O-O".to_string(),
                2 => "O-O-O".to_string(),
                _ => panic!("Invalid castling move"),
            }
        } else {
            let src_square = self.source();
            let promotion = self.promotion();

            let piece_str = match moved_piece {
                PieceType::Pawn => {
                    if is_capture {
                        src_square.file_char().to_string()
                    } else {
                        "".to_string()
                    }
                }
                PieceType::Knight => "N".to_string(),
                PieceType::Bishop => "B".to_string(),
                PieceType::Rook => "R".to_string(),
                PieceType::Queen => "Q".to_string(),
                PieceType::King => "K".to_string(),
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
                piece_str,
                disambiguation_str,
                capture_str,
                dst_square.to_string(),
                promotion_str
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
