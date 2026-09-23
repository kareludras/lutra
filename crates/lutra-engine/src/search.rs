use crate::eval::{evaluate, material_value};
use crate::tt::{Bound, DEFAULT_HASH_MB, TranspositionTable, TtEntry};
use lutra_movegen::{Board, Color, Move, Piece, generate_legal_moves, is_in_check};
use std::time::{Duration, Instant};

/// Score assigned to being checkmated at the root (ply 0). Actual mate
/// scores are offset by ply so that faster mates score better/worse than
/// slower ones, letting the search prefer quicker wins and slower losses.
pub const MATE_VALUE: i32 = 1_000_000;
const INFINITY: i32 = MATE_VALUE + 1;

/// A score this close to `MATE_VALUE` is treated as a confirmed forced
/// mate, at which point deepening further cannot improve the result.
pub const MATE_THRESHOLD: i32 = MATE_VALUE - 1000;

/// Hard cap on search ply. A long forced-check sequence (the in-check
/// branch of quiescence must search every legal response, and check
/// extensions add depth) has no natural bound, so this caps worst-case
/// recursion regardless of position complexity.
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

/// Quiescence delta pruning margin: a capture that can't lift the static
/// eval back to alpha even with this much positional slack is skipped.
const DELTA_MARGIN: i32 = 200;

/// Move-ordering score bands. Higher is searched first.
const ORDER_TT_MOVE: i32 = 2_000_000;
const ORDER_CAPTURE: i32 = 1_000_000;
const ORDER_KILLER_1: i32 = 900_000;
const ORDER_KILLER_2: i32 = 800_000;

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
    /// Nodes visited by the whole search, across all iterations.
    pub nodes: u64,
}

/// Long-lived search state that persists between moves of a game: the
/// transposition table. A UCI front end keeps one `Engine` for the whole
/// session; the free functions below create a small throwaway one.
pub struct Engine {
    tt: TranspositionTable,
}

impl Default for Engine {
    fn default() -> Self {
        Engine::new(DEFAULT_HASH_MB)
    }
}

impl Engine {
    pub fn new(hash_mb: usize) -> Self {
        Engine {
            tt: TranspositionTable::new(hash_mb),
        }
    }

    /// Resizes (and clears) the transposition table.
    pub fn set_hash_mb(&mut self, hash_mb: usize) {
        self.tt = TranspositionTable::new(hash_mb);
    }

    /// Forgets everything learned in the previous game.
    pub fn new_game(&mut self) {
        self.tt.clear();
    }

    /// Searches with increasing depth (1, 2, 3, ...) up to
    /// `limits.max_depth`, stopping early if the time budget is exceeded or
    /// a forced mate is found. The deadline is enforced inside each
    /// iteration, not just between them, and is capped at
    /// `ABSOLUTE_MAX_SEARCH_TIME`. Returns the result of the last fully
    /// completed iteration; if even depth 1 was interrupted, returns the
    /// best move found so far in it (at worst the first legal move).
    ///
    /// `game_history` holds the `Board::hash` of every earlier position in
    /// the game, oldest first, excluding `board` itself. Positions
    /// repeating one of them are scored as draws.
    pub fn search(
        &mut self,
        board: &Board,
        game_history: &[u64],
        limits: SearchLimits,
    ) -> Option<SearchResult> {
        let start = Instant::now();
        let budget = limits.move_time.map_or(ABSOLUTE_MAX_SEARCH_TIME, |t| {
            t.min(ABSOLUTE_MAX_SEARCH_TIME)
        });
        self.run(board, game_history, limits.max_depth, Some((start, budget)))
    }

    /// Searches exactly to `depth` with no time limit or safety ceiling, so
    /// the node count is fully deterministic. For benchmarking, not play.
    pub fn search_fixed_depth(&mut self, board: &Board, depth: u32) -> Option<SearchResult> {
        self.run(board, &[], depth, None)
    }

