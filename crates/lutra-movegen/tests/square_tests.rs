use lutra_movegen::{Color, Square};

#[test]
fn lerf_indexing_matches_expected_squares() {
    assert_eq!(Square::A1.index(), 0);
    assert_eq!(Square::H1.index(), 7);
    assert_eq!(Square::A8.index(), 56);
    assert_eq!(Square::H8.index(), 63);
}

#[test]
fn file_and_rank_round_trip() {
    let sq = Square::from_file_rank(4, 3); // e4
    assert_eq!(sq.file(), 4);
    assert_eq!(sq.rank(), 3);
}

#[test]
fn algebraic_round_trip() {
    for s in ["a1", "h1", "a8", "h8", "e4", "d5"] {
        let sq = Square::from_algebraic(s).expect("valid algebraic square");
        assert_eq!(sq.to_algebraic(), s);
    }
}

#[test]
fn algebraic_rejects_invalid_input() {
    assert_eq!(Square::from_algebraic("i1"), None);
    assert_eq!(Square::from_algebraic("a9"), None);
    assert_eq!(Square::from_algebraic("a"), None);
    assert_eq!(Square::from_algebraic("a11"), None);
}

#[test]
fn opposite_color_flips() {
    assert_eq!(Color::White.opposite(), Color::Black);
    assert_eq!(Color::Black.opposite(), Color::White);
}
