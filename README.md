# Lutra

A UCI chess engine written in Rust.

## Progress

- [x] Board types (Square, Piece, Color, Bitboard)
- [x] Knight, king, and pawn attack tables
- [x] Sliding piece attacks (bishop, rook, queen)
- [x] Board struct with starting position
- [x] FEN parsing and serialization
- [x] Move type
- [x] Pseudo-legal knight, king, sliding, and pawn moves (pushes, captures, promotions, en passant)
- [x] Castling move generation
- [x] Make/unmake move
- [x] Legality filtering (king safety)
- [x] Perft validated against standard test positions
- [x] Basic alpha-beta search with quiescence and iterative deepening
- [x] Minimal UCI protocol support
- [x] Benchmarking harness (fastchess/SPRT)
- [ ] Classical evaluation
- [x] Advanced search heuristics (TT, move ordering, PVS, null move, LMR)
- [ ] Automated Elo testing
## Testing strength

Requires [fastchess](https://github.com/Disservin/fastchess/releases) and
Python 3. Point the scripts at fastchess with `--fastchess PATH` or the
`FASTCHESS` environment variable.

```sh
# SPRT: is the working tree at least 5 Elo stronger than main?
python scripts/sprt.py

# Compare two commits, or play a fixed number of games for an Elo estimate
python scripts/sprt.py --base HEAD~1 --dev HEAD
python scripts/sprt.py --games 400 --tc 4+0.04

# Check that moves are answered within their time budget
python scripts/timecheck.py target/release/lutra
```

`sprt.py` builds each git ref once (cached under `target/sprt/builds`) and
writes games, logs and fastchess's resume file to `target/sprt/runs/`.
Games start from `scripts/books/lutra-8ply.epd`, a set of roughly level
8-ply openings made by `lutra genbook 3000 8 2026`; pass `--book` to use a
standard book instead.

`lutra bench` searches a fixed set of positions to a fixed depth and prints
the total node count, which is deterministic: a change meant only to speed
up the engine must leave it unchanged.
