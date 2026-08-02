use crate::bitboard::Bitboard;
use crate::square::Square;

const FILE_A: u64 = 0x0101_0101_0101_0101;
const FILE_B: u64 = FILE_A << 1;
const FILE_G: u64 = FILE_A << 6;
const FILE_H: u64 = FILE_A << 7;

const NOT_FILE_A: u64 = !FILE_A;
const NOT_FILE_H: u64 = !FILE_H;
const NOT_FILE_AB: u64 = !(FILE_A | FILE_B);
const NOT_FILE_GH: u64 = !(FILE_G | FILE_H);

/// Squares a knight on `sq` attacks. File masks prevent wraparound at board edges.
pub fn knight_attacks(sq: Square) -> Bitboard {
    let b = 1u64 << sq.index();

    let mut attacks = 0u64;
    attacks |= (b << 17) & NOT_FILE_A;
    attacks |= (b << 15) & NOT_FILE_H;
    attacks |= (b << 10) & NOT_FILE_AB;
    attacks |= (b << 6) & NOT_FILE_GH;
    attacks |= (b >> 17) & NOT_FILE_H;
    attacks |= (b >> 15) & NOT_FILE_A;
    attacks |= (b >> 10) & NOT_FILE_GH;
    attacks |= (b >> 6) & NOT_FILE_AB;

    Bitboard::new(attacks)
}

/// Squares a king on `sq` attacks. File masks prevent wraparound at board edges.
pub fn king_attacks(sq: Square) -> Bitboard {
    let b = 1u64 << sq.index();

    let mut attacks = 0u64;
    attacks |= b << 8;
    attacks |= b >> 8;
    attacks |= (b << 1) & NOT_FILE_A;
    attacks |= (b >> 1) & NOT_FILE_H;
    attacks |= (b << 9) & NOT_FILE_A;
    attacks |= (b << 7) & NOT_FILE_H;
    attacks |= (b >> 7) & NOT_FILE_A;
    attacks |= (b >> 9) & NOT_FILE_H;

    Bitboard::new(attacks)
}
