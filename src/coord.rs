use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    num::NonZero,
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
    str::FromStr,
};

use crate::{color::Color, coord_y, error::InvalidByte};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParseCoordError {
    InvalidX(char),
    InvalidY(char),
    NotEnoughCharacter(u8),
    Unexpected(char),
}
impl Display for ParseCoordError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ParseCoordError::InvalidX(x) => write!(
                f,
                "found `{x}`, characters from `a` to `h` were expected instead"
            )?,
            ParseCoordError::InvalidY(y) => write!(
                f,
                "found `{y}`, characters from `1` to `8` were expected instead"
            )?,
            ParseCoordError::NotEnoughCharacter(len) => write!(
                f,
                "provided string have length of {len} characters, 2 were expected"
            )?,
            ParseCoordError::Unexpected(c) => write!(f, "unexpected `{c}`")?,
        }
        Ok(())
    }
}
impl Error for ParseCoordError {}

// Bit structure: 10XXXYYY
// first two bits is always `10` for `NonZero` size optimizations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Coord(NonZero<u8>);

impl Coord {
    pub fn new(x: u8, y: u8) -> Self {
        debug_assert!(x < 8);
        debug_assert!(y < 8);
        let byte = 0b1000_0000 | (x << 3) | y;
        Coord(NonZero::new(byte).unwrap())
    }
    pub fn from_chars(x: char, y: char) -> Result<Self, ParseCoordError> {
        let x = match x {
            'a'..='h' => x as u8 - b'a',
            _ => return Err(ParseCoordError::InvalidX(x)),
        };
        let y = match y {
            '1'..='8' => 7 - (y as u8 - b'1'),
            _ => return Err(ParseCoordError::InvalidY(y)),
        };
        Ok(Coord::new(x, y))
    }
    pub fn new_checked(x: u8, y: u8) -> Option<Self> {
        if x >= 8 || y >= 8 {
            None
        } else {
            Some(Self::new(x, y))
        }
    }
    pub fn x(self) -> u8 {
        (self.0.get() >> 3) & 0b_111
    }
    pub fn y(self) -> u8 {
        self.0.get() & 0b_111
    }
    pub fn move_by(self, movement: Vector) -> Option<Self> {
        Self::new_checked(
            self.x().checked_add_signed(movement.x)?,
            self.y().checked_add_signed(movement.y)?,
        )
    }
    pub fn is_aligned(
        self,
        other: Self,
        directions: &[Vector],
    ) -> Option<impl Iterator<Item = Self>> {
        directions.iter().copied().find_map(|direction| {
            if direction.is_aligned(other - self) {
                Some(self.line_exclusive_exclusive(other, direction))
            } else {
                None
            }
        })
    }
    pub fn is_aligned_with_bishop(self, other: Self) -> Option<impl Iterator<Item = Self>> {
        self.is_aligned(other, &Vector::BISHOP_DIRECTIONS)
    }
    pub fn is_aligned_with_rook(self, other: Self) -> Option<impl Iterator<Item = Self>> {
        self.is_aligned(other, &Vector::ROOK_DIRECTIONS)
    }
    pub fn is_aligned_with_queen(self, other: Self) -> Option<impl Iterator<Item = Self>> {
        self.is_aligned(other, &Vector::QUEEN_DIRECTIONS)
    }
    pub fn line(self, direction: Vector, start: i8) -> impl Iterator<Item = Self> {
        debug_assert_ne!(direction, Vector::ZERO);
        debug_assert_eq!(direction, direction.as_unit());
        (start..).map_while(move |difference| self.move_by(direction * difference))
    }
    pub fn line_inclusive(self, direction: Vector) -> impl Iterator<Item = Self> {
        self.line(direction, 0)
    }
    pub fn line_exclusive(self, direction: Vector) -> impl Iterator<Item = Self> {
        self.line(direction, 1)
    }
    pub fn line_inclusive_exclusive(
        self,
        end: Coord,
        direction: Vector,
    ) -> impl Iterator<Item = Self> {
        self.line_inclusive(direction)
            .take_while(move |position| *position != end)
    }
    pub fn line_exclusive_exclusive(
        self,
        end: Coord,
        direction: Vector,
    ) -> impl Iterator<Item = Self> {
        self.line_inclusive_exclusive(end, direction).skip(1)
    }
    pub fn line_inclusive_inclusive(
        self,
        end: Coord,
        direction: Vector,
    ) -> impl Iterator<Item = Self> {
        let mut resume = true;
        self.line_inclusive(direction).take_while(move |position| {
            resume && {
                resume = *position != end;
                true
            }
        })
    }
    pub fn line_exclusive_inclusive(
        self,
        end: Coord,
        direction: Vector,
    ) -> impl Iterator<Item = Self> {
        self.line_inclusive_inclusive(end, direction).skip(1)
    }
    pub fn is_inside_of(self, bound_1: Self, bound_2: Self) -> bool {
        (Ord::min(bound_1.x(), bound_2.x())..=Ord::max(bound_1.x(), bound_2.x()))
            .contains(&self.x())
            && (Ord::min(bound_1.y(), bound_2.y())..=Ord::max(bound_1.y(), bound_2.y()))
                .contains(&self.y())
    }
    pub fn color(self) -> Color {
        match (self.x() + self.y()) % 2 {
            0 => Color::White,
            1 => Color::Black,
            _ => unreachable!(),
        }
    }
}
pub fn home_rank(color: Color) -> u8 {
    match color {
        Color::White => coord_y!("1"),
        Color::Black => coord_y!("8"),
    }
}
pub fn pawn_home_rank(color: Color) -> u8 {
    match color {
        Color::White => coord_y!("2"),
        Color::Black => coord_y!("7"),
    }
}
pub fn pawn_promotion_rank(color: Color) -> u8 {
    match color {
        Color::White => coord_y!("8"),
        Color::Black => coord_y!("1"),
    }
}
impl Display for Coord {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let x = (self.x() + b'a') as char;
        let y = 8 - self.y();
        write!(f, "{x}{y}")?;
        Ok(())
    }
}
impl FromStr for Coord {
    type Err = ParseCoordError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        let Some(x) = chars.next() else {
            return Err(ParseCoordError::NotEnoughCharacter(0));
        };
        let Some(y) = chars.next() else {
            return Err(ParseCoordError::NotEnoughCharacter(1));
        };
        if let Some(c) = chars.next() {
            return Err(ParseCoordError::Unexpected(c));
        }
        Coord::from_chars(x, y)
    }
}
impl TryFrom<u8> for Coord {
    type Error = InvalidByte;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if (value >> 6) & 0b_11 == 0b_10 {
            Ok(Coord(NonZero::new(value).unwrap()))
        } else {
            Err(InvalidByte)
        }
    }
}
impl From<Coord> for u8 {
    fn from(value: Coord) -> Self {
        value.0.get()
    }
}
impl Sub<Self> for Coord {
    type Output = Vector;

