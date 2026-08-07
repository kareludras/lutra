use lutra_movegen::movegen::generate_knight_moves;
use lutra_movegen::{Board, Color, Piece, Square};

#[test]
fn knight_on_empty_board_has_eight_quiet_moves() {
    let mut board = Board::empty();
    let d4 = Square::from_algebraic("d4").unwrap();
    board.put_piece(Color::White, Piece::Knight, d4);

    let moves = generate_knight_moves(&board, Color::White);
    assert_eq!(moves.len(), 8);
    assert!(moves.iter().all(|m| !m.is_capture()));
}

#[test]
fn knight_cannot_move_onto_own_piece() {
    let mut board = Board::empty();
    let b1 = Square::from_algebraic("b1").unwrap();
    let a3 = Square::from_algebraic("a3").unwrap();
    board.put_piece(Color::White, Piece::Knight, b1);
    board.put_piece(Color::White, Piece::Pawn, a3);

    let moves = generate_knight_moves(&board, Color::White);
    assert!(!moves.iter().any(|m| m.to == a3));
}

#[test]
fn knight_can_capture_enemy_piece() {
    let mut board = Board::empty();
    let b1 = Square::from_algebraic("b1").unwrap();
    let a3 = Square::from_algebraic("a3").unwrap();
    board.put_piece(Color::White, Piece::Knight, b1);
    board.put_piece(Color::Black, Piece::Pawn, a3);

    let moves = generate_knight_moves(&board, Color::White);
    let capture = moves
        .iter()
        .find(|m| m.to == a3)
        .expect("capture move should exist");
    assert!(capture.is_capture());
    assert_eq!(capture.captured, Some(Piece::Pawn));
}

#[test]
fn starting_position_knight_moves_are_only_to_rank_three() {
    let board = Board::starting_position();
    let moves = generate_knight_moves(&board, Color::White);

    // Both knights, each with exactly 2 legal-looking squares (a3/c3, f3/h3),
    // since all other knight-reachable squares are occupied by own pawns.
    assert_eq!(moves.len(), 4);
    assert!(moves.iter().all(|m| !m.is_capture()));
    assert!(moves.iter().all(|m| m.to.rank() == 2));
}
