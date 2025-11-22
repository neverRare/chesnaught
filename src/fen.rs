use std::{
    fmt::Display,
    iter::{Peekable, once, repeat},
    str::FromStr,
};

use crate::chess::{Board, Color, Piece, PieceKind, pawn_home_rank};

#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Fen(pub Board);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseFenError {
    NotEnoughSquaresOnRow,
    ExceedingSquaresOnRow,
    UnexpectedChar(char),
    UnexpectedEol,
}
impl FromStr for Fen {
    type Err = ParseFenError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut characters = s.chars();
        let mut board = [[None; 8]; 8];

        let mut x: u8 = 0;
        let mut y: u8 = 0;
        while x < 8 || y < 7 {
            if let Some(c) = characters.next() {
                if c == '/' {
                    if x == 8 {
                        x = 0;
                        y += 1;
                    } else {
                        return Err(ParseFenError::NotEnoughSquaresOnRow);
                    }
                } else if matches!(c, '1'..='8') {
                    x = x
                        .checked_add(c as u8 - b'0')
                        .ok_or(ParseFenError::ExceedingSquaresOnRow)?;
                    if x > 8 {
                        return Err(ParseFenError::ExceedingSquaresOnRow);
                    }
                } else {
                    let (color, kind) = match c {
                        'P' => (Color::White, PieceKind::Pawn),
                        'N' => (Color::White, PieceKind::Knight),
                        'B' => (Color::White, PieceKind::Bishop),
                        'R' => (Color::White, PieceKind::Rook),
                        'Q' => (Color::White, PieceKind::Queen),
                        'K' => (Color::White, PieceKind::King),
                        'p' => (Color::Black, PieceKind::Pawn),
                        'n' => (Color::Black, PieceKind::Knight),
                        'b' => (Color::Black, PieceKind::Bishop),
                        'r' => (Color::Black, PieceKind::Rook),
                        'q' => (Color::Black, PieceKind::Queen),
                        'k' => (Color::Black, PieceKind::King),
                        c => return Err(ParseFenError::UnexpectedChar(c)),
                    };
                    board[y as usize][x as usize] = Some(Piece {
                        color,
                        kind,
                        moved: kind != PieceKind::Pawn || pawn_home_rank(color) != y,
                        just_moved_twice_as_pawn: false,
                    });
                    x += 1;
                }
            } else {
                return Err(ParseFenError::UnexpectedEol);
            }
        }
        let space = characters.next().ok_or(ParseFenError::UnexpectedEol)?;
        if space != ' ' {
            return Err(ParseFenError::UnexpectedChar(space));
        }
        let current_player = match characters.next().ok_or(ParseFenError::UnexpectedEol)? {
            'w' | 'W' => Color::White,
            'b' | 'B' => Color::Black,
            c => return Err(ParseFenError::UnexpectedChar(c)),
        };
        let board = Board {
            current_player,
            board,
        };
        // TODO: handle castling and en passant
        Ok(Fen(board))
    }
}
impl Display for Fen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let board = self.0;
        for (row, first) in board.board.into_iter().zip(once(true).chain(repeat(false))) {
            if !first {
                write!(f, "/")?;
            }
            let mut pieces = row.into_iter().peekable();
            while let Some(piece) = pieces.next() {
                if let Some(piece) = piece {
                    write!(f, "{}", piece.fen())?;
                } else {
                    let mut count = 1;
                    while pieces.peek().is_some_and(Option::is_none) {
                        pieces.next();
                        count += 1;
                    }
                    write!(f, "{count}")?;
                }
            }
        }
        write!(f, " {}", board.current_player.lowercase())?;
        write!(f, " {}", board.castling_rights())?;
        if let Some(position) = board.en_passant_destinations().next() {
            write!(f, " {position}")?;
        } else {
            write!(f, " -")?;
        }
        write!(f, " 0 1")?;
        Ok(())
    }
}
