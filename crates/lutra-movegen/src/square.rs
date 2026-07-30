/// A square on the board, indexed 0..=63 using little-endian rank-file (LERF)
/// mapping: a1 = 0, b1 = 1, ..., h1 = 7, a2 = 8, ..., h8 = 63.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Square(u8);

impl Square {
    pub const A1: Square = Square(0);
    pub const H1: Square = Square(7);
    pub const A8: Square = Square(56);
    pub const H8: Square = Square(63);

    /// Builds a `Square` from a raw index. Panics if `index > 63`.
    #[inline]
    pub const fn new(index: u8) -> Self {
        debug_assert!(index < 64);
        Square(index)
    }

    /// Builds a `Square` from zero-indexed file (0..=7, a..=h) and rank (0..=7, 1..=8).
    #[inline]
    pub const fn from_file_rank(file: u8, rank: u8) -> Self {
        debug_assert!(file < 8 && rank < 8);
        Square(rank * 8 + file)
    }

    #[inline]
    pub const fn index(self) -> u8 {
        self.0
    }

    #[inline]
    pub const fn file(self) -> u8 {
        self.0 % 8
    }

    #[inline]
    pub const fn rank(self) -> u8 {
        self.0 / 8
    }

    /// Parses algebraic notation like "e4" into a `Square`.
    pub fn from_algebraic(s: &str) -> Option<Self> {
        let bytes = s.as_bytes();
        if bytes.len() != 2 {
            return None;
        }
        let file = bytes[0];
        let rank = bytes[1];
        if !(b'a'..=b'h').contains(&file) || !(b'1'..=b'8').contains(&rank) {
            return None;
        }
        let file = file - b'a';
        let rank = rank - b'1';
        Some(Square::from_file_rank(file, rank))
    }

    /// Renders this square as algebraic notation like "e4".
    pub fn to_algebraic(self) -> String {
        let file_char = (b'a' + self.file()) as char;
        let rank_char = (b'1' + self.rank()) as char;
        format!("{}{}", file_char, rank_char)
    }
}

impl std::fmt::Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_algebraic())
    }
}

/// The side to move or the color of a piece.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    White,
    Black,
}

impl Color {
    #[inline]
    pub const fn opposite(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lerf_indexing_matches_expected_squares() {
        assert_eq!(Square::A1.index(), 0);
        assert_eq!(Square::H1.index(), 7);
        assert_eq!(Square::A8.index(), 56);
        assert_eq!(Square::H8.index(), 63);
    }

    #[test]
    fn file_and_rank_round_trip() {
        let sq = Square::from_file_rank(4, 3); // e4
        assert_eq!(sq.file(), 4);
        assert_eq!(sq.rank(), 3);
    }

    #[test]
    fn algebraic_round_trip() {
        for s in ["a1", "h1", "a8", "h8", "e4", "d5"] {
            let sq = Square::from_algebraic(s).expect("valid algebraic square");
            assert_eq!(sq.to_algebraic(), s);
        }
    }

    #[test]
    fn algebraic_rejects_invalid_input() {
        assert_eq!(Square::from_algebraic("i1"), None);
        assert_eq!(Square::from_algebraic("a9"), None);
        assert_eq!(Square::from_algebraic("a"), None);
        assert_eq!(Square::from_algebraic("a11"), None);
    }

    #[test]
    fn opposite_color_flips() {
        assert_eq!(Color::White.opposite(), Color::Black);
        assert_eq!(Color::Black.opposite(), Color::White);
    }
}
