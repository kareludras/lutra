use crate::eval::evaluate;
use lutra_movegen::{Board, Move, generate_legal_moves, is_in_check};
use std::time::{Duration, Instant};

/// Score assigned to being checkmated at the root (ply 0). Actual mate
/// scores are offset by ply so that faster mates score better/worse than
/// slower ones, letting the search prefer quicker wins and slower losses.
pub const MATE_VALUE: i32 = 1_000_000;
const INFINITY: i32 = MATE_VALUE + 1;

/// Hard safety cap on quiescence recursion depth. A long forced-check
/// sequence (the in-check branch must search every legal response, not
/// just captures) has no natural bound and can recurse extremely deep in
/// complex real-game positions. This caps worst-case recursion regardless
/// of position complexity.
const MAX_PLY: u32 = 100;

/// Absolute wall-clock ceiling for any single `iterative_deepening` call,
/// applied on top of `limits.move_time` and also to depth-only searches,
/// which otherwise have no time bound at all. Defense-in-depth against a
/// GUI declaring the engine unresponsive.
pub const ABSOLUTE_MAX_SEARCH_TIME: Duration = Duration::from_secs(5);

/// The deadline is checked once every `CLOCK_CHECK_INTERVAL` nodes; a power
/// of two so the check is a cheap mask. Small enough that even a slow node
/// rate overshoots the deadline by well under a millisecond.
const CLOCK_CHECK_INTERVAL: u64 = 256;

/// A score this close to `MATE_VALUE` is treated as a confirmed forced
/// mate, at which point deepening further cannot improve the result.
const MATE_THRESHOLD: i32 = MATE_VALUE - 1000;

/// Stopping conditions for iterative deepening: a hard depth cap, and an
/// optional wall-clock budget enforced throughout the search.
#[derive(Debug, Clone, Copy)]
pub struct SearchLimits {
    pub max_depth: u32,
    pub move_time: Option<Duration>,
}

impl SearchLimits {
    pub fn depth(max_depth: u32) -> Self {
        SearchLimits {
            max_depth,
            move_time: None,
        }
    }

    pub fn time(move_time: Duration, max_depth: u32) -> Self {
        SearchLimits {
            max_depth,
            move_time: Some(move_time),
        }
    }
}

/// Result of one completed iterative-deepening pass.
#[derive(Debug, Clone, Copy)]
pub struct SearchResult {
    pub best_move: Move,
    pub score: i32,
    pub depth: u32,
}

/// Per-search state: the deadline, and the hashes of every position on the
/// path from the last irreversible game move down to the current node,
/// used for repetition detection. Once `stopped` is set, every search
/// function unwinds immediately and the in-progress iteration's scores are
/// garbage that must be discarded.
struct Searcher {
    deadline: Option<Instant>,
    nodes: u64,
    stopped: bool,
    path: Vec<u64>,
}

impl Searcher {
    fn new(deadline: Option<Instant>, game_history: &[u64]) -> Self {
        Searcher {
            deadline,
            nodes: 0,
            stopped: false,
            path: game_history.to_vec(),
        }
    }

    fn unbounded() -> Self {
        Searcher::new(None, &[])
    }

    fn should_stop(&mut self) -> bool {
        if self.stopped {
            return true;
        }
        self.nodes += 1;
        if self.nodes & (CLOCK_CHECK_INTERVAL - 1) == 0
            && let Some(deadline) = self.deadline
            && Instant::now() >= deadline
        {
            self.stopped = true;
        }
        self.stopped
    }

    /// True if `board` already occurred earlier on the path (game history
    /// plus the current search line). Only positions since the last capture
    /// or pawn move (`halfmove_clock`) can match, and only those with the
    /// same side to move, so the scan steps back two plies at a time.
    /// A single earlier occurrence is scored as a draw: if repeating is good
    /// enough for one side once, it will be again.
    fn is_repetition(&self, board: &Board, hash: u64) -> bool {
        let lookback = (board.halfmove_clock() as usize).min(self.path.len());
        self.path
            .iter()
            .rev()
            .take(lookback)
            .skip(1)
            .step_by(2)
            .any(|&h| h == hash)
    }

