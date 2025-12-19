use std::{
    cmp::Ordering,
    collections::HashMap,
    iter::from_fn,
    mem::replace,
    sync::{
        LazyLock,
        atomic::{self, AtomicBool},
        mpsc::{Sender, channel},
    },
    thread::{Builder, panicking},
};

use rustc_hash::FxHashMap;

use crate::{
    board::{Board, HashableBoard, Lan},
    color::Color,
    end_state::EndState,
    heuristics::{Estimated, Score},
};

type MoveTreePair = (Lan, Option<Lan>, GameTreeInner);

#[derive(Debug, Clone)]
enum Data {
    Board(Box<Board>),
    Children {
        board: Box<HashableBoard>,
        children: Vec<MoveTreePair>,
    },
    End(EndState),
}

#[derive(Debug, Clone)]
struct GameTreeInner {
    data: Data,
    score: Option<Score>,
}
impl GameTreeInner {
    fn new(board: Board) -> Self {
        let data = if let Err(state) = board.valid_moves() {
            Data::End(state)
        } else {
            Data::Board(Box::new(board))
        };
        GameTreeInner { data, score: None }
    }
    fn drop(self) {
        static DROPPER: LazyLock<Option<Sender<GameTreeInner>>> = LazyLock::new(|| {
            let (sender, receiver) = channel();

            let result = Builder::new().spawn(|| {
                for game_tree in receiver {
                    drop(game_tree);
                }
            });
            result.ok().map(|_| sender)
        });
        match &*DROPPER {
            Some(sender) => sender.send(self).unwrap(),
            None => drop(self),
        }
    }
    fn board(&self) -> Option<HashableBoard> {
        match &self.data {
            Data::Board(board) => Some(board.as_hashable()),
            Data::Children { board, .. } => Some(**board),
            Data::End(_) => None,
        }
    }
    fn children(&self) -> Option<&Vec<MoveTreePair>> {
        if let Data::Children { children, .. } = &self.data {
            Some(children)
        } else {
            None
        }
    }
    fn children_or_init(&mut self) -> Option<&mut Vec<MoveTreePair>> {
        match &mut self.data {
            Data::Board(board) => {
                let board = Board::clone(board);
                let hashable = board.as_hashable();
                self.data = Data::Children {
                    board: Box::new(hashable),
                    children: board
                        .valid_moves()
                        .unwrap()
                        .map(|movement| {
                            let (first, second) = movement.as_lan_pair(&board);
                            (
                                first,
                                second,
                                GameTreeInner::new(board.clone_and_move(movement)),
                            )
                        })
                        .collect(),
                };
            }
            Data::Children { .. } => (),
            Data::End(_) => return None,
        }
        let Data::Children { children, .. } = &mut self.data else {
            unreachable!()
        };
        Some(children)
    }
    fn current_player(&self) -> Option<Color> {
        match &self.data {
            Data::Board(board) => Some(board.current_player()),
            Data::Children { board, .. } => Some(board.current_player),
            Data::End(_) => None,
        }
    }
    fn alpha_beta(
        &mut self,
        depth: u32,
        alpha: Score,
        beta: Score,
        table: &mut Table,
        stop_signal: Option<&AtomicBool>,
    ) {
        if stop_signal.is_some_and(|signal| signal.load(atomic::Ordering::Relaxed)) {
            // Do nothing
        } else if let Data::End(state) = self.data {
            let score = match state {
                EndState::Win(color) => Score::Win(color),
                EndState::Draw => Score::Estimated(Estimated::default()),
            };
            self.score = Some(score);
        } else {
            let board = self.board().unwrap();

            if let Some(score) = table.get_transposition(&board) {
                self.score = Some(*score);
                return;
            }
            if table.contains_repetition(&board) {
                return;
            }
            if depth == 0 {
                self.score = Some(self.estimate());
            } else {
                let current_player = self.current_player().unwrap();
                let children = self.children_or_init().unwrap();
                let mut alpha_beta = AlphaBetaState::new(current_player, alpha, beta);

                table.insert_repetition(board);
                for (_, _, game_tree) in children.iter_mut() {
                    game_tree.alpha_beta(depth - 1, alpha, beta, table, stop_signal);
                    if let Some(score) = game_tree.score
                        && alpha_beta.set(score)
                    {
                        break;
                    }
                }
                table.remove_repetition(&board);
                children.sort_unstable_by(|(_, _, a), (_, _, b)| match (a.score, b.score) {
                    (None, None) => Ordering::Equal,
                    (None, Some(_)) => Ordering::Less,
                    (Some(_), None) => Ordering::Greater,
                    (Some(a), Some(b)) => match current_player {
                        Color::White => Ord::cmp(&b, &a),
                        Color::Black => Ord::cmp(&a, &b),
                    },
                });
                self.score = Some(alpha_beta.score);
            }
            table.insert_transposition(board, self.score.unwrap());
        }
    }
    fn estimate(&self) -> Score {
        let estimated = if let Data::Board(board) = &self.data {
            board.estimate()
        } else if let Some(score) = self.score {
            return score;
        } else if cfg!(debug_assertions) {
            panic!(concat!(
                "this node only contains board data meant for hashing alone. ",
                "the original board data is discarded to save memory space. ",
                "while it is possible to convert it back, we shouldn't resort ",
                "to this"
            ));
        } else {
            let board: Board = self
                .board()
                .expect("can't estimate score on board with ended state")
                .try_into()
                .unwrap();
            board.estimate()
        };
        Score::Estimated(estimated)
    }
}
#[derive(Debug, Clone)]
pub struct GameTree(GameTreeInner);

