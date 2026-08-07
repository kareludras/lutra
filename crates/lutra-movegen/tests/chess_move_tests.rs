use lutra_movegen::{Move, Piece, Square};

#[test]
fn quiet_move_displays_as_uci_long_algebraic() {
    let mv = Move::new(
        Square::from_algebraic("e2").unwrap(),
        Square::from_algebraic("e4").unwrap(),
        Piece::Pawn,
    );
    assert_eq!(mv.to_string(), "e2e4");
    assert!(!mv.is_capture());
}

#[test]
fn promotion_move_displays_with_lowercase_promo_letter() {
    let mv = Move::new(
        Square::from_algebraic("e7").unwrap(),
        Square::from_algebraic("e8").unwrap(),
        Piece::Pawn,
    )
    .with_promotion(Piece::Queen);
    assert_eq!(mv.to_string(), "e7e8q");
}

#[test]
fn capture_move_reports_is_capture() {
    let mv = Move::new(
        Square::from_algebraic("e4").unwrap(),
        Square::from_algebraic("d5").unwrap(),
        Piece::Pawn,
    )
    .with_capture(Piece::Pawn);
    assert!(mv.is_capture());
    assert_eq!(mv.captured, Some(Piece::Pawn));
}

#[test]
fn en_passant_move_reports_is_capture_even_without_captured_field() {
    let mv = Move::new(
        Square::from_algebraic("e5").unwrap(),
        Square::from_algebraic("d6").unwrap(),
        Piece::Pawn,
    )
    .as_en_passant();
    assert!(mv.is_capture());
    assert_eq!(mv.captured, None);
    assert!(mv.is_en_passant);
}
