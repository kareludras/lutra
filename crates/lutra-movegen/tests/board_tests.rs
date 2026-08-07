use lutra_movegen::{Board, CastlingRights, Color, Piece, Square};

#[test]
fn empty_board_has_no_pieces() {
    let board = Board::empty();
    assert!(board.all_occupancy().is_empty());
    for piece in Piece::ALL {
        assert!(board.pieces(Color::White, piece).is_empty());
        assert!(board.pieces(Color::Black, piece).is_empty());
    }
}

#[test]
fn starting_position_has_correct_piece_counts() {
    let board = Board::starting_position();

    assert_eq!(board.pieces(Color::White, Piece::Pawn).count(), 8);
    assert_eq!(board.pieces(Color::White, Piece::Knight).count(), 2);
    assert_eq!(board.pieces(Color::White, Piece::Bishop).count(), 2);
    assert_eq!(board.pieces(Color::White, Piece::Rook).count(), 2);
    assert_eq!(board.pieces(Color::White, Piece::Queen).count(), 1);
    assert_eq!(board.pieces(Color::White, Piece::King).count(), 1);

    assert_eq!(board.pieces(Color::Black, Piece::Pawn).count(), 8);
    assert_eq!(board.pieces(Color::Black, Piece::Knight).count(), 2);
    assert_eq!(board.pieces(Color::Black, Piece::Bishop).count(), 2);
    assert_eq!(board.pieces(Color::Black, Piece::Rook).count(), 2);
    assert_eq!(board.pieces(Color::Black, Piece::Queen).count(), 1);
    assert_eq!(board.pieces(Color::Black, Piece::King).count(), 1);
}

#[test]
fn starting_position_has_32_total_pieces() {
    let board = Board::starting_position();
    assert_eq!(board.all_occupancy().count(), 32);
}

#[test]
fn starting_position_occupancy_matches_ranks_one_two_seven_eight() {
    let board = Board::starting_position();
    let white = board.occupancy(Color::White);
    let black = board.occupancy(Color::Black);

    for file in 0..8 {
        assert!(white.contains(Square::from_file_rank(file, 0)));
        assert!(white.contains(Square::from_file_rank(file, 1)));
        assert!(black.contains(Square::from_file_rank(file, 6)));
        assert!(black.contains(Square::from_file_rank(file, 7)));
    }
    assert_eq!(white.count(), 16);
    assert_eq!(black.count(), 16);
}

#[test]
fn starting_position_places_kings_and_queens_correctly() {
    let board = Board::starting_position();
    assert!(board.pieces(Color::White, Piece::King).contains(Square::from_algebraic("e1").unwrap()));
    assert!(board.pieces(Color::White, Piece::Queen).contains(Square::from_algebraic("d1").unwrap()));
    assert!(board.pieces(Color::Black, Piece::King).contains(Square::from_algebraic("e8").unwrap()));
    assert!(board.pieces(Color::Black, Piece::Queen).contains(Square::from_algebraic("d8").unwrap()));
}

#[test]
fn starting_position_has_white_to_move_and_all_castling_rights() {
    let board = Board::starting_position();
    assert_eq!(board.side_to_move(), Color::White);
    assert!(board.castling_rights().has(CastlingRights::WHITE_KINGSIDE));
    assert!(board.castling_rights().has(CastlingRights::WHITE_QUEENSIDE));
    assert!(board.castling_rights().has(CastlingRights::BLACK_KINGSIDE));
    assert!(board.castling_rights().has(CastlingRights::BLACK_QUEENSIDE));
    assert_eq!(board.en_passant(), None);
}

#[test]
fn put_and_remove_piece_round_trip() {
    let mut board = Board::empty();
    let e4 = Square::from_algebraic("e4").unwrap();
    board.put_piece(Color::White, Piece::Knight, e4);
    assert!(board.pieces(Color::White, Piece::Knight).contains(e4));

    board.remove_piece(Color::White, Piece::Knight, e4);
    assert!(!board.pieces(Color::White, Piece::Knight).contains(e4));
}

#[test]
fn castling_rights_set_and_clear_individual_flags() {
    let mut rights = CastlingRights::NONE;
    assert!(!rights.has(CastlingRights::WHITE_KINGSIDE));

    rights.set(CastlingRights::WHITE_KINGSIDE);
    assert!(rights.has(CastlingRights::WHITE_KINGSIDE));
    assert!(!rights.has(CastlingRights::WHITE_QUEENSIDE));

    rights.clear(CastlingRights::WHITE_KINGSIDE);
    assert!(!rights.has(CastlingRights::WHITE_KINGSIDE));
}