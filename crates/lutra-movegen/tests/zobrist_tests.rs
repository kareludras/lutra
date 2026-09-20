use lutra_movegen::{Board, generate_legal_moves};

fn play(board: &Board, uci_moves: &[&str]) -> Board {
    let mut board = board.clone();
    for token in uci_moves {
        let mv = generate_legal_moves(&board, board.side_to_move())
            .into_iter()
            .find(|m| m.to_string() == *token)
            .unwrap_or_else(|| panic!("{token} should be legal"));
        board = board.make_move(mv);
    }
    board
}

#[test]
fn transposed_move_orders_hash_equal() {
    let start = Board::starting_position();
    let a = play(&start, &["g1f3", "b8c6", "b1c3"]);
    let b = play(&start, &["b1c3", "b8c6", "g1f3"]);
    assert_eq!(a.hash(), b.hash());
}

#[test]
fn knight_shuffle_returns_to_the_starting_hash() {
    // Move counters differ, but the position is the same, which is what
    // repetition detection needs.
    let start = Board::starting_position();
    let back = play(&start, &["g1f3", "g8f6", "f3g1", "f6g8"]);
    assert_eq!(back.hash(), start.hash());
}

#[test]
fn side_to_move_changes_the_hash() {
    let white = Board::from_fen("4k3/8/8/8/8/8/8/4K3 w - - 0 1").unwrap();
    let black = Board::from_fen("4k3/8/8/8/8/8/8/4K3 b - - 0 1").unwrap();
    assert_ne!(white.hash(), black.hash());
}

#[test]
fn castling_rights_change_the_hash() {
    let all = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap();
    let some = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w Kq - 0 1").unwrap();
    assert_ne!(all.hash(), some.hash());
}

#[test]
fn en_passant_square_changes_the_hash() {
    let with_ep =
        Board::from_fen("rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3").unwrap();
    let without_ep =
        Board::from_fen("rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq - 0 3").unwrap();
    assert_ne!(with_ep.hash(), without_ep.hash());
}
