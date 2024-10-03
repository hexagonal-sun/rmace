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
    mmove::{Move, MoveType},
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
    pv: ArrayVec<PvStack, MAX_PLY>,
    ttable: TTable,
    time: TimeMan,
    report_callback: Option<Box<dyn Fn(&SearchResults)>>,
    to_depth: Option<usize>,
    results: SearchResults,
}

const INF: i32 = i32::MAX - 2;
pub const MATE: i32 = INF - 1;

impl Search {
    pub fn order_moves(&self, moves: &mut MoveList) {
        // order captures first.
        moves.sort_by(|x, y| y.mvv_lva().cmp(&x.mvv_lva()));

        // then promotions.
        moves.sort_by(|x, y| {
            let x_sc = match x.kind {
                MoveType::Promote(p) => p.kind().score(),
                _ => 0,
            };

            let y_sc = match y.kind {
                MoveType::Promote(p) => p.kind().score(),
                _ => 0,
            };

            y_sc.cmp(&x_sc)
        });

        // Always investigate the corresponding node from the previous PV first
        if let Some(tentry) = self.ttable.lookup(self.pos.hash()) {
            match tentry.kind {
                EntryKind::Score(m) => {
                    if let Some(idx) = moves.iter().position(|x| *x == m) {
                        moves.swap(idx, 0);
                    }
                }
                _ => {}
            }
        }
    }

    pub fn get_initial_move(&mut self) -> Option<Move> {
        let mut moves = MoveGen::new(&self.pos).gen();
        moves.sort_by(|x, y| y.mvv_lva().cmp(&x.mvv_lva()));
        moves.first().copied()
    }

