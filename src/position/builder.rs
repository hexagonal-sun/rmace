use anyhow::{bail, Result};
use strum::IntoEnumIterator;

use crate::piece::{Colour, Piece, PieceKind};

use super::{
    bitboard::BitBoard,
    castling_rights::CastlingRights,
    locus::{Locus, Rank},
    Position,
};

pub struct PositionBuilder {
    pos: Position,
}

impl Default for PositionBuilder {
    fn default() -> Self {
        Self::new()
    }
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

    pub fn with_piece_at(self, p: Piece, l: Locus) -> Self {
        let bb = self.pos[p].set_piece_at(l);

        self.with_piece_board(p, bb)
    }

    pub fn with_en_passant(mut self, l: Locus) -> Result<Self> {
        let (r, _) = l.to_rank_file();
        if r != Rank::Six && r != Rank::Three {
            bail!("Invalid rank for en-passant locus")
        } else {
            self.pos.en_passant = Some(l);
            Ok(self)
        }
    }

    pub fn with_castling_rights(mut self, cr: CastlingRights) -> Self {
        self.pos.castling_rights = cr;
        self
    }

    pub fn build(mut self) -> Position {
        let mut pieces = 0u8;
        PieceKind::iter().for_each(|k| {
            pieces += self.pos[Piece::new(k, Colour::White)].popcount() as u8;
            pieces += self.pos[Piece::new(k, Colour::Black)].popcount() as u8;
        });
        self.pos.material_count = pieces;
        self.pos
    }
}
