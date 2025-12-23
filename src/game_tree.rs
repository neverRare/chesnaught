use std::{
    cmp::Ordering,
    collections::HashMap,
    iter::from_fn,
    mem::replace,
    sync::{
        LazyLock, RwLock,
        atomic::{self, AtomicBool},
        mpsc::{Sender, channel},
    },
    thread::{Builder, ScopedJoinHandle, panicking, scope},
};

use rustc_hash::FxHashMap;

use crate::{
    board::{Board, HashableBoard, Lan},
    color::Color,
    end_state::EndState,
    heuristics::Score,
    misc::cold_path,
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
        let data = if let Some(end_state) = board.end_state() {
            Data::End(end_state)
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
    fn search(&mut self, board: HashableBoard, setting: AlphaBetaSetting) -> u32 {
        let mut nodes = 1;
        let current_player = self.current_player().unwrap();
        let children = self.children_or_init().unwrap();
        let mut alpha_beta = AlphaBetaState::new(current_player, setting.alpha, setting.beta);

        let mut searched_children = 0;

        let mut write = setting.table.write().unwrap();
        write.insert_repetition(board);
        drop(write);
        if setting.multithread_depth == Some(0) {
            for chunk in children.chunks_mut(setting.thread_count) {
                searched_children += chunk.len();
                let stop = scope(|scope| {
                    let handles: Vec<_> = chunk
                        .iter_mut()
                        .map(|(_, _, game_tree)| {
                            scope.spawn(move || {
                                let nodes = game_tree.alpha_beta(AlphaBetaSetting {
                                    depth: setting.depth - 1,
                                    alpha: alpha_beta.alpha,
                                    beta: alpha_beta.beta,
                                    table: setting.table,
                                    multithread_depth: None,
                                    thread_count: setting.thread_count,
                                    stop_signal: setting.stop_signal,
                                });
                                (nodes, game_tree.score)
                            })
                        })
                        .collect();
                    while !handles.iter().all(ScopedJoinHandle::is_finished) {}
                    let mut stop = false;
                    for handle in handles {
                        let (b, score) = handle.join().unwrap();
                        nodes += b;
                        if !stop
                            && let Some(score) = score
                            && alpha_beta.set(score)
                        {
                            stop = true;
                        }
                    }
                    stop
                });
                if stop {
                    break;
                }
            }
        } else {
            for (_, _, game_tree) in &mut *children {
                nodes += game_tree.alpha_beta(AlphaBetaSetting {
                    depth: setting.depth - 1,
                    alpha: alpha_beta.alpha,
                    beta: alpha_beta.beta,
                    table: setting.table,
                    multithread_depth: setting.multithread_depth.map(|depth| depth - 1),
                    thread_count: setting.thread_count,
                    stop_signal: setting.stop_signal,
                });
                searched_children += 1;
                if let Some(score) = game_tree.score
                    && alpha_beta.set(score)
                {
                    break;
                }
            }
        }
        let mut write = setting.table.write().unwrap();
        write.remove_repetition(&board);
        drop(write);
        children[..searched_children].sort_unstable_by(|(_, _, a), (_, _, b)| {
            let ord = match (a.score, b.score) {
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Less,
                (Some(_), None) => Ordering::Greater,
                (Some(a), Some(b)) => match current_player {
                    Color::White => Ord::cmp(&a, &b),
                    Color::Black => Ord::cmp(&a, &b).reverse(),
                },
            };
            ord.reverse()
        });
        self.score = Some(alpha_beta.score);
        let mut write = setting.table.write().unwrap();
        write.insert_transposition(board, alpha_beta.score);
        drop(write);
        nodes
    }
    fn alpha_beta(&mut self, setting: AlphaBetaSetting) -> u32 {
        if setting
            .stop_signal
            .is_some_and(|signal| signal.load(atomic::Ordering::Relaxed))
        {
            // Do nothing
            1
        } else if let Data::End(end_state) = self.data {
            self.score = Some(Score::from_end_state(end_state));
            1
        } else {
            let board = self.board().unwrap();

            let read = setting.table.read().unwrap();

            if let Some(score) = read.get_transposition(&board) {
                self.score = Some(*score);
                return 1;
            }
            if read.contains_repetition(&board) {
                return 1;
            }
            drop(read);
            if setting.depth == 0 {
                self.score = Some(self.estimate());
                1
            } else {
                self.search(board, setting)
            }
        }
    }
    fn estimate(&self) -> Score {
        let estimated = if let Some(score) = self.score {
            return score;
        } else if let Data::Board(board) = &self.data {
            board.estimate()
        } else if cfg!(debug_assertions) {
            panic!("missing score");
        } else {
            // last resort

            cold_path();
            match &self.data {
                Data::Board(_) => unreachable!(),
                Data::Children { board, .. } => {
                    let board: Board = (**board).try_into().unwrap();
                    board.estimate()
                }
                Data::End(end_state) => return Score::from_end_state(*end_state),
            }
        };
        Score::Estimated(estimated)
    }
    fn best_move_tree_pair(&self) -> Option<&MoveTreePair> {
        self.children().map(|children| &children[0])
    }
}
#[derive(Debug, Clone, Copy)]
struct AlphaBetaSetting<'lock, 'table, 'bool> {
    depth: u32,
    alpha: Score,
    beta: Score,
    table: &'lock RwLock<&'table mut Table>,
    multithread_depth: Option<u32>,
    thread_count: usize,
    stop_signal: Option<&'bool AtomicBool>,
}
#[derive(Debug, Clone)]
pub struct GameTree(GameTreeInner);

