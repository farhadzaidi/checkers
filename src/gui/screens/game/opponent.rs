use crate::game::{GameSession, Move, Side};
use crate::search::{self, SearchLimits};
use eframe::egui;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::Duration;

/// How long the engine thinks before each move
const THINK_TIME: Duration = Duration::from_secs(1);

/// The computer opponent: the side it plays, and the search it's running on a background thread
pub struct Opponent {
    side: Side,
    thinking: Option<Thinking>,
    // The engine panicked or picked an illegal move, so it stops playing rather than crash the app
    failed: bool,
}

/// A search running on a background thread, which sends back its move when it's done
struct Thinking {
    stop: Arc<AtomicBool>,
    receiver: Receiver<Option<Move>>,
}

impl Drop for Thinking {
    /// Tell the search to stop when its answer is no longer wanted. Its channel goes with it,
    /// so a late answer for a position that's gone can never be played
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

impl Opponent {
    pub fn new(side: Side) -> Self {
        Self {
            side,
            thinking: None,
            failed: false,
        }
    }

    pub fn side(&self) -> Side {
        self.side
    }

    pub fn has_failed(&self) -> bool {
        self.failed
    }

    /// Think about the session's position when it's our move, and return the move once it's ready
    pub fn update(&mut self, session: &GameSession, ctx: &egui::Context) -> Option<Move> {
        let our_move = session.current_position().side_to_move() == self.side;
        if !our_move || self.failed || session.outcome().is_some() {
            return None;
        }

        let Some(thinking) = &self.thinking else {
            self.thinking = Some(start_thinking(session, ctx.clone()));
            return None;
        };

        match thinking.receiver.try_recv() {
            Err(TryRecvError::Empty) => None,
            Ok(Some(mv)) if session.moves().contains(&mv) => {
                self.thinking = None;
                Some(mv)
            }
            // No move, an illegal one, or the search panicked before answering
            Ok(_) | Err(TryRecvError::Disconnected) => {
                eprintln!("the engine didn't return a legal move");
                self.thinking = None;
                self.failed = true;
                None
            }
        }
    }

    /// Stop thinking about the current position, because it's about to change
    pub fn cancel(&mut self) {
        self.thinking = None;
    }
}

/// Start a search for the session's position on a background thread, waking the UI when it's done
fn start_thinking(session: &GameSession, ctx: egui::Context) -> Thinking {
    let stop = Arc::new(AtomicBool::new(false));
    let (sender, receiver) = mpsc::channel();

    // The search gets the position and the game's history, its own copies to use on its thread
    let board = session.current_position();
    let history = session.history();
    let limits = SearchLimits::time(THINK_TIME);

    let thread_stop = Arc::clone(&stop);
    thread::spawn(move || {
        let mv = search::search(&board, &history, limits, &thread_stop);
        // Nobody may be listening anymore, if the search was cancelled
        let _ = sender.send(mv);
        ctx.request_repaint();
    });

    Thinking { stop, receiver }
}
