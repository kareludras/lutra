use crate::bitboard::Bitboard;
use crate::square::Square;

const BISHOP_DIRECTIONS: [(i8, i8); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
const ROOK_DIRECTIONS: [(i8, i8); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

/// Walks each direction from `sq` one square at a time, stopping at the board
/// edge or the first occupied square (inclusive, since that square is capturable).
fn ray_attacks(sq: Square, occupancy: Bitboard, directions: &[(i8, i8)]) -> Bitboard {
    let mut attacks = Bitboard::EMPTY;
    let start_file = sq.file() as i8;
    let start_rank = sq.rank() as i8;

    for &(df, dr) in directions {
        let mut file = start_file + df;
        let mut rank = start_rank + dr;

        while (0..8).contains(&file) && (0..8).contains(&rank) {
            let target = Square::from_file_rank(file as u8, rank as u8);
            attacks.set(target);
            if occupancy.contains(target) {
                break;
            }
            file += df;
            rank += dr;
        }
    }

    attacks
}

pub fn bishop_attacks(sq: Square, occupancy: Bitboard) -> Bitboard {
    ray_attacks(sq, occupancy, &BISHOP_DIRECTIONS)
}

pub fn rook_attacks(sq: Square, occupancy: Bitboard) -> Bitboard {
    ray_attacks(sq, occupancy, &ROOK_DIRECTIONS)
}

pub fn queen_attacks(sq: Square, occupancy: Bitboard) -> Bitboard {
    bishop_attacks(sq, occupancy) | rook_attacks(sq, occupancy)
}
