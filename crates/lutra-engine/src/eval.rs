use lutra_movegen::attacks::{knight_attacks, pawn_attacks};
use lutra_movegen::sliding::{bishop_attacks, queen_attacks, rook_attacks};
use lutra_movegen::{Bitboard, Board, Color, Piece, Square};

/// Standard centipawn material values, used for move ordering and pruning
/// margins in search (not by `evaluate`, which has its own tapered values).
/// King is excluded: it is never captured.
pub fn material_value(piece: Piece) -> i32 {
    match piece {
        Piece::Pawn => 100,
        Piece::Knight => 320,
        Piece::Bishop => 330,
        Piece::Rook => 500,
        Piece::Queen => 900,
        Piece::King => 0,
    }
}

/// A middlegame/endgame score pair. `evaluate` blends the two by game
/// phase ("tapered eval"), so terms can matter differently as material
/// comes off: a centralised king is a liability early and an asset late.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Score {
    mg: i32,
    eg: i32,
}

impl Score {
    const fn new(mg: i32, eg: i32) -> Self {
        Score { mg, eg }
    }
}

impl std::ops::AddAssign for Score {
    fn add_assign(&mut self, rhs: Score) {
        self.mg += rhs.mg;
        self.eg += rhs.eg;
    }
}

impl std::ops::SubAssign for Score {
    fn sub_assign(&mut self, rhs: Score) {
        self.mg -= rhs.mg;
        self.eg -= rhs.eg;
    }
}

/// Phase weight per piece; 24 = all minor and major pieces on the board
/// (pure middlegame), 0 = only kings and pawns (pure endgame).
const PHASE_WEIGHT: [i32; 6] = [0, 1, 1, 2, 4, 0];
const MAX_PHASE: i32 = 24;

/// Evaluation from the perspective of the side to move: positive means the
/// side to move is better. Symmetric: mirroring the position (flipping the
/// board and swapping colours) gives the same score.
pub fn evaluate(board: &Board) -> i32 {
    let mut score = Score::default();
    let mut phase = 0;

    for piece in Piece::ALL {
        for sq in board.pieces(Color::White, piece) {
            score += pesto::value(piece, sq.index() as usize ^ 56);
            phase += PHASE_WEIGHT[piece.index()];
        }
        for sq in board.pieces(Color::Black, piece) {
            score -= pesto::value(piece, sq.index() as usize);
            phase += PHASE_WEIGHT[piece.index()];
        }
    }

    score += terms(board, Color::White);
    score -= terms(board, Color::Black);

    // Early promotions can push the phase past the maximum.
    let phase = phase.min(MAX_PHASE);
    let white_view = (score.mg * phase + score.eg * (MAX_PHASE - phase)) / MAX_PHASE;
    match board.side_to_move() {
        Color::White => white_view,
        Color::Black => -white_view,
    }
}

// Hand-set (untuned) weights for terms PeSTO's tables can't express, since
// they depend on more than one piece. Together they measured +68 +/- 31 Elo
// over PeSTO alone (400 games, 2+0.02); individual terms are untested and
// are the first candidates for tuning.
const BISHOP_PAIR: Score = Score::new(30, 50);
const DOUBLED_PAWN: Score = Score::new(-10, -20);
const ISOLATED_PAWN: Score = Score::new(-10, -10);
/// Indexed by the pawn's rank from its own side (1 = home rank).
const PASSED_PAWN: [Score; 8] = [
    Score::new(0, 0),
    Score::new(0, 10),
    Score::new(5, 15),
    Score::new(10, 25),
    Score::new(20, 45),
    Score::new(35, 70),
    Score::new(55, 110),
    Score::new(0, 0),
];
const ROOK_OPEN_FILE: Score = Score::new(25, 10);
const ROOK_SEMI_OPEN_FILE: Score = Score::new(12, 8);
/// Per reachable square above/below a typical count (`MOBILITY_BASELINE`),
/// for knight, bishop, rook, queen. Squares attacked by enemy pawns don't
/// count.
const MOBILITY: [Score; 4] = [
    Score::new(4, 4),
    Score::new(5, 5),
    Score::new(2, 4),
    Score::new(1, 2),
];
const MOBILITY_BASELINE: [i32; 4] = [4, 6, 7, 14];

