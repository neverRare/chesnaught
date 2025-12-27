use std::{
    num::NonZero,
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, Sender, channel, sync_channel},
    },
    thread::{sleep, spawn},
    time::{Duration, Instant},
};

use crate::{
    board::{Board, Lan},
    game_tree::{GameTree, Table},
    heuristics::Score,
};

enum Input {
    Ready,
    SetBoard(Board),
    Move(Lan),
    Calculate {
        depth: Option<NonZero<u32>>,
        nodes: Option<NonZero<u32>>,
        mate_in_plies: Option<NonZero<u32>>,
        info_callback: Box<dyn FnMut(Info) + Send>,
        best_move_callback: Box<dyn FnOnce(Option<Lan>, Option<Lan>) + Send>,
        stop_signal: Arc<AtomicBool>,
    },
    SetHashMaxCapacity(usize),
    ClearHash,
    SetThread(NonZero<usize>),
}
pub struct Info {
    pub depth: NonZero<u32>,
    pub time: Duration,
    pub nodes: NonZero<u32>,
    pub pv: Box<[Lan]>,
    pub score: Option<Score>,
    pub hash_capacity: usize,
}
#[derive(Debug)]
pub struct Engine {
    stop_signal: Option<Arc<AtomicBool>>,
    input: Sender<Input>,
    ready: Receiver<()>,
    ponder: Arc<RwLock<Option<Lan>>>,
}
impl Engine {
    pub fn new() -> Self {
        let (input, input_receiver) = channel();
        let (ready_sender, ready) = sync_channel(0);
        let ponder = Arc::new(RwLock::new(None));
        let ponder_ = Arc::clone(&ponder);
        spawn(move || {
            let mut game_tree = GameTree::new(Board::starting_position());
            let mut table = Table::new(0);
            let mut thread = 1;
            let mut last_depth = 1;
            for input in input_receiver {
                match input {
                    Input::Ready => {
                        if ready_sender.send(()).is_err() {
                            // Engine has been dropped, we don't need to process more inputs
                            return;
                        }
                    }
                    Input::SetBoard(board) => {
                        last_depth = 1;
                        game_tree = GameTree::new(board);
                    }
                    Input::Move(movement) => {
                        last_depth = Ord::max(last_depth - 1, 1);
                        game_tree.move_piece(movement);
                    }
                    Input::Calculate {
                        depth,
                        nodes: max_nodes,
                        mate_in_plies,
                        mut info_callback,
                        best_move_callback,
                        stop_signal,
                    } => {
                        let start = if let Some(movement) = game_tree.best_move() {
                            info_callback(Info {
                                depth: NonZero::new(1).unwrap(),
                                time: Duration::ZERO,
                                nodes: NonZero::new(2).unwrap(),
                                pv: [movement].into(),
                                score: game_tree.score(),
                                hash_capacity: table.capacity(),
                            });
                            match depth {
                                Some(depth) => Ord::min(depth.get(), last_depth),
                                None => last_depth,
                            }
                        } else {
                            1
                        };
                        for i in start.. {
                            last_depth = i;
                            let start = Instant::now();
                            let nodes = game_tree.calculate_with_stop_signal(
                                i,
                                &mut table,
                                &stop_signal,
                                thread,
                            );
                            info_callback(Info {
                                depth: NonZero::new(i).unwrap(),
                                time: start.elapsed(),
                                nodes: NonZero::new(nodes).unwrap(),
                                pv: game_tree.best_line().collect(),
                                score: game_tree.score(),
                                hash_capacity: table.capacity(),
                            });
                            if stop_signal.load(Ordering::Relaxed)
                                || depth.is_some_and(|depth| i >= depth.get())
                                || max_nodes.is_some_and(|max_nodes| nodes >= max_nodes.get())
                                || (game_tree.score().is_some_and(Score::is_win)
                                    && mate_in_plies.is_some_and(|plies| i <= plies.get()))
                            {
                                break;
                            }
                        }
                        let mut best_line = game_tree.best_line().fuse();
                        let (movement, pondered_move) = if let Some(movement) = best_line.next() {
                            (Some(movement), best_line.next())
                        } else {
                            drop(best_line);
                            game_tree.calculate(1, &mut table, 1);
                            let mut best_line = game_tree.best_line().fuse();
                            (best_line.next(), best_line.next())
                        };
                        if let Some(movement) = pondered_move {
                            let mut write = ponder.write().unwrap();
                            *write = Some(movement);
                            drop(write);
                        }
                        best_move_callback(movement, pondered_move);
                    }
                    Input::SetHashMaxCapacity(capacity) => table.set_max_capacity(capacity),
                    Input::ClearHash => table.clear_allocation(),
                    Input::SetThread(new_value) => thread = new_value.get(),
                }
            }
        });
        Engine {
            stop_signal: None,
            input,
            ready,
            ponder: ponder_,
        }
    }
    pub fn ready(&self) {
        self.input.send(Input::Ready).unwrap();
        self.ready.recv().unwrap();
    }
    pub fn set_board(&self, board: Board) {
        self.input.send(Input::SetBoard(board)).unwrap();
    }
    pub fn move_piece(&self, movement: Lan) {
        self.input.send(Input::Move(movement)).unwrap();
    }
    pub fn calculate(
        &mut self,
        duration: Option<Duration>,
        depth: Option<NonZero<u32>>,
        nodes: Option<NonZero<u32>>,
        mate_in_plies: Option<NonZero<u32>>,
        info_callback: impl FnMut(Info) + Send + 'static,
        best_move_callback: impl FnOnce(Option<Lan>, Option<Lan>) + Send + 'static,
    ) {
        let stop_signal = Arc::new(AtomicBool::new(false));
        if let Some(duration) = duration {
            let stop_signal = stop_signal.clone();
            spawn(move || {
                sleep(duration);
                stop_signal.store(true, Ordering::Relaxed);
            });
        }
        self.input
            .send(Input::Calculate {
                depth,
                nodes,
                mate_in_plies,
                info_callback: Box::new(info_callback),
                best_move_callback: Box::new(best_move_callback),
                stop_signal: stop_signal.clone(),
            })
            .unwrap();
        self.stop_signal = Some(stop_signal);
    }
    pub fn stop(&self) {
        if let Some(stop_signal) = &self.stop_signal {
            stop_signal.store(true, Ordering::Relaxed);
        }
    }
    pub fn ponder(&self) -> Option<Lan> {
        let read = self.ponder.read().unwrap();
        *read
    }
    pub fn set_hash_max_capacity(&self, max_capacity: usize) {
        self.input
            .send(Input::SetHashMaxCapacity(max_capacity))
            .unwrap();
    }
    pub fn clear_hash(&self) {
        self.input.send(Input::ClearHash).unwrap();
    }
    pub fn set_thread(&self, thread: NonZero<usize>) {
        self.input.send(Input::SetThread(thread)).unwrap();
    }
}
