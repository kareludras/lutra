use lutra_engine::{MATE_VALUE, negamax, search_best_move};
use lutra_movegen::{Board, CastlingRights, Color, Piece, Square};

#[test]
fn search_finds_a_free_queen_capture() {
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
    let a1 = Square::A1;
    let a8 = Square::A8;
    board.put_piece(Color::White, Piece::Rook, a1);
    board.put_piece(Color::Black, Piece::Queen, a8);

    let mv = search_best_move(&board, 2).expect("a move should be found");
    assert_eq!(mv.from, a1);
    assert_eq!(mv.to, a8);
}

#[test]
fn negamax_on_checkmate_position_returns_mate_score_at_ply_zero() {
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

    let score = negamax(&board, 3, 0, -MATE_VALUE - 1, MATE_VALUE + 1);
    assert_eq!(score, -MATE_VALUE);
}

#[test]
fn negamax_on_stalemate_position_returns_zero() {
    let mut board = Board::empty();
    board.set_side_to_move(Color::Black);
    board.put_piece(Color::Black, Piece::King, Square::A8);
    board.put_piece(
        Color::White,
        Piece::King,
        Square::from_algebraic("c7").unwrap(),
    );
    board.put_piece(
        Color::White,
        Piece::Queen,
        Square::from_algebraic("b6").unwrap(),
    );

    let score = negamax(&board, 3, 0, -MATE_VALUE - 1, MATE_VALUE + 1);
    assert_eq!(score, 0);
}

#[test]
fn search_prefers_immediate_mate_over_slower_mate() {
    // White can deliver mate in one (Qh5-f7 style back-rank pattern) or play
    // a quieter winning move; the search should choose the immediate mate.
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

    let mv = search_best_move(&board, 3).expect("a move should be found");
    let after = board.make_move(mv);
    let opponent_moves = lutra_movegen::generate_legal_moves(&after, Color::Black);
    assert!(
        opponent_moves.is_empty() && lutra_movegen::is_in_check(&after, Color::Black),
        "search should find an immediate checkmate when one is available"
    );
}

#[test]
fn deeper_search_does_not_blunder_a_free_rook() {
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
        Piece::Knight,
        Square::from_algebraic("b1").unwrap(),
    );
    board.put_piece(
        Color::Black,
        Piece::Rook,
        Square::from_algebraic("a3").unwrap(),
    );
    board.set_castling_rights(CastlingRights::NONE);

    let mv = search_best_move(&board, 2).expect("a move should be found");
    assert_eq!(mv.to, Square::from_algebraic("a3").unwrap());
}
