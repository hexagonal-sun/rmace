use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, sleep},
    time::Duration,
};

use arrayvec::ArrayVec;

use crate::{
    mmove::Move,
    parsers::uci_move::UciMove,
    piece::Colour,
    position::{
        eval::Evaluator,
        movegen::{MoveGen, MoveList},
        Position,
    },
};

const MAX_PLY: usize = 100;

type PvStack = ArrayVec<Move, MAX_PLY>;

#[derive(Clone)]
pub struct Search {
    pos: Position,
    deadline: Option<Duration>,
    nodes: u32,
    should_exit: Arc<AtomicBool>,
    last_pv: PvStack,
}

const INF: i32 = i32::MAX - 2;
const MATE: i32 = INF - 1;

impl Search {
    pub fn order_moves(&self, ply: u32, moves: &mut MoveList) {
        // order captures first.
        moves.sort_by(|x, y| y.mvv_lva().cmp(&x.mvv_lva()));

        // Always investigate the corresponding node from the previous PV first
        if let Some(mmove) = self.last_pv.get(ply as usize) {
            if let Some(idx) = moves.iter().position(|x| *x == *mmove) {
                moves.swap(idx, 0);
            }
        }
    }

    pub fn go(mut self) -> Move {
        let deadline = self.deadline.unwrap_or(Duration::from_secs(5));
        let should_exit = self.should_exit.clone();
        let mut best_move = None;

        thread::spawn(move || {
            sleep(deadline);
            should_exit.store(true, Ordering::Relaxed);
        });

        let mut depth = 1;
        let mut pv = PvStack::new();

        loop {
            self.nodes = 0;
            let best_score = self.search(-INF, INF, 0, depth as u32, &mut pv);

            // Take the last move from the previous iteration, since when the
            // exit flag is true, we didn't complete the search.
            if self.should_exit.load(Ordering::Relaxed) {
                return best_move.unwrap();
            }

            self.last_pv = pv.clone();

            best_move = self.last_pv.first().copied();

            println!(
                "info depth {} pv{} score cp {} nodes {}",
                depth,
                self.last_pv.iter().map(|x| UciMove::from(*x)).fold(
                    String::new(),
                    |mut accum, x| {
                        accum.push_str(&format!(" {}", x).to_owned());
                        accum
                    }
                ),
                if best_score == MATE {
                    format!("mate {}", self.last_pv.len().div_ceil(2))
                } else if best_score == -MATE {
                    format!("mate -{}", self.last_pv.len().div_ceil(2))
                } else {
                    format!("{}", best_score)
                },
                self.nodes,
            );

            if best_score == MATE || best_score == -MATE {
                return best_move.unwrap();
            }

            depth += 1;
        }
    }

    fn quiescence(&mut self, mut alpha: i32, beta: i32) -> i32 {
        let eval = Evaluator::eval(&self.pos);
        let stand_pat = if self.pos.to_play() == Colour::White {
            eval
        } else {
            -eval
        };

        if stand_pat > beta {
            return beta;
        }

        if alpha < stand_pat {
            alpha = stand_pat;
        }

        if (self.nodes & 0xfff == 0xfff) && self.should_exit.load(Ordering::Relaxed) {
            return 0;
        }

        self.nodes += 1;

        let mut cap_moves = MoveGen::new(&mut self.pos).gen();
        cap_moves.retain(|x| x.capture.is_some());

        for cap_move in cap_moves {
            let token = self.pos.make_move(cap_move);
            if MoveGen::new(&self.pos).in_check(self.pos.to_play().next()) {
                self.pos.undo_move(token);
                continue;
            }
            let score = -self.quiescence(-beta, -alpha);
            self.pos.undo_move(token);
            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
            }
        }

