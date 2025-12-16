use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    num::NonZero,
};

use crate::{color::Color, misc::InvalidByte};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PieceKind {
    // other types relies on `PieceKind` being non-zero
    Pawn = 1,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}
impl PieceKind {
    pub const PROMOTION_CHOICES: [Self; 4] = [
        PieceKind::Queen,
        PieceKind::Rook,
        PieceKind::Bishop,
        PieceKind::Knight,
    ];
    pub const STARTING_CONFIGURATION: [Self; 8] = [
        PieceKind::Rook,
        PieceKind::Knight,
        PieceKind::Bishop,
        PieceKind::Queen,
        PieceKind::King,
        PieceKind::Bishop,
        PieceKind::Knight,
        PieceKind::Rook,
    ];
    pub fn chess960(id: u16) -> [Self; 8] {
        fn get_position(configuration: &[PieceKind], mut index: u16) -> usize {
            for (i, cell) in configuration.iter().enumerate() {
                if *cell == PieceKind::Pawn {
                    if index == 0 {
                        return i;
                    }
                    index -= 1;
                }
            }
            unreachable!()
        }
        assert!(id < 960, "{id} must be < 960");

        let mut state = id;
        let bishop_1 = state % 4;
        state /= 4;

        let bishop_2 = state % 4;
        state /= 4;

        let queen = state % 6;
        state /= 6;

        let knights = state;

        let mut configuration = [PieceKind::Pawn; 8];
        configuration[bishop_1 as usize * 2] = PieceKind::Bishop;
        configuration[bishop_2 as usize * 2 + 1] = PieceKind::Bishop;
        configuration[get_position(&configuration, queen)] = PieceKind::Queen;

        let (a, b) = match knights {
            n @ 0..4 => (n, 4),
            n @ 4..7 => (n - 4, 3),
            n @ 7..9 => (n - 7, 2),
            9 => (0, 1),
            _ => unreachable!(),
        };
        let a = get_position(&configuration, a);
        let b = get_position(&configuration, b);
        configuration[a] = PieceKind::Knight;
        configuration[b] = PieceKind::Knight;

        let mut piece = PieceKind::Rook;
        for cell in &mut configuration {
            if *cell == PieceKind::Pawn {
                *cell = piece;
                piece = match piece {
                    PieceKind::King => PieceKind::Rook,
                    PieceKind::Rook => PieceKind::King,
                    _ => unreachable!(),
                };
            }
        }
        configuration
    }
    pub fn uppercase(self) -> char {
        match self {
            PieceKind::Pawn => 'P',
            PieceKind::Knight => 'N',
            PieceKind::Bishop => 'B',
            PieceKind::Rook => 'R',
            PieceKind::Queen => 'Q',
            PieceKind::King => 'K',
        }
    }
    pub fn lowercase(self) -> char {
        match self {
            PieceKind::Pawn => 'p',
            PieceKind::Knight => 'n',
            PieceKind::Bishop => 'b',
            PieceKind::Rook => 'r',
            PieceKind::Queen => 'q',
            PieceKind::King => 'k',
        }
    }
    pub fn from_fen(c: char) -> Result<Self, InvalidFenPiece> {
        let piece = match c {
            'p' | 'P' => PieceKind::Pawn,
            'n' | 'N' => PieceKind::Knight,
            'b' | 'B' => PieceKind::Bishop,
            'r' | 'R' => PieceKind::Rook,
            'q' | 'Q' => PieceKind::Queen,
            'k' | 'K' => PieceKind::King,
            c => return Err(InvalidFenPiece(c)),
        };
        Ok(piece)
    }
}
impl Display for PieceKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            PieceKind::Pawn => write!(f, "pawn")?,
            PieceKind::Knight => write!(f, "knight")?,
            PieceKind::Bishop => write!(f, "bishop")?,
            PieceKind::Rook => write!(f, "rook")?,
            PieceKind::Queen => write!(f, "queen")?,
            PieceKind::King => write!(f, "king")?,
        }
        Ok(())
    }
}
impl TryFrom<u8> for PieceKind {
    type Error = InvalidByte;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let piece = match value {
            0 | 7.. => return Err(InvalidByte),
            1 => PieceKind::Pawn,
            2 => PieceKind::Knight,
            3 => PieceKind::Bishop,
            4 => PieceKind::Rook,
            5 => PieceKind::Queen,
            6 => PieceKind::King,
        };
        Ok(piece)
    }
}
impl From<PieceKind> for u8 {
    fn from(value: PieceKind) -> Self {
        match value {
            PieceKind::Pawn => 1,
            PieceKind::Knight => 2,
            PieceKind::Bishop => 3,
            PieceKind::Rook => 4,
            PieceKind::Queen => 5,
            PieceKind::King => 6,
        }
    }
}

// Bit structure: 0000CPPP
// C - Color
// P - Piece kind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColoredPieceKind(NonZero<u8>);
impl ColoredPieceKind {
    pub fn new(color: Color, piece: PieceKind) -> Self {
        let color: u8 = color.into();
        let piece: u8 = piece.into();
        let data = (color << 3) | (piece);
        ColoredPieceKind(NonZero::new(data).unwrap())
    }
    pub fn color(self) -> Color {
        ((self.0.get() >> 3) & 0b_1).try_into().unwrap()
    }
    pub fn piece(self) -> PieceKind {
        (self.0.get() & 0b_111).try_into().unwrap()
    }
    pub fn fen(self) -> char {
        match self.color() {
            Color::White => self.piece().uppercase(),
            Color::Black => self.piece().lowercase(),
        }
    }
    pub fn from_fen(c: char) -> Result<Self, InvalidFenPiece> {
        let piece = PieceKind::from_fen(c)?;
        let color = if c.is_ascii_uppercase() {
            Color::White
        } else {
            Color::Black
        };
        Ok(ColoredPieceKind::new(color, piece))
    }
    pub fn figurine(self) -> char {
        match (self.color(), self.piece()) {
            (Color::White, PieceKind::Pawn) => '♙',
            (Color::White, PieceKind::Knight) => '♘',
            (Color::White, PieceKind::Bishop) => '♗',
            (Color::White, PieceKind::Rook) => '♖',
            (Color::White, PieceKind::Queen) => '♕',
            (Color::White, PieceKind::King) => '♔',
            (Color::Black, PieceKind::Pawn) => '♟',
            (Color::Black, PieceKind::Knight) => '♞',
            (Color::Black, PieceKind::Bishop) => '♝',
            (Color::Black, PieceKind::Rook) => '♜',
            (Color::Black, PieceKind::Queen) => '♛',
            (Color::Black, PieceKind::King) => '♚',
        }
    }
}
impl Display for ColoredPieceKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.color(), self.piece())?;
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InvalidFenPiece(pub char);
impl Display for InvalidFenPiece {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "found `{}`, expected one of `p`, `n`, `b`, `r`, `k`, `q`, or uppercase forms of these letters",
            self.0
        )?;
        Ok(())
    }
}
impl Error for InvalidFenPiece {}