impl GameTree {
    pub fn new(board: Board) -> Self {
        GameTree(GameTreeInner::new(board))
    }
    pub fn move_piece(&mut self, movement: Lan) {
        let new = match &mut self.0.data {
            Data::Board(_) => {
                let dummy = Data::End(EndState::Draw);
                let data = replace(&mut self.0.data, dummy);
                let Data::Board(board) = data else {
                    unreachable!()
                };
                let mut board = *board;
                board.move_lan(movement);
                GameTreeInner::new(board)
            }
            Data::Children { children, .. } => {
                let (_, _, children) = children.swap_remove(
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
    fn calculate_raw(
        &mut self,
        depth: u32,
        table: &mut Table,
        thread_count: usize,
        stop_signal: Option<&AtomicBool>,
    ) -> u32 {
        table.clear();
        let multithread_depth = if thread_count > 1 {
            Some(depth / 2)
        } else {
            None
        };
        self.0.alpha_beta(AlphaBetaSetting {
            depth,
            alpha: Score::BLACK_WINS,
            beta: Score::WHITE_WINS,
            table: &RwLock::new(table),
            multithread_depth,
            thread_count,
            stop_signal,
        })
    }
    pub fn calculate(&mut self, depth: u32, table: &mut Table, thread_count: usize) -> u32 {
        self.calculate_raw(depth, table, thread_count, None)
    }
    pub fn calculate_with_stop_signal(
        &mut self,
        depth: u32,
        table: &mut Table,
        stop_signal: &AtomicBool,
        thread_count: usize,
    ) -> u32 {
        self.calculate_raw(depth, table, thread_count, Some(stop_signal))
    }
    pub fn best_move(&self) -> Option<Lan> {
        self.0
            .best_move_tree_pair()
            .map(|(movement, _, _)| *movement)
    }
    pub fn score(&self) -> Option<Score> {
        self.0.score
    }
    pub fn best_line(&self) -> impl Iterator<Item = Lan> {
        let mut game_tree = &self.0;
        from_fn(move || {
            game_tree
                .best_move_tree_pair()
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
    max_capacity: usize,
}
impl Table {
    pub const ELEMENT_SIZE: usize = size_of::<(HashableBoard, TableValue)>();

    pub fn new(max_capacity: usize) -> Self {
        Table {
            table: HashMap::default(),
            max_capacity: Ord::min(max_capacity, <i32>::MAX as usize),
        }
    }
    pub fn capacity(&self) -> usize {
        self.table.capacity()
    }
    pub fn max_capacity(&self) -> usize {
        self.max_capacity
    }
    pub fn set_max_capacity(&mut self, max_capacity: usize) {
        self.max_capacity = Ord::min(max_capacity, <i32>::MAX as usize);
        if self.capacity() > self.max_capacity {
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
            let max_capacity = self.max_capacity.saturating_sub(self.capacity()) / 2;
            if self.table.len() < self.capacity() || self.capacity() <= max_capacity {
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
                self.score = Ord::max(self.score, score);
                if self.score >= self.beta {
                    return true;
                }
                self.alpha = Ord::max(self.alpha, self.score);
            }
            Color::Black => {
                self.score = Ord::min(self.score, score);
                if self.score <= self.alpha {
                    return true;
                }
                self.beta = Ord::min(self.beta, self.score);
            }
        }
        false
    }
}
