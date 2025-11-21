use std::{
    error::Error,
    fmt::Display,
    iter::{once, repeat},
    ops::{Index, IndexMut, Not, RangeInclusive},
    str::FromStr,
};

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
    // fn uppercase(self) -> char {
    //     self.lowercase().to_ascii_uppercase()
    // }
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
                write!(f, "unexpected `{c}`, only one character is expected")?
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
impl Color {
    fn is_white(self) -> bool {
        self == Color::Black
    }
    fn is_black(self) -> bool {
        self == Color::Black
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Board {
    pub current_player: Color,
    pub board: [[Option<Piece>; 8]; 8],
}
impl Board {
    pub fn new() -> Self {
        Default::default()
    }
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
    //         .flat_map(Option::into_iter)
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
                    .all(|position| self.is_attacked_by(position, !self.current_player))
                {
                    if self.is_dead() {
                        Some(EndState::Draw)
                    } else {
                        None
                    }
                } else if self.is_attacked_by(king.position, !self.current_player) {
                    if self.valid_moves().next().is_none() {
                        Some(EndState::Win(!self.current_player))
                    } else {
                        None
                    }
                } else if self.is_dead() || self.valid_moves().next().is_none() {
                    Some(EndState::Draw)
                } else {
                    None
                }
            }
        }
    }
    fn is_dead(self) -> bool {
        [Color::White, Color::Black].into_iter().all(|color| {
            self.pieces_of(color)
                .try_fold(0, |num, piece| {
                    if matches!(
                        piece.piece.kind,
                        PieceKind::Knight | PieceKind::Bishop | PieceKind::King
                    ) {
                        Some(num + 1)
                    } else {
                        None
                    }
                })
                .is_some_and(|count| count <= 2)
        })
    }
    pub fn move_piece(&mut self, movement: Move) {
        self.current_player = !self.current_player;
        for piece in self.iter_mut() {
            piece.just_moved_twice_as_pawn = false;
        }
        match movement {
            Move::RegularMove {
                origin,
                destination,
            } => {
                self[destination] = self[origin].take();

                if let Some(piece) = &mut self[destination] {
                    piece.moved = true;
                    if piece.kind == PieceKind::Pawn
                        && origin.x == destination.x
                        && ((piece.color.is_white() && origin.y + 2 == destination.y)
                            || (piece.color.is_black() && origin.y == destination.y + 2))
                    {
                        piece.just_moved_twice_as_pawn = true;
                    }
                }
            }
            Move::Castle(CastleMove {
                king_origin,
                king_destination,
                rook_origin,
                rook_destination,
            }) => {
                let mut king = self[king_origin].take();
                let mut rook = self[rook_origin].take();

                if let Some(piece) = &mut king {
                    piece.moved = true;
                }
                if let Some(piece) = &mut rook {
                    piece.moved = true;
                }
                self[king_destination] = king;
                self[rook_destination] = rook;
            }
            Move::EnPassant {
                pawn_origin,
                pawn_destination,
                captured_pawn,
            } => {
                self[pawn_destination] = self[pawn_origin].take();

                if let Some(piece) = &mut self[pawn_destination] {
                    piece.moved = true;
                }
                self[captured_pawn] = None;
            }
            Move::Promotion {
                origin,
                destination,
                kind,
            } => {
                if let Some(piece) = &mut self[destination] {
                    piece.moved = true;
                    piece.kind = kind;
                    if origin.x == destination.x
                        && ((piece.color.is_white() && origin.y + 2 == destination.y)
                            || (piece.color.is_black() && origin.y == destination.y + 2))
                    {
                        piece.just_moved_twice_as_pawn = true;
                    }
                }
            }
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
            .any(|position| self.square_contains(position, color, [PieceKind::Pawn]))
            || position
                .knight_moves()
                .any(|position| self.square_contains(position, color, [PieceKind::Knight]))
            || position
                .bishop_lines()
                .any(|line| self.line_contains(line, color, [PieceKind::Bishop, PieceKind::Queen]))
            || position
                .rook_lines()
                .any(|line| self.line_contains(line, color, [PieceKind::Rook, PieceKind::Queen]))
            || position
                .king_moves()
                .any(|position| self.square_contains(position, color, [PieceKind::King]))
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
    fn square_contains<const N: usize>(
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
    fn possible_squares_on_line(
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
}
impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (piece, first) in self.pieces().zip(once(true).chain(repeat(false))) {
            if first {
                write!(f, ", ")?;
            }
            write!(f, "{}", piece)?;
        }
        Ok(())
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
        Coord { x: 0, y: 0 }
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
            .filter_map(move |(x, y)| {
                if x == 0 && y == 0 {
                    None
                } else {
                    self.move_by(x, y)
                }
            })
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
        Color::White => 0,
        Color::Black => 7,
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
                write!(f, "unexpected `{c}`, only 2 characters are expected")?
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
    fn moves(self) -> Box<dyn Iterator<Item = Move>> {
        match self.piece.kind {
            PieceKind::Pawn => {
                Box::new(
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
                                    self.board[*position]
                                        .is_some_and(|piece| piece.color != self.piece.color)
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
                                    .map(|kind| Move::Promotion {
                                        origin: self.position,
                                        destination,
                                        kind,
                                    })
                                    .into_iter()
                                    .take(4)
                                } else {
                                    [
                                        Move::RegularMove {
                                            origin: self.position,
                                            destination,
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
                                        piece.color != self.piece.color
                                            && piece.just_moved_twice_as_pawn
                                    })
                                })
                                .filter_map(move |captured_pawn| {
                                    Some(Move::EnPassant {
                                        pawn_origin: self.position,
                                        pawn_destination: captured_pawn
                                            .move_by(0, pawn_direction(self.piece.color))?,
                                        captured_pawn,
                                    })
                                }),
                        ),
                )
            }
            PieceKind::Knight => Box::new(
                self.position
                    .knight_moves()
                    .filter(move |position| {
                        self.board[*position].is_none_or(|piece| piece.color != self.piece.color)
                    })
                    .map(move |destination| Move::RegularMove {
                        origin: self.position,
                        destination,
                    }),
            ),
            PieceKind::Bishop => Box::new(
                self.position
                    .bishop_lines()
                    .flat_map(move |line| {
                        self.board.possible_squares_on_line(line, self.piece.color)
                    })
                    .map(move |destination| Move::RegularMove {
                        origin: self.position,
                        destination,
                    }),
            ),
            PieceKind::Rook => Box::new(
                self.position
                    .rook_lines()
                    .flat_map(move |line| {
                        self.board.possible_squares_on_line(line, self.piece.color)
                    })
                    .map(move |destination| Move::RegularMove {
                        origin: self.position,
                        destination,
                    }),
            ),
            PieceKind::Queen => Box::new(
                self.position
                    .queen_lines()
                    .flat_map(move |line| {
                        self.board.possible_squares_on_line(line, self.piece.color)
                    })
                    .map(move |destination| Move::RegularMove {
                        origin: self.position,
                        destination,
                    }),
            ),
            PieceKind::King => {
                Box::new(
                    self.position
                        .king_moves()
                        .filter(move |position| {
                            self.board[*position]
                                .is_none_or(|piece| piece.color != self.piece.color)
                        })
                        .map(move |destination| Move::RegularMove {
                            origin: self.position,
                            destination,
                        })
                        .chain(
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
                                        .next()
                                        .and_then(|piece| {
                                            if piece.piece.color == self.piece.color
                                                && piece.piece.kind == PieceKind::Rook
                                                && !piece.piece.moved
                                            {
                                                let king_destination = match direction {
                                                    -1 => 2,
                                                    1 => 6,
                                                    _ => unreachable!(),
                                                };
                                                let rook_destination = match direction {
                                                    -1 => 3,
                                                    1 => 5,
                                                    _ => unreachable!(),
                                                };
                                                Some(CastleMove {
                                                    king_origin: self.position,
                                                    king_destination: Coord {
                                                        x: king_destination,
                                                        y: self.position.y,
                                                    },
                                                    rook_origin: piece.position,
                                                    rook_destination: Coord {
                                                        x: rook_destination,
                                                        y: piece.position.y,
                                                    },
                                                })
                                            } else {
                                                None
                                            }
                                        })
                                })
                                .filter(move |castle_move| {
                                    [
                                        (
                                            PieceKind::Rook,
                                            number_range_inclusive(
                                                castle_move.rook_origin.x,
                                                castle_move.rook_destination.x,
                                            ),
                                        ),
                                        (
                                            PieceKind::King,
                                            number_range_inclusive(
                                                castle_move.king_origin.x,
                                                castle_move.king_destination.x,
                                            ),
                                        ),
                                    ]
                                    .into_iter()
                                    .all(
                                        |(kind, mut range)| {
                                            range.all(|x| {
                                                let position = Coord {
                                                    x,
                                                    y: match kind {
                                                        PieceKind::Rook => {
                                                            castle_move.rook_origin.y
                                                        }
                                                        PieceKind::King => {
                                                            castle_move.king_origin.y
                                                        }
                                                        _ => unreachable!(),
                                                    },
                                                };
                                                self.board[position].is_none_or(|piece| {
                                                    piece.color == self.piece.color
                                                        && match piece.kind {
                                                            PieceKind::Rook => {
                                                                position == castle_move.rook_origin
                                                            }
                                                            PieceKind::King => {
                                                                position == castle_move.king_origin
                                                            }
                                                            _ => false,
                                                        }
                                                }) && !(kind == PieceKind::King
                                                    && self.board.is_attacked_by(
                                                        position,
                                                        !self.piece.color,
                                                    ))
                                            })
                                        },
                                    )
                                })
                                .map(Move::Castle),
                        ),
                )
            }
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
pub struct CastleMove {
    king_origin: Coord,
    king_destination: Coord,
    rook_origin: Coord,
    rook_destination: Coord,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Move {
    RegularMove {
        origin: Coord,
        destination: Coord,
    },
    Castle(CastleMove),
    EnPassant {
        pawn_origin: Coord,
        pawn_destination: Coord,
        captured_pawn: Coord,
    },
    Promotion {
        origin: Coord,
        destination: Coord,
        kind: PieceKind,
    },
}
impl Move {
    fn dummy() -> Self {
        Move::RegularMove {
            origin: Coord::dummy(),
            destination: Coord::dummy(),
        }
    }
    pub fn destination(self) -> Coord {
        match self {
            Move::RegularMove {
                origin: _,
                destination,
            } => destination,
            Move::Castle(castle_move) => castle_move.king_destination,
            Move::EnPassant {
                pawn_origin: _,
                pawn_destination,
                captured_pawn: _,
            } => pawn_destination,
            Move::Promotion {
                origin: _,
                destination,
                kind: _,
            } => destination,
        }
    }
    pub fn promotion_piece(self) -> Option<PieceKind> {
        match self {
            Move::Promotion {
                origin: _,
                destination: _,
                kind,
            } => Some(kind),
            _ => None,
        }
    }
}
impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Move::RegularMove {
                origin,
                destination,
            } => {
                write!(f, "{origin}{destination}")?;
            }
            Move::Castle(CastleMove {
                king_origin,
                king_destination,
                rook_origin: _,
                rook_destination: _,
            }) => {
                write!(f, "{king_origin}{king_destination}")?;
            }
            Move::EnPassant {
                pawn_origin,
                pawn_destination,
                captured_pawn: _,
            } => {
                write!(f, "{pawn_origin}{pawn_destination}")?;
            }
            Move::Promotion {
                origin,
                destination,
                kind,
            } => {
                write!(f, "{origin}{destination}{}", kind.lowercase())?;
            }
        }
        Ok(())
    }
}
fn number_range_inclusive(a: u8, b: u8) -> RangeInclusive<u8> {
    Ord::min(a, b)..=Ord::max(a, b)
}
