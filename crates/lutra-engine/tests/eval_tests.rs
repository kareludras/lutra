use lutra_engine::evaluate;
use lutra_movegen::{Board, Color, Piece, Square};

#[test]
fn starting_position_is_material_balanced() {
    let board = Board::starting_position();
    assert_eq!(evaluate(&board), 0);
}

#[test]
fn missing_queen_evaluates_negative_for_side_to_move() {
    let mut board = Board::starting_position();
    let d1 = Square::from_algebraic("d1").unwrap();
    board.remove_piece(Color::White, Piece::Queen, d1);
    // White to move, down a queen: material eval should be strongly negative.
    assert_eq!(evaluate(&board), -900);
}

#[test]
fn extra_material_evaluates_positive_for_side_to_move() {
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

    assert_eq!(evaluate(&board), 500);
}

#[test]
fn evaluation_is_symmetric_from_black_to_moves_perspective() {
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
    board.put_piece(Color::Black, Piece::Rook, Square::A8);
    board.set_side_to_move(Color::Black);

    // Black to move, up a rook: positive from black's perspective.
    assert_eq!(evaluate(&board), 500);
}

#[test]
fn king_only_position_evaluates_to_zero() {
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
    assert_eq!(evaluate(&board), 0);
}
