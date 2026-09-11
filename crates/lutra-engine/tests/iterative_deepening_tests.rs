use lutra_engine::{MATE_VALUE, SearchLimits, iterative_deepening, search_best_move};
use lutra_movegen::{Board, Color, Piece, Square, generate_legal_moves, is_in_check};
use std::time::Duration;

#[test]
fn depth_limited_search_reaches_the_requested_depth() {
    let board = Board::starting_position();
    let result =
        iterative_deepening(&board, SearchLimits::depth(3)).expect("a result should be found");
    assert_eq!(result.depth, 3);
}

#[test]
fn depth_limited_search_agrees_with_search_best_move() {
    let mut board = Board::empty();
    board.put_piece(
        Color::White,
        Piece::King,
        Square::from_algebraic("e1").unwrap(),
    );
    board.put_piece(
        Color::Black,
        Piece::King,
        Square::from_algebraic("e8").unwrap(),
    );
    board.put_piece(Color::White, Piece::Rook, Square::A1);
    board.put_piece(Color::Black, Piece::Queen, Square::A8);

    let result =
        iterative_deepening(&board, SearchLimits::depth(2)).expect("a result should be found");
    let direct = search_best_move(&board, 2).expect("a move should be found");
    assert_eq!(result.best_move, direct);
}

#[test]
fn time_limited_search_stops_before_max_depth() {
    let board = Board::starting_position();
    let limits = SearchLimits::time(Duration::from_nanos(1), 50);
    let result = iterative_deepening(&board, limits).expect("depth 1 always completes");
    // A near-zero time budget should complete the guaranteed first
    // iteration and then stop well before reaching the full depth.
    assert!(result.depth < 50);
}

#[test]
fn zero_time_budget_still_returns_a_legal_move() {
    // Regression test: even an essentially-zero time budget must not
    // prevent the engine from returning a legal move, since a UCI engine
    // can never legitimately respond with "no move" while one exists.
    // Note: with interruptible search checking the clock every N nodes
    // rather than every single node, a very fast/shallow iteration can
    // complete before ever checking an already-expired deadline, so this
    // doesn't guarantee exactly depth 1 - only that it returns quickly
    // and stays well short of the requested max depth.
    let board = Board::starting_position();
    let limits = SearchLimits::time(Duration::from_nanos(1), 50);
    let start = std::time::Instant::now();
    let result = iterative_deepening(&board, limits).expect("must always find a move");
    let elapsed = start.elapsed();

    assert!(result.depth < 50);
    assert!(
        elapsed < Duration::from_secs(1),
        "search took {elapsed:?} under a near-zero time budget"
    );

    let legal_moves = generate_legal_moves(&board, Color::White);
    assert!(legal_moves.contains(&result.best_move));
}

#[test]
fn search_stops_early_once_a_forced_mate_is_found() {
    let mut board = Board::empty();
    board.put_piece(
        Color::White,
        Piece::King,
        Square::from_algebraic("e1").unwrap(),
    );
    board.put_piece(Color::Black, Piece::King, Square::H8);
    board.put_piece(
        Color::Black,
        Piece::Pawn,
        Square::from_algebraic("g7").unwrap(),
    );
    board.put_piece(
        Color::Black,
        Piece::Pawn,
        Square::from_algebraic("h7").unwrap(),
    );
    board.put_piece(
        Color::White,
        Piece::Queen,
        Square::from_algebraic("h5").unwrap(),
    );
    board.put_piece(
        Color::White,
        Piece::Rook,
        Square::from_algebraic("g1").unwrap(),
    );

    let result =
        iterative_deepening(&board, SearchLimits::depth(20)).expect("a result should be found");

    assert!(
        result.depth < 20,
        "search should stop well before max depth once mate is found"
    );
    assert!(result.score.abs() >= MATE_VALUE - 1000);

    let after = board.make_move(result.best_move);
    let opponent_moves = generate_legal_moves(&after, Color::Black);
    assert!(opponent_moves.is_empty() && is_in_check(&after, Color::Black));
}

