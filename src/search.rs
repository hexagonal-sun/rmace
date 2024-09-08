use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread::{self, sleep},
    time::Duration,
};

use crate::{
    mmove::Move,
    parsers::uci_move::UciMove,
    piece::Colour,
    position::{eval::Evaluator, Position},
};

#[derive(Clone)]
pub struct Search {
    pos: Position,
    deadline: Option<Duration>,
    nodes: u32,
    should_exit: Arc<AtomicBool>,
}

const INF: i32 = i32::MAX - 2;

impl Search {
    pub fn go(mut self) -> Move {
        let deadline = self.deadline.unwrap_or(Duration::from_secs(5));
        let should_exit = self.should_exit.clone();
        let mut best_move = None;

        thread::spawn(move || {
            sleep(deadline);
            should_exit.store(true, Ordering::Relaxed);
        });

        let mut depth = 1;
        loop {
            let mut pv = Vec::with_capacity(depth);
            self.nodes = 0;
            let best_score = self.search(-INF, INF, depth as u32, &mut pv);

            // Take the last move from the previous iteration, since when the
            // exit flag is true, we didn't complete the search.
            if self.should_exit.load(Ordering::Relaxed) {
                return best_move.unwrap();
            }

            best_move = pv.last().copied();

            println!(
                "info depth {} pv {} score {} cp nodes {}",
                depth,
                pv.iter()
                    .rev()
                    .map(|x| UciMove::from(*x))
                    .fold(String::new(), |mut accum, x| {
                        accum.push_str(&format!("{} ", x).to_owned());
                        accum
                    }),
                best_score,
                self.nodes,
            );
            depth += 1;
        }
    }

    fn search(&mut self, mut alpha: i32, beta: i32, depth: u32, pv: &mut Vec<Move>) -> i32 {
        if depth == 0 {
            let eval = Evaluator::eval(&self.pos);
            return if self.pos.to_play() == Colour::White {
                eval
            } else {
                -eval
            };
        }

        let mmoves = self.pos.movegen();

        // Checkmate detection.
        if mmoves.len() == 0 {
            return -INF;
        }

        let mut local_pv = Vec::with_capacity(depth as usize);

        for m in mmoves {
            let token = self.pos.make_move(m);
            let eval = -self.search(-beta, -alpha, depth - 1, &mut local_pv);
            self.pos.undo_move(token);

            // Timeout detection.
            if (self.nodes & 0xfff == 0xfff) && self.should_exit.load(Ordering::Relaxed) {
                return 0;
            }

            self.nodes += 1;

            if eval >= beta {
                return beta;
            }

            if eval > alpha {
                alpha = eval;
                pv.clear();
                pv.extend_from_slice(&local_pv);
                pv.push(m)
            }
        }

        alpha
    }
}

pub struct SearchBuilder {
    srch: Search,
}

impl SearchBuilder {
    pub fn new(pos: Position) -> Self {
        Self {
            srch: Search {
                pos,
                deadline: None,
                nodes: 0,
                should_exit: Arc::new(AtomicBool::new(false)),
            },
        }
    }

    pub fn with_deadline(mut self, deadline: Duration) -> Self {
        self.srch.deadline = Some(deadline);
        self
    }

    pub fn build(self) -> Search {
        self.srch
    }
}
