use crate::bitboard::Bitboard;
use crate::castling::CastlingRights;
use crate::piece::Piece;
use crate::square::{Color, Square};

/// Full board state: piece placement, side to move, castling rights,
/// en passant target, and the move counters needed for FEN/UCI later.
#[derive(Debug, Clone)]
pub struct Board {
    pieces: [[Bitboard; 6]; 2], // indexed [color.index()][piece.index()]
    side_to_move: Color,
    castling_rights: CastlingRights,
    en_passant: Option<Square>,
    halfmove_clock: u16,
    fullmove_number: u16,
}

impl Board {
    /// An empty board: no pieces, white to move, no castling rights.
    pub fn empty() -> Self {
        Board {
            pieces: [[Bitboard::EMPTY; 6]; 2],
            side_to_move: Color::White,
            castling_rights: CastlingRights::NONE,
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    /// The standard chess starting position.
    pub fn starting_position() -> Self {
        let mut board = Board::empty();
        board.castling_rights = CastlingRights::ALL;

        use Piece::*;
        let back_rank = [Rook, Knight, Bishop, Queen, King, Bishop, Knight, Rook];

        for (file, piece) in back_rank.iter().enumerate() {
            board.put_piece(Color::White, *piece, Square::from_file_rank(file as u8, 0));
            board.put_piece(Color::Black, *piece, Square::from_file_rank(file as u8, 7));
        }
        for file in 0..8 {
            board.put_piece(Color::White, Pawn, Square::from_file_rank(file, 1));
            board.put_piece(Color::Black, Pawn, Square::from_file_rank(file, 6));
        }

        board
    }

    #[inline]
    pub fn pieces(&self, color: Color, piece: Piece) -> Bitboard {
        self.pieces[color.index()][piece.index()]
    }

    #[inline]
    pub fn put_piece(&mut self, color: Color, piece: Piece, sq: Square) {
        self.pieces[color.index()][piece.index()].set(sq);
    }

    #[inline]
    pub fn remove_piece(&mut self, color: Color, piece: Piece, sq: Square) {
        self.pieces[color.index()][piece.index()].clear(sq);
    }

    /// All squares occupied by `color`'s pieces.
    pub fn occupancy(&self, color: Color) -> Bitboard {
        self.pieces[color.index()]
            .iter()
            .fold(Bitboard::EMPTY, |acc, &bb| acc | bb)
    }

    /// All occupied squares, either color.
    pub fn all_occupancy(&self) -> Bitboard {
        self.occupancy(Color::White) | self.occupancy(Color::Black)
    }

    #[inline]
    pub fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    #[inline]
    pub fn castling_rights(&self) -> CastlingRights {
        self.castling_rights
    }

    #[inline]
    pub fn en_passant(&self) -> Option<Square> {
        self.en_passant
    }
}