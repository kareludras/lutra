use lutra_movegen::{Board, CastlingRights, Color, Move, Piece, Square};

#[test]
fn quiet_pawn_push_moves_piece_and_flips_side_to_move() {
    let mut board = Board::empty();
    let e2 = Square::from_algebraic("e2").unwrap();
    let e3 = Square::from_algebraic("e3").unwrap();
    board.put_piece(Color::White, Piece::Pawn, e2);

    let mv = Move::new(e2, e3, Piece::Pawn);
    let after = board.make_move(mv);

    assert!(!after.pieces(Color::White, Piece::Pawn).contains(e2));
    assert!(after.pieces(Color::White, Piece::Pawn).contains(e3));
    assert_eq!(after.side_to_move(), Color::Black);
}

#[test]
fn double_push_sets_en_passant_square_for_white() {
    let mut board = Board::empty();
    let e2 = Square::from_algebraic("e2").unwrap();
    let e4 = Square::from_algebraic("e4").unwrap();
    board.put_piece(Color::White, Piece::Pawn, e2);

    let mv = Move::new(e2, e4, Piece::Pawn).as_double_push();
    let after = board.make_move(mv);

    assert_eq!(after.en_passant(), Square::from_algebraic("e3"));
}

#[test]
fn double_push_sets_en_passant_square_for_black() {
    let mut board = Board::empty();
    board.set_side_to_move(Color::Black);
    let e7 = Square::from_algebraic("e7").unwrap();
    let e5 = Square::from_algebraic("e5").unwrap();
    board.put_piece(Color::Black, Piece::Pawn, e7);

    let mv = Move::new(e7, e5, Piece::Pawn).as_double_push();
    let after = board.make_move(mv);

    assert_eq!(after.en_passant(), Square::from_algebraic("e6"));
}

#[test]
fn quiet_move_clears_previous_en_passant_square() {
    let mut board = Board::empty();
    board.set_en_passant(Square::from_algebraic("e3"));
    let a2 = Square::from_algebraic("a2").unwrap();
    let a3 = Square::from_algebraic("a3").unwrap();
    board.put_piece(Color::White, Piece::Pawn, a2);

    let mv = Move::new(a2, a3, Piece::Pawn);
    let after = board.make_move(mv);

    assert_eq!(after.en_passant(), None);
}

#[test]
fn capture_removes_the_captured_piece() {
    let mut board = Board::empty();
    let e4 = Square::from_algebraic("e4").unwrap();
    let d5 = Square::from_algebraic("d5").unwrap();
    board.put_piece(Color::White, Piece::Pawn, e4);
    board.put_piece(Color::Black, Piece::Pawn, d5);

    let mv = Move::new(e4, d5, Piece::Pawn).with_capture(Piece::Pawn);
    let after = board.make_move(mv);

    assert!(!after.pieces(Color::Black, Piece::Pawn).contains(d5));
    assert!(after.pieces(Color::White, Piece::Pawn).contains(d5));
}

#[test]
fn en_passant_capture_removes_pawn_beside_destination_not_on_it() {
    let mut board = Board::empty();
    let e5 = Square::from_algebraic("e5").unwrap();
    let d5 = Square::from_algebraic("d5").unwrap();
    let d6 = Square::from_algebraic("d6").unwrap();
    board.put_piece(Color::White, Piece::Pawn, e5);
    board.put_piece(Color::Black, Piece::Pawn, d5);
    board.set_en_passant(Some(d6));

    let mv = Move::new(e5, d6, Piece::Pawn).as_en_passant();
    let after = board.make_move(mv);

    assert!(!after.pieces(Color::Black, Piece::Pawn).contains(d5));
    assert!(after.pieces(Color::White, Piece::Pawn).contains(d6));
}

#[test]
fn promotion_places_the_promoted_piece_not_a_pawn() {
    let mut board = Board::empty();
    let e7 = Square::from_algebraic("e7").unwrap();
    let e8 = Square::from_algebraic("e8").unwrap();
    board.put_piece(Color::White, Piece::Pawn, e7);

    let mv = Move::new(e7, e8, Piece::Pawn).with_promotion(Piece::Queen);
    let after = board.make_move(mv);

    assert!(after.pieces(Color::White, Piece::Queen).contains(e8));
    assert!(!after.pieces(Color::White, Piece::Pawn).contains(e8));
}

#[test]
fn kingside_castle_moves_both_king_and_rook() {
    let mut board = Board::empty();
    let e1 = Square::from_algebraic("e1").unwrap();
    let g1 = Square::from_algebraic("g1").unwrap();
    let f1 = Square::from_algebraic("f1").unwrap();
    board.put_piece(Color::White, Piece::King, e1);
    board.put_piece(Color::White, Piece::Rook, Square::H1);
    board.set_castling_rights(CastlingRights::ALL);

    let mv = Move::new(e1, g1, Piece::King).as_castle();
    let after = board.make_move(mv);

    assert!(after.pieces(Color::White, Piece::King).contains(g1));
    assert!(after.pieces(Color::White, Piece::Rook).contains(f1));
    assert!(!after.pieces(Color::White, Piece::Rook).contains(Square::H1));
}

