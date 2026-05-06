use crate::Piece;
use crate::r#move::Move;

#[derive(Debug, Clone)]
pub(crate) struct PgnMoveData {
    pub(crate) move_: Move,
    pub(crate) annotation: Option<String>,
    pub(crate) nag: Option<u8>,
}

impl PgnMoveData {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render(
        &self,
        moved_piece: Piece,
        disambiguation_str: &str,
        is_check: bool,
        is_checkmate: bool,
        is_capture: bool,
        include_annotations: bool,
        include_nags: bool,
    ) -> String {
        let mut result = self.move_.san(
            moved_piece,
            disambiguation_str,
            is_check,
            is_checkmate,
            is_capture,
        );

        if include_annotations && let Some(annotation) = &self.annotation {
            result.push_str(annotation);
        }

        if include_nags && let Some(nag) = self.nag {
            result.push_str(&format!(" ${}", nag));
        }

        result
    }
}
