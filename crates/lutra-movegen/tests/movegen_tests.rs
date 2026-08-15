use lutra_movegen::movegen::{
    generate_bishop_moves, generate_king_moves, generate_knight_moves, generate_pawn_moves,
    generate_queen_moves, generate_rook_moves,
};
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

#[test]
fn king_on_empty_board_has_eight_quiet_moves() {
    let mut board = Board::empty();
    let d4 = Square::from_algebraic("d4").unwrap();
    board.put_piece(Color::White, Piece::King, d4);

    let moves = generate_king_moves(&board, Color::White);
    assert_eq!(moves.len(), 8);
    assert!(moves.iter().all(|m| !m.is_capture()));
}

#[test]
fn king_in_corner_has_three_quiet_moves() {
    let mut board = Board::empty();
    board.put_piece(Color::White, Piece::King, Square::A1);

    let moves = generate_king_moves(&board, Color::White);
    assert_eq!(moves.len(), 3);
}

#[test]
fn king_cannot_move_onto_own_piece() {
    let mut board = Board::empty();
    let e1 = Square::from_algebraic("e1").unwrap();
    let e2 = Square::from_algebraic("e2").unwrap();
    board.put_piece(Color::White, Piece::King, e1);
    board.put_piece(Color::White, Piece::Pawn, e2);

    let moves = generate_king_moves(&board, Color::White);
    assert!(!moves.iter().any(|m| m.to == e2));
    assert_eq!(moves.len(), 4);
}

#[test]
fn king_can_capture_enemy_piece() {
    let mut board = Board::empty();
    let e1 = Square::from_algebraic("e1").unwrap();
    let e2 = Square::from_algebraic("e2").unwrap();
    board.put_piece(Color::White, Piece::King, e1);
    board.put_piece(Color::Black, Piece::Pawn, e2);

    let moves = generate_king_moves(&board, Color::White);
    let capture = moves
        .iter()
        .find(|m| m.to == e2)
        .expect("capture move should exist");
    assert!(capture.is_capture());
    assert_eq!(capture.captured, Some(Piece::Pawn));
}

#[test]
fn starting_position_king_has_no_moves() {
    let board = Board::starting_position();
    let moves = generate_king_moves(&board, Color::White);
    assert!(moves.is_empty());
}

#[test]
fn bishop_on_empty_board_d4_has_thirteen_moves() {
    let mut board = Board::empty();
    let d4 = Square::from_algebraic("d4").unwrap();
    board.put_piece(Color::White, Piece::Bishop, d4);

    let moves = generate_bishop_moves(&board, Color::White);
    assert_eq!(moves.len(), 13);
    assert!(moves.iter().all(|m| !m.is_capture()));
}

#[test]
fn bishop_stops_before_own_piece() {
    let mut board = Board::empty();
    let d4 = Square::from_algebraic("d4").unwrap();
    let f6 = Square::from_algebraic("f6").unwrap();
    let g7 = Square::from_algebraic("g7").unwrap();
    board.put_piece(Color::White, Piece::Bishop, d4);
    board.put_piece(Color::White, Piece::Pawn, f6);

    let moves = generate_bishop_moves(&board, Color::White);
    assert!(!moves.iter().any(|m| m.to == f6));
    assert!(!moves.iter().any(|m| m.to == g7));
}

#[test]
fn bishop_can_capture_and_stops_there() {
    let mut board = Board::empty();
    let d4 = Square::from_algebraic("d4").unwrap();
    let f6 = Square::from_algebraic("f6").unwrap();
    let g7 = Square::from_algebraic("g7").unwrap();
    board.put_piece(Color::White, Piece::Bishop, d4);
    board.put_piece(Color::Black, Piece::Pawn, f6);

    let moves = generate_bishop_moves(&board, Color::White);
    let capture = moves
        .iter()
        .find(|m| m.to == f6)
        .expect("capture should exist");
    assert!(capture.is_capture());
    assert!(!moves.iter().any(|m| m.to == g7));
}

