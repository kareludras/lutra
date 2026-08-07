use crate::piece::Piece;
use crate::square::Square;
use std::fmt;

/// A single move: which piece moved where, plus any capture, promotion, or
/// special-move flags needed later for make/unmake and legality checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub piece: Piece,
    pub captured: Option<Piece>,
    pub promotion: Option<Piece>,
    pub is_en_passant: bool,
    pub is_castle: bool,
    pub is_double_push: bool,
}

impl Move {
    pub fn new(from: Square, to: Square, piece: Piece) -> Self {
        Move {
            from,
            to,
            piece,
            captured: None,
            promotion: None,
            is_en_passant: false,
            is_castle: false,
            is_double_push: false,
        }
    }

    pub fn with_capture(mut self, captured: Piece) -> Self {
        self.captured = Some(captured);
        self
    }

    pub fn with_promotion(mut self, promotion: Piece) -> Self {
        self.promotion = Some(promotion);
        self
    }

    pub fn as_en_passant(mut self) -> Self {
        self.is_en_passant = true;
        self
    }

    pub fn as_castle(mut self) -> Self {
        self.is_castle = true;
        self
    }

    pub fn as_double_push(mut self) -> Self {
        self.is_double_push = true;
        self
    }

    pub fn is_capture(self) -> bool {
        self.captured.is_some() || self.is_en_passant
    }
}

/// UCI long algebraic notation: "e2e4", or "e7e8q" for a promotion.
impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.from, self.to)?;
        if let Some(promo) = self.promotion {
            write!(f, "{}", promo.to_char().to_ascii_lowercase())?;
        }
        Ok(())
    }
}
