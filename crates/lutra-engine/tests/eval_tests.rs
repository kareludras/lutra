use lutra_engine::{bench::BENCH_FENS, evaluate};
use lutra_movegen::{Board, Color, Piece, Square};

/// The same position with the board flipped vertically and the colours
/// swapped, from the other side's point of view. A correct evaluation
/// scores it identically (the side to move is equally well off).
fn mirror_fen(fen: &str) -> String {
    let fields: Vec<&str> = fen.split_whitespace().collect();
    let swap_case = |s: &str| -> String {
        s.chars()
            .map(|c| {
                if c.is_ascii_uppercase() {
                    c.to_ascii_lowercase()
                } else {
                    c.to_ascii_uppercase()
                }
            })
            .collect()
    };
    let placement: Vec<String> = fields[0].split('/').rev().map(swap_case).collect();
    let side = if fields[1] == "w" { "b" } else { "w" };
    let castling = if fields[2] == "-" {
        "-".to_string()
    } else {
        // Keep FEN order: uppercase (white) rights first.
        let swapped = swap_case(fields[2]);
        let (mut upper, mut lower): (String, String) = (String::new(), String::new());
        for c in swapped.chars() {
            if c.is_ascii_uppercase() {
                upper.push(c)
            } else {
                lower.push(c)
            }
        }
        upper + &lower
    };
    let ep = if fields[3] == "-" {
        "-".to_string()
    } else {
        let mut chars = fields[3].chars();
        let file = chars.next().unwrap();
        let rank = chars.next().unwrap().to_digit(10).unwrap();
        format!("{file}{}", 9 - rank)
    };
    let mut out = vec![placement.join("/"), side.to_string(), castling, ep];
    out.extend(fields[4..].iter().map(|s| s.to_string()));
    out.join(" ")
}

fn book_fens() -> Vec<String> {
    let book = include_str!("../../../scripts/books/lutra-8ply.epd");
    book.lines()
        .filter(|l| !l.trim().is_empty())
        .map(str::to_string)
        .collect()
}

#[test]
fn evaluation_is_mirror_symmetric() {
    let fens: Vec<String> = BENCH_FENS
        .iter()
        .map(|f| f.to_string())
        .chain(book_fens())
        .collect();
    for fen in &fens {
        let board = Board::from_fen(fen).unwrap();
        let mirrored = Board::from_fen(&mirror_fen(fen)).unwrap();
        assert_eq!(
            evaluate(&board),
            evaluate(&mirrored),
            "asymmetric eval for {fen}"
        );
    }
}

#[test]
fn starting_position_is_balanced() {
    assert_eq!(evaluate(&Board::starting_position()), 0);
}

#[test]
fn side_to_move_flips_the_sign() {
    let white = Board::from_fen("4k3/8/8/8/8/8/8/R3K3 w - - 0 1").unwrap();
    let black = Board::from_fen("4k3/8/8/8/8/8/8/R3K3 b - - 0 1").unwrap();
    assert_eq!(evaluate(&white), -evaluate(&black));
}

#[test]
fn missing_queen_evaluates_strongly_negative_for_side_to_move() {
    let mut board = Board::starting_position();
    let d1 = Square::from_algebraic("d1").unwrap();
    board.remove_piece(Color::White, Piece::Queen, d1);
    assert!(evaluate(&board) < -800);
}

#[test]
fn extra_rook_evaluates_positive_for_side_to_move() {
    let board = Board::from_fen("4k3/8/8/8/8/8/8/R3K3 w - - 0 1").unwrap();
    assert!(evaluate(&board) > 400);
}

#[test]
fn extra_rook_for_black_is_positive_when_black_is_to_move() {
    let board = Board::from_fen("r3k3/8/8/8/8/8/8/4K3 b - - 0 1").unwrap();
    assert!(evaluate(&board) > 400);
}

#[test]
fn bare_kings_on_mirrored_squares_evaluate_to_zero() {
    let board = Board::from_fen("4k3/8/8/8/8/8/8/4K3 w - - 0 1").unwrap();
    assert_eq!(evaluate(&board), 0);
}

#[test]
fn king_belongs_in_the_corner_in_the_middlegame_and_the_centre_in_the_endgame() {
    // Same material, king castled vs. king wandering to the centre.
    let castled = Board::from_fen("rnbq1rk1/pppppppp/8/8/8/8/PPPPPPPP/RNBQ1RK1 w - - 0 1").unwrap();
    let wandering =
        Board::from_fen("rnbq1rk1/pppppppp/8/8/4K3/8/PPPPPPPP/RNBQ1R2 w - - 0 1").unwrap();
    assert!(evaluate(&castled) > evaluate(&wandering));

    let corner = Board::from_fen("6k1/8/8/8/8/8/P7/K7 w - - 0 1").unwrap();
    let centre = Board::from_fen("6k1/8/8/8/3K4/8/P7/8 w - - 0 1").unwrap();
    assert!(evaluate(&centre) > evaluate(&corner));
}

#[test]
fn advanced_pawns_are_worth_more_in_the_endgame() {
    let home = Board::from_fen("4k3/8/8/8/8/8/P7/4K3 w - - 0 1").unwrap();
    let advanced = Board::from_fen("4k3/P7/8/8/8/8/8/4K3 w - - 0 1").unwrap();
    assert!(evaluate(&advanced) > evaluate(&home) + 50);
}

#[test]
fn mirror_fen_round_trips() {
    for fen in BENCH_FENS {
        assert_eq!(mirror_fen(&mirror_fen(fen)), fen);
    }
}