    fn run(
        &mut self,
        board: &Board,
        game_history: &[u64],
        max_depth: u32,
        time: Option<(Instant, Duration)>,
    ) -> Option<SearchResult> {
        let deadline = time.map(|(start, budget)| start + budget);
        let mut searcher = Searcher::new(&mut self.tt, deadline, game_history);
        let mut best: Option<SearchResult> = None;

        for depth in 1..=max_depth {
            if depth > 1
                && let Some((start, budget)) = time
                && start.elapsed() >= budget
            {
                break;
            }

            match searcher.search_root(board, depth as i32) {
                RootOutcome::NoMoves => return None,
                RootOutcome::Interrupted(best_move) => {
                    if best.is_none() {
                        best = Some(SearchResult {
                            best_move,
                            score: 0,
                            depth: 0,
                            nodes: 0,
                        });
                    }
                    break;
                }
                RootOutcome::Complete(best_move, score) => {
                    best = Some(SearchResult {
                        best_move,
                        score,
                        depth,
                        nodes: 0,
                    });
                    if score.abs() >= MATE_THRESHOLD {
                        break;
                    }
                }
            }
        }

        best.map(|result| SearchResult {
            nodes: searcher.nodes,
            ..result
        })
    }
}

/// Per-search state: the deadline, move-ordering tables, and the hashes of
/// every position on the path from the last irreversible game move down to
/// the current node, used for repetition detection. Once `stopped` is set,
/// every search function unwinds immediately and the in-progress
/// iteration's scores are garbage that must be discarded.
struct Searcher<'a> {
    tt: &'a mut TranspositionTable,
    deadline: Option<Instant>,
    nodes: u64,
    stopped: bool,
    path: Vec<u64>,
    /// Two quiet moves per ply that recently caused a beta cutoff.
    killers: Vec<[Option<Move>; 2]>,
    /// Quiet-move cutoff history, indexed [color][from][to].
    history: Box<[[[i32; 64]; 64]; 2]>,
}

