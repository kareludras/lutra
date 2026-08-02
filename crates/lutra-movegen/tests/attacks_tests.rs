use lutra_movegen::attacks::{king_attacks, knight_attacks, pawn_attacks};
use lutra_movegen::{Color, Square};

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

#[test]
fn white_pawn_on_e4_attacks_d5_and_f5() {
    let e4 = Square::from_file_rank(4, 3);
    let attacks = pawn_attacks(Color::White, e4);
    assert_eq!(attacks.count(), 2);
    assert!(attacks.contains(Square::from_file_rank(3, 4))); // d5
    assert!(attacks.contains(Square::from_file_rank(5, 4))); // f5
}

#[test]
fn black_pawn_on_e5_attacks_d4_and_f4() {
    let e5 = Square::from_file_rank(4, 4);
    let attacks = pawn_attacks(Color::Black, e5);
    assert_eq!(attacks.count(), 2);
    assert!(attacks.contains(Square::from_file_rank(3, 3))); // d4
    assert!(attacks.contains(Square::from_file_rank(5, 3))); // f4
}

#[test]
fn white_pawn_on_a4_has_only_one_attack() {
    let a4 = Square::from_file_rank(0, 3);
    let attacks = pawn_attacks(Color::White, a4);
    assert_eq!(attacks.count(), 1);
    assert!(attacks.contains(Square::from_file_rank(1, 4))); // b5
}

#[test]
fn black_pawn_on_h4_has_only_one_attack() {
    let h4 = Square::from_file_rank(7, 3);
    let attacks = pawn_attacks(Color::Black, h4);
    assert_eq!(attacks.count(), 1);
    assert!(attacks.contains(Square::from_file_rank(6, 2))); // g3
}

#[test]
fn pawn_attacks_never_wrap_around_files() {
    let a4 = Square::from_file_rank(0, 3);
    for sq in pawn_attacks(Color::White, a4) {
        assert_ne!(sq.file(), 7, "white pawn on a4 wrapped to file h");
    }
}
