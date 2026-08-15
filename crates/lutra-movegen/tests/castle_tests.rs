use lutra_movegen::movegen::generate_castle_moves;
use lutra_movegen::{Board, CastlingRights, Color, Piece, Square};

fn empty_back_rank_board(rights: CastlingRights) -> Board {
    let mut board = Board::empty();
    board.put_piece(
        Color::White,
        Piece::King,
        Square::from_algebraic("e1").unwrap(),
    );
    board.put_piece(Color::White, Piece::Rook, Square::A1);
    board.put_piece(Color::White, Piece::Rook, Square::H1);
    board.put_piece(
        Color::Black,
        Piece::King,
        Square::from_algebraic("e8").unwrap(),
    );
    board.put_piece(Color::Black, Piece::Rook, Square::A8);
    board.put_piece(Color::Black, Piece::Rook, Square::H8);
    board.set_castling_rights(rights);
    board
}

#[test]
fn starting_position_has_no_castling_moves() {
    let board = Board::starting_position();
    assert!(generate_castle_moves(&board, Color::White).is_empty());
    assert!(generate_castle_moves(&board, Color::Black).is_empty());
}

#[test]
fn clear_path_allows_both_castling_moves() {
    let board = empty_back_rank_board(CastlingRights::ALL);
    let moves = generate_castle_moves(&board, Color::White);
    assert_eq!(moves.len(), 2);
    assert!(moves.iter().all(|m| m.is_castle));

    let g1 = Square::from_algebraic("g1").unwrap();
    let c1 = Square::from_algebraic("c1").unwrap();
    assert!(moves.iter().any(|m| m.to == g1));
    assert!(moves.iter().any(|m| m.to == c1));
}

#[test]
fn missing_rights_prevents_castling() {
    let board = empty_back_rank_board(CastlingRights::NONE);
    assert!(generate_castle_moves(&board, Color::White).is_empty());
}

#[test]
fn piece_blocking_kingside_path_prevents_that_side_only() {
    let mut board = empty_back_rank_board(CastlingRights::ALL);
    board.put_piece(
        Color::White,
        Piece::Bishop,
        Square::from_algebraic("f1").unwrap(),
    );

    let moves = generate_castle_moves(&board, Color::White);
    assert_eq!(moves.len(), 1);
    let c1 = Square::from_algebraic("c1").unwrap();
    assert_eq!(moves[0].to, c1);
}

#[test]
fn piece_blocking_queenside_b_file_prevents_that_side_only() {
    let mut board = empty_back_rank_board(CastlingRights::ALL);
    board.put_piece(
        Color::White,
        Piece::Knight,
        Square::from_algebraic("b1").unwrap(),
    );

    let moves = generate_castle_moves(&board, Color::White);
    assert_eq!(moves.len(), 1);
    let g1 = Square::from_algebraic("g1").unwrap();
    assert_eq!(moves[0].to, g1);
}

#[test]
fn king_in_check_prevents_any_castling() {
    let mut board = empty_back_rank_board(CastlingRights::ALL);
    board.put_piece(
        Color::Black,
        Piece::Rook,
        Square::from_algebraic("e5").unwrap(),
    );

    assert!(generate_castle_moves(&board, Color::White).is_empty());
}

#[test]
fn attacked_transit_square_prevents_kingside_castling() {
    let mut board = empty_back_rank_board(CastlingRights::ALL);
    // Black rook attacks f1, the square the king passes through kingside.
    board.put_piece(
        Color::Black,
        Piece::Rook,
        Square::from_algebraic("f8").unwrap(),
    );

    let moves = generate_castle_moves(&board, Color::White);
    let g1 = Square::from_algebraic("g1").unwrap();
    assert!(!moves.iter().any(|m| m.to == g1));
}

#[test]
fn attacked_destination_square_prevents_queenside_castling() {
    let mut board = empty_back_rank_board(CastlingRights::ALL);
    // Black rook attacks c1, the king's queenside destination.
    board.put_piece(
        Color::Black,
        Piece::Rook,
        Square::from_algebraic("c8").unwrap(),
    );

    let moves = generate_castle_moves(&board, Color::White);
    let c1 = Square::from_algebraic("c1").unwrap();
    assert!(!moves.iter().any(|m| m.to == c1));
}

#[test]
fn black_castling_moves_generate_on_rank_eight() {
    let board = empty_back_rank_board(CastlingRights::ALL);
    let moves = generate_castle_moves(&board, Color::Black);
    assert_eq!(moves.len(), 2);
    assert!(moves.iter().all(|m| m.to.rank() == 7));
}
