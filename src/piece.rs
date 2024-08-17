use std::fmt::{Display, Write};

use num_enum::{FromPrimitive, TryFromPrimitive};
use strum::{EnumCount, EnumIter};

#[derive(Clone, Copy, PartialEq, EnumIter, Debug)]
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

#[derive(Clone, Copy, PartialEq, EnumCount, EnumIter, TryFromPrimitive, Debug)]
#[repr(usize)]
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

macro_rules! mkp {
    ($colour:ident, $kind:ident) => {
        crate::piece::Piece::new(
            crate::piece::PieceKind::$kind,
            crate::piece::Colour::$colour,
        )
    };
}

pub(crate) use mkp;

#[derive(Clone, Copy, PartialEq)]
pub struct Piece {
    pub idx: usize,
}

impl Piece {
    pub fn new(kind: PieceKind, colour: Colour) -> Self {
        Self {
            idx: kind as usize + colour as usize,
        }
    }

    pub fn kind(self) -> PieceKind {
        PieceKind::try_from(self.idx % PieceKind::COUNT).unwrap()
    }

    pub fn colour(self) -> Colour {
        if self.idx / PieceKind::COUNT == 0 {
            Colour::White
        } else {
            Colour::Black
        }
    }
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let piece = format!("{}", self.kind());

        let s = if self.colour() == Colour::Black {
            piece
        } else {
            piece.to_ascii_uppercase()
        };

        f.write_str(s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use crate::piece::PieceKind;

    use super::{Colour, Piece};

    #[test]
    fn piece_colour_and_kind() {
        for c in Colour::iter() {
            for p in PieceKind::iter() {
                let piece = Piece::new(p, c);
                assert_eq!(piece.kind(), p);
                assert_eq!(piece.colour(), c);
            }
        }
    }
}
