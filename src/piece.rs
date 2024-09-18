use std::fmt::{Debug, Display, Write};

use num_enum::TryFromPrimitive;
use strum::{EnumCount, EnumIter};

#[derive(Clone, Copy, PartialEq, EnumIter, Debug)]
pub enum Colour {
    White = 0,
    Black = 1,
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
#[repr(u8)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceKind {
    pub fn score(self) -> u32 {
        match self {
            PieceKind::Pawn => 100,
            PieceKind::Knight => 300,
            PieceKind::Bishop => 350,
            PieceKind::Rook => 500,
            PieceKind::Queen => 1000,
            PieceKind::King => 10000,
        }
    }
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
    pub idx: u8,
}

impl Debug for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {:?}", self.colour(), self.kind())
    }
}

impl Piece {
    pub fn new(kind: PieceKind, colour: Colour) -> Self {
        Self {
            idx: kind as u8 + (colour as u8 * PieceKind::COUNT as u8),
        }
    }

    pub fn kind(self) -> PieceKind {
        PieceKind::try_from(self.idx % PieceKind::COUNT as u8).unwrap()
    }

    pub fn colour(self) -> Colour {
        if self.idx / PieceKind::COUNT as u8 == 0 {
            Colour::White
        } else {
            Colour::Black
        }
    }

    pub fn to_idx(self) -> usize {
        self.idx as usize
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
