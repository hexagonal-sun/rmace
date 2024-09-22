use crate::mmove::Move;
use arrayvec::ArrayVec;
use itertools::Itertools;
use std::time::Duration;

const MAX_DEPTH: usize = 20;
const MIN_EARLY_YIELD_DEPTH: usize = 5;

pub enum TimeAction {
    YieldResult,
    Iterate(Duration),
}

#[derive(Clone)]
pub struct TimeMan {
    pub(super) time_left: Option<Duration>,
    pub(super) increment: Option<Duration>,
    scores: ArrayVec<i32, MAX_DEPTH>,
    best_moves: ArrayVec<Move, MAX_DEPTH>,
}

struct Stats {
    max: i32,
    min: i32,
}

impl Stats {
    fn new() -> Self {
        Self {
            max: i32::MIN,
            min: i32::MAX,
        }
    }

    pub fn add_value(&mut self, v: i32) {
        self.max = i32::max(self.max, v);
        self.min = i32::min(self.min, v);
    }

    pub fn range(&self) -> i32 {
        self.max - self.min
    }
}

impl TimeMan {
    pub fn new() -> Self {
        Self {
            time_left: None,
            increment: None,
            scores: ArrayVec::new(),
            best_moves: ArrayVec::new(),
        }
    }

    pub fn iter_complete(
        &mut self,
        score: i32,
        best_move: Move,
        time_taken: Duration,
    ) -> TimeAction {
        self.scores.push(score);
        self.best_moves.push(best_move);
        let depth = self.best_moves.len();

        if let Some(ref mut d) = self.time_left {
            *d -= time_taken;

            // If we have less than 5 millies remaining, yield now since even a
            // depth 2 search could take longer.
            if *d < Duration::from_millis(5) {
                return TimeAction::YieldResult;
            }
        }

        let percent_time_to_use = if score < -500 {
            // Things haven't gone great. Use more time up in the hoeps that we
            // can maybe recover the position.
            0.35
        } else {
            0.15
        };

        let time_left = self
            .time_left
            .map(|x| x.mul_f32(percent_time_to_use))
            .unwrap_or(Duration::from_secs(5));

        if depth < MIN_EARLY_YIELD_DEPTH {
            return TimeAction::Iterate(time_left);
        }

        let score_stats =
            self.scores
                .iter()
                .rev()
                .take(MIN_EARLY_YIELD_DEPTH)
                .fold(Stats::new(), |mut s, x| {
                    s.add_value(*x);
                    s
                });

        if score_stats.range() < 20 {
            return TimeAction::YieldResult;
        }

        if self
            .best_moves
            .iter()
            .rev()
            .take(MIN_EARLY_YIELD_DEPTH)
            .all_equal()
        {
            return TimeAction::YieldResult;
        }

        TimeAction::Iterate(time_left)
    }
}
