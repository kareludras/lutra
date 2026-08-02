use lutra_movegen::Piece;

#[test]
fn all_pieces_have_unique_indices() {
    let mut indices: Vec<usize> = Piece::ALL.iter().map(|p| p.index()).collect();
    indices.sort_unstable();
    assert_eq!(indices, vec![0, 1, 2, 3, 4, 5]);
}

#[test]
fn piece_chars_match_standard_notation() {
    assert_eq!(Piece::Pawn.to_char(), 'P');
    assert_eq!(Piece::Knight.to_char(), 'N');
    assert_eq!(Piece::Bishop.to_char(), 'B');
    assert_eq!(Piece::Rook.to_char(), 'R');
    assert_eq!(Piece::Queen.to_char(), 'Q');
    assert_eq!(Piece::King.to_char(), 'K');
}