    /// Extends search past the depth cutoff by resolving captures (and, if
    /// in check, all responses) until the position is quiet. This avoids
    /// the horizon effect, where a plain depth-limited search might stop
    /// right before an obviously bad trade completes.
    ///
    /// Repetitions aren't checked here: captures are irreversible, so only
    /// check-evasion sequences could repeat, and those are bounded by
    /// `MAX_PLY`.
    fn quiescence(&mut self, board: &Board, mut alpha: i32, beta: i32, ply: u32) -> i32 {
        if self.should_stop() {
            return 0;
        }
        if ply >= MAX_PLY {
            return evaluate(board);
        }

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

        // Can't "stand pat" while in check: the position isn't quiet until
        // the check is resolved, so every legal response must be searched.
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
            let score = -self.quiescence(&child, -beta, -alpha, ply + 1);
            if self.stopped {
                return 0;
            }
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
    /// perspective of the side to move at `board` (positive is good for
    /// them).
    fn negamax(&mut self, board: &Board, depth: u32, ply: u32, mut alpha: i32, beta: i32) -> i32 {
        if self.should_stop() {
            return 0;
        }

        let hash = board.hash();
        if ply > 0 && self.is_repetition(board, hash) {
            return 0;
        }

        let color = board.side_to_move();
        let moves = generate_legal_moves(board, color);

        if moves.is_empty() {
            return if is_in_check(board, color) {
                -(MATE_VALUE - ply as i32)
            } else {
                0
            };
        }

        // Fifty-move rule. Checked after the mate test because checkmate on
        // the hundredth half-move still wins.
        if ply > 0 && board.halfmove_clock() >= 100 {
            return 0;
        }

        if depth == 0 {
            return self.quiescence(board, alpha, beta, ply);
        }

        self.path.push(hash);
        let mut best = -INFINITY;
        for mv in moves {
            let child = board.make_move(mv);
            let score = -self.negamax(&child, depth - 1, ply + 1, -beta, -alpha);
            if self.stopped {
                break;
            }
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
        self.path.pop();

        if self.stopped { 0 } else { best }
    }

    /// One root search iteration at a fixed depth.
    fn search_root(&mut self, board: &Board, depth: u32) -> RootOutcome {
        let color = board.side_to_move();
        let moves = generate_legal_moves(board, color);
        if moves.is_empty() {
            return RootOutcome::NoMoves;
        }

        let mut best_move = moves[0];
        let mut best_score = -INFINITY;
        let mut alpha = -INFINITY;
        let beta = INFINITY;

        self.path.push(board.hash());
        for mv in moves {
            let child = board.make_move(mv);
            let score = -self.negamax(&child, depth.saturating_sub(1), 1, -beta, -alpha);
            if self.stopped {
                self.path.pop();
                return RootOutcome::Interrupted(best_move);
            }
            if score > best_score {
                best_score = score;
                best_move = mv;
            }
            if best_score > alpha {
                alpha = best_score;
            }
        }
        self.path.pop();

        RootOutcome::Complete(best_move, best_score)
    }
}

/// Outcome of one root iteration.
enum RootOutcome {
    /// Every root move was searched to the requested depth.
    Complete(Move, i32),
    /// The deadline hit mid-iteration. Carries the best move among the root
    /// moves fully searched before the stop (or the first legal move if
    /// none were), so a move is always available.
    Interrupted(Move),
    /// No legal moves (checkmate or stalemate).
    NoMoves,
}

/// Extends search past the depth cutoff by resolving captures (and, if in
/// check, all responses) until the position is quiet.
pub fn quiescence(board: &Board, alpha: i32, beta: i32, ply: u32) -> i32 {
    Searcher::unbounded().quiescence(board, alpha, beta, ply)
}

/// Negamax search with alpha-beta pruning. Returns a score from the
/// perspective of the side to move at `board` (positive is good for them).
pub fn negamax(board: &Board, depth: u32, ply: u32, alpha: i32, beta: i32) -> i32 {
    Searcher::unbounded().negamax(board, depth, ply, alpha, beta)
}

/// Searches `depth` plies and returns the best move found, if any legal
/// move exists (returns `None` on checkmate or stalemate).
pub fn search_best_move(board: &Board, depth: u32) -> Option<Move> {
    match Searcher::unbounded().search_root(board, depth) {
        RootOutcome::Complete(mv, _) | RootOutcome::Interrupted(mv) => Some(mv),
        RootOutcome::NoMoves => None,
    }
}

/// Searches with increasing depth (1, 2, 3, ...) up to `limits.max_depth`,
/// stopping early if the time budget is exceeded or a forced mate is found.
/// The deadline is enforced inside each iteration, not just between them,
/// and is capped at `ABSOLUTE_MAX_SEARCH_TIME`. Returns the result of the
/// last fully completed iteration; if even depth 1 was interrupted, returns
/// the best move found so far in it (at worst the first legal move).
pub fn iterative_deepening(board: &Board, limits: SearchLimits) -> Option<SearchResult> {
    iterative_deepening_with_history(board, &[], limits)
}

/// Like `iterative_deepening`, but aware of the game so far:
/// `game_history` holds the `Board::hash` of every earlier position in the
/// game, oldest first, excluding `board` itself. Positions repeating one of
/// them are scored as draws, so the engine neither walks into a repetition
/// when winning nor misses one when losing.
pub fn iterative_deepening_with_history(
    board: &Board,
    game_history: &[u64],
    limits: SearchLimits,
) -> Option<SearchResult> {
    let start = Instant::now();
    let budget = limits.move_time.map_or(ABSOLUTE_MAX_SEARCH_TIME, |t| {
        t.min(ABSOLUTE_MAX_SEARCH_TIME)
    });
    let mut searcher = Searcher::new(Some(start + budget), game_history);
    let mut best: Option<SearchResult> = None;

    for depth in 1..=limits.max_depth {
        if depth > 1 && start.elapsed() >= budget {
            break;
        }

        match searcher.search_root(board, depth) {
            RootOutcome::NoMoves => return None,
            RootOutcome::Interrupted(best_move) => {
                if best.is_none() {
                    best = Some(SearchResult {
                        best_move,
                        score: 0,
                        depth: 0,
                    });
                }
                break;
            }
            RootOutcome::Complete(best_move, score) => {
                best = Some(SearchResult {
                    best_move,
                    score,
                    depth,
                });
                if score.abs() >= MATE_THRESHOLD {
                    break;
                }
            }
        }
    }

    best
}
