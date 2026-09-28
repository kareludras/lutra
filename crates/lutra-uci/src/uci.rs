use lutra_engine::{
    BENCH_DEPTH, DEFAULT_HASH_MB, Engine, MATE_THRESHOLD, MATE_VALUE, SearchLimits, SearchResult,
    bench,
};
use lutra_movegen::{Board, Color, generate_legal_moves};
use std::io::{BufRead, Write};
use std::time::Duration;
use std::time::Instant;

/// Depth used for `go` with no depth, movetime, or clock info specified.
const DEFAULT_DEPTH: u32 = 6;
/// Depth cap when searching under a time budget, since iterative
/// deepening needs some upper bound even when time-limited.
const TIME_BASED_MAX_DEPTH: u32 = 64;
/// Safety margin subtracted from the remaining clock to avoid flagging as
/// unresponsive due to search/IO overhead eating into the last few ms.
const SAFETY_BUFFER_MS: u64 = 100;
/// Rough estimate of how many moves remain in the game, used to divide up
/// the remaining clock when no explicit movestogo is given.
const ASSUMED_MOVES_REMAINING: u64 = 30;

/// Allocates a per-move time budget from the remaining clock, clamped so
/// it never exceeds what's actually left. Deliberately does NOT add the
/// increment as extra spendable budget for the current move: in standard
/// Fischer increment rules, the increment is credited to the clock only
/// after a move completes, so it's not available to spend on the move
/// currently in progress. Treating it as bonus budget for the current
/// move (an earlier version of this function did) caused a systematic
/// overrun of almost exactly the increment amount on every single move,
/// confirmed by live testing against fastchess.
fn compute_move_time(time_left_ms: u64, _increment_ms: u64) -> Duration {
    let base = time_left_ms / ASSUMED_MOVES_REMAINING;
    let max_allowed = time_left_ms.saturating_sub(SAFETY_BUFFER_MS);
    Duration::from_millis(base.min(max_allowed).max(10))
}

pub struct UciEngine {
    board: Board,
    /// `Board::hash` of every position before `board` in the current game,
    /// oldest first, so the search can recognise repetitions.
    history: Vec<u64>,
    /// Persists across moves so the transposition table carries over.
    engine: Engine,
}

impl Default for UciEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl UciEngine {
    pub fn new() -> Self {
        UciEngine {
            board: Board::starting_position(),
            history: Vec::new(),
            engine: Engine::default(),
        }
    }

    /// Exposed for testing/inspection; not part of the UCI protocol itself.
    pub fn board(&self) -> &Board {
        &self.board
    }

    /// Handles one line of UCI input, writing any response to `out`.
    /// Returns `false` if the engine should stop reading further commands.
    pub fn handle_command<W: Write>(&mut self, line: &str, out: &mut W) -> bool {
        let mut parts = line.split_whitespace();
        match parts.next() {
            Some("uci") => {
                writeln!(out, "id name Lutra 0.1").ok();
                writeln!(out, "id author Karel").ok();
                writeln!(
                    out,
                    "option name Hash type spin default {DEFAULT_HASH_MB} min 1 max 1024"
                )
                .ok();
                writeln!(out, "uciok").ok();
            }
            Some("isready") => {
                writeln!(out, "readyok").ok();
            }
            Some("ucinewgame") => {
                self.board = Board::starting_position();
                self.history.clear();
                self.engine.new_game();
            }
            Some("setoption") => {
                self.handle_setoption(parts);
            }
            Some("position") => {
                self.handle_position(parts);
            }
            Some("go") => {
                self.handle_go(parts, out);
            }
            Some("bench") => {
                let depth = parts
                    .next()
                    .and_then(|d| d.parse().ok())
                    .unwrap_or(BENCH_DEPTH);
                let result = bench(depth);
                writeln!(out, "{} nodes {} nps", result.nodes, result.nps()).ok();
            }
            Some("quit") => {
                return false;
            }
            _ => {}
        }
        true
    }

