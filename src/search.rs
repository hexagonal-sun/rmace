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
}

const INF: i32 = i32::MAX - 2;

impl Search {
    pub fn go(mut self) -> Move {
        let should_exit = Arc::new(AtomicBool::new(false));

        let (tx, rx) = mpsc::channel();
        let deadline = self.deadline.unwrap_or(Duration::from_secs(5));

        {
            let should_exit = should_exit.clone();
            thread::spawn(move || {
                let mut depth = 1;
                loop {
                    let mut pv = Vec::with_capacity(depth);
                    self.nodes = 0;
                    let best_score = self.search(-INF, INF, depth as u32, &mut pv, &should_exit);

                    if should_exit.load(Ordering::Relaxed) {
                        return;
                    }

                    let best_move = pv.last().copied().unwrap();
                    println!(
                        "info depth {} pv {} score {} cp nodes {}",
                        depth,
                        pv.iter().rev().map(|x| UciMove::from(*x)).fold(
                            String::new(),
                            |mut accum, x| {
                                accum.push_str(&format!("{} ", x).to_owned());
                                accum
                            }
                        ),
                        best_score,
                        self.nodes,
                    );
                    let _ = tx.send(best_move);
                    depth += 1;
                }
            });
        }

        sleep(deadline);
        should_exit.store(true, Ordering::Relaxed);

        let mut best_move = rx.recv().unwrap();

        loop {
            if let Ok(m) = rx.try_recv() {
                best_move = m;
            } else {
                break;
            }
        }

        best_move
    }

    fn search(
        &mut self,
        mut alpha: i32,
        beta: i32,
        depth: u32,
        pv: &mut Vec<Move>,
        should_exit: &Arc<AtomicBool>,
    ) -> i32 {
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
            println!("CHECKMATE?!?");
            return -INF;
        }

        let mut local_pv = Vec::with_capacity(depth as usize);

        for m in mmoves {
            let token = self.pos.make_move(m);
            let eval = -self.search(-beta, -alpha, depth - 1, &mut local_pv, should_exit);
            self.pos.undo_move(token);

            // Timeout detection.
            if (self.nodes & 0xfff == 0xfff) && should_exit.load(Ordering::Relaxed) {
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
