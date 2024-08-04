use std::mem::variant_count;

use crate::piece::{Colour, PieceKind};

mod locus;
mod movegen;

#[derive(Clone, Copy, PartialEq)]
#[repr(transparent)]
struct BitBoard {
    inner: u64,
}

#[derive(Clone, PartialEq)]
struct Position {
    bboards: [BitBoard; variant_count::<PieceKind>() * 2],
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
