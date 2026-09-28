use crate::search::Engine;
use lutra_movegen::Board;
use std::time::{Duration, Instant};

/// Default depth for `bench`. Each position is searched to exactly this
/// depth with no time limit, so the total node count is deterministic.
pub const BENCH_DEPTH: u32 = 7;

/// A fixed mix of openings, middlegames and endgames. Never change this
/// list casually: the bench node count is only comparable between builds
/// that search the same positions.
pub const BENCH_FENS: [&str; 12] = [
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
    "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
    "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
    "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
    "r1bqkb1r/pp3pp1/2nppn2/7p/3NP1PP/2N5/PPP2P2/R1BQKBR1 w Qkq - 0 9",
    "rnb2rk1/ppp2pbp/3p2p1/3Pp2n/2P1P2q/2N1BP2/PP1Q2PP/R3KBNR w KQ - 3 9",
    "2r3k1/pp3ppp/4p3/3p4/3P4/2P1P3/PP3PPP/2R3K1 b - - 0 25",
    "8/5pk1/6p1/7p/3R3P/6P1/5PK1/1r6 w - - 0 40",
    "8/8/4k3/8/2p5/8/B2K4/8 w - - 0 1",
    "6k1/5ppp/8/8/8/8/1Q3PPP/6K1 b - - 0 1",
];

#[derive(Debug, Clone, Copy)]
pub struct BenchResult {
    pub nodes: u64,
    pub elapsed: Duration,
}

impl BenchResult {
    pub fn nps(&self) -> u64 {
        let secs = self.elapsed.as_secs_f64();
        if secs > 0.0 {
            (self.nodes as f64 / secs) as u64
        } else {
            0
        }
    }
}

/// Searches every `BENCH_FENS` position to `depth` and totals the nodes.
/// The node count acts as a signature of the search: a change that is
/// meant to be a pure speedup must leave it unchanged.
pub fn bench(depth: u32) -> BenchResult {
    let start = Instant::now();
    let nodes = BENCH_FENS
        .iter()
        .map(|fen| {
            let board = Board::from_fen(fen).expect("bench FENs are valid");
            // A fresh engine per position keeps each count independent of
            // the order positions are searched in.
            Engine::default()
                .search_fixed_depth(&board, depth)
                .map_or(0, |r| r.nodes)
        })
        .sum();
    BenchResult {
        nodes,
        elapsed: start.elapsed(),
    }
}
