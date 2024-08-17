use crate::{piece::Piece, position::locus::Locus};

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct PieceMove {
    pub piece: Piece,
    pub src: Locus,
    pub dst: Locus,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Move {
    Move(PieceMove),
    Attack(PieceMove, Piece),
}
