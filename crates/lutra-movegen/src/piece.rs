/// A chess piece type, independent of color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl Piece {
    pub const ALL: [Piece; 6] = [
        Piece::Pawn,
        Piece::Knight,
        Piece::Bishop,
        Piece::Rook,
        Piece::Queen,
        Piece::King,
    ];

    /// Index into the 0..=5 range, useful for indexing into piece-bitboard arrays.
    #[inline]
    pub const fn index(self) -> usize {
        match self {
            Piece::Pawn => 0,
            Piece::Knight => 1,
            Piece::Bishop => 2,
            Piece::Rook => 3,
            Piece::Queen => 4,
            Piece::King => 5,
        }
    }

    /// The standard algebraic notation letter (uppercase), e.g. Knight -> 'N'.
    pub const fn to_char(self) -> char {
        match self {
            Piece::Pawn => 'P',
            Piece::Knight => 'N',
            Piece::Bishop => 'B',
            Piece::Rook => 'R',
            Piece::Queen => 'Q',
            Piece::King => 'K',
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_pieces_have_unique_indices() {
        let mut indices: Vec<usize> = Piece::ALL.iter().map(|p| p.index()).collect();
        indices.sort_unstable();
        assert_eq!(indices, vec![0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn piece_chars_match_standard_notation() {
        assert_eq!(Piece::Pawn.to_char(), 'P');
        assert_eq!(Piece::Knight.to_char(), 'N');
        assert_eq!(Piece::Bishop.to_char(), 'B');
        assert_eq!(Piece::Rook.to_char(), 'R');
        assert_eq!(Piece::Queen.to_char(), 'Q');
        assert_eq!(Piece::King.to_char(), 'K');
    }
}
