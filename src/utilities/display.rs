use crate::Bitboard;

pub trait BitboardDisplay {
    /// Prints the Bitboard as a binary number.
    fn print(&self);

    /// Converts the Bitboard to a Charboard.
    fn to_cb(self) -> Charboard;
    
    /// Pretty prints the Bitboard as a chess board.
    fn print_pretty(&self);
}

impl BitboardDisplay for Bitboard {
    fn print(&self) {
        for i in 0..8 {
            let shift_amt = 8 * (7 - i);
            println!("{:08b}", (self & (0xFF << shift_amt)) >> shift_amt);
        }
    }

    fn to_cb(mut self) -> Charboard {
        let mut cb: Charboard = [[' '; 8]; 8];
        for i in 0..8 {
            for j in 0..8 {
                if self & 1 != 0 {
                    cb[7 - i][7 - j] = 'X';
                }
                self >>= 1;
            }
        }
        cb
    }
    
    fn print_pretty(&self) {
        self.to_cb().print();
    }
}

/// A type alias for a chess board represented as a 2D array of characters.
pub type Charboard = [[char; 8]; 8];

pub trait CharboardDisplay {
    /// Converts the Charboard to a string representation.
    fn to_string(&self) -> String;
    
    /// Prints the Charboard.
    fn print(&self);
}

impl CharboardDisplay for Charboard {
    fn to_string(&self) -> String {
        let mut res = String::new();
        for i in 0..8u8 {
            res += &*format!("{} ", 8 - i);
            for j in 0..8u8 {
                if self[i as usize][j as usize] == ' ' {
                    res.push('.');
                } else {
                    res.push(self[i as usize][j as usize])
                }
                res.push(' ');
            }
            res.push('\n')
        }
        res + "  a b c d e f g h"
    }

    fn print(&self) {
        println!("{}", self.to_string());
    }
}
