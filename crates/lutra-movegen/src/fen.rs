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

    pub fn to_fen(&self) -> String {
        let placement = piece_placement_to_fen(self);
        let side_to_move = match self.side_to_move() {
            Color::White => "w",
            Color::Black => "b",
        };
        let castling = castling_rights_to_fen(self.castling_rights());
        let en_passant = match self.en_passant() {
            Some(sq) => sq.to_algebraic(),
            None => "-".to_string(),
        };

        format!(
            "{placement} {side_to_move} {castling} {en_passant} {} {}",
            self.halfmove_clock(),
            self.fullmove_number()
        )
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

fn piece_placement_to_fen(board: &Board) -> String {
    let mut ranks = Vec::with_capacity(8);

    for rank in (0..8).rev() {
        let mut rank_str = String::new();
        let mut empty_run = 0u8;

        for file in 0..8 {
            let sq = Square::from_file_rank(file, rank);
            match board.piece_at(sq) {
                Some((color, piece)) => {
                    if empty_run > 0 {
                        rank_str.push_str(&empty_run.to_string());
                        empty_run = 0;
                    }
                    let c = piece.to_char();
                    rank_str.push(if color == Color::White {
                        c
                    } else {
                        c.to_ascii_lowercase()
                    });
                }
                None => empty_run += 1,
            }
        }
        if empty_run > 0 {
            rank_str.push_str(&empty_run.to_string());
        }
        ranks.push(rank_str);
    }

    ranks.join("/")
}

fn castling_rights_to_fen(rights: CastlingRights) -> String {
    let mut s = String::new();
    if rights.has(CastlingRights::WHITE_KINGSIDE) {
        s.push('K');
    }
    if rights.has(CastlingRights::WHITE_QUEENSIDE) {
        s.push('Q');
    }
    if rights.has(CastlingRights::BLACK_KINGSIDE) {
        s.push('k');
    }
    if rights.has(CastlingRights::BLACK_QUEENSIDE) {
        s.push('q');
    }
    if s.is_empty() {
        s.push('-');
    }
    s
}