const FILE_A: u64 = 0x0101_0101_0101_0101;

fn file_mask(file: u8) -> Bitboard {
    Bitboard(FILE_A << file)
}

fn adjacent_files(file: u8) -> Bitboard {
    let left = if file > 0 { FILE_A << (file - 1) } else { 0 };
    let right = if file < 7 { FILE_A << (file + 1) } else { 0 };
    Bitboard(left | right)
}

/// Squares strictly in front of `sq` from `color`'s point of view.
fn ranks_ahead(color: Color, sq: Square) -> Bitboard {
    let rank = sq.rank() as u32;
    Bitboard(match color {
        Color::White => u64::MAX.checked_shl((rank + 1) * 8).unwrap_or(0),
        Color::Black => (1u64 << (rank * 8)) - 1,
    })
}

/// Multi-piece terms for `color`, from white's sign convention handled by
/// the caller (added for white, subtracted for black).
fn terms(board: &Board, color: Color) -> Score {
    let them = color.opposite();
    let mut score = Score::default();

    if board.pieces(color, Piece::Bishop).count() >= 2 {
        score += BISHOP_PAIR;
    }

    let own_pawns = board.pieces(color, Piece::Pawn);
    let their_pawns = board.pieces(them, Piece::Pawn);
    for sq in own_pawns {
        let file = sq.file();
        if (own_pawns & file_mask(file)).count() > 1 {
            // Counted once per pawn on a doubled file, so a pair costs
            // twice the weight.
            score += DOUBLED_PAWN;
        }
        if (own_pawns & adjacent_files(file)).is_empty() {
            score += ISOLATED_PAWN;
        }
        let front_span = (file_mask(file) | adjacent_files(file)) & ranks_ahead(color, sq);
        if (their_pawns & front_span).is_empty() {
            let relative_rank = match color {
                Color::White => sq.rank(),
                Color::Black => 7 - sq.rank(),
            };
            score += PASSED_PAWN[relative_rank as usize];
        }
    }

    for sq in board.pieces(color, Piece::Rook) {
        let file = file_mask(sq.file());
        if (own_pawns & file).is_empty() {
            score += if (their_pawns & file).is_empty() {
                ROOK_OPEN_FILE
            } else {
                ROOK_SEMI_OPEN_FILE
            };
        }
    }

    let occupied = board.all_occupancy();
    let mut pawn_attacked = Bitboard::EMPTY;
    for sq in their_pawns {
        pawn_attacked |= pawn_attacks(them, sq);
    }
    let mobility_area = !(board.occupancy(color) | pawn_attacked);
    for (i, piece) in [Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen]
        .into_iter()
        .enumerate()
    {
        for sq in board.pieces(color, piece) {
            let attacks = match piece {
                Piece::Knight => knight_attacks(sq),
                Piece::Bishop => bishop_attacks(sq, occupied),
                Piece::Rook => rook_attacks(sq, occupied),
                _ => queen_attacks(sq, occupied),
            };
            let count = (attacks & mobility_area).count() as i32 - MOBILITY_BASELINE[i];
            score += Score::new(MOBILITY[i].mg * count, MOBILITY[i].eg * count);
        }
    }

    score
}

/// Piece values and piece-square tables from PeSTO by Ronald Friederich
/// (published on the Chess Programming Wiki), tuned as a set. Tables are
/// laid out as printed, a8..h8 first, so a white piece on square `sq`
/// (a1 = 0) uses index `sq ^ 56` and a black piece uses `sq` directly.
mod pesto {
    use super::Score;
    use lutra_movegen::Piece;

    const MG_VALUE: [i32; 6] = [82, 337, 365, 477, 1025, 0];
    const EG_VALUE: [i32; 6] = [94, 281, 297, 512, 936, 0];

