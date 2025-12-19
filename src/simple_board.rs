use std::ops::{Index, IndexMut};

use crate::{board_display::IndexableBoard, coord::Coord, piece::ColoredPieceKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct SimpleBoard<T>(pub [[T; 8]; 8]);

impl<T> Index<Coord> for SimpleBoard<T> {
    type Output = T;

    fn index(&self, index: Coord) -> &Self::Output {
        &self.0[index.y() as usize][index.x() as usize]
    }
}
impl<T> IndexMut<Coord> for SimpleBoard<T> {
    fn index_mut(&mut self, index: Coord) -> &mut Self::Output {
        &mut self.0[index.y() as usize][index.x() as usize]
    }
}
impl IndexableBoard for SimpleBoard<ColoredPieceKind> {
    fn index(&self, position: Coord) -> Option<ColoredPieceKind> {
        Some(self[position])
    }
}
impl IndexableBoard for SimpleBoard<Option<ColoredPieceKind>> {
    fn index(&self, position: Coord) -> Option<ColoredPieceKind> {
        self[position]
    }
}
