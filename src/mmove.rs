use std::fmt::Debug;

use crate::{
    piece::{Piece, PieceKind},
    position::locus::{Locus, Rank},
};

#[derive(Clone, Copy, PartialEq)]
pub enum CastlingMoveType {
    Queenside,
    Kingside,
}

#[derive(Clone, Copy, PartialEq)]
pub struct Move {
    pub piece: Piece,
    pub src: Locus,
    pub dst: Locus,
    pub capture: Option<Piece>,
    pub promote: Option<Piece>,
    pub castling_move: Option<CastlingMoveType>,
    pub set_ep: bool,
}

impl Move {
    pub fn ep_loc(self) -> Option<Locus> {
        if self.set_ep {
            let (r, _) = self.src.to_rank_file();
            if r == Rank::Two {
                self.src.north()
            } else {
                self.src.south()
            }
        } else {
            None
        }
    }
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
    sets_ep: bool,
    castling_move: Option<CastlingMoveType>,
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
                sets_ep: false,
                castling_move: None,
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
            set_ep: self.extra.sets_ep,
            castling_move: self.extra.castling_move,
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

    pub fn sets_ep(mut self) -> Self {
        assert_eq!(self.piece.kind(), PieceKind::Pawn);
        self.extra.sets_ep = true;
        self
    }

    pub fn is_castling_move(mut self, kind: CastlingMoveType) -> Self {
        self.extra.castling_move = Some(kind);
        self
    }
}
