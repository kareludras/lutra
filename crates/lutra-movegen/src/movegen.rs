use crate::attacks::{king_attacks, knight_attacks, pawn_attacks};
use crate::bitboard::Bitboard;
use crate::board::Board;
use crate::chess_move::Move;
use crate::piece::Piece;
use crate::sliding::{bishop_attacks, queen_attacks, rook_attacks};
use crate::square::{Color, Square};

/// Converts a set of attacked squares into `Move`s from `from`, tagging
/// captures where the target is occupied by an enemy piece.
fn push_moves_from_targets(
    board: &Board,
    color: Color,
    piece: Piece,
    from: Square,
    targets: Bitboard,
    moves: &mut Vec<Move>,
) {
    let enemy_occupancy = board.occupancy(color.opposite());

    for to in targets {
        let mv = Move::new(from, to, piece);
        if enemy_occupancy.contains(to) {
            let (_, captured) = board.piece_at(to).expect("enemy piece must be here");
            moves.push(mv.with_capture(captured));
        } else {
            moves.push(mv);
        }
    }
}

/// Pseudo-legal knight moves for `color` (does not check whether the move
/// leaves that color's own king in check; that filtering happens later).
pub fn generate_knight_moves(board: &Board, color: Color) -> Vec<Move> {
    let mut moves = Vec::new();
    let own_occupancy = board.occupancy(color);

    for from in board.pieces(color, Piece::Knight) {
        let targets = knight_attacks(from) & !own_occupancy;
        push_moves_from_targets(board, color, Piece::Knight, from, targets, &mut moves);
    }

    moves
}

/// Pseudo-legal king moves for `color`, one step in any direction.
/// Castling is handled separately.
pub fn generate_king_moves(board: &Board, color: Color) -> Vec<Move> {
    let mut moves = Vec::new();
    let own_occupancy = board.occupancy(color);

    for from in board.pieces(color, Piece::King) {
        let targets = king_attacks(from) & !own_occupancy;
        push_moves_from_targets(board, color, Piece::King, from, targets, &mut moves);
    }

    moves
}

/// Pseudo-legal bishop moves for `color`.
pub fn generate_bishop_moves(board: &Board, color: Color) -> Vec<Move> {
    let mut moves = Vec::new();
    let own_occupancy = board.occupancy(color);
    let all_occupancy = board.all_occupancy();

    for from in board.pieces(color, Piece::Bishop) {
        let targets = bishop_attacks(from, all_occupancy) & !own_occupancy;
        push_moves_from_targets(board, color, Piece::Bishop, from, targets, &mut moves);
    }

    moves
}

/// Pseudo-legal rook moves for `color`.
pub fn generate_rook_moves(board: &Board, color: Color) -> Vec<Move> {
    let mut moves = Vec::new();
    let own_occupancy = board.occupancy(color);
    let all_occupancy = board.all_occupancy();

    for from in board.pieces(color, Piece::Rook) {
        let targets = rook_attacks(from, all_occupancy) & !own_occupancy;
        push_moves_from_targets(board, color, Piece::Rook, from, targets, &mut moves);
    }

    moves
}

/// Pseudo-legal queen moves for `color`.
pub fn generate_queen_moves(board: &Board, color: Color) -> Vec<Move> {
    let mut moves = Vec::new();
    let own_occupancy = board.occupancy(color);
    let all_occupancy = board.all_occupancy();

    for from in board.pieces(color, Piece::Queen) {
        let targets = queen_attacks(from, all_occupancy) & !own_occupancy;
        push_moves_from_targets(board, color, Piece::Queen, from, targets, &mut moves);
    }

    moves
}

const PROMOTION_PIECES: [Piece; 4] = [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight];

/// Pseudo-legal pawn pushes, captures, and promotions for `color`, excluding
/// en passant (generated separately once implemented).
pub fn generate_pawn_moves(board: &Board, color: Color) -> Vec<Move> {
    let mut moves = Vec::new();
    let all_occupancy = board.all_occupancy();
    let enemy_occupancy = board.occupancy(color.opposite());

    let (start_rank, promotion_rank, direction): (u8, u8, i8) = match color {
        Color::White => (1, 7, 1),
        Color::Black => (6, 0, -1),
    };

    for from in board.pieces(color, Piece::Pawn) {
        let single_rank = (from.rank() as i8 + direction) as u8;
        let single_target = Square::from_file_rank(from.file(), single_rank);

        if !all_occupancy.contains(single_target) {
            if single_rank == promotion_rank {
                for promo in PROMOTION_PIECES {
                    moves.push(Move::new(from, single_target, Piece::Pawn).with_promotion(promo));
                }
            } else {
                moves.push(Move::new(from, single_target, Piece::Pawn));

                if from.rank() == start_rank {
                    let double_rank = (from.rank() as i8 + 2 * direction) as u8;
                    let double_target = Square::from_file_rank(from.file(), double_rank);
                    if !all_occupancy.contains(double_target) {
                        moves.push(Move::new(from, double_target, Piece::Pawn).as_double_push());
                    }
                }
            }
        }

        let capture_targets = pawn_attacks(color, from) & enemy_occupancy;
        for to in capture_targets {
            let (_, captured) = board.piece_at(to).expect("enemy piece must be here");
            if to.rank() == promotion_rank {
                for promo in PROMOTION_PIECES {
                    moves.push(
                        Move::new(from, to, Piece::Pawn)
                            .with_capture(captured)
                            .with_promotion(promo),
                    );
                }
            } else {
                moves.push(Move::new(from, to, Piece::Pawn).with_capture(captured));
            }
        }
    }

    moves
}
