use crate::square::Square;
use std::fmt;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

/// A set of squares represented as a 64-bit mask, one bit per square,
/// using the same LERF ordering as `Square`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bitboard(pub u64);

impl Bitboard {
    pub const EMPTY: Bitboard = Bitboard(0);
    pub const FULL: Bitboard = Bitboard(u64::MAX);

    #[inline]
    pub const fn new(bits: u64) -> Self {
        Bitboard(bits)
    }

    #[inline]
    pub const fn from_square(sq: Square) -> Self {
        Bitboard(1u64 << sq.index())
    }

    #[inline]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub const fn count(self) -> u32 {
        self.0.count_ones()
    }

    #[inline]
    pub const fn contains(self, sq: Square) -> bool {
        (self.0 & (1u64 << sq.index())) != 0
    }

    #[inline]
    pub fn set(&mut self, sq: Square) {
        self.0 |= 1u64 << sq.index();
    }

    #[inline]
    pub fn clear(&mut self, sq: Square) {
        self.0 &= !(1u64 << sq.index());
    }

    #[inline]
    pub fn toggle(&mut self, sq: Square) {
        self.0 ^= 1u64 << sq.index();
    }

    /// Returns the lowest-indexed set square, if any, without modifying `self`.
    #[inline]
    pub const fn lsb(self) -> Option<Square> {
        if self.0 == 0 {
            None
        } else {
            Some(Square::new(self.0.trailing_zeros() as u8))
        }
    }

    /// Pops (removes and returns) the lowest-indexed set square, if any.
    #[inline]
    pub fn pop_lsb(&mut self) -> Option<Square> {
        let sq = self.lsb()?;
        self.0 &= self.0 - 1; // clears the lowest set bit
        Some(sq)
    }
}

/// Iterates over the set squares in ascending index order, consuming the bitboard.
impl Iterator for Bitboard {
    type Item = Square;

    #[inline]
    fn next(&mut self) -> Option<Square> {
        self.pop_lsb()
    }
}

impl BitAnd for Bitboard {
    type Output = Bitboard;
    #[inline]
    fn bitand(self, rhs: Bitboard) -> Bitboard {
        Bitboard(self.0 & rhs.0)
    }
}

impl BitAndAssign for Bitboard {
    #[inline]
    fn bitand_assign(&mut self, rhs: Bitboard) {
        self.0 &= rhs.0;
    }
}

impl BitOr for Bitboard {
    type Output = Bitboard;
    #[inline]
    fn bitor(self, rhs: Bitboard) -> Bitboard {
        Bitboard(self.0 | rhs.0)
    }
}

impl BitOrAssign for Bitboard {
    #[inline]
    fn bitor_assign(&mut self, rhs: Bitboard) {
        self.0 |= rhs.0;
    }
}

impl BitXor for Bitboard {
    type Output = Bitboard;
    #[inline]
    fn bitxor(self, rhs: Bitboard) -> Bitboard {
        Bitboard(self.0 ^ rhs.0)
    }
}

impl BitXorAssign for Bitboard {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Bitboard) {
        self.0 ^= rhs.0;
    }
}

impl Not for Bitboard {
    type Output = Bitboard;
    #[inline]
    fn not(self) -> Bitboard {
        Bitboard(!self.0)
    }
}

/// Prints the bitboard as an 8x8 grid, rank 8 at the top (matches how a board looks on screen).
impl fmt::Display for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rank in (0..8).rev() {
            for file in 0..8 {
                let sq = Square::from_file_rank(file, rank);
                let c = if self.contains(sq) { '1' } else { '.' };
                write!(f, "{c} ")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_board_has_no_bits() {
        assert!(Bitboard::EMPTY.is_empty());
        assert_eq!(Bitboard::EMPTY.count(), 0);
    }

    #[test]
    fn set_and_contains() {
        let mut bb = Bitboard::EMPTY;
        bb.set(Square::A1);
        assert!(bb.contains(Square::A1));
        assert!(!bb.contains(Square::H8));
        assert_eq!(bb.count(), 1);
    }

    #[test]
    fn clear_removes_bit() {
        let e4 = Square::from_file_rank(4, 3);
        let mut bb = Bitboard::from_square(e4);
        bb.clear(e4);
        assert!(bb.is_empty());
    }

    #[test]
    fn pop_lsb_iterates_in_ascending_order() {
        let mut bb = Bitboard::EMPTY;
        bb.set(Square::H8);
        bb.set(Square::A1);
        bb.set(Square::from_file_rank(4, 3)); // e4

        let squares: Vec<Square> = bb.collect();
        assert_eq!(
            squares,
            vec![Square::A1, Square::from_file_rank(4, 3), Square::H8,]
        );
    }

    #[test]
    fn bitwise_ops_combine_as_expected() {
        let a = Bitboard::from_square(Square::A1);
        let b = Bitboard::from_square(Square::H8);
        let union = a | b;
        assert!(union.contains(Square::A1));
        assert!(union.contains(Square::H8));
        assert_eq!(union.count(), 2);

        let intersection = a & b;
        assert!(intersection.is_empty());

        let xor = union ^ a;
        assert_eq!(xor, b);
    }

    #[test]
    fn not_inverts_all_bits() {
        let a = Bitboard::from_square(Square::A1);
        let inverted = !a;
        assert!(!inverted.contains(Square::A1));
        assert!(inverted.contains(Square::H8));
    }
}