        alpha
    }

    fn search(&mut self, mut alpha: i32, beta: i32, ply: u32, depth: u32, pv: &mut PvStack) -> i32 {
        if self.pos.has_repeated() {
            return 0;
        }

        if depth == 0 {
            return self.quiescence(alpha, beta);
        }

        let mut mmoves = MoveGen::new(&mut self.pos).gen();
        self.order_moves(ply, &mut mmoves);

        let mut local_pv = PvStack::new();

        let mut legal_moves = 0;
        let mut eval = -INF;

        for m in mmoves {
            let token = self.pos.make_move(m);
            if MoveGen::new(&self.pos).in_check(self.pos.to_play().next()) {
                self.pos.undo_move(token);
                continue;
            }
            legal_moves += 1;
            local_pv.clear();

            if legal_moves == 1 {
                eval = -self.search(-beta, -alpha, ply + 1, depth - 1, &mut local_pv);
            } else {
                eval = -self.search(-alpha - 1, -alpha, ply + 1, depth - 1, &mut local_pv);

                if (eval > alpha) && (eval < beta) {
                    eval = -self.search(-beta, -alpha, ply + 1, depth - 1, &mut local_pv);
                }
            }

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
                pv.push(m);
                let _ = pv.try_extend_from_slice(&local_pv);
            }
        }

        if legal_moves == 0 {
            return if MoveGen::new(&self.pos).in_check(self.pos.to_play()) {
                -MATE
            } else {
                0
            };
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
                last_pv: ArrayVec::new(),
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

#[cfg(test)]
mod test {
    use arrayvec::ArrayVec;

    use crate::{
        mmove::MoveBuilder,
        piece::mkp,
        position::{
            locus::loc,
            movegen::{MoveGen, MoveList},
            Position,
        },
    };

    use super::SearchBuilder;

    #[test]
    fn move_ordering() {
        let mut pos = Position::default();
        let principle_move = MoveBuilder::new(mkp!(White, Pawn), loc!(g 2))
            .with_dst(loc!(g 3))
            .build();

        let mut srch = SearchBuilder::new(pos.clone()).build();
        srch.last_pv = ArrayVec::from_iter(vec![
            MoveBuilder::new(mkp!(Black, Pawn), loc!(a 3))
                .with_dst(loc!(a 4))
                .build(),
            MoveBuilder::new(mkp!(Black, Pawn), loc!(a 3))
                .with_dst(loc!(a 4))
                .build(),
            principle_move,
            MoveBuilder::new(mkp!(Black, Pawn), loc!(a 3))
                .with_dst(loc!(a 4))
                .build(),
        ]);
        let mut moves = MoveGen::new(&mut pos).gen();
        srch.order_moves(2, &mut moves);

        let low_val_capture = MoveBuilder::new(mkp!(Black, Queen), loc!(a 1))
            .with_dst(loc!(b 1))
            .with_capture(mkp!(White, Pawn));

        let mid_val_capture = MoveBuilder::new(mkp!(Black, Rook), loc!(a 1))
            .with_dst(loc!(b 1))
            .with_capture(mkp!(White, Bishop));

        let high_val_capture = MoveBuilder::new(mkp!(Black, Pawn), loc!(a 1))
            .with_dst(loc!(b 1))
            .with_capture(mkp!(White, Queen));

        let no_capture = MoveBuilder::new(mkp!(Black, Pawn), loc!(a 1)).with_dst(loc!(b 1));

        let mut some_moves = MoveList::new();
        some_moves.push(no_capture.build());
        some_moves.push(low_val_capture.build());
        some_moves.push(mid_val_capture.build());
        some_moves.push(high_val_capture.build());

        srch.order_moves(2, &mut some_moves);

        assert_eq!(
            some_moves.to_vec(),
            vec![
                high_val_capture,
                mid_val_capture,
                low_val_capture,
                no_capture,
            ]
            .iter()
            .map(|x| x.build())
            .collect::<Vec<_>>()
        );

        assert_eq!(*moves.first().unwrap(), principle_move);
    }
}
