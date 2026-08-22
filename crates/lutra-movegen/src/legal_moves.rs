use crate::attack_info::is_in_check;
use crate::board::Board;
use crate::chess_move::Move;
use crate::movegen::{
    generate_bishop_moves, generate_castle_moves, generate_king_moves, generate_knight_moves,
    generate_pawn_moves, generate_queen_moves, generate_rook_moves,
};
use crate::square::Color;

/// All legal moves for `color`: every pseudo-legal move across all piece
/// types, minus any that would leave `color`'s own king in check.
pub fn generate_legal_moves(board: &Board, color: Color) -> Vec<Move> {
    let mut pseudo_legal = Vec::new();
    pseudo_legal.extend(generate_knight_moves(board, color));
    pseudo_legal.extend(generate_king_moves(board, color));
    pseudo_legal.extend(generate_bishop_moves(board, color));
    pseudo_legal.extend(generate_rook_moves(board, color));
    pseudo_legal.extend(generate_queen_moves(board, color));
    pseudo_legal.extend(generate_pawn_moves(board, color));
    pseudo_legal.extend(generate_castle_moves(board, color));

    pseudo_legal
        .into_iter()
        .filter(|&mv| {
            let after = board.make_move(mv);
            !is_in_check(&after, color)
        })
        .collect()
}
