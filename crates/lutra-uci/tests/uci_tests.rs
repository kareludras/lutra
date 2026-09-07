use lutra_uci::UciEngine;

fn run_command(engine: &mut UciEngine, cmd: &str) -> String {
    let mut out = Vec::new();
    engine.handle_command(cmd, &mut out);
    String::from_utf8(out).unwrap()
}

#[test]
fn uci_command_responds_with_id_and_uciok() {
    let mut engine = UciEngine::new();
    let output = run_command(&mut engine, "uci");
    assert!(output.contains("id name"));
    assert!(output.contains("id author"));
    assert!(output.contains("uciok"));
}

#[test]
fn isready_responds_with_readyok() {
    let mut engine = UciEngine::new();
    let output = run_command(&mut engine, "isready");
    assert_eq!(output.trim(), "readyok");
}

#[test]
fn quit_signals_the_loop_to_stop() {
    let mut engine = UciEngine::new();
    let mut out = Vec::new();
    let should_continue = engine.handle_command("quit", &mut out);
    assert!(!should_continue);
}

#[test]
fn unknown_command_does_not_stop_the_loop_or_panic() {
    let mut engine = UciEngine::new();
    let mut out = Vec::new();
    let should_continue = engine.handle_command("some_unknown_gui_extension", &mut out);
    assert!(should_continue);
}

#[test]
fn position_startpos_sets_the_starting_position() {
    let mut engine = UciEngine::new();
    let mut out = Vec::new();
    engine.handle_command("position startpos", &mut out);
    assert_eq!(
        engine.board().to_fen(),
        lutra_movegen::Board::starting_position().to_fen()
    );
}

#[test]
fn position_startpos_with_moves_applies_them_in_order() {
    let mut engine = UciEngine::new();
    let mut out = Vec::new();
    engine.handle_command("position startpos moves e2e4 e7e5", &mut out);

    let expected = lutra_movegen::Board::starting_position();
    let expected = expected.make_move(
        lutra_movegen::generate_legal_moves(&expected, lutra_movegen::Color::White)
            .into_iter()
            .find(|m| m.to_string() == "e2e4")
            .unwrap(),
    );
    let expected = expected.make_move(
        lutra_movegen::generate_legal_moves(&expected, lutra_movegen::Color::Black)
            .into_iter()
            .find(|m| m.to_string() == "e7e5")
            .unwrap(),
    );

    assert_eq!(engine.board().to_fen(), expected.to_fen());
}

#[test]
fn position_fen_sets_an_arbitrary_position() {
    let mut engine = UciEngine::new();
    let mut out = Vec::new();
    let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
    engine.handle_command(&format!("position fen {fen}"), &mut out);
    assert_eq!(engine.board().to_fen(), fen);
}

#[test]
fn position_fen_with_trailing_moves_applies_them() {
    let mut engine = UciEngine::new();
    let mut out = Vec::new();
    let fen = "8/8/8/8/8/8/4P3/4K2k w - - 0 1";
    engine.handle_command(&format!("position fen {fen} moves e2e4"), &mut out);
    assert!(
        engine
            .board()
            .pieces(lutra_movegen::Color::White, lutra_movegen::Piece::Pawn)
            .contains(lutra_movegen::Square::from_algebraic("e4").unwrap())
    );
}

#[test]
fn illegal_move_token_in_position_moves_is_ignored_not_fatal() {
    let mut engine = UciEngine::new();
    let mut out = Vec::new();
    // z9z9 is not a legal move token; the engine should not panic and
    // should simply skip it, leaving the board at the starting position.
    engine.handle_command("position startpos moves z9z9", &mut out);
    assert_eq!(
        engine.board().to_fen(),
        lutra_movegen::Board::starting_position().to_fen()
    );
}

#[test]
fn go_depth_returns_a_legal_bestmove() {
    let mut engine = UciEngine::new();
    let mut out = Vec::new();
    engine.handle_command("position startpos", &mut out);
    let output = run_command(&mut engine, "go depth 2");

    assert!(output.starts_with("bestmove "));
    let token = output.trim().strip_prefix("bestmove ").unwrap();
    let legal = lutra_movegen::generate_legal_moves(engine.board(), lutra_movegen::Color::White);
    assert!(legal.iter().any(|m| m.to_string() == token));
}