#[test]
fn queenside_castle_moves_both_king_and_rook() {
    let mut board = Board::empty();
    let e1 = Square::from_algebraic("e1").unwrap();
    let c1 = Square::from_algebraic("c1").unwrap();
    let d1 = Square::from_algebraic("d1").unwrap();
    board.put_piece(Color::White, Piece::King, e1);
    board.put_piece(Color::White, Piece::Rook, Square::A1);
    board.set_castling_rights(CastlingRights::ALL);

    let mv = Move::new(e1, c1, Piece::King).as_castle();
    let after = board.make_move(mv);

    assert!(after.pieces(Color::White, Piece::King).contains(c1));
    assert!(after.pieces(Color::White, Piece::Rook).contains(d1));
    assert!(!after.pieces(Color::White, Piece::Rook).contains(Square::A1));
}

#[test]
fn king_move_clears_both_castling_rights_for_that_color() {
    let mut board = Board::empty();
    let e1 = Square::from_algebraic("e1").unwrap();
    let e2 = Square::from_algebraic("e2").unwrap();
    board.put_piece(Color::White, Piece::King, e1);
    board.set_castling_rights(CastlingRights::ALL);

    let mv = Move::new(e1, e2, Piece::King);
    let after = board.make_move(mv);

    assert!(!after.castling_rights().has(CastlingRights::WHITE_KINGSIDE));
    assert!(!after.castling_rights().has(CastlingRights::WHITE_QUEENSIDE));
    assert!(after.castling_rights().has(CastlingRights::BLACK_KINGSIDE));
    assert!(after.castling_rights().has(CastlingRights::BLACK_QUEENSIDE));
}

#[test]
fn rook_move_from_home_square_clears_only_that_sides_right() {
    let mut board = Board::empty();
    let b1 = Square::from_algebraic("b1").unwrap();
    board.put_piece(Color::White, Piece::Rook, Square::A1);
    board.set_castling_rights(CastlingRights::ALL);

    let mv = Move::new(Square::A1, b1, Piece::Rook);
    let after = board.make_move(mv);

    assert!(!after.castling_rights().has(CastlingRights::WHITE_QUEENSIDE));
    assert!(after.castling_rights().has(CastlingRights::WHITE_KINGSIDE));
}

#[test]
fn capturing_enemy_rook_on_home_square_clears_that_sides_right() {
    let mut board = Board::empty();
    let a8 = Square::A8;
    let a1 = Square::A1;
    board.put_piece(Color::White, Piece::Rook, a1);
    board.put_piece(Color::Black, Piece::Rook, a8);
    board.set_castling_rights(CastlingRights::ALL);

    let mv = Move::new(a1, a8, Piece::Rook).with_capture(Piece::Rook);
    let after = board.make_move(mv);

    assert!(!after.castling_rights().has(CastlingRights::BLACK_QUEENSIDE));
    assert!(after.castling_rights().has(CastlingRights::BLACK_KINGSIDE));
}

#[test]
fn halfmove_clock_resets_on_pawn_move() {
    let mut board = Board::empty();
    board.set_halfmove_clock(10);
    let e2 = Square::from_algebraic("e2").unwrap();
    let e3 = Square::from_algebraic("e3").unwrap();
    board.put_piece(Color::White, Piece::Pawn, e2);

    let after = board.make_move(Move::new(e2, e3, Piece::Pawn));
    assert_eq!(after.halfmove_clock(), 0);
}

#[test]
fn halfmove_clock_resets_on_capture() {
    let mut board = Board::empty();
    board.set_halfmove_clock(10);
    let b1 = Square::from_algebraic("b1").unwrap();
    let a3 = Square::from_algebraic("a3").unwrap();
    board.put_piece(Color::White, Piece::Knight, b1);
    board.put_piece(Color::Black, Piece::Pawn, a3);

    let mv = Move::new(b1, a3, Piece::Knight).with_capture(Piece::Pawn);
    let after = board.make_move(mv);
    assert_eq!(after.halfmove_clock(), 0);
}

#[test]
fn halfmove_clock_increments_on_quiet_non_pawn_move() {
    let mut board = Board::empty();
    board.set_halfmove_clock(3);
    let b1 = Square::from_algebraic("b1").unwrap();
    let c3 = Square::from_algebraic("c3").unwrap();
    board.put_piece(Color::White, Piece::Knight, b1);

    let after = board.make_move(Move::new(b1, c3, Piece::Knight));
    assert_eq!(after.halfmove_clock(), 4);
}

#[test]
fn fullmove_number_increments_after_black_moves_not_white() {
    let mut board = Board::empty();
    let b1 = Square::from_algebraic("b1").unwrap();
    let c3 = Square::from_algebraic("c3").unwrap();
    board.put_piece(Color::White, Piece::Knight, b1);

    let after_white = board.make_move(Move::new(b1, c3, Piece::Knight));
    assert_eq!(after_white.fullmove_number(), 1);

    let mut black_board = Board::empty();
    black_board.set_side_to_move(Color::Black);
    let b8 = Square::from_algebraic("b8").unwrap();
    let c6 = Square::from_algebraic("c6").unwrap();
    black_board.put_piece(Color::Black, Piece::Knight, b8);

    let after_black = black_board.make_move(Move::new(b8, c6, Piece::Knight));
    assert_eq!(after_black.fullmove_number(), 2);
}

#[test]
fn make_move_does_not_mutate_the_original_board() {
    let mut board = Board::empty();
    let e2 = Square::from_algebraic("e2").unwrap();
    let e3 = Square::from_algebraic("e3").unwrap();
    board.put_piece(Color::White, Piece::Pawn, e2);
    let original = board.clone();

    let _after = board.make_move(Move::new(e2, e3, Piece::Pawn));
    assert_eq!(board, original);
}
