use lutra_movegen::Board;

const STARTPOS_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

fn assert_round_trips(fen: &str) {
    let board = Board::from_fen(fen).expect("valid FEN should parse");
    assert_eq!(board.to_fen(), fen);
}

#[test]
fn startpos_round_trips() {
    assert_round_trips(STARTPOS_FEN);
}

#[test]
fn position_with_captures_and_gaps_round_trips() {
    assert_round_trips("rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3");
}

#[test]
fn position_with_no_castling_rights_round_trips() {
    assert_round_trips("8/8/8/4k3/8/8/8/4K3 w - - 0 1");
}

#[test]
fn position_with_partial_castling_rights_round_trips() {
    assert_round_trips("r3k2r/8/8/8/8/8/8/R3K2R w Kq - 0 1");
}

#[test]
fn position_with_black_to_move_round_trips() {
    assert_round_trips("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1");
}

#[test]
fn position_with_large_move_counters_round_trips() {
    assert_round_trips("8/8/8/4k3/8/8/8/4K3 w - - 45 123");
}

#[test]
fn to_fen_on_hardcoded_starting_position_matches_standard_fen() {
    let board = Board::starting_position();
    assert_eq!(board.to_fen(), STARTPOS_FEN);
}
