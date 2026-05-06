//! Contains [`Position::make_move`] and [`Position::unmake_move`].

use crate::Color;
use crate::ConstDoublePawnPushFile;
use crate::DoublePawnPushFile;
use crate::File;
use crate::Flank;
use crate::Move;
use crate::MoveFlag;
use crate::Piece;
use crate::Position;
use crate::PositionContext;
use crate::Rank;
use crate::Square;

impl<const N: usize, const STM: Color> Position<N, STM> {
    /// Applies a move in place, updating board state for the opponent.
    ///
    /// After this returns, the layout matches the resulting type of [`Position::make_move`]
    /// (`Position<N, { STM.other() }>`).
    pub fn make_move(&mut self, move_: Move) {
        debug_assert!(self.num_contexts < N);

        let from = move_.from();
        let to = move_.to();
        let flag = move_.flag();

        let mut new_context = PositionContext::blank();
        new_context.halfmove_clock = self.context().halfmove_clock + 1;
        new_context.castling_rights = self.context().castling_rights;

        let piece_at_to = self.board.piece_at(to);
        if piece_at_to != Piece::Null {
            self.board
                .remove_piece_and_color(STM.other(), piece_at_to, to);
            new_context.captured_piece = piece_at_to;
            new_context.halfmove_clock = 0;
        }

        let piece_at_from = self.board.piece_at(from);
        if piece_at_from == Piece::Pawn {
            new_context.halfmove_clock = 0;
            new_context.double_pawn_push_file = DoublePawnPushFile::from_pawn_step(from, to);
        }

        self.board
            .move_piece_and_color(STM, piece_at_from, from, to);

        match flag {
            MoveFlag::Promotion => {
                self.board.remove_piece_at(Piece::Pawn, to);
                self.board.put_piece_at(move_.promotion(), to);
                new_context.halfmove_clock = 0;
            }
            MoveFlag::EnPassant => {
                let capture_square =
                    Square::from_u8((to as u8).wrapping_add_signed(en_passant_capture_offset(STM)));
                self.board
                    .remove_piece_and_color(STM.other(), Piece::Pawn, capture_square);
                new_context.captured_piece = Piece::Pawn;
                new_context.halfmove_clock = 0;
            }
            MoveFlag::Castling => {
                let flank = to.file().flank();
                let rook_from = castling_rook_from_square(flank, STM);
                let rook_to = castling_rook_to_square(flank, STM);
                self.board.move_color(STM, rook_from, rook_to);
                self.board.move_piece(Piece::Rook, rook_from, rook_to);
                new_context.halfmove_clock = 0;
            }
            _ => {}
        }

        new_context.castling_rights = new_context.castling_rights.after_move(from).after_move(to);

        self.halfmove += 1;
        self.push_context(new_context);
        self.update_pins_and_checks_for_stm(STM.other());
    }

    /// Undoes a move in place. After this, memory matches the resulting type of [`Position::unmake_move`].
    pub fn unmake_move(&mut self, move_: Move) {
        let from = move_.from();
        let to = move_.to();
        let flag = move_.flag();
        let side_just_moved = STM.other();

        let piece_at_to = self.board.piece_at(to);
        self.board
            .move_piece_and_color(side_just_moved, piece_at_to, to, from);

        let captured_piece = self.context().captured_piece;
        if captured_piece != Piece::Null {
            self.board.put_piece_and_color(STM, captured_piece, to);
        }

        match flag {
            MoveFlag::NormalMove => {}
            MoveFlag::Promotion => {
                let promoted = self.board.piece_at(from);
                self.board.remove_piece_at(promoted, from);
                self.board.put_piece_at(Piece::Pawn, from);
            }
            MoveFlag::EnPassant => {
                let capture_square = Square::from_u8(
                    (to as u8).wrapping_add_signed(en_passant_capture_offset(side_just_moved)),
                );
                self.board
                    .move_piece_and_color(STM, Piece::Pawn, to, capture_square);
            }
            MoveFlag::Castling => {
                let flank = to.file().flank();
                let rook_from = castling_rook_from_square(flank, side_just_moved);
                let rook_to = castling_rook_to_square(flank, side_just_moved);
                self.board
                    .move_piece_and_color(side_just_moved, Piece::Rook, rook_to, rook_from);
            }
        }

        self.halfmove -= 1;
        self.decrement_context_stack_for_unmake();
    }
}

const fn en_passant_capture_offset(stm: Color) -> i8 {
    match stm {
        Color::White => 8,
        Color::Black => -8,
    }
}

const fn castling_rook_from_square(flank: Flank, color: Color) -> Square {
    let rank = Rank::One.from_perspective(color);
    match flank {
        Flank::Kingside => Square::from_rank_and_file(rank, File::H),
        Flank::Queenside => Square::from_rank_and_file(rank, File::A),
    }
}

const fn castling_rook_to_square(flank: Flank, color: Color) -> Square {
    let rank = Rank::One.from_perspective(color);
    match flank {
        Flank::Kingside => Square::from_rank_and_file(rank, File::F),
        Flank::Queenside => Square::from_rank_and_file(rank, File::D),
    }
}
