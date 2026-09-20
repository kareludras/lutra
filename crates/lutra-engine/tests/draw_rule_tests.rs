use lutra_engine::{SearchLimits, iterative_deepening, iterative_deepening_with_history};
use lutra_movegen::{Board, generate_legal_moves};

#[test]
fn losing_side_scores_a_draw_when_every_reply_repeats_the_game() {
    // Black (bare king) is lost on material, but every legal reply leads
    // to a position that already occurred earlier in the game with the
    // same side to move, so each reply is a draw by repetition.
    let board = Board::from_fen("7k/8/5K2/8/8/8/8/Q7 b - - 20 60").unwrap();

    let without_history =
        iterative_deepening(&board, SearchLimits::depth(3)).expect("black has legal moves");
    assert!(without_history.score < -500, "black should be lost");

    // History is oldest first and ends just before `board`. A reply (white
    // to move) can only repeat a white-to-move position, i.e. every other
    // entry counting back from the end, so pad with filler for the
    // black-to-move positions in between.
    let mut history = Vec::new();
    for mv in generate_legal_moves(&board, board.side_to_move()) {
        history.push(1); // filler: some black-to-move position
        history.push(board.make_move(mv).hash());
    }
    let result = iterative_deepening_with_history(&board, &history, SearchLimits::depth(3))
        .expect("black has legal moves");
    assert_eq!(result.score, 0);
}

#[test]
fn fifty_move_rule_scores_quiet_continuations_as_draws() {
    // Queen up, but 99 half-moves without a capture or pawn move: any quiet
    // move completes the fifty-move rule. There is no mate in one, so the
    // best White can get is a draw.
    let board = Board::from_fen("8/8/8/7k/3Q4/8/8/K7 w - - 99 120").unwrap();
    let result = iterative_deepening(&board, SearchLimits::depth(3)).expect("white has moves");
    assert_eq!(result.score, 0);

    let fresh = Board::from_fen("8/8/8/7k/3Q4/8/8/K7 w - - 0 120").unwrap();
    let result = iterative_deepening(&fresh, SearchLimits::depth(3)).expect("white has moves");
    assert!(result.score > 500);
}

#[test]
fn winning_side_avoids_a_move_that_repeats_the_game() {
    // White to move is a queen up. Pretend the position after Qd4-d1 is the
    // one Black just moved out of: playing it again would throw the win
    // away by repetition.
    let board = Board::from_fen("8/8/8/7k/3Q4/8/8/K7 w - - 10 40").unwrap();
    let repeat_move = generate_legal_moves(&board, board.side_to_move())
        .into_iter()
        .find(|m| m.to_string() == "d4d1")
        .unwrap();
    let history = vec![board.make_move(repeat_move).hash()];
    let result = iterative_deepening_with_history(&board, &history, SearchLimits::depth(3))
        .expect("white has moves");
    assert_ne!(result.best_move, repeat_move);
    assert!(result.score > 500);
}
