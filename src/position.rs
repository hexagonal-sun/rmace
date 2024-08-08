use bitboard::BitBoard;
use strum::EnumCount;

use crate::piece::{Colour, PieceKind};

pub mod bitboard;
pub mod locus;
pub mod movegen;

#[derive(Clone, PartialEq)]
struct Position {
    bboards: [BitBoard; PieceKind::COUNT * 2],
    to_play: Colour,
}

impl Position {
    pub fn lookup(&self, colour: Colour, kind: PieceKind) -> BitBoard {
        self.bboards[colour as usize + kind as usize]
    }

    pub fn movegen(&self) -> Vec<Position> {
        todo!()
    }
}
