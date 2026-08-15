use lutra_movegen::{Board, Color, Piece, Square, is_in_check, is_square_attacked};

#[test]
fn square_attacked_by_pawn() {
    let mut board = Board::empty();
    let d5 = Square::from_algebraic("d5").unwrap();
    board.put_piece(
        Color::White,
        Piece::Pawn,
        Square::from_algebraic("e4").unwrap(),
    );
    assert!(is_square_attacked(&board, d5, Color::White));
}

#[test]
fn square_not_attacked_by_pawn_directly_ahead() {
    let mut board = Board::empty();
    let e5 = Square::from_algebraic("e5").unwrap();
    board.put_piece(
        Color::White,
        Piece::Pawn,
        Square::from_algebraic("e4").unwrap(),
    );
    assert!(!is_square_attacked(&board, e5, Color::White));
}

#[test]
fn square_attacked_by_knight() {
    let mut board = Board::empty();
    let a3 = Square::from_algebraic("a3").unwrap();
    board.put_piece(
        Color::Black,
        Piece::Knight,
        Square::from_algebraic("b1").unwrap(),
    );
    assert!(is_square_attacked(&board, a3, Color::Black));
}

#[test]
fn square_attacked_by_rook_along_open_file() {
    let mut board = Board::empty();
    let a8 = Square::from_algebraic("a8").unwrap();
    board.put_piece(Color::White, Piece::Rook, Square::A1);
    assert!(is_square_attacked(&board, a8, Color::White));
}

#[test]
fn square_not_attacked_by_rook_when_blocked() {
    let mut board = Board::empty();
    let a8 = Square::from_algebraic("a8").unwrap();
    let a4 = Square::from_algebraic("a4").unwrap();
    board.put_piece(Color::White, Piece::Rook, Square::A1);
    board.put_piece(Color::White, Piece::Pawn, a4);
    assert!(!is_square_attacked(&board, a8, Color::White));
}

#[test]
fn square_attacked_by_bishop_diagonal() {
    let mut board = Board::empty();
    let h8 = Square::from_algebraic("h8").unwrap();
    board.put_piece(Color::Black, Piece::Bishop, Square::A1);
    assert!(is_square_attacked(&board, h8, Color::Black));
}

#[test]
fn square_attacked_by_queen_straight_and_diagonal() {
    let mut board = Board::empty();
    let d8 = Square::from_algebraic("d8").unwrap();
    board.put_piece(
        Color::White,
        Piece::Queen,
        Square::from_algebraic("d1").unwrap(),
    );
    assert!(is_square_attacked(&board, d8, Color::White));
}

#[test]
fn square_attacked_by_king_adjacent() {
    let mut board = Board::empty();
    let e2 = Square::from_algebraic("e2").unwrap();
    board.put_piece(
        Color::Black,
        Piece::King,
        Square::from_algebraic("e1").unwrap(),
    );
    assert!(is_square_attacked(&board, e2, Color::Black));
}

#[test]
fn king_in_check_from_rook() {
    let mut board = Board::empty();
    board.put_piece(
        Color::White,
        Piece::King,
        Square::from_algebraic("e1").unwrap(),
    );
    board.put_piece(
        Color::Black,
        Piece::Rook,
        Square::from_algebraic("e8").unwrap(),
    );
    assert!(is_in_check(&board, Color::White));
}

#[test]
fn king_not_in_check_when_unattacked() {
    let board = Board::starting_position();
    assert!(!is_in_check(&board, Color::White));
    assert!(!is_in_check(&board, Color::Black));
}
