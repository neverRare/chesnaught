use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    iter::{from_fn, once, repeat},
    num::ParseIntError,
    ops::{Index, IndexMut},
    str::FromStr,
};

use crate::{
    board_display::IndexableBoard,
    chess::{
        Color, ColoredPieceKind, Coord, HashableBoard, InvalidCastlingCharacter, InvalidFenPiece,
        ParseColorError, ParseCoordError, PieceKind,
    },
    coord_x, coord_y,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseFenError {
    ExceededRowCount,
    ExceededSquareCount,
    InvalidRowCount(usize),
    InvalidSquareCount(usize),
    InvalidSpaceCharacter(char),
    InvalidFenPiece(InvalidFenPiece),
    ParseColorError(ParseColorError),
    InvalidCastlingCharacter(InvalidCastlingCharacter),
    ParseCoordError(ParseCoordError),
    ParseIntError(ParseIntError),
    Unexpected(char),
    UnexpectedEol,
}
impl Display for ParseFenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ParseFenError::ExceededRowCount => {
                write!(f, "exceeded number of rows, 8 were expected")?;
            }
            ParseFenError::ExceededSquareCount => {
                write!(f, "exceeded number of squares, 8 were expected")?;
            }
            ParseFenError::InvalidRowCount(rows) => {
                write!(f, "found {rows} rows, 8 were expected instead")?;
            }
            ParseFenError::InvalidSquareCount(squares) => {
                write!(f, "found {squares} squares, 8 were expected instead")?;
            }
            ParseFenError::InvalidSpaceCharacter(c) => {
                write!(f, "found {c}, numbers from 1 to 8 were expected instead")?;
            }
            ParseFenError::InvalidFenPiece(err) => write!(f, "{err}")?,
            ParseFenError::ParseColorError(err) => write!(f, "{err}")?,
            ParseFenError::InvalidCastlingCharacter(err) => write!(f, "{err}")?,
            ParseFenError::ParseCoordError(err) => write!(f, "{err}")?,
            ParseFenError::ParseIntError(err) => write!(f, "{err}")?,
            ParseFenError::Unexpected(c) => write!(f, "unexpected `{c}`")?,
            ParseFenError::UnexpectedEol => write!(f, "unexpected end of line")?,
        }
        Ok(())
    }
}
impl Error for ParseFenError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ParseFenError::InvalidFenPiece(err) => Some(err),
            ParseFenError::ParseColorError(err) => Some(err),
            ParseFenError::InvalidCastlingCharacter(err) => Some(err),
            ParseFenError::ParseCoordError(err) => Some(err),
            ParseFenError::ParseIntError(err) => Some(err),
            _ => None,
        }
    }
}
impl From<InvalidFenPiece> for ParseFenError {
    fn from(value: InvalidFenPiece) -> Self {
        ParseFenError::InvalidFenPiece(value)
    }
}
impl From<ParseColorError> for ParseFenError {
    fn from(value: ParseColorError) -> Self {
        ParseFenError::ParseColorError(value)
    }
}
impl From<InvalidCastlingCharacter> for ParseFenError {
    fn from(value: InvalidCastlingCharacter) -> Self {
        ParseFenError::InvalidCastlingCharacter(value)
    }
}
impl From<ParseCoordError> for ParseFenError {
    fn from(value: ParseCoordError) -> Self {
        ParseFenError::ParseCoordError(value)
    }
}
impl From<ParseIntError> for ParseFenError {
    fn from(value: ParseIntError) -> Self {
        ParseFenError::ParseIntError(value)
    }
}
pub struct Fen {
    pub board: HashableBoard,
    pub half_move: u32,
    pub full_move: u32,
}
impl Display for Fen {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (first, row) in once(true)
            .chain(repeat(false))
            .zip(self.board.board.into_iter())
        {
            enum Item {
                Piece(ColoredPieceKind),
                Space(u8),
            }
            if !first {
                write!(f, "/")?;
            }
            let mut row = row.into_iter().peekable();
            let items = from_fn(|| {
                row.next().map(|piece| {
                    if let Some(piece) = piece {
                        Item::Piece(piece)
                    } else {
                        let mut count = 1;
                        while let Some(None) = row.peek() {
                            count += 1;
                            row.next();
                        }
                        Item::Space(count)
                    }
                })
            });
            for item in items {
                match item {
                    Item::Piece(piece) => write!(f, "{}", piece.fen())?,
                    Item::Space(space) => write!(f, "{space}")?,
                }
            }
        }
        write!(f, " {}", self.board.current_player)?;
        let use_standard_castling = [Color::White, Color::Black].into_iter().all(|color| {
            let row = match color {
                Color::White => self.board.board[coord_y!("1")],
                Color::Black => self.board.board[coord_y!("8")],
            };
            let king_in_position = row
                .into_iter()
                .position(|piece| piece == Some(ColoredPieceKind::new(color, PieceKind::King)))
                == Some(coord_x!("e"));
            self.board.castling_right.all(color).all(|rook| {
                if king_in_position {
                    let range = match rook {
                        coord_x!("a") => coord_x!("b")..=coord_x!("d"),
                        coord_x!("h") => coord_x!("f")..=coord_x!("g"),
                        _ => return false,
                    };
                    !range.into_iter().any(|x| {
                        let x: usize = x.try_into().unwrap();
                        row[x] == Some(ColoredPieceKind::new(color, PieceKind::Rook))
                    })
                } else {
                    false
                }
            })
        });
        if use_standard_castling {
            write!(f, " {}", self.board.castling_right.standard_fen_display())?;
        } else {
            write!(f, " {}", self.board.castling_right)?;
        }
        if let Some(en_passant_target) = self.board.en_passant_target {
            write!(f, " {en_passant_target}")?;
        } else {
            write!(f, " -")?;
        }
        write!(f, " {} {}", self.half_move, self.full_move)?;
        Ok(())
    }
}
impl FromStr for Fen {
    type Err = ParseFenError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut sections = s.split(' ');

