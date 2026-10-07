# Lutra

A UCI chess engine written in Rust.

## Download and use

Binaries for each release are on the
[releases page](https://github.com/kareludras/lutra/releases):

| File                       | Platform              |
|----------------------------|-----------------------|
| `lutra-windows-x86_64.exe` | Windows, 64-bit       |
| `lutra-linux-x86_64`       | Linux, x86-64         |
| `lutra-macos-arm64`        | macOS, Apple Silicon  |

Lutra is a UCI engine with no board of its own. Load it into a chess GUI
such as [Cute Chess](https://cutechess.com), [Arena](http://www.playwitharena.de),
[BanksiaGUI](https://banksiagui.com) or [En Croissant](https://encroissant.org)
and add it as a new UCI engine.

Before the first run:

- Windows: SmartScreen may warn about an unknown publisher. Choose
  More info, then Run anyway.
- Linux: `chmod +x lutra-linux-x86_64`
- macOS: `chmod +x lutra-macos-arm64 && xattr -d com.apple.quarantine lutra-macos-arm64`

Options: `Hash` (MB, default 16). Running `lutra bench` from a terminal
prints a node count and speed, which is a quick check that the binary
works.

Intel Macs and other platforms can build from source with
`cargo build --release` (Rust 1.88 or newer); the binary is
`target/release/lutra`.

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
- [x] Classical evaluation (tapered PeSTO tables, pawn structure, mobility)
- [x] Advanced search heuristics (TT, move ordering, PVS, null move, LMR)
- [x] Automated Elo testing (GitHub Actions)

## Rating

About 1810 CCRL Blitz (95% CI 1770-1854), measured at commit 1485478.

400 game gauntlet against Stash versions with known CCRL Blitz ratings,
100 games per opponent, each opening played with both colors.

| Opponent     | CCRL | W-D-L    | Score |
|--------------|------|----------|-------|
| Stash 9.0.1  | 1271 | 88-10-2  | 93.0% |
| Stash 11.0.1 | 1687 | 52-8-40  | 56.0% |
| Stash 12.0   | 1883 | 35-7-58  | 38.5% |
| Stash 14.0   | 2059 | 30-10-60 | 35.0% |

fastchess 1.8.2, 10+0.1, 1 thread, 16 MB hash, openings from
`scripts/books/lutra-8ply.epd`. Stash was built from the upstream tags.
The rating is a maximum likelihood fit over all games with the opponent
ratings held fixed. CCRL plays 2'+1" on different hardware, so the number
is approximate.

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

### In CI

The `Elo` workflow (`.github/workflows/elo.yml`) runs on every pull request
that touches the engine: it plays the PR against `main` for 400 games at
4+0.04 and puts the Elo estimate, W/L/D and both bench counts in the job
summary. The check fails only on a clear regression, when even the
optimistic end of the 95% confidence interval scores below 50%; smaller
differences are reported, not enforced. Games are uploaded as an artifact.

Run it by hand from the Actions tab to test any two refs, play more games,
or run a full SPRT (`mode: sprt`) until fastchess accepts H0 or H1.
