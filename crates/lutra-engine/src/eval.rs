use lutra_movegen::{Board, Piece};

/// Standard centipawn material values. King is excluded (never captured;
/// its "value" is handled separately once king safety terms exist).
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

/// Material balance from the perspective of the side to move: positive
/// means the side to move has more material, negative means less.
pub fn evaluate(board: &Board) -> i32 {
    let side = board.side_to_move();
    let opponent = side.opposite();

    Piece::ALL
        .iter()
        .map(|&piece| {
            let value = material_value(piece);
            let own = board.pieces(side, piece).count() as i32;
            let their = board.pieces(opponent, piece).count() as i32;
            (own - their) * value
        })
        .sum()
}
