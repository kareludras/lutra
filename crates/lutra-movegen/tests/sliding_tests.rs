use lutra_movegen::sliding::{bishop_attacks, queen_attacks, rook_attacks};
use lutra_movegen::{Bitboard, Square};

#[test]
fn rook_on_a1_empty_board_attacks_full_file_and_rank() {
    let attacks = rook_attacks(Square::A1, Bitboard::EMPTY);
    assert_eq!(attacks.count(), 14);
}

#[test]
fn bishop_on_d4_empty_board() {
    let d4 = Square::from_file_rank(3, 3);
    let attacks = bishop_attacks(d4, Bitboard::EMPTY);
    assert_eq!(attacks.count(), 13);
}

#[test]
fn rook_stops_at_first_blocker_inclusive() {
    let a4 = Square::from_file_rank(0, 3);
    let mut occ = Bitboard::EMPTY;
    occ.set(a4);
    let attacks = rook_attacks(Square::A1, occ);

    let a2 = Square::from_file_rank(0, 1);
    let a3 = Square::from_file_rank(0, 2);
    let a5 = Square::from_file_rank(0, 4);

    assert!(attacks.contains(a2));
    assert!(attacks.contains(a3));
    assert!(attacks.contains(a4)); // blocker itself is capturable
    assert!(!attacks.contains(a5)); // nothing past the blocker
    assert_eq!(attacks.count(), 10); // 7 along rank 1 + a2,a3,a4
}

#[test]
fn bishop_stops_at_first_blocker_inclusive() {
    let d4 = Square::from_file_rank(3, 3);
    let f6 = Square::from_file_rank(5, 5);
    let mut occ = Bitboard::EMPTY;
    occ.set(f6);
    let attacks = bishop_attacks(d4, occ);

    let e5 = Square::from_file_rank(4, 4);
    let g7 = Square::from_file_rank(6, 6);

    assert!(attacks.contains(e5));
    assert!(attacks.contains(f6)); // blocker itself is capturable
    assert!(!attacks.contains(g7)); // nothing past the blocker
}

#[test]
fn queen_attacks_is_union_of_bishop_and_rook() {
    let d4 = Square::from_file_rank(3, 3);
    let q = queen_attacks(d4, Bitboard::EMPTY);
    let b = bishop_attacks(d4, Bitboard::EMPTY);
    let r = rook_attacks(d4, Bitboard::EMPTY);
    assert_eq!(q, b | r);
}
