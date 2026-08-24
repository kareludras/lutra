use crate::eval::evaluate;
use lutra_movegen::{Board, generate_legal_moves, is_in_check};

/// Score assigned to being checkmated at the root (ply 0). Actual mate
/// scores are offset by ply so that faster mates score better/worse than
/// slower ones, letting the search prefer quicker wins and slower losses.
pub const MATE_VALUE: i32 = 1_000_000;
const INFINITY: i32 = MATE_VALUE + 1;

/// Negamax search with alpha-beta pruning. Returns a score from the
/// perspective of the side to move at `board` (positive is good for them).
pub fn negamax(board: &Board, depth: u32, ply: u32, mut alpha: i32, beta: i32) -> i32 {
    let color = board.side_to_move();
    let moves = generate_legal_moves(board, color);

    if moves.is_empty() {
        return if is_in_check(board, color) {
            -(MATE_VALUE - ply as i32)
        } else {
            0
        };
    }

    if depth == 0 {
        return evaluate(board);
    }

    let mut best = -INFINITY;
    for mv in moves {
        let child = board.make_move(mv);
        let score = -negamax(&child, depth - 1, ply + 1, -beta, -alpha);
        if score > best {
            best = score;
        }
        if best > alpha {
            alpha = best;
        }
        if alpha >= beta {
            break;
        }
    }

    best
}

/// Searches `depth` plies and returns the best move found, if any legal
/// move exists (returns `None` on checkmate or stalemate).
pub fn search_best_move(board: &Board, depth: u32) -> Option<lutra_movegen::Move> {
    let color = board.side_to_move();
    let moves = generate_legal_moves(board, color);
    if moves.is_empty() {
        return None;
    }

    let mut best_move = moves[0];
    let mut best_score = -INFINITY;
    let mut alpha = -INFINITY;
    let beta = INFINITY;

    for mv in moves {
        let child = board.make_move(mv);
        let score = -negamax(&child, depth.saturating_sub(1), 1, -beta, -alpha);
        if score > best_score {
            best_score = score;
            best_move = mv;
        }
        if best_score > alpha {
            alpha = best_score;
        }
    }

    Some(best_move)
}