impl GameTree {
    pub fn new(board: Board) -> Self {
        GameTree(GameTreeInner::new(board))
    }
    pub fn move_piece(&mut self, movement: Lan) {
        let new = match &mut self.0.data {
            Data::Board(board) => GameTreeInner::new(board.clone_and_move_lan(movement)),
            Data::Children { children, .. } => {
                let (_, _, children) = children.remove(
                    children
                        .iter()
                        .position(|(b, c, _)| movement == *b || Some(movement) == *c)
                        .unwrap(),
                );
                children
            }
            Data::End(_) => panic!("cannot move on end state"),
        };
        replace(&mut self.0, new).drop();
    }
    pub fn calculate(&mut self, depth: u32, table: &mut Table) {
        table.clear();
        self.0
            .alpha_beta(depth, Score::BLACK_WINS, Score::WHITE_WINS, table, None);
    }
    pub fn calculate_with_stop_signal(
        &mut self,
        depth: u32,
        table: &mut Table,
        stop_signal: &AtomicBool,
    ) {
        table.clear();
        self.0.alpha_beta(
            depth,
            Score::BLACK_WINS,
            Score::WHITE_WINS,
            table,
            Some(stop_signal),
        );
    }
    fn best_move_tree_pair(&self) -> Option<&MoveTreePair> {
        self.0.children().and_then(|children| children.first())
    }
    pub fn best_move(&self) -> Option<Lan> {
        self.best_move_tree_pair().map(|(movement, _, _)| *movement)
    }
    pub fn score(&self) -> Option<Score> {
        self.0.score
    }
    pub fn best_line(&self) -> impl Iterator<Item = Lan> {
        let mut game_tree = &self.0;
        from_fn(move || {
            self.best_move_tree_pair()
                .map(|(movement, _, new_game_tree)| {
                    game_tree = new_game_tree;
                    *movement
                })
        })
    }
}
impl Drop for GameTree {
    fn drop(&mut self) {
        if !panicking() {
            let dummy = GameTreeInner {
                data: Data::End(EndState::Draw),
                score: None,
            };
            replace(&mut self.0, dummy).drop();
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct TableValue {
    transposition: Option<Score>,
    repetition: bool,
}
#[derive(Debug, Clone, Default)]
pub struct Table {
    table: FxHashMap<HashableBoard, TableValue>,
    capacity: usize,
}
impl Table {
    pub const ELEMENT_SIZE: usize = size_of::<(HashableBoard, TableValue)>();

    pub fn new(capacity: usize) -> Self {
        Table {
            table: HashMap::default(),
            capacity: Ord::min(capacity, <i32>::MAX as usize),
        }
    }
    pub fn set_capacity(&mut self, capacity: usize) {
        self.capacity = Ord::min(capacity, <i32>::MAX as usize);
        if self.table.capacity() > self.capacity {
            self.clear_allocation();
        }
    }
    fn get_transposition(&self, board: &HashableBoard) -> Option<&Score> {
        self.table
            .get(board)
            .and_then(|value| value.transposition.as_ref())
    }
    fn contains_repetition(&self, board: &HashableBoard) -> bool {
        self.table.get(board).is_some_and(|value| value.repetition)
    }
    fn inspect_element(&mut self, board: HashableBoard, f: impl FnOnce(&mut TableValue)) {
        if let Some(value) = self.table.get_mut(&board) {
            f(value);
        } else {
            let max_capacity = self.capacity.saturating_sub(self.table.capacity()) / 2;
            if self.table.len() < self.table.capacity() || self.table.capacity() <= max_capacity {
                let mut value = TableValue::default();
                f(&mut value);
                self.table.insert(board, value);
            }
        }
    }
    fn insert_transposition(&mut self, board: HashableBoard, score: Score) {
        self.inspect_element(board, |value| value.transposition = Some(score));
    }
    fn insert_repetition(&mut self, board: HashableBoard) {
        self.inspect_element(board, |value| value.repetition = true);
    }
    fn remove_repetition(&mut self, board: &HashableBoard) {
        if let Some(value) = self.table.get_mut(board) {
            value.repetition = false;
        }
    }
    fn clear(&mut self) {
        self.table.clear();
    }
    pub fn clear_allocation(&mut self) {
        self.table = HashMap::default();
    }
}
struct AlphaBetaState {
    current_player: Color,
    alpha: Score,
    beta: Score,
    score: Score,
}
impl AlphaBetaState {
    fn new(current_player: Color, alpha: Score, beta: Score) -> Self {
        AlphaBetaState {
            current_player,
            alpha,
            beta,
            score: match current_player {
                Color::White => Score::BLACK_WINS,
                Color::Black => Score::WHITE_WINS,
            },
        }
    }
    fn set(&mut self, score: Score) -> bool {
        match self.current_player {
            Color::White => {
                if score > self.score {
                    self.score = score;
                }
                if self.score >= self.beta {
                    return true;
                }
                self.alpha = Ord::max(self.alpha, self.score);
            }
            Color::Black => {
                if score < self.score {
                    self.score = score;
                }
                if self.score <= self.alpha {
                    return true;
                }
                self.beta = Ord::min(self.beta, self.score);
            }
        }
        false
    }
}
