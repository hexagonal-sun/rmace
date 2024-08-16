use std::fmt::{Display, Write};

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

impl Display for PieceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match *self {
            PieceKind::Pawn => 'p',
            PieceKind::Rook => 'r',
            PieceKind::Knight => 'n',
            PieceKind::Bishop => 'b',
            PieceKind::King => 'k',
            PieceKind::Queen => 'q',
        };
        f.write_char(c)
    }
}
