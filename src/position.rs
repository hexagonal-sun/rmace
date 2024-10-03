use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};

use arrayvec::ArrayVec;
use bitboard::BitBoard;
use builder::PositionBuilder;
use castling_rights::CastlingRights;
use eval::Evaluator;
use locus::{loc, File, Locus, Rank};
use strum::{EnumCount, IntoEnumIterator};
use zobrist::{Zobrist, ZobristKey};

use crate::{
    mmove::{CastlingMoveType, Move, MoveType},
    piece::{mkp, Colour, Piece, PieceKind, PieceKindIter},
};

pub mod bitboard;
pub mod builder;
pub mod castling_rights;
pub mod eval;
pub mod fen;
pub mod locus;
pub mod movegen;
pub mod zobrist;

#[must_use = "Moves must either be undone, or made permanent"]
pub struct UndoToken;

impl UndoToken {
    pub fn consume(self) {}
}

#[derive(Clone, PartialEq)]
struct UndoMove {
    mmove: Move,
    ep_state: Option<Locus>,
    castling_rights: CastlingRights,
    hash: ZobristKey,
}

#[derive(Clone, PartialEq)]
pub struct Position {
    bboards: [BitBoard; PieceKind::COUNT * 2],
    to_play: Colour,
    en_passant: Option<Locus>,
    castling_rights: CastlingRights,
    material_count: u8,
    move_stack: ArrayVec<UndoMove, 512>,
    zobrist: Zobrist,
    hash: ZobristKey,
}

impl Position {
    pub fn to_play(&self) -> Colour {
        self.to_play
    }

    pub fn piece_at_loc(&self, l: Locus) -> Option<Piece> {
        for p in PieceKind::iter() {
            let w_bb = self.bboards[p as usize];
            let b_bb = self.bboards[p as usize + PieceKind::COUNT];

            if w_bb.has_piece_at(l) {
                return Some(Piece::new(p, Colour::White));
            }

            if b_bb.has_piece_at(l) {
                return Some(Piece::new(p, Colour::Black));
            }
        }

        None
    }

    pub fn hash(&self) -> ZobristKey {
        self.hash
    }

    pub fn all_pieces_for_colour(&self, colour: Colour) -> BitBoard {
        let mut b = BitBoard::empty();

        for kind in PieceKind::iter() {
            b = b | self[Piece::new(kind, colour)];
        }

        b
    }

    #[inline(always)]
    pub fn last_move(&self) -> Option<Move> {
        self.move_stack.last().map(|x| x.mmove)
    }

    #[inline(always)]
    fn clr_piece_at(&mut self, p: Piece, loc: Locus) {
        self[p] = self[p].clear_piece_at(loc);
        self.hash ^= self.zobrist.piece_loc_key(p, loc);
    }

    #[inline(always)]
    fn set_piece_at(&mut self, p: Piece, loc: Locus) {
        self[p] = self[p].set_piece_at(loc);
        self.hash ^= self.zobrist.piece_loc_key(p, loc);
    }

    pub fn has_repeated(&self) -> bool {
        if self.move_stack.len() < 2 {
            return false;
        }

        let mut idx = self.move_stack.len() - 2;
        let mut hash_repeated = 0;

        loop {
            if self.move_stack[idx].hash == self.hash {
                hash_repeated += 1;

                if hash_repeated == 2 {
                    return true;
                }
            }

            if idx < 2 {
                return false;
            }

            idx -= 2;
        }
    }

    #[inline(always)]
    fn get_castling_rook_positions(c: Colour, kind: CastlingMoveType) -> (Locus, Locus) {
        match (c, kind) {
            (Colour::White, CastlingMoveType::Kingside) => (loc!(h 1), loc!(f 1)),
            (Colour::White, CastlingMoveType::Queenside) => (loc!(a 1), loc!(d 1)),
            (Colour::Black, CastlingMoveType::Kingside) => (loc!(h 8), loc!(f 8)),
            (Colour::Black, CastlingMoveType::Queenside) => (loc!(a 8), loc!(d 8)),
        }
    }

