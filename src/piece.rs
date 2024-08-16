use strum::{EnumCount, EnumIter};

#[derive(Clone, Copy, PartialEq)]
pub enum Colour {
    White = 0,
    Black = PieceKind::COUNT as isize,
}

impl Colour {
    pub fn next(self) -> Colour {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

#[derive(Clone, Copy, PartialEq, EnumCount, EnumIter)]
pub enum PieceKind {
    Pawn,
    Rook,
    Knight,
    Bishop,
    King,
    Queen,
}