#[test]
fn rook_on_empty_board_a1_has_fourteen_moves() {
    let mut board = Board::empty();
    board.put_piece(Color::White, Piece::Rook, Square::A1);

    let moves = generate_rook_moves(&board, Color::White);
    assert_eq!(moves.len(), 14);
    assert!(moves.iter().all(|m| !m.is_capture()));
}

#[test]
fn rook_stops_before_own_piece_and_can_capture_enemy() {
    let mut board = Board::empty();
    let a4 = Square::from_algebraic("a4").unwrap();
    let a5 = Square::from_algebraic("a5").unwrap();
    board.put_piece(Color::White, Piece::Rook, Square::A1);
    board.put_piece(Color::Black, Piece::Pawn, a4);

    let moves = generate_rook_moves(&board, Color::White);
    let capture = moves
        .iter()
        .find(|m| m.to == a4)
        .expect("capture should exist");
    assert!(capture.is_capture());
    assert!(!moves.iter().any(|m| m.to == a5));
}

#[test]
fn queen_on_empty_board_d4_combines_bishop_and_rook_moves() {
    let mut board = Board::empty();
    let d4 = Square::from_algebraic("d4").unwrap();
    board.put_piece(Color::White, Piece::Queen, d4);

    let queen_moves = generate_queen_moves(&board, Color::White);

    let mut bishop_board = Board::empty();
    bishop_board.put_piece(Color::White, Piece::Bishop, d4);
    let bishop_moves = generate_bishop_moves(&bishop_board, Color::White);

    let mut rook_board = Board::empty();
    rook_board.put_piece(Color::White, Piece::Rook, d4);
    let rook_moves = generate_rook_moves(&rook_board, Color::White);

    assert_eq!(queen_moves.len(), bishop_moves.len() + rook_moves.len());
}

#[test]
fn starting_position_sliding_pieces_have_no_moves() {
    let board = Board::starting_position();
    assert!(generate_bishop_moves(&board, Color::White).is_empty());
    assert!(generate_rook_moves(&board, Color::White).is_empty());
    assert!(generate_queen_moves(&board, Color::White).is_empty());
}

#[test]
fn starting_position_white_pawns_have_sixteen_moves() {
    let board = Board::starting_position();
    let moves = generate_pawn_moves(&board, Color::White);
    // 8 pawns, each with single push + double push = 16, no captures available.
    assert_eq!(moves.len(), 16);
    assert!(moves.iter().all(|m| !m.is_capture()));
}

#[test]
fn starting_position_double_pushes_are_flagged() {
    let board = Board::starting_position();
    let moves = generate_pawn_moves(&board, Color::White);
    let double_pushes: Vec<_> = moves.iter().filter(|m| m.is_double_push).collect();
    assert_eq!(double_pushes.len(), 8);
    assert!(double_pushes.iter().all(|m| m.to.rank() == 3));
}

#[test]
fn pawn_blocked_directly_ahead_cannot_push() {
    let mut board = Board::empty();
    let e2 = Square::from_algebraic("e2").unwrap();
    let e3 = Square::from_algebraic("e3").unwrap();
    board.put_piece(Color::White, Piece::Pawn, e2);
    board.put_piece(Color::Black, Piece::Pawn, e3);

    let moves = generate_pawn_moves(&board, Color::White);
    assert!(moves.is_empty());
}

#[test]
fn pawn_blocked_on_double_push_square_only_gets_single_push() {
    let mut board = Board::empty();
    let e2 = Square::from_algebraic("e2").unwrap();
    let e4 = Square::from_algebraic("e4").unwrap();
    board.put_piece(Color::White, Piece::Pawn, e2);
    board.put_piece(Color::Black, Piece::Pawn, e4);

    let moves = generate_pawn_moves(&board, Color::White);
    assert_eq!(moves.len(), 1);
    assert!(!moves[0].is_double_push);
}

#[test]
fn pawn_not_on_start_rank_has_no_double_push() {
    let mut board = Board::empty();
    let e3 = Square::from_algebraic("e3").unwrap();
    board.put_piece(Color::White, Piece::Pawn, e3);

    let moves = generate_pawn_moves(&board, Color::White);
    assert_eq!(moves.len(), 1);
    assert!(!moves[0].is_double_push);
}

