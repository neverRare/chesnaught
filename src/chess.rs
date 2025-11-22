use std::{
    cmp::Ordering,
    error::Error,
    fmt::Display,
    ops::{Index, IndexMut, Not, RangeInclusive},
    str::FromStr,
};

use crate::{coord, coord_x, coord_y};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}
impl PieceKind {
    fn lowercase(self) -> char {
        match self {
            PieceKind::Pawn => 'p',
            PieceKind::Knight => 'n',
            PieceKind::Bishop => 'b',
            PieceKind::Rook => 'r',
            PieceKind::Queen => 'q',
            PieceKind::King => 'k',
        }
    }
    fn uppercase(self) -> char {
        self.lowercase().to_ascii_uppercase()
    }
    // fn algebraic_notation_char(self) -> Option<char> {
    //     match self {
    //         PieceKind::Pawn => None,
    //         PieceKind::Knight => Some('N'),
    //         PieceKind::Bishop => Some('B'),
    //         PieceKind::Rook => Some('R'),
    //         PieceKind::Queen => Some('Q'),
    //         PieceKind::King => Some('K'),
    //     }
    // }
    // fn algebraic_notation(self) -> &'static str {
    //     match self {
    //         PieceKind::Pawn => "",
    //         PieceKind::Knight => "N",
    //         PieceKind::Bishop => "B",
    //         PieceKind::Rook => "R",
    //         PieceKind::Queen => "Q",
    //         PieceKind::King => "K",
    //     }
    // }
}
impl Display for PieceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsePieceKindError {
    Empty,
    UnknownSymbol(char),
    UnexpectedSymbol(char),
}
impl Display for ParsePieceKindError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsePieceKindError::Empty => write!(f, "expected one character, found none instead")?,
            ParsePieceKindError::UnknownSymbol(c) => write!(
                f,
                "`{c}` is neither of `p`, `n`, `b`, `r`, `q`, `k`, uppercase letter of any of these, or unicode chess symbols"
            )?,
            ParsePieceKindError::UnexpectedSymbol(c) => {
                write!(f, "unexpected `{c}`, only one character is expected")?;
            }
        }
        Ok(())
    }
}
impl Error for ParsePieceKindError {}

impl FromStr for PieceKind {
    type Err = ParsePieceKindError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut characters = s.chars();
        let piece = characters
            .next()
            .ok_or(ParsePieceKindError::Empty)?
            .try_into()?;