    #[inline(always)]
    pub fn make_move(&mut self, mmove: Move) -> UndoToken {
        let undo = UndoMove {
            mmove,
            ep_state: self.en_passant,
            castling_rights: self.castling_rights,
            hash: self.hash,
        };

        if let Some(ep_loc) = self.en_passant {
            self.hash ^= self.zobrist.ep_key(ep_loc);
            self.en_passant = None;
        }

        self.clr_piece_at(mmove.piece, mmove.src);

        match mmove.kind {
            MoveType::Normal => {
                if let Some(p) = mmove.capture {
                    self.clr_piece_at(p, mmove.dst);
                }
                self.set_piece_at(mmove.piece, mmove.dst);
            }
            MoveType::DoublePPush => {
                let ep_loc = match mmove.piece.colour() {
                    Colour::White => mmove.dst.south().unwrap(),
                    Colour::Black => mmove.dst.north().unwrap(),
                };
                self.en_passant = Some(ep_loc);
                self.hash ^= self.zobrist.ep_key(ep_loc);
                self.set_piece_at(mmove.piece, mmove.dst);
            }
            MoveType::EnPassant => {
                let c = self.to_play;
                self.clr_piece_at(
                    Piece::new(PieceKind::Pawn, c.next()),
                    match c {
                        Colour::White => mmove.dst.south().unwrap(),
                        Colour::Black => mmove.dst.north().unwrap(),
                    },
                );
                self.set_piece_at(mmove.piece, mmove.dst);
            }
            MoveType::Promote(promo_piece) => {
                if let Some(cap_piece) = mmove.capture {
                    self.clr_piece_at(cap_piece, mmove.dst);
                }
                self.set_piece_at(promo_piece, mmove.dst);
            }
            MoveType::Castle(castle_kind) => {
                let (rook_src, rook_dst) =
                    Self::get_castling_rook_positions(self.to_play, castle_kind);

                let rook = Piece::new(PieceKind::Rook, self.to_play);
                self.set_piece_at(mmove.piece, mmove.dst);
                self.clr_piece_at(rook, rook_src);
                self.set_piece_at(rook, rook_dst);
            }
        }

        if self.castling_rights[self.to_play].has_any() {
            // Clear castling rights.
            match mmove.piece.kind() {
                PieceKind::King => {
                    self.castling_rights[self.to_play].clear_all();
                    self.hash ^= self.zobrist.castling_rights_key(self.to_play(), loc!(a 1));
                    self.hash ^= self.zobrist.castling_rights_key(self.to_play(), loc!(h 1));
                }
                PieceKind::Rook => {
                    self.castling_rights.clear(self.to_play, mmove.src);
                    self.hash ^= self.zobrist.castling_rights_key(self.to_play(), mmove.src);
                }
                _ => {}
            }
        }

        if self.castling_rights[self.to_play().next()].has_any() {
            self.castling_rights.clear(self.to_play().next(), mmove.dst);
        }

        self.to_play = self.to_play.next();

        self.hash ^= self.zobrist.btm_key();
        self.move_stack.push(undo);

        UndoToken
    }

    pub fn undo_move(&mut self, token: UndoToken) {
        token.consume();

        // Safety: We can unwrap here, since the only way for the caller to call
        // undo_move is with an undo token which can only be obtained from
        // make_move.
        let undo = self.move_stack.pop().unwrap();
        let mmove = undo.mmove;
        self.to_play = self.to_play.next();

        match mmove.kind {
            MoveType::Normal => {
                self.clr_piece_at(mmove.piece, mmove.dst);
                if let Some(p) = mmove.capture {
                    self.set_piece_at(p, mmove.dst);
                }
            }
            MoveType::DoublePPush => {
                self.clr_piece_at(mmove.piece, mmove.dst);
            }
            MoveType::EnPassant => {
                self.clr_piece_at(mmove.piece, mmove.dst);
                let c = self.to_play;
                self.set_piece_at(
                    Piece::new(PieceKind::Pawn, c.next()),
                    match c {
                        Colour::White => mmove.dst.south().unwrap(),
                        Colour::Black => mmove.dst.north().unwrap(),
                    },
                );
            }
            MoveType::Promote(promo_piece) => {
                self.clr_piece_at(promo_piece, mmove.dst);
                if let Some(cap_piece) = mmove.capture {
                    self.set_piece_at(cap_piece, mmove.dst);
                }
            }
            MoveType::Castle(castle_kind) => {
                let (rook_src, rook_dst) =
                    Self::get_castling_rook_positions(self.to_play, castle_kind);

                let rook = Piece::new(PieceKind::Rook, self.to_play);
                self.clr_piece_at(mmove.piece, mmove.dst);
                self.set_piece_at(rook, rook_src);
                self.clr_piece_at(rook, rook_dst);
            }
        }
        self.set_piece_at(mmove.piece, mmove.src);

        self[mmove.piece] = self[mmove.piece]
            .set_piece_at(mmove.src)
            .clear_piece_at(mmove.dst);

        self.en_passant = undo.ep_state;
        self.castling_rights = undo.castling_rights;
        self.hash = undo.hash;
    }

