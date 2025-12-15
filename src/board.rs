use std::{
    cell::OnceCell,
    cmp::Ordering,
    collections::HashSet,
    error::Error,
    fmt::{self, Display, Formatter},
    hash::Hash,
    iter::{FusedIterator, once},
    num::NonZero,
    ops::{Index, IndexMut, Range},
    rc::Rc,
    str::FromStr,
};

use crate::{
    board_display::IndexableBoard,
    castling_right::CastlingRight,
    color::Color,
    coord::{Coord, ParseCoordError, Vector},
    end_state::EndState,
    misc::InvalidByte,
    piece::{ColoredPieceKind, InvalidFenPiece, PieceKind},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InvalidBoard {
    ExceededPieces(ExceededPieces),
    NoKing,
    NonPlayerInCheck,
    MoreThanTwoCheckers,
    RookNotFound,
    InvalidEnPassantRank,
    EnPassantPawnNotFound,
    PawnOnHomeRank,
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
            InvalidBoard::RookNotFound => write!(f, "rook not found for castling right")?,
            InvalidBoard::InvalidEnPassantRank => {
                write!(f, "en passant target may only be on ranks 3 or 6")?;
            }
            InvalidBoard::EnPassantPawnNotFound => {
                write!(f, "pawn in front of en passant target is not found")?;
            }
            InvalidBoard::PawnOnHomeRank => write!(f, "pawns cannot be on ranks 1 or 8")?,
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
    pub fn color(self) -> Color {
        self.piece.color()
    }
    pub fn piece(self) -> PieceKind {
        self.piece.piece()
    }
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
                if let Some((capture, piece)) = board.index_and_piece(destination) {
                    (piece.color() != self.color()).then_some({
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
                    if let Some((capture, piece)) = board.index_and_piece(destination) {
                        resume = false;
                        (piece.color() != self.color()).then_some({
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
        let forward_jumps = if self.position.y() == Coord::pawn_home_rank(self.color()) {
            2
        } else {
            1
        };
        self.position
            .line_exclusive(Vector::pawn_single_move(self.color()))
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
                    == Vector::pawn_double_move(self.color()))
                .then(|| {
                    self.position
                        .move_by(Vector::pawn_single_move(self.color()))
                        .unwrap()
                })
                .filter(|en_passant_target| {
                    board.can_attack_by_pawn(*en_passant_target, !self.color())
                }),
                castling_right: board.castling_right,
            })
            .chain(
                Vector::pawn_attacks(self.color())
                    .into_iter()
                    .filter_map(move |movement| self.position.move_by(movement))
                    .filter_map(move |destination| {
                        let capture = if Some(destination) == board.en_passant_target {
                            board
                                .get_index_with_kind(
                                    destination
                                        .move_by(Vector::pawn_single_move(!self.color()))
                                        .unwrap(),
                                    !self.color(),
                                    PieceKind::Pawn,
                                )
                                .expect("pawn that performed double move not found")
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
                let promotion_choices: &'static [_] = if movement.movement.destination.y()
                    == Coord::pawn_promotion_rank(self.color())
                {
                    &[
                        Some(PieceKind::Queen),
                        Some(PieceKind::Rook),
                        Some(PieceKind::Bishop),
                        Some(PieceKind::Knight),
                    ]
                } else {
                    &[None]
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
        let moves: Box<dyn Iterator<Item = Move>> = match self.piece() {
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
                let castling_right = if self.position.y() == Coord::home_rank(self.color()) {
                    board
                        .castling_right
                        .to_removed(self.color(), self.position.x())
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
                let castling_right = board.castling_right.to_cleared(self.color());
                Box::new(
                    self.step_moves(index, board, &Vector::KING_MOVES)
                        .map(move |movement| movement.to_simple_move(castling_right)),
                )
            }
        };
        moves.map(move |movement| {
            if let Some(capture) = movement.movement.capture {
                Move {
                    castling_right: movement.castling_right.to_removed_for_rook_capture(
                        board[capture].expect("captured piece not found"),
                    ),
                    ..movement
                }
            } else {
                movement
            }
        })
    }
    fn can_be_blocked(self, target: Coord, blocker: Coord) -> bool {
        self.position == blocker
            || (matches!(
                self.piece(),
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
        (Color::White, PieceKind::King) => 0..1,
        (Color::White, PieceKind::Queen) => 1..2,
        (Color::White, PieceKind::Rook) => 2..4,
        (Color::White, PieceKind::Bishop) => 4..6,
        (Color::White, PieceKind::Knight) => 6..8,
        (Color::White, PieceKind::Pawn) => 8..16,
        (Color::Black, PieceKind::King) => 16..17,
        (Color::Black, PieceKind::Queen) => 17..18,
        (Color::Black, PieceKind::Rook) => 18..20,
        (Color::Black, PieceKind::Bishop) => 20..22,
        (Color::Black, PieceKind::Knight) => 22..24,
        (Color::Black, PieceKind::Pawn) => 24..32,
    }
}
impl Board {
    pub fn starting_position() -> Self {
        HashableBoard::from_configuration(PieceKind::STARTING_CONFIGURATION)
            .try_into()
            .unwrap()
    }
    pub fn chess960(id: u16) -> Self {
        HashableBoard::from_configuration(PieceKind::chess960(id))
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
    pub fn index(&self, position: Coord) -> Option<ColoredPieceKind> {
        self[position].map(|index| self[index].unwrap().piece)
    }
    fn index_and_piece(&self, position: Coord) -> Option<(PieceIndex, ColoredPieceKind)> {
        self[position].map(|index| (index, self[index].unwrap().piece))
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
                .filter(move |item| item.piece() == piece),
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
                .filter(move |(_, item)| item.piece() == piece),
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
                    .filter(move |piece| pieces.contains(&piece.piece())),
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
                b.color() == color && b.piece() == piece
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
                            b.color() == color && b.piece() == piece
                        },
                    )
                })
        } else {
            self.pieces_by_kind(color, piece)
                .any(|piece| moves.contains(&(piece.position - position)))
        }
    }
    fn can_attack_by_pawn(&self, position: Coord, color: Color) -> bool {
        self.any_moves_has(
            position,
            &Vector::pawn_attacks(!color),
            color,
            PieceKind::Pawn,
        )
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
            Color::Black => self.pieces[16].map(|piece| (16.try_into().unwrap(), piece)),
        }
    }
    fn pawns(&self, color: Color) -> impl Iterator<Item = Piece> {
        self.range(original_piece_range(color, PieceKind::Pawn))
            .filter(|item| item.piece() == PieceKind::Pawn)
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
            let king_on_home = king.position.y() == Coord::home_rank(color);
            self.castling_right.all(color).all(|x| {
                king_on_home
                    && self
                        .get_with_kind_indexed(
                            Coord::new(x, Coord::home_rank(color)),
                            color,
                            PieceKind::Rook,
                        )
                        .is_some()
            })
        }) {
            return Err(InvalidBoard::RookNotFound);
        }
        if let Some(en_passant_target) = self.en_passant_target {
            let (color, pawn_position) = en_passant_target
                .pawn_from_en_passant_target()
                .ok_or(InvalidBoard::InvalidEnPassantRank)?;
            if !self.position_has(pawn_position, color, PieceKind::Pawn) {
                return Err(InvalidBoard::EnPassantPawnNotFound);
            }
        }
        for pawn in self.pawns(Color::White).chain(self.pawns(Color::Black)) {
            if Coord::HOME_RANKS.contains(&pawn.position.y()) {
                return Err(InvalidBoard::PawnOnHomeRank);
            }
        }
        Ok(())
    }
    fn attackers_with_inspect(
        &self,
        position: Coord,
        color: Color,
        checker: impl Fn(Coord) -> bool + Clone,
    ) -> impl FusedIterator<Item = Piece> {
        self.pieces(color).filter(move |piece| match piece.piece() {
            PieceKind::Pawn => (position - piece.position).is_pawn_attack(color),
            PieceKind::Knight => (position - piece.position).is_knight_move(),
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
            PieceKind::King => (position - piece.position).is_king_move(),
        })
    }
    fn attackers(&self, position: Coord, color: Color) -> impl FusedIterator<Item = Piece> {
        self.attackers_with_inspect(position, color, |position| self[position].is_some())
    }
    fn is_move_attacked(&self, indices: &[PieceIndex], destination: Coord, color: Color) -> bool {
        self.attackers_with_inspect(destination, color, |position| {
            self[position].is_some_and(|index| !indices.contains(&index))
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
            .try_fold(None, |piece_left, piece| match piece.piece() {
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
        let (king_index, king) = self
            .king_indexed(self.current_player)
            .expect("king not found");
        let castling_right = self.castling_right;
        let new_castling_right = castling_right.to_cleared(self.current_player);
        castling_right
            .all(self.current_player)
            .filter(move |_| !check)
            .filter_map(move |x| {
                let (rook_index, rook) = self
                    .get_with_kind_indexed(
                        Coord::new(x, Coord::home_rank(self.current_player)),
                        self.current_player,
                        PieceKind::Rook,
                    )
                    .expect("rook not found");
                let (king_destination, rook_destination) =
                    match Ord::cmp(&king.position.x(), &rook.position.x()) {
                        Ordering::Less => (
                            Coord::new(
                                Coord::CASTLING_KING_DESTINATION_KINGSIDE,
                                Coord::home_rank(self.current_player),
                            ),
                            Coord::new(
                                Coord::CASTLING_ROOK_DESTINATION_KINGSIDE,
                                Coord::home_rank(self.current_player),
                            ),
                        ),

                        Ordering::Equal => unreachable!(),
                        Ordering::Greater => (
                            Coord::new(
                                Coord::CASTLING_KING_DESTINATION_QUEENSIDE,
                                Coord::home_rank(self.current_player),
                            ),
                            Coord::new(
                                Coord::CASTLING_ROOK_DESTINATION_QUEENSIDE,
                                Coord::home_rank(self.current_player),
                            ),
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
                    let direction = (destination - origin).as_unit();
                    debug_assert_eq!(
                        direction.y, 0,
                        "{origin} and {destination} are not in the same rank",
                    );
                    origin
                        .line_exclusive_inclusive(destination, direction)
                        .all(|position| {
                            (position == other_position || self[position].is_none())
                                && (piece != PieceKind::King
                                    || !self.is_move_attacked(
                                        &[king_index, rook_index],
                                        position,
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
        let king = self.king(self.current_player).expect("king not found");
        let mut attackers_iter = self.attackers(king.position, !self.current_player).fuse();
        let attackers = [attackers_iter.next(), attackers_iter.next()];
        debug_assert_eq!(
            attackers_iter.next(),
            None,
            "more than 2 pieces checking the king"
        );
        let check = attackers[0].is_some();
        let non_castling_moves = self
            .pieces_indexed(self.current_player)
            .flat_map(move |(index, piece)| {
                let valid_destination_when_pinned: Option<Rc<[_]>> =
                    if piece.piece() == PieceKind::King {
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
                if piece.piece() == PieceKind::King {
                    !self.is_move_attacked(
                        &[movement.movement.index],
                        movement.movement.destination,
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
                            if let Some(index) = movement.movement.capture {
                                let captured = self[index].expect("captured piece not found");

                                // check if the captured piece is also located
                                // on the piece's destination

                                // otherwise, it's en passant and check for the pin
                                (captured.position == movement.movement.destination)
                                    || self
                                        .pinned_with_inspect(
                                            king.position,
                                            captured.position,
                                            piece.color(),
                                            |position| {
                                                position != piece.position
                                                    && (position == movement.movement.destination
                                                        || self[position].is_some())
                                            },
                                        )
                                        .is_none()
                            } else {
                                true
                            }
                        }
                }
            })
            .map(|(movement, _, _)| movement);
        (non_castling_moves.chain(self.castling_moves(check)), check)
    }
    pub fn move_piece(&mut self, movement: &impl Moveable) {
        let movement = movement.as_move(self);
        let current_player = self.current_player;
        let piece = self[movement.movement.index]
            .as_mut()
            .expect("piece not found");
        piece.position = movement.movement.destination;
        if let Some(promotion) = movement.promotion {
            piece.piece = ColoredPieceKind::new(current_player, promotion);
        }
        if let Some(index) = movement.movement.capture {
            self[index] = None;
        }
        if let Some(movement) = movement.castling_rook {
            let rook = self[movement.index].as_mut().expect("rook not found");
            rook.position = movement.destination;
        }
        self.en_passant_target = movement.en_passant_target;
        self.castling_right = movement.castling_right;
        self.current_player = !self.current_player;

        self.indices = OnceCell::new();

        if cfg!(debug_assertions) {
            self.validate().unwrap();
        }
    }
    pub fn clone_and_move(&self, movement: &impl Moveable) -> Self {
        let mut new = self.clone();
        new.move_piece(movement);
        new
    }
    pub fn move_assert(&mut self, lan: Lan) {
        let valid_moves: HashSet<_> = self.valid_moves().into_iter().flatten().collect();
        let movement = lan.as_move(self);
        assert!(
            valid_moves.contains(&movement),
            "`{lan}` is an invalid move"
        );
        self.move_piece(&movement);
    }
    pub fn assert_piece_cant_move(&self, position: Coord) {
        let valid_moves: Vec<_> = self.valid_moves().into_iter().flatten().collect();
        assert!(
            !valid_moves
                .into_iter()
                .any(|movement| self[movement.movement.index].unwrap().position == position),
            "found valid move for piece in position {position}",
        );
    }
    pub fn assert_move_is_valid(&self, lan: Lan) {
        let valid_moves: HashSet<_> = self.valid_moves().into_iter().flatten().collect();
        let movement = lan.as_move(self);
        assert!(
            valid_moves.contains(&movement),
            "`{lan}` is an invalid move"
        );
    }
    pub fn assert_move_is_invalid(&self, lan: Lan) {
        let valid_moves: HashSet<_> = self
            .valid_moves()
            .into_iter()
            .flatten()
            .flat_map(|movement| movement.as_lan_iter(self))
            .collect();
        assert!(!valid_moves.contains(&lan), "`{lan}` is a valid move");
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
        self.index(position)
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
    pub fn chess960(id: u16) -> Self {
        HashableBoard::from_configuration(PieceKind::chess960(id))
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
    pub fn fix_castling_rights(&mut self) {
        for color in [Color::White, Color::Black] {
            let row = self.board[Coord::home_rank(color) as usize];
            if let Some(king) = row
                .into_iter()
                .position(|x| x == Some(ColoredPieceKind::new(color, PieceKind::King)))
            {
                let king: u8 = king.try_into().unwrap();

                for x in self.castling_right.all(color) {
                    if row[x as usize] != Some(ColoredPieceKind::new(color, PieceKind::Rook)) {
                        let king_rook_ord = Ord::cmp(&king, &x);
                        let range = match king_rook_ord {
                            Ordering::Less => (king + 1)..=Coord::LAST_FILE,
                            Ordering::Equal => {
                                self.castling_right.remove(color, x);
                                continue;
                            }
                            Ordering::Greater => Coord::FIRST_FILE..=(king - 1),
                        };
                        let mut rooks = range.filter(|x| {
                            row[*x as usize] == Some(ColoredPieceKind::new(color, PieceKind::Rook))
                        });
                        let rook = match king_rook_ord {
                            Ordering::Less => rooks.next_back(),
                            Ordering::Equal => unreachable!(),
                            Ordering::Greater => rooks.next(),
                        };
                        let Some(new_x) = rook else {
                            self.castling_right.remove(color, x);
                            continue;
                        };
                        self.castling_right.remove(color, x);
                        self.castling_right.add(color, new_x);
                    }
                }
            } else {
                self.castling_right.clear(color);
            }
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
                                .all(|piece| piece.unwrap().piece() == PieceKind::Pawn)
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
        if let Some(en_passant_target) = board.en_passant_target {
            let color = Coord::en_passant_target_color(en_passant_target.y())
                .ok_or(InvalidBoard::InvalidEnPassantRank)?;
            if !board.can_attack_by_pawn(en_passant_target, !color) {
                board.en_passant_target = None;
            }
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
    fn as_move(&self, board: &Board) -> Move;
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
    fn as_ambiguous_lan_pair(self, board: &Board) -> (Lan, Option<Lan>) {
        let piece = board[self.movement.index].expect("piece not found");
        (
            Lan {
                origin: piece.position,
                destination: self.movement.destination,
                promotion: self.promotion,
            },
            self.castling_rook.map(|rook| Lan {
                origin: piece.position,
                destination: board[rook.index]
                    .expect("captured piece not found")
                    .position,
                promotion: self.promotion,
            }),
        )
    }
    pub fn as_lan_pair(self, board: &Board) -> (Lan, Option<Lan>) {
        let (regular, chess960) = self.as_ambiguous_lan_pair(board);
        if let Some(chess960) = chess960 {
            if regular.origin.x() == Coord::KING_ORIGIN
                && Coord::ROOK_ORIGINS.contains(&chess960.destination.x())
            {
                (regular, Some(chess960))
            } else if (regular.destination - regular.origin).is_king_move() {
                (chess960, None)
            } else {
                (chess960, Some(regular))
            }
        } else {
            (regular, None)
        }
    }
    pub fn as_lan_iter(self, board: &Board) -> impl Iterator<Item = Lan> {
        let (first, second) = self.as_lan_pair(board);
        once(first).chain(second)
    }
    pub fn as_lan(self, board: &Board) -> Lan {
        let (regular, chess960) = self.as_ambiguous_lan_pair(board);
        if let Some(chess960) = chess960
            && regular.origin.x() == Coord::KING_ORIGIN
            && Coord::ROOK_ORIGINS.contains(&chess960.destination.x())
        {
            regular
        } else {
            chess960.unwrap_or(regular)
        }
    }
    pub fn as_lan_chess960(self, board: &Board) -> Lan {
        let (regular, chess960) = self.as_ambiguous_lan_pair(board);
        chess960.unwrap_or(regular)
    }
}
impl Moveable for Move {
    fn as_move(&self, _: &Board) -> Move {
        *self
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParseLanError {
    InvalidChar,
    ParseCoordError(ParseCoordError),
    InvalidFenPiece(InvalidFenPiece),
    Unexpected(char),
}
impl From<ParseCoordError> for ParseLanError {
    fn from(value: ParseCoordError) -> Self {
        ParseLanError::ParseCoordError(value)
    }
}
impl From<InvalidFenPiece> for ParseLanError {
    fn from(value: InvalidFenPiece) -> Self {
        ParseLanError::InvalidFenPiece(value)
    }
}
impl Display for ParseLanError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ParseLanError::InvalidChar => write!(f, "provided string contains invalid character")?,
            ParseLanError::ParseCoordError(err) => write!(f, "{err}")?,
            ParseLanError::InvalidFenPiece(err) => write!(f, "{err}")?,
            ParseLanError::Unexpected(c) => write!(f, "unexpected `{c}`")?,
        }
        Ok(())
    }
}
impl Error for ParseLanError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ParseLanError::ParseCoordError(err) => Some(err),
            ParseLanError::InvalidFenPiece(err) => Some(err),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Lan {
    pub origin: Coord,
    pub destination: Coord,
    pub promotion: Option<PieceKind>,
}
impl Lan {
    #[allow(
        clippy::too_many_lines,
        reason = "I hope the provided comments are enough"
    )]
    pub fn as_move(self, board: &Board) -> Move {
        let (index, piece) = board.index_and_piece(self.origin).expect("piece not found");
        let capture = board[self.destination];

        let movement;
        let castling_rook;
        let castling_right;

        // Handle castling
        if let Some(rook) = capture
            && board[rook].unwrap().piece == ColoredPieceKind::new(piece.color(), PieceKind::Rook)
        {
            // "King takes rook" castling configuration e.g. e1h1
            let (king_destination, rook_destination) =
                match Ord::cmp(&self.origin.x(), &self.destination.x()) {
                    Ordering::Less => (
                        Coord::CASTLING_KING_DESTINATION_KINGSIDE,
                        Coord::CASTLING_ROOK_DESTINATION_KINGSIDE,
                    ),
                    Ordering::Equal => panic!("king and rook on the same file"),
                    Ordering::Greater => (
                        Coord::CASTLING_KING_DESTINATION_QUEENSIDE,
                        Coord::CASTLING_ROOK_DESTINATION_QUEENSIDE,
                    ),
                };
            movement = SimpleMove {
                index,
                destination: Coord::new(king_destination, self.origin.y()),
                capture: None,
            };
            castling_rook = Some(SimpleMove {
                index: rook,
                destination: Coord::new(rook_destination, self.origin.y()),
                capture: None,
            });
            castling_right = board.castling_right.to_cleared(piece.color());
        } else if piece.piece() == PieceKind::King
            && !(self.destination - self.origin).is_king_move()
        {
            // "King to king's destination" castling configuration e.g. e1g1
            // This doesn't apply when it's a legal king move
            let (king_rook_ord, rook_destination) = match self.destination.x() {
                Coord::CASTLING_KING_DESTINATION_QUEENSIDE => (
                    Ordering::Greater,
                    Coord::CASTLING_ROOK_DESTINATION_QUEENSIDE,
                ),
                Coord::CASTLING_KING_DESTINATION_KINGSIDE => {
                    (Ordering::Less, Coord::CASTLING_ROOK_DESTINATION_KINGSIDE)
                }
                _ => panic!(
                    "invalid king destination when castling: {}",
                    self.destination
                ),
            };
            let (rook, _) = board
                .pieces_by_kind_indexed(piece.color(), PieceKind::Rook)
                .find(|(_, rook)| {
                    rook.position.y() == self.origin.y()
                        && Ord::cmp(&self.origin.x(), &rook.position.x()) == king_rook_ord
                })
                .expect("rook not found");
            movement = SimpleMove {
                index,
                destination: self.destination,
                capture: None,
            };
            castling_rook = Some(SimpleMove {
                index: rook,
                destination: Coord::new(rook_destination, self.origin.y()),
                capture: None,
            });
            castling_right = board.castling_right.to_cleared(piece.color());
        } else {
            // Moves other than castling
            let capture = if board.en_passant_target == Some(self.destination) {
                let pawn = self
                    .destination
                    .move_by(Vector::pawn_single_move(!piece.color()))
                    .unwrap();
                Some(
                    board
                        .get_index_with_kind(pawn, !piece.color(), PieceKind::Pawn)
                        .expect("pawn that performed double move not found"),
                )
            } else {
                capture
            };
            movement = SimpleMove {
                index,
                destination: self.destination,
                capture,
            };
            castling_rook = None;
            if piece.piece() == PieceKind::King {
                castling_right = board.castling_right.to_cleared(piece.color());
            } else if piece.piece() == PieceKind::Rook
                && self.origin.y() == Coord::home_rank(piece.color())
            {
                castling_right = board
                    .castling_right
                    .to_removed(piece.color(), self.origin.x());
            } else {
                castling_right = board.castling_right;
            }
        }
        let castling_right = if let Some(index) = movement.capture {
            castling_right
                .to_removed_for_rook_capture(board[index].expect("captured piece not found"))
        } else {
            castling_right
        };
        let en_passant_target = if piece.piece() == PieceKind::Pawn
            && (movement.destination - self.origin) == Vector::pawn_double_move(piece.color())
        {
            let en_passant_target = self
                .origin
                .move_by(Vector::pawn_single_move(piece.color()))
                .unwrap();
            board
                .can_attack_by_pawn(en_passant_target, !piece.color())
                .then_some(en_passant_target)
        } else {
            None
        };
        Move {
            movement,
            castling_rook,
            promotion: self.promotion,
            en_passant_target,
            castling_right,
        }
    }
}
impl Display for Lan {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.origin, self.destination)?;
        if let Some(promotion) = self.promotion {
            write!(f, "{}", promotion.lowercase())?;
        }
        Ok(())
    }
}
impl FromStr for Lan {
    type Err = ParseLanError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let origin = s.get(0..2).ok_or(ParseLanError::InvalidChar)?.parse()?;
        let destination = s.get(2..4).ok_or(ParseLanError::InvalidChar)?.parse()?;
        let mut rest = s.get(4..).ok_or(ParseLanError::InvalidChar)?.chars().fuse();
        let promotion = rest.next().map(PieceKind::from_fen).transpose()?;

        if let Some(c) = rest.next() {
            return Err(ParseLanError::Unexpected(c));
        }
        Ok(Lan {
            origin,
            destination,
            promotion,
        })
    }
}
impl Moveable for Lan {
    fn as_move(&self, board: &Board) -> Move {
        Lan::as_move(*self, board)
    }
}
#[cfg(test)]
mod test {
    use crate::{board::Board, color::Color, coord, end_state::EndState, fen::Fen};

    #[test]
    fn checkmate() {
        let board: Fen = "4k3/8/r7/r3K3/r7/8/8/8 w - - 0 1".parse().unwrap();
        let board: Board = board.board.try_into().unwrap();
        assert!(matches!(
            board.valid_moves(),
            Err(EndState::Win(Color::Black))
        ));
    }
    #[test]
    fn stalemate() {
        let board: Fen = "3rkr2/8/r7/4K3/r7/8/8/8 w - - 0 1".parse().unwrap();
        let board: Board = board.board.try_into().unwrap();
        assert!(matches!(board.valid_moves(), Err(EndState::Draw)));
    }
    #[test]
    fn dead_position() {
        let board: Fen = "4k3/8/8/8/8/8/8/4K3 w - - 0 1".parse().unwrap();
        let board: Board = board.board.try_into().unwrap();
        assert!(matches!(board.valid_moves(), Err(EndState::Draw)));
    }
    #[test]
    fn dead_position_with_knight() {
        let board: Fen = "1n2k3/8/8/8/8/8/8/4K3 w - - 0 1".parse().unwrap();
        let board: Board = board.board.try_into().unwrap();
        assert!(matches!(board.valid_moves(), Err(EndState::Draw)));
    }
    #[test]
    fn dead_position_with_bishops() {
        let board: Fen = "4kb2/4b3/8/8/8/8/8/4K3 w - - 0 1".parse().unwrap();
        let board: Board = board.board.try_into().unwrap();
        assert!(matches!(board.valid_moves(), Err(EndState::Draw)));
    }
    #[test]
    fn bishop_of_different_color_is_alive() {
        let board: Fen = "2b1kb2/8/8/8/8/8/8/4K3 w - - 0 1".parse().unwrap();
        let board: Board = board.board.try_into().unwrap();
        assert!(board.valid_moves().is_ok());
    }
    #[test]
    fn castling() {
        let board: Fen = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1".parse().unwrap();
        let mut board: Board = board.board.try_into().unwrap();
        board.move_assert("e1g1".parse().unwrap());
        board.move_assert("e8a8".parse().unwrap());

        assert_eq!(
            board.as_hashable(),
            "2kr3r/8/8/8/8/8/8/R4RK1 w - - 0 1"
                .parse::<Fen>()
                .unwrap()
                .board
        );
    }
    #[test]
    fn cant_castle_when_blocked() {
        let board: Fen = "r3k2r/8/8/8/8/8/8/R3K1NR w KQkq - 0 1".parse().unwrap();
        let mut board: Board = board.board.try_into().unwrap();
        board.assert_move_is_invalid("e1g1".parse().unwrap());
        board.move_assert("e1c1".parse().unwrap());

        assert_eq!(
            board.as_hashable(),
            "r3k2r/8/8/8/8/8/8/2KR2NR b kq - 0 1"
                .parse::<Fen>()
                .unwrap()
                .board
        );
    }
    #[test]
    fn cant_castle_after_move() {
        let board: Fen = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1".parse().unwrap();
        let mut board: Board = board.board.try_into().unwrap();
        board.move_assert("a1a2".parse().unwrap());
        board.move_assert("e8e7".parse().unwrap());
        board.move_assert("a2a1".parse().unwrap());
        board.move_assert("e7e8".parse().unwrap());
        board.assert_move_is_invalid("e1c1".parse().unwrap());
        board.move_assert("e1g1".parse().unwrap());
        board.assert_move_is_invalid("e8c8".parse().unwrap());
        board.assert_move_is_invalid("e8g8".parse().unwrap());

        assert_eq!(
            board.as_hashable(),
            "r3k2r/8/8/8/8/8/8/R4RK1 b - - 0 1"
                .parse::<Fen>()
                .unwrap()
                .board
        );
    }
    #[test]
    fn cant_castle_after_rook_captured() {
        let board: Fen = "r3k2r/8/6N1/8/8/8/8/4K3 w kq - 0 1".parse().unwrap();
        let mut board: Board = board.board.try_into().unwrap();
        board.move_assert("g6h8".parse().unwrap());
        board.assert_move_is_invalid("e8g8".parse().unwrap());
        board.assert_move_is_valid("e8c8".parse().unwrap());

        assert_eq!(
            board.as_hashable(),
            "r3k2N/8/8/8/8/8/8/4K3 b q - 0 1"
                .parse::<Fen>()
                .unwrap()
                .board
        );
    }
    #[test]
    fn cant_castle_while_checked() {
        let board: Fen = "4k3/4r3/8/8/8/8/8/R3K2R w KQ - 0 1".parse().unwrap();
        let board: Board = board.board.try_into().unwrap();
        board.assert_move_is_invalid("e1g1".parse().unwrap());
    }
    #[test]
    fn cant_castle_through_checked() {
        let board: Fen = "4k3/5r2/8/8/8/8/8/R3K2R w KQ - 0 1".parse().unwrap();
        let board: Board = board.board.try_into().unwrap();
        board.assert_move_is_invalid("e1g1".parse().unwrap());
    }
    #[test]
    fn cant_castle_into_checked() {
        let board: Fen = "4k3/6r1/8/8/8/8/8/R3K2R w KQ - 0 1".parse().unwrap();
        let board: Board = board.board.try_into().unwrap();
        board.assert_move_is_invalid("e1g1".parse().unwrap());
    }
    #[test]
    fn chess960_castling_special_case() {
        let board: Fen = "4k3/8/8/8/8/8/8/rR2K2R w BH - 0 1".parse().unwrap();
        let board: Board = board.board.try_into().unwrap();
        board.assert_move_is_invalid("e1b1".parse().unwrap());
    }
    #[test]
    fn promotion() {
        let board: Fen = "4k3/6P1/8/8/8/8/8/4K3 w - - 0 1".parse().unwrap();
        let mut board: Board = board.board.try_into().unwrap();
        board.move_assert("g7g8q".parse().unwrap());
        assert_eq!(
            board.as_hashable(),
            "4k1Q1/8/8/8/8/8/8/4K3 b - - 0 1"
                .parse::<Fen>()
                .unwrap()
                .board
        );
    }
    #[test]
    fn en_passant() {
        let board: Fen = "4k3/8/8/8/5p2/8/4P3/4K3 w - - 0 1".parse().unwrap();
        let mut board: Board = board.board.try_into().unwrap();
        board.move_assert("e2e4".parse().unwrap());
        board.move_assert("f4e3".parse().unwrap());
        assert_eq!(
            board.as_hashable(),
            "4k3/8/8/8/8/4p3/8/4K3 w - - 0 1"
                .parse::<Fen>()
                .unwrap()
                .board
        );
    }
    #[test]
    fn lose_of_en_passant_right() {
        let board: Fen = "4k3/8/8/8/5p2/8/4P3/4K3 w - - 0 1".parse().unwrap();
        let mut board: Board = board.board.try_into().unwrap();
        board.move_assert("e2e4".parse().unwrap());
        board.move_assert("e8d8".parse().unwrap());
        board.move_assert("e1d1".parse().unwrap());
        board.assert_move_is_invalid("f4e3".parse().unwrap());
        assert_eq!(
            board.as_hashable(),
            "3k4/8/8/8/4Pp2/8/8/3K4 b - - 0 1"
                .parse::<Fen>()
                .unwrap()
                .board
        );
    }
    #[test]
    fn en_passant_pawn_is_pinned() {
        let board: Fen = "8/8/8/8/k4p1R/8/4P3/4K3 w - - 0 1".parse().unwrap();
        let mut board: Board = board.board.try_into().unwrap();
        board.move_assert("e2e4".parse().unwrap());
        board.assert_move_is_invalid("f4e3".parse().unwrap());
        assert_eq!(
            board.as_hashable(),
            "8/8/8/8/k3Pp1R/8/8/4K3 b - e3 0 1"
                .parse::<Fen>()
                .unwrap()
                .board
        );
    }
    #[test]
    fn en_passant_pawn_is_not_pinned() {
        let board: Fen = "4k3/8/8/8/5p2/8/4P3/4R1K1 w - - 0 1".parse().unwrap();
        let mut board: Board = board.board.try_into().unwrap();
        board.move_assert("e2e4".parse().unwrap());
        board.move_assert("f4e3".parse().unwrap());
        assert_eq!(
            board.as_hashable(),
            "4k3/8/8/8/8/4p3/8/4R1K1 w - - 0 1"
                .parse::<Fen>()
                .unwrap()
                .board
        );
    }
    #[test]
    fn cant_move_to_check() {
        let board: Fen = "4kr2/8/8/8/8/8/8/4K3 w - - 0 1".parse().unwrap();
        let board: Board = board.board.try_into().unwrap();
        board.assert_move_is_invalid("e1f1".parse().unwrap());
    }
    #[test]
    fn pin() {
        let board: Fen = "4k3/4r3/8/8/8/8/4N3/4K3 w - - 0 1".parse().unwrap();
        let board: Board = board.board.try_into().unwrap();
        board.assert_piece_cant_move(coord!("e2"));
    }
    #[test]
    fn can_capture_pinning_piece() {
        let board: Fen = "4k3/4r3/8/8/8/8/4R3/4K3 w - - 0 1".parse().unwrap();
        let mut board: Board = board.board.try_into().unwrap();
        board.move_assert("e2e7".parse().unwrap());
        assert_eq!(
            board.as_hashable(),
            "4k3/4R3/8/8/8/8/8/4K3 b - - 0 1"
                .parse::<Fen>()
                .unwrap()
                .board
        );
    }
    #[test]
    fn can_move_along_line_when_pinned() {
        let board: Fen = "4k3/4r3/8/8/8/8/4R3/4K3 w - - 0 1".parse().unwrap();
        let mut board: Board = board.board.try_into().unwrap();
        board.move_assert("e2e6".parse().unwrap());
        assert_eq!(
            board.as_hashable(),
            "4k3/4r3/4R3/8/8/8/8/4K3 b - - 0 1"
                .parse::<Fen>()
                .unwrap()
                .board
        );
    }
    #[test]
    fn check_must_be_responded() {
        let board: Fen = "4k3/4r3/8/8/8/8/P7/4K3 w - - 0 1".parse().unwrap();
        let board: Board = board.board.try_into().unwrap();
        board.assert_move_is_invalid("a2a4".parse().unwrap());
    }
    #[test]
    fn can_take_checking_piece() {
        let board: Fen = "4k3/4r3/5B2/8/8/8/8/4K3 w - - 0 1".parse().unwrap();
        let mut board: Board = board.board.try_into().unwrap();
        board.move_assert("f6e7".parse().unwrap());
        assert_eq!(
            board.as_hashable(),
            "4k3/4B3/8/8/8/8/8/4K3 b - - 0 1"
                .parse::<Fen>()
                .unwrap()
                .board
        );
    }
    #[test]
    fn can_block_check() {
        let board: Fen = "4k3/4r3/5B2/8/8/8/8/4K3 w - - 0 1".parse().unwrap();
        let mut board: Board = board.board.try_into().unwrap();
        board.move_assert("f6e5".parse().unwrap());
        assert_eq!(
            board.as_hashable(),
            "4k3/4r3/8/4B3/8/8/8/4K3 b - - 0 1"
                .parse::<Fen>()
                .unwrap()
                .board
        );
    }
    #[test]
    fn cant_take_on_double_check() {
        let board: Fen = "4k3/4r1R1/8/8/8/8/6n1/4K3 w - - 0 1".parse().unwrap();
        let board: Board = board.board.try_into().unwrap();
        board.assert_move_is_invalid("g7e7".parse().unwrap());
        board.assert_move_is_invalid("g7g2".parse().unwrap());
    }
}
