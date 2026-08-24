use lutra_engine::{MATE_VALUE, evaluate, quiescence, search_best_move};
use lutra_movegen::{Board, Color, Piece, Square};

#[test]
fn quiescence_resolves_a_free_capture_that_static_eval_misses() {
    let mut board = Board::empty();
    board.set_side_to_move(Color::Black);
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
    board.put_piece(
        Color::White,
        Piece::Queen,
        Square::from_algebraic("d5").unwrap(),
    );
    board.put_piece(
        Color::Black,
        Piece::Pawn,
        Square::from_algebraic("c6").unwrap(),
    );

    let static_eval = evaluate(&board);
    let qsearch_eval = quiescence(&board, -MATE_VALUE - 1, MATE_VALUE + 1, 0);

    // Statically (before the capture), black looks down a queen for nothing.
    assert!(static_eval < 0);
    // Quiescence should find that black can just take the undefended queen,
    // swinging the evaluation strongly in black's favor.
    assert!(qsearch_eval > 0);
}

#[test]
fn quiescence_on_a_quiet_position_matches_static_eval() {
    let board = Board::starting_position();
    let static_eval = evaluate(&board);
    let qsearch_eval = quiescence(&board, -MATE_VALUE - 1, MATE_VALUE + 1, 0);
    assert_eq!(static_eval, qsearch_eval);
}

#[test]
fn quiescence_does_not_stand_pat_while_in_check() {
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
    board.put_piece(
        Color::Black,
        Piece::Rook,
        Square::from_algebraic("e5").unwrap(),
    );
    board.put_piece(Color::White, Piece::Rook, Square::A1);
    // White to move, in check from the rook, with only a king move to escape
    // (no capturing evasion available). Quiescence must still search this.
    let score = quiescence(&board, -MATE_VALUE - 1, MATE_VALUE + 1, 0);
    // Material is balanced and the escape doesn't lose anything, so the
    // score should reflect a roughly even position, not a stand-pat
    // evaluation that ignores the check entirely.
    assert!(score.abs() < 100);
}

#[test]
fn shallow_search_avoids_horizon_blunder_thanks_to_quiescence() {
    // A depth-1 search that, without quiescence, might walk into a position
    // one ply beyond what it can see and misjudge a losing capture.
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
    board.put_piece(
        Color::White,
        Piece::Queen,
        Square::from_algebraic("d1").unwrap(),
    );
    board.put_piece(
        Color::Black,
        Piece::Pawn,
        Square::from_algebraic("e7").unwrap(),
    );
    board.put_piece(
        Color::Black,
        Piece::Knight,
        Square::from_algebraic("c6").unwrap(),
    );

    // Should not choose to move the queen somewhere it can be freely
    // captured next move; with quiescence resolving captures, the search
    // should avoid an immediately losing queen move.
    let mv = search_best_move(&board, 1).expect("a move should be found");
    let after = board.make_move(mv);
    let black_moves = lutra_movegen::generate_legal_moves(&after, Color::Black);
    let queen_still_safe = !black_moves.iter().any(|m| m.captured == Some(Piece::Queen));
    assert!(queen_still_safe);
}
