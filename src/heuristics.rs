use std::cmp::Ordering;

use crate::chess::{Color, EndState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Estimated {
    king_safety: i32,
    square_control: i32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Advantage {
    End(EndState),
    Estimated(i32),
}
impl PartialOrd for Advantage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Advantage {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Advantage::Estimated(a), Advantage::Estimated(b)) => a.cmp(b),
            (Advantage::End(EndState::Draw), Advantage::Estimated(advantage)) => 0.cmp(advantage),
            (Advantage::Estimated(advantage), Advantage::End(EndState::Draw)) => advantage.cmp(&0),

            (
                Advantage::End(EndState::Win(Color::Black)),
                Advantage::End(EndState::Win(Color::Black)),
            ) => Ordering::Equal,
            (Advantage::End(EndState::Win(Color::Black)), _) => Ordering::Less,
            (_, Advantage::End(EndState::Win(Color::Black))) => Ordering::Greater,

            (
                Advantage::End(EndState::Win(Color::White)),
                Advantage::End(EndState::Win(Color::White)),
            ) => Ordering::Equal,
            (Advantage::End(EndState::Win(Color::White)), _) => Ordering::Greater,
            (_, Advantage::End(EndState::Win(Color::White))) => Ordering::Less,

            (Advantage::End(EndState::Draw), Advantage::End(EndState::Draw)) => Ordering::Equal,
        }
    }
}