#[test]
fn pawn_can_capture_diagonally() {
    let mut board = Board::empty();
    let e4 = Square::from_algebraic("e4").unwrap();
    let d5 = Square::from_algebraic("d5").unwrap();
    board.put_piece(Color::White, Piece::Pawn, e4);
    board.put_piece(Color::Black, Piece::Pawn, d5);

    let moves = generate_pawn_moves(&board, Color::White);
    let capture = moves
        .iter()
        .find(|m| m.to == d5)
        .expect("capture should exist");
    assert!(capture.is_capture());
    assert_eq!(capture.captured, Some(Piece::Pawn));
}

#[test]
fn pawn_cannot_capture_own_piece() {
    let mut board = Board::empty();
    let e4 = Square::from_algebraic("e4").unwrap();
    let d5 = Square::from_algebraic("d5").unwrap();
    board.put_piece(Color::White, Piece::Pawn, e4);
    board.put_piece(Color::White, Piece::Pawn, d5);

    let moves = generate_pawn_moves(&board, Color::White);
    assert!(!moves.iter().any(|m| m.to == d5));
}

#[test]
fn black_pawn_pushes_toward_rank_one() {
    let mut board = Board::empty();
    let e7 = Square::from_algebraic("e7").unwrap();
    board.put_piece(Color::Black, Piece::Pawn, e7);

    let moves = generate_pawn_moves(&board, Color::Black);
    assert_eq!(moves.len(), 2);
    assert!(
        moves
            .iter()
            .any(|m| m.to == Square::from_algebraic("e6").unwrap())
    );
    assert!(
        moves
            .iter()
            .any(|m| m.to == Square::from_algebraic("e5").unwrap())
    );
}

#[test]
fn pawn_push_to_final_rank_generates_four_promotion_choices() {
    let mut board = Board::empty();
    let e7 = Square::from_algebraic("e7").unwrap();
    board.put_piece(Color::White, Piece::Pawn, e7);

    let moves = generate_pawn_moves(&board, Color::White);
    assert_eq!(moves.len(), 4);

    let promos: Vec<Piece> = moves.iter().map(|m| m.promotion.unwrap()).collect();
    assert!(promos.contains(&Piece::Queen));
    assert!(promos.contains(&Piece::Rook));
    assert!(promos.contains(&Piece::Bishop));
    assert!(promos.contains(&Piece::Knight));
    assert!(moves.iter().all(|m| !m.is_capture()));
}

#[test]
fn pawn_capture_promotion_generates_four_choices_with_capture_flag() {
    let mut board = Board::empty();
    let e7 = Square::from_algebraic("e7").unwrap();
    let d8 = Square::from_algebraic("d8").unwrap();
    board.put_piece(Color::White, Piece::Pawn, e7);
    board.put_piece(Color::Black, Piece::Rook, d8);

    let moves = generate_pawn_moves(&board, Color::White);
    let capture_promos: Vec<_> = moves.iter().filter(|m| m.to == d8).collect();
    assert_eq!(capture_promos.len(), 4);
    assert!(capture_promos.iter().all(|m| m.is_capture()));
    assert!(
        capture_promos
            .iter()
            .all(|m| m.captured == Some(Piece::Rook))
    );
}

#[test]
fn pawn_push_blocked_at_final_rank_generates_no_promotions() {
    let mut board = Board::empty();
    let e7 = Square::from_algebraic("e7").unwrap();
    let e8 = Square::from_algebraic("e8").unwrap();
    board.put_piece(Color::White, Piece::Pawn, e7);
    board.put_piece(Color::Black, Piece::Rook, e8);

    let moves = generate_pawn_moves(&board, Color::White);
    assert!(moves.is_empty());
}

#[test]
fn black_pawn_promotes_on_rank_one() {
    let mut board = Board::empty();
    let e2 = Square::from_algebraic("e2").unwrap();
    board.put_piece(Color::Black, Piece::Pawn, e2);

    let moves = generate_pawn_moves(&board, Color::Black);
    assert_eq!(moves.len(), 4);
    assert!(moves.iter().all(|m| m.to.rank() == 0));
}
