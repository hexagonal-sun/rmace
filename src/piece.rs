use std::mem::variant_count;

#[derive(Clone, Copy, PartialEq)]
pub enum Colour {
    White = 0,
    Black = variant_count::<PieceKind>() as isize,
}

impl Colour {
    pub fn next(self) -> Colour {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum PieceKind {
    Pawn,
    Rook,
    Knight,
    Bishop,
    King,
    Queen,
}
