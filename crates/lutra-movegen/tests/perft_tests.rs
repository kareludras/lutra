use lutra_movegen::{Board, perft};

struct PerftCase {
    name: &'static str,
    fen: &'static str,
    depth: u32,
    expected: u64,
}

fn run_case(case: &PerftCase) {
    let board = Board::from_fen(case.fen).expect("valid FEN");
    let nodes = perft(&board, case.depth);
    assert_eq!(
        nodes, case.expected,
        "{} failed at depth {}: got {}, expected {}",
        case.name, case.depth, nodes, case.expected
    );
}

#[test]
fn perft_position_1_startpos() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    for (depth, expected) in [(1, 20), (2, 400), (3, 8902), (4, 197281), (5, 4865609)] {
        run_case(&PerftCase {
            name: "position 1 (startpos)",
            fen,
            depth,
            expected,
        });
    }
}

#[test]
fn perft_position_2_kiwipete() {
    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    for (depth, expected) in [(1, 48), (2, 2039), (3, 97862), (4, 4085603)] {
        run_case(&PerftCase {
            name: "position 2 (kiwipete)",
            fen,
            depth,
            expected,
        });
    }
}

#[test]
fn perft_position_3() {
    let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
    for (depth, expected) in [(1, 14), (2, 191), (3, 2812), (4, 43238), (5, 674624)] {
        run_case(&PerftCase {
            name: "position 3",
            fen,
            depth,
            expected,
        });
    }
}

#[test]
fn perft_position_4() {
    let fen = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
    for (depth, expected) in [(1, 6), (2, 264), (3, 9467), (4, 422333)] {
        run_case(&PerftCase {
            name: "position 4",
            fen,
            depth,
            expected,
        });
    }
}

#[test]
fn perft_position_5() {
    let fen = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
    for (depth, expected) in [(1, 44), (2, 1486), (3, 62379), (4, 2103487)] {
        run_case(&PerftCase {
            name: "position 5",
            fen,
            depth,
            expected,
        });
    }
}

#[test]
fn perft_position_6() {
    let fen = "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10";
    for (depth, expected) in [(1, 46), (2, 2079), (3, 89890), (4, 3894594)] {
        run_case(&PerftCase {
            name: "position 6",
            fen,
            depth,
            expected,
        });
    }
}
