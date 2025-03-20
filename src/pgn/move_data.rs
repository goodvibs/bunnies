use crate::utils::PieceType;
use crate::r#move::Move;

#[derive(Debug, Clone)]
pub struct PgnMoveData {
    pub mv: Move,
    pub annotation: Option<String>,
    pub nag: Option<u8>,
}

impl PgnMoveData {
    pub fn render(&self, moved_piece: PieceType, disambiguation_str: &str, is_check: bool, is_checkmate: bool, is_capture: bool, include_annotations: bool, include_nags: bool) -> String {
        let mut result = self.mv.san(moved_piece, disambiguation_str, is_check, is_checkmate, is_capture);

        if include_annotations {
            if let Some(annotation) = &self.annotation {
                result.push_str(annotation);
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