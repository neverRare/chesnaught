use std::{
    num::NonZero,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, Sender, channel, sync_channel},
    },
    thread::{sleep, spawn},
    time::Duration,
};

use crate::{
    board::{Board, Lan},
    game_tree::{GameTree, Table},
};

enum Input {
    Ready,
    SetBoard(Board),
    Move(Lan),
    Calculate {
        depth: Option<NonZero<u32>>,
        callback: Box<dyn FnOnce(Option<Lan>) + Send>,
        stop_signal: Arc<AtomicBool>,
    },
    SetHashSize(usize),
    ClearHash,
}
pub struct Engine {
    stop_signal: Option<Arc<AtomicBool>>,
    input: Sender<Input>,
    ready: Receiver<()>,
}
impl Engine {
    pub fn new() -> Self {
        let (input, input_receiver) = channel();
        let (ready_sender, ready) = sync_channel(0);
        spawn(move || {
            let mut game_tree = GameTree::new(Board::starting_position());
            let mut table = Table::new(0);
            for input in input_receiver {
                match input {
                    Input::Ready => {
                        if ready_sender.send(()).is_err() {
                            // Engine has been dropped, we don't need to process more inputs
                            return;
                        }
                    }
                    Input::SetBoard(board) => game_tree = GameTree::new(board),
                    Input::Move(movement) => game_tree.move_piece(movement),
                    Input::Calculate {
                        depth,
                        callback,
                        stop_signal,
                    } => {
                        for i in 1.. {
                            game_tree.calculate_with_stop_signal(i, &mut table, &stop_signal);
                            if stop_signal.load(Ordering::Relaxed)
                                || depth.is_some_and(|depth| i >= depth.get())
                            {
                                break;
                            }
                        }
                        callback(game_tree.best_move().or_else(|| {
                            game_tree.calculate(1, &mut table);
                            game_tree.best_move()
                        }));
                    }
                    Input::SetHashSize(size) => table.set_size(size),
                    Input::ClearHash => table.clear_allocation(),
                }
            }
        });
        Engine {
            stop_signal: None,
            input,
            ready,
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
        callback: impl FnOnce(Option<Lan>) + Send + 'static,
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
                callback: Box::new(callback),
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
    pub fn set_hash_size(&self, size: usize) {
        self.input.send(Input::SetHashSize(size)).unwrap();
    }
    pub fn clear_hash(&self) {
        self.input.send(Input::ClearHash).unwrap();
    }
}
