use crate::attacks::{king_attacks, knight_attacks, pawn_attacks};
use crate::board::Board;
use crate::piece::Piece;
use crate::sliding::{bishop_attacks, rook_attacks};
use crate::square::{Color, Square};

/// Whether any piece of color `by` attacks `sq` on the current board.
pub fn is_square_attacked(board: &Board, sq: Square, by: Color) -> bool {
    let occupancy = board.all_occupancy();

    // A pawn of `by` attacks `sq` iff `sq` would attack that pawn's square
    // under the opposite color's attack pattern (the relation is symmetric).
    if (pawn_attacks(by.opposite(), sq) & board.pieces(by, Piece::Pawn)).count() > 0 {
        return true;
    }
    if (knight_attacks(sq) & board.pieces(by, Piece::Knight)).count() > 0 {
        return true;
    }
    if (king_attacks(sq) & board.pieces(by, Piece::King)).count() > 0 {
        return true;
    }

    let diagonal_attackers = board.pieces(by, Piece::Bishop) | board.pieces(by, Piece::Queen);
    if (bishop_attacks(sq, occupancy) & diagonal_attackers).count() > 0 {
        return true;
    }

    let straight_attackers = board.pieces(by, Piece::Rook) | board.pieces(by, Piece::Queen);
    if (rook_attacks(sq, occupancy) & straight_attackers).count() > 0 {
        return true;
    }

    false
}

/// Whether `color`'s king is currently in check.
pub fn is_in_check(board: &Board, color: Color) -> bool {
    let king_square = board
        .pieces(color, Piece::King)
        .lsb()
        .expect("board must have a king for each color");
    is_square_attacked(board, king_square, color.opposite())
}
