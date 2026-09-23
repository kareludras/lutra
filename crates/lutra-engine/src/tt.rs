use lutra_movegen::Move;

/// How a stored score relates to the true score of the position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bound {
    /// The exact score (a PV node).
    Exact,
    /// A lower bound: the search failed high (score >= beta).
    Lower,
    /// An upper bound: no move beat alpha (score <= alpha).
    Upper,
}

#[derive(Debug, Clone, Copy)]
pub struct TtEntry {
    pub key: u64,
    pub best_move: Option<Move>,
    pub score: i32,
    pub depth: i32,
    pub bound: Bound,
}

/// Fixed-size hash table of earlier search results, indexed by Zobrist
/// hash. Lets the search reuse work across transpositions and across
/// iterative-deepening iterations, and supplies the best move from the
/// previous visit for move ordering. Always-replace: the newest result
/// wins, which is simple and works well with iterative deepening.
pub struct TranspositionTable {
    entries: Vec<Option<TtEntry>>,
}

pub const DEFAULT_HASH_MB: usize = 16;

impl TranspositionTable {
    pub fn new(megabytes: usize) -> Self {
        let entry_size = std::mem::size_of::<Option<TtEntry>>();
        let count = (megabytes.max(1) * 1024 * 1024 / entry_size).max(1);
        TranspositionTable {
            entries: vec![None; count],
        }
    }

    pub fn clear(&mut self) {
        self.entries.fill(None);
    }

    fn index(&self, key: u64) -> usize {
        // Multiply-shift maps the full 64-bit key onto the table size
        // without needing a power-of-two length.
        ((key as u128 * self.entries.len() as u128) >> 64) as usize
    }

    pub fn probe(&self, key: u64) -> Option<TtEntry> {
        self.entries[self.index(key)].filter(|e| e.key == key)
    }

    pub fn store(&mut self, entry: TtEntry) {
        let i = self.index(entry.key);
        self.entries[i] = Some(entry);
    }
}
