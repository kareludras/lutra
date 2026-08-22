use crate::board::Board;
use crate::castling::CastlingRights;
use crate::chess_move::Move;
use crate::piece::Piece;
use crate::square::{Color, Square};

impl Board {
    /// Applies `mv` and returns the resulting board. Does not check legality;
    /// the caller is expected to have generated `mv` from this position (or
    /// to check `is_in_check` on the result if legality matters).
    pub fn make_move(&self, mv: Move) -> Board {
        let mut board = self.clone();
        let color = board.side_to_move();
        let opponent = color.opposite();

        board.remove_piece(color, mv.piece, mv.from);

        if mv.is_en_passant {
            let captured_sq = Square::from_file_rank(mv.to.file(), mv.from.rank());
            board.remove_piece(opponent, Piece::Pawn, captured_sq);
        } else if let Some(captured) = mv.captured {
            board.remove_piece(opponent, captured, mv.to);
        }

        let placed_piece = mv.promotion.unwrap_or(mv.piece);
        board.put_piece(color, placed_piece, mv.to);

        if mv.is_castle {
            let (rook_from, rook_to) = castle_rook_squares(mv.to);
            board.remove_piece(color, Piece::Rook, rook_from);
            board.put_piece(color, Piece::Rook, rook_to);
        }

        let mut rights = board.castling_rights();
        update_castling_rights(&mut rights, color, &mv);
        board.set_castling_rights(rights);

        if mv.is_double_push {
            let ep_rank = (mv.from.rank() + mv.to.rank()) / 2;
            board.set_en_passant(Some(Square::from_file_rank(mv.from.file(), ep_rank)));
        } else {
            board.set_en_passant(None);
        }

        if mv.piece == Piece::Pawn || mv.is_capture() {
            board.set_halfmove_clock(0);
        } else {
            board.set_halfmove_clock(board.halfmove_clock() + 1);
        }

        if color == Color::Black {
            board.set_fullmove_number(board.fullmove_number() + 1);
        }

        board.set_side_to_move(opponent);

        board
    }
}

/// Squares kingside/queenside rooks move to when castling, keyed by the
/// king's destination square.
fn castle_rook_squares(king_to: Square) -> (Square, Square) {
    let rank = king_to.rank();
    if king_to.file() == 6 {
        (
            Square::from_file_rank(7, rank),
            Square::from_file_rank(5, rank),
        )
    } else {
        (
            Square::from_file_rank(0, rank),
            Square::from_file_rank(3, rank),
        )
    }
}

/// Clears castling rights invalidated by `mv`: a king moving clears both
/// rights for its color; a rook moving from or being captured on its home
/// square clears that specific right.
fn update_castling_rights(rights: &mut CastlingRights, color: Color, mv: &Move) {
    if mv.piece == Piece::King {
        match color {
            Color::White => {
                rights.clear(CastlingRights::WHITE_KINGSIDE | CastlingRights::WHITE_QUEENSIDE)
            }
            Color::Black => {
                rights.clear(CastlingRights::BLACK_KINGSIDE | CastlingRights::BLACK_QUEENSIDE)
            }
        }
    }

    clear_if_home_square(rights, mv.from, Square::A1, CastlingRights::WHITE_QUEENSIDE);
    clear_if_home_square(rights, mv.from, Square::H1, CastlingRights::WHITE_KINGSIDE);
    clear_if_home_square(rights, mv.from, Square::A8, CastlingRights::BLACK_QUEENSIDE);
    clear_if_home_square(rights, mv.from, Square::H8, CastlingRights::BLACK_KINGSIDE);

    clear_if_home_square(rights, mv.to, Square::A1, CastlingRights::WHITE_QUEENSIDE);
    clear_if_home_square(rights, mv.to, Square::H1, CastlingRights::WHITE_KINGSIDE);
    clear_if_home_square(rights, mv.to, Square::A8, CastlingRights::BLACK_QUEENSIDE);
    clear_if_home_square(rights, mv.to, Square::H8, CastlingRights::BLACK_KINGSIDE);
}

fn clear_if_home_square(rights: &mut CastlingRights, sq: Square, home: Square, flag: u8) {
    if sq == home {
        rights.clear(flag);
    }
}
