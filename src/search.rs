use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, sleep},
    time::{Duration, Instant},
};

use arrayvec::ArrayVec;
use time::{TimeAction, TimeMan};
use ttable::{EntryKind, TEntry, TTable};

use crate::{
    mmove::Move,
    piece::Colour,
    position::{
        eval::Evaluator,
        movegen::{MoveGen, MoveList},
        Position,
    },
};

mod time;
mod ttable;

const MAX_PLY: usize = 100;

type PvStack = ArrayVec<Move, MAX_PLY>;

#[derive(Default)]
pub struct SearchResults {
    pub depth: usize,
    pub pv: PvStack,
    pub eval: i32,
    pub nodes: u32,
    pub qnodes: u32,
    pub ttable_hits: u32,
    pub beta_cutoffs: u32,
    pub alpha_increases: u32,
}

pub struct Search {
    pos: Position,
    should_exit: Arc<AtomicBool>,
    last_pv: PvStack,
    ttable: TTable,
    time: TimeMan,
    report_callback: Option<Box<dyn Fn(&SearchResults)>>,
    results: SearchResults,
}

const INF: i32 = i32::MAX - 2;
pub const MATE: i32 = INF - 1;

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

    pub fn get_initial_move(&mut self) -> Option<Move> {
        let mut moves = MoveGen::new(&self.pos).gen();
        moves.sort_by(|x, y| y.mvv_lva().cmp(&x.mvv_lva()));
        moves.first().copied()
    }

    fn obtain_pv(&mut self) {
        self.results.pv.clear();
        let mut pos = self.pos.clone();

        while let Some(tentry) = self.ttable.lookup(pos.hash()) {
            match tentry.kind {
                EntryKind::Score(mv) => {
                    self.results.pv.push(mv);
                    pos.make_move(mv).consume();
                }
                _ => panic!("Unexpected tentry node type in PV"),
            }
        }
    }

    pub fn go(mut self) -> Move {
        let mut best_move = self.get_initial_move();

        let mut depth = 1;
        let mut deadline = Duration::MAX;

        loop {
            self.results = SearchResults::default();
            self.results.depth = depth;
            let now = Instant::now();
            self.should_exit = Arc::new(AtomicBool::new(false));
            let should_exit = self.should_exit.clone();

            thread::spawn(move || {
                sleep(deadline);
                should_exit.store(true, Ordering::Relaxed);
            });

            self.results.eval = self.search(-INF, INF, 0, depth as u32);

            // Take the last move from the previous iteration, since when the
            // exit flag is true, we didn't complete the search.
            if self.should_exit.load(Ordering::Relaxed) {
                return best_move.unwrap();
            }

            self.obtain_pv();
            self.last_pv = self.results.pv.clone();

            best_move = self.last_pv.first().copied();

            if let Some(ref cb) = self.report_callback {
                cb(&self.results);
            }

            match self
                .time
                .iter_complete(self.results.eval, best_move.unwrap(), now.elapsed())
            {
                time::TimeAction::YieldResult => return best_move.unwrap(),
                TimeAction::Iterate(d) => deadline = d,
            }

            if self.results.eval == MATE || self.results.eval == -MATE {
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

        if (self.results.nodes & 0xfff == 0xfff) && self.should_exit.load(Ordering::Relaxed) {
            return 0;
        }

        let mut cap_moves = MoveGen::new(&mut self.pos).gen();
        cap_moves.retain(|x| x.capture.is_some());

        for cap_move in cap_moves {
            self.results.nodes += 1;
            self.results.qnodes += 1;

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

    fn search(&mut self, mut alpha: i32, beta: i32, ply: u32, depth: u32) -> i32 {
        if let Some(entry) = self.ttable.lookup(self.pos.hash()) {
            if entry.kind.is_score() && (entry.eval == -MATE || entry.eval == MATE) {
                return entry.eval;
            }

            if entry.depth >= depth {
                self.results.ttable_hits += 1;
                match entry.kind {
                    EntryKind::Score(_) => {
                        return entry.eval;
                    }
                    EntryKind::Alpha => {
                        if entry.eval <= alpha {
                            return alpha;
                        }
                    }
                    EntryKind::Beta => {
                        if entry.eval >= beta {
                            return beta;
                        }
                    }
                }
            }
        }

        if self.pos.has_repeated() {
            return 0;
        }

        if depth == 0 {
            return self.quiescence(alpha, beta);
        }

        let mut mmoves = MoveGen::new(&mut self.pos).gen();
        self.order_moves(ply, &mut mmoves);

        let mut legal_moves = 0;
        let mut eval = -INF;

        let mut tentry = TEntry {
            hash: self.pos.hash(),
            depth,
            kind: EntryKind::Alpha,
            eval,
        };

        for m in mmoves {
            let token = self.pos.make_move(m);
            if MoveGen::new(&self.pos).in_check(self.pos.to_play().next()) {
                self.pos.undo_move(token);
                continue;
            }
            legal_moves += 1;

            if legal_moves == 1 {
                eval = -self.search(-beta, -alpha, ply + 1, depth - 1);
            } else {
                eval = -self.search(-alpha - 1, -alpha, ply + 1, depth - 1);

                if (eval > alpha) && (eval < beta) {
                    eval = -self.search(-beta, -alpha, ply + 1, depth - 1);
                }
            }

            self.pos.undo_move(token);

            // Timeout detection.
            if (self.results.nodes & 0xfff == 0xfff) && self.should_exit.load(Ordering::Relaxed) {
                return 0;
            }

            self.results.nodes += 1;

            if eval >= beta {
                tentry.kind = EntryKind::Beta;
                tentry.eval = beta;
                self.ttable.insert(tentry);
                self.results.beta_cutoffs += 1;
                return beta;
            }

            if eval > alpha {
                alpha = eval;
                tentry.kind = EntryKind::Score(m);
                self.results.alpha_increases += 1;
            }
        }

        if legal_moves == 0 {
            return if MoveGen::new(&self.pos).in_check(self.pos.to_play()) {
                -MATE
            } else {
                0
            };
        }

        tentry.eval = alpha;
        self.ttable.insert(tentry);

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
                results: SearchResults::default(),
                should_exit: Arc::new(AtomicBool::new(false)),
                last_pv: ArrayVec::new(),
                ttable: TTable::new(),
                time: TimeMan::new(),
                report_callback: None,
            },
        }
    }

    pub fn with_deadline(mut self, deadline: Duration) -> Self {
        self.srch.time.time_left = Some(deadline);
        self
    }

    pub fn with_report_callback(mut self, callback: impl Fn(&SearchResults) + 'static) -> Self {
        self.srch.report_callback = Some(Box::new(callback));
        self
    }

    pub fn with_increment(mut self, increment: Duration) -> Self {
        self.srch.time.increment = Some(increment);
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
