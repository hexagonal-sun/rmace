use std::cmp::{max, min};

use crate::{
    mmove::Move,
    piece::Colour,
    position::{eval::Evaluator, Position},
};

fn search(pos: &mut Position, depth: u8) -> i32 {
    if depth == 0 {
        return Evaluator::eval(pos);
    }

    let mut eval: i32 = if pos.to_play() == Colour::White {
        i32::MIN
    } else {
        i32::MAX
    };

    for m in pos.movegen() {
        let token = pos.make_move(m);
        let v = search(pos, depth - 1);
        pos.undo_move(token);
        eval = if pos.to_play() == Colour::White {
            max(eval, v)
        } else {
            min(eval, v)
        }
    }

    eval
}

pub struct Search<'a> {
    pos: &'a mut Position,
}

impl<'a> Search<'a> {
    pub fn new(pos: &'a mut Position) -> Self {
        Self { pos }
    }

    pub fn go(&mut self) -> Move {
        let mmoves = self.pos.movegen();
        let mut mmoves: Vec<_> = mmoves
            .iter()
            .map(|m| {
                let token = self.pos.make_move(*m);
                let value = search(self.pos, 4);
                self.pos.undo_move(token);
                (m, value)
            })
            .collect();

        mmoves.sort_by(|x, y| x.1.cmp(&y.1));

        *mmoves[0].0
    }
}
