use crate::piece_type::PieceType;
use crate::r#move::{Move};
use crate::r#move::move_flag::MoveFlag;

impl Move {
    /// Returns the SAN (Standard Algebraic Notation) representation of the move.
    pub fn san(&self, moved_piece: PieceType, disambiguation_file: Option<char>, disambiguation_rank: Option<char>, is_check: bool, is_checkmate: bool, is_capture: bool) -> String {
        let dst_square = self.get_destination();
        let flag = self.get_flag();

        let move_str = if flag == MoveFlag::Castling {
            if dst_square.get_file() == 6 {
                "O-O".to_string()
            } else {
                "O-O-O".to_string()
            }
        } else {
            let src_square = self.get_source();
            let promotion = self.get_promotion();

            let piece_str = match moved_piece {
                PieceType::Pawn => if is_capture { src_square.get_file_char().to_string().as_str() } else { "" },
                PieceType::Knight => "N",
                PieceType::Bishop => "B",
                PieceType::Rook => "R",
                PieceType::Queen => "Q",
                PieceType::King => "K",
                _ => panic!("Invalid piece type")
            };

            let capture_str = if is_capture { "x" } else { "" };

            let promotion_str = match flag {
                MoveFlag::Promotion => format!("={}", promotion.to_char()),
                _ => "".to_string()
            };

            let disambiguation_str = match (disambiguation_file, disambiguation_rank) {
                (Some(file), Some(rank)) => format!("{}{}", file, rank),
                (Some(file), None) => format!("{}", file),
                (None, Some(rank)) => format!("{}", rank),
                (None, None) => "".to_string()
            };

            format!("{}{}{}{}{}", piece_str, disambiguation_str, capture_str, dst_square.to_string(), promotion_str)
        };

        let check_or_checkmate_str = if is_checkmate { "#" } else if is_check { "+" } else { "" };

        format!("{}{}", move_str, check_or_checkmate_str)
    }
}