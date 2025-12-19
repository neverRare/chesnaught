use std::ops::{Index, IndexMut};

use crate::{board_display::IndexableBoard, coord::Coord, piece::ColoredPieceKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct SimpleBoard<T>(pub [[T; 8]; 8]);

impl<T> SimpleBoard<T> {
    pub fn copy_row(self, y: u8) -> [T; 8]
    where
        T: Copy,
    {
        self.0[y as usize]
    }
    pub fn row(&self, y: u8) -> &[T; 8] {
        &self.0[y as usize]
    }
    pub fn row_mut(&mut self, y: u8) -> &mut [T; 8] {
        &mut self.0[y as usize]
    }
    pub fn into_rows(self) -> impl Iterator<Item = [T; 8]> {
        self.0.into_iter()
    }
    pub fn into_positioned_values(self) -> impl Iterator<Item = (Coord, T)> {
        (0..).zip(self.into_rows()).flat_map(|(y, row)| {
            (0..)
                .zip(row)
                .map(move |(x, item)| (Coord::new(x, y), item))
        })
    }
}
impl<T> Index<Coord> for SimpleBoard<T> {
    type Output = T;

    fn index(&self, index: Coord) -> &Self::Output {
        &self.row(index.y())[index.x() as usize]
    }
}
impl<T> IndexMut<Coord> for SimpleBoard<T> {
    fn index_mut(&mut self, index: Coord) -> &mut Self::Output {
        &mut self.row_mut(index.y())[index.x() as usize]
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
