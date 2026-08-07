use crate::attacks::knight_attacks;
use crate::board::Board;
use crate::chess_move::Move;
use crate::piece::Piece;
use crate::square::Color;

/// Pseudo-legal knight moves for `color` (does not check whether the move
/// leaves that color's own king in check; that filtering happens later).
pub fn generate_knight_moves(board: &Board, color: Color) -> Vec<Move> {
    let mut moves = Vec::new();
    let own_occupancy = board.occupancy(color);
    let enemy_occupancy = board.occupancy(color.opposite());

    for from in board.pieces(color, Piece::Knight) {
        let targets = knight_attacks(from) & !own_occupancy;

        for to in targets {
            let mv = Move::new(from, to, Piece::Knight);
            if enemy_occupancy.contains(to) {
                let (_, captured_piece) = board.piece_at(to).expect("enemy piece must be here");
                moves.push(mv.with_capture(captured_piece));
            } else {
                moves.push(mv);
            }
        }
    }

    moves
}
