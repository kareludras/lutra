/// Which castling moves are still legally available, as a 4-bit flag set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CastlingRights(u8);

impl CastlingRights {
    pub const WHITE_KINGSIDE: u8 = 0b0001;
    pub const WHITE_QUEENSIDE: u8 = 0b0010;
    pub const BLACK_KINGSIDE: u8 = 0b0100;
    pub const BLACK_QUEENSIDE: u8 = 0b1000;

    pub const NONE: CastlingRights = CastlingRights(0);
    pub const ALL: CastlingRights = CastlingRights(
        Self::WHITE_KINGSIDE | Self::WHITE_QUEENSIDE | Self::BLACK_KINGSIDE | Self::BLACK_QUEENSIDE,
    );

    #[inline]
    pub const fn has(self, flag: u8) -> bool {
        (self.0 & flag) != 0
    }

    #[inline]
    pub fn set(&mut self, flag: u8) {
        self.0 |= flag;
    }

    #[inline]
    pub fn clear(&mut self, flag: u8) {
        self.0 &= !flag;
    }
}

impl Default for CastlingRights {
    fn default() -> Self {
        CastlingRights::NONE
    }
}