#[test]
fn no_legal_moves_returns_none() {
    let mut board = Board::empty();
    board.put_piece(Color::White, Piece::King, Square::H1);
    board.put_piece(
        Color::White,
        Piece::Pawn,
        Square::from_algebraic("g2").unwrap(),
    );
    board.put_piece(
        Color::White,
        Piece::Pawn,
        Square::from_algebraic("h2").unwrap(),
    );
    board.put_piece(Color::Black, Piece::Rook, Square::A1);

    let result = iterative_deepening(&board, SearchLimits::depth(5));
    assert!(result.is_none());
}

#[test]
fn search_does_not_grossly_overshoot_a_realistic_time_budget() {
    // Regression test for a real bug found via live fastchess testing: a
    // single deep iteration could take far longer than the requested
    // budget because the old implementation only checked the clock
    // between iterations, not during one. Under a 10s/game time control
    // (roughly the ~333ms per-move budget lutra-uci computes), a single
    // move must not balloon to multiple seconds.
    let board = Board::starting_position();
    let budget = Duration::from_millis(333);
    let limits = SearchLimits::time(budget, 64);

    let start = std::time::Instant::now();
    let result = iterative_deepening(&board, limits).expect("a result should be found");
    let elapsed = start.elapsed();

    let legal_moves = generate_legal_moves(&board, Color::White);
    assert!(legal_moves.contains(&result.best_move));
    // Generous margin over the budget to account for the depth-1 guarantee
    // and the coarse (every-1024-node) deadline check granularity, while
    // still catching gross multi-second overshoots like the one observed.
    assert!(
        elapsed < Duration::from_secs(2),
        "search took {elapsed:?} against a {budget:?} budget"
    );
}

#[test]
fn even_extreme_time_pressure_returns_a_move_with_bounded_overrun() {
    // Regression test for a real bug found via live fastchess self-play:
    // roughly a third of games were lost on time by a very consistent
    // ~100-114ms overrun. Root cause was that depth 1 was unconditionally
    // exempt from the deadline (to always guarantee a move), but depth 1's
    // own real cost isn't free in this unoptimized engine, so once a
    // game's clock ran down late-game, depth 1 alone could legitimately
    // exceed the shrinking per-move budget every time. Depth 1 must now be
    // interruptible like every other depth, falling back to an instant
    // first-legal-move return if even it can't complete in time.
    let board = Board::starting_position();
    let budget = Duration::from_millis(5);
    let limits = SearchLimits::time(budget, 64);

    let start = std::time::Instant::now();
    let result = iterative_deepening(&board, limits).expect("must always find a move");
    let elapsed = start.elapsed();

    let legal_moves = generate_legal_moves(&board, Color::White);
    assert!(legal_moves.contains(&result.best_move));
    // A tiny budget should trigger the near-instant fallback, not a full
    // uninterruptible depth-1 search; bounded generously to absorb
    // reasonable overhead while catching the ~100ms+ class of regression.
    assert!(
        elapsed < Duration::from_millis(50),
        "search took {elapsed:?} against a {budget:?} budget - this is the exact overrun pattern seen in live testing"
    );
}

#[test]
fn depth_only_search_never_exceeds_the_absolute_safety_ceiling() {
    // Regression test for a real bug found via live fastchess testing at
    // -concurrency 1 (ruling out CPU contention): a game stalled entirely
    // ("not responsive"), distinct from the smaller ~100ms overruns fixed
    // separately. SearchLimits::depth() has no move_time at all, so
    // previously such a search had zero wall-clock bound whatsoever - a
    // wide, tactically complex position could in principle take a very
    // long but finite time even with the quiescence ply cap in place,
    // since the ply cap bounds recursion depth, not node count or time.
    // iterative_deepening must now always respect an absolute ceiling
    // regardless of the requested limits.
    let board = Board::starting_position();
    let start = std::time::Instant::now();
    let result =
        iterative_deepening(&board, SearchLimits::depth(200)).expect("a result should be found");
    let elapsed = start.elapsed();

    let legal_moves = generate_legal_moves(&board, Color::White);
    assert!(legal_moves.contains(&result.best_move));
    assert!(
        elapsed < Duration::from_secs(6),
        "search took {elapsed:?}, exceeding the absolute safety ceiling"
    );
}
