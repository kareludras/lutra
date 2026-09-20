use crate::board::Board;
use crate::castling::CastlingRights;
use crate::piece::Piece;
use crate::square::Color;

/// SplitMix64: a tiny, high-quality generator, usable in `const fn` so the
/// Zobrist keys are fixed at compile time and identical on every run.
const fn splitmix64(state: u64) -> (u64, u64) {
    let state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    (state, z ^ (z >> 31))
}

struct Keys {
    pieces: [[[u64; 64]; 6]; 2],
    castling: [u64; 16],
    en_passant_file: [u64; 8],
    black_to_move: u64,
}

const KEYS: Keys = {
    let mut state = 0x4C55_5452_4121_u64; // "LUTRA!"
    let mut keys = Keys {
        pieces: [[[0; 64]; 6]; 2],
        castling: [0; 16],
        en_passant_file: [0; 8],
        black_to_move: 0,
    };
    let mut c = 0;
    while c < 2 {
        let mut p = 0;
        while p < 6 {
            let mut sq = 0;
            while sq < 64 {
                let (s, k) = splitmix64(state);
                state = s;
                keys.pieces[c][p][sq] = k;
                sq += 1;
            }
            p += 1;
        }
        c += 1;
    }
    let mut i = 0;
    while i < 16 {
        let (s, k) = splitmix64(state);
        state = s;
        keys.castling[i] = k;
        i += 1;
    }
    let mut f = 0;
    while f < 8 {
        let (s, k) = splitmix64(state);
        state = s;
        keys.en_passant_file[f] = k;
        f += 1;
    }
    let (_, k) = splitmix64(state);
    keys.black_to_move = k;
    keys
};

impl Board {
    /// Zobrist hash of the position: piece placement, side to move,
    /// castling rights and en passant file. Move counters are excluded, so
    /// two occurrences of the same position hash equal - which is exactly
    /// what repetition detection and a transposition table need.
    ///
    /// Computed from scratch (one XOR per piece); cheap next to move
    /// generation in this copy-make design.
    pub fn hash(&self) -> u64 {
        let mut h = 0;
        for color in [Color::White, Color::Black] {
            for piece in Piece::ALL {
                for sq in self.pieces(color, piece) {
                    h ^= KEYS.pieces[color.index()][piece.index()][sq.index() as usize];
                }
            }
        }
        h ^= KEYS.castling[castling_index(self.castling_rights())];
        if let Some(ep) = self.en_passant() {
            h ^= KEYS.en_passant_file[ep.file() as usize];
        }
        if self.side_to_move() == Color::Black {
            h ^= KEYS.black_to_move;
        }
        h
    }
}

fn castling_index(rights: CastlingRights) -> usize {
    [
        CastlingRights::WHITE_KINGSIDE,
        CastlingRights::WHITE_QUEENSIDE,
        CastlingRights::BLACK_KINGSIDE,
        CastlingRights::BLACK_QUEENSIDE,
    ]
    .iter()
    .enumerate()
    .filter(|&(_, &flag)| rights.has(flag))
    .map(|(i, _)| 1 << i)
    .sum()
}
