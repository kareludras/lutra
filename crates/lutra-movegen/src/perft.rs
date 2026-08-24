use crate::board::Board;
use crate::chess_move::Move;
use crate::legal_moves::generate_legal_moves;

/// Counts leaf nodes reachable in exactly `depth` plies from `board`. Used to
/// validate move generation correctness against known-good reference counts.
pub fn perft(board: &Board, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }

    let moves = generate_legal_moves(board, board.side_to_move());
    if depth == 1 {
        return moves.len() as u64;
    }

    moves
        .iter()
        .map(|&mv| perft(&board.make_move(mv), depth - 1))
        .sum()
}

/// Per-move node counts at `depth`, used to isolate which move is
/// responsible for an incorrect perft total.
pub fn perft_divide(board: &Board, depth: u32) -> Vec<(Move, u64)> {
    let moves = generate_legal_moves(board, board.side_to_move());

    moves
        .iter()
        .map(|&mv| {
            let count = if depth <= 1 {
                1
            } else {
                perft(&board.make_move(mv), depth - 1)
            };
            (mv, count)
        })
        .collect()
}
