use lutra_engine::{SearchLimits, iterative_deepening};
use lutra_movegen::{Board, Color, generate_legal_moves};
use std::io::{BufRead, Write};
use std::time::Duration;

/// Depth used for `go` with no depth, movetime, or clock info specified.
const DEFAULT_DEPTH: u32 = 6;
/// Depth cap when searching under a time budget, since iterative
/// deepening needs some upper bound even when time-limited.
const TIME_BASED_MAX_DEPTH: u32 = 64;
/// Safety margin subtracted from the remaining clock to avoid flagging as
/// unresponsive due to search/IO overhead eating into the last few ms.
const SAFETY_BUFFER_MS: u64 = 50;
/// Rough estimate of how many moves remain in the game, used to divide up
/// the remaining clock when no explicit movestogo is given.
const ASSUMED_MOVES_REMAINING: u64 = 30;

/// Allocates a per-move time budget from the remaining clock and increment,
/// clamped so it never exceeds what's actually left on the clock.
fn compute_move_time(time_left_ms: u64, increment_ms: u64) -> Duration {
    let base = time_left_ms / ASSUMED_MOVES_REMAINING;
    let budget = base + increment_ms;
    let max_allowed = time_left_ms.saturating_sub(SAFETY_BUFFER_MS);
    Duration::from_millis(budget.min(max_allowed).max(10))
}

pub struct UciEngine {
    board: Board,
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
                writeln!(out, "uciok").ok();
            }
            Some("isready") => {
                writeln!(out, "readyok").ok();
            }
            Some("ucinewgame") => {
                self.board = Board::starting_position();
            }
            Some("position") => {
                self.handle_position(parts);
            }
            Some("go") => {
                self.handle_go(parts, out);
            }
            Some("quit") => {
                return false;
            }
            _ => {}
        }
        true
    }

    fn handle_position<'a>(&mut self, mut parts: impl Iterator<Item = &'a str>) {
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

        match iterative_deepening(&self.board, limits) {
            Some(result) => writeln!(out, "bestmove {}", result.best_move).ok(),
            None => writeln!(out, "bestmove 0000").ok(),
        };
    }
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