        let board = parse_board(sections.next().ok_or(ParseFenError::UnexpectedEol)?)?;

        let current_player = sections
            .next()
            .ok_or(ParseFenError::UnexpectedEol)?
            .parse()?;

        let castling_right = sections
            .next()
            .ok_or(ParseFenError::UnexpectedEol)?
            .parse()?;

        let en_passant_target = sections.next().ok_or(ParseFenError::UnexpectedEol)?;
        let en_passant_target = (en_passant_target != "-")
            .then(|| en_passant_target.parse())
            .transpose()?;

        let half_move = sections
            .next()
            .ok_or(ParseFenError::UnexpectedEol)?
            .parse()?;

        let full_move = sections
            .next()
            .ok_or(ParseFenError::UnexpectedEol)?
            .parse()?;

        if let Some(section) = sections.next() {
            return Err(ParseFenError::Unexpected(
                section.chars().next().unwrap_or(' '),
            ));
        }
        Ok(Fen {
            board: HashableBoard {
                board,
                current_player,
                castling_right,
                en_passant_target,
            },
            half_move,
            full_move,
        })
    }
}
impl Index<Coord> for Fen {
    type Output = Option<ColoredPieceKind>;

    fn index(&self, index: Coord) -> &Self::Output {
        &self.board[index]
    }
}
impl IndexMut<Coord> for Fen {
    fn index_mut(&mut self, index: Coord) -> &mut Self::Output {
        &mut self.board[index]
    }
}
impl IndexableBoard for Fen {
    fn index(&self, position: Coord) -> Option<ColoredPieceKind> {
        self[position]
    }
}
fn parse_board(src: &str) -> Result<[[Option<ColoredPieceKind>; 8]; 8], ParseFenError> {
    let mut board = [[None; 8]; 8];
    let mut last_y = 0;
    for (y, row) in src.split('/').enumerate() {
        if y >= 8 {
            return Err(ParseFenError::ExceededRowCount);
        }
        let mut x = 0;
        for c in row.chars() {
            if matches!(c, '0' | '9') {
                return Err(ParseFenError::InvalidSpaceCharacter(c));
            } else if matches!(c, '1'..='8') {
                x += (c as u8 - b'0') as usize;
            } else if x >= 8 {
                return Err(ParseFenError::ExceededSquareCount);
            } else {
                board[y][x] = Some(ColoredPieceKind::from_fen(c)?);
                x += 1;
            }
        }
        if x < 8 {
            return Err(ParseFenError::InvalidSquareCount(x));
        }
        last_y = y + 1;
    }
    if last_y < 8 {
        return Err(ParseFenError::InvalidRowCount(last_y));
    }
    Ok(board)
}
