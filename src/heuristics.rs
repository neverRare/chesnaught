use std::cmp::Ordering;

use crate::chess::{Board, Color, EndState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Estimated {
    king_safety: i32,
    square_control: i32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Advantage {
    End(EndState),
    Estimated(Estimated),
}
impl Default for Advantage {
    fn default() -> Self {
        Advantage::Estimated(Estimated::default())
    }
}
impl PartialOrd for Advantage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Advantage {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Advantage::Estimated(a), Advantage::Estimated(b)) => Ord::cmp(a, b),
            (Advantage::End(EndState::Draw), Advantage::Estimated(advantage)) => {
                Ord::cmp(&Estimated::default(), advantage)
            }
            (Advantage::Estimated(advantage), Advantage::End(EndState::Draw)) => {
                Ord::cmp(advantage, &Estimated::default())
            }

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
pub fn estimate(board: Board) -> Estimated {
    let [white, black] = [Color::White, Color::Black].map(|color| {
        let board = board.into_switched_color(color);
        let opposing_king = board.king_of(!color).unwrap();
        let mut king_safety = 0;
        let mut square_control = 0;
        for movement in board.moves() {
            if board.is_move_valid(movement) {
                let moved_board = board.into_moved(movement);
                if moved_board.is_attacked_by(opposing_king.position, color) {
                    king_safety += 1;
                }
            }
            square_control += 1;
        }
        Estimated {
            king_safety,
            square_control,
        }
    });
    Estimated {
        king_safety: white.king_safety - black.king_safety,
        square_control: white.square_control - black.square_control,
    }
}
