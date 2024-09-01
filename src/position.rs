use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};

use arrayvec::ArrayVec;
use bitboard::BitBoard;
use builder::PositionBuilder;
use locus::{loc, File, Locus, Rank};
use strum::{EnumCount, IntoEnumIterator};

use crate::{
    mmove::Move,
    piece::{mkp, Colour, Piece, PieceKind, PieceKindIter},
};

pub mod bitboard;
pub mod builder;
pub mod fen;
pub mod locus;
pub mod movegen;

#[must_use = "Moves must either be undone, or made permanent"]
pub struct UndoToken;

impl UndoToken {
    pub fn consume(self) {}
}

#[derive(Clone, PartialEq)]
struct UndoMove {
    mmove: Move,
    ep_state: Option<Locus>,
    ep_capture: Option<Locus>,
}

#[derive(Clone, PartialEq)]
pub struct Position {
    bboards: [BitBoard; PieceKind::COUNT * 2],
    to_play: Colour,
    en_passant: Option<Locus>,
    move_stack: ArrayVec<UndoMove, 256>,
}

impl Position {
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

    pub fn all_pieces_for_colour(&self, colour: Colour) -> BitBoard {
        let mut b = BitBoard::empty();

        for kind in PieceKind::iter() {
            b = b | self[Piece::new(kind, colour)];
        }

        b
    }

    pub fn blockers(&self) -> BitBoard {
        let mut b = BitBoard::empty();

        for bb in self.bboards.iter() {
            b = b.or(*bb);
        }

        b
    }

    fn clr_piece_at(&mut self, p: Piece, loc: Locus) {
        self[p] = self[p].clear_piece_at(loc);
    }

    fn set_piece_at(&mut self, p: Piece, loc: Locus) {
        self[p] = self[p].set_piece_at(loc);
    }

    pub fn make_move(&mut self, mmove: Move) -> UndoToken {
        let mut undo = UndoMove {
            mmove,
            ep_state: self.en_passant,
            ep_capture: None,
        };

        self[mmove.piece] = self[mmove.piece]
            .clear_piece_at(mmove.src)
            .set_piece_at(mmove.dst);

        if let Some(fallen) = mmove.capture {
            self.clr_piece_at(fallen, mmove.dst);
        }

        if let Some(promotion) = mmove.promote {
            self.clr_piece_at(mmove.piece, mmove.dst);
            self.set_piece_at(promotion, mmove.dst);
        }

        if let Some(ep_loc) = self.en_passant {
            if mmove.dst == ep_loc && mmove.piece.kind() == PieceKind::Pawn {
                let captured_pawn_colour = self.to_play.next();
                let captured_piece = Piece::new(PieceKind::Pawn, captured_pawn_colour);
                let pawn_loc = if captured_pawn_colour == Colour::White {
                    ep_loc.north().unwrap()
                } else {
                    ep_loc.south().unwrap()
                };

                self.clr_piece_at(captured_piece, pawn_loc);
                undo.ep_capture = Some(pawn_loc);
            }
        }

        if mmove.set_ep {
            let (r, _) = mmove.src.to_rank_file();

            self.en_passant = Some(if r == Rank::Two {
                mmove.src.north().unwrap()
            } else {
                mmove.src.south().unwrap()
            });
        } else {
            self.en_passant = None;
        }

        self.to_play = self.to_play.next();
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

        if let Some(promotion) = mmove.promote {
            self.set_piece_at(mmove.piece, mmove.dst);
            self.clr_piece_at(promotion, mmove.dst);
        }

        if let Some(fallen) = mmove.capture {
            self.set_piece_at(fallen, mmove.dst);
        }

        if let Some(loc) = undo.ep_capture {
            self.set_piece_at(
                Piece::new(PieceKind::Pawn, mmove.piece.colour().next()),
                loc,
            )
        }

        self[mmove.piece] = self[mmove.piece]
            .set_piece_at(mmove.src)
            .clear_piece_at(mmove.dst);

        self.to_play = self.to_play.next();
        self.en_passant = undo.ep_state;
    }

    pub fn empty() -> Self {
        Self {
            bboards: [BitBoard::empty(); PieceKind::COUNT * 2],
            to_play: Colour::White,
            en_passant: None,
            move_stack: ArrayVec::new(),
        }
    }

    pub fn iter_opponent_bbds(&self) -> OpponentBbIter {
        OpponentBbIter {
            pos: &self,
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
            write!(f, "{rank:?} ")?;
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
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::{locus::loc, Position};
    use crate::{
        mmove::MoveBuilder,
        piece::{mkp, Colour},
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
            .sets_ep()
            .build();

        pos.make_move(mmove).consume();

        assert_eq!(pos.en_passant.unwrap(), loc!(e 3));

        let token = pos.make_move(
            MoveBuilder::new(mkp!(Black, Knight), loc!(b 8))
                .with_dst(loc!(c 6))
                .build(),
        );

        assert!(matches!(pos.en_passant, None));

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
                .sets_ep()
                .build(),
        )
        .consume();

        let p2 = pos.clone();

        let token = pos.make_move(
            MoveBuilder::new(mkp!(White, Pawn), loc!(a 5))
                .with_dst(loc!(b 6))
                .build(),
        );

        assert!(!pos[mkp!(Black, Pawn)].has_piece_at(loc!(b 5)));

        pos.undo_move(token);

        assert_eq!(pos, p2);
    }
}
