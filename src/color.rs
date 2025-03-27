#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
/// Represents a side of the board.
pub enum Color {
    White=0, Black=1
}

impl Color {
    /// Returns a Color from a boolean value, where true is black and false is white.
    pub const fn from_is_black(is_black: bool) -> Color {
        unsafe { std::mem::transmute::<bool, Color>(is_black) }
    }

    /// Returns the other color.
    pub const fn other(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    /// An array of all Colors (2 colors).
    pub const ALL: [Color; 2] = [Color::White, Color::Black];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color() {
        assert_eq!(Color::White as u8, 0);
        assert_eq!(Color::Black as u8, 1);
        assert_eq!(Color::White.other(), Color::Black);
        assert_eq!(Color::Black.other(), Color::White);
        assert_eq!(Color::from_is_black(false), Color::White);
        assert_eq!(Color::from_is_black(true), Color::Black);
    }
}