    fn sub(self, rhs: Self) -> Self::Output {
        Vector {
            x: <i8>::try_from(self.x()).unwrap() - <i8>::try_from(rhs.x()).unwrap(),
            y: <i8>::try_from(self.y()).unwrap() - <i8>::try_from(rhs.y()).unwrap(),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Vector {
    pub x: i8,
    pub y: i8,
}
impl Vector {
    pub const ZERO: Self = Vector { x: 0, y: 0 };

    pub const KNIGHT_MOVES: [Self; 8] = [
        Vector { x: -1, y: -2 },
        Vector { x: 1, y: -2 },
        Vector { x: -1, y: 2 },
        Vector { x: 1, y: 2 },
        Vector { x: -2, y: -1 },
        Vector { x: 2, y: -1 },
        Vector { x: -2, y: 1 },
        Vector { x: 2, y: 1 },
    ];
    pub const KING_MOVES: [Self; 8] = [
        Vector { x: -1, y: -1 },
        Vector { x: 0, y: -1 },
        Vector { x: 1, y: -1 },
        Vector { x: -1, y: 0 },
        Vector { x: 1, y: 0 },
        Vector { x: -1, y: 1 },
        Vector { x: 0, y: 1 },
        Vector { x: 1, y: 1 },
    ];
    pub const ROOK_DIRECTIONS: [Self; 4] = [
        Vector { x: -1, y: 0 },
        Vector { x: 1, y: 0 },
        Vector { x: 0, y: -1 },
        Vector { x: 0, y: 1 },
    ];
    pub const BISHOP_DIRECTIONS: [Self; 4] = [
        Vector { x: -1, y: -1 },
        Vector { x: 1, y: -1 },
        Vector { x: -1, y: 1 },
        Vector { x: 1, y: 1 },
    ];
    pub const QUEEN_DIRECTIONS: [Self; 8] = Vector::KING_MOVES;

    pub fn pawn_single_move(color: Color) -> Self {
        Vector {
            x: 0,
            y: pawn_direction(color),
        }
    }
    pub fn pawn_double_move(color: Color) -> Self {
        Vector::pawn_single_move(color) * 2
    }
    pub fn pawn_attacks(color: Color) -> [Self; 2] {
        [-1, 1].map(|x| Vector {
            x,
            y: pawn_direction(color),
        })
    }
    pub fn is_aligned(self, other: Self) -> bool {
        self.as_unit() == other.as_unit() && self.x * other.y == other.x * self.y
    }
    pub fn is_king_move(self) -> bool {
        (-1..=1).contains(&self.x) && (-1..=1).contains(&self.y) && !(self.x == 0 && self.y == 0)
    }
    pub fn is_knight_move(self) -> bool {
        let x = self.x.unsigned_abs();
        let y = self.y.unsigned_abs();
        (x == 1 && y == 2) || (x == 2 && y == 1)
    }
    pub fn is_pawn_attack(self, color: Color) -> bool {
        self.x.unsigned_abs() == 1 && self.y == pawn_direction(color)
    }
    pub fn as_unit(self) -> Self {
        Vector {
            x: self.x.signum(),
            y: self.y.signum(),
        }
    }
}
pub fn pawn_direction(color: Color) -> i8 {
    match color {
        Color::White => -1,
        Color::Black => 1,
    }
}
impl Neg for Vector {
    type Output = Vector;

    fn neg(self) -> Self::Output {
        Vector {
            x: -self.x,
            y: -self.y,
        }
    }
}
impl Add<Self> for Vector {
    type Output = Vector;

    fn add(self, rhs: Self) -> Self::Output {
        Vector {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
impl AddAssign<Self> for Vector {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}
impl Sub<Self> for Vector {
    type Output = Vector;

    fn sub(self, rhs: Self) -> Self::Output {
        Vector {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}
impl SubAssign<Self> for Vector {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}
impl Mul<i8> for Vector {
    type Output = Vector;

    fn mul(self, rhs: i8) -> Self::Output {
        Vector {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}
impl MulAssign<i8> for Vector {
    fn mul_assign(&mut self, rhs: i8) {
        self.x *= rhs;
        self.y *= rhs;
    }
}
#[cfg(test)]
mod test {
    use crate::{coord, coord::Vector};

    #[test]
    fn adjacent_exclusive_exclusive_line_is_empty() {
        assert_eq!(
            coord!("e4")
                .line_exclusive_exclusive(coord!("e5"), Vector { x: 0, y: -1 })
                .next(),
            None
        );
    }
}