        if let Some(c) = characters.next() {
            return Err(ParsePieceKindError::UnexpectedSymbol(c));
        }
        Ok(piece)
    }
}
impl TryFrom<char> for PieceKind {
    type Error = ParsePieceKindError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        let piece = match value {
            'p' | 'P' | '♙' | '♟' => PieceKind::Pawn,
            'n' | 'N' | '♘' | '♞' => PieceKind::Knight,
            'b' | 'B' | '♗' | '♝' => PieceKind::Bishop,
            'r' | 'R' | '♖' | '♜' => PieceKind::Rook,
            'q' | 'Q' | '♕' | '♛' => PieceKind::Queen,
            'k' | 'K' | '♔' | '♚' => PieceKind::King,
            c => return Err(ParsePieceKindError::UnknownSymbol(c)),
        };
        Ok(piece)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    White,
    Black,
}
impl Color {
    pub fn lowercase(self) -> char {
        match self {
            Color::White => 'w',
            Color::Black => 'b',
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
impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Color::White => write!(f, "white")?,
            Color::Black => write!(f, "black")?,
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Piece {
    pub color: Color,
    pub kind: PieceKind,
    pub moved: bool,
    pub just_moved_twice_as_pawn: bool,
}
impl Piece {
    pub fn figurine(self) -> char {
        match (self.color, self.kind) {
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
    pub fn fen(self) -> char {
        match self.color {
            Color::White => self.kind.uppercase(),
            Color::Black => self.kind.lowercase(),
        }
    }
}
impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.color, self.kind)?;
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EndState {
    Win(Color),
    Draw,
}
impl Display for EndState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EndState::Win(color) => write!(f, "{color} wins")?,
            EndState::Draw => write!(f, "draw")?,
        }
        Ok(())
    }
}
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CastlingRights {
    white_king_side: bool,
    white_queen_side: bool,
    black_king_side: bool,
    black_queen_side: bool,
}
impl Display for CastlingRights {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut printed = false;
        if self.white_king_side {
            printed = true;
            write!(f, "K")?;
        }
        if self.white_queen_side {
            printed = true;
            write!(f, "Q")?;
        }
        if self.black_king_side {
            printed = true;
            write!(f, "k")?;
        }
        if self.black_queen_side {
            printed = true;
            write!(f, "q")?;
        }
        if !printed {
            write!(f, "-")?;
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Board {
    pub current_player: Color,
    pub board: [[Option<Piece>; 8]; 8],
}
impl Board {
    pub fn new() -> Self {
        Board::default()
    }
    // fn blank(current_player: Color) -> Self {
    //     Board {
    //         current_player,
    //         board: [[None; 8]; 8],
    //     }
    // }
    // fn iter(&self) -> impl Iterator<Item = &Piece> {
    //     self.board
    //         .iter()
    //         .flat_map(|row| row.iter())
    //         .flat_map(Option::iter)
    // }
    fn iter_mut(&mut self) -> impl Iterator<Item = &mut Piece> {
        self.board
            .iter_mut()
            .flat_map(|row| row.iter_mut())
            .flat_map(Option::iter_mut)
    }
    // fn into_iter(self) -> impl Iterator<Item = Piece> {
    //     self.board
    //         .into_iter()
    //         .flat_map(|row| row.into_iter())
    //         .filter_map(|piece| piece)
    // }
    fn pieces(self) -> impl Iterator<Item = PieceWithContext> {
        self.board.into_iter().zip(0..).flat_map(move |(row, y)| {
            row.into_iter().zip(0..).flat_map(move |(piece, x)| {
                piece.into_iter().map(move |piece| PieceWithContext {
                    piece,
                    position: Coord { x, y },
                    board: self,
                })
            })
        })
    }
    fn pieces_of(self, color: Color) -> impl Iterator<Item = PieceWithContext> {
        self.pieces()
            .filter(move |piece| piece.piece.color == color)
    }
    fn king_of(self, color: Color) -> Option<PieceWithContext> {
        self.pieces_of(color)
            .find(|piece| piece.piece.kind == PieceKind::King)
    }
    pub fn state(self) -> Option<EndState> {
        let white_king = self.king_of(Color::White);
        let black_king = self.king_of(Color::Black);
        match (white_king, black_king) {
            (None, None) => Some(EndState::Draw),
            (Some(_), None) => Some(EndState::Win(Color::White)),
            (None, Some(_)) => Some(EndState::Win(Color::Black)),
            (Some(white_king), Some(black_king)) => {
                let king = match self.current_player {
                    Color::White => white_king,
                    Color::Black => black_king,
                };
                if !king
                    .position
                    .king_moves()
                    .filter(|position| {
                        self[*position].is_none_or(|piece| piece.color != self.current_player)
                    })
                    .all(|position| {
                        self.is_attacked_after_move(king.position, position, !self.current_player)
                    })
                {
                    self.is_dead().then_some(EndState::Draw)
                } else if self.is_attacked_by(king.position, !self.current_player) {
                    (!self.has_valid_moves()).then_some(EndState::Win(!self.current_player))
                } else {
                    (self.is_dead() || !self.has_valid_moves()).then_some(EndState::Draw)
                }
            }
        }
    }
    fn is_dead(self) -> bool {
        [Color::White, Color::Black].into_iter().all(|color| {
            self.pieces_of(color)
                .try_fold(0, |num, piece| {
                    matches!(
                        piece.piece.kind,
                        PieceKind::Knight | PieceKind::Bishop | PieceKind::King
                    )
                    .then(|| num + 1)
                })
                .is_some_and(|count| count <= 2)
        })
    }
    pub fn move_piece(&mut self, movement: Move) {
        let piece_movement = movement.movement;

        self.current_player = !self.current_player;
        for piece in self.iter_mut() {
            piece.just_moved_twice_as_pawn = false;
        }
        let mut piece = self[piece_movement.origin]
            .take()
            .expect("origin position should contain a piece");
        let rook_and_destination = movement.castling_rook.map(|movement| {
            (
                self[movement.origin]
                    .take()
                    .expect("origin position should contain a piece"),
                movement.destination,
            )
        });

        piece.moved = true;
        if let Some(promoted_piece) = movement.promotion_piece {
            piece.kind = promoted_piece;
        }
        if piece.kind == PieceKind::Pawn
            && piece_movement.origin.x == piece_movement.destination.x
            && match piece.color {
                Color::White => piece_movement.origin.y == piece_movement.destination.y + 2,
                Color::Black => piece_movement.origin.y + 2 == piece_movement.destination.y,
            }
        {
            piece.just_moved_twice_as_pawn = true;
        }
        self[piece_movement.destination] = Some(piece);
        if let Some((mut rook, destination)) = rook_and_destination {
            rook.moved = true;
            self[destination] = Some(rook);
        }
        if let Some(captured) = movement.en_passant_capture {
            self[captured] = None;
        }
    }
    fn into_moved(self, movement: Move) -> Self {
        let mut moved = self;
        moved.move_piece(movement);
        moved
    }
    fn is_attacked_by(self, position: Coord, color: Color) -> bool {
        position
            .pawn_captures(!color)
            .any(|position| self.position_contains(position, color, [PieceKind::Pawn]))
            || position
                .knight_moves()
                .any(|position| self.position_contains(position, color, [PieceKind::Knight]))
            || position
                .bishop_lines()
                .any(|line| self.line_contains(line, color, [PieceKind::Bishop, PieceKind::Queen]))
            || position
                .rook_lines()
                .any(|line| self.line_contains(line, color, [PieceKind::Rook, PieceKind::Queen]))
            || position
                .king_moves()
                .any(|position| self.position_contains(position, color, [PieceKind::King]))
    }
    fn is_attacked_after_move(self, origin: Coord, position: Coord, color: Color) -> bool {
        let mut board = self;
        board[origin] = None;
        board.is_attacked_by(position, color)
    }
    fn moves(self) -> impl Iterator<Item = Move> {
        self.pieces_of(self.current_player)
            .flat_map(PieceWithContext::moves)
    }
    fn valid_moves(self) -> impl Iterator<Item = Move> {
        self.moves()
            .filter(move |movement| self.is_move_valid(*movement))
    }
    fn is_move_valid(self, movement: Move) -> bool {
        let current_player = self.current_player;
        let moved = self.into_moved(movement);
        !moved
            .king_of(current_player)
            .is_some_and(|king| moved.is_attacked_by(king.position, !current_player))
    }
    fn has_valid_moves(self) -> bool {
        self.valid_moves().next().is_some()
    }
    fn position_contains<const N: usize>(
        self,
        position: Coord,
        color: Color,
        pieces: [PieceKind; N],
    ) -> bool {
        self[position].is_some_and(|piece| piece.color == color && pieces.contains(&piece.kind))
    }
    fn line_contains<const N: usize>(
        self,
        mut line: impl Iterator<Item = Coord>,
        color: Color,
        pieces: [PieceKind; N],
    ) -> bool {
        line.find_map(|position| self[position])
            .is_some_and(|piece| piece.color == color && pieces.contains(&piece.kind))
    }
    fn moveable_position_on_line(
        self,
        line: impl Iterator<Item = Coord>,
        color: Color,
    ) -> impl Iterator<Item = Coord> {
        let mut stop_next = false;
        line.take_while(move |position| {
            if stop_next {
                false
            } else {
                self[*position].is_none_or(|piece| {
                    if piece.color == color {
                        false
                    } else {
                        stop_next = true;
                        true
                    }
                })
            }
        })
    }
    pub fn castling_rights(self) -> CastlingRights {
        let [
            (white_king_side, white_queen_side),
            (black_king_side, black_queen_side),
        ] = [Color::White, Color::Black].map(|color| {
            if let Some(king) = self.king_of(color) {
                if king.piece.moved {
                    (false, false)
                } else {
                    let mut king_side = false;
                    let mut queen_side = false;

                    for piece in self.pieces_of(color) {
                        if piece.piece.kind == PieceKind::Rook && !piece.piece.moved {
                            match Ord::cmp(&king.position.x, &piece.position.x) {
                                Ordering::Less => king_side = true,
                                Ordering::Equal => (),
                                Ordering::Greater => queen_side = true,
                            }
                            if king_side && queen_side {
                                break;
                            }
                        }
                    }
                    (king_side, queen_side)
                }
            } else {
                (false, false)
            }
        });
        CastlingRights {
            white_king_side,
            white_queen_side,
            black_king_side,
            black_queen_side,
        }
    }
    pub fn en_passant_destinations(self) -> impl Iterator<Item = Coord> {
        self.pieces()
            .filter(|piece| piece.piece.just_moved_twice_as_pawn)
            .map(|piece| {
                piece
                    .position
                    .move_by(0, -pawn_direction(piece.piece.color))
                    .expect("en passant destination shouldn't be out of bounds")
            })
    }
}
impl Index<Coord> for Board {
    type Output = Option<Piece>;

    fn index(&self, index: Coord) -> &Self::Output {
        &self.board[index.y as usize][index.x as usize]
    }
}
impl IndexMut<Coord> for Board {
    fn index_mut(&mut self, index: Coord) -> &mut Self::Output {
        &mut self.board[index.y as usize][index.x as usize]
    }
}
impl Default for Board {
    fn default() -> Self {
        let pieces = [
            PieceKind::Rook,
            PieceKind::Knight,
            PieceKind::Bishop,
            PieceKind::Queen,
            PieceKind::King,
            PieceKind::Bishop,
            PieceKind::Knight,
            PieceKind::Rook,
        ];
        Self {
            current_player: Color::White,
            board: [
                pieces.map(|kind| {
                    Some(Piece {
                        color: Color::Black,
                        kind,
                        moved: false,
                        just_moved_twice_as_pawn: false,
                    })
                }),
                [Some(Piece {
                    color: Color::Black,
                    kind: PieceKind::Pawn,
                    moved: false,
                    just_moved_twice_as_pawn: false,
                }); 8],
                [None; 8],
                [None; 8],
                [None; 8],
                [None; 8],
                [Some(Piece {
                    color: Color::White,
                    kind: PieceKind::Pawn,
                    moved: false,
                    just_moved_twice_as_pawn: false,
                }); 8],
                pieces.map(|kind| {
                    Some(Piece {
                        color: Color::White,
                        kind,
                        moved: false,
                        just_moved_twice_as_pawn: false,
                    })
                }),
            ],
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Coord {
    pub x: u8,
    pub y: u8,
}
impl Coord {
    fn dummy() -> Self {
        coord!("a8")
    }
    pub fn board_color(self) -> Color {
        match (self.x + self.y) % 2 {
            0 => Color::White,
            1 => Color::Black,
            _ => unreachable!(),
        }
    }
    fn is_valid(self) -> bool {
        self.x < 8 && self.y < 8
    }
    fn move_by(self, x: i8, y: i8) -> Option<Self> {
        let x = self.x.checked_add_signed(x)?;
        let y = self.y.checked_add_signed(y)?;
        Some(Coord { x, y }).filter(|position| position.is_valid())
    }
    fn king_moves(self) -> impl Iterator<Item = Self> {
        (-1..=1)
            .flat_map(|x| (-1..=1).map(move |y| (x, y)))
            .filter(|(x, y)| *x != 0 || *y != 0)
            .filter_map(move |(x, y)| self.move_by(x, y))
    }
    fn line(self, x: i8, y: i8) -> impl Iterator<Item = Self> {
        (1..).map_while(move |distance| self.move_by(x * distance, y * distance))
    }
    fn rook_lines(self) -> impl Iterator<Item = impl Iterator<Item = Self>> {
        [(-1, 0), (1, 0), (0, -1), (0, 1)]
            .into_iter()
            .map(move |(x, y)| self.line(x, y))
    }
    fn bishop_lines(self) -> impl Iterator<Item = impl Iterator<Item = Self>> {
        [(-1, -1), (1, -1), (-1, 1), (1, 1)]
            .into_iter()
            .map(move |(x, y)| self.line(x, y))
    }
    fn queen_lines(self) -> impl Iterator<Item = impl Iterator<Item = Self>> {
        (-1..=1)
            .flat_map(|x| (-1..=1).map(move |y| (x, y)))
            .filter(|(x, y)| *x != 0 || *y != 0)
            .map(move |(x, y)| self.line(x, y))
    }
    fn knight_moves(self) -> impl Iterator<Item = Self> {
        [(1, 2), (2, 1)]
            .into_iter()
            .flat_map(|(x, y)| [(x, y), (-x, y), (x, -y), (-x, -y)])
            .filter_map(move |(x, y)| self.move_by(x, y))
    }
    fn pawn_captures(self, color: Color) -> impl Iterator<Item = Self> {
        let y = pawn_direction(color);
        [(1, y), (-1, y)]
            .into_iter()
            .filter_map(move |(x, y)| self.move_by(x, y))
    }
    fn en_passant_target(self) -> impl Iterator<Item = Self> {
        [(1, 0), (-1, 0)]
            .into_iter()
            .filter_map(move |(x, y)| self.move_by(x, y))
    }
    pub fn from_char(x: char, y: char) -> Result<Self, ParseCoordError> {
        let x = match x {
            'a'..='h' => x as u8 - b'a',
            _ => return Err(ParseCoordError::InvalidX(x)),
        };
        let y = match y {
            '1'..='8' => 7 - (y as u8 - b'1'),
            _ => return Err(ParseCoordError::InvalidY(y)),
        };
        Ok(Coord { x, y })
    }
}
fn pawn_direction(color: Color) -> i8 {
    match color {
        Color::White => -1,
        Color::Black => 1,
    }
}
fn promotion_rank(color: Color) -> u8 {
    match color {
        Color::White => coord_y!("8"),
        Color::Black => coord_y!("1"),
    }
}
pub fn pawn_home_rank(color: Color) -> u8 {
    match color {
        Color::White => coord_y!("2"),
        Color::Black => coord_y!("7"),
    }
}
impl Display for Coord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x = (self.x + b'a') as char;
        let y = 8 - self.y;
        write!(f, "{x}{y}")?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseCoordError {
    Empty,
    YNotProvided,
    InvalidX(char),
    InvalidY(char),
    UnexpectedSymbol(char),
}
impl Display for ParseCoordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseCoordError::Empty => write!(f, "expected 2 characters, found none instead")?,
            ParseCoordError::YNotProvided => write!(f, "expected 2 characters, found 1 instead")?,
            ParseCoordError::InvalidX(c) => write!(f, "`{c}` is not a letter from a to h")?,
            ParseCoordError::InvalidY(c) => write!(f, "`{c}` is not a number from 1 to 8")?,
            ParseCoordError::UnexpectedSymbol(c) => {
                write!(f, "unexpected `{c}`, only 2 characters are expected")?;
            }
        }
        Ok(())
    }
}
impl Error for ParseCoordError {}

impl FromStr for Coord {
    type Err = ParseCoordError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut characters = s.chars();
        let x = characters.next().ok_or(ParseCoordError::Empty)?;
        let y = characters.next().ok_or(ParseCoordError::YNotProvided)?;
        let coord = Coord::from_char(x, y)?;
        if let Some(c) = characters.next() {
            return Err(ParseCoordError::UnexpectedSymbol(c));
        }
        Ok(coord)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PieceWithContext {
    pub piece: Piece,
    pub position: Coord,
    pub board: Board,
}
impl PieceWithContext {
    fn moves_from_positions(
        self,
        positions: impl Iterator<Item = Coord>,
    ) -> impl Iterator<Item = Move> {
        positions
            .filter(move |position| {
                self.board[*position].is_none_or(|piece| piece.color != self.piece.color)
            })
            .map(move |destination| SimpleMove {
                origin: self.position,
                destination,
            })
            .map(SimpleMove::as_simple_move)
    }
    fn moves_from_lines(
        self,
        lines: impl Iterator<Item = impl Iterator<Item = Coord>>,
    ) -> impl Iterator<Item = Move> {
        lines
            .flat_map(move |line| self.board.moveable_position_on_line(line, self.piece.color))
            .map(move |destination| SimpleMove {
                origin: self.position,
                destination,
            })
            .map(SimpleMove::as_simple_move)
    }
    fn pawn_moves(self) -> impl Iterator<Item = Move> {
        // regular forward move
        self.position
            .line(0, pawn_direction(self.piece.color))
            .take_while(move |position| self.board[*position].is_none())
            .take(if self.piece.moved { 1 } else { 2 })
            .chain(
                // regular capture moves
                self.position
                    .pawn_captures(self.piece.color)
                    .filter(move |position| {
                        self.board[*position].is_some_and(|piece| piece.color != self.piece.color)
                    }),
            )
            .flat_map(
                // turn into promotion if possible
                move |destination| {
                    if destination.y == promotion_rank(self.piece.color) {
                        [
                            PieceKind::Knight,
                            PieceKind::Bishop,
                            PieceKind::Rook,
                            PieceKind::Queen,
                        ]
                        .map(|promotion_piece| Move {
                            movement: SimpleMove {
                                origin: self.position,
                                destination,
                            },
                            castling_rook: None,
                            en_passant_capture: None,
                            promotion_piece: Some(promotion_piece),
                        })
                        .into_iter()
                        .take(4)
                    } else {
                        [
                            Move {
                                movement: SimpleMove {
                                    origin: self.position,
                                    destination,
                                },
                                castling_rook: None,
                                en_passant_capture: None,
                                promotion_piece: None,
                            },
                            Move::dummy(),
                            Move::dummy(),
                            Move::dummy(),
                        ]
                        .into_iter()
                        .take(1)
                    }
                },
            )
            .chain(
                // en passant
                self.position
                    .en_passant_target()
                    .filter(move |position| {
                        self.board[*position].is_some_and(|piece| {
                            piece.color != self.piece.color && piece.just_moved_twice_as_pawn
                        })
                    })
                    .filter_map(move |captured| {
                        Some(Move {
                            movement: SimpleMove {
                                origin: self.position,
                                destination: captured
                                    .move_by(0, pawn_direction(self.piece.color))?,
                            },
                            castling_rook: None,
                            en_passant_capture: Some(captured),
                            promotion_piece: None,
                        })
                    }),
            )
    }
    fn knight_moves(self) -> impl Iterator<Item = Move> {
        self.moves_from_positions(self.position.knight_moves())
    }
    fn bishop_moves(self) -> impl Iterator<Item = Move> {
        self.moves_from_lines(self.position.bishop_lines())
    }
    fn rook_moves(self) -> impl Iterator<Item = Move> {
        self.moves_from_lines(self.position.rook_lines())
    }
    fn queen_moves(self) -> impl Iterator<Item = Move> {
        self.moves_from_lines(self.position.queen_lines())
    }
    fn king_moves(self) -> impl Iterator<Item = Move> {
        // regular moves
        self.moves_from_positions(self.position.king_moves()).chain(
            // castling
            [-1, 1]
                .into_iter()
                .filter(move |_| !self.piece.moved)
                .filter_map(move |direction| {
                    self.position
                        .line(direction, 0)
                        .filter_map(|position| {
                            Some(PieceWithContext {
                                piece: self.board[position]?,
                                position,
                                board: self.board,
                            })
                        })
                        .find_map(|piece| {
                            (piece.piece.color == self.piece.color
                                && piece.piece.kind == PieceKind::Rook
                                && !piece.piece.moved)
                                .then(|| {
                                    let king_destination = match direction {
                                        -1 => coord_x!("c"),
                                        1 => coord_x!("g"),
                                        _ => unreachable!(),
                                    };
                                    let rook_destination = match direction {
                                        -1 => coord_x!("d"),
                                        1 => coord_x!("f"),
                                        _ => unreachable!(),
                                    };
                                    (
                                        SimpleMove {
                                            origin: self.position,
                                            destination: Coord {
                                                x: king_destination,
                                                y: self.position.y,
                                            },
                                        },
                                        SimpleMove {
                                            origin: piece.position,
                                            destination: Coord {
                                                x: rook_destination,
                                                y: piece.position.y,
                                            },
                                        },
                                    )
                                })
                        })
                })
                .filter(move |(king, rook)| {
                    [
                        (
                            PieceKind::Rook,
                            number_range_inclusive(rook.origin.x, rook.destination.x),
                        ),
                        (
                            PieceKind::King,
                            number_range_inclusive(king.origin.x, king.destination.x),
                        ),
                    ]
                    .into_iter()
                    .all(|(kind, mut range)| {
                        range.all(|x| {
                            let position = Coord {
                                x,
                                y: match kind {
                                    PieceKind::Rook => rook.origin.y,
                                    PieceKind::King => king.origin.y,
                                    _ => unreachable!(),
                                },
                            };
                            self.board[position].is_none_or(|piece| {
                                piece.color == self.piece.color
                                    && match piece.kind {
                                        PieceKind::Rook => position == rook.origin,
                                        PieceKind::King => position == king.origin,
                                        _ => false,
                                    }
                            }) && !(kind == PieceKind::King
                                && self.board.is_attacked_by(position, !self.piece.color))
                        })
                    })
                })
                .map(|(movement, castling_rook)| Move {
                    movement,
                    castling_rook: Some(castling_rook),
                    en_passant_capture: None,
                    promotion_piece: None,
                }),
        )
    }
    fn moves(self) -> Box<dyn Iterator<Item = Move>> {
        match self.piece.kind {
            PieceKind::Pawn => Box::new(self.pawn_moves()),
            PieceKind::Knight => Box::new(self.knight_moves()),
            PieceKind::Bishop => Box::new(self.bishop_moves()),
            PieceKind::Rook => Box::new(self.rook_moves()),
            PieceKind::Queen => Box::new(self.queen_moves()),
            PieceKind::King => Box::new(self.king_moves()),
        }
    }
    pub fn valid_moves(self) -> impl Iterator<Item = Move> {
        self.moves()
            .filter(move |movement| self.board.is_move_valid(*movement))
    }
}
impl Display for PieceWithContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} on {}", self.piece, self.position)?;
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SimpleMove {
    pub origin: Coord,
    pub destination: Coord,
}
impl Display for SimpleMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.origin, self.destination)?;
        Ok(())
    }
}
impl SimpleMove {
    fn dummy() -> Self {
        SimpleMove {
            origin: Coord::dummy(),
            destination: Coord::dummy(),
        }
    }
    fn as_simple_move(self) -> Move {
        Move {
            movement: self,
            castling_rook: None,
            en_passant_capture: None,
            promotion_piece: None,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Move {
    pub movement: SimpleMove,
    pub castling_rook: Option<SimpleMove>,
    pub en_passant_capture: Option<Coord>,
    pub promotion_piece: Option<PieceKind>,
}
impl Move {
    fn dummy() -> Self {
        SimpleMove::dummy().as_simple_move()
    }
}
impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.movement)?;
        if let Some(piece) = self.promotion_piece {
            write!(f, "{}", piece.lowercase())?;
        }
        Ok(())
    }
}
fn number_range_inclusive(a: u8, b: u8) -> RangeInclusive<u8> {
    Ord::min(a, b)..=Ord::max(a, b)
}
#[cfg(test)]
mod test {
    use crate::{
        chess::{Board, Color, Coord, EndState, PieceWithContext},
        coord,
        fen::Fen,
    };

