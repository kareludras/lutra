use lutra_movegen::Square;
use lutra_movegen::attacks::{king_attacks, knight_attacks};

#[test]
fn knight_on_a1_attacks_only_b3_and_c2() {
    let attacks = knight_attacks(Square::A1);
    let b3 = Square::from_file_rank(1, 2);
    let c2 = Square::from_file_rank(2, 1);
    assert_eq!(attacks.count(), 2);
    assert!(attacks.contains(b3));
    assert!(attacks.contains(c2));
}

#[test]
fn knight_on_d4_has_eight_attacks() {
    let d4 = Square::from_file_rank(3, 3);
    assert_eq!(knight_attacks(d4).count(), 8);
}

#[test]
fn knight_attacks_never_wrap_around_files() {
    for sq in knight_attacks(Square::H1) {
        assert_ne!(sq.file(), 0, "knight on h1 wrapped to file a");
    }
}

#[test]
fn king_on_a1_attacks_three_squares() {
    let attacks = king_attacks(Square::A1);
    assert_eq!(attacks.count(), 3);
    assert!(attacks.contains(Square::from_file_rank(0, 1))); // a2
    assert!(attacks.contains(Square::from_file_rank(1, 0))); // b1
    assert!(attacks.contains(Square::from_file_rank(1, 1))); // b2
}

#[test]
fn king_on_d4_has_eight_attacks() {
    let d4 = Square::from_file_rank(3, 3);
    assert_eq!(king_attacks(d4).count(), 8);
}

#[test]
fn king_attacks_never_wrap_around_files() {
    for sq in king_attacks(Square::A1) {
        assert!(sq.file() <= 1, "king on a1 wrapped past file b");
    }
}