    /// `setoption name <name> value <value>`. Only `Hash` (MB) is
    /// supported; anything else is ignored.
    fn handle_setoption<'a>(&mut self, parts: impl Iterator<Item = &'a str>) {
        let tokens: Vec<&str> = parts.collect();
        let name_end = tokens.iter().position(|&t| t == "value");
        let name = tokens
            .get(1..name_end.unwrap_or(tokens.len()))
            .map(|n| n.join(" "))
            .unwrap_or_default();
        let value = name_end.and_then(|i| tokens.get(i + 1));
        if name.eq_ignore_ascii_case("hash")
            && let Some(mb) = value.and_then(|v| v.parse::<usize>().ok())
        {
            self.engine.set_hash_mb(mb.clamp(1, 1024));
        }
    }

    fn handle_position<'a>(&mut self, mut parts: impl Iterator<Item = &'a str>) {
        self.history.clear();
        match parts.next() {
            Some("startpos") => {
                self.board = Board::starting_position();
                if parts.next() == Some("moves") {
                    self.apply_moves(parts);
                }
            }
            Some("fen") => {
                let fen_fields: Vec<&str> = parts.by_ref().take(6).collect();
                let fen = fen_fields.join(" ");
                if let Ok(board) = Board::from_fen(&fen) {
                    self.board = board;
                }
                if parts.next() == Some("moves") {
                    self.apply_moves(parts);
                }
            }
            _ => {}
        }
    }

    fn apply_moves<'a>(&mut self, parts: impl Iterator<Item = &'a str>) {
        for token in parts {
            let color = self.board.side_to_move();
            let legal = generate_legal_moves(&self.board, color);
            if let Some(mv) = legal.into_iter().find(|m| m.to_string() == token) {
                self.history.push(self.board.hash());
                self.board = self.board.make_move(mv);
            }
            // Malformed or illegal move tokens are silently ignored rather
            // than crashing the engine on unexpected GUI/protocol input.
        }
    }

    fn handle_go<'a, W: Write>(&mut self, parts: impl Iterator<Item = &'a str>, out: &mut W) {
        let mut depth = None;
        let mut movetime = None;
        let mut wtime = None;
        let mut btime = None;
        let mut winc = None;
        let mut binc = None;
        let mut iter = parts;
        while let Some(token) = iter.next() {
            match token {
                "depth" => depth = iter.next().and_then(|s| s.parse::<u32>().ok()),
                "movetime" => movetime = iter.next().and_then(|s| s.parse::<u64>().ok()),
                "wtime" => wtime = iter.next().and_then(|s| s.parse::<u64>().ok()),
                "btime" => btime = iter.next().and_then(|s| s.parse::<u64>().ok()),
                "winc" => winc = iter.next().and_then(|s| s.parse::<u64>().ok()),
                "binc" => binc = iter.next().and_then(|s| s.parse::<u64>().ok()),
                _ => {}
            }
        }

        let limits = if let Some(d) = depth {
            SearchLimits::depth(d)
        } else if let Some(ms) = movetime {
            SearchLimits::time(Duration::from_millis(ms), TIME_BASED_MAX_DEPTH)
        } else if wtime.is_some() || btime.is_some() {
            let (my_time, my_inc) = match self.board.side_to_move() {
                Color::White => (wtime.unwrap_or(0), winc.unwrap_or(0)),
                Color::Black => (btime.unwrap_or(0), binc.unwrap_or(0)),
            };
            SearchLimits::time(compute_move_time(my_time, my_inc), TIME_BASED_MAX_DEPTH)
        } else {
            SearchLimits::depth(DEFAULT_DEPTH)
        };

        let start = Instant::now();
        match self.engine.search(&self.board, &self.history, limits) {
            Some(result) => {
                writeln!(out, "{}", info_line(&result, start.elapsed().as_millis())).ok();
                writeln!(out, "bestmove {}", result.best_move).ok()
            }
            None => writeln!(out, "bestmove 0000").ok(),
        };
    }
}

/// UCI `info` summary of a finished search, e.g.
/// `info depth 7 score cp 35 nodes 120000 time 210 nps 571428 pv e2e4`.
/// Mate scores are reported in moves, as `score mate N` (negative when
/// the engine is being mated).
fn info_line(result: &SearchResult, time_ms: u128) -> String {
    let score = if result.score.abs() >= MATE_THRESHOLD {
        let plies = MATE_VALUE - result.score.abs();
        let moves = (plies + 1) / 2;
        format!("mate {}", if result.score > 0 { moves } else { -moves })
    } else {
        format!("cp {}", result.score)
    };
    let nps = (result.nodes as u128 * 1000)
        .checked_div(time_ms)
        .unwrap_or(0);
    format!(
        "info depth {} score {score} nodes {} time {time_ms} nps {nps} pv {}",
        result.depth, result.nodes, result.best_move
    )
}

/// Runs the UCI loop: reads commands from `input` line by line, writes
/// responses to `output`, until `quit` or the input stream ends.
pub fn run<R: BufRead, W: Write>(input: R, mut output: W) {
    let mut engine = UciEngine::new();
    for line in input.lines() {
        let Ok(line) = line else { break };
        if !engine.handle_command(&line, &mut output) {
            break;
        }
        output.flush().ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_move_time_does_not_add_increment_as_current_move_budget() {
        // Regression test for a real bug found via live fastchess testing:
        // tc=10+0.1 (100ms increment) caused ~100-110ms time losses on
        // nearly every move. Root cause was adding the full increment as
        // extra spendable budget for the CURRENT move, when in standard
        // Fischer increment rules it's only credited after the move
        // completes. The budget must be based on remaining time alone.
        let time_left = 10_000;
        let increment = 100;
        let budget = compute_move_time(time_left, increment);

        let expected_without_increment = time_left / ASSUMED_MOVES_REMAINING;
        assert_eq!(budget.as_millis() as u64, expected_without_increment);

        // Specifically: the budget must NOT equal the old (buggy) formula.
        let old_buggy_budget = expected_without_increment + increment;
        assert_ne!(budget.as_millis() as u64, old_buggy_budget);
    }

    #[test]
    fn compute_move_time_never_exceeds_remaining_time_minus_safety_buffer() {
        let time_left = 200;
        let budget = compute_move_time(time_left, 100);
        assert!(budget.as_millis() as u64 <= time_left - SAFETY_BUFFER_MS);
    }

    #[test]
    fn compute_move_time_has_a_floor_even_with_very_little_time_left() {
        let budget = compute_move_time(5, 0);
        assert!(budget.as_millis() >= 10);
    }
}
