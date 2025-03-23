use std::cell::RefCell;
use std::rc::Rc;
use crate::utils::Color;
use crate::utils::ColoredPiece;
use crate::utils::Square;
use crate::state::{Board, GameContext, GameResult, State};

pub const INITIAL_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Eq, PartialEq, Debug)]
pub enum FenParseError {
    InvalidFieldCount(usize),
    InvalidRankCount(usize),
    InvalidBoardRow(String),
    InvalidSideToMove(String),
    InvalidCastlingRights(String),
    InvalidEnPassantTarget(String),
    InvalidHalfmoveClock(String),
    InvalidFullmoveNumber(String),
    InvalidPosition(String)
}

fn parse_side_to_move(fen_side_to_move: &str) -> Result<Color, FenParseError> {
    match fen_side_to_move {
        "w" => Ok(Color::White),
        "b" => Ok(Color::Black),
        _ => Err(FenParseError::InvalidSideToMove(fen_side_to_move.to_string()))
    }
}

fn parse_castling_rights(fen_castling_rights: &str) -> Result<u8, FenParseError> {
    if fen_castling_rights == "-" {
        Ok(0)
    } else {
        let mut castling_rights = 0;
        for c in fen_castling_rights.chars() {
            match c {
                'K' => castling_rights |= 0b1000,
                'Q' => castling_rights |= 0b0100,
                'k' => castling_rights |= 0b0010,
                'q' => castling_rights |= 0b0001,
                _ => return Err(FenParseError::InvalidCastlingRights(fen_castling_rights.to_string())),
            }
        }
        Ok(castling_rights)
    }
}

fn parse_en_passant_target(fen_en_passant_target: &str) -> Result<i8, FenParseError> {
    if fen_en_passant_target == "-" {
        Ok(-1)
    } else {
        if fen_en_passant_target.len() != 2 {
            return Err(FenParseError::InvalidEnPassantTarget(fen_en_passant_target.to_string()));
        }
        let file = fen_en_passant_target.chars().nth(0).unwrap();
        let rank = fen_en_passant_target.chars().nth(1).unwrap();
        if file < 'a' || file > 'h' || rank < '1' || rank > '8' {
            return Err(FenParseError::InvalidEnPassantTarget(fen_en_passant_target.to_string()));
        }
        Ok(file as i8 - 'a' as i8)
    }
}

fn parse_fen_halfmove_clock(fen_halfmove_clock: &str) -> Result<u8, FenParseError> {
    match fen_halfmove_clock.parse::<u8>() {
        Ok(halfmove_clock) if halfmove_clock <= 100 => Ok(halfmove_clock),
        _ => Err(FenParseError::InvalidHalfmoveClock(fen_halfmove_clock.to_string()))
    }
}

fn parse_fen_fullmove_number(fen_fullmove_number: &str) -> Result<u16, FenParseError> {
    match fen_fullmove_number.parse::<u16>() {
        Ok(fullmove_number) if fullmove_number > 0 => Ok(fullmove_number),
        _ => Err(FenParseError::InvalidFullmoveNumber(fen_fullmove_number.to_string()))
    }
}

fn parse_fen_board_row(row: &str, row_from_top: u8, board: &mut Board) -> Result<(), FenParseError> {
    assert!(row_from_top < 8);

    let mut file = 0;
    for c in row.chars() {
        if c.is_ascii_digit() {
            file += c.to_digit(10).unwrap() as u8;
            if file > 8 {
                return Err(FenParseError::InvalidBoardRow(row.to_string()));
            }
        }
        else if c.is_ascii_alphabetic() {
            match ColoredPiece::from_char(c) {
                ColoredPiece::NoPiece => return Err(FenParseError::InvalidBoardRow(row.to_string())),
                colored_piece => {
                    let dst =  unsafe { Square::from(row_from_top * 8 + file) };
                    board.put_colored_piece_at(colored_piece, dst);

                    file += 1;
                }
            };
        }
        else {
            return Err(FenParseError::InvalidBoardRow(row.to_string()));
        }
    }

    if file == 8 {
        Ok(())
    } else {
        Err(FenParseError::InvalidBoardRow(row.to_string()))
    }
}

