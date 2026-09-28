use lutra_engine::Engine;
use lutra_movegen::{Board, generate_legal_moves};
use std::collections::HashSet;
use std::io::Write;

/// Openings deeper than this are mostly decided by the random moves.
const MAX_PLIES: u32 = 16;
/// Positions whose shallow-search score is further from equal than this
/// (centipawns) are dropped: one side has simply blundered material.
const MAX_IMBALANCE_CP: i32 = 60;
const FILTER_DEPTH: u32 = 5;

/// `lutra genbook <count> [plies] [seed]`: prints `count` distinct opening
/// positions as FEN lines (EPD-compatible), each reached by `plies` random
/// legal moves from the start position and roughly balanced according to
/// a short search. Deterministic for a given seed.
pub fn run(args: &[String], out: &mut impl Write) -> Result<(), String> {
    let count: usize =
        parse_arg(args, 0, "count")?.ok_or("usage: lutra genbook <count> [plies] [seed]")?;
    let plies: u32 = parse_arg(args, 1, "plies")?.unwrap_or(8).min(MAX_PLIES);
    let seed: u64 = parse_arg(args, 2, "seed")?.unwrap_or(1);

    let mut rng = XorShift64(seed.max(1));
    let mut engine = Engine::new(1);
    let mut seen = HashSet::new();
    let mut found = 0;

    while found < count {
        let Some(board) = random_opening(&mut rng, plies) else {
            continue;
        };
        if !seen.insert(board.hash()) {
            continue;
        }
        engine.new_game();
        let Some(result) = engine.search_fixed_depth(&board, FILTER_DEPTH) else {
            continue; // game already over
        };
        if result.score.abs() > MAX_IMBALANCE_CP {
            continue;
        }
        writeln!(out, "{}", board.to_fen()).map_err(|e| e.to_string())?;
        found += 1;
    }
    Ok(())
}

fn parse_arg<T: std::str::FromStr>(
    args: &[String],
    i: usize,
    name: &str,
) -> Result<Option<T>, String> {
    args.get(i)
        .map(|a| a.parse().map_err(|_| format!("invalid {name}: {a}")))
        .transpose()
}

fn random_opening(rng: &mut XorShift64, plies: u32) -> Option<Board> {
    let mut board = Board::starting_position();
    for _ in 0..plies {
        let moves = generate_legal_moves(&board, board.side_to_move());
        if moves.is_empty() {
            return None;
        }
        let mv = moves[(rng.next() % moves.len() as u64) as usize];
        board = board.make_move(mv);
    }
    Some(board)
}

struct XorShift64(u64);

impl XorShift64 {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}
