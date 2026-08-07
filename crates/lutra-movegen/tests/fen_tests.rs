use lutra_movegen::{Board, CastlingRights, Color, FenError, Piece, Square};

const STARTPOS_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[test]
fn startpos_fen_matches_hardcoded_starting_position() {
    let from_fen = Board::from_fen(STARTPOS_FEN).unwrap();
    let hardcoded = Board::starting_position();

    for piece in Piece::ALL {
        assert_eq!(
            from_fen.pieces(Color::White, piece),
            hardcoded.pieces(Color::White, piece)
        );
        assert_eq!(
            from_fen.pieces(Color::Black, piece),
            hardcoded.pieces(Color::Black, piece)
        );
    }
    assert_eq!(from_fen.side_to_move(), hardcoded.side_to_move());
    assert_eq!(from_fen.en_passant(), hardcoded.en_passant());
}

#[test]
fn fen_with_partial_castling_rights_parses_correctly() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w Kq - 0 1";
    let board = Board::from_fen(fen).unwrap();
    assert!(board.castling_rights().has(CastlingRights::WHITE_KINGSIDE));
    assert!(!board.castling_rights().has(CastlingRights::WHITE_QUEENSIDE));
    assert!(!board.castling_rights().has(CastlingRights::BLACK_KINGSIDE));
    assert!(board.castling_rights().has(CastlingRights::BLACK_QUEENSIDE));
}

#[test]
fn fen_with_no_castling_rights_parses_as_none() {
    let fen = "8/8/8/8/8/8/8/8 w - - 0 1";
    let board = Board::from_fen(fen).unwrap();
    assert_eq!(board.castling_rights(), CastlingRights::NONE);
}

#[test]
fn fen_with_en_passant_square_parses_correctly() {
    let fen = "rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3";
    let board = Board::from_fen(fen).unwrap();
    assert_eq!(board.en_passant(), Square::from_algebraic("d6"));
}

#[test]
fn fen_black_to_move_parses_correctly() {
    let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
    let board = Board::from_fen(fen).unwrap();
    assert_eq!(board.side_to_move(), Color::Black);
}

#[test]
fn fen_halfmove_and_fullmove_counters_parse_correctly() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 12 34";
    let board = Board::from_fen(fen).unwrap();
    assert_eq!(board.halfmove_clock(), 12);
    assert_eq!(board.fullmove_number(), 34);
}

#[test]
fn fen_with_wrong_field_count_is_rejected() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -";
    assert_eq!(Board::from_fen(fen), Err(FenError::WrongFieldCount));
}

#[test]
fn fen_with_wrong_rank_count_is_rejected() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP w KQkq - 0 1";
    assert_eq!(Board::from_fen(fen), Err(FenError::InvalidPiecePlacement));
}

#[test]
fn fen_with_invalid_piece_letter_is_rejected() {
    let fen = "rnbqkbnx/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    assert_eq!(Board::from_fen(fen), Err(FenError::InvalidPiecePlacement));
}

#[test]
fn fen_with_rank_not_summing_to_eight_is_rejected() {
    let fen = "rnbqkbn/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    assert_eq!(Board::from_fen(fen), Err(FenError::InvalidPiecePlacement));
}

#[test]
fn fen_with_invalid_side_to_move_is_rejected() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR x KQkq - 0 1";
    assert_eq!(Board::from_fen(fen), Err(FenError::InvalidSideToMove));
}

#[test]
fn fen_with_invalid_castling_rights_is_rejected() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w XYZq - 0 1";
    assert_eq!(Board::from_fen(fen), Err(FenError::InvalidCastlingRights));
}

#[test]
fn fen_with_invalid_en_passant_is_rejected() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq z9 0 1";
    assert_eq!(Board::from_fen(fen), Err(FenError::InvalidEnPassant));
}
