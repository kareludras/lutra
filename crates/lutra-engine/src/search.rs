use crate::eval::evaluate;
use lutra_movegen::{Board, generate_legal_moves, is_in_check};

/// Score assigned to being checkmated at the root (ply 0). Actual mate
/// scores are offset by ply so that faster mates score better/worse than
/// slower ones, letting the search prefer quicker wins and slower losses.
pub const MATE_VALUE: i32 = 1_000_000;
const INFINITY: i32 = MATE_VALUE + 1;

/// Extends search past the depth cutoff by resolving captures (and, if in
/// check, all responses) until the position is quiet. This avoids the
/// horizon effect, where a plain depth-limited search might stop right
/// before an obviously bad trade completes.
pub fn quiescence(board: &Board, mut alpha: i32, beta: i32, ply: u32) -> i32 {
    let color = board.side_to_move();
    let in_check = is_in_check(board, color);
    let moves = generate_legal_moves(board, color);

    if moves.is_empty() {
        return if in_check {
            -(MATE_VALUE - ply as i32)
        } else {
            0
        };
    }

    let stand_pat = evaluate(board);

    // Can't "stand pat" while in check: the position isn't quiet until the
    // check is resolved, so every legal response must be searched.
    if !in_check {
        if stand_pat >= beta {
            return beta;
        }
        if stand_pat > alpha {
            alpha = stand_pat;
        }
    }

    let candidates: Vec<_> = if in_check {
        moves
    } else {
        moves.into_iter().filter(|m| m.is_capture()).collect()
    };

    for mv in candidates {
        let child = board.make_move(mv);
        let score = -quiescence(&child, -beta, -alpha, ply + 1);
        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }

    alpha
}

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
        return quiescence(board, alpha, beta, ply);
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

/// One root search iteration at a fixed depth: returns the best move and
/// its score, or `None` if there are no legal moves (checkmate/stalemate).
fn search_root(board: &Board, depth: u32) -> Option<(lutra_movegen::Move, i32)> {
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

    Some((best_move, best_score))
}

/// Searches `depth` plies and returns the best move found, if any legal
/// move exists (returns `None` on checkmate or stalemate).
pub fn search_best_move(board: &Board, depth: u32) -> Option<lutra_movegen::Move> {
    search_root(board, depth).map(|(mv, _)| mv)
}

/// Result of one completed iterative-deepening pass.
#[derive(Debug, Clone, Copy)]
pub struct SearchResult {
    pub best_move: lutra_movegen::Move,
    pub score: i32,
    pub depth: u32,
}

/// Stopping conditions for iterative deepening: a hard depth cap, and an
/// optional wall-clock budget checked between iterations.
#[derive(Debug, Clone, Copy)]
pub struct SearchLimits {
    pub max_depth: u32,
    pub move_time: Option<std::time::Duration>,
}

impl SearchLimits {
    pub fn depth(max_depth: u32) -> Self {
        SearchLimits {
            max_depth,
            move_time: None,
        }
    }

    pub fn time(move_time: std::time::Duration, max_depth: u32) -> Self {
        SearchLimits {
            max_depth,
            move_time: Some(move_time),
        }
    }
}

/// A score this close to `MATE_VALUE` is treated as a confirmed forced
/// mate, at which point deepening further cannot improve the result.
const MATE_THRESHOLD: i32 = MATE_VALUE - 1000;

/// Searches with increasing depth (1, 2, 3, ...) up to `limits.max_depth`,
/// stopping early if the time budget is exceeded or a forced mate is found.
/// Returns the result of the last fully completed iteration.
pub fn iterative_deepening(board: &Board, limits: SearchLimits) -> Option<SearchResult> {
    let start = std::time::Instant::now();
    let mut best: Option<SearchResult> = None;

    for depth in 1..=limits.max_depth {
        // Always complete at least the first iteration: an engine must
        // return some legal move if one exists, even under extreme time
        // pressure, so the time check only applies from depth 2 onward.
        if depth > 1
            && let Some(move_time) = limits.move_time
            && start.elapsed() >= move_time
        {
            break;
        }

        let (best_move, score) = search_root(board, depth)?;

        best = Some(SearchResult {
            best_move,
            score,
            depth,
        });

        if score.abs() >= MATE_THRESHOLD {
            break;
        }
    }

    best
}