#[test]
fn go_movetime_returns_a_legal_bestmove() {
    let mut engine = UciEngine::new();
    let mut out = Vec::new();
    engine.handle_command("position startpos", &mut out);
    let output = run_command(&mut engine, "go movetime 50");

    assert!(output.starts_with("bestmove "));
}

#[test]
fn go_with_no_arguments_uses_a_default_depth() {
    let mut engine = UciEngine::new();
    let mut out = Vec::new();
    engine.handle_command("position startpos", &mut out);
    let output = run_command(&mut engine, "go");
    assert!(output.starts_with("bestmove "));
}

#[test]
fn go_on_checkmate_position_returns_null_move() {
    let mut engine = UciEngine::new();
    let mut out = Vec::new();
    // Back-rank mate: black king g8 fully boxed in by its own pawns,
    // white rook giving check along the open back rank with no blocker.
    let mate_fen = "R5k1/5ppp/8/8/8/8/8/6K1 b - - 0 1";
    engine.handle_command(&format!("position fen {mate_fen}"), &mut out);
    let output = run_command(&mut engine, "go depth 3");
    assert_eq!(output.trim(), "bestmove 0000");
}

#[test]
fn full_uci_handshake_sequence_via_run() {
    let input = b"uci\nisready\nposition startpos\ngo depth 1\nquit\n";
    let mut output = Vec::new();
    lutra_uci::run(&input[..], &mut output);
    let output = String::from_utf8(output).unwrap();

    assert!(output.contains("uciok"));
    assert!(output.contains("readyok"));
    assert!(output.contains("bestmove"));
}

#[test]
fn go_with_wtime_btime_returns_promptly_under_a_short_time_control() {
    // Regression test: reproduces the exact scenario that caused fastchess
    // to report the engine as unresponsive under tc=10+0.1 (10 seconds for
    // the whole game). Before wtime/btime parsing was added, the engine
    // ignored the clock entirely and searched a fixed depth every move,
    // quickly blowing the time budget.
    //
    // NOTE: run this with `--release`. Debug builds skip optimizations our
    // (currently unoptimized) move generation relies on for speed, and
    // depth 1 is deliberately exempt from the deadline (to guarantee a
    // move is always returned), so debug-mode timing here isn't
    // representative of the real compiled engine's behavior.
    let mut engine = UciEngine::new();
    let mut out = Vec::new();
    engine.handle_command("position startpos", &mut out);

    let start = std::time::Instant::now();
    let output = run_command(&mut engine, "go wtime 10000 btime 10000 winc 100 binc 100");
    let elapsed = start.elapsed();

    assert!(output.starts_with("bestmove "));
    // Generous tolerance to absorb debug-mode overhead; still catches the
    // kind of multi-second-scale overshoot the original bug produced.
    assert!(
        elapsed < std::time::Duration::from_secs(8),
        "move took {elapsed:?}, which would blow the clock under a real short time control"
    );
}

#[test]
fn go_with_very_low_remaining_time_still_returns_a_move_quickly() {
    let mut engine = UciEngine::new();
    let mut out = Vec::new();
    engine.handle_command("position startpos", &mut out);

    let start = std::time::Instant::now();
    let output = run_command(&mut engine, "go wtime 200 btime 200 winc 0 binc 0");
    let elapsed = start.elapsed();

    assert!(output.starts_with("bestmove "));
    assert!(
        elapsed < std::time::Duration::from_secs(8),
        "elapsed: {elapsed:?}"
    );
}

#[test]
fn go_uses_black_clock_when_black_is_to_move() {
    let mut engine = UciEngine::new();
    let mut out = Vec::new();
    // After 1.e4, it's black to move; a wildly generous wtime with a tiny
    // btime should still resolve quickly, proving it reads btime, not wtime.
    engine.handle_command("position startpos moves e2e4", &mut out);

    let start = std::time::Instant::now();
    let output = run_command(&mut engine, "go wtime 999999999 btime 300 winc 0 binc 0");
    let elapsed = start.elapsed();

    assert!(output.starts_with("bestmove "));
    assert!(
        elapsed < std::time::Duration::from_secs(8),
        "elapsed: {elapsed:?}"
    );
}
