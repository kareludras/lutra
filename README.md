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
- [ ] Minimal UCI protocol support
- [ ] Benchmarking harness (fastchess/SPRT)
- [ ] Classical evaluation
- [ ] Advanced search heuristics
- [ ] Automated Elo testing