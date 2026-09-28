use lutra_engine::{BENCH_DEPTH, bench, bench::BENCH_FENS};
use lutra_movegen::Board;

#[test]
fn bench_positions_are_all_valid() {
    for fen in BENCH_FENS {
        assert!(Board::from_fen(fen).is_ok(), "invalid bench FEN: {fen}");
    }
}

#[test]
fn bench_node_count_is_deterministic() {
    let first = bench(3);
    let second = bench(3);
    assert!(first.nodes > 0);
    assert_eq!(first.nodes, second.nodes);
}

#[test]
fn default_bench_depth_is_searchable() {
    // Guard against raising BENCH_DEPTH to something that takes minutes.
    assert!((1..=10).contains(&BENCH_DEPTH));
}