fn parse_fen_board(fen_board: &str) -> Result<Board, FenParseError> {
    let fen_board_rows: Vec<&str> = fen_board.split('/').collect();

    let row_count = fen_board_rows.len();
    if row_count != 8 {
        return Err(FenParseError::InvalidRankCount(row_count));
    }

    let mut board = Board::blank();
    for (row_from_top, fen_board_row) in fen_board_rows.into_iter().enumerate() {
        parse_fen_board_row(fen_board_row, row_from_top as u8, &mut board)?;
    }

    Ok(board)
}

impl State {
    pub fn from_fen(fen: &str) -> Result<State, FenParseError> {
        let fen_parts: Vec<&str> = fen.split_ascii_whitespace().collect();
        if fen_parts.len() != 6 {
            return Err(FenParseError::InvalidFieldCount(fen_parts.len()));
        }
        
        match fen_parts[..] {
            [
                fen_board,
                fen_side_to_move,
                fen_castling_rights,
                fen_en_passant_target,
                fen_halfmove_clock,
                fen_fullmove_number
            ] => {
                let side_to_move = parse_side_to_move(fen_side_to_move)?;
                let castling_rights = parse_castling_rights(fen_castling_rights)?;
                let double_pawn_push = parse_en_passant_target(fen_en_passant_target)?;
                let halfmove_clock = parse_fen_halfmove_clock(fen_halfmove_clock)?;
                let fullmove_number = parse_fen_fullmove_number(fen_fullmove_number)?;
                let board = parse_fen_board(fen_board)?;

                let zobrist_hash = board.calc_zobrist_hash();
                let halfmove = (fullmove_number - 1) * 2 + if side_to_move == Color::Black { 1 } else { 0 };
                let mut context = GameContext::new_without_previous(castling_rights, zobrist_hash, board.calc_attacks_mask(side_to_move));
                context.double_pawn_push = double_pawn_push;
                context.halfmove_clock = halfmove_clock;

                let state = State {
                    board,
                    side_to_move,
                    halfmove,
                    result: GameResult::None,
                    context: Rc::new(RefCell::new(context)),
                };

                if state.is_unequivocally_valid() {
                    Ok(state)
                } else {
                    Err(FenParseError::InvalidPosition(fen.to_string()))
                }
            },
            _ => Err(FenParseError::InvalidFieldCount(fen_parts.len())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::State;
    
    #[test]
    fn test_from_fen() {
        let fen = "8/1P1n1B2/5P2/4pkNp/1PQ4K/p2p2P1/8/3R1N2 w - - 0 1";
        let state_result = State::from_fen(fen);
        assert!(state_result.is_ok());
        // assert_eq!(state_result.unwrap().to_fen(), fen);

        let fen = "1k2N1K1/4Q3/6p1/2B2B2/p1PPb3/2P2Nb1/2r5/n7 b - - 36 18";
        let state_result = State::from_fen(fen);
        assert!(state_result.is_err());
        assert_eq!(state_result.err().unwrap(), FenParseError::InvalidPosition(fen.to_string()));

        let fen = "1k2N1K1/4Q3/6p1/2B2B2/p1PPb3/2P2Nb1/2r5/n7 b - - 35 18";
        let state_result = State::from_fen(fen);
        assert!(state_result.is_ok(), "{:?}", state_result);
        // assert_eq!(state_result.unwrap().to_fen(), fen);
        
        let fen = "r3k3/P3P3/1B3q2/N3P2P/R6N/8/np2b2p/1K3n2 w q - 100 96";
        let state_result = State::from_fen(fen);
        assert!(state_result.is_ok());
        // assert_eq!(state_result.unwrap().to_fen(), fen);

        let fen = "nb4K1/2N4p/8/3P1rk1/1r2P3/5p2/3P1Q2/B2R1b2 b - - 0 1";
        let state_result = State::from_fen(fen);
        assert!(state_result.is_ok());
        // assert_eq!(state_result.unwrap().to_fen(), fen);
    }
}