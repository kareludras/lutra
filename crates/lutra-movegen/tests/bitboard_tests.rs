use lutra_movegen::{Bitboard, Square};

#[test]
fn empty_board_has_no_bits() {
    assert!(Bitboard::EMPTY.is_empty());
    assert_eq!(Bitboard::EMPTY.count(), 0);
}

#[test]
fn set_and_contains() {
    let mut bb = Bitboard::EMPTY;
    bb.set(Square::A1);
    assert!(bb.contains(Square::A1));
    assert!(!bb.contains(Square::H8));
    assert_eq!(bb.count(), 1);
}

#[test]
fn clear_removes_bit() {
    let e4 = Square::from_file_rank(4, 3);
    let mut bb = Bitboard::from_square(e4);
    bb.clear(e4);
    assert!(bb.is_empty());
}

#[test]
fn pop_lsb_iterates_in_ascending_order() {
    let mut bb = Bitboard::EMPTY;
    bb.set(Square::H8);
    bb.set(Square::A1);
    bb.set(Square::from_file_rank(4, 3)); // e4

    let squares: Vec<Square> = bb.collect();
    assert_eq!(
        squares,
        vec![Square::A1, Square::from_file_rank(4, 3), Square::H8]
    );
}

#[test]
fn bitwise_ops_combine_as_expected() {
    let a = Bitboard::from_square(Square::A1);
    let b = Bitboard::from_square(Square::H8);
    let union = a | b;
    assert!(union.contains(Square::A1));
    assert!(union.contains(Square::H8));
    assert_eq!(union.count(), 2);

    let intersection = a & b;
    assert!(intersection.is_empty());

    let xor = union ^ a;
    assert_eq!(xor, b);
}

#[test]
fn not_inverts_all_bits() {
    let a = Bitboard::from_square(Square::A1);
    let inverted = !a;
    assert!(!inverted.contains(Square::A1));
    assert!(inverted.contains(Square::H8));
}