    pub fn empty() -> Self {
        Self {
            bboards: [BitBoard::empty(); PieceKind::COUNT * 2],
            to_play: Colour::White,
            en_passant: None,
            castling_rights: CastlingRights::empty(),
            move_stack: ArrayVec::new(),
            material_count: 0,
            zobrist: Zobrist::new(),
            hash: 0,
        }
    }

    pub fn iter_opponent_bbds(&self) -> OpponentBbIter {
        OpponentBbIter {
            pos: self,
            p: PieceKind::iter(),
        }
    }
}

pub struct OpponentBbIter<'a> {
    pos: &'a Position,
    p: PieceKindIter,
}

impl Iterator for OpponentBbIter<'_> {
    type Item = (Piece, BitBoard);

    fn next(&mut self) -> Option<Self::Item> {
        let kind = self.p.next()?;
        let piece = Piece::new(kind, self.pos.to_play.next());

        Some((piece, self.pos[piece]))
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for rank in Rank::iter().rev() {
            write!(f, "{rank} ")?;
            for file in File::iter() {
                write!(
                    f,
                    "{}",
                    self.piece_at_loc(Locus::from_rank_file(rank, file))
                        .map(|x| format!("{x} "))
                        .unwrap_or(". ".to_string())
                )?;
            }
            writeln!(f)?;
        }

        write!(f, "  ")?;
        for file in File::iter() {
            write!(f, "{file:?} ")?;
        }

        writeln!(f, "\nEvaluation: {}", Evaluator::eval(self))?;

        Ok(())
    }
}

impl Index<Piece> for Position {
    type Output = BitBoard;

    fn index(&self, index: Piece) -> &Self::Output {
        self.bboards.index(index.idx as usize)
    }
}

impl IndexMut<Piece> for Position {
    fn index_mut(&mut self, index: Piece) -> &mut Self::Output {
        self.bboards.index_mut(index.idx as usize)
    }
}

