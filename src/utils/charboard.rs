use crate::Bitboard;

/// A type alias for a chess board represented as a 2D array of characters.
pub type Charboard = [[char; 8]; 8];

/// Converts a Charboard to a Bitboard.
pub fn cb_to_bb(cb: &Charboard) -> Bitboard {
    let mut bb: Bitboard = 0;
    for i in 0..8 {
        for j in 0..8 {
            if cb[i][j] != ' ' {
                bb |= 1 << (63 - (i * 8 + j));
            }
        }
    }
    bb
}

/// Converts a Bitboard to a Charboard.
pub fn bb_to_cb(mut bb: Bitboard) -> Charboard {
    let mut cb: Charboard = [[' '; 8]; 8];
    for i in 0..8 {
        for j in 0..8 {
            if bb & 1 != 0 {
                cb[7 - i][7 - j] = 'X';
            }
            bb >>= 1;
        }
    }
    cb
}

/// Prints a Bitboard as a chess board.
pub fn print_bb_pretty(bb: Bitboard) {
    print_cb(&bb_to_cb(bb));
}

/// Returns a string representation of a Charboard.
pub fn cb_to_string(cb: &Charboard) -> String {
    let mut res = String::new();
    for i in 0..8u8 {
        res += &*format!("{} ", 8 - i);
        for j in 0..8u8 {
            if cb[i as usize][j as usize] == ' ' {
                res.push('.');
            }
            else {
                res.push(cb[i as usize][j as usize])
            }
            res.push(' ');
        }
        res.push('\n')
    }
    res + "  a b c d e f g h"
}

/// Prints a Charboard.
pub fn print_cb(cb: &Charboard) {
    println!("{}", cb_to_string(cb));
}
