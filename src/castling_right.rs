use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use crate::{board::Piece, color::Color, coord::Coord, piece::PieceKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InvalidCastlingCharacter(pub char);

impl Display for InvalidCastlingCharacter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "found {}, expected one of `k`, `q`, letters from `a` to `h`, or uppercase forms of these letters",
            self.0
        )?;
        Ok(())
    }
}
impl Error for InvalidCastlingCharacter {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CastlingRight {
    white: u8,
    black: u8,
}
impl CastlingRight {
    pub fn none() -> Self {
        CastlingRight { white: 0, black: 0 }
    }
    pub fn from_configuration(configuration: [PieceKind; 8]) -> Self {
        let mut castling_right = CastlingRight::none();
        for (i, piece) in configuration.into_iter().enumerate() {
            if piece == PieceKind::Rook {
                castling_right.add(Color::White, i.try_into().unwrap());
                castling_right.add(Color::Black, i.try_into().unwrap());
            }
        }
        castling_right
    }
    pub fn all(self, color: Color) -> impl DoubleEndedIterator<Item = u8> {
        (0..8).filter(move |x| self.get(color, *x))
    }
    fn byte(self, color: Color) -> u8 {
        match color {
            Color::White => self.white,
            Color::Black => self.black,
        }
    }
    fn byte_mut(&mut self, color: Color) -> &mut u8 {
        match color {
            Color::White => &mut self.white,
            Color::Black => &mut self.black,
        }
    }
    pub fn get(self, color: Color, x: u8) -> bool {
        debug_assert!(x < 8, "{x} should be < 8");
        match (self.byte(color) >> x) & 0b_1 {
            0 => false,
            1 => true,
            _ => unreachable!(),
        }
    }
    pub fn add(&mut self, color: Color, x: u8) {
        debug_assert!(x < 8, "{x} should be < 8");
        let byte = self.byte_mut(color);
        *byte |= 0b_1 << x;
    }
    pub fn to_added(self, color: Color, x: u8) -> Self {
        let mut new = self;
        new.add(color, x);
        new
    }
    pub fn remove(&mut self, color: Color, x: u8) {
        debug_assert!(x < 8, "{x} should be < 8");
        let byte = self.byte_mut(color);
        *byte &= !(0b_1 << x);
    }
    pub fn to_removed(self, color: Color, x: u8) -> Self {
        let mut new = self;
        new.remove(color, x);
        new
    }
    pub fn clear(&mut self, color: Color) {
        let byte = self.byte_mut(color);
        *byte = 0;
    }
    pub fn to_cleared(self, color: Color) -> Self {
        let mut new = self;
        new.clear(color);
        new
    }
    pub fn standard_fen_display(self) -> StandardCastlingRight {
        StandardCastlingRight(self)
    }
    pub fn remove_for_rook_capture(&mut self, captured: Piece) {
        if captured.piece() == PieceKind::Rook
            && captured.position.y() == Coord::home_rank(captured.color())
        {
            self.remove(captured.color(), captured.position.x());
        }
    }
    pub fn to_removed_for_rook_capture(self, captured: Piece) -> Self {
        let mut new = self;
        new.remove_for_rook_capture(captured);
        new
    }
}
impl Display for CastlingRight {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut written = false;
        for color in [Color::White, Color::Black] {
            let start = match color {
                Color::White => b'A',
                Color::Black => b'a',
            };
            for x in self.all(color) {
                written = true;
                write!(f, "{}", (x + start) as char)?;
            }
        }
        if !written {
            write!(f, "-")?;
        }
        Ok(())
    }
}
impl FromStr for CastlingRight {
    type Err = InvalidCastlingCharacter;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut castling_right = CastlingRight::none();
        for c in s.chars() {
            match c {
                'Q' => castling_right.add(Color::White, Coord::ROOK_ORIGIN_QUEENSIDE),
                'K' => castling_right.add(Color::White, Coord::ROOK_ORIGIN_KINGSIDE),
                'q' => castling_right.add(Color::Black, Coord::ROOK_ORIGIN_QUEENSIDE),
                'k' => castling_right.add(Color::Black, Coord::ROOK_ORIGIN_KINGSIDE),
                'A'..='H' => castling_right.add(Color::White, c as u8 - b'A'),
                'a'..='h' => castling_right.add(Color::Black, c as u8 - b'a'),
                '-' => (),
                c => return Err(InvalidCastlingCharacter(c)),
            }
        }
        Ok(castling_right)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StandardCastlingRight(pub CastlingRight);
impl From<CastlingRight> for StandardCastlingRight {
    fn from(value: CastlingRight) -> Self {
        StandardCastlingRight(value)
    }
}
impl From<StandardCastlingRight> for CastlingRight {
    fn from(value: StandardCastlingRight) -> Self {
        value.0
    }
}
impl Display for StandardCastlingRight {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut written = false;
        for color in [Color::White, Color::Black] {
            for x in self.0.all(color).rev() {
                let c = match (color, x) {
                    (Color::White, Coord::ROOK_ORIGIN_QUEENSIDE) => 'Q',
                    (Color::White, Coord::ROOK_ORIGIN_KINGSIDE) => 'K',
                    (Color::Black, Coord::ROOK_ORIGIN_QUEENSIDE) => 'q',
                    (Color::Black, Coord::ROOK_ORIGIN_KINGSIDE) => 'k',
                    (color, x) => {
                        let start = match color {
                            Color::White => b'A',
                            Color::Black => b'a',
                        };
                        (x + start) as char
                    }
                };
                written = true;
                write!(f, "{c}")?;
            }
        }
        if !written {
            write!(f, "-")?;
        }
        Ok(())
    }
}
impl FromStr for StandardCastlingRight {
    type Err = InvalidCastlingCharacter;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(StandardCastlingRight)
    }
}