    #[test]
    fn checkmate() {
        let Fen(board) = "8/8/R7/R3k3/R7/8/8/4K3 b - - 0 1".parse().unwrap();
        assert_eq!(board.state(), Some(EndState::Win(Color::White)));
    }
    #[test]
    fn stalemate() {
        let Fen(board) = "8/8/R7/4k3/R7/8/8/3RKR2 b - - 0 1".parse().unwrap();
        assert_eq!(board.state(), Some(EndState::Draw));
    }
    #[test]
    fn dead_position() {
        let Fen(board) = "3bk3/8/8/8/8/8/8/3NK3 w - - 0 1".parse().unwrap();
        assert_eq!(board.state(), Some(EndState::Draw));
    }
    #[test]
    fn en_passant() {
        let Fen(mut board) = "4k3/5p2/8/4P3/8/8/8/4K3 b - - 0 1".parse().unwrap();
        move_piece_with_assert(&mut board, coord!("f7"), coord!("f5"));
        assert!(board[coord!("f5")].unwrap().just_moved_twice_as_pawn);
        move_piece_with_assert(&mut board, coord!("e5"), coord!("f6"));
        assert!(board[coord!("f5")].is_none());
    }
    #[test]
    fn lose_of_en_passant_rights() {
        let Fen(mut board) = "4k3/3p1p2/8/8/4P3/8/8/4K3 b - - 0 1".parse().unwrap();
        move_piece_with_assert(&mut board, coord!("d7"), coord!("d5"));
        move_piece_with_assert(&mut board, coord!("e4"), coord!("e5"));
        move_piece_with_assert(&mut board, coord!("f7"), coord!("f5"));
        assert!(!board[coord!("d5")].unwrap().just_moved_twice_as_pawn);
        assert!(board[coord!("f5")].unwrap().just_moved_twice_as_pawn);
        move_piece_with_assert(&mut board, coord!("e1"), coord!("d1"));
        move_piece_with_assert(&mut board, coord!("e8"), coord!("d8"));
        assert!(!board[coord!("f5")].unwrap().just_moved_twice_as_pawn);
        assert_no_move(&mut board, coord!("e5"), coord!("f6"));
    }
    fn move_piece_with_assert(board: &mut Board, origin: Coord, destination: Coord) {
        let piece = board[origin].expect("origin position should contain a piece");
        assert_eq!(piece.color, board.current_player);
        let piece = PieceWithContext {
            piece,
            position: origin,
            board: *board,
        };
        board.move_piece(
            piece
                .valid_moves()
                .find(|movement| movement.movement.destination == destination)
                .unwrap(),
        );
    }
    fn assert_no_move(board: &mut Board, origin: Coord, destination: Coord) {
        let piece = board[origin].expect("origin position should contain a piece");
        assert_eq!(piece.color, board.current_player);
        let piece = PieceWithContext {
            piece,
            position: origin,
            board: *board,
        };
        assert!(
            piece
                .valid_moves()
                .find(|movement| {
                    movement.movement.origin == origin
                        && movement.movement.destination == destination
                })
                .is_none()
        )
    }
}
