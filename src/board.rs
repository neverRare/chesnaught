use std::{
    cell::OnceCell,
    cmp::Ordering,
    error::Error,
    fmt::{self, Display, Formatter},
    hash::Hash,
    iter::FusedIterator,
    num::NonZero,
    ops::{Index, IndexMut, Range},
    rc::Rc,
    str::FromStr,
};

use crate::{
    board_display::IndexableBoard,
    castling_right::CastlingRight,
    color::Color,
    coord::{Coord, ParseCoordError, Vector, home_rank, pawn_home_rank, pawn_promotion_rank},
    coord_x,
    end_state::EndState,
    error::InvalidByte,
    piece::{ColoredPieceKind, InvalidFenPiece, PieceKind},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InvalidBoard {
    ExceededPieces(ExceededPieces),
    NoKing,
    NonPlayerInCheck,
    MoreThanTwoCheckers,
    InvalidCastlingRight,
    InvalidEnPassantTarget,
}
impl From<ExceededPieces> for InvalidBoard {
    fn from(value: ExceededPieces) -> Self {
        InvalidBoard::ExceededPieces(value)
    }
}
impl Display for InvalidBoard {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            InvalidBoard::ExceededPieces(err) => write!(f, "{err}")?,
            InvalidBoard::NoKing => write!(f, "no kings found")?,
            InvalidBoard::NonPlayerInCheck => write!(f, "non-player in check")?,
            InvalidBoard::MoreThanTwoCheckers => {
                write!(f, "found more than 2 pieces delivering check")?;
            }
            InvalidBoard::InvalidCastlingRight => write!(f, "invalid castling right")?,
            InvalidBoard::InvalidEnPassantTarget => write!(f, "invalid en passant target")?,
        }
        Ok(())
    }
}
impl Error for InvalidBoard {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            InvalidBoard::ExceededPieces(err) => Some(err),
            _ => None,
        }
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
        let mut resume = true;
        self.position
            .line_exclusive(direction)
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
            .line_exclusive(Vector::pawn_single_move(self.piece.color()))
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
                            board
                                .get_index_with_kind(
                                    destination
                                        .move_by(Vector::pawn_single_move(!self.piece.color()))
                                        .unwrap(),
                                    !self.piece.color(),
                                    PieceKind::Pawn,
                                )
                                .unwrap()
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
    pub fn from_configuration(configuration: [PieceKind; 8]) -> Self {
        HashableBoard::from_configuration(configuration)
            .try_into()
            .unwrap()
    }
    pub fn current_player(&self) -> Color {
        self.current_player
    }
    pub fn as_hashable(&self) -> HashableBoard {
        let mut board = [[None; 8]; 8];
        for piece in self.all_pieces() {
            board[piece.position.y() as usize][piece.position.x() as usize] = Some(piece.piece);
        }
        HashableBoard {
            board,
            current_player: self.current_player,
            castling_right: self.castling_right,
            en_passant_target: self.en_passant_target,
        }
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
    fn get_index_with_kind(
        &self,
        position: Coord,
        color: Color,
        piece: PieceKind,
    ) -> Option<PieceIndex> {
        if let Some(indices) = self.indices.get() {
            indices[position.y() as usize * 8 + position.x() as usize]
        } else {
            self.pieces_by_kind_indexed(color, piece)
                .find(|(_, piece)| piece.position == position)
                .map(|(index, _)| index)
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
    pub fn validate(&self) -> Result<(), InvalidBoard> {
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
        pinned_position: Coord,
        color: Color,
        checker: impl Fn(Coord) -> bool + Clone,
    ) -> Option<impl Iterator<Item = Coord>> {
        let direction = pinned_position - king;
        if Vector::QUEEN_DIRECTIONS
            .into_iter()
            .any(|valid_direction| direction.is_aligned(valid_direction))
        {
            let direction = direction.as_unit();
            if pinned_position
                .line_exclusive_exclusive(king, -direction)
                .any(checker.clone())
            {
                None
            } else {
                let pieces = if Vector::BISHOP_DIRECTIONS.contains(&direction) {
                    &[PieceKind::Bishop, PieceKind::Queen]
                } else {
                    &[PieceKind::Rook, PieceKind::Queen]
                };
                self.pieces_by_kinds(!color, pieces)
                    .find_map(|pinning_piece| {
                        if direction.is_aligned(pinning_piece.position - king)
                            && pinned_position.is_inside_of(pinning_piece.position, king)
                        {
                            (!pinning_piece
                                .position
                                .line_exclusive_exclusive(pinned_position, -direction)
                                .any(checker.clone()))
                            .then(|| {
                                pinning_piece
                                    .position
                                    .line_inclusive_exclusive(king, -direction)
                            })
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
        pinned_position: Coord,
        color: Color,
    ) -> Option<impl Iterator<Item = Coord>> {
        self.pinned_with_inspect(king, pinned_position, color, |position| {
            self[position].is_some()
        })
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
                        .line_exclusive_inclusive(destination, (destination - origin).as_unit())
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
        let mut attackers_iter = self.attackers(king.position, !self.current_player).fuse();
        let attackers = [attackers_iter.next(), attackers_iter.next()];
        debug_assert_eq!(attackers_iter.next(), None);
        let check = attackers[0].is_some();
        let non_castling_moves = self
            .pieces_indexed(self.current_player)
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
                    !self.is_move_attacked(
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
    pub fn move_piece(&mut self, movement: &impl Moveable) {
        movement.move_board(self);
    }
    pub fn clone_and_move(&self, movement: &impl Moveable) -> Self {
        let mut new = self.clone();
        new.move_piece(movement);
        new
    }
    pub fn display_raw_moves(&self) {
        for (index, piece) in self.pieces_indexed(self.current_player) {
            println!("moves of {piece}:");
            for movement in piece.non_castling_moves(index, self) {
                println!("{}", movement.as_long_algebraic_notation(self));
            }
            println!();
        }
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
pub struct HashableBoard {
    pub board: [[Option<ColoredPieceKind>; 8]; 8],
    pub current_player: Color,
    pub castling_right: CastlingRight,
    pub en_passant_target: Option<Coord>,
}
impl HashableBoard {
    pub fn starting_position() -> Self {
        HashableBoard::from_configuration(PieceKind::STARTING_CONFIGURATION)
    }
    pub fn from_configuration(configuration: [PieceKind; 8]) -> Self {
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
    type Error = InvalidBoard;

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
                                return Err(ExceededPieces::Pawn.into());
                            }
                            return Err(ExceededPieces::PromotedPiece.into());
                        }
                        PieceKind::King => return Err(ExceededPieces::King.into()),
                        _ => return Err(ExceededPieces::PromotedPiece.into()),
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
        board.validate()?;
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
    pub fn as_long_algebraic_notation(self, board: &Board) -> LongAlgebraicNotation {
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
    pub fn as_long_algebraic_notation_chess_960(self, board: &Board) -> LongAlgebraicNotation {
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
        board.current_player = !board.current_player;

        board.indices = OnceCell::new();

        if cfg!(debug_assertions) {
            board.validate().unwrap();
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParseMoveError {
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
pub struct LongAlgebraicNotation {
    pub origin: Coord,
    pub destination: Coord,
    pub promotion: Option<PieceKind>,
}
impl LongAlgebraicNotation {
    pub fn as_move(self, board: &Board) -> Move {
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
#[cfg(test)]
mod test {
    use crate::{board::Board, coord, fen::Fen};

    #[test]
    fn pin() {
        let board: Fen = "4k3/4r3/8/8/8/8/4N3/4K3 w - - 0 1".parse().unwrap();
        let board: Board = board.board.try_into().unwrap();
        let valid_moves: Vec<_> = board
            .valid_moves()
            .unwrap()
            .map(|movement| movement.as_long_algebraic_notation(&board))
            .collect();
        assert!(
            valid_moves
                .iter()
                .find(|movement| movement.origin == coord!("e2"))
                .is_none()
        );
    }
}
