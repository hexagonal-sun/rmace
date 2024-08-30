use std::fmt::Debug;

use crate::{piece::Piece, position::locus::Locus};

#[derive(Clone, Copy, PartialEq)]
pub struct Move {
    pub piece: Piece,
    pub src: Locus,
    pub dst: Locus,
    pub capture: Option<Piece>,
    pub promote: Option<Piece>,
    // pub en_passant_capture: bool,
    // pub set_ep: bool,
}

impl Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} -> {:?}", self.src, self.dst)?;

        if let Some(promotion) = self.promote {
            write!(f, " promotes to {promotion:?}")?;
        }

        if let Some(capture) = self.capture {
            write!(f, " takes {capture:?}")?;
        }

        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct NeedsDst {}

#[derive(Clone, Copy)]
pub struct HasDst {
    dst: Locus,
    promotion: Option<Piece>,
    capture: Option<Piece>,
}

#[derive(Clone, Copy)]
pub struct MoveBuilder<T: Clone + Copy> {
    piece: Piece,
    src: Locus,
    extra: T,
}

impl MoveBuilder<()> {
    pub fn new(piece: Piece, src: Locus) -> MoveBuilder<NeedsDst> {
        MoveBuilder {
            piece,
            src,
            extra: NeedsDst {},
        }
    }
}

impl MoveBuilder<NeedsDst> {
    pub fn with_dst(self, dst: Locus) -> MoveBuilder<HasDst> {
        MoveBuilder {
            piece: self.piece,
            src: self.src,
            extra: HasDst {
                dst,
                promotion: None,
                capture: None,
            },
        }
    }
}

impl MoveBuilder<HasDst> {
    pub fn build(self) -> Move {
        Move {
            piece: self.piece,
            src: self.src,
            dst: self.extra.dst,
            capture: self.extra.capture,
            promote: self.extra.promotion,
        }
    }

    pub fn with_capture(mut self, piece: Piece) -> Self {
        self.extra.capture = Some(piece);
        self
    }

    pub fn with_pawn_promotion(mut self, piece: Piece) -> Self {
        self.extra.promotion = Some(piece);
        self
    }
}