    pub fn go(mut self) -> SearchResults {
        let mut depth = 1;
        let mut deadline = Duration::MAX;
        let mut last_results = SearchResults::default();

        loop {
            self.results = SearchResults::default();
            self.results.depth = depth;
            let now = Instant::now();
            self.should_exit = Arc::new(AtomicBool::new(false));
            let should_exit = self.should_exit.clone();

            if self.to_depth.is_none() {
                thread::spawn(move || {
                    sleep(deadline);
                    should_exit.store(true, Ordering::Relaxed);
                });
            }

            self.results.eval = self.search(-INF, INF, 0, depth as u32);

            // Take the last results from the previous iteration, since when the
            // exit flag is true, we didn't complete the search.
            if self.should_exit.load(Ordering::Relaxed) {
                return last_results;
            }

            self.results.pv = self.pv[0].clone();

            if let Some(ref cb) = self.report_callback {
                cb(&self.results);
            }

            if let Some(srch_depth) = self.to_depth {
                if srch_depth == self.results.depth {
                    return self.results;
                }
            } else {
                match self.time.iter_complete(
                    self.results.eval,
                    *self.results.pv.first().unwrap(),
                    now.elapsed(),
                ) {
                    time::TimeAction::YieldResult => return self.results,
                    TimeAction::Iterate(d) => deadline = d,
                }
            }

            if self.results.eval == MATE || self.results.eval == -MATE {
                return self.results;
            }

            depth += 1;
            last_results = self.results;
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

    fn search(&mut self, mut alpha: i32, beta: i32, ply: usize, depth: u32) -> i32 {
        if let Some(entry) = self.ttable.lookup(self.pos.hash()) {
            if entry.kind.is_score() && (entry.eval == -MATE || entry.eval == MATE) {
                return entry.eval;
            }

            if entry.depth >= depth {
                self.results.ttable_hits += 1;
                match entry.kind {
                    EntryKind::Score(_) => {
                        self.pv[ply].clear();
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
            self.pv[ply].clear();
            return 0;
        }

        if depth == 0 {
            return self.quiescence(alpha, beta);
        }

        let mut mmoves = MoveGen::new(&mut self.pos).gen();
        self.order_moves(&mut mmoves);

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
                self.pv[ply].clear();
                self.pv[ply].push(m);
                self.pv[ply + 1]
                    .clone()
                    .into_iter()
                    .for_each(|m| self.pv[ply].push(m));
                self.results.alpha_increases += 1;
            }
        }

        if legal_moves == 0 {
            self.pv[ply].clear();
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
                pv: ArrayVec::from_iter((0..MAX_PLY).map(|_| PvStack::new())),
                ttable: TTable::new(),
                time: TimeMan::new(),
                to_depth: None,
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

    pub fn with_depth(mut self, depth: usize) -> Self {
        self.srch.to_depth = Some(depth);
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
        }, search::{ttable::{EntryKind, TEntry}, MATE},
    };

    use super::SearchBuilder;

    #[test]
    fn move_ordering() {
        let mut pos = Position::default();
        let principle_move = MoveBuilder::new(mkp!(White, Pawn), loc!(g 2))
            .with_dst(loc!(g 3))
            .build();

        let mut srch = SearchBuilder::new(pos.clone()).build();
        srch.ttable.insert(TEntry {
            hash: pos.hash(),
            depth: 1,
            kind: EntryKind::Score(principle_move),
            eval: 5,
        });

        let mut moves = MoveGen::new(&mut pos).gen();
        srch.order_moves(&mut moves);
        assert_eq!(*moves.first().unwrap(), principle_move);

        let low_val_capture = MoveBuilder::new(mkp!(Black, Queen), loc!(a 1))
            .with_dst(loc!(b 1))
            .with_capture(mkp!(White, Pawn));

        let mid_val_capture = MoveBuilder::new(mkp!(Black, Rook), loc!(a 1))
            .with_dst(loc!(b 1))
            .with_capture(mkp!(White, Bishop));

        let high_val_capture = MoveBuilder::new(mkp!(Black, Pawn), loc!(a 1))
            .with_dst(loc!(b 1))
            .with_capture(mkp!(White, Queen));

        let high_val_promote = MoveBuilder::new(mkp!(Black, Pawn), loc!(a 1))
            .with_dst(loc!(b 1))
            .with_pawn_promotion(mkp!(Black, Queen));

        let low_val_promote = MoveBuilder::new(mkp!(Black, Pawn), loc!(a 1))
            .with_dst(loc!(b 1))
            .with_pawn_promotion(mkp!(Black, Rook));

        let no_capture = MoveBuilder::new(mkp!(Black, Pawn), loc!(a 1)).with_dst(loc!(b 1));

        let mut some_moves = MoveList::new();
        some_moves.push(no_capture.build());
        some_moves.push(low_val_promote.build());
        some_moves.push(high_val_promote.build());
        some_moves.push(low_val_capture.build());
        some_moves.push(mid_val_capture.build());
        some_moves.push(high_val_capture.build());

        srch.order_moves(&mut some_moves);

        assert_eq!(
            some_moves.to_vec(),
            vec![
                high_val_promote,
                low_val_promote,
                high_val_capture,
                mid_val_capture,
                low_val_capture,
                no_capture,
            ]
            .iter()
            .map(|x| x.build())
            .collect::<Vec<_>>()
        );
    }

    #[test]
    fn mate_3_pos1() {
        let pos =
            Position::from_fen("4r1k1/p1qn1ppp/1p3n2/4NR2/3P4/B5Q1/P1r3PP/R6K w - - 1 20").unwrap();

        let results = SearchBuilder::new(pos).with_depth(6).build().go();

        assert_eq!(results.eval, MATE);
    }

    #[test]
    fn mate_3_pos2() {
        let pos =
            Position::from_fen("r1b1k2r/pp3p2/6p1/3pq3/1P4P1/P2BPQp1/5PP1/R4RK1 b kq - 1 20").unwrap();

        let results = SearchBuilder::new(pos).with_depth(6).build().go();

        assert_eq!(results.eval, MATE);
    }

    #[test]
    fn mate_3_pos3() {
        let pos =
            Position::from_fen("1r4k1/4pp1p/3p2p1/1P1Pn3/Q3P3/3n2PP/5qBK/1R3R2 b - - 1 31").unwrap();

        let results = SearchBuilder::new(pos).with_depth(6).build().go();

        assert_eq!(results.eval, MATE);
    }

    #[test]
    fn mate_4_pos1() {
        let pos =
            Position::from_fen("1r4k1/4pp1p/3p2p1/1P1Pn3/Q3P3/3n2PP/5qBK/1R3R2 b - - 1 31").unwrap();

        let results = SearchBuilder::new(pos).with_depth(6).build().go();

        assert_eq!(results.eval, MATE);
    }
}
