use crate::piece::{Colour, Piece};

use super::{bitboard::BitBoard, locus::Locus, Position};

pub struct PositionBuilder {
    pos: Position,
}

impl PositionBuilder {
    pub fn new() -> Self {
        Self {
            pos: Position::empty(),
        }
    }

    pub fn with_next_turn(mut self, colour: Colour) -> Self {
        self.pos.to_play = colour;

        self
    }

    pub fn with_piece_board(mut self, p: Piece, bb: BitBoard) -> Self {
        self.pos[p] = bb;

        self
    }

    pub fn with_piece_at(mut self, p: Piece, l: Locus) -> Self {
        let bb = self.pos[p].set_piece_at(l);

        self.with_piece_board(p, bb)
    }

    pub fn build(self) -> Position {
        self.pos
    }
}
