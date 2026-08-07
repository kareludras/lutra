use crate::board::Board;
use crate::castling::CastlingRights;
use crate::piece::Piece;
use crate::square::{Color, Square};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FenError {
    WrongFieldCount,
    InvalidPiecePlacement,
    InvalidSideToMove,
    InvalidCastlingRights,
    InvalidEnPassant,
    InvalidHalfmoveClock,
    InvalidFullmoveNumber,
}

impl fmt::Display for FenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            FenError::WrongFieldCount => "FEN must have exactly 6 space-separated fields",
            FenError::InvalidPiecePlacement => "invalid piece placement field",
            FenError::InvalidSideToMove => "side to move must be 'w' or 'b'",
            FenError::InvalidCastlingRights => "invalid castling rights field",
            FenError::InvalidEnPassant => "invalid en passant target square",
            FenError::InvalidHalfmoveClock => "invalid halfmove clock",
            FenError::InvalidFullmoveNumber => "invalid fullmove number",
        };
        write!(f, "{msg}")
    }
}

impl std::error::Error for FenError {}

impl Board {
    pub fn from_fen(fen: &str) -> Result<Board, FenError> {
        let fields: Vec<&str> = fen.split_whitespace().collect();
        if fields.len() != 6 {
            return Err(FenError::WrongFieldCount);
        }

        let mut board = Board::empty();
        parse_piece_placement(&mut board, fields[0])?;
        board.set_side_to_move(parse_side_to_move(fields[1])?);
        board.set_castling_rights(parse_castling_rights(fields[2])?);
        board.set_en_passant(parse_en_passant(fields[3])?);
        board.set_halfmove_clock(
            fields[4]
                .parse()
                .map_err(|_| FenError::InvalidHalfmoveClock)?,
        );
        board.set_fullmove_number(
            fields[5]
                .parse()
                .map_err(|_| FenError::InvalidFullmoveNumber)?,
        );

        Ok(board)
    }
}

fn parse_piece_placement(board: &mut Board, field: &str) -> Result<(), FenError> {
    let ranks: Vec<&str> = field.split('/').collect();
    if ranks.len() != 8 {
        return Err(FenError::InvalidPiecePlacement);
    }

    // FEN lists ranks from 8 down to 1; our rank index is 0-based from rank 1.
    for (rank_from_top, rank_str) in ranks.iter().enumerate() {
        let rank = 7 - rank_from_top as u8;
        let mut file = 0u8;

        for c in rank_str.chars() {
            if let Some(skip) = c.to_digit(10) {
                file += skip as u8;
            } else {
                let piece = Piece::from_char(c).ok_or(FenError::InvalidPiecePlacement)?;
                let color = if c.is_ascii_uppercase() {
                    Color::White
                } else {
                    Color::Black
                };
                if file >= 8 {
                    return Err(FenError::InvalidPiecePlacement);
                }
                board.put_piece(color, piece, Square::from_file_rank(file, rank));
                file += 1;
            }
        }

        if file != 8 {
            return Err(FenError::InvalidPiecePlacement);
        }
    }

    Ok(())
}

fn parse_side_to_move(field: &str) -> Result<Color, FenError> {
    match field {
        "w" => Ok(Color::White),
        "b" => Ok(Color::Black),
        _ => Err(FenError::InvalidSideToMove),
    }
}

fn parse_castling_rights(field: &str) -> Result<CastlingRights, FenError> {
    if field == "-" {
        return Ok(CastlingRights::NONE);
    }

    let mut rights = CastlingRights::NONE;
    for c in field.chars() {
        let flag = match c {
            'K' => CastlingRights::WHITE_KINGSIDE,
            'Q' => CastlingRights::WHITE_QUEENSIDE,
            'k' => CastlingRights::BLACK_KINGSIDE,
            'q' => CastlingRights::BLACK_QUEENSIDE,
            _ => return Err(FenError::InvalidCastlingRights),
        };
        rights.set(flag);
    }
    Ok(rights)
}

fn parse_en_passant(field: &str) -> Result<Option<Square>, FenError> {
    if field == "-" {
        return Ok(None);
    }
    Square::from_algebraic(field)
        .map(Some)
        .ok_or(FenError::InvalidEnPassant)
}
