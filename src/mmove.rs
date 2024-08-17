use std::fmt::Debug ;

use crate::{piece::Piece, position::locus::Locus};

#[derive(Clone, Copy, PartialEq)]
pub struct PieceMove {
    pub piece: Piece,
    pub src: Locus,
    pub dst: Locus,
}

impl Debug for PieceMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} -> {:?}", self.src, self.dst)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Move {
    Move(PieceMove),
    Attack(PieceMove, Piece),
}

impl Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Move::Move(m) => write!(f, "{m:?}"),
            Move::Attack(m, p) => write!(f, "{m:?} takes {p:?}"),
        }
    }
}
