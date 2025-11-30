use std::{
    cell::OnceCell,
    cmp::Ordering,
    error::Error,
    fmt::{self, Display, Formatter},
    hash::Hash,
    iter::FusedIterator,
    num::NonZero,
    ops::{Add, AddAssign, Index, IndexMut, Mul, MulAssign, Neg, Not, Range, Sub, SubAssign},
    rc::Rc,
    str::FromStr,
};

use crate::{board_display::IndexableBoard, coord_x, coord_y};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InvalidByte;

impl Display for InvalidByte {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "invalid byte")?;
        Ok(())
    }
}
impl Error for InvalidByte {}

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParseColorError;
impl Display for ParseColorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "provided string was not `w`, `b`, `W`, `B`, `white`, or `black`"
        )?;
        Ok(())
    }
}
impl Error for ParseColorError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Color {
    White = 1,
    Black = 0,
}
impl Color {
    pub fn lowercase(self) -> char {
        match self {
            Color::White => 'w',
            Color::Black => 'b',
        }
    }
}
impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Color::White => write!(f, "white")?,
            Color::Black => write!(f, "black")?,
        }
        Ok(())
    }
}
impl FromStr for Color {
    type Err = ParseColorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let color = match s {
            "w" | "W" | "white" => Color::White,
            "b" | "B" | "black" => Color::Black,
            _ => return Err(ParseColorError),
        };
        Ok(color)
    }
}
impl TryFrom<u8> for Color {
    type Error = InvalidByte;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let color = match value {
            0 => Color::Black,
            1 => Color::White,
            2.. => return Err(InvalidByte),
        };
        Ok(color)
    }
}
impl From<Color> for u8 {
    fn from(value: Color) -> Self {
        match value {
            Color::White => 1,
            Color::Black => 0,
        }
    }
}
impl Not for Color {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
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
        assert!(x < 8);
        assert!(y < 8);
        let byte = 0b1000_0000 | (y << 3) | x;
        Coord(NonZero::new(byte).unwrap())
    }
    pub fn from_chars(x: char, y: char) -> Result<Self, ParseCoordError> {
        let x = match x {
            'a'..='h' => x as u8 - b'a',
            _ => return Err(ParseCoordError::InvalidX(x)),
        };
        let y = match y {
            '1'..='8' => 7 - (x - b'1'),
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
                Some(self.line_until_exclusive(direction, 1, other))
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
        (start..).map_while(move |difference| self.move_by(direction * difference))
    }
    pub fn line_until_exclusive(
        self,
        direction: Vector,
        start: i8,
        end: Coord,
    ) -> impl Iterator<Item = Self> {
        self.line(direction, start)
            .take_while(move |position| *position != end)
    }
    pub fn line_until_inclusive(
        self,
        direction: Vector,
        start: i8,
        end: Coord,
    ) -> impl Iterator<Item = Self> {
        let mut resume = true;
        self.line(direction, start).take_while(move |position| {
            resume && {
                resume = *position != end;
                true
            }
        })
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
            x: self.x().checked_signed_diff(rhs.x()).unwrap(),
            y: self.y().checked_signed_diff(rhs.y()).unwrap(),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Vector {
    pub x: i8,
    pub y: i8,
}
impl Vector {
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
        Vector { x: -1, y: -2 },
        Vector { x: 1, y: -2 },
        Vector { x: -1, y: 2 },
        Vector { x: 1, y: 2 },
        Vector { x: -2, y: -1 },
        Vector { x: 2, y: -1 },
        Vector { x: -2, y: 1 },
        Vector { x: 2, y: 1 },
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

    pub fn is_aligned(self, other: Self) -> bool {
        self.as_unit() == other.as_unit() && self.x * other.y == other.x * self.y
    }
    pub fn is_king_move(self) -> bool {
        (-1..=1).contains(&self.x) && (-1..=1).contains(&self.y) && !(self.x == 0 && self.y == 0)
    }
    pub fn is_knight_move(self) -> bool {
        let x = self.x.abs();
        let y = self.y.abs();
        (x == 1 && y == 2) || (x == 2 && y == 1)
    }
    pub fn is_pawn_attack(self, color: Color) -> bool {
        self.x.abs() == 1 && self.y == pawn_direction(color)
    }
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
        [
            Vector {
                x: -1,
                y: pawn_direction(color),
            },
            Vector {
                x: 1,
                y: pawn_direction(color),
            },
        ]
    }
    pub fn as_unit(self) -> Self {
        Vector {
            x: self.x.signum(),
            y: self.x.signum(),
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Piece {
    pub piece: ColoredPieceKind,
    pub position: Coord,
}
impl Piece {
    fn step_moves(
        self,
        index: PieceIndex,
        board: &Board,
        moves: &[Vector],
    ) -> impl Iterator<Item = SimpleMove> {
        moves
            .iter()
            .copied()
            .filter_map(move |movement| self.position.move_by(movement))
            .filter_map(move |destination| {
                if let Some(capture) = board[destination] {
                    (board[capture].unwrap().piece.color() != self.piece.color()).then_some({
                        SimpleMove {
                            index,
                            destination,
                            capture: Some(capture),
                        }
                    })
                } else {
                    Some(SimpleMove {
                        index,
                        destination,
                        capture: None,
                    })
                }
            })
    }
    fn directional_moves(
        self,
        index: PieceIndex,
        board: &Board,
        direction: Vector,
    ) -> impl Iterator<Item = SimpleMove> {
        let mut resume = false;
        self.position
            .line(direction, 1)
            .map_while(move |destination| {
                if resume {
                    if let Some(capture) = board[destination] {
                        resume = false;
                        (board[capture].unwrap().piece.color() != self.piece.color()).then_some({
                            SimpleMove {
                                index,
                                destination,
                                capture: Some(capture),
                            }
                        })
                    } else {
                        Some(SimpleMove {
                            index,
                            destination,
                            capture: None,
                        })
                    }
                } else {
                    None
                }
            })
    }
    fn all_directional_moves(
        self,
        index: PieceIndex,
        board: &Board,
        directions: &[Vector],
    ) -> impl Iterator<Item = SimpleMove> {
        directions
            .iter()
            .copied()
            .flat_map(move |direction| self.directional_moves(index, board, direction))
    }
    fn pawn_moves(self, index: PieceIndex, board: &Board) -> impl Iterator<Item = Move> {
        static PROMOTION_CHOICES: [Option<PieceKind>; 4] = [
            Some(PieceKind::Queen),
            Some(PieceKind::Rook),
            Some(PieceKind::Bishop),
            Some(PieceKind::Knight),
        ];
        static NON_PROMOTION_CHOICES: [Option<PieceKind>; 1] = [None];
        let forward_jumps = if self.position.y() == pawn_home_rank(self.piece.color()) {
            2
        } else {
            1
        };
        self.position
            .line(Vector::pawn_single_move(self.piece.color()), 1)
            .take(forward_jumps)
            .take_while(|position| board[*position].is_none())
            .map(move |destination| Move {
                movement: SimpleMove {
                    index,
                    destination,
                    capture: None,
                },
                castling_rook: None,
                promotion: None,
                en_passant_target: ((destination - self.position)
                    == Vector::pawn_double_move(self.piece.color()))
                .then(|| {
                    self.position
                        .move_by(Vector::pawn_single_move(self.piece.color()))
                        .unwrap()
                })
                .filter(|en_passant_target| {
                    board.any_moves_has(
                        *en_passant_target,
                        &Vector::pawn_attacks(self.piece.color()),
                        !self.piece.color(),
                        PieceKind::Pawn,
                    )
                }),
                castling_right: board.castling_right,
            })
            .chain(
                Vector::pawn_attacks(self.piece.color())
                    .into_iter()
                    .filter_map(move |movement| self.position.move_by(movement))
                    .filter_map(move |destination| {
                        let capture = if Some(destination) == board.en_passant_target {
                            let (capture_index, _) = board
                                .get_with_kind_indexed(
                                    destination
                                        .move_by(Vector::pawn_single_move(!self.piece.color()))
                                        .unwrap(),
                                    !self.piece.color(),
                                    PieceKind::Pawn,
                                )
                                .unwrap();
                            capture_index
                        } else {
                            board[destination]?
                        };
                        Some(SimpleMove {
                            index,
                            destination,
                            capture: Some(capture),
                        })
                    })
                    .map(|movement| movement.to_simple_move(board.castling_right)),
            )
            .flat_map(move |movement| {
                let promotion_choices: &[_] = if movement.movement.destination.y()
                    == pawn_promotion_rank(self.piece.color())
                {
                    &PROMOTION_CHOICES
                } else {
                    &NON_PROMOTION_CHOICES
                };
                promotion_choices
                    .iter()
                    .copied()
                    .map(move |promotion| Move {
                        promotion,
                        ..movement
                    })
            })
    }
    fn non_castling_moves(self, index: PieceIndex, board: &Board) -> impl Iterator<Item = Move> {
        let moves: Box<dyn Iterator<Item = Move>> = match self.piece.piece() {
            PieceKind::Pawn => Box::new(self.pawn_moves(index, board)),
            PieceKind::Knight => Box::new(
                self.step_moves(index, board, &Vector::KNIGHT_MOVES)
                    .map(|movement| movement.to_simple_move(board.castling_right)),
            ),
            PieceKind::Bishop => Box::new(
                self.all_directional_moves(index, board, &Vector::BISHOP_DIRECTIONS)
                    .map(|movement| movement.to_simple_move(board.castling_right)),
            ),
            PieceKind::Rook => {
                let castling_right = if self.position.y() == home_rank(self.piece.color()) {
                    board
                        .castling_right
                        .to_removed(self.piece.color(), self.position.x())
                } else {
                    board.castling_right
                };
                Box::new(
                    self.all_directional_moves(index, board, &Vector::ROOK_DIRECTIONS)
                        .map(move |movement| movement.to_simple_move(castling_right)),
                )
            }
            PieceKind::Queen => Box::new(
                self.all_directional_moves(index, board, &Vector::QUEEN_DIRECTIONS)
                    .map(|movement| movement.to_simple_move(board.castling_right)),
            ),
            PieceKind::King => {
                let castling_right = board.castling_right.to_cleared(self.piece.color());
                Box::new(
                    self.step_moves(index, board, &Vector::KING_MOVES)
                        .map(move |movement| movement.to_simple_move(castling_right)),
                )
            }
        };
        moves.map(move |movement| {
            if let Some(capture) = movement.movement.capture {
                let piece = board[capture].unwrap();
                if piece.piece.piece() == PieceKind::Rook
                    && piece.piece.color() != self.piece.color()
                    && piece.position.y() == home_rank(piece.piece.color())
                {
                    Move {
                        castling_right: movement
                            .castling_right
                            .to_removed(piece.piece.color(), piece.position.x()),
                        ..movement
                    }
                } else {
                    movement
                }
            } else {
                movement
            }
        })
    }
    fn can_be_blocked(self, target: Coord, blocker: Coord) -> bool {
        self.position == blocker
            || (matches!(
                self.piece.piece(),
                PieceKind::Bishop | PieceKind::Rook | PieceKind::Queen
            ) && (target - self.position).is_aligned(blocker - self.position)
                && blocker.is_inside_of(self.position, target))
    }
}
impl Display for Piece {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} on {}", self.piece, self.position)?;
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EndState {
    Win(Color),
    Draw,
}
impl Display for EndState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            EndState::Win(color) => write!(f, "{color} wins")?,
            EndState::Draw => write!(f, "draw")?,
        }
        Ok(())
    }
}
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
    pub fn all(self, color: Color) -> impl Iterator<Item = u8> {
        (0..8).filter(move |x| self.get(color, *x))
    }
    pub fn get(self, color: Color, x: u8) -> bool {
        assert!(x < 8);
        let byte = match color {
            Color::White => self.white,
            Color::Black => self.black,
        };
        match (byte >> x) & 0b_1 {
            0 => false,
            1 => true,
            _ => unreachable!(),
        }
    }
    pub fn add(&mut self, color: Color, x: u8) {
        assert!(x < 8);
        let byte = match color {
            Color::White => &mut self.white,
            Color::Black => &mut self.black,
        };
        *byte |= 0b_1 << x;
    }
    pub fn to_added(self, color: Color, x: u8) -> Self {
        let mut new = self;
        new.add(color, x);
        new
    }
    pub fn remove(&mut self, color: Color, x: u8) {
        assert!(x < 8);
        let byte = match color {
            Color::White => &mut self.white,
            Color::Black => &mut self.black,
        };
        *byte &= !(0b_1 << x);
    }
    pub fn to_removed(self, color: Color, x: u8) -> Self {
        let mut new = self;
        new.remove(color, x);
        new
    }
    pub fn clear(&mut self, color: Color) {
        match color {
            Color::White => self.white = 0,
            Color::Black => self.black = 0,
        }
    }
    pub fn to_cleared(self, color: Color) -> Self {
        let mut new = self;
        new.clear(color);
        new
    }
    pub fn standard_fen_display(self) -> StandardCastlingRight {
        StandardCastlingRight(self)
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
                let c: char = (x + start).into();
                write!(f, "{c}")?;
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
                'Q' => castling_right.add(Color::White, coord_x!("a")),
                'K' => castling_right.add(Color::White, coord_x!("h")),
                'q' => castling_right.add(Color::Black, coord_x!("a")),
                'k' => castling_right.add(Color::Black, coord_x!("h")),
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
impl Display for StandardCastlingRight {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut written = false;
        for color in [Color::White, Color::Black] {
            for x in self.0.all(color) {
                let c = match (color, x) {
                    (Color::White, coord_x!("a")) => 'Q',
                    (Color::White, coord_x!("h")) => 'K',
                    (Color::Black, coord_x!("a")) => 'q',
                    (Color::Black, coord_x!("h")) => 'k',
                    _ => panic!("invalid rook file"),
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
        s.parse()
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum InvalidBoard {
    NoKing,
    NonPlayerInCheck,
    MoreThanTwoCheckers,
    InvalidCastlingRight,
    InvalidEnPassantTarget,
    PawnInHomeRank,
}
impl Display for InvalidBoard {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            InvalidBoard::NoKing => write!(f, "no kings found")?,
            InvalidBoard::NonPlayerInCheck => write!(f, "non-player in check")?,
            InvalidBoard::MoreThanTwoCheckers => {
                write!(f, "found more than 2 pieces delivering check")?;
            }
            InvalidBoard::InvalidCastlingRight => write!(f, "invalid castling right")?,
            InvalidBoard::InvalidEnPassantTarget => write!(f, "invalid en passant target")?,
            InvalidBoard::PawnInHomeRank => write!(f, "pawn in home rank")?,
        }
        Ok(())
    }
}
impl Error for InvalidBoard {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PieceIndex(NonZero<u8>);
impl From<PieceIndex> for usize {
    fn from(value: PieceIndex) -> Self {
        (value.0.get() & 0b_11111) as usize
    }
}
impl TryFrom<usize> for PieceIndex {
    type Error = InvalidByte;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value > 32 {
            return Err(InvalidByte);
        }
        let byte: u8 = value.try_into().unwrap();
        Ok(PieceIndex(NonZero::new(byte | 0b_1000_0000).unwrap()))
    }
}
#[derive(Debug, Clone)]
pub struct Board {
    pieces: [Option<Piece>; 32],
    indices: OnceCell<[Option<PieceIndex>; 64]>,
    current_player: Color,
    castling_right: CastlingRight,
    en_passant_target: Option<Coord>,
}
fn original_piece_range(color: Color, piece: PieceKind) -> Range<usize> {
    match (color, piece) {
        (Color::White, PieceKind::Pawn) => 8..16,
        (Color::White, PieceKind::Knight) => 6..8,
        (Color::White, PieceKind::Bishop) => 4..6,
        (Color::White, PieceKind::Rook) => 2..4,
        (Color::White, PieceKind::Queen) => 1..2,
        (Color::White, PieceKind::King) => 0..1,
        (Color::Black, PieceKind::Pawn) => 24..32,
        (Color::Black, PieceKind::Knight) => 22..24,
        (Color::Black, PieceKind::Bishop) => 20..22,
        (Color::Black, PieceKind::Rook) => 18..20,
        (Color::Black, PieceKind::Queen) => 17..18,
        (Color::Black, PieceKind::King) => 16..17,
    }
}
impl Board {
    pub fn starting_position() -> Self {
        Board::from_configuration(PieceKind::STARTING_CONFIGURATION)
    }
    fn from_configuration(configuration: [PieceKind; 8]) -> Self {
        HashableBoard::from_configuration(configuration)
            .try_into()
            .unwrap()
    }
    pub fn current_player(&self) -> Color {
        self.current_player
    }
    fn range(&self, range: Range<usize>) -> impl Iterator<Item = Piece> {
        self.pieces[range].iter().copied().flatten()
    }
    fn range_indexed(&self, range: Range<usize>) -> impl Iterator<Item = (PieceIndex, Piece)> {
        let slice = &self.pieces[range.clone()];
        range
            .into_iter()
            .map(|index| index.try_into().unwrap())
            .zip(slice.iter().copied())
            .filter_map(|(index, piece)| piece.map(|piece| (index, piece)))
    }
    fn all_pieces(&self) -> impl Iterator<Item = Piece> {
        self.pieces.iter().copied().flatten()
    }
    fn all_pieces_indexed(&self) -> impl Iterator<Item = (PieceIndex, Piece)> {
        self.pieces
            .iter()
            .copied()
            .enumerate()
            .filter_map(|(i, piece)| piece.map(|piece| (i.try_into().unwrap(), piece)))
    }
    fn pieces(&self, color: Color) -> impl FusedIterator<Item = Piece> {
        let slice = match color {
            Color::White => &self.pieces[0..16],
            Color::Black => &self.pieces[16..32],
        };
        slice.iter().copied().flatten()
    }
    fn non_kings(&self, color: Color) -> impl Iterator<Item = Piece> {
        let slice = match color {
            Color::White => &self.pieces[1..16],
            Color::Black => &self.pieces[17..32],
        };
        slice.iter().copied().flatten()
    }
    fn pieces_indexed(&self, color: Color) -> impl Iterator<Item = (PieceIndex, Piece)> {
        match color {
            Color::White => self.range_indexed(0..16),
            Color::Black => self.range_indexed(16..32),
        }
    }
    fn pieces_by_kind(&self, color: Color, piece: PieceKind) -> impl Iterator<Item = Piece> {
        let definite_pieces = if piece == PieceKind::Pawn {
            self.range(0..0)
        } else {
            self.range(original_piece_range(color, piece))
        };
        definite_pieces.chain(
            self.range(original_piece_range(color, PieceKind::Pawn))
                .filter(move |item| item.piece.piece() == piece),
        )
    }
    fn pieces_by_kind_indexed(
        &self,
        color: Color,
        piece: PieceKind,
    ) -> impl Iterator<Item = (PieceIndex, Piece)> {
        let range = if piece == PieceKind::Pawn {
            0..0
        } else {
            original_piece_range(color, piece)
        };
        self.range_indexed(range).chain(
            self.range_indexed(original_piece_range(color, PieceKind::Pawn))
                .filter(move |(_, item)| item.piece.piece() == piece),
        )
    }
    fn pieces_by_kinds(&self, color: Color, pieces: &[PieceKind]) -> impl Iterator<Item = Piece> {
        pieces
            .iter()
            .copied()
            .map(move |piece| {
                if piece == PieceKind::Pawn {
                    0..0
                } else {
                    original_piece_range(color, piece)
                }
            })
            .flat_map(|range| self.range(range))
            .chain(
                self.range(original_piece_range(color, PieceKind::Pawn))
                    .filter(move |piece| pieces.contains(&piece.piece.piece())),
            )
    }
    fn get_with_kind_indexed(
        &self,
        position: Coord,
        color: Color,
        piece: PieceKind,
    ) -> Option<(PieceIndex, Piece)> {
        if let Some(indices) = self.indices.get() {
            indices[position.y() as usize * 8 + position.x() as usize]
                .map(|index| (index, self[index].unwrap()))
        } else {
            self.pieces_by_kind_indexed(color, piece)
                .find(|(_, piece)| piece.position == position)
        }
    }
    fn position_has(&self, position: Coord, color: Color, piece: PieceKind) -> bool {
        if let Some(indices) = self.indices.get() {
            indices[position.y() as usize * 8 + position.x() as usize].is_some_and(|index| {
                let b = self[index].unwrap();
                b.piece.color() == color && b.piece.piece() == piece
            })
        } else {
            self.pieces_by_kind(color, piece)
                .any(|piece| piece.position == position)
        }
    }
    fn any_moves_has(
        &self,
        position: Coord,
        moves: &[Vector],
        color: Color,
        piece: PieceKind,
    ) -> bool {
        if let Some(indices) = self.indices.get() {
            moves
                .iter()
                .copied()
                .filter_map(|movement| position.move_by(movement))
                .any(|position| {
                    indices[position.y() as usize * 8 + position.x() as usize].is_some_and(
                        |index| {
                            let b = self[index].unwrap();
                            b.piece.color() == color && b.piece.piece() == piece
                        },
                    )
                })
        } else {
            self.pieces_by_kind(color, piece)
                .any(|piece| moves.contains(&(piece.position - position)))
        }
    }
    fn indices(&self) -> &[Option<PieceIndex>; 64] {
        self.indices.get_or_init(|| {
            let mut board = [None; 64];
            for (i, piece) in self.all_pieces_indexed() {
                board[(piece.position.y() as usize * 8) + piece.position.x() as usize] = Some(i);
            }
            board
        })
    }
    fn bishops_and_rooks_and_queens(&self, color: Color) -> impl Iterator<Item = Piece> {
        self.pieces_by_kinds(
            color,
            &[PieceKind::Bishop, PieceKind::Rook, PieceKind::Queen],
        )
    }
    fn king(&self, color: Color) -> Option<Piece> {
        match color {
            Color::White => self.pieces[0],
            Color::Black => self.pieces[16],
        }
    }
    fn king_indexed(&self, color: Color) -> Option<(PieceIndex, Piece)> {
        match color {
            Color::White => self.pieces[0].map(|piece| (0.try_into().unwrap(), piece)),
            Color::Black => self.pieces[0].map(|piece| (16.try_into().unwrap(), piece)),
        }
    }
    fn pawns(&self, color: Color) -> impl Iterator<Item = Piece> {
        self.range(original_piece_range(color, PieceKind::Pawn))
            .filter(|item| item.piece.piece() == PieceKind::Pawn)
    }
    fn validate(&self) -> Result<(), InvalidBoard> {
        let (Some(king), Some(opponent_king)) = (
            self.king(self.current_player),
            self.king(!self.current_player),
        ) else {
            return Err(InvalidBoard::NoKing);
        };
        if self
            .attackers(opponent_king.position, self.current_player)
            .next()
            .is_some()
        {
            return Err(InvalidBoard::NonPlayerInCheck);
        }
        if self
            .attackers(king.position, !self.current_player)
            .nth(2)
            .is_some()
        {
            return Err(InvalidBoard::MoreThanTwoCheckers);
        }
        if ![Color::White, Color::Black].into_iter().all(|color| {
            let king = self.king(color).unwrap();
            let king_on_home = king.position.y() == home_rank(color);
            self.castling_right.all(color).all(|x| {
                king_on_home
                    && self
                        .get_with_kind_indexed(
                            Coord::new(x, home_rank(color)),
                            color,
                            PieceKind::Rook,
                        )
                        .is_some()
            })
        }) {
            return Err(InvalidBoard::InvalidCastlingRight);
        }
        if let Some(en_passant_target) = self.en_passant_target
            && ![Color::White, Color::Black].into_iter().any(|color| {
                en_passant_target
                    .move_by(Vector::pawn_single_move(color))
                    .is_some_and(|position| self.position_has(position, color, PieceKind::Pawn))
            })
        {
            return Err(InvalidBoard::InvalidEnPassantTarget);
        }
        if [Color::White, Color::Black]
            .into_iter()
            .flat_map(|color| self.pawns(color))
            .any(|pawn| matches!(pawn.position.y(), coord_y!("1") | coord_y!("8")))
        {
            return Err(InvalidBoard::PawnInHomeRank);
        }
        Ok(())
    }
    fn attackers_with_inspect(
        &self,
        position: Coord,
        color: Color,
        checker: impl Fn(Coord) -> bool + Clone,
    ) -> impl FusedIterator<Item = Piece> {
        self.pieces(color)
            .filter(move |piece| match piece.piece.piece() {
                PieceKind::Pawn => (piece.position - position).is_pawn_attack(color),
                PieceKind::Knight => (piece.position - position).is_knight_move(),
                PieceKind::Bishop => piece
                    .position
                    .is_aligned_with_bishop(position)
                    .is_some_and(|mut inside| !inside.any(checker.clone())),
                PieceKind::Rook => piece
                    .position
                    .is_aligned_with_rook(position)
                    .is_some_and(|mut inside| !inside.any(checker.clone())),
                PieceKind::Queen => piece
                    .position
                    .is_aligned_with_queen(position)
                    .is_some_and(|mut inside| !inside.any(checker.clone())),
                PieceKind::King => (piece.position - position).is_king_move(),
            })
    }
    fn attackers(&self, position: Coord, color: Color) -> impl FusedIterator<Item = Piece> {
        self.attackers_with_inspect(position, color, |position| self[position].is_some())
    }
    fn is_move_attacked(&self, index: PieceIndex, destination: Coord, color: Color) -> bool {
        self.attackers_with_inspect(destination, color, |position| {
            self[position].is_some_and(|b| b != index)
        })
        .next()
        .is_some()
    }
    fn pinned_with_inspect(
        &self,
        king: Coord,
        position: Coord,
        color: Color,
        checker: impl Fn(Coord) -> bool + Clone,
    ) -> Option<impl Iterator<Item = Coord>> {
        let direction = position - king;
        if Vector::QUEEN_DIRECTIONS
            .iter()
            .any(|valid_direction| direction.is_aligned(*valid_direction))
        {
            let direction = direction.as_unit();
            if position
                .line_until_exclusive(-direction, 1, king)
                .any(checker.clone())
            {
                None
            } else {
                let pieces = if Vector::BISHOP_DIRECTIONS.contains(&direction) {
                    &[PieceKind::Bishop, PieceKind::Queen]
                } else {
                    &[PieceKind::Rook, PieceKind::Queen]
                };
                self.pieces_by_kinds(!color, pieces).find_map(|piece| {
                    if direction.is_aligned(piece.position - king)
                        && position.is_inside_of(piece.position, king)
                    {
                        (!piece
                            .position
                            .line_until_exclusive(-direction, 1, position)
                            .any(checker.clone()))
                        .then(|| piece.position.line_until_exclusive(-direction, 0, king))
                    } else {
                        None
                    }
                })
            }
        } else {
            None
        }
    }
    fn valid_destinations_when_pinned(
        &self,
        king: Coord,
        position: Coord,
        color: Color,
    ) -> Option<impl Iterator<Item = Coord>> {
        self.pinned_with_inspect(king, position, color, |position| self[position].is_some())
    }
    fn one_side_is_dead(&self, color: Color) -> Option<bool> {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        enum PieceLeft {
            Knight,
            Bishop(Color),
        }
        self.non_kings(color)
            .try_fold(None, |piece_left, piece| match piece.piece.piece() {
                PieceKind::Knight => piece_left.is_none().then_some(Some(PieceLeft::Knight)),
                PieceKind::Bishop => {
                    let color = piece.position.color();
                    match piece_left {
                        Some(PieceLeft::Bishop(b)) if color == b => {
                            Some(Some(PieceLeft::Bishop(color)))
                        }
                        Some(_) => None,
                        None => Some(Some(PieceLeft::Bishop(color))),
                    }
                }
                _ => None,
            })
            .map(|piece_left| piece_left.is_none())
    }
    fn has_lone_king(&self, color: Color) -> bool {
        self.non_kings(color).next().is_none()
    }
    fn is_dead(&self) -> bool {
        match self.one_side_is_dead(Color::White) {
            Some(false) => self.has_lone_king(Color::Black),
            Some(true) => self.one_side_is_dead(Color::Black).is_some(),
            None => false,
        }
    }
    pub fn valid_moves(&self) -> Result<impl Iterator<Item = Move>, EndState> {
        if self.is_dead() {
            Err(EndState::Draw)
        } else {
            let (valid_moves, check) = self.valid_moves_and_check();
            let mut valid_moves = valid_moves.peekable();
            if valid_moves.peek().is_some() {
                Ok(valid_moves)
            } else if check {
                Err(EndState::Win(!self.current_player))
            } else {
                Err(EndState::Draw)
            }
        }
    }
    fn castling_moves(&self, check: bool) -> impl Iterator<Item = Move> {
        let (king_index, king) = self.king_indexed(self.current_player).unwrap();
        let castling_right = self.castling_right;
        let new_castling_right = castling_right.to_cleared(self.current_player);
        castling_right
            .all(self.current_player)
            .filter(move |_| !check)
            .filter_map(move |x| {
                let (rook_index, rook) = self
                    .get_with_kind_indexed(
                        Coord::new(x, home_rank(self.current_player)),
                        self.current_player,
                        PieceKind::Rook,
                    )
                    .unwrap();
                let (king_destination, rook_destination) =
                    match Ord::cmp(&king.position.x(), &rook.position.x()) {
                        Ordering::Less => (
                            Coord::new(coord_x!("g"), home_rank(self.current_player)),
                            Coord::new(coord_x!("f"), home_rank(self.current_player)),
                        ),
                        Ordering::Equal => unreachable!(),
                        Ordering::Greater => (
                            Coord::new(coord_x!("c"), home_rank(self.current_player)),
                            Coord::new(coord_x!("d"), home_rank(self.current_player)),
                        ),
                    };
                [
                    (
                        king.position,
                        king_destination,
                        rook.position,
                        PieceKind::King,
                    ),
                    (
                        rook.position,
                        rook_destination,
                        king.position,
                        PieceKind::Rook,
                    ),
                ]
                .into_iter()
                .all(|(origin, destination, other_position, piece)| {
                    origin
                        .line_until_inclusive((destination - origin).as_unit(), 1, destination)
                        .all(|position| {
                            (position == other_position || self[position].is_none())
                                && (piece != PieceKind::King
                                    || self.is_move_attacked(
                                        rook_index,
                                        destination,
                                        !self.current_player,
                                    ))
                        })
                })
                .then_some(Move {
                    movement: SimpleMove {
                        index: king_index,
                        destination: king_destination,
                        capture: None,
                    },
                    castling_rook: Some(SimpleMove {
                        index: rook_index,
                        destination: rook_destination,
                        capture: None,
                    }),
                    promotion: None,
                    en_passant_target: None,
                    castling_right: new_castling_right,
                })
            })
    }
    fn valid_moves_and_check(&self) -> (impl Iterator<Item = Move> + '_, bool) {
        let king = self.king(self.current_player).unwrap();
        let mut attackers_iter = self.attackers(king.position, self.current_player).fuse();
        let attackers = [attackers_iter.next(), attackers_iter.next()];
        debug_assert!(attackers_iter.next().is_none());
        let check = !attackers.is_empty();
        let non_castling_moves = self
            .all_pieces_indexed()
            .flat_map(move |(index, piece)| {
                let valid_destination_when_pinned: Option<Rc<[_]>> =
                    if piece.piece.piece() == PieceKind::King {
                        None
                    } else {
                        self.valid_destinations_when_pinned(
                            king.position,
                            piece.position,
                            self.current_player,
                        )
                        .map(Iterator::collect)
                    };
                piece
                    .non_castling_moves(index, self)
                    .map(move |movement| (movement, piece, valid_destination_when_pinned.clone()))
            })
            .filter(move |(movement, piece, valid_destination_when_pinned)| {
                if piece.piece.piece() == PieceKind::King {
                    self.is_move_attacked(
                        movement.movement.index,
                        piece.position,
                        !self.current_player,
                    )
                } else {
                    attackers
                        .into_iter()
                        .map_while(|attacker| attacker)
                        .all(|piece| {
                            piece.can_be_blocked(king.position, movement.movement.destination)
                        })
                        && valid_destination_when_pinned
                            .as_ref()
                            .is_none_or(|valid_destinations| {
                                valid_destinations.contains(&movement.movement.destination)
                            })
                        && {
                            // special case for en passant when the captured pawn is pinned
                            let en_passant = movement.movement.capture.is_some_and(|index| {
                                self[index].unwrap().position != movement.movement.destination
                            });
                            (!en_passant)
                                || self
                                    .pinned_with_inspect(
                                        king.position,
                                        self[movement.movement.capture.unwrap()].unwrap().position,
                                        piece.piece.color(),
                                        |position| {
                                            position != piece.position
                                                && (position == movement.movement.destination
                                                    || self[position].is_some())
                                        },
                                    )
                                    .is_none()
                        }
                }
            })
            .map(|(movement, _, _)| movement);
        (non_castling_moves.chain(self.castling_moves(check)), check)
    }
    fn move_piece(&mut self, movement: &impl Moveable) {
        movement.move_board(self);
    }
    pub fn clone_and_move(&self, movement: &impl Moveable) -> Self {
        let mut new = self.clone();
        new.move_piece(movement);
        new
    }
}
impl Index<Coord> for Board {
    type Output = Option<PieceIndex>;

    fn index(&self, index: Coord) -> &Self::Output {
        &self.indices()[(index.y() as usize * 8) + index.x() as usize]
    }
}
impl Index<PieceIndex> for Board {
    type Output = Option<Piece>;

    fn index(&self, index: PieceIndex) -> &Self::Output {
        let index: usize = index.into();
        &self.pieces[index]
    }
}
impl IndexMut<PieceIndex> for Board {
    fn index_mut(&mut self, index: PieceIndex) -> &mut Self::Output {
        let index: usize = index.into();
        &mut self.pieces[index]
    }
}
impl IndexableBoard for Board {
    fn index(&self, position: Coord) -> Option<ColoredPieceKind> {
        self[position].map(|index| self[index].unwrap().piece)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExceededPieces {
    PromotedPiece,
    Pawn,
    King,
}
impl Display for ExceededPieces {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ExceededPieces::PromotedPiece => {
                write!(f, "exceeded allowable number of promoted pieces")?;
            }
            ExceededPieces::Pawn => write!(f, "found more than 8 pawns")?,
            ExceededPieces::King => write!(f, "found more than 1 kings")?,
        }
        Ok(())
    }
}
impl Error for ExceededPieces {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HashableBoard {
    pub board: [[Option<ColoredPieceKind>; 8]; 8],
    pub current_player: Color,
    pub castling_right: CastlingRight,
    pub en_passant_target: Option<Coord>,
}
impl HashableBoard {
    fn starting_position() -> Self {
        HashableBoard::from_configuration(PieceKind::STARTING_CONFIGURATION)
    }
    fn from_configuration(configuration: [PieceKind; 8]) -> Self {
        let castling_right = CastlingRight::from_configuration(configuration);
        let board = [
            configuration.map(|piece| Some(ColoredPieceKind::new(Color::Black, piece))),
            [Some(ColoredPieceKind::new(Color::Black, PieceKind::Pawn)); 8],
            [None; 8],
            [None; 8],
            [None; 8],
            [None; 8],
            [Some(ColoredPieceKind::new(Color::White, PieceKind::Pawn)); 8],
            configuration.map(|piece| Some(ColoredPieceKind::new(Color::White, piece))),
        ];
        HashableBoard {
            board,
            current_player: Color::White,
            castling_right,
            en_passant_target: None,
        }
    }
}
impl TryFrom<HashableBoard> for Board {
    type Error = ExceededPieces;

    fn try_from(value: HashableBoard) -> Result<Self, Self::Error> {
        let mut pieces = [None; 32];
        for (y, row) in value.board.iter().enumerate() {
            'upper: for (x, piece) in row.iter().enumerate() {
                if let Some(piece) = piece {
                    let range = original_piece_range(piece.color(), piece.piece());
                    let pawn_range = match piece.piece() {
                        PieceKind::King | PieceKind::Pawn => 0..0,
                        _ => original_piece_range(piece.color(), PieceKind::Pawn),
                    };
                    let [piece_rack, pawn_rack] =
                        pieces.get_disjoint_mut([range, pawn_range]).unwrap();
                    for square in piece_rack.iter_mut().chain(pawn_rack.iter_mut()) {
                        if square.is_none() {
                            *square = Some(Piece {
                                piece: *piece,
                                position: Coord::new(x.try_into().unwrap(), y.try_into().unwrap()),
                            });
                            continue 'upper;
                        }
                    }
                    match piece.piece() {
                        PieceKind::Pawn => {
                            if pieces[original_piece_range(piece.color(), PieceKind::Pawn)]
                                .iter()
                                .copied()
                                .all(|piece| piece.unwrap().piece.piece() == PieceKind::Pawn)
                            {
                                return Err(ExceededPieces::Pawn);
                            }
                            return Err(ExceededPieces::PromotedPiece);
                        }
                        PieceKind::King => return Err(ExceededPieces::King),
                        _ => return Err(ExceededPieces::PromotedPiece),
                    }
                }
            }
        }
        let mut board = Board {
            pieces,
            indices: OnceCell::new(),
            current_player: value.current_player,
            castling_right: value.castling_right,
            en_passant_target: value.en_passant_target,
        };
        if let Some(en_passant_target) = board.en_passant_target
            && ![Color::White, Color::Black].into_iter().any(|color| {
                board
                    .pawns(color)
                    .any(|piece| (en_passant_target - piece.position).is_pawn_attack(color))
            })
        {
            board.en_passant_target = None;
        }
        Ok(board)
    }
}
impl Index<Coord> for HashableBoard {
    type Output = Option<ColoredPieceKind>;

    fn index(&self, index: Coord) -> &Self::Output {
        &self.board[index.y() as usize][index.x() as usize]
    }
}
impl IndexMut<Coord> for HashableBoard {
    fn index_mut(&mut self, index: Coord) -> &mut Self::Output {
        &mut self.board[index.y() as usize][index.x() as usize]
    }
}
impl IndexableBoard for HashableBoard {
    fn index(&self, position: Coord) -> Option<ColoredPieceKind> {
        self[position]
    }
}
pub trait Moveable {
    fn move_board(&self, board: &mut Board);
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SimpleMove {
    index: PieceIndex,
    destination: Coord,
    capture: Option<PieceIndex>,
}
impl SimpleMove {
    fn to_simple_move(self, castling_right: CastlingRight) -> Move {
        Move {
            movement: self,
            castling_rook: None,
            promotion: None,
            en_passant_target: None,
            castling_right,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Move {
    movement: SimpleMove,
    castling_rook: Option<SimpleMove>,
    promotion: Option<PieceKind>,
    en_passant_target: Option<Coord>,
    castling_right: CastlingRight,
}
impl Move {
    fn as_long_algebraic_notation(self, board: &Board) -> LongAlgebraicNotation {
        let piece = board[self.movement.index].unwrap();
        if let Some(rook) = self.castling_rook {
            let rook = board[rook.index].unwrap();
            if piece.position.x() == coord_x!("e")
                && matches!(rook.position.x(), coord_x!("a") | coord_x!("h"))
            {
                LongAlgebraicNotation {
                    origin: piece.position,
                    destination: self.movement.destination,
                    promotion: self.promotion,
                }
            } else {
                LongAlgebraicNotation {
                    origin: piece.position,
                    destination: rook.position,
                    promotion: self.promotion,
                }
            }
        } else {
            LongAlgebraicNotation {
                origin: piece.position,
                destination: self.movement.destination,
                promotion: self.promotion,
            }
        }
    }
    fn as_long_algebraic_notation_chess_960(self, board: &Board) -> LongAlgebraicNotation {
        let piece = board[self.movement.index].unwrap();
        if let Some(rook) = self.castling_rook {
            LongAlgebraicNotation {
                origin: piece.position,
                destination: board[rook.index].unwrap().position,
                promotion: self.promotion,
            }
        } else {
            LongAlgebraicNotation {
                origin: piece.position,
                destination: self.movement.destination,
                promotion: self.promotion,
            }
        }
    }
}
impl Moveable for Move {
    fn move_board(&self, board: &mut Board) {
        let current_player = board.current_player;
        let piece = board[self.movement.index].as_mut().unwrap();
        piece.position = self.movement.destination;
        if let Some(promotion) = self.promotion {
            piece.piece = ColoredPieceKind::new(current_player, promotion);
        }
        if let Some(index) = self.movement.capture {
            board[index] = None;
        }
        if let Some(movement) = self.castling_rook {
            let rook = board[movement.index].as_mut().unwrap();
            rook.position = movement.destination;
        }
        board.en_passant_target = self.en_passant_target;
        board.castling_right = self.castling_right;

        board.indices = OnceCell::new();
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ParseMoveError {
    InvalidChar,
    ParseCoordError(ParseCoordError),
    InvalidFenPiece(InvalidFenPiece),
    Unexpected(char),
}
impl From<ParseCoordError> for ParseMoveError {
    fn from(value: ParseCoordError) -> Self {
        ParseMoveError::ParseCoordError(value)
    }
}
impl From<InvalidFenPiece> for ParseMoveError {
    fn from(value: InvalidFenPiece) -> Self {
        ParseMoveError::InvalidFenPiece(value)
    }
}
impl Display for ParseMoveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ParseMoveError::InvalidChar => write!(f, "provided string contains invalid character")?,
            ParseMoveError::ParseCoordError(err) => write!(f, "{err}")?,
            ParseMoveError::InvalidFenPiece(err) => write!(f, "{err}")?,
            ParseMoveError::Unexpected(c) => write!(f, "unexpected `{c}`")?,
        }
        Ok(())
    }
}
impl Error for ParseMoveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ParseMoveError::ParseCoordError(err) => Some(err),
            ParseMoveError::InvalidFenPiece(err) => Some(err),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct LongAlgebraicNotation {
    origin: Coord,
    destination: Coord,
    promotion: Option<PieceKind>,
}
impl LongAlgebraicNotation {
    fn as_move(self, board: &Board) -> Move {
        todo!()
    }
}
impl Display for LongAlgebraicNotation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.origin, self.destination)?;
        if let Some(promotion) = self.promotion {
            write!(f, "{}", promotion.lowercase())?;
        }
        Ok(())
    }
}
impl FromStr for LongAlgebraicNotation {
    type Err = ParseMoveError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let origin = s.get(0..2).ok_or(ParseMoveError::InvalidChar)?.parse()?;
        let destination = s.get(2..4).ok_or(ParseMoveError::InvalidChar)?.parse()?;
        let mut rest = s
            .get(4..)
            .ok_or(ParseMoveError::InvalidChar)?
            .chars()
            .fuse();
        let promotion = rest.next().map(PieceKind::from_fen).transpose()?;

        if let Some(c) = rest.next() {
            return Err(ParseMoveError::Unexpected(c));
        }
        Ok(LongAlgebraicNotation {
            origin,
            destination,
            promotion,
        })
    }
}
impl Moveable for LongAlgebraicNotation {
    fn move_board(&self, board: &mut Board) {
        board.move_piece(&self.as_move(board));
    }
}