impl<'a> Searcher<'a> {
    fn new(
        tt: &'a mut TranspositionTable,
        deadline: Option<Instant>,
        game_history: &[u64],
    ) -> Self {
        Searcher {
            tt,
            deadline,
            nodes: 0,
            stopped: false,
            path: game_history.to_vec(),
            killers: vec![[None; 2]; MAX_PLY as usize + 1],
            history: Box::new([[[0; 64]; 64]; 2]),
        }
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

    /// Orders `moves` best-first: the transposition-table move, then
    /// captures by most valuable victim / least valuable attacker, then
    /// killer moves, then quiet moves by history score.
    fn order_moves(&self, moves: &mut [Move], tt_move: Option<Move>, ply: u32, color: Color) {
        let killers = self.killers[ply as usize];
        moves.sort_by_cached_key(|&mv| {
            let score = if Some(mv) == tt_move {
                ORDER_TT_MOVE
            } else if mv.is_capture() || mv.promotion.is_some() {
                ORDER_CAPTURE + mvv_lva(mv)
            } else if Some(mv) == killers[0] {
                ORDER_KILLER_1
            } else if Some(mv) == killers[1] {
                ORDER_KILLER_2
            } else {
                self.history[color.index()][mv.from.index() as usize][mv.to.index() as usize]
            };
            -score
        });
    }

    fn record_quiet_cutoff(&mut self, mv: Move, ply: u32, color: Color, depth: i32) {
        let killers = &mut self.killers[ply as usize];
        if killers[0] != Some(mv) {
            killers[1] = killers[0];
            killers[0] = Some(mv);
        }
        let entry =
            &mut self.history[color.index()][mv.from.index() as usize][mv.to.index() as usize];
        // Bounded well below the killer band so history never outranks it.
        *entry = (*entry + depth * depth).min(ORDER_KILLER_2 / 2);
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

        let mut candidates: Vec<Move> = if in_check {
            moves
        } else {
            moves
                .into_iter()
                .filter(|m| m.is_capture() || m.promotion == Some(Piece::Queen))
                .filter(|m| {
                    // Delta pruning: skip captures that can't possibly
                    // raise the score to alpha.
                    m.promotion.is_some() || stand_pat + captured_value(*m) + DELTA_MARGIN > alpha
                })
                .collect()
        };
        candidates.sort_by_cached_key(|&m| -mvv_lva(m));

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

    /// Principal variation search (negamax with alpha-beta pruning) plus
    /// transposition table, null-move pruning, late-move reductions and
    /// check extensions. Returns a score from the perspective of the side
    /// to move at `board` (positive is good for them).
    fn negamax(
        &mut self,
        board: &Board,
        mut depth: i32,
        ply: u32,
        mut alpha: i32,
        beta: i32,
        allow_null: bool,
    ) -> i32 {
        if self.should_stop() {
            return 0;
        }

        let hash = board.hash();
        if ply > 0 && self.is_repetition(board, hash) {
            return 0;
        }

        let color = board.side_to_move();
        let in_check = is_in_check(board, color);
        let mut moves = generate_legal_moves(board, color);

        if moves.is_empty() {
            return if in_check {
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

        if ply >= MAX_PLY {
            return evaluate(board);
        }

        // Check extension: never drop into quiescence while in check, and
        // look one ply further along forcing lines.
        if in_check {
            depth += 1;
        }

        if depth <= 0 {
            return self.quiescence(board, alpha, beta, ply);
        }

        let is_pv = beta - alpha > 1;
        let tt_entry = self.tt.probe(hash);
        if let Some(entry) = tt_entry
            && ply > 0
            && !is_pv
            && entry.depth >= depth
        {
            let score = score_from_tt(entry.score, ply);
            let usable = match entry.bound {
                Bound::Exact => true,
                Bound::Lower => score >= beta,
                Bound::Upper => score <= alpha,
            };
            if usable {
                return score;
            }
        }
        let tt_move = tt_entry.and_then(|e| e.best_move);

        // Null-move pruning: if passing the turn still fails high, a real
        // move almost certainly would too. Unsafe in check and in pawn
        // endgames (zugzwang), so skipped there.
        if allow_null
            && !is_pv
            && !in_check
            && depth >= 3
            && ply > 0
            && beta.abs() < MATE_THRESHOLD
            && has_non_pawn_material(board, color)
            && evaluate(board) >= beta
        {
            let reduction = 2 + depth / 4;
            self.path.push(hash);
            let score = -self.negamax(
                &board.make_null_move(),
                depth - 1 - reduction,
                ply + 1,
                -beta,
                -beta + 1,
                false,
            );
            self.path.pop();
            if self.stopped {
                return 0;
            }
            if score >= beta {
                return beta;
            }
        }

        self.order_moves(&mut moves, tt_move, ply, color);

        let original_alpha = alpha;
        let mut best = -INFINITY;
        let mut best_move = moves[0];
        self.path.push(hash);
        for (i, &mv) in moves.iter().enumerate() {
            let child = board.make_move(mv);
            let quiet = !mv.is_capture() && mv.promotion.is_none();

            let score = if i == 0 {
                -self.negamax(&child, depth - 1, ply + 1, -beta, -alpha, true)
            } else {
                // Late-move reductions: well-ordered moves late in the list
                // rarely matter, so search them shallower first.
                let reduction = if depth >= 3
                    && i >= 3
                    && quiet
                    && !in_check
                    && !is_in_check(&child, child.side_to_move())
                {
                    1 + (i >= 8 && depth >= 6) as i32
                } else {
                    0
                };
                // Null-window probe; widen only if it beats alpha.
                let mut score = -self.negamax(
                    &child,
                    depth - 1 - reduction,
                    ply + 1,
                    -alpha - 1,
                    -alpha,
                    true,
                );
                if score > alpha && reduction > 0 {
                    score = -self.negamax(&child, depth - 1, ply + 1, -alpha - 1, -alpha, true);
                }
                if score > alpha && score < beta {
                    score = -self.negamax(&child, depth - 1, ply + 1, -beta, -alpha, true);
                }
                score
            };

            if self.stopped {
                break;
            }
            if score > best {
                best = score;
                best_move = mv;
            }
            if best > alpha {
                alpha = best;
            }
            if alpha >= beta {
                if quiet {
                    self.record_quiet_cutoff(mv, ply, color, depth);
                }
                break;
            }
        }
        self.path.pop();

        if self.stopped {
            return 0;
        }

        let bound = if best >= beta {
            Bound::Lower
        } else if best > original_alpha {
            Bound::Exact
        } else {
            Bound::Upper
        };
        self.tt.store(TtEntry {
            key: hash,
            best_move: Some(best_move),
            score: score_to_tt(best, ply),
            depth,
            bound,
        });

        best
    }

    /// One root search iteration at a fixed depth.
    fn search_root(&mut self, board: &Board, depth: i32) -> RootOutcome {
        let color = board.side_to_move();
        let mut moves = generate_legal_moves(board, color);
        if moves.is_empty() {
            return RootOutcome::NoMoves;
        }

        let hash = board.hash();
        let tt_move = self.tt.probe(hash).and_then(|e| e.best_move);
        self.order_moves(&mut moves, tt_move, 0, color);

        let mut best_move = moves[0];
        let mut best_score = -INFINITY;
        let mut alpha = -INFINITY;
        let beta = INFINITY;

        self.path.push(hash);
        for (i, &mv) in moves.iter().enumerate() {
            let child = board.make_move(mv);
            let score = if i == 0 {
                -self.negamax(&child, depth - 1, 1, -beta, -alpha, true)
            } else {
                let score = -self.negamax(&child, depth - 1, 1, -alpha - 1, -alpha, true);
                if score > alpha {
                    -self.negamax(&child, depth - 1, 1, -beta, -alpha, true)
                } else {
                    score
                }
            };
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

        self.tt.store(TtEntry {
            key: hash,
            best_move: Some(best_move),
            score: score_to_tt(best_score, 0),
            depth,
            bound: Bound::Exact,
        });

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

fn captured_value(mv: Move) -> i32 {
    if mv.is_en_passant {
        material_value(Piece::Pawn)
    } else {
        mv.captured.map_or(0, material_value)
    }
}

/// Most valuable victim, least valuable attacker: prefer winning big
/// pieces with small ones. Promotions count as capturing the new piece.
fn mvv_lva(mv: Move) -> i32 {
    let victim = captured_value(mv) + mv.promotion.map_or(0, material_value);
    victim * 10 - material_value(mv.piece) / 10
}

fn has_non_pawn_material(board: &Board, color: Color) -> bool {
    [Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen]
        .iter()
        .any(|&p| !board.pieces(color, p).is_empty())
}

/// Mate scores are stored relative to the node rather than the root, so a
/// mate found via one path is still scored correctly when reached via
/// another path at a different ply.
fn score_to_tt(score: i32, ply: u32) -> i32 {
    if score >= MATE_THRESHOLD {
        score + ply as i32
    } else if score <= -MATE_THRESHOLD {
        score - ply as i32
    } else {
        score
    }
}

fn score_from_tt(score: i32, ply: u32) -> i32 {
    if score >= MATE_THRESHOLD {
        score - ply as i32
    } else if score <= -MATE_THRESHOLD {
        score + ply as i32
    } else {
        score
    }
}

/// A throwaway engine for the one-shot free functions below; small so that
/// allocating it per call stays cheap.
fn scratch_engine() -> Engine {
    Engine::new(1)
}

/// Extends search past the depth cutoff by resolving captures (and, if in
/// check, all responses) until the position is quiet.
pub fn quiescence(board: &Board, alpha: i32, beta: i32, ply: u32) -> i32 {
    let mut engine = scratch_engine();
    Searcher::new(&mut engine.tt, None, &[]).quiescence(board, alpha, beta, ply)
}

/// Negamax search with alpha-beta pruning. Returns a score from the
/// perspective of the side to move at `board` (positive is good for them).
pub fn negamax(board: &Board, depth: u32, ply: u32, alpha: i32, beta: i32) -> i32 {
    let mut engine = scratch_engine();
    Searcher::new(&mut engine.tt, None, &[]).negamax(board, depth as i32, ply, alpha, beta, true)
}

/// Searches `depth` plies and returns the best move found, if any legal
/// move exists (returns `None` on checkmate or stalemate).
pub fn search_best_move(board: &Board, depth: u32) -> Option<Move> {
    scratch_engine()
        .search_fixed_depth(board, depth)
        .map(|r| r.best_move)
}

/// One-shot `Engine::search` with no game history and a fresh table.
pub fn iterative_deepening(board: &Board, limits: SearchLimits) -> Option<SearchResult> {
    iterative_deepening_with_history(board, &[], limits)
}

/// One-shot `Engine::search` with a fresh table.
pub fn iterative_deepening_with_history(
    board: &Board,
    game_history: &[u64],
    limits: SearchLimits,
) -> Option<SearchResult> {
    scratch_engine().search(board, game_history, limits)
}

/// Searches exactly to `depth` with no time limit or safety ceiling, so the
/// node count is fully deterministic. For benchmarking, not for play.
pub fn search_fixed_depth(board: &Board, depth: u32) -> Option<SearchResult> {
    scratch_engine().search_fixed_depth(board, depth)
}