impl Default for Position {
    fn default() -> Self {
        let pawns = 0b11111111;
        let rooks = 0b10000001;
        let knights = 0b01000010;
        let bishops = 0b00100100;

        PositionBuilder::new()
            .with_next_turn(Colour::White)
            .with_piece_board(mkp!(White, Pawn), BitBoard::new(pawns << 8))
            .with_piece_board(mkp!(Black, Pawn), BitBoard::new(pawns << 48))
            .with_piece_board(mkp!(White, Rook), BitBoard::new(rooks))
            .with_piece_board(mkp!(Black, Rook), BitBoard::new(rooks << 56))
            .with_piece_board(mkp!(White, Knight), BitBoard::new(knights))
            .with_piece_board(mkp!(Black, Knight), BitBoard::new(knights << 56))
            .with_piece_board(mkp!(White, Bishop), BitBoard::new(bishops))
            .with_piece_board(mkp!(Black, Bishop), BitBoard::new(bishops << 56))
            .with_piece_board(mkp!(White, King), loc!(e 1).to_bitboard())
            .with_piece_board(mkp!(Black, King), loc!(e 8).to_bitboard())
            .with_piece_board(mkp!(White, Queen), loc!(d 1).to_bitboard())
            .with_piece_board(mkp!(Black, Queen), loc!(d 8).to_bitboard())
            .with_castling_rights(CastlingRights::default())
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::{locus::loc, Position};
    use crate::{
        mmove::{CastlingMoveType, MoveBuilder},
        piece::{mkp, Colour, Piece, PieceKind},
    };

    #[test]
    fn undo_move() {
        let mut pos = Position::default();

        let token = pos.make_move(
            MoveBuilder::new(mkp!(White, Pawn), loc!(e 2))
                .with_dst(loc!(e 3))
                .build(),
        );

        assert_ne!(pos, Position::default());

        if pos.to_play == Colour::White {
            return;
        }

        pos.undo_move(token);

        assert_eq!(pos, Position::default());
    }

    #[test]
    fn set_en_passant() {
        let mut pos = Position::default();
        let mmove = MoveBuilder::new(mkp!(White, Pawn), loc!(e 2))
            .with_dst(loc!(e 4))
            .is_double_pawn_push()
            .build();

        pos.make_move(mmove).consume();

        assert_eq!(pos.en_passant.unwrap(), loc!(e 3));

        let token = pos.make_move(
            MoveBuilder::new(mkp!(Black, Knight), loc!(b 8))
                .with_dst(loc!(c 6))
                .build(),
        );

        assert!(pos.en_passant.is_none());

        pos.undo_move(token);

        assert_eq!(pos.en_passant.unwrap(), loc!(e 3));
    }

    #[test]
    fn en_passant_capture_undo() {
        let mut pos =
            Position::from_fen("rnbqkb1r/pppppppp/5n2/P7/8/8/1PPPPPPP/RNBQKBNR b KQkq - 0 2")
                .unwrap();

        pos.make_move(
            MoveBuilder::new(mkp!(Black, Pawn), loc!(b 7))
                .with_dst(loc!(b 5))
                .is_double_pawn_push()
                .build(),
        )
        .consume();

        let p2 = pos.clone();

        let token = pos.make_move(
            MoveBuilder::new(mkp!(White, Pawn), loc!(a 5))
                .with_dst(loc!(b 6))
                .is_en_passant_capture()
                .build(),
        );

        assert!(!pos[mkp!(Black, Pawn)].has_piece_at(loc!(b 5)));

        pos.undo_move(token);

        assert_eq!(pos, p2);
    }

    #[test]
    fn make_move_castle_king_side() {
        let mut pos =
            Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQK2R w KQkq - 0 1").unwrap();

        let p2 = pos.clone();

        let token = pos.make_move(
            MoveBuilder::new(mkp!(White, King), loc!(e 1))
                .with_dst(loc!(g 1))
                .is_castling_move(CastlingMoveType::Kingside)
                .build(),
        );

        assert!(pos[Piece::new(PieceKind::Rook, Colour::White)].has_piece_at(loc!(f 1)));
        assert!(!pos[Piece::new(PieceKind::Rook, Colour::White)].has_piece_at(loc!(h 1)));

        pos.undo_move(token);

        assert_eq!(pos, p2);

        let mut pos =
            Position::from_fen("rnbqk2r/pppppppp/8/8/8/8/PPPPPPPP/RNBQK2R b KQkq - 0 1").unwrap();

        let p2 = pos.clone();

        let token = pos.make_move(
            MoveBuilder::new(mkp!(Black, King), loc!(e 8))
                .with_dst(loc!(g 8))
                .is_castling_move(CastlingMoveType::Kingside)
                .build(),
        );

        assert!(pos[Piece::new(PieceKind::Rook, Colour::Black)].has_piece_at(loc!(f 8)));
        assert!(!pos[Piece::new(PieceKind::Rook, Colour::Black)].has_piece_at(loc!(h 8)));

        pos.undo_move(token);

        assert_eq!(pos, p2);
    }

    #[test]
    fn castling_rights_clear() {
        let mut pos = Position::from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/P1N2Q1p/1PPBBPPP/R3K2R b KQkq - 0 1",
        )
        .unwrap();

        pos.make_move(
            MoveBuilder::new(mkp!(Black, Rook), loc!(a 8))
                .with_dst(loc!(b 8))
                .build(),
        )
        .consume();

        assert!(pos.castling_rights[Colour::Black].king_side());
        assert!(!pos.castling_rights[Colour::Black].queen_side());

        let mut pos = Position::from_fen(
            "r3k2r/p1ppqpb1/bn2pnN1/3P4/1p2P3/2N2Q2/PPPBBPpP/R3K2R w KQkq - 0 2",
        )
        .unwrap();

        pos.make_move(
            MoveBuilder::new(mkp!(White, Knight), loc!(g 6))
                .with_dst(loc!(h 8))
                .with_capture(mkp!(Black, Rook))
                .build(),
        )
        .consume();

        assert!(!pos.castling_rights[Colour::Black].king_side());
        assert!(pos.castling_rights[Colour::Black].queen_side());

        let mut pos =
            Position::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/P7/1PP1NnPP/RNBQK2R b KQ - 0 8")
                .unwrap();

        pos.make_move(
            MoveBuilder::new(mkp!(Black, Knight), loc!(f 2))
                .with_dst(loc!(h 1))
                .with_capture(mkp!(White, Rook))
                .build(),
        )
        .consume();

        assert!(!pos.castling_rights[Colour::White].king_side());
        assert!(pos.castling_rights[Colour::White].queen_side());

        // Ensure that capturing a promoted rook doesn't cancel castling rights.
        let mut pos =
            Position::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBPPP3/q4N2/P5PP/r2Q1RK1 w kq - 0 2")
                .unwrap();

        pos.make_move(
            MoveBuilder::new(mkp!(White, Queen), loc!(d 1))
                .with_dst(loc!(a 1))
                .with_capture(mkp!(Black, Rook))
                .build(),
        )
        .consume();

        assert!(pos.castling_rights[Colour::Black].king_side());
        assert!(pos.castling_rights[Colour::Black].queen_side());
    }