    pub(super) fn value(piece: Piece, index: usize) -> Score {
        let p = piece.index();
        Score::new(
            MG_VALUE[p] + MG_TABLES[p][index],
            EG_VALUE[p] + EG_TABLES[p][index],
        )
    }

    const MG_TABLES: [[i32; 64]; 6] = [MG_PAWN, MG_KNIGHT, MG_BISHOP, MG_ROOK, MG_QUEEN, MG_KING];
    const EG_TABLES: [[i32; 64]; 6] = [EG_PAWN, EG_KNIGHT, EG_BISHOP, EG_ROOK, EG_QUEEN, EG_KING];

    #[rustfmt::skip]
    const MG_PAWN: [i32; 64] = [
          0,   0,   0,   0,   0,   0,   0,   0,
         98, 134,  61,  95,  68, 126,  34, -11,
         -6,   7,  26,  31,  65,  56,  25, -20,
        -14,  13,   6,  21,  23,  12,  17, -23,
        -27,  -2,  -5,  12,  17,   6,  10, -25,
        -26,  -4,  -4, -10,   3,   3,  33, -12,
        -35,  -1, -20, -23, -15,  24,  38, -22,
          0,   0,   0,   0,   0,   0,   0,   0,
    ];
    #[rustfmt::skip]
    const EG_PAWN: [i32; 64] = [
          0,   0,   0,   0,   0,   0,   0,   0,
        178, 173, 158, 134, 147, 132, 165, 187,
         94, 100,  85,  67,  56,  53,  82,  84,
         32,  24,  13,   5,  -2,   4,  17,  17,
         13,   9,  -3,  -7,  -7,  -8,   3,  -1,
          4,   7,  -6,   1,   0,  -5,  -1,  -8,
         13,   8,   8,  10,  13,   0,   2,  -7,
          0,   0,   0,   0,   0,   0,   0,   0,
    ];
    #[rustfmt::skip]
    const MG_KNIGHT: [i32; 64] = [
        -167, -89, -34, -49,  61, -97, -15, -107,
         -73, -41,  72,  36,  23,  62,   7,  -17,
         -47,  60,  37,  65,  84, 129,  73,   44,
          -9,  17,  19,  53,  37,  69,  18,   22,
         -13,   4,  16,  13,  28,  19,  21,   -8,
         -23,  -9,  12,  10,  19,  17,  25,  -16,
         -29, -53, -12,  -3,  -1,  18, -14,  -19,
        -105, -21, -58, -33, -17, -28, -19,  -23,
    ];
    #[rustfmt::skip]
    const EG_KNIGHT: [i32; 64] = [
        -58, -38, -13, -28, -31, -27, -63, -99,
        -25,  -8, -25,  -2,  -9, -25, -24, -52,
        -24, -20,  10,   9,  -1,  -9, -19, -41,
        -17,   3,  22,  22,  22,  11,   8, -18,
        -18,  -6,  16,  25,  16,  17,   4, -18,
        -23,  -3,  -1,  15,  10,  -3, -20, -22,
        -42, -20, -10,  -5,  -2, -20, -23, -44,
        -29, -51, -23, -15, -22, -18, -50, -64,
    ];
    #[rustfmt::skip]
    const MG_BISHOP: [i32; 64] = [
        -29,   4, -82, -37, -25, -42,   7,  -8,
        -26,  16, -18, -13,  30,  59,  18, -47,
        -16,  37,  43,  40,  35,  50,  37,  -2,
         -4,   5,  19,  50,  37,  37,   7,  -2,
         -6,  13,  13,  26,  34,  12,  10,   4,
          0,  15,  15,  15,  14,  27,  18,  10,
          4,  15,  16,   0,   7,  21,  33,   1,
        -33,  -3, -14, -21, -13, -12, -39, -21,
    ];
    #[rustfmt::skip]
    const EG_BISHOP: [i32; 64] = [
        -14, -21, -11,  -8,  -7,  -9, -17, -24,
         -8,  -4,   7, -12,  -3, -13,  -4, -14,
          2,  -8,   0,  -1,  -2,   6,   0,   4,
         -3,   9,  12,   9,  14,  10,   3,   2,
         -6,   3,  13,  19,   7,  10,  -3,  -9,
        -12,  -3,   8,  10,  13,   3,  -7, -15,
        -14, -18,  -7,  -1,   4,  -9, -15, -27,
        -23,  -9, -23,  -5,  -9, -16,  -5, -17,
    ];
    #[rustfmt::skip]
    const MG_ROOK: [i32; 64] = [
         32,  42,  32,  51,  63,   9,  31,  43,
         27,  32,  58,  62,  80,  67,  26,  44,
         -5,  19,  26,  36,  17,  45,  61,  16,
        -24, -11,   7,  26,  24,  35,  -8, -20,
        -36, -26, -12,  -1,   9,  -7,   6, -23,
        -45, -25, -16, -17,   3,   0,  -5, -33,
        -44, -16, -20,  -9,  -1,  11,  -6, -71,
        -19, -13,   1,  17,  16,   7, -37, -26,
    ];
    #[rustfmt::skip]
    const EG_ROOK: [i32; 64] = [
         13,  10,  18,  15,  12,  12,   8,   5,
         11,  13,  13,  11,  -3,   3,   8,   3,
          7,   7,   7,   5,   4,  -3,  -5,  -3,
          4,   3,  13,   1,   2,   1,  -1,   2,
          3,   5,   8,   4,  -5,  -6,  -8, -11,
         -4,   0,  -5,  -1,  -7, -12,  -8, -16,
         -6,  -6,   0,   2,  -9,  -9, -11,  -3,
         -9,   2,   3,  -1,  -5, -13,   4, -20,
    ];
    #[rustfmt::skip]
    const MG_QUEEN: [i32; 64] = [
        -28,   0,  29,  12,  59,  44,  43,  45,
        -24, -39,  -5,   1, -16,  57,  28,  54,
        -13, -17,   7,   8,  29,  56,  47,  57,
        -27, -27, -16, -16,  -1,  17,  -2,   1,
         -9, -26,  -9, -10,  -2,  -4,   3,  -3,
        -14,   2, -11,  -2,  -5,   2,  14,   5,
        -35,  -8,  11,   2,   8,  15,  -3,   1,
         -1, -18,  -9,  10, -15, -25, -31, -50,
    ];
    #[rustfmt::skip]
    const EG_QUEEN: [i32; 64] = [
         -9,  22,  22,  27,  27,  19,  10,  20,
        -17,  20,  32,  41,  58,  25,  30,   0,
        -20,   6,   9,  49,  47,  35,  19,   9,
          3,  22,  24,  45,  57,  40,  57,  36,
        -18,  28,  19,  47,  31,  34,  39,  23,
        -16, -27,  15,   6,   9,  17,  10,   5,
        -22, -23, -30, -16, -16, -23, -36, -32,
        -33, -28, -22, -43,  -5, -32, -20, -41,
    ];
    #[rustfmt::skip]
    const MG_KING: [i32; 64] = [
        -65,  23,  16, -15, -56, -34,   2,  13,
         29,  -1, -20,  -7,  -8,  -4, -38, -29,
         -9,  24,   2, -16, -20,   6,  22, -22,
        -17, -20, -12, -27, -30, -25, -14, -36,
        -49,  -1, -27, -39, -46, -44, -33, -51,
        -14, -14, -22, -46, -44, -30, -15, -27,
          1,   7,  -8, -64, -43, -16,   9,   8,
        -15,  36,  12, -54,   8, -28,  24,  14,
    ];
    #[rustfmt::skip]
    const EG_KING: [i32; 64] = [
        -74, -35, -18, -18, -11,  15,   4, -17,
        -12,  17,  14,  17,  17,  38,  23,  11,
         10,  17,  23,  15,  20,  45,  44,  13,
         -8,  22,  24,  27,  26,  33,  26,   3,
        -18,  -4,  21,  24,  27,  23,   9, -11,
        -19,  -3,  11,  21,  23,  16,   7,  -9,
        -27, -11,   4,  13,  14,   4,  -5, -17,
        -53, -34, -21, -11, -28, -14, -24, -43,
    ];
}
