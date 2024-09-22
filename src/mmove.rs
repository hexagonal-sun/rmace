use std::{fmt::Debug, marker::PhantomData, mem::MaybeUninit, ptr::addr_of_mut};

use crate::{piece::Piece, position::locus::Locus};

#[derive(Clone, Copy, PartialEq)]
pub enum CastlingMoveType {
    Queenside,
    Kingside,
}

#[derive(Clone, Copy, PartialEq)]
pub enum MoveType {
    Normal,
    DoublePPush,
    EnPassant,
    Castle(CastlingMoveType),
    Promote(Piece),
}

#[derive(Clone, Copy, PartialEq)]
pub struct Move {
    pub piece: Piece,
    pub src: Locus,
    pub dst: Locus,
    pub capture: Option<Piece>,
    pub kind: MoveType,
}

#[rustfmt::skip]
const MVV_LVA: [[i32; 12]; 12] = [
    [105, 205, 305, 405, 505, 605,  105, 205, 305, 405, 505, 605],
    [104, 204, 304, 404, 504, 604,  104, 204, 304, 404, 504, 604],
    [103, 203, 303, 403, 503, 603,  103, 203, 303, 403, 503, 603],
    [102, 202, 302, 402, 502, 602,  102, 202, 302, 402, 502, 602],
    [101, 201, 301, 401, 501, 601,  101, 201, 301, 401, 501, 601],
    [100, 200, 300, 400, 500, 600,  100, 200, 300, 400, 500, 600],

    [105, 205, 305, 405, 505, 605,  105, 205, 305, 405, 505, 605],
    [104, 204, 304, 404, 504, 604,  104, 204, 304, 404, 504, 604],
    [103, 203, 303, 403, 503, 603,  103, 203, 303, 403, 503, 603],
    [102, 202, 302, 402, 502, 602,  102, 202, 302, 402, 502, 602],
    [101, 201, 301, 401, 501, 601,  101, 201, 301, 401, 501, 601],
    [100, 200, 300, 400, 500, 600,  100, 200, 300, 400, 500, 600],
];

impl Move {
    // Compute MVV-LVA for a given move.
    pub fn mvv_lva(self) -> i32 {
        match self.capture {
            Some(captured) => MVV_LVA[self.piece.to_idx() as usize][captured.to_idx() as usize],
            None => 0,
        }
    }
}

impl Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> {}", self.src, self.dst)?;

        if let MoveType::Promote(p) = self.kind {
            write!(f, " promotes to {p:?}")?;
        }

        if let Some(capture) = self.capture {
            write!(f, " takes {capture:?}")?;
        }

        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct NeedsDst;

#[derive(Clone, Copy)]
pub struct HasDst;

#[derive(Clone, Copy)]
pub struct MoveBuilder<T: Clone + Copy> {
    mmove: MaybeUninit<Move>,
    phantom: PhantomData<T>,
}

impl MoveBuilder<()> {
    pub fn new(piece: Piece, src: Locus) -> MoveBuilder<NeedsDst> {
        let mut mmove: MaybeUninit<Move> = MaybeUninit::uninit();
        let ptr = mmove.as_mut_ptr();

        unsafe {
            addr_of_mut!((*ptr).src).write(src);
            addr_of_mut!((*ptr).piece).write(piece);
        }

        MoveBuilder {
            mmove,
            phantom: PhantomData,
        }
    }
}

impl MoveBuilder<NeedsDst> {
    pub fn with_dst(mut self, dst: Locus) -> MoveBuilder<HasDst> {
        let ptr = self.mmove.as_mut_ptr();

        unsafe {
            addr_of_mut!((*ptr).dst).write(dst);
            addr_of_mut!((*ptr).capture).write(None);
            addr_of_mut!((*ptr).kind).write(MoveType::Normal);
        }

        MoveBuilder {
            mmove: self.mmove,
            phantom: PhantomData,
        }
    }
}

impl MoveBuilder<HasDst> {
    pub fn build(self) -> Move {
        unsafe { self.mmove.assume_init() }
    }

    pub fn with_capture(mut self, piece: Piece) -> Self {
        let ptr = self.mmove.as_mut_ptr();

        unsafe {
            addr_of_mut!((*ptr).capture).write(Some(piece));
        }

        self
    }

    pub fn with_pawn_promotion(mut self, piece: Piece) -> Self {
        let ptr = self.mmove.as_mut_ptr();

        unsafe {
            addr_of_mut!((*ptr).kind).write(MoveType::Promote(piece));
        }

        self
    }

    pub fn is_en_passant_capture(mut self) -> Self {
        let ptr = self.mmove.as_mut_ptr();

        unsafe {
            addr_of_mut!((*ptr).kind).write(MoveType::EnPassant);
        }

        self
    }

    pub fn is_double_pawn_push(mut self) -> Self {
        let ptr = self.mmove.as_mut_ptr();

        unsafe {
            addr_of_mut!((*ptr).kind).write(MoveType::DoublePPush);
        }

        self
    }

    pub fn is_castling_move(mut self, kind: CastlingMoveType) -> Self {
        let ptr = self.mmove.as_mut_ptr();

        unsafe {
            addr_of_mut!((*ptr).kind).write(MoveType::Castle(kind));
        }

        self
    }
}