    #[test]
    fn repeated() {
        let mut pos =
            Position::from_fen("rn2kbnr/p3pppp/8/8/8/8/3PPPPP/RN2KBNR w KQkq - 0 1").unwrap();

        pos.make_move(
            MoveBuilder::new(mkp!(White, Rook), loc!(a 1))
                .with_dst(loc!(a 4))
                .build(),
        )
        .consume();

        assert!(!pos.has_repeated());

        pos.make_move(
            MoveBuilder::new(mkp!(Black, Knight), loc!(g 8))
                .with_dst(loc!(f 6))
                .build(),
        )
        .consume();

        assert!(!pos.has_repeated());

        pos.make_move(
            MoveBuilder::new(mkp!(White, Knight), loc!(b 1))
                .with_dst(loc!(c 3))
                .build(),
        )
        .consume();

        assert!(!pos.has_repeated());

        pos.make_move(
            MoveBuilder::new(mkp!(Black, Knight), loc!(f 6))
                .with_dst(loc!(g 8))
                .build(),
        )
        .consume();

        assert!(!pos.has_repeated());

        pos.make_move(
            MoveBuilder::new(mkp!(White, Knight), loc!(c 3))
                .with_dst(loc!(b 1))
                .build(),
        )
        .consume();

        assert!(!pos.has_repeated());

        pos.make_move(
            MoveBuilder::new(mkp!(Black, Knight), loc!(g 8))
                .with_dst(loc!(f 6))
                .build(),
        )
        .consume();

        assert!(!pos.has_repeated());

        pos.make_move(
            MoveBuilder::new(mkp!(White, Knight), loc!(b 1))
                .with_dst(loc!(c 3))
                .build(),
        )
        .consume();

        assert!(!pos.has_repeated());

        pos.make_move(
            MoveBuilder::new(mkp!(Black, Knight), loc!(f 6))
                .with_dst(loc!(g 8))
                .build(),
        )
        .consume();

        assert!(!pos.has_repeated());

        pos.make_move(
            MoveBuilder::new(mkp!(White, Knight), loc!(c 3))
                .with_dst(loc!(b 1))
                .build(),
        )
        .consume();

        assert!(pos.has_repeated());
    }
}
