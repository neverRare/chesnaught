use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, Sender, channel},
    },
    thread::{sleep, spawn},
    time::Duration,
};

use crate::{
    board::{Board, Lan},
    game_tree::GameTree,
};

enum Input {
    Ready,
    SetBoard(Board),
    Move(Lan),
    Calculate {
        depth: Option<u32>,
        callback: Box<dyn FnOnce(Lan) + Send>,
        stop_signal: Arc<AtomicBool>,
    },
}
pub struct Engine {
    stop_signal: Option<Arc<AtomicBool>>,
    input: Sender<Input>,
    ready: Receiver<()>,
}
impl Engine {
    pub fn new() -> Self {
        let (input, input_receiver) = channel();
        let (ready_sender, ready) = channel();
        spawn(move || {
            let mut game_tree = GameTree::new(Board::starting_position());
            for input in input_receiver {
                match input {
                    Input::Ready => ready_sender.send(()).unwrap(),
                    Input::SetBoard(board) => game_tree = GameTree::new(board),
                    Input::Move(movement) => game_tree.move_piece(movement),
                    Input::Calculate {
                        depth,
                        callback,
                        stop_signal,
                    } => {
                        for i in 1.. {
                            game_tree.calculate_with_stop_signal(i, &stop_signal);
                            if depth.is_some_and(|depth| i >= depth) {
                                break;
                            }
                        }
                        if let Some(movement) = game_tree.best_move() {
                            callback(movement);
                        } else {
                            game_tree.calculate(1);
                            callback(game_tree.best_move().unwrap());
                        }
                    }
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
        depth: Option<u32>,
        callback: impl Fn(Lan) + Send + 'static,
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
}
