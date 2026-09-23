use lutra_engine::{Engine, MATE_VALUE, SearchLimits, iterative_deepening};
use lutra_movegen::Board;

#[test]
fn finds_a_quiet_sacrifice_mate_in_two() {
    // 1. Ra6! bxa6 2. b7#. The key move is a quiet rook sacrifice, which
    // late-move reductions and null-move pruning must not hide.
    let board = Board::from_fen("kbK5/pp6/1P6/8/8/8/8/R7 w - - 0 1").unwrap();
    let result = iterative_deepening(&board, SearchLimits::depth(6)).expect("white has moves");
    assert_eq!(result.best_move.to_string(), "a1a6");
    assert_eq!(result.score, MATE_VALUE - 3);
}

#[test]
fn transposition_table_makes_a_repeat_search_cheaper() {
    let board =
        Board::from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10")
            .unwrap();
    let mut engine = Engine::new(16);
    let first = engine.search_fixed_depth(&board, 5).unwrap();
    let second = engine.search_fixed_depth(&board, 5).unwrap();
    assert!(
        second.nodes < first.nodes / 2,
        "second search used {} nodes vs {} for the first",
        second.nodes,
        first.nodes
    );

    engine.new_game();
    let after_clear = engine.search_fixed_depth(&board, 5).unwrap();
    assert_eq!(after_clear.nodes, first.nodes);
}

#[test]
fn sharp_middlegame_reaches_depth_six_quickly() {
    // Regression guard: before move ordering, quiescence search exploded in
    // capture-heavy positions like this one and depth 2 took over 5s.
    let board =
        Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
            .unwrap();
    let start = std::time::Instant::now();
    let result = iterative_deepening(&board, SearchLimits::depth(6)).unwrap();
    assert_eq!(result.depth, 6);
    assert!(start.elapsed() < std::time::Duration::from_secs(4));
